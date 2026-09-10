use ai_agent_evidence_binder::{
    ledger::{CommandLimits, DispatchIntent, Event, LedgerWriter, Observation},
    process::run_managed,
    RequestBinding,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

#[path = "../src/process/test_support.rs"]
mod test_support;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "binder-process-{}-{}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        fs::create_dir(p.join("out")).unwrap();
        Self(p)
    }
    fn writer(&self) -> LedgerWriter {
        LedgerWriter::open(self.0.join("ledger")).unwrap()
    }
    fn intent(&self, fixture: &str) -> DispatchIntent {
        DispatchIntent {
            approved_output_root: self.0.join("out").to_str().unwrap().into(),
            mandatory_files: vec!["binder-stderr.bin".into(), "binder-stdout.bin".into()],
            command: vec![
                std::env::current_exe().unwrap().to_str().unwrap().into(),
                "--ignored".into(),
                "--exact".into(),
                fixture.into(),
                "--nocapture".into(),
            ],
            binder_identity: "process-test".into(),
            limits: CommandLimits {
                runtime_ms: 3000,
                pipe_drain_ms: 100,
                ..CommandLimits::default()
            },
            predecessor: None,
        }
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        if std::thread::panicking() || std::env::var_os("BINDER_PROCESS_RETAIN").is_some() {
            eprintln!("retained process evidence: {}", self.0.display());
        } else {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
fn request() -> RequestBinding {
    RequestBinding {
        registry_id: "1".repeat(64),
        entry_index: 1,
        entry_hash: "2".repeat(64),
        event_type_id: 300,
        event_record_id: "3".repeat(64),
    }
}

#[test]
fn actual_success_binds_owned_bytes_to_durable_intent() {
    let s = Sandbox::new();
    let mut w = s.writer();
    let done = run_managed(
        &mut w,
        request(),
        "success",
        s.intent("fixture_success"),
        &AtomicBool::new(false),
    )
    .expect("actual managed completion");
    assert_eq!(done.request(), &request());
    assert_eq!(done.attempt_id(), "success");
    assert!(fs::read_to_string(done.stdout_path())
        .unwrap()
        .contains("literal stdout payload"));
    assert_eq!(
        fs::read(done.stderr_path()).unwrap(),
        b"literal stderr payload"
    );
    assert_eq!(done.stderr_bytes(), 22);
    assert!(matches!(w.report().frames[0].event, Event::Intent(_)));
    assert!(w.report().frames.iter().any(|f| matches!(
        f.event,
        Event::Observation(Observation::Exited { code: Some(0) })
    )));
    assert_eq!(
        w.report()
            .frames
            .iter()
            .filter(|f| matches!(f.event, Event::Observation(Observation::PipeEof { .. })))
            .count(),
        2
    );
}
/// Actual public Store provisioning, actual subprocess, and actual Store capture.
/// Keep the non-Clone completion alive through prepare, commit and readback.
#[test]
fn managed_completion_remains_alive_during_actual_store_capture() {
    use evidence_registry::*;
    let s = Sandbox::new();
    let mut store = AuthoritativeRegistryStore::initialize_selected_profile(
        s.0.join("store"),
        RegistryId::try_from([91; 32].as_slice()).unwrap(),
        "process-capture",
    )
    .unwrap();
    let scope = |profile| {
        ScopeRecord::new(ScopeRecordInput {
            scope_profile_id: profile,
            scope_profile_version: 1,
            scope_payload: vec![],
            scope_label: None,
        })
        .unwrap()
    };
    let freeze_scope = scope(2);
    let review_scope = scope(1);
    store.stage_scope_record(&freeze_scope).unwrap();
    store.stage_scope_record(&review_scope).unwrap();
    let method = MethodRecord::new(MethodRecordInput {
        method_profile_id: 1,
        method_profile_version: 1,
        method_payload: vec![],
        method_label: None,
    })
    .unwrap();
    let check = CheckRecord::new(CheckRecordInput {
        check_profile_id: 1,
        check_profile_version: 1,
        check_payload: vec![],
        check_label: None,
    })
    .unwrap();
    store.stage_method_record(&method).unwrap();
    store.stage_check_record(&check).unwrap();
    let checks = CheckSetRecord::new(CheckSetRecordInput {
        check_refs: vec![check.record_id()],
    })
    .unwrap();
    store.stage_check_set_record(&checks).unwrap();
    let policy = store
        .register_selected_minimal_policy(freeze_scope.record_id())
        .unwrap();
    let policy_id = RecordId::try_from(policy.event_record_id().as_bytes().as_slice()).unwrap();
    let review_policy = store
        .register_selected_review_policy(
            review_scope.record_id(),
            vec![ReviewAdmissionReviewRequirement::new(
                1,
                review_scope.record_id(),
                method.record_id(),
                checks.record_id(),
                1,
            )
            .unwrap()],
            Some(vec![1]),
            Some(vec![1]),
            Some(vec![1, 2]),
        )
        .unwrap();
    fs::create_dir(s.0.join("source")).unwrap();
    fs::write(s.0.join("source/target.txt"), b"real target").unwrap();
    let target = store
        .prepare_selected_embedded_freeze(SelectedEmbeddedFreezePreparationInput {
            source_root: s.0.join("source"),
            freeze_attempt_id: FreezeAttemptId::try_from([92; 32].as_slice()).unwrap(),
            policy_record_id: policy_id,
        })
        .unwrap();
    let target = store
        .commit_prepared_selected_embedded_freeze(target)
        .unwrap();
    let selected = store
        .record_selected_review_request(SelectedReviewRequestInput {
            freeze_authority: target,
            review_policy_record_id: RecordId::try_from(
                review_policy.event_record_id().as_bytes().as_slice(),
            )
            .unwrap(),
            review_role_id: 1,
        })
        .unwrap();
    let reference = selected.request_event_reference();
    let binding = ai_agent_evidence_binder::bridge::request_binding(reference);
    let mut writer = s.writer();
    let done = run_managed(
        &mut writer,
        binding.clone(),
        "capture",
        s.intent("fixture_success"),
        &AtomicBool::new(false),
    )
    .expect("actual managed process completion before capture");
    let expected_stdout = fs::read(done.stdout_path()).unwrap();
    let expected_stderr = fs::read(done.stderr_path()).unwrap();
    let prepared = store
        .prepare_selected_embedded_freeze_with_limits(
            SelectedEmbeddedFreezePreparationInput {
                source_root: PathBuf::from(&done.intent().approved_output_root),
                freeze_attempt_id: FreezeAttemptId::try_from([93; 32].as_slice()).unwrap(),
                policy_record_id: policy_id,
            },
            SelectedEmbeddedCaptureLimits {
                max_files: 2,
                max_content_bytes: usize::try_from(done.stdout_bytes() + done.stderr_bytes())
                    .unwrap(),
            },
        )
        .expect("Store capture must coexist with BOTH retained root and spool handles");
    let committed = store
        .commit_prepared_selected_embedded_freeze(prepared)
        .unwrap();
    let snapshot = store
        .read_selected_embedded_payload(committed.committed_event_reference().clone())
        .unwrap();
    assert_eq!(snapshot.manifest().input().artifact_count, 2);
    assert_eq!(
        snapshot.artifact_bytes(),
        &[expected_stderr, expected_stdout]
    );
    assert_eq!(done.request(), &binding);
    assert_eq!(done.attempt_id(), "capture");
    #[cfg(windows)]
    {
        // Completion keeps the source objects protected, not just capture-compatible.
        for path in [done.stdout_path(), done.stderr_path()] {
            assert!(fs::OpenOptions::new().write(true).open(path).is_err());
            assert!(fs::rename(path, path.with_extension("moved")).is_err());
        }
        assert!(fs::rename(s.0.join("out"), s.0.join("moved-root")).is_err());
    }
    drop(done);
}

#[test]
fn timeout_is_bounded_and_persisted() {
    let s = Sandbox::new();
    let mut w = s.writer();
    let mut intent = s.intent("fixture_sleep");
    intent.limits.runtime_ms = 40;
    let start = std::time::Instant::now();
    let result = run_managed(
        &mut w,
        request(),
        "timeout",
        intent,
        &AtomicBool::new(false),
    );
    assert!(
        result.is_err(),
        "a timed-out process cannot produce a witness"
    );
    assert!(start.elapsed() < std::time::Duration::from_millis(650));
    assert!(w.report().frames.iter().any(|f| matches!(
        f.event,
        Event::Observation(Observation::Fault {
            fault: ai_agent_evidence_binder::ledger::FaultKind::Timeout,
            ..
        })
    )));
}
#[test]
fn inherited_pipe_after_exit_zero_is_not_completion() {
    let s = Sandbox::new();
    let mut w = s.writer();
    let start = std::time::Instant::now();
    let result = run_managed(
        &mut w,
        request(),
        "descendant",
        s.intent("fixture_descendant"),
        &AtomicBool::new(false),
    );
    assert!(
        result.is_err(),
        "exit0 without both EOF must not mint completion"
    );
    assert!(start.elapsed() < std::time::Duration::from_millis(650));
    assert!(w.report().frames.iter().any(|f| matches!(
        f.event,
        Event::Observation(Observation::Exited { code: Some(0) })
    )));
    assert!(w.report().frames.iter().any(|f| matches!(
        f.event,
        Event::Observation(Observation::Fault {
            fault: ai_agent_evidence_binder::ledger::FaultKind::PipeDrainTimeout,
            ..
        })
    )));
}
#[test]
#[ignore]
fn fixture_descendant() {
    let _descendant = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "fixture_sleep", "--nocapture"])
        .stdin(std::process::Stdio::null())
        .spawn()
        .unwrap();
    std::process::exit(0);
}
#[test]
fn flood_is_capped_before_spool_write_and_fault_is_persisted() {
    for (fixture, pipe, fault) in [
        (
            "fixture_flood_stdout",
            "binder-stdout.bin",
            ai_agent_evidence_binder::ledger::FaultKind::StdoutOverflow,
        ),
        (
            "fixture_flood_stderr",
            "binder-stderr.bin",
            ai_agent_evidence_binder::ledger::FaultKind::StderrOverflow,
        ),
    ] {
        let s = Sandbox::new();
        let mut w = s.writer();
        let mut intent = s.intent(fixture);
        intent.limits.stdout_bytes = 256;
        intent.limits.stderr_bytes = 256;
        assert!(run_managed(&mut w, request(), "flood", intent, &AtomicBool::new(false)).is_err());
        assert!(
            fs::metadata(s.0.join("out").join(pipe)).unwrap().len() <= 256,
            "overflow bytes reached spool"
        );
        assert!(w.report().frames.iter().any(|f| matches!(f.event, Event::Observation(Observation::Fault { fault: seen, .. }) if seen == fault)));
    }
}
#[test]
#[ignore]
fn fixture_flood_stdout() {
    use std::io::Write;
    std::io::stdout().write_all(&[b'x'; 65536]).unwrap();
}
#[test]
#[ignore]
fn fixture_flood_stderr() {
    use std::io::Write;
    std::io::stderr().write_all(&[b'x'; 65536]).unwrap();
}
#[test]
fn cancellation_is_observed_while_actual_child_is_live() {
    let s = Sandbox::new();
    let mut w = s.writer();
    let cancel = AtomicBool::new(false);
    let start = std::time::Instant::now();
    let result = std::thread::scope(|scope| {
        scope.spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(60));
            cancel.store(true, Ordering::Release);
        });
        run_managed(
            &mut w,
            request(),
            "cancel",
            s.intent("fixture_sleep"),
            &cancel,
        )
    });
    assert!(result.is_err(), "cancellation must prevent completion");
    assert!(start.elapsed() < std::time::Duration::from_millis(650));
    assert!(w.report().frames.iter().any(|f| matches!(
        f.event,
        Event::Observation(Observation::Fault {
            fault: ai_agent_evidence_binder::ledger::FaultKind::Cancelled,
            ..
        })
    )));
}
#[test]
fn preflight_collision_preserves_competitor_and_records_no_spawn() {
    for name in ["binder-stdout.bin", "binder-stderr.bin"] {
        let s = Sandbox::new();
        let mut w = s.writer();
        fs::write(s.0.join("out").join(name), b"competitor").unwrap();
        assert!(run_managed(
            &mut w,
            request(),
            "collision",
            s.intent("fixture_marker"),
            &AtomicBool::new(false)
        )
        .is_err());
        assert_eq!(fs::read(s.0.join("out").join(name)).unwrap(), b"competitor");
        assert!(!s.0.join("out/marker").exists());
        assert!(
            w.report()
                .frames
                .iter()
                .any(|f| matches!(f.event, Event::Observation(Observation::SpawnFailed { .. }))),
            "preflight failure must be retained"
        );
        let other = if name == "binder-stdout.bin" {
            "binder-stderr.bin"
        } else {
            "binder-stdout.bin"
        };
        assert!(!s.0.join("out").join(other).exists());
    }
}
#[test]
#[ignore]
fn fixture_marker() {
    fs::write("marker", b"spawned").unwrap();
}
#[test]
fn live_spool_replacement_is_blocked_or_rejects_completion() {
    let s = Sandbox::new();
    let mut w = s.writer();
    let result = run_managed(
        &mut w,
        request(),
        "replace",
        s.intent("fixture_replace_spool"),
        &AtomicBool::new(false),
    );
    let competitor = fs::read(s.0.join("out/competitor")).unwrap();
    assert_eq!(competitor, b"competitor");
    if s.0.join("out/replaced").exists() {
        assert!(result.is_err(), "substituted spool produced a live witness");
        assert_eq!(
            fs::read(s.0.join("out/binder-stdout.bin")).unwrap(),
            b"competitor"
        );
    } else {
        assert!(
            result.is_ok(),
            "blocked substitution should preserve owned spool"
        );
    }
}
#[test]
#[ignore]
fn fixture_replace_spool() {
    fs::write("competitor", b"competitor").unwrap();
    fs::copy("competitor", "replacement").unwrap();
    if fs::rename("replacement", "binder-stdout.bin").is_ok() {
        fs::write("replaced", b"yes").unwrap();
    }
}
#[test]
fn unsupported_descriptor_budget_is_rejected_before_spawn() {
    let s = Sandbox::new();
    let mut w = s.writer();
    let mut intent = s.intent("fixture_marker");
    intent.limits.open_descriptors = 1;
    assert!(
        run_managed(&mut w, request(), "small", intent, &AtomicBool::new(false)).is_err(),
        "declared descriptor budget was not enforced"
    );
    assert!(!s.0.join("out/marker").exists());
    assert!(!s.0.join("out/binder-stdout.bin").exists());
    assert!(w.report().frames.iter().any(|f| matches!(&f.event, Event::Observation(Observation::SpawnFailed { code }) if code == "unsupported_descriptor_budget")));
}
#[test]
fn failed_os_spawn_is_recorded_separately() {
    let s = Sandbox::new();
    let mut w = s.writer();
    let mut intent = s.intent("fixture_marker");
    intent.command[0] = s.0.join("missing.exe").to_str().unwrap().into();
    assert!(run_managed(
        &mut w,
        request(),
        "missing",
        intent,
        &AtomicBool::new(false)
    )
    .is_err());
    assert!(w.report().frames.iter().any(|f| matches!(&f.event, Event::Observation(Observation::SpawnFailed { code }) if code == "os_spawn_failed")), "spawn failure was not retained");
    assert!(!w
        .report()
        .frames
        .iter()
        .any(|f| matches!(f.event, Event::Observation(Observation::Spawned { .. }))));
}
#[cfg(windows)]
#[test]
fn implicit_batch_shell_is_rejected_before_spawn() {
    let s = Sandbox::new();
    let mut w = s.writer();
    let script = s.0.join("run.cmd");
    fs::write(&script, b"@echo off\r\necho spawned>marker\r\n").unwrap();
    let mut intent = s.intent("fixture_marker");
    intent.command = vec![script.to_str().unwrap().into()];
    assert!(
        run_managed(&mut w, request(), "batch", intent, &AtomicBool::new(false)).is_err(),
        "Command implicitly introduced cmd.exe"
    );
    assert!(!s.0.join("out/marker").exists());
    assert!(w.report().frames.iter().any(|f| matches!(&f.event, Event::Observation(Observation::SpawnFailed { code }) if code == "unsupported_executable")));
}
#[cfg(unix)]
#[test]
fn relative_executable_is_rejected_before_path_lookup() {
    let s = Sandbox::new();
    let mut w = s.writer();
    let mut intent = s.intent("fixture_marker");
    intent.command[0] = "fixture_success".into();
    assert!(
        run_managed(
            &mut w,
            request(),
            "relative",
            intent,
            &AtomicBool::new(false)
        )
        .is_err(),
        "relative executable was accepted for PATH lookup"
    );
    assert!(!s.0.join("out/marker").exists());
    assert!(!s.0.join("out/binder-stdout.bin").exists());
    assert!(w.report().frames.iter().any(|f| matches!(&f.event, Event::Observation(Observation::SpawnFailed { code }) if code == "unsupported_executable")));
}
#[test]
fn failed_intent_has_no_spawn_or_spool_side_effect() {
    let s = Sandbox::new();
    let mut w = s.writer();
    // Actual link-identity failure inside append_intent; Windows locks block dirty writes.
    fs::hard_link(s.0.join("ledger"), s.0.join("ledger-alias")).unwrap();
    assert!(run_managed(
        &mut w,
        request(),
        "invalid",
        s.intent("fixture_marker"),
        &AtomicBool::new(false)
    )
    .is_err());
    assert!(!s.0.join("out/marker").exists());
    assert!(!s.0.join("out/binder-stdout.bin").exists());
}
#[test]
fn one_dispatch_cannot_issue_a_second_witness_even_after_reopen() {
    let s = Sandbox::new();
    let mut w = s.writer();
    let completed = run_managed(
        &mut w,
        request(),
        "once",
        s.intent("fixture_marker"),
        &AtomicBool::new(false),
    )
    .unwrap();
    let frames = w.report().frames.len();
    assert!(run_managed(
        &mut w,
        request(),
        "once",
        s.intent("fixture_marker"),
        &AtomicBool::new(false)
    )
    .is_err());
    assert_eq!(w.report().frames.len(), frames);
    drop(completed);
    drop(w);
    fs::remove_file(s.0.join("out/marker")).unwrap();
    let mut w = s.writer();
    assert!(run_managed(
        &mut w,
        request(),
        "once",
        s.intent("fixture_marker"),
        &AtomicBool::new(false)
    )
    .is_err());
    assert!(!s.0.join("out/marker").exists());
    assert!(w.report().attempts().iter().all(|a| !a.live_witness));
}
#[test]
fn producer_nonzero_and_crash_never_mint_completion() {
    for fixture in ["fixture_nonzero", "fixture_abort"] {
        let s = Sandbox::new();
        let mut w = s.writer();
        assert!(run_managed(
            &mut w,
            request(),
            "failed",
            s.intent(fixture),
            &AtomicBool::new(false)
        )
        .is_err());
        assert!(w.report().frames.iter().any(|f| matches!(f.event, Event::Observation(Observation::Exited { code }) if code != Some(0))));
        assert!(!w
            .report()
            .frames
            .iter()
            .any(|f| matches!(f.event, Event::Publication(_))));
    }
}
#[test]
fn observation_append_failure_kills_actual_child_without_a_witness() {
    use ai_agent_evidence_binder::{ledger::LedgerError, process::ProcessError};
    use std::{io::Read, net::TcpListener};
    let s = Sandbox::new();
    let mut w = s.writer();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    fs::write(
        s.0.join("out/barrier"),
        listener.local_addr().unwrap().to_string(),
    )
    .unwrap();
    let (result, child, mut stream) = std::thread::scope(|scope| {
        let runner = scope.spawn(|| {
            run_managed(
                &mut w,
                request(),
                "ledger_drift",
                s.intent("fixture_tamper_ledger"),
                &AtomicBool::new(false),
            )
        });
        let mut stream = test_support::accept(&listener);
        let mut pid = [0; 4];
        stream.read_exact(&mut pid).unwrap();
        let child = test_support::ChildProbe::open(u32::from_le_bytes(pid));
        test_support::send(&mut stream, b'G');
        (runner.join().unwrap(), child, stream)
    });
    let error = result.expect_err("failed observation must never mint completion");
    fs::write(s.0.join("process-error.txt"), format!("{error:?}")).unwrap();
    fs::write(
        s.0.join("report.json"),
        serde_json::to_vec_pretty(&w.report().frames).unwrap(),
    )
    .unwrap();
    child.assert_killed();
    test_support::assert_disconnected(&mut stream);
    let ProcessError::Cleanup { primary, detail } = error else {
        panic!("missing exit-persistence failure")
    };
    assert_eq!(detail, "exit_persistence:Poisoned");
    let ProcessError::Cleanup { primary, detail } = *primary else {
        panic!("missing fault-persistence failure")
    };
    assert_eq!(detail, "observation_persistence:Poisoned");
    assert!(matches!(
        *primary,
        ProcessError::Ledger(LedgerError::Substitution)
    ));
    assert!(w
        .report()
        .attempts()
        .iter()
        .all(|attempt| !attempt.live_witness));
    assert!(!w
        .report()
        .frames
        .iter()
        .any(|f| matches!(f.event, Event::Publication(_))));
    assert!(!s.0.join("out/after_tamper").exists());
    assert!(matches!(
        w.append_observation("ledger_drift", Observation::Exited { code: Some(0) }),
        Err(LedgerError::Poisoned)
    ));
    drop(w);
    let before = fs::read(s.0.join("ledger")).unwrap();
    // Keep the alias: removing tamper to obtain a clean reopen would erase the
    // failure. A cold public reader/writer must still fail closed on identity.
    assert!(matches!(
        ai_agent_evidence_binder::ledger::read_ledger(s.0.join("ledger")),
        Err(LedgerError::Substitution)
    ));
    assert!(matches!(
        LedgerWriter::open(s.0.join("ledger")),
        Err(LedgerError::Substitution)
    ));
    assert_eq!(fs::read(s.0.join("ledger")).unwrap(), before);
    assert_eq!(fs::read(s.0.join("ledger-alias")).unwrap(), before);
    assert!(s.0.join("out/binder-stdout.bin").is_file());
    assert!(s.0.join("out/binder-stderr.bin").is_file());
}
#[test]
fn linked_root_is_rejected_without_writing_through_it() {
    let s = Sandbox::new();
    let mut w = s.writer();
    fs::create_dir(s.0.join("elsewhere")).unwrap();
    let link = s.0.join("linked");
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(s.0.join("elsewhere"), &link).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(s.0.join("elsewhere"), &link).unwrap();
    let mut intent = s.intent("fixture_marker");
    intent.approved_output_root = link.to_str().unwrap().into();
    assert!(run_managed(&mut w, request(), "link", intent, &AtomicBool::new(false)).is_err());
    assert!(!s.0.join("elsewhere/binder-stdout.bin").exists());
    #[cfg(windows)]
    fs::remove_dir(link).unwrap();
    #[cfg(unix)]
    fs::remove_file(link).unwrap();
}
#[test]
fn shell_metacharacters_remain_literal_arguments_and_stdin_is_null() {
    let s = Sandbox::new();
    let mut w = s.writer();
    let mut intent = s.intent("fixture_stdin");
    intent.command.extend([
        "--skip".into(),
        "nothing & echo injected > injected-marker".into(),
    ]);
    let done = run_managed(
        &mut w,
        request(),
        "literal",
        intent,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(fs::read_to_string(done.stdout_path())
        .unwrap()
        .contains("stdin_eof"));
    assert!(!s.0.join("out/injected-marker").exists());
}
#[test]
fn supervisor_crash_reopens_history_without_resuming_or_spawning() {
    let s = Sandbox::new();
    let mut owner = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "fixture_supervisor_owner",
            "--nocapture",
        ])
        .current_dir(&s.0)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while !s.0.join("out/crash_started").exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    owner.kill().unwrap();
    while owner.try_wait().unwrap().is_none() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(s.0.join("out/crash_started").exists());
    assert!(owner.try_wait().unwrap().is_some());
    let mut w = s.writer();
    let count = w.report().frames.len();
    assert!(w.report().attempts().iter().all(|a| !a.live_witness));
    assert!(run_managed(
        &mut w,
        request(),
        "crashed",
        s.intent("fixture_marker"),
        &AtomicBool::new(false)
    )
    .is_err());
    assert_eq!(w.report().frames.len(), count);
    assert!(!s.0.join("out/marker").exists());
    assert!(!w
        .report()
        .frames
        .iter()
        .any(|f| matches!(f.event, Event::Publication(_))));
    drop(w);
    // The supervised child is deliberately not a process-tree containment claim.
    // This bounded fixture exits itself; wait only to clean up this test's files.
    while !s.0.join("out/crash_child_done").exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(s.0.join("out/crash_child_done").exists());
}
#[test]
#[ignore]
fn fixture_nonzero() {
    std::process::exit(7);
}
#[test]
#[ignore]
fn fixture_abort() {
    std::process::abort();
}
#[test]
#[ignore]
fn fixture_tamper_ledger() {
    use std::io::Write;
    let mut stream = std::net::TcpStream::connect(fs::read_to_string("barrier").unwrap()).unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(3)))
        .unwrap();
    stream.write_all(&std::process::id().to_le_bytes()).unwrap();
    test_support::receive(&mut stream, b'G');
    fs::hard_link("../ledger", "../ledger-alias").unwrap();
    std::io::stdout().write_all(b"tampered").unwrap();
    // Bytes alone do not append a ledger observation. Close the native stdout
    // handle to force an EOF observation while this actual child remains live.
    test_support::close_stdout();
    // No elapsed-time guess: marker is reachable only through an explicit
    // release which the parent never sends before verifying native termination.
    test_support::receive(&mut stream, b'X');
    fs::write("after_tamper", b"alive").unwrap();
    std::process::exit(88);
}
#[test]
#[ignore]
fn fixture_stdin() {
    use std::io::Read;
    let mut b = [0; 1];
    assert_eq!(std::io::stdin().read(&mut b).unwrap(), 0);
    println!("stdin_eof");
}
#[test]
#[ignore]
fn fixture_supervisor_owner() {
    let s = Sandbox(std::env::current_dir().unwrap());
    let mut w = s.writer();
    let _ = run_managed(
        &mut w,
        request(),
        "crashed",
        s.intent("fixture_crash_child"),
        &AtomicBool::new(false),
    );
    std::process::exit(3);
}
#[test]
#[ignore]
fn fixture_crash_child() {
    fs::write("crash_started", b"live").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1500));
    fs::write("crash_child_done", b"done").unwrap();
    std::process::exit(0);
}
#[test]
fn already_cancelled_intent_never_launches() {
    let s = Sandbox::new();
    let mut w = s.writer();
    assert!(run_managed(
        &mut w,
        request(),
        "cancelled",
        s.intent("fixture_marker"),
        &AtomicBool::new(true)
    )
    .is_err());
    assert!(
        !w.report()
            .frames
            .iter()
            .any(|f| matches!(f.event, Event::Observation(Observation::Spawned { .. }))),
        "pre-cancelled dispatch spawned a child"
    );
    assert!(w.report().frames.iter().any(|f| matches!(&f.event, Event::Observation(Observation::SpawnFailed { code }) if code == "cancelled_before_spawn")));
    assert!(!s.0.join("out/marker").exists());
}
#[test]
#[ignore]
fn fixture_sleep() {
    std::thread::sleep(std::time::Duration::from_millis(800));
}
#[test]
#[ignore]
fn fixture_success() {
    use std::io::Write;
    std::io::stdout()
        .write_all(b"literal stdout payload")
        .unwrap();
    std::io::stderr()
        .write_all(b"literal stderr payload")
        .unwrap();
}
