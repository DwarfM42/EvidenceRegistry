use ai_agent_evidence_binder::{ledger::*, RequestBinding};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "binder-ledger-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn ledger(&self) -> PathBuf {
        self.0.join("attempts.ledger")
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
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
fn intent(root: &Sandbox) -> DispatchIntent {
    DispatchIntent {
        approved_output_root: root.0.to_string_lossy().into_owned(),
        mandatory_files: vec!["submission.json".into()],
        command: vec!["some-agent".into(), "review".into()],
        binder_identity: "test-binder-build".into(),
        limits: CommandLimits::default(),
        predecessor: None,
    }
}
fn complete(w: &mut LedgerWriter, attempt: &str) {
    w.append_observation(attempt, Observation::Spawned { pid: 42 })
        .unwrap();
    w.append_observation(attempt, Observation::Exited { code: Some(0) })
        .unwrap();
    for pipe in [Pipe::Stdout, Pipe::Stderr] {
        w.append_observation(
            attempt,
            Observation::PipeEof {
                pipe,
                bytes: 0,
                sha256: "a".repeat(64),
            },
        )
        .unwrap();
    }
}
fn reference() -> RetainedReference {
    RetainedReference {
        registry_id: "1".repeat(64),
        entry_index: 2,
        entry_hash: "4".repeat(64),
        event_type_id: 301,
        event_record_id: "5".repeat(64),
    }
}
#[test]
fn publication_crash_uncertainty_and_terminal_refs_are_not_latest_only() {
    let root = Sandbox::new();
    let mut w = LedgerWriter::open(root.ledger()).unwrap();
    drop(w.append_intent(request(), "a1", intent(&root)).unwrap());
    complete(&mut w, "a1");
    w.append_publication(
        "a1",
        Publication::Intent {
            operation: PublicationKind::OutputCapture,
        },
    )
    .unwrap();
    drop(w);
    let r = read_ledger(root.ledger()).unwrap();
    assert_eq!(
        r.attempts()[0].publications[0].status,
        PublicationStatus::Unresolved
    );
    let mut w = LedgerWriter::open(root.ledger()).unwrap();
    assert!(w
        .append_publication(
            "a1",
            Publication::Returned {
                operation: PublicationKind::OutputCapture,
                references: vec![reference()],
                receipt: "late untrusted claim".into()
            }
        )
        .is_err());
    let mut retry = intent(&root);
    retry.predecessor = Some("a1".into());
    drop(w.append_intent(request(), "a2", retry).unwrap());
    complete(&mut w, "a2");
    w.append_publication(
        "a2",
        Publication::Intent {
            operation: PublicationKind::OutputCapture,
        },
    )
    .unwrap();
    w.append_publication(
        "a2",
        Publication::Uncertain {
            operation: PublicationKind::OutputCapture,
            code: "return_lost".into(),
        },
    )
    .unwrap();
    assert!(w
        .append_publication(
            "a2",
            Publication::Intent {
                operation: PublicationKind::OutputCapture
            }
        )
        .is_err());
    let mut retry = intent(&root);
    retry.predecessor = Some("a2".into());
    drop(w.append_intent(request(), "a3", retry).unwrap());
    complete(&mut w, "a3");
    for operation in [
        PublicationKind::OutputCapture,
        PublicationKind::ReviewResult,
        PublicationKind::Admission,
    ] {
        w.append_publication("a3", Publication::Intent { operation })
            .unwrap();
        w.append_publication(
            "a3",
            Publication::Returned {
                operation,
                references: vec![reference()],
                receipt: "test-only returned receipt".into(),
            },
        )
        .unwrap();
    }
    drop(w);
    let r = read_ledger(root.ledger()).unwrap();
    let a = r.attempts();
    assert_eq!(a.len(), 3);
    assert_eq!(a[0].publications[0].status, PublicationStatus::Unresolved);
    assert_eq!(
        a[1].publications[0].status,
        PublicationStatus::Uncertain("return_lost".into())
    );
    assert_eq!(a[2].publications.len(), 3);
    assert_eq!(a[2].publications[2].references, vec![reference()]);
    assert!(a.iter().all(|a| !a.live_witness));
}
#[test]
fn separate_preparation_and_capture_mutations_each_have_an_intent() {
    let root = Sandbox::new();
    let mut w = LedgerWriter::open(root.ledger()).unwrap();
    drop(w.append_intent(request(), "a", intent(&root)).unwrap());
    complete(&mut w, "a");
    let prepare: Publication = serde_json::from_value(
        serde_json::json!({"kind":"Intent", "operation":"OutputPreparation"}),
    )
    .unwrap();
    w.append_publication("a", prepare).unwrap();
    assert!(w
        .append_publication(
            "a",
            Publication::Intent {
                operation: PublicationKind::OutputCapture
            }
        )
        .is_err());
    let returned: Publication = serde_json::from_value(serde_json::json!({"kind":"Returned", "operation":"OutputPreparation", "references":[reference()], "receipt":"actual return fixture"})).unwrap();
    w.append_publication("a", returned).unwrap();
    w.append_publication(
        "a",
        Publication::Intent {
            operation: PublicationKind::OutputCapture,
        },
    )
    .unwrap();
    drop(w);
    assert_eq!(
        read_ledger(root.ledger()).unwrap().attempts()[0]
            .publications
            .len(),
        2
    );
}
#[test]
fn captured_submission_fault_survives_reopen_without_reviving_publication() {
    let root = Sandbox::new();
    let mut w = LedgerWriter::open(root.ledger()).unwrap();
    assert_eq!(w.path(), root.ledger());
    drop(
        w.append_intent(request(), "captured", intent(&root))
            .unwrap(),
    );
    complete(&mut w, "captured");
    w.append_publication(
        "captured",
        Publication::Intent {
            operation: PublicationKind::OutputCapture,
        },
    )
    .unwrap();
    w.append_publication(
        "captured",
        Publication::Returned {
            operation: PublicationKind::OutputCapture,
            references: vec![reference()],
            receipt: "synthetic local grammar fixture; not Store evidence".into(),
        },
    )
    .unwrap();
    w.append_observation(
        "captured",
        Observation::Fault {
            fault: FaultKind::InvalidSubmission,
            detail: "invalid_submission".into(),
        },
    )
    .expect("capture may succeed before submission validation fails");
    assert!(matches!(
        w.append_publication(
            "captured",
            Publication::Intent {
                operation: PublicationKind::ReviewResult,
            }
        ),
        Err(LedgerError::Invalid)
    ));
    assert!(matches!(
        w.append_observation("captured", Observation::Exited { code: Some(0) }),
        Err(LedgerError::Invalid)
    ));
    drop(w);
    let report = read_ledger(root.ledger()).unwrap();
    assert!(report.boundary.is_none());
    assert!(matches!(&report.frames.last().unwrap().event,
        Event::Observation(Observation::Fault {fault: FaultKind::InvalidSubmission, detail})
        if detail == "invalid_submission"));
    assert_eq!(
        report.attempts()[0].publications[0].status,
        PublicationStatus::KnownReferences
    );
    assert!(!report.attempts()[0].live_witness);
    let mut reopened = LedgerWriter::open(root.ledger()).unwrap();
    assert!(matches!(
        reopened.append_observation(
            "captured",
            Observation::Fault {
                fault: FaultKind::Unavailable,
                detail: "historical_attempt".into(),
            }
        ),
        Err(LedgerError::NotLive)
    ));
}

#[test]
fn distinct_faults_remain_retained_and_block_publication() {
    for fault in [
        "Timeout",
        "Cancelled",
        "StdoutOverflow",
        "StderrOverflow",
        "PipeDrainTimeout",
        "StreamTruncated",
        "ObservationWriteFailed",
        "CaptureFailed",
        "InvalidSubmission",
        "Unavailable",
    ] {
        let root = Sandbox::new();
        let mut w = LedgerWriter::open(root.ledger()).unwrap();
        drop(w.append_intent(request(), "a", intent(&root)).unwrap());
        w.append_observation("a", Observation::Spawned { pid: 42 })
            .unwrap();
        w.append_observation(
            "a",
            obs(serde_json::json!({"kind":"Fault", "fault":fault, "detail":"bounded diagnostic"})),
        )
        .unwrap();
        w.append_observation("a", Observation::Exited { code: Some(0) })
            .unwrap();
        for pipe in [Pipe::Stdout, Pipe::Stderr] {
            w.append_observation(
                "a",
                Observation::PipeEof {
                    pipe,
                    bytes: 0,
                    sha256: "a".repeat(64),
                },
            )
            .unwrap();
        }
        assert!(w
            .append_publication(
                "a",
                Publication::Intent {
                    operation: PublicationKind::OutputCapture
                }
            )
            .is_err());
        drop(w);
        let r = read_ledger(root.ledger()).unwrap();
        assert!(serde_json::to_string(&r.frames[2]).unwrap().contains(fault));
    }
}
#[test]
fn competing_writer_and_reader_are_busy_in_another_process() {
    let root = Sandbox::new();
    let mut w = LedgerWriter::open(root.ledger()).unwrap();
    drop(w.append_intent(request(), "locked", intent(&root)).unwrap());
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "child_probe", "--nocapture"])
        .env("BINDER_LEDGER_CHILD", "busy")
        .env("BINDER_LEDGER_PATH", root.ledger())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(matches!(read_ledger(root.ledger()), Err(LedgerError::Busy)));
    drop(w);
    assert_eq!(read_ledger(root.ledger()).unwrap().frames.len(), 1);
}
#[test]
fn child_probe() {
    let Ok(mode) = std::env::var("BINDER_LEDGER_CHILD") else {
        return;
    };
    let path = PathBuf::from(std::env::var_os("BINDER_LEDGER_PATH").unwrap());
    if mode == "busy" {
        assert!(matches!(LedgerWriter::open(&path), Err(LedgerError::Busy)));
        assert!(matches!(read_ledger(&path), Err(LedgerError::Busy)));
    } else if mode == "write_exit" {
        let mut w = LedgerWriter::open(&path).unwrap();
        let i = serde_json::from_str(&std::env::var("BINDER_LEDGER_INTENT").unwrap()).unwrap();
        drop(w.append_intent(request(), "cold", i).unwrap());
        // No destructors: the OS, not a stale lock-file cleanup, releases lock.
        std::process::exit(0);
    } else if mode == "read_cold" {
        let r = read_ledger(&path).unwrap();
        assert_eq!(r.frames.len(), 1);
        assert_eq!(r.attempts()[0].launch, LaunchProjection::Unresolved);
        assert!(!r.attempts()[0].live_witness);
        let mut w = LedgerWriter::open(&path).unwrap();
        assert!(matches!(
            w.append_observation("cold", Observation::Spawned { pid: 1 }),
            Err(LedgerError::NotLive)
        ));
    } else {
        panic!("unknown child test mode");
    }
}
#[test]
fn actual_process_exit_and_separate_cold_reopen_do_not_resume() {
    let root = Sandbox::new();
    for mode in ["write_exit", "read_cold"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "child_probe", "--nocapture"])
            .env("BINDER_LEDGER_CHILD", mode)
            .env("BINDER_LEDGER_PATH", root.ledger())
            .env(
                "BINDER_LEDGER_INTENT",
                serde_json::to_string(&intent(&root)).unwrap(),
            )
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{:?} {} {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert_eq!(read_ledger(root.ledger()).unwrap().frames.len(), 1);
}
fn raw_frame(payload: &[u8]) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    let len = (payload.len() as u32).to_le_bytes();
    let mut h = Sha256::new();
    h.update(b"binder-ledger-frame-v1\0");
    h.update(len);
    h.update(payload);
    let mut out = len.to_vec();
    out.extend(payload);
    out.extend(h.finalize());
    out
}
fn fixture() -> (Sandbox, Vec<u8>, usize) {
    let root = Sandbox::new();
    let mut w = LedgerWriter::open(root.ledger()).unwrap();
    drop(w.append_intent(request(), "a", intent(&root)).unwrap());
    let first_end = w.report().valid_bytes as usize;
    w.append_observation("a", Observation::Spawned { pid: 1 })
        .unwrap();
    drop(w);
    let bytes = fs::read(root.ledger()).unwrap();
    (root, bytes, first_end)
}
#[test]
fn actual_torn_length_payload_and_digest_keep_valid_prefix_never_truncate() {
    let (root, bytes, first_end) = fixture();
    for end in [
        first_end + 1,
        first_end + 3,
        first_end + 4,
        first_end + 12,
        bytes.len() - 32,
        bytes.len() - 1,
    ] {
        fs::write(root.ledger(), &bytes[..end]).unwrap();
        let r = read_ledger(root.ledger()).unwrap();
        assert_eq!(r.frames.len(), 1);
        assert_eq!(r.valid_bytes, first_end as u64);
        assert_eq!(
            r.boundary,
            Some(Boundary {
                offset: first_end as u64,
                kind: BoundaryKind::Torn
            })
        );
        assert!(matches!(
            LedgerWriter::open(root.ledger()),
            Err(LedgerError::Corrupt(_))
        ));
        assert_eq!(fs::read(root.ledger()).unwrap(), bytes[..end]);
    }
    for n in 0..8 {
        fs::write(root.ledger(), &bytes[..n]).unwrap();
        assert!(matches!(
            LedgerWriter::open(root.ledger()),
            Err(LedgerError::Corrupt(_))
        ));
    }
}
#[test]
fn bad_digest_sequence_prevhash_request_attempt_and_order_stop_at_boundary() {
    let (root, bytes, first_end) = fixture();
    let size = u32::from_le_bytes(bytes[first_end..first_end + 4].try_into().unwrap()) as usize;
    let original: Frame =
        serde_json::from_slice(&bytes[first_end + 4..first_end + 4 + size]).unwrap();
    for case in 0..8 {
        let mut f = original.clone();
        let kind = match case {
            0 => {
                f.sequence += 1;
                BoundaryKind::Chain
            }
            1 => {
                f.prev_hash = "f".repeat(64);
                BoundaryKind::Chain
            }
            2 => {
                f.request.entry_hash = "e".repeat(64);
                BoundaryKind::Order
            }
            3 => {
                f.attempt_id = "missing".into();
                BoundaryKind::Order
            }
            4 => {
                f.event = Event::Observation(Observation::Exited { code: Some(0) });
                BoundaryKind::Order
            }
            5 => {
                f.version = 2;
                BoundaryKind::Grammar
            }
            6 => {
                f.request.event_type_id = 301;
                BoundaryKind::Order
            }
            _ => {
                f.request.registry_id = "A".repeat(64);
                BoundaryKind::Order
            }
        };
        let mut modified = bytes[..first_end].to_vec();
        modified.extend(raw_frame(&serde_json::to_vec(&f).unwrap()));
        fs::write(root.ledger(), &modified).unwrap();
        let r = read_ledger(root.ledger()).unwrap();
        assert_eq!(r.frames.len(), 1);
        assert_eq!(r.boundary.unwrap().kind, kind);
        assert!(matches!(
            LedgerWriter::open(root.ledger()),
            Err(LedgerError::Corrupt(_))
        ));
        assert_eq!(fs::read(root.ledger()).unwrap(), modified);
    }
    let mut bad = bytes.clone();
    *bad.last_mut().unwrap() ^= 1;
    fs::write(root.ledger(), bad).unwrap();
    assert_eq!(
        read_ledger(root.ledger()).unwrap().boundary.unwrap().kind,
        BoundaryKind::Digest
    );
}
#[test]
fn unknown_duplicate_missing_keys_and_noncanonical_json_are_corrupt() {
    let (root, bytes, first_end) = fixture();
    let n = u32::from_le_bytes(bytes[first_end..first_end + 4].try_into().unwrap()) as usize;
    let payload = String::from_utf8(bytes[first_end + 4..first_end + 4 + n].to_vec()).unwrap();
    for malformed in [
        payload.replacen("{", "{\"extra\":0,", 1),
        payload.replacen("{", "{\"version\":1,", 1),
        payload.replace("\"pid\":1", "\"other\":1"),
        format!(" {payload}"),
    ] {
        let mut changed = bytes[..first_end].to_vec();
        changed.extend(raw_frame(malformed.as_bytes()));
        fs::write(root.ledger(), changed).unwrap();
        assert_eq!(
            read_ledger(root.ledger()).unwrap().boundary.unwrap().kind,
            BoundaryKind::Grammar
        );
    }
}
#[test]
fn duplicate_ids_retries_and_request_grammar_fail_before_mutation() {
    let root = Sandbox::new();
    let mut w = LedgerWriter::open(root.ledger()).unwrap();
    drop(w.append_intent(request(), "first", intent(&root)).unwrap());
    assert!(w.append_intent(request(), "first", intent(&root)).is_err());
    assert!(w.append_intent(request(), "second", intent(&root)).is_err());
    let mut i = intent(&root);
    i.predecessor = Some("absent".into());
    assert!(w.append_intent(request(), "second", i).is_err());
    for case in 0..5 {
        let mut q = request();
        match case {
            0 => q.registry_id = "X".repeat(64),
            1 => q.entry_index = u64::MAX,
            2 => q.entry_hash = "a".repeat(63),
            3 => q.event_type_id = 301,
            _ => q.event_record_id = "b".repeat(65),
        };
        assert!(w.append_intent(q, "invalid", intent(&root)).is_err());
    }
    assert_eq!(w.report().frames.len(), 1);
}
#[test]
fn concrete_command_and_frame_limits_are_enforced_before_write() {
    let root = Sandbox::new();
    let mut w = LedgerWriter::open(root.ledger()).unwrap();
    for case in 0..12 {
        let mut i = intent(&root);
        match case {
            0 => i.command = vec!["x".into(); 65],
            1 => i.command = vec!["x".repeat(4097)],
            2 => i.command = vec!["x".repeat(4096); 5],
            3 => i.binder_identity = "x".repeat(257),
            4 => i.mandatory_files.push("../escape".into()),
            5 => i.mandatory_files.push("submission.json".into()),
            6 => i.approved_output_root = "relative".into(),
            7 => i.limits.runtime_ms += 1,
            8 => i.limits.stdout_bytes = 0,
            9 => i.limits.pipe_drain_ms += 1,
            10 => i.limits.output_files += 1,
            _ => i.limits.open_descriptors += 1,
        }
        assert!(
            w.append_intent(request(), "invalid", i).is_err(),
            "case {case}"
        );
        assert_eq!(w.report().frames.len(), 0);
    }
    let mut large = intent(&root);
    large.mandatory_files = (0..256)
        .map(|n| format!("f{n:03}{}", "x".repeat(300)))
        .collect();
    assert!(matches!(
        w.append_intent(request(), "large", large),
        Err(LedgerError::Limit)
    ));
    drop(w);
    let mut bytes = fs::read(root.ledger()).unwrap();
    bytes.extend(((MAX_FRAME_BYTES as u32) + 1).to_le_bytes());
    fs::write(root.ledger(), bytes).unwrap();
    assert_eq!(
        read_ledger(root.ledger()).unwrap().boundary.unwrap().kind,
        BoundaryKind::Length
    );
}
#[test]
fn hardlinks_are_rejected_on_open_and_during_live_append() {
    let root = Sandbox::new();
    let mut w = LedgerWriter::open(root.ledger()).unwrap();
    let alias = root.0.join("alias");
    fs::hard_link(root.ledger(), &alias).unwrap();
    assert!(matches!(
        w.append_intent(request(), "a", intent(&root)),
        Err(LedgerError::Substitution)
    ));
    assert!(matches!(
        w.append_intent(request(), "b", intent(&root)),
        Err(LedgerError::Poisoned)
    ));
    drop(w);
    assert!(matches!(
        LedgerWriter::open(root.ledger()),
        Err(LedgerError::Substitution)
    ));
    assert!(matches!(
        read_ledger(root.ledger()),
        Err(LedgerError::Substitution)
    ));
    assert_eq!(fs::read(&alias).unwrap(), b"BNDLGR01");
}
#[test]
fn creation_race_does_not_admit_two_live_writers() {
    let root = Sandbox::new();
    let start = std::sync::Arc::new(std::sync::Barrier::new(8));
    let held = std::sync::Arc::new(std::sync::Barrier::new(8));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let start = start.clone();
            let held = held.clone();
            let path = root.ledger();
            std::thread::spawn(move || {
                start.wait();
                let w = LedgerWriter::open(path);
                let success = w.is_ok();
                held.wait();
                drop(w);
                success
            })
        })
        .collect();
    let successes = threads
        .into_iter()
        .map(|t| t.join().unwrap())
        .filter(|ok| *ok)
        .count();
    assert!(successes <= 1);
    // If an existing opener locks before the new-file creator, both fail closed;
    // availability is not obtained by treating an existing empty file as new.
    if successes == 1 {
        assert!(read_ledger(root.ledger()).unwrap().boundary.is_none());
    }
}

// Fixture writer for adversarial replay controls only, never a product producer.
fn append_fixture(bytes: &mut Vec<u8>, sequence: u64, attempt: &str, event: Event) {
    let prev_hash = if bytes.len() == 8 {
        "0".repeat(64)
    } else {
        bytes[bytes.len() - 32..]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    };
    let f = Frame {
        version: 1,
        sequence,
        prev_hash,
        request: request(),
        attempt_id: attempt.into(),
        event,
    };
    bytes.extend(raw_frame(&serde_json::to_vec(&f).unwrap()));
}
#[test]
fn hard_frame_count_and_attempt_count_limits_stop_reader_and_writer() {
    let root = Sandbox::new();
    let mut bytes = b"BNDLGR01".to_vec();
    for n in 0..MAX_ATTEMPTS {
        let mut i = intent(&root);
        if n > 0 {
            i.predecessor = Some(format!("a{}", n - 1));
        }
        append_fixture(&mut bytes, n as u64 + 1, &format!("a{n}"), Event::Intent(i));
    }
    fs::write(root.ledger(), &bytes).unwrap();
    let mut w = LedgerWriter::open(root.ledger()).unwrap();
    let mut i = intent(&root);
    i.predecessor = Some("a0".into());
    assert!(matches!(
        w.append_intent(request(), "overflow", i.clone()),
        Err(LedgerError::Limit)
    ));
    drop(w);
    append_fixture(
        &mut bytes,
        MAX_ATTEMPTS as u64 + 1,
        "overflow",
        Event::Intent(i),
    );
    fs::write(root.ledger(), &bytes).unwrap();
    let r = read_ledger(root.ledger()).unwrap();
    assert_eq!(r.frames.len(), MAX_ATTEMPTS);
    assert_eq!(r.boundary.unwrap().kind, BoundaryKind::Limit);
    let mut bytes = b"BNDLGR01".to_vec();
    append_fixture(&mut bytes, 1, "a", Event::Intent(intent(&root)));
    append_fixture(
        &mut bytes,
        2,
        "a",
        Event::Observation(Observation::Spawned { pid: 1 }),
    );
    for seq in 3..=MAX_FRAMES as u64 {
        append_fixture(
            &mut bytes,
            seq,
            "a",
            Event::Observation(Observation::Fault {
                fault: FaultKind::Timeout,
                detail: "fixture".into(),
            }),
        );
    }
    fs::write(root.ledger(), &bytes).unwrap();
    let mut w = LedgerWriter::open(root.ledger()).unwrap();
    let mut i = intent(&root);
    i.predecessor = Some("a".into());
    assert!(matches!(
        w.append_intent(request(), "overflow", i),
        Err(LedgerError::Limit)
    ));
    drop(w);
    append_fixture(
        &mut bytes,
        MAX_FRAMES as u64 + 1,
        "a",
        Event::Observation(Observation::Fault {
            fault: FaultKind::Timeout,
            detail: "fixture".into(),
        }),
    );
    fs::write(root.ledger(), &bytes).unwrap();
    let r = read_ledger(root.ledger()).unwrap();
    assert_eq!(r.frames.len(), MAX_FRAMES);
    assert_eq!(r.boundary.unwrap().kind, BoundaryKind::Limit);
}
#[test]
fn total_byte_limit_preserves_bounded_prefix_and_refuses_append() {
    let root = Sandbox::new();
    let mut bytes = b"BNDLGR01".to_vec();
    let mut big = intent(&root);
    big.mandatory_files = (0..200)
        .map(|n| format!("f{n:03}{}", "x".repeat(300)))
        .collect();
    for n in 0..1000 {
        let mut i = big.clone();
        if n > 0 {
            i.predecessor = Some(format!("a{}", n - 1));
        }
        append_fixture(&mut bytes, n + 1, &format!("a{n}"), Event::Intent(i));
    }
    append_fixture(
        &mut bytes,
        1001,
        "a0",
        Event::Observation(Observation::Spawned { pid: 1 }),
    );
    let mut seq = 1002;
    loop {
        let before = bytes.len();
        append_fixture(
            &mut bytes,
            seq,
            "a0",
            Event::Observation(Observation::Fault {
                fault: FaultKind::Timeout,
                detail: "x".repeat(1024),
            }),
        );
        if bytes.len() as u64 > MAX_LEDGER_BYTES {
            fs::write(root.ledger(), &bytes[..before]).unwrap();
            let mut w = LedgerWriter::open(root.ledger()).unwrap();
            let mut i = big.clone();
            i.predecessor = Some("a0".into());
            assert!(matches!(
                w.append_intent(request(), "next", i),
                Err(LedgerError::Limit)
            ));
            drop(w);
            fs::write(root.ledger(), &bytes).unwrap();
            let r = read_ledger(root.ledger()).unwrap();
            assert_eq!(r.valid_bytes, before as u64);
            assert_eq!(r.boundary.unwrap().kind, BoundaryKind::Limit);
            assert!(matches!(
                LedgerWriter::open(root.ledger()),
                Err(LedgerError::Corrupt(_))
            ));
            break;
        }
        seq += 1;
        assert!(seq < MAX_FRAMES as u64);
    }
}
#[test]
fn incomplete_or_bad_publication_observation_grammar_cannot_advance() {
    let root = Sandbox::new();
    let mut w = LedgerWriter::open(root.ledger()).unwrap();
    drop(w.append_intent(request(), "a", intent(&root)).unwrap());
    assert!(w
        .append_observation("a", Observation::Exited { code: Some(0) })
        .is_err());
    assert!(w
        .append_publication(
            "a",
            Publication::Intent {
                operation: PublicationKind::OutputCapture
            }
        )
        .is_err());
    w.append_observation("a", Observation::Spawned { pid: 1 })
        .unwrap();
    assert!(w
        .append_observation("a", Observation::Spawned { pid: 2 })
        .is_err());
    assert!(w
        .append_observation(
            "a",
            Observation::SpawnFailed {
                code: "late_failure".into()
            }
        )
        .is_err());
    assert!(w
        .append_observation(
            "a",
            Observation::PipeEof {
                pipe: Pipe::Stdout,
                bytes: u64::MAX,
                sha256: "a".repeat(64)
            }
        )
        .is_err());
    assert!(w
        .append_observation(
            "a",
            Observation::Fault {
                fault: FaultKind::Unavailable,
                detail: "x".repeat(1025)
            }
        )
        .is_err());
    w.append_observation("a", Observation::Exited { code: Some(0) })
        .unwrap();
    w.append_observation(
        "a",
        Observation::PipeEof {
            pipe: Pipe::Stdout,
            bytes: 0,
            sha256: "a".repeat(64),
        },
    )
    .unwrap();
    assert!(w
        .append_publication(
            "a",
            Publication::Intent {
                operation: PublicationKind::OutputCapture
            }
        )
        .is_err());
    w.append_observation(
        "a",
        Observation::PipeEof {
            pipe: Pipe::Stderr,
            bytes: 0,
            sha256: "a".repeat(64),
        },
    )
    .unwrap();
    assert!(w
        .append_publication(
            "a",
            Publication::Intent {
                operation: PublicationKind::Admission
            }
        )
        .is_err());
    assert!(w
        .append_publication(
            "a",
            Publication::Returned {
                operation: PublicationKind::OutputCapture,
                references: vec![reference()],
                receipt: "no intent".into()
            }
        )
        .is_err());
    w.append_publication(
        "a",
        Publication::Intent {
            operation: PublicationKind::OutputCapture,
        },
    )
    .unwrap();
    assert!(w
        .append_publication(
            "a",
            Publication::Intent {
                operation: PublicationKind::ReviewResult
            }
        )
        .is_err());
    for case in 0..5 {
        let mut refs = vec![reference()];
        let mut receipt = "receipt".to_string();
        match case {
            0 => refs.clear(),
            1 => refs[0].registry_id = "2".repeat(64),
            2 => refs = vec![reference(); 9],
            3 => receipt = "x".repeat(4097),
            _ => refs[0].entry_hash = "invalid".into(),
        };
        assert!(w
            .append_publication(
                "a",
                Publication::Returned {
                    operation: PublicationKind::OutputCapture,
                    references: refs,
                    receipt
                }
            )
            .is_err());
    }
    w.append_publication(
        "a",
        Publication::Uncertain {
            operation: PublicationKind::OutputCapture,
            code: "return_lost".into(),
        },
    )
    .unwrap();
    assert!(w
        .append_publication(
            "a",
            Publication::Returned {
                operation: PublicationKind::OutputCapture,
                references: vec![reference()],
                receipt: "cannot resolve by assertion".into()
            }
        )
        .is_err());
}
#[cfg(windows)]
#[test]
fn held_windows_file_and_parent_cannot_be_replaced() {
    let root = Sandbox::new();
    let w = LedgerWriter::open(root.ledger()).unwrap();
    assert!(fs::rename(root.ledger(), root.0.join("moved")).is_err());
    assert!(fs::rename(&root.0, root.0.with_extension("moved")).is_err());
    let competitor = root.0.join("competitor");
    fs::write(&competitor, "competitor").unwrap();
    assert!(fs::rename(&competitor, root.ledger()).is_err());
    assert_eq!(fs::read(&competitor).unwrap(), b"competitor");
    drop(w);
}
#[cfg(any(windows, unix))]
#[test]
fn symlink_is_refused_without_following_target() {
    let root = Sandbox::new();
    let target = root.0.join("target");
    fs::write(&target, b"unchanged").unwrap();
    #[cfg(windows)]
    let result = std::os::windows::fs::symlink_file(&target, root.ledger());
    #[cfg(unix)]
    let result = std::os::unix::fs::symlink(&target, root.ledger());
    if let Err(e) = result {
        #[cfg(windows)]
        if e.raw_os_error() == Some(1314) {
            eprintln!("NOT_EXECUTED: symlink privilege unavailable");
            return;
        }
        panic!("symlink fixture failed: {e}");
    }
    assert!(LedgerWriter::open(root.ledger()).is_err());
    assert!(read_ledger(root.ledger()).is_err());
    assert_eq!(fs::read(target).unwrap(), b"unchanged");
}
fn obs(value: serde_json::Value) -> Observation {
    serde_json::from_value(value).unwrap()
}
#[test]
fn failure_retry_success_keeps_both_attempts() {
    let root = Sandbox::new();
    let mut w = LedgerWriter::open(root.ledger()).unwrap();
    drop(w.append_intent(request(), "failed", intent(&root)).unwrap());
    w.append_observation(
        "failed",
        obs(serde_json::json!({"kind":"SpawnFailed","code":"executable_missing"})),
    )
    .unwrap();
    let mut retry = intent(&root);
    retry.predecessor = Some("failed".into());
    drop(w.append_intent(request(), "retry", retry).unwrap());
    w.append_observation("retry", Observation::Spawned { pid: 42 })
        .unwrap();
    w.append_observation("retry", obs(serde_json::json!({"kind":"Exited","code":0})))
        .unwrap();
    for pipe in ["Stdout", "Stderr"] {
        w.append_observation("retry", obs(serde_json::json!({"kind":"PipeEof", "pipe":pipe,"bytes":0,"sha256":"a".repeat(64)}))).unwrap();
    }
    drop(w);
    let r = read_ledger(root.ledger()).unwrap();
    assert_eq!(r.frames.len(), 7);
    let a = r.attempts();
    assert_eq!(a.len(), 2);
    assert_eq!(a[0].attempt_id, "failed");
    assert_eq!(a[1].predecessor.as_deref(), Some("failed"));
    assert!(a.iter().all(|a| !a.live_witness));
}
#[test]
fn durable_intent_reopens_as_unresolved_without_live_permission() {
    let root = Sandbox::new();
    let mut writer = LedgerWriter::open(root.ledger()).unwrap();
    let permit = writer
        .append_intent(request(), "a1", intent(&root))
        .unwrap();
    assert_eq!(permit.attempt_id(), "a1");
    drop(permit);
    drop(writer);
    let report = read_ledger(root.ledger()).unwrap();
    assert!(report.boundary.is_none());
    assert_eq!(report.frames.len(), 1);
    let attempts = report.attempts();
    assert_eq!(attempts[0].launch, LaunchProjection::Unresolved);
    assert!(!attempts[0].live_witness);
    let mut reopened = LedgerWriter::open(root.ledger()).unwrap();
    assert!(reopened
        .append_observation("a1", Observation::Spawned { pid: 12 })
        .is_err());
    drop(reopened);
    assert_eq!(read_ledger(root.ledger()).unwrap().frames.len(), 1);
}
