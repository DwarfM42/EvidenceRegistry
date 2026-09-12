use super::*;
use ai_agent_evidence_binder::{
    bridge::{run_to_admission, BridgeError, OutputCapturePlan},
    ledger::{CommandLimits, DispatchIntent, Event, Observation, Publication, PublicationKind},
    Submission,
};
use sha2::{Digest, Sha256};
use std::sync::atomic::AtomicBool;

pub(super) fn run(mut args: Args) -> Result<(i32, Value)> {
    let root = existing(Path::new(&args.take("--workspace")?))?;
    let attempt = args.take("--attempt")?;
    if attempt.is_empty()
        || attempt.len() > 64
        || !attempt
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
    {
        return Err("portable_attempt_id_required".into());
    }
    let capture_bytes = exact_id(&args.take("--capture-attempt")?)?;
    let capture_attempt = checked(FreezeAttemptId::try_from(capture_bytes.as_slice()))?;
    let output = fresh_path(Path::new(&args.take("--output")?))?;
    // This small consumer deliberately restricts capture to one fresh direct child.
    if output.parent().map(key) != Some(key(&existing(&root.join("outputs"))?)) {
        return Err("output_must_be_fresh_child_of_workspace_outputs".into());
    }
    let input = existing(Path::new(&args.take("--approve-input-root")?))?;
    let permissions = args.take("--tool-permissions")?;
    if permissions.is_empty()
        || permissions.len() > 4096
        || permissions.chars().any(char::is_control)
    {
        return Err("explicit_bounded_tool_permission_note_required".into());
    }
    let limits = CommandLimits {
        runtime_ms: args.number("--runtime-ms")?,
        pipe_drain_ms: args.number("--pipe-drain-ms")?,
        stdout_bytes: args.number("--stdout-bytes")?,
        stderr_bytes: args.number("--stderr-bytes")?,
        output_bytes: args.number("--output-bytes")?,
        output_files: checked(u32::try_from(args.number("--output-files")?))?,
        open_descriptors: checked(u32::try_from(args.number("--open-descriptors")?))?,
    };
    let max = CommandLimits::default();
    if limits.open_descriptors != 64
        || limits.output_files < 4
        || [
            (limits.runtime_ms, max.runtime_ms),
            (limits.pipe_drain_ms, max.pipe_drain_ms),
            (limits.stdout_bytes, max.stdout_bytes),
            (limits.stderr_bytes, max.stderr_bytes),
            (limits.output_bytes, max.output_bytes),
            (u64::from(limits.output_files), u64::from(max.output_files)),
        ]
        .iter()
        .any(|(n, m)| *n == 0 || n > m)
    {
        return Err("unsupported_capture_or_process_limits".into());
    }
    let predecessor = args.flags.remove("--predecessor");
    let command = std::mem::take(&mut args.argv);
    args.done()?;
    if command.is_empty()
        || command.len() > 64
        || command
            .iter()
            .any(|s| s.is_empty() || s.len() > 4096 || s.chars().any(char::is_control))
        || command.iter().map(String::len).sum::<usize>() > 16384
    {
        return Err("bounded_literal_argv_after_separator_required".into());
    }
    if !Path::new(&command[0]).is_absolute() {
        return Err("absolute_executable_required".into());
    }
    let binding = read_binding(&root)?;
    if key(&input) != key(Path::new(&binding.source_root)) || overlaps(&input, &root) {
        return Err("approved_input_locator_mismatch".into());
    }
    let mut store = checked(AuthoritativeRegistryStore::open_selected_profile(
        root.join("store"),
    ))?;
    let q = resolve(&store, &binding.request)?;
    let request = checked(store.validate_selected_review_request(q.clone()))?;
    let capture_policy = resolve(&store, &binding.capture_policy)?;
    if request_binding(request.freeze_authority().committed_event_reference()) != binding.target
        || rid(&capture_policy) != request.freeze_authority().policy_record_id()
    {
        return Err("target_or_capture_policy_binding_mismatch".into());
    }
    // Fresh Core attempt ID, including prior failed STARTs, not only completed Freezes.
    for r in store
        .retained_journal()
        .references()
        .filter(|r| r.event_type_id().value() == 100)
    {
        let bytes = store.resolve(rid(&r)).ok_or("start_record_missing")?;
        let start = checked(FreezeAttemptStartRecord::decode_authoritative(bytes))?;
        if start.input().freeze_attempt_id == capture_attempt {
            return Err("capture_attempt_already_retained".into());
        }
    }
    // Never recreate a deleted ledger. Init owns creation; run only appends.
    if !root.join("ledger").is_file() {
        return Err("ledger_missing_no_recreation".into());
    }
    let mut writer = checked(LedgerWriter::open(root.join("ledger")))?;
    let attempts = writer.report().attempts();
    if attempts.iter().any(|a| a.attempt_id == attempt) {
        return Err("attempt_already_retained".into());
    }
    if let Some(last) = attempts.last() {
        if predecessor.as_deref() != Some(last.attempt_id.as_str())
            || last.request != binding.request
        {
            return Err("explicit_immediate_predecessor_for_same_request_required".into());
        }
    } else if predecessor.is_some() {
        return Err("predecessor_not_retained".into());
    }
    // These local notes are not authority or sandbox enforcement. An exclusive
    // create reserves the attempt even if the process crashes before ledger intent.
    let identity = executable_identity()?;
    let capture_hex: String = capture_bytes.iter().map(|b| format!("{b:02x}")).collect();
    write_new(
        &root
            .join("approvals")
            .join(format!("capture.{capture_hex}.json")),
        &json!({"authoritative":false,"reserved_for_attempt":attempt}),
    )?;
    write_new(
        &root.join("approvals").join(format!("{attempt}.json")),
        &json!({
            "schema":1,"authoritative":false,"request":binding.request,"capture_attempt":capture_bytes,
            "approved_input_root":input,"approved_output_root":output,"tool_permissions":permissions,
            "permissions_enforced_by_binder":false,"stdin":"null","environment":"inherited",
            "limits":limits,"predecessor":predecessor,"binder_identity":identity
        }),
    )?;
    checked(fs::create_dir(&output))?;
    let declaration = DispatchIntent {
        approved_output_root: output.to_str().ok_or("path_encoding")?.into(),
        mandatory_files: vec![
            "binder-stderr.bin".into(),
            "binder-stdout.bin".into(),
            "review.txt".into(),
            "submission.json".into(),
        ],
        command,
        binder_identity: identity,
        limits,
        predecessor,
    };
    let outcome = run_to_admission(
        &mut store,
        &mut writer,
        q,
        &attempt,
        declaration,
        OutputCapturePlan {
            freeze_attempt_id: capture_attempt,
            policy_record_id: rid(&capture_policy),
        },
        &AtomicBool::new(false),
    );
    let observations: Vec<_> = writer
        .report()
        .frames
        .iter()
        .filter(|f| f.attempt_id == attempt)
        .filter_map(|f| match &f.event {
            Event::Observation(observation) => Some(observation.clone()),
            _ => None,
        })
        .collect();
    let launch = if observations
        .iter()
        .any(|o| matches!(o, Observation::Spawned { .. }))
    {
        "observed_spawn"
    } else if observations
        .iter()
        .any(|o| matches!(o, Observation::SpawnFailed { .. }))
    {
        "observed_spawn_failure"
    } else {
        "not_observed_or_uncertain"
    };
    let exit_code = observations.iter().find_map(|o| match o {
        Observation::Exited { code } => Some(*code),
        _ => None,
    });
    let (code, disposition, result, output_freeze, admission, failure) = match outcome {
        Ok(o) => {
            let (code, disposition, admission) = match o.admission {
                AuthoritativeReviewAdmissionRuntimeOutcome::Published(p) => {
                    let accepted = p.journal_reference().event_type_id().value() == 302;
                    (
                        if accepted { 0 } else { 10 },
                        if accepted { "accepted" } else { "rejected" },
                        json!({"reference":request_binding(p.journal_reference()),"durability":format!("{:?}",p.durability())}),
                    )
                }
                AuthoritativeReviewAdmissionRuntimeOutcome::PublishedReceiptUncertain(_) => {
                    (13, "publication_uncertain", Value::Null)
                }
                AuthoritativeReviewAdmissionRuntimeOutcome::PreTerminal(_) => {
                    (12, "preterminal", Value::Null)
                }
            };
            (
                code,
                disposition,
                Some(request_binding(o.result.result_event_reference())),
                Some(request_binding(o.output_freeze.committed_event_reference())),
                admission,
                Value::Null,
            )
        }
        Err(e) => {
            let (code, stage) = match e {
                BridgeError::Process(_) => (11, "process"),
                BridgeError::FailureRecording { .. } | BridgeError::Ledger(_) => {
                    (13, "ledger_or_failure_recording")
                }
                BridgeError::Admission(_) => (13, "admission_publication"),
                BridgeError::Submission(_) => (12, "submission"),
                BridgeError::Preparation(_)
                | BridgeError::Capture(_)
                | BridgeError::Readback(_)
                | BridgeError::CaptureSet(_) => (12, "capture"),
                BridgeError::Result(_) => (12, "result"),
                BridgeError::Section82(_) => (12, "section82"),
                _ => (12, "bridge_preflight"),
            };
            // Never echo Debug(e): command/process errors might carry private text.
            let returned_reference = |operation| {
                writer
                    .report()
                    .frames
                    .iter()
                    .filter(|f| f.attempt_id == attempt)
                    .find_map(|f| match &f.event {
                        Event::Publication(Publication::Returned {
                            operation: op,
                            references,
                            ..
                        }) if *op == operation => references.first().and_then(|r| {
                            let binding = RequestBinding {
                                registry_id: r.registry_id.clone(),
                                entry_index: r.entry_index,
                                entry_hash: r.entry_hash.clone(),
                                event_type_id: r.event_type_id,
                                event_record_id: r.event_record_id.clone(),
                            };
                            resolve(&store, &binding).ok().map(|r| request_binding(&r))
                        }),
                        _ => None,
                    })
            };
            (
                code,
                "not_completed",
                returned_reference(PublicationKind::ReviewResult),
                returned_reference(PublicationKind::OutputCapture),
                Value::Null,
                json!(stage),
            )
        }
    };
    let result_recorded = if result.is_some() {
        json!(true)
    } else if writer.report().frames.iter().any(|f| {
        f.attempt_id == attempt
            && matches!(
                f.event,
                Event::Publication(Publication::Intent {
                    operation: PublicationKind::ReviewResult
                })
            )
    }) {
        Value::Null // mutation attempted; absence of a returned receipt is not absence of R
    } else {
        json!(false)
    };
    drop(writer);
    drop(store);
    Ok((
        code,
        json!({"operation":"run","attempt":attempt,"request":binding.request,
        "launch":launch,"process_exit":exit_code,"result_recorded":result_recorded,"result":result,
        "output_freeze":output_freeze,"disposition":disposition,"admission":admission,
        "failure_stage":failure,"inspection_required":true,"argv_disclosed":false}),
    ))
}
fn executable_identity() -> Result<String> {
    let mut file = checked(fs::File::open(checked(std::env::current_exe())?))?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let n = checked(file.read(&mut buffer))?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("companion-file-sha256:{:x}", hash.finalize()))
}
/// Controlled native fake writes untrusted files only. Never publishes Records.
pub(super) fn fake(mut args: Args) -> Result<(i32, Value)> {
    let root = existing(Path::new(&args.take("--workspace")?))?;
    let mode = args.take("--mode")?;
    args.done()?;
    if !["clean", "rejected", "malformed", "nonzero"].contains(&mode.as_str()) {
        return Err("unknown_fake_mode".into());
    }
    let binding = read_binding(&root)?;
    if mode == "nonzero" {
        return Ok((7, json!({"fake":true,"mode":mode})));
    }
    let mut review = checked(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open("review.txt"),
    )?;
    checked(review.write_all(
        b"DETERMINISTIC FAKE, NOT AI: no semantic inspection performed. Claims only.\n",
    ))?;
    if mode == "malformed" {
        let mut f = checked(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open("submission.json"),
        )?;
        checked(f.write_all(b"{malformed"))?;
    } else {
        write_new(
            Path::new("submission.json"),
            &Submission {
                schema: 1,
                request: binding.request,
                method_status: if mode == "clean" { 1 } else { 2 },
                finding_state: 1,
                reason_codes: vec![],
                findings: vec![],
                reviewer_metadata: Some("deterministic fake; not AI".into()),
                artifacts: vec!["review.txt".into()],
            },
        )?;
    }
    eprintln!("deterministic fake stderr; no internal delegation observed");
    Ok((0, json!({"fake":true,"mode":mode})))
}
