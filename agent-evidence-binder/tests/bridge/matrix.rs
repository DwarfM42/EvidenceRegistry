//! Public-seam negative/positive tracers. Native fixture children are not AI.
use super::*;
use ai_agent_evidence_binder::{
    bridge::{BridgeError, BridgeOutcome},
    inspection::inspect,
    ledger::{FaultKind, Observation},
    SubmissionError,
};

#[test]
fn failed_then_explicit_success_retains_both_attempts_on_cold_inspection() {
    let mut f = Fixture::new();
    fs::write(
        f.base.join("out/fixture-input.json"),
        b"{first malformed attempt",
    )
    .unwrap();
    let declaration = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let failed = execute(&mut f, &mut writer, "first", declaration);
    assert!(
        matches!(
            failed,
            Err(BridgeError::Submission(SubmissionError::Grammar))
        ),
        "{failed:?}"
    );
    assert_no_result_or_admission(&f);
    let first_output = f
        .store
        .retained_journal()
        .references()
        .filter(|r| r.event_type_id().value() == 101)
        .last()
        .unwrap();
    let first_bytes = f
        .store
        .read_selected_embedded_payload(first_output.clone())
        .unwrap();
    // Preserve the original output directory. A fresh root and explicit predecessor
    // are required; no deletion, hidden re-run or reconstructed live witness.
    let second_root = f.base.join("second-out");
    fs::create_dir(&second_root).unwrap();
    let submission = Submission {
        schema: 1,
        request: binding(f.request.request_event_reference()),
        method_status: 1,
        finding_state: 1,
        reason_codes: vec![],
        findings: vec![],
        reviewer_metadata: Some("explicit second fake attempt; not AI".into()),
        artifacts: vec!["review.txt".into()],
    };
    fs::write(
        second_root.join("fixture-input.json"),
        serde_json::to_vec(&submission).unwrap(),
    )
    .unwrap();
    let mut declaration = f.intent("fake_reviewer");
    declaration.approved_output_root = second_root.to_str().unwrap().into();
    declaration.predecessor = Some("first".into());
    f.plan.freeze_attempt_id = FreezeAttemptId::try_from([64; 32].as_slice()).unwrap();
    let outcome = execute(&mut f, &mut writer, "second", declaration).unwrap();
    let AuthoritativeReviewAdmissionRuntimeOutcome::Published(p) = outcome.admission else {
        panic!("expected actual publication")
    };
    assert_eq!(p.journal_reference().event_type_id().value(), 302);
    assert_eq!(
        f.store
            .read_selected_embedded_payload(first_output)
            .unwrap()
            .artifact_bytes(),
        first_bytes.artifact_bytes()
    );
    assert_eq!(writer.report().attempts().len(), 2);
    assert_eq!(
        writer
            .report()
            .frames
            .iter()
            .filter(|frame| matches!(frame.event, Event::Observation(Observation::Spawned { .. })))
            .count(),
        2
    );
    drop(writer);
    fs::write(f.base.join("expected-inspection.json"), serde_json::to_vec(&serde_json::json!({"attempts": 2, "observed_failure_attempts": 1, "store_results": 1, "store_admissions_accepted": 1, "independent_reviews_established": 0})).unwrap()).unwrap();
    cold_child(&f.base);
    let report = inspect(f.base.join("store"), f.base.join("ledger")).unwrap();
    assert_eq!(report.attempts[0].status, "ObservedFailure");
    assert_eq!(report.attempts[1].predecessor.as_deref(), Some("first"));
    assert_eq!(report.attempts[1].status, "HistoricalReferencesValidated");
}

#[test]
fn mandatory_omitted_from_agent_list_is_independently_captured() {
    let mut f = Fixture::new();
    f.input(1);
    edit_input(&f, |v| {
        v["artifacts"] = serde_json::json!([]);
    });
    fs::create_dir(f.base.join("out/nested")).unwrap();
    fs::write(
        f.base.join("out/nested/unlisted.txt"),
        b"independently enumerated",
    )
    .unwrap();
    let mut declaration = f.intent("fake_reviewer");
    declaration
        .mandatory_files
        .push("nested/unlisted.txt".into());
    declaration.mandatory_files.sort();
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let outcome = execute(&mut f, &mut writer, "empty-candidates", declaration).unwrap();
    let snapshot = f
        .store
        .read_selected_embedded_payload(outcome.output_freeze.committed_event_reference().clone())
        .unwrap();
    assert_eq!(snapshot.manifest().input().artifact_count, 6);
    assert!(snapshot
        .artifact_bytes()
        .iter()
        .any(|b| b == b"independently enumerated"));
    assert!(snapshot
        .artifact_bytes()
        .iter()
        .any(|b| b.starts_with(b"CLEAN is only")));
    let AuthoritativeReviewAdmissionRuntimeOutcome::Published(p) = outcome.admission else {
        panic!("expected publication")
    };
    assert_eq!(p.journal_reference().event_type_id().value(), 302);
    drop(writer);
    cold_child(&f.base);
}

#[test]
fn submission_from_another_managed_execution_and_request_is_not_rebound() {
    let mut f = Fixture::new();
    f.input(1);
    let declaration = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let first = execute(&mut f, &mut writer, "original", declaration).unwrap();
    let bytes = fs::read(f.base.join("out/submission.json")).unwrap();
    let old_request = f.request.request_event_reference().clone();
    f.request = f
        .store
        .record_selected_review_request(SelectedReviewRequestInput {
            freeze_authority: f.request.freeze_authority().clone(),
            review_policy_record_id: record_id(f.request.request().policy_authority_ref()),
            review_role_id: 1,
        })
        .unwrap();
    assert_ne!(f.request.request_event_reference(), &old_request);
    let second = f.base.join("cross-output");
    fs::create_dir(&second).unwrap();
    fs::write(second.join("fixture-input.json"), &bytes).unwrap();
    let mut declaration = f.intent("fake_reviewer");
    declaration.approved_output_root = second.to_str().unwrap().into();
    f.plan.freeze_attempt_id = FreezeAttemptId::try_from([65; 32].as_slice()).unwrap();
    let result = execute(&mut f, &mut writer, "cross-request", declaration);
    assert!(
        matches!(
            result,
            Err(BridgeError::Submission(SubmissionError::RequestMismatch))
        ),
        "{result:?}"
    );
    assert_captured_bytes(&f, &bytes);
    assert_eq!(
        f.store
            .retained_journal()
            .references()
            .filter(|r| r.event_type_id().value() == 301)
            .collect::<Vec<_>>(),
        vec![first.result.result_event_reference().clone()]
    );
    assert_fault(&writer, FaultKind::InvalidSubmission, "invalid_submission");
    drop(writer);
    cold_child(&f.base);
}

#[test]
fn duplicate_attempt_delivery_never_spawns_or_publishes_twice() {
    let mut f = Fixture::new();
    f.input(1);
    let declaration = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    execute(&mut f, &mut writer, "same", declaration.clone()).unwrap();
    let frame_count = writer.report().frames.len();
    let head = f.store.retained_journal().current_head_reference();
    let result = execute(&mut f, &mut writer, "same", declaration);
    assert!(
        matches!(
            result,
            Err(BridgeError::Process(
                ai_agent_evidence_binder::process::ProcessError::Ledger(_)
            ))
        ),
        "{result:?}"
    );
    assert_eq!(writer.report().frames.len(), frame_count);
    assert_eq!(f.store.retained_journal().current_head_reference(), head);
    drop(writer);
    cold_child(&f.base);
}

#[test]
fn nonexistent_finding_stops_before_result_without_losing_output() {
    let mut f = Fixture::new();
    f.input(1);
    let bytes = edit_input(&f, |v| {
        v["finding_state"] = 2.into();
        v["findings"] = serde_json::json!(["ff".repeat(32)]);
    });
    let declaration = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let result = execute(&mut f, &mut writer, "missing-finding", declaration);
    assert!(matches!(result, Err(BridgeError::Result(_))), "{result:?}");
    assert_no_result_or_admission(&f);
    assert_captured_bytes(&f, &bytes);
    assert!(writer.report().frames.iter().any(|frame| matches!(&frame.event, Event::Publication(Publication::Uncertain { operation: PublicationKind::ReviewResult, code }) if code == "result_failed")));
    drop(writer);
    cold_child(&f.base);
    let report = inspect(f.base.join("store"), f.base.join("ledger")).unwrap();
    assert_eq!(report.attempts[0].status, "Unresolved");
    assert_eq!(report.counts.store_admissions_rejected, 0);
}

#[test]
fn invalid_output_policy_records_uncertain_preparation_without_retry() {
    let mut f = Fixture::new();
    f.input(1);
    f.plan.policy_record_id = RecordId::try_from([255; 32].as_slice()).unwrap();
    let declaration = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let result = execute(&mut f, &mut writer, "bad-output-policy", declaration);
    assert!(
        matches!(
            result,
            Err(BridgeError::Preparation(
                SelectedEmbeddedFreezePreparationError::PolicyUnavailable
            ))
        ),
        "{result:?}"
    );
    assert_eq!(
        f.store.retained_journal().current_head_reference(),
        *f.request.request_event_reference()
    );
    assert_no_result_or_admission(&f);
    assert!(writer.report().frames.iter().any(|frame| matches!(&frame.event, Event::Publication(Publication::Uncertain { operation: PublicationKind::OutputPreparation, code }) if code == "output_preparation_failed")));
    assert!(f.base.join("out/submission.json").exists());
    drop(writer);
    cold_child(&f.base);
}

#[test]
fn output_hardlink_is_rejected_by_store_capture_not_followed() {
    let mut f = Fixture::new();
    f.input(1);
    fs::write(f.base.join("outside.txt"), b"outside linked evidence").unwrap();
    fs::hard_link(f.base.join("outside.txt"), f.base.join("out/linked.txt")).unwrap();
    capture_source_invalid(&mut f, "hardlink");
}
#[cfg(unix)]
#[test]
fn output_symlink_is_rejected_by_store_capture_not_followed() {
    let mut f = Fixture::new();
    f.input(1);
    fs::write(f.base.join("outside.txt"), b"outside linked evidence").unwrap();
    std::os::unix::fs::symlink(f.base.join("outside.txt"), f.base.join("out/linked.txt")).unwrap();
    capture_source_invalid(&mut f, "symlink");
}
#[cfg(windows)]
#[test]
fn output_junction_is_rejected_by_store_capture_not_followed() {
    let mut f = Fixture::new();
    f.input(1);
    fs::create_dir(f.base.join("outside-directory")).unwrap();
    fs::write(
        f.base.join("outside-directory/secret.txt"),
        b"outside junction evidence",
    )
    .unwrap();
    let output = std::process::Command::new("cmd.exe")
        .args(["/D", "/C", "mklink", "/J"])
        .arg(f.base.join("out/linked-directory"))
        .arg(f.base.join("outside-directory"))
        .output()
        .unwrap();
    fs::write(f.base.join("junction-stdout.bin"), &output.stdout).unwrap();
    fs::write(f.base.join("junction-stderr.bin"), &output.stderr).unwrap();
    assert!(output.status.success(), "junction setup failed: {output:?}");
    capture_source_invalid(&mut f, "junction");
}
fn capture_source_invalid(f: &mut Fixture, attempt: &str) {
    let declaration = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let result = execute(f, &mut writer, attempt, declaration);
    assert!(
        matches!(
            result,
            Err(BridgeError::Preparation(
                SelectedEmbeddedFreezePreparationError::SourceInvalid
            ))
        ),
        "{result:?}"
    );
    assert_no_result_or_admission(f);
    assert_eq!(
        f.store.retained_journal().current_head_reference(),
        *f.request.request_event_reference()
    );
    assert!(writer.report().frames.iter().any(|frame| matches!(
        &frame.event,
        Event::Publication(Publication::Uncertain {
            operation: PublicationKind::OutputPreparation,
            ..
        })
    )));
    assert!(f.base.join("out/submission.json").exists());
    drop(writer);
    cold_child(&f.base);
}
#[test]
fn source_mutation_after_capture_does_not_rewrite_retained_evidence() {
    let mut f = Fixture::new();
    f.input(1);
    let declaration = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let outcome = execute(&mut f, &mut writer, "source-mutation", declaration).unwrap();
    let r = outcome.output_freeze.committed_event_reference().clone();
    let before = f.store.read_selected_embedded_payload(r.clone()).unwrap();
    fs::write(
        f.base.join("out/submission.json"),
        b"replaced source after capture",
    )
    .unwrap();
    fs::write(
        f.base.join("out/review.txt"),
        b"rewritten source after capture",
    )
    .unwrap();
    assert_eq!(
        f.store
            .read_selected_embedded_payload(r)
            .unwrap()
            .artifact_bytes(),
        before.artifact_bytes()
    );
    drop(writer);
    cold_child(&f.base);
}
#[test]
fn corrupt_target_payload_prevents_dispatch_before_intent() {
    let mut f = Fixture::new();
    f.input(1);
    let declaration = f.intent("fake_reviewer");
    let target_path = f
        .base
        .join("store/roots")
        .join(hex(f
            .request
            .freeze_authority()
            .freeze_attempt_id()
            .as_bytes()))
        .join("payload/target.txt");
    assert!(target_path.is_file());
    fs::write(target_path, b"intentional negative target corruption").unwrap();
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let result = execute(&mut f, &mut writer, "corrupt-target", declaration);
    assert!(matches!(result, Err(BridgeError::Request(_))), "{result:?}");
    assert!(writer.report().frames.is_empty());
    assert!(!f.base.join("out/binder-stdout.bin").exists());
    assert_no_result_or_admission(&f);
}
#[test]
fn retained_output_payload_corruption_is_reported_by_new_process_inspection() {
    let mut f = Fixture::new();
    f.input(1);
    let declaration = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let outcome = execute(&mut f, &mut writer, "retained-mutation", declaration).unwrap();
    let payload = f
        .base
        .join("store/roots")
        .join(hex(outcome.output_freeze.freeze_attempt_id().as_bytes()))
        .join("payload/submission.json");
    assert!(payload.is_file());
    fs::write(payload, b"intentional negative retained output corruption").unwrap();
    assert!(f
        .store
        .read_selected_embedded_payload(outcome.output_freeze.committed_event_reference().clone())
        .is_err());
    drop(writer);
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "matrix::corrupt_payload_inspector",
            "--nocapture",
        ])
        .current_dir(&f.base)
        .output()
        .unwrap();
    fs::write(f.base.join("corrupt-cold-stdout.bin"), &output.stdout).unwrap();
    fs::write(f.base.join("corrupt-cold-stderr.bin"), &output.stderr).unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("CORRUPT_PAYLOAD_NOT_ACCEPTED_AS_VALID")
    );
}
#[test]
#[ignore = "native cold negative-inspection subprocess entry"]
fn corrupt_payload_inspector() {
    let ledger_before = fs::read("ledger").unwrap();
    let report = inspect("store", "ledger").unwrap();
    assert_eq!(report.counts.attempts, 1);
    assert_ne!(report.store.status, "ValidatedCapturedView");
    assert_ne!(report.attempts[0].status, "HistoricalReferencesValidated");
    assert!(!report.attempts[0].live_witness);
    assert!(!report.attempts[0].historical_receipt_recovered);
    assert_eq!(fs::read("ledger").unwrap(), ledger_before);
    println!("{}", serde_json::to_string(&report).unwrap());
    println!("CORRUPT_PAYLOAD_NOT_ACCEPTED_AS_VALID");
}

#[test]
fn spawn_failure_preserves_attempt_without_store_publication() {
    process_failure("spawn");
}
#[test]
fn nonzero_exit_with_valid_submission_does_not_reach_capture() {
    process_failure("nonzero");
}
#[test]
fn timed_out_execution_does_not_reach_capture_or_retry() {
    process_failure("timeout");
}
#[test]
fn cancellation_before_spawn_does_not_reach_capture() {
    process_failure("cancel");
}
#[test]
fn bounded_pipe_failure_does_not_reach_capture() {
    process_failure("pipe");
}
fn process_failure(mode: &str) {
    let mut f = Fixture::new();
    f.input(1);
    fs::write(f.base.join("out/failure-mode.txt"), mode).unwrap();
    let mut declaration = f.intent("matrix::failing_reviewer");
    if mode == "spawn" {
        declaration.command = vec![f.base.join("absent-reviewer.exe").to_str().unwrap().into()];
    }
    if mode == "timeout" {
        declaration.limits.runtime_ms = 500;
    }
    if mode == "pipe" {
        declaration.limits.stdout_bytes = 1;
    }
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let result = run_to_admission(
        &mut f.store,
        &mut writer,
        f.request.request_event_reference().clone(),
        mode,
        declaration,
        f.plan.clone(),
        &AtomicBool::new(mode == "cancel"),
    );
    assert!(
        matches!(result, Err(BridgeError::Process(_))),
        "{mode}: {result:?}"
    );
    if mode == "nonzero" {
        assert!(
            matches!(
                result,
                Err(BridgeError::Process(
                    ai_agent_evidence_binder::process::ProcessError::Nonzero
                ))
            ),
            "{result:?}"
        );
        assert_eq!(
            fs::read(f.base.join("out/submission.json")).unwrap(),
            fs::read(f.base.join("out/fixture-input.json")).unwrap()
        );
    }
    assert_no_result_or_admission(&f);
    assert_eq!(
        f.store.retained_journal().current_head_reference(),
        *f.request.request_event_reference()
    );
    assert_eq!(writer.report().attempts().len(), 1);
    assert!(!writer
        .report()
        .frames
        .iter()
        .any(|frame| matches!(frame.event, Event::Publication(_))));
    let spawn_count = writer
        .report()
        .frames
        .iter()
        .filter(|frame| matches!(frame.event, Event::Observation(Observation::Spawned { .. })))
        .count();
    assert_eq!(
        spawn_count,
        usize::from(!matches!(mode, "spawn" | "cancel"))
    );
    drop(writer);
    fs::write(f.base.join("expected-inspection.json"), serde_json::to_vec(&serde_json::json!({"attempts":1,"observed_failure_attempts":1,"store_results":0,"store_admissions_accepted":0,"store_admissions_rejected":0})).unwrap()).unwrap();
    cold_child(&f.base);
}
#[test]
#[ignore = "native negative execution fixture subprocess entry"]
fn failing_reviewer() {
    let mode = fs::read_to_string("failure-mode.txt").unwrap();
    if mode == "timeout" {
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
    fake_reviewer();
    if mode == "nonzero" {
        std::process::exit(9);
    }
}

fn execute(
    f: &mut Fixture,
    writer: &mut LedgerWriter,
    attempt: &str,
    declaration: DispatchIntent,
) -> Result<BridgeOutcome, BridgeError> {
    run_to_admission(
        &mut f.store,
        writer,
        f.request.request_event_reference().clone(),
        attempt,
        declaration,
        f.plan.clone(),
        &AtomicBool::new(false),
    )
}
fn edit_input(f: &Fixture, edit: impl FnOnce(&mut serde_json::Value)) -> Vec<u8> {
    let path = f.base.join("out/fixture-input.json");
    let mut value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    edit(&mut value);
    let bytes = serde_json::to_vec(&value).unwrap();
    fs::write(path, &bytes).unwrap();
    bytes
}
fn rejected_submission(bytes: impl FnOnce(&Fixture) -> Vec<u8>, expected: SubmissionError) {
    let mut f = Fixture::new();
    f.input(1);
    let bytes = bytes(&f);
    fs::write(f.base.join("out/fixture-input.json"), &bytes).unwrap();
    let declaration = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let result = execute(&mut f, &mut writer, "invalid", declaration);
    assert!(
        matches!(result, Err(BridgeError::Submission(e)) if e == expected),
        "{result:?}"
    );
    assert_no_result_or_admission(&f);
    assert_fault(&writer, FaultKind::InvalidSubmission, "invalid_submission");
    assert_captured_bytes(&f, &bytes);
    drop(writer);
    cold_child(&f.base);
    let report = inspect(f.base.join("store"), f.base.join("ledger")).unwrap();
    assert_eq!(report.counts.attempts, 1);
    assert_eq!(report.counts.observed_failure_attempts, 1);
    assert_eq!(report.counts.store_results, 0);
}

#[test]
fn missing_submission_is_retained_as_capture_with_no_result() {
    let mut f = Fixture::new();
    let declaration = f.intent("matrix::no_submission_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let result = execute(&mut f, &mut writer, "missing-submission", declaration);
    assert!(
        matches!(result, Err(BridgeError::CaptureSet("missing_submission"))),
        "{result:?}"
    );
    assert_no_result_or_admission(&f);
    assert_fault(&writer, FaultKind::InvalidSubmission, "missing_submission");
    assert_captured_bytes(&f, b"review retained without structured submission");
    drop(writer);
    cold_child(&f.base);
}
#[test]
#[ignore = "native fixture subprocess entry, not an unexecuted behavior test"]
fn no_submission_reviewer() {
    fs::write(
        "review.txt",
        b"review retained without structured submission",
    )
    .unwrap();
    println!("reviewer exited zero but did not produce submission");
}
#[test]
fn unknown_json_key_is_rejected_after_retention() {
    rejected_submission(
        |f| {
            edit_input(f, |v| {
                v["unknown"] = true.into();
            })
        },
        SubmissionError::Grammar,
    );
}
#[test]
fn duplicate_json_key_is_rejected_after_retention() {
    rejected_submission(
        |f| {
            let bytes = fs::read(f.base.join("out/fixture-input.json")).unwrap();
            let text = String::from_utf8(bytes).unwrap();
            text.replacen("\"schema\":1", "\"schema\":1,\"schema\":1", 1)
                .into_bytes()
        },
        SubmissionError::Grammar,
    );
}
#[test]
fn duplicate_nested_request_key_is_rejected_after_retention() {
    rejected_submission(
        |f| {
            let text = fs::read_to_string(f.base.join("out/fixture-input.json")).unwrap();
            text.replacen(
                "\"event_type_id\":300",
                "\"event_type_id\":300,\"event_type_id\":300",
                1,
            )
            .into_bytes()
        },
        SubmissionError::Grammar,
    );
}
#[test]
fn missing_field_is_rejected_after_retention() {
    rejected_submission(
        |f| {
            edit_input(f, |v| {
                v.as_object_mut().unwrap().remove("method_status");
            })
        },
        SubmissionError::Grammar,
    );
}
#[test]
fn undefined_status_is_rejected_after_retention() {
    rejected_submission(
        |f| {
            edit_input(f, |v| {
                v["method_status"] = 999.into();
            })
        },
        SubmissionError::Grammar,
    );
}
#[test]
fn fake_role_scope_policy_and_digest_are_not_authority_inputs() {
    for key in ["review_role_id", "scope_id", "policy_record_id", "digest"] {
        rejected_submission(
            |f| {
                edit_input(f, |v| {
                    v[key] = "ff".repeat(32).into();
                })
            },
            SubmissionError::Grammar,
        );
    }
}
#[test]
fn external_and_ambiguous_artifact_paths_never_select_external_bytes() {
    for name in [
        "../secret.txt",
        "/secret.txt",
        "C:/secret.txt",
        "a\\b",
        "review.txt/",
        "CON",
        "a//b",
        "./review.txt",
    ] {
        rejected_submission(
            |f| {
                fs::write(
                    f.base.join("secret.txt"),
                    b"must not capture this outside root",
                )
                .unwrap();
                edit_input(f, |v| {
                    v["artifacts"] = serde_json::json!([name]);
                })
            },
            SubmissionError::ArtifactName,
        );
    }
}
#[test]
fn duplicate_artifact_candidate_is_rejected_after_retention() {
    rejected_submission(
        |f| {
            edit_input(f, |v| {
                v["artifacts"] = serde_json::json!(["review.txt", "review.txt"]);
            })
        },
        SubmissionError::ArtifactName,
    );
}
