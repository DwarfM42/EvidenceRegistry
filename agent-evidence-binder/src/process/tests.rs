use super::*;
use crate::ledger::{CommandLimits, Event};
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
#[cfg(windows)]
thread_local! {
    static HELD_COMPLETION_WRITER: std::cell::RefCell<Option<File>> = const { std::cell::RefCell::new(None) };
}
fn probe(hook: TestHook, expected: &str) {
    probe_with(Some(hook), None, expected);
}
fn probe_with(hook: Option<TestHook>, completion_hook: Option<CompletionHook>, expected: &str) {
    let root = std::env::temp_dir().join(format!(
        "binder-process-fault-{}-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(root.join("out")).unwrap();
    let mut writer = LedgerWriter::open(root.join("ledger")).unwrap();
    let request = RequestBinding {
        registry_id: "1".repeat(64),
        entry_index: 1,
        entry_hash: "2".repeat(64),
        event_type_id: 300,
        event_record_id: "3".repeat(64),
    };
    let intent = DispatchIntent {
        approved_output_root: root.join("out").to_str().unwrap().into(),
        mandatory_files: vec![STDOUT_NAME.into()],
        command: vec![
            std::env::current_exe().unwrap().to_str().unwrap().into(),
            "--ignored".into(),
            "--exact".into(),
            "process::fault_tests::fixture_writer".into(),
            "--nocapture".into(),
        ],
        binder_identity: "test".into(),
        limits: CommandLimits {
            runtime_ms: 1000,
            pipe_drain_ms: 100,
            ..CommandLimits::default()
        },
        predecessor: None,
    };
    let result = run_inner(
        &mut writer,
        request,
        "fault",
        intent,
        &AtomicBool::new(false),
        hook,
        completion_hook,
    );
    assert!(result.is_err());
    if expected == "completion_hold" {
        assert!(
            matches!(&result, Err(ProcessError::CompletionHold(e)) if e.raw_os_error() == Some(32)),
            "actual sharing syscall failure: {result:?}"
        );
        assert!(writer.report().frames.iter().any(|f| matches!(
            f.event,
            Event::Observation(Observation::Exited { code: Some(0) })
        )));
        assert_eq!(
            writer
                .report()
                .frames
                .iter()
                .filter(|f| matches!(f.event, Event::Observation(Observation::PipeEof { .. })))
                .count(),
            2
        );
    }
    assert!(writer.report().frames.iter().any(|f| matches!(&f.event, Event::Observation(Observation::Fault { detail, .. }) if detail == expected)), "missing distinct {expected}: {:?}", writer.report().frames);
    #[cfg(windows)]
    HELD_COMPLETION_WRITER.with(|held| held.borrow_mut().take());
    assert!(root.join("out").join(STDOUT_NAME).exists());
    assert!(root.join("out").join(STDERR_NAME).exists());
    drop(writer);
    let reopened = LedgerWriter::open(root.join("ledger")).unwrap();
    assert!(reopened.report().frames.iter().any(|f| matches!(&f.event, Event::Observation(Observation::Fault { detail, .. }) if detail == expected)));
    assert!(reopened.report().attempts().iter().all(|a| !a.live_witness));
    drop(reopened);
    std::fs::remove_dir_all(root).unwrap();
}
#[cfg(windows)]
#[test]
fn surviving_writable_object_prevents_completion_and_persists_hold_fault() {
    for i in 0..2 {
        let hook: CompletionHook = if i == 0 {
            |_, spools| {
                HELD_COMPLETION_WRITER
                    .with(|held| *held.borrow_mut() = Some(spools[0].try_clone().unwrap()))
            }
        } else {
            |_, spools| {
                HELD_COMPLETION_WRITER
                    .with(|held| *held.borrow_mut() = Some(spools[1].try_clone().unwrap()))
            }
        };
        probe_with(None, Some(hook), "completion_hold");
    }
}
#[cfg(windows)]
#[test]
fn same_object_transition_keeps_originals_until_both_leaf_checks_succeed() {
    let root = std::env::temp_dir().join(format!(
        "binder-transition-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    ));
    std::fs::create_dir(&root).unwrap();
    let held_root = platform::open(&root, true, false).unwrap();
    let out = root.join(STDOUT_NAME);
    let err = root.join(STDERR_NAME);
    let mut spools = [
        platform::open(&out, false, true).unwrap(),
        platform::open(&err, false, true).unwrap(),
    ];
    for spool in &mut spools {
        spool.write_all(b"x").unwrap();
        spool.sync_all().unwrap();
    }
    let root_identity = platform::identity(&held_root).unwrap();
    let identities = [
        platform::identity(&spools[0]).unwrap(),
        platform::identity(&spools[1]).unwrap(),
    ];
    // The second leaf fails after acquiring both reduced handles. Originals must
    // still be held and writable; no partial first-leaf capability transition.
    assert_eq!(
        platform::finish_spools(&mut spools, [&out, &err], [1, 2])
            .unwrap_err()
            .to_string(),
        "completion_hold_size"
    );
    for spool in &mut spools {
        spool.write_all(b"y").unwrap();
        spool.sync_all().unwrap();
    }
    platform::finish_spools(&mut spools, [&out, &err], [2, 2]).unwrap();
    assert_eq!(platform::identity(&held_root).unwrap(), root_identity);
    use std::os::windows::fs::OpenOptionsExt;
    let store_root = OpenOptions::new()
        .read(true)
        .share_mode(1)
        .custom_flags(0x0220_0000)
        .open(&root)
        .unwrap();
    assert_eq!(platform::identity(&store_root).unwrap(), root_identity);
    for (i, path) in [&out, &err].into_iter().enumerate() {
        assert_eq!(platform::identity(&spools[i]).unwrap(), identities[i]);
        assert_eq!(spools[i].write(b"z").unwrap_err().raw_os_error(), Some(5));
        let store_leaf = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .custom_flags(0x0020_0000)
            .open(path)
            .unwrap();
        assert_eq!(platform::identity(&store_leaf).unwrap(), identities[i]);
        assert!(OpenOptions::new().write(true).open(path).is_err());
        assert!(std::fs::remove_file(path).is_err());
    }
    drop(store_root);
    drop(held_root);
    drop(spools);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn actual_spool_write_error_has_distinct_persisted_fault() {
    probe(
        |_, spools, root| {
            // Same inode, actual read-only OS handle: metadata passes, WriteFile/write fails.
            spools[0] = File::open(root.join(STDOUT_NAME)).unwrap();
        },
        "spool_write_Stdout",
    );
}
#[test]
fn actual_pipe_read_error_has_distinct_persisted_fault() {
    probe(
        |stdout, _, root| {
            #[cfg(windows)]
            {
                use std::os::windows::io::OwnedHandle;
                let handle: OwnedHandle = File::open(root.join(STDOUT_NAME)).unwrap().into();
                *stdout = std::process::ChildStdout::from(handle);
            }
            #[cfg(unix)]
            {
                use std::os::fd::OwnedFd;
                let handle: OwnedFd = OpenOptions::new()
                    .write(true)
                    .open(root.join(STDOUT_NAME))
                    .unwrap()
                    .into();
                *stdout = std::process::ChildStdout::from(handle);
            }
        },
        "pipe_read_Stdout",
    );
}
#[test]
fn spool_identity_drift_after_both_eof_prevents_completion() {
    probe_with(
        None,
        Some(|root, _| {
            std::fs::hard_link(root.join(STDOUT_NAME), root.join("post-eof-alias")).unwrap();
        }),
        "spool_validation",
    );
}
#[path = "test_support.rs"]
mod test_support;
use std::net::{TcpListener, TcpStream};
thread_local! {
    static TAMPER_LISTENER: std::cell::RefCell<Option<TcpListener>> = const { std::cell::RefCell::new(None) };
    static TAMPER_CHILD: std::cell::RefCell<Option<(test_support::ChildProbe, TcpStream)>> = const { std::cell::RefCell::new(None) };
}

#[test]
fn ledger_tamper_before_next_observation_is_not_preemptive_containment() {
    let root = std::env::temp_dir().join(format!(
        "binder-tamper-order-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(root.join("out")).unwrap();
    eprintln!("retained tamper ordering evidence: {}", root.display());
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    std::fs::write(
        root.join("out/barrier"),
        listener.local_addr().unwrap().to_string(),
    )
    .unwrap();
    TAMPER_LISTENER.with(|held| *held.borrow_mut() = Some(listener));
    let mut writer = LedgerWriter::open(root.join("ledger")).unwrap();
    let request = RequestBinding {
        registry_id: "1".repeat(64),
        entry_index: 1,
        entry_hash: "2".repeat(64),
        event_type_id: 300,
        event_record_id: "3".repeat(64),
    };
    let intent = DispatchIntent {
        approved_output_root: root.join("out").to_str().unwrap().into(),
        mandatory_files: vec![STDOUT_NAME.into()],
        command: vec![
            std::env::current_exe().unwrap().to_str().unwrap().into(),
            "--ignored".into(),
            "--exact".into(),
            "process::fault_tests::fixture_barrier_tamper".into(),
            "--nocapture".into(),
        ],
        binder_identity: "test".into(),
        limits: CommandLimits {
            runtime_ms: 3000,
            pipe_drain_ms: 100,
            ..CommandLimits::default()
        },
        predecessor: None,
    };
    let result = run_inner(
        &mut writer,
        request,
        "tamper-order",
        intent,
        &AtomicBool::new(false),
        Some(|_, _, _| {
            // Existing hook executes strictly after durable Spawned. Hold the
            // supervisor here until the child has performed its pre-detection write.
            let listener = TAMPER_LISTENER.with(|held| held.borrow_mut().take().unwrap());
            let mut stream = test_support::accept(&listener);
            let mut pid = [0; 4];
            stream.read_exact(&mut pid).unwrap();
            let child = test_support::ChildProbe::open(u32::from_le_bytes(pid));
            test_support::send(&mut stream, b'G');
            test_support::receive(&mut stream, b'M');
            TAMPER_CHILD.with(|held| *held.borrow_mut() = Some((child, stream)));
        }),
        None,
    );
    let error = result.unwrap_err();
    std::fs::write(root.join("process-error.txt"), format!("{error:?}")).unwrap();
    std::fs::write(
        root.join("report.json"),
        serde_json::to_vec_pretty(&writer.report().frames).unwrap(),
    )
    .unwrap();
    TAMPER_CHILD.with(|held| {
        let (child, mut stream) = held.borrow_mut().take().unwrap();
        child.assert_killed();
        test_support::assert_disconnected(&mut stream);
    });
    assert!(writer
        .report()
        .frames
        .iter()
        .any(|f| matches!(f.event, Event::Observation(Observation::Spawned { .. }))));
    // Mutation precedes detection, not vice versa. Keep the side effect and the
    // failed append evidence; killing on detection cannot undo earlier writes.
    assert_eq!(
        std::fs::read(root.join("out/after_tamper")).unwrap(),
        b"pre-detection child side effect"
    );
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
    assert!(writer
        .report()
        .attempts()
        .iter()
        .all(|attempt| !attempt.live_witness));
    assert!(!writer
        .report()
        .frames
        .iter()
        .any(|f| matches!(f.event, Event::Publication(_))));
    assert!(matches!(
        writer.append_observation("tamper-order", Observation::Exited { code: Some(0) }),
        Err(LedgerError::Poisoned)
    ));
    drop(writer);
    let before = std::fs::read(root.join("ledger")).unwrap();
    assert!(matches!(
        crate::ledger::read_ledger(root.join("ledger")),
        Err(LedgerError::Substitution)
    ));
    assert!(matches!(
        LedgerWriter::open(root.join("ledger")),
        Err(LedgerError::Substitution)
    ));
    assert_eq!(std::fs::read(root.join("ledger")).unwrap(), before);
    assert_eq!(std::fs::read(root.join("ledger-alias")).unwrap(), before);
    assert!(root.join("out").join(STDOUT_NAME).is_file());
    assert!(root.join("out").join(STDERR_NAME).is_file());
}

#[test]
#[ignore]
fn fixture_barrier_tamper() {
    let mut stream = TcpStream::connect(std::fs::read_to_string("barrier").unwrap()).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    stream.write_all(&std::process::id().to_le_bytes()).unwrap();
    test_support::receive(&mut stream, b'G');
    std::fs::hard_link("../ledger", "../ledger-alias").unwrap();
    std::fs::write("after_tamper", b"pre-detection child side effect").unwrap();
    test_support::send(&mut stream, b'M');
    test_support::close_stdout();
    // Cannot exit normally while the supervisor handles the forced EOF append.
    test_support::receive(&mut stream, b'X');
    std::process::exit(88);
}

#[test]
#[ignore]
fn fixture_writer() {
    std::io::stdout().write_all(b"write failure probe").unwrap();
    std::thread::sleep(Duration::from_millis(300));
}
