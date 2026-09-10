//! Public-only Store provisioning and actual deterministic fake native reviewer.
//! No positive Record/Journal bytes are fabricated or directly written.
use ai_agent_evidence_binder::{
    bridge::{run_to_admission, OutputCapturePlan},
    ledger::{
        read_ledger, CommandLimits, DispatchIntent, Event, LedgerWriter, Publication,
        PublicationKind,
    },
    RequestBinding, Submission,
};
use evidence_registry::*;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
#[path = "bridge/matrix.rs"]
mod matrix;

#[test]
fn namespace_overlap_rejected_before_any_intent_or_spawn() {
    let mut f = Fixture::new();
    f.input(1);
    let declaration = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("out/ledger")).unwrap();
    let result = run_to_admission(
        &mut f.store,
        &mut writer,
        f.request.request_event_reference().clone(),
        "overlap",
        declaration,
        f.plan.clone(),
        &AtomicBool::new(false),
    );
    assert!(
        matches!(
            result,
            Err(ai_agent_evidence_binder::bridge::BridgeError::Preflight(
                "namespace_overlap"
            ))
        ),
        "expected predispatch overlap guard: {result:?}"
    );
    assert!(writer.report().frames.is_empty());
    assert!(!f.base.join("out/binder-stdout.bin").exists());
}

#[test]
fn fixed_outputs_cannot_be_waived_before_dispatch() {
    for omitted in ["submission.json", "binder-stdout.bin", "binder-stderr.bin"] {
        let mut f = Fixture::new();
        let mut declaration = f.intent("fake_reviewer");
        declaration.mandatory_files.retain(|name| name != omitted);
        // A bad declaration must fail before the otherwise failing spawn, not
        // consume an attempt or depend on the later Core output capture.
        declaration.command = vec![f.base.join("absent-agent.exe").to_str().unwrap().into()];
        let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
        let result = run_to_admission(
            &mut f.store,
            &mut writer,
            f.request.request_event_reference().clone(),
            "waived",
            declaration,
            f.plan.clone(),
            &AtomicBool::new(false),
        );
        assert!(
            matches!(
                result,
                Err(ai_agent_evidence_binder::bridge::BridgeError::Preflight(
                    "missing_fixed_declaration"
                ))
            ),
            "{omitted}: {result:?}"
        );
        assert!(writer.report().frames.is_empty());
        assert!(!f.base.join("out/binder-stdout.bin").exists());
        assert!(!f.base.join("out/binder-stderr.bin").exists());
    }
}

include!("bridge/fixture.rs");
#[test]
fn clean_syntax_claim_that_fails_selected_policy_is_completed_rejection() {
    let mut f = Fixture::new();
    f.input(2);
    let declaration = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let result = run_to_admission(
        &mut f.store,
        &mut writer,
        f.request.request_event_reference().clone(),
        "rejected",
        declaration,
        f.plan.clone(),
        &AtomicBool::new(false),
    )
    .unwrap();
    let AuthoritativeReviewAdmissionRuntimeOutcome::Published(p) = result.admission else {
        panic!("not a completed Store policy decision")
    };
    assert_eq!(p.journal_reference().event_type_id().value(), 303);
    assert_eq!(result.result.result().method_status(), 2);
    drop(writer);
    cold_child(&f.base);
}
#[test]
fn malformed_submission_is_retained_with_fault_and_no_result() {
    let mut f = Fixture::new();
    fs::write(f.base.join("out/fixture-input.json"), b"{malformed").unwrap();
    let declaration = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let result = run_to_admission(
        &mut f.store,
        &mut writer,
        f.request.request_event_reference().clone(),
        "malformed",
        declaration,
        f.plan.clone(),
        &AtomicBool::new(false),
    );
    assert!(
        matches!(
            result,
            Err(ai_agent_evidence_binder::bridge::BridgeError::Submission(_))
        ),
        "{result:?}"
    );
    assert_eq!(
        f.store
            .retained_journal()
            .references()
            .filter(|r| r.event_type_id().value() == 301)
            .count(),
        0
    );
    assert!(writer.report().frames.iter().any(|f| matches!(
        &f.event,
        Event::Observation(ai_agent_evidence_binder::ledger::Observation::Fault {
            fault: ai_agent_evidence_binder::ledger::FaultKind::InvalidSubmission,
            ..
        })
    )));
    let output = f
        .store
        .retained_journal()
        .references()
        .filter(|r| r.event_type_id().value() == 101)
        .last()
        .unwrap();
    let snapshot = f.store.read_selected_embedded_payload(output).unwrap();
    assert!(snapshot.artifact_bytes().iter().any(|b| b == b"{malformed"));
    drop(writer);
    cold_child(&f.base);
}
#[test]
fn declared_file_and_byte_limits_reach_bounded_store_intake() {
    for mode in ["files", "bytes"] {
        let mut f = Fixture::new();
        f.input(1);
        let mut declaration = f.intent("fake_reviewer");
        if mode == "files" {
            declaration.limits.output_files = 4;
        } else {
            declaration.limits.output_bytes = 1;
        }
        let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
        let result = run_to_admission(
            &mut f.store,
            &mut writer,
            f.request.request_event_reference().clone(),
            mode,
            declaration,
            f.plan.clone(),
            &AtomicBool::new(false),
        );
        assert!(
            matches!(
                result,
                Err(ai_agent_evidence_binder::bridge::BridgeError::Preparation(
                    SelectedEmbeddedFreezePreparationError::SourceResourceLimit
                ))
            ),
            "{mode}: {result:?}"
        );
        assert_eq!(
            f.store.retained_journal().current_head_reference(),
            *f.request.request_event_reference()
        );
        assert!(writer.report().frames.iter().any(|f| matches!(
            &f.event,
            Event::Publication(Publication::Uncertain {
                operation: PublicationKind::OutputPreparation,
                ..
            })
        )));
        drop(writer);
        cold_child(&f.base);
    }
}
#[test]
fn missing_mandatory_artifact_is_not_silently_dropped() {
    let mut f = Fixture::new();
    f.input(1);
    let mut declaration = f.intent("fake_reviewer");
    declaration.mandatory_files.push("z-required.txt".into());
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let result = run_to_admission(
        &mut f.store,
        &mut writer,
        f.request.request_event_reference().clone(),
        "missing",
        declaration,
        f.plan.clone(),
        &AtomicBool::new(false),
    );
    assert!(
        matches!(
            result,
            Err(ai_agent_evidence_binder::bridge::BridgeError::CaptureSet(
                "missing_mandatory"
            ))
        ),
        "{result:?}"
    );
    assert_eq!(
        f.store
            .retained_journal()
            .references()
            .filter(|r| r.event_type_id().value() == 301)
            .count(),
        0
    );
    assert!(writer.report().frames.iter().any(|f|matches!(&f.event,Event::Observation(ai_agent_evidence_binder::ledger::Observation::Fault{fault:ai_agent_evidence_binder::ledger::FaultKind::CaptureFailed,detail}) if detail=="missing_mandatory")));
    assert_captured_bytes(&f, &fs::read(f.base.join("out/submission.json")).unwrap());
    assert_no_result_or_admission(&f);
    drop(writer);
    cold_child(&f.base);
}

#[test]
fn candidate_must_be_a_member_of_store_captured_manifest() {
    let mut f = Fixture::new();
    f.input(1);
    let input = f.base.join("out/fixture-input.json");
    let mut submission: Submission = serde_json::from_slice(&fs::read(&input).unwrap()).unwrap();
    submission.artifacts.push("not-produced.txt".into());
    fs::write(&input, serde_json::to_vec(&submission).unwrap()).unwrap();
    let declaration = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let result = run_to_admission(
        &mut f.store,
        &mut writer,
        f.request.request_event_reference().clone(),
        "candidate-missing",
        declaration,
        f.plan.clone(),
        &AtomicBool::new(false),
    );
    assert!(
        matches!(
            result,
            Err(ai_agent_evidence_binder::bridge::BridgeError::CaptureSet(
                "candidate_not_captured"
            ))
        ),
        "{result:?}"
    );
    assert_no_result_or_admission(&f);
    assert_fault(
        &writer,
        ai_agent_evidence_binder::ledger::FaultKind::InvalidSubmission,
        "candidate_not_captured",
    );
    assert_captured_bytes(&f, &fs::read(input).unwrap());
    drop(writer);
    cold_child(&f.base);
}

fn assert_no_result_or_admission(f: &Fixture) {
    assert_eq!(
        f.store
            .retained_journal()
            .references()
            .filter(|r| matches!(r.event_type_id().value(), 301..=303))
            .count(),
        0
    );
}
fn assert_fault(
    writer: &LedgerWriter,
    expected: ai_agent_evidence_binder::ledger::FaultKind,
    code: &str,
) {
    assert!(writer.report().frames.iter().any(|frame| matches!(&frame.event, Event::Observation(ai_agent_evidence_binder::ledger::Observation::Fault { fault, detail }) if *fault == expected && detail == code)));
}
fn assert_captured_bytes(f: &Fixture, bytes: &[u8]) {
    let output = f
        .store
        .retained_journal()
        .references()
        .filter(|r| r.event_type_id().value() == 101)
        .last()
        .unwrap();
    let snapshot = f.store.read_selected_embedded_payload(output).unwrap();
    assert!(snapshot.artifact_bytes().iter().any(|b| b == bytes));
}

#[test]
fn invalid_retained_request_never_creates_managed_attempt() {
    let mut f = Fixture::new();
    f.input(1);
    let declaration = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let wrong = f.store.retained_journal().references().next().unwrap();
    let result = run_to_admission(
        &mut f.store,
        &mut writer,
        wrong,
        "wrong",
        declaration,
        f.plan.clone(),
        &AtomicBool::new(false),
    );
    assert!(
        matches!(
            result,
            Err(ai_agent_evidence_binder::bridge::BridgeError::Request(_))
        ),
        "{result:?}"
    );
    assert!(writer.report().frames.is_empty());
    assert!(!f.base.join("out/binder-stdout.bin").exists());
}

// A new native process resolves the exact ledger-returned references from Store;
// it neither uses the producer's objects nor resumes any attempt.
#[test]
#[ignore = "cold verifier subprocess entry, launched by bridge tests"]
fn cold_verifier() {
    let report = read_ledger("ledger").unwrap();
    assert!(report.boundary.is_none());
    assert!(report.attempts().iter().all(|a| !a.live_witness));
    let mut store = AuthoritativeRegistryStore::open_selected_profile("store").unwrap();
    let head = store.retained_journal().current_head_reference();
    for a in report.attempts() {
        let q = store
            .retained_journal()
            .references()
            .find(|r| binding(r) == a.request)
            .expect("exact Q");
        store.validate_selected_review_request(q).unwrap();
        for p in a.publications {
            for r in p.references {
                let reference = store
                    .retained_journal()
                    .references()
                    .find(|s| {
                        let b = binding(s);
                        b.registry_id == r.registry_id
                            && b.entry_index == r.entry_index
                            && b.entry_hash == r.entry_hash
                            && b.event_type_id == r.event_type_id
                            && b.event_record_id == r.event_record_id
                    })
                    .expect("exact returned reference, not latest candidate");
                match p.operation {
                    PublicationKind::OutputPreparation => {
                        assert_eq!(reference.event_type_id().value(), 100)
                    }
                    PublicationKind::OutputCapture => {
                        store.read_selected_embedded_payload(reference).unwrap();
                    }
                    PublicationKind::ReviewResult => {
                        store
                            .validate_selected_review_result(reference.clone())
                            .unwrap();
                        store
                            .derive_selected_review_admission_section_82(reference)
                            .unwrap();
                    }
                    PublicationKind::Admission => {
                        assert!(matches!(reference.event_type_id().value(), 302 | 303))
                    }
                }
            }
        }
    }
    let inspection = ai_agent_evidence_binder::inspection::inspect("store", "ledger").unwrap();
    assert!(inspection.read_only);
    assert_eq!(inspection.counts.attempts, report.attempts().len());
    assert!(inspection
        .attempts
        .iter()
        .all(|a| !a.live_witness && !a.historical_receipt_recovered));
    if let Ok(bytes) = fs::read("expected-inspection.json") {
        let expected: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let actual = serde_json::to_value(&inspection).unwrap();
        for (key, value) in expected.as_object().unwrap() {
            assert_eq!(&actual["counts"][key], value, "cold count {key}");
        }
    }
    println!(
        "COLD_INSPECTION={}",
        serde_json::to_string(&inspection).unwrap()
    );
    assert_eq!(head, store.retained_journal().current_head_reference());
    println!("COLD_EXACT_REFERENCES_VERIFIED_NO_RESUME");
}
fn cold_child(base: &std::path::Path) {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "cold_verifier", "--nocapture"])
        .current_dir(base)
        .output()
        .unwrap();
    fs::write(base.join("cold-stdout.bin"), &output.stdout).unwrap();
    fs::write(base.join("cold-stderr.bin"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "cold child: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("COLD_EXACT_REFERENCES_VERIFIED_NO_RESUME"));
}
#[test]
#[ignore = "fake native subprocess entry; actually invoked by integration tests, not AI"]
fn fake_reviewer() {
    fs::write("submission.json", fs::read("fixture-input.json").unwrap()).unwrap();
    fs::write(
        "review.txt",
        b"CLEAN is only this fake reviewer's untrusted claim\n",
    )
    .unwrap();
    println!("fake reviewer stdout; subagent claims are not direct observations");
    eprintln!("fake reviewer stderr");
}
#[test]
fn real_process_separate_capture_exact_result_admission_and_cold_reopen() {
    let mut f = Fixture::new();
    f.input(1);
    let intent = f.intent("fake_reviewer");
    let mut writer = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let outcome = run_to_admission(
        &mut f.store,
        &mut writer,
        f.request.request_event_reference().clone(),
        "first",
        intent,
        f.plan.clone(),
        &AtomicBool::new(false),
    )
    .expect("complete managed Store bridge");
    assert_ne!(
        outcome.output_freeze.committed_event_reference(),
        f.request.freeze_authority().committed_event_reference()
    );
    assert_eq!(outcome.result.request(), &f.request);
    assert_eq!(
        outcome.result.result().freeze_authority_ref(),
        f.request.request().freeze_authority_ref()
    );
    assert_eq!(
        outcome.result.result().manifest_id(),
        f.request.request().manifest_id()
    );
    assert_eq!(
        outcome.result.result().review_package_anchor_id(),
        f.request.request().review_package_anchor_id()
    );
    let AuthoritativeReviewAdmissionRuntimeOutcome::Published(publication) = &outcome.admission
    else {
        panic!("expected Store publication: {:?}", outcome.admission)
    };
    assert_eq!(publication.journal_reference().event_type_id().value(), 302);
    let captured = f
        .store
        .read_selected_embedded_payload(outcome.output_freeze.committed_event_reference().clone())
        .unwrap();
    assert_eq!(captured.manifest().input().artifact_count, 5);
    let operations: Vec<_> = writer
        .report()
        .frames
        .iter()
        .filter_map(|f| match &f.event {
            Event::Publication(Publication::Returned { operation, .. }) => Some(*operation),
            _ => None,
        })
        .collect();
    assert_eq!(
        operations,
        [
            PublicationKind::OutputPreparation,
            PublicationKind::OutputCapture,
            PublicationKind::ReviewResult,
            PublicationKind::Admission
        ]
    );
    let admission_ref = publication.journal_reference().clone();
    drop(writer);
    assert_eq!(
        read_ledger(f.base.join("ledger")).unwrap().attempts().len(),
        1
    );
    let cold = AuthoritativeRegistryStore::open_selected_profile(f.base.join("store")).unwrap();
    assert_eq!(
        cold.validate_selected_review_result(outcome.result.result_event_reference().clone())
            .unwrap(),
        outcome.result
    );
    assert_eq!(
        cold.read_selected_embedded_payload(
            outcome.output_freeze.committed_event_reference().clone()
        )
        .unwrap()
        .artifact_bytes(),
        captured.artifact_bytes()
    );
    assert_eq!(
        cold.retained_journal().current_head_reference(),
        admission_ref
    );
    drop(cold);
    cold_child(&f.base);
}
