//! Real public-only Store producers; ledger observations below are SYNTHETIC
//! historical-projection fixtures, not evidence of managed reviewer execution.
use ai_agent_evidence_binder::{
    inspection::inspect,
    ledger::{CommandLimits, DispatchIntent, LedgerWriter},
    RequestBinding,
};
use evidence_registry::*;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn binding(r: &JournalReference) -> RequestBinding {
    RequestBinding {
        registry_id: hex(r.registry_id().as_bytes()),
        entry_index: r.entry_index().value(),
        entry_hash: hex(r.entry_hash().as_bytes()),
        event_type_id: r.event_type_id().value().into(),
        event_record_id: hex(r.event_record_id().as_bytes()),
    }
}
fn record_id(r: &JournalReference) -> RecordId {
    RecordId::try_from(r.event_record_id().as_bytes().as_slice()).unwrap()
}
struct Fixture {
    base: PathBuf,
    store: AuthoritativeRegistryStore,
    request: RecordedSelectedReviewRequest,
    freeze_policy: RecordId,
    target_payload: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let base = std::env::current_dir()
            .unwrap()
            .join("../target")
            .join(format!(
                "binder-inspection-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&base).unwrap();
        let base = fs::canonicalize(base).unwrap();
        fs::create_dir(base.join("source")).unwrap();
        fs::create_dir(base.join("out")).unwrap();
        fs::write(
            base.join("source/target.txt"),
            b"public Store fixture; no real AI review",
        )
        .unwrap();
        let mut store = AuthoritativeRegistryStore::initialize_selected_profile(
            base.join("store"),
            RegistryId::try_from([71; 32].as_slice()).unwrap(),
            "inspection-synthetic-history",
        )
        .unwrap();
        let scope = |p| {
            ScopeRecord::new(ScopeRecordInput {
                scope_profile_id: p,
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
        let freeze_policy = store
            .register_selected_minimal_policy(freeze_scope.record_id())
            .unwrap();
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
        let prepared = store
            .prepare_selected_embedded_freeze(SelectedEmbeddedFreezePreparationInput {
                source_root: base.join("source"),
                freeze_attempt_id: FreezeAttemptId::try_from([72; 32].as_slice()).unwrap(),
                policy_record_id: record_id(&freeze_policy),
            })
            .unwrap();
        let target_payload = prepared.payload_directory().to_owned();
        let target = store
            .commit_prepared_selected_embedded_freeze(prepared)
            .unwrap();
        let request = store
            .record_selected_review_request(SelectedReviewRequestInput {
                freeze_authority: target,
                review_policy_record_id: record_id(&review_policy),
                review_role_id: 1,
            })
            .unwrap();
        Self {
            base,
            store,
            request,
            freeze_policy: record_id(&freeze_policy),
            target_payload,
        }
    }
    fn intent(&self, predecessor: Option<&str>) -> DispatchIntent {
        DispatchIntent {
            approved_output_root: self.base.join("out").to_str().unwrap().into(),
            mandatory_files: vec!["submission.json".into()],
            command: vec!["SYNTHETIC-NOT-SPAWNED".into()],
            binder_identity: "synthetic-history-fixture-not-execution".into(),
            limits: CommandLimits::default(),
            predecessor: predecessor.map(str::to_owned),
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

fn synthetic_completion(f: &Fixture, w: &mut LedgerWriter, id: &str, predecessor: Option<&str>) {
    use ai_agent_evidence_binder::ledger::{Observation, Pipe};
    w.append_intent(
        binding(f.request.request_event_reference()),
        id,
        f.intent(predecessor),
    )
    .unwrap()
    .consume();
    w.append_observation(id, Observation::Spawned { pid: 1 })
        .unwrap();
    w.append_observation(id, Observation::Exited { code: Some(0) })
        .unwrap();
    for pipe in [Pipe::Stdout, Pipe::Stderr] {
        w.append_observation(
            id,
            Observation::PipeEof {
                pipe,
                bytes: 0,
                sha256: "00".repeat(32),
            },
        )
        .unwrap();
    }
}
fn returned(
    w: &mut LedgerWriter,
    id: &str,
    op: ai_agent_evidence_binder::ledger::PublicationKind,
    r: &JournalReference,
) {
    use ai_agent_evidence_binder::ledger::{Publication, RetainedReference};
    let b = binding(r);
    w.append_publication(id, Publication::Intent { operation: op })
        .unwrap();
    w.append_publication(
        id,
        Publication::Returned {
            operation: op,
            references: vec![RetainedReference {
                registry_id: b.registry_id,
                entry_index: b.entry_index,
                entry_hash: b.entry_hash,
                event_type_id: b.event_type_id,
                event_record_id: b.event_record_id,
            }],
            receipt: "synthetic historical observation, not execution or live receipt".into(),
        },
    )
    .unwrap();
}
fn store_publications(f: &mut Fixture, status: u64, nonce: u8) -> Vec<JournalReference> {
    fs::write(
        f.base.join("out/submission.json"),
        b"opaque fake-reviewer output fixture",
    )
    .unwrap();
    let prepared = f
        .store
        .prepare_selected_embedded_freeze(SelectedEmbeddedFreezePreparationInput {
            source_root: f.base.join("out"),
            freeze_attempt_id: FreezeAttemptId::try_from([nonce; 32].as_slice()).unwrap(),
            policy_record_id: f.freeze_policy,
        })
        .unwrap();
    let start = prepared.start_event_reference().clone();
    let output = f
        .store
        .commit_prepared_selected_embedded_freeze(prepared)
        .unwrap();
    let result = f
        .store
        .record_selected_review_result(SelectedReviewResultInput {
            request_event_reference: f.request.request_event_reference().clone(),
            method_status: status,
            finding_state: 1,
            reason_codes: vec![],
            findings: vec![],
            reviewer_metadata: Some("FAKE reviewer; synthetic ledger history; not AI".into()),
        })
        .unwrap();
    let admission = f
        .store
        .complete_selected_review_admission(result.result_event_reference().clone())
        .unwrap();
    let AuthoritativeReviewAdmissionRuntimeOutcome::Published(admission) = admission else {
        panic!("fixture Store publication unavailable: {admission:?}")
    };
    vec![
        start,
        output.committed_event_reference().clone(),
        result.result_event_reference().clone(),
        admission.journal_reference().clone(),
    ]
}
#[test]
fn all_returned_publications_resolve_but_never_recover_live_success_or_receipts() {
    use ai_agent_evidence_binder::ledger::PublicationKind::*;
    let mut f = Fixture::new();
    let mut w = LedgerWriter::open(f.base.join("ledger")).unwrap();
    synthetic_completion(&f, &mut w, "accepted", None);
    let refs = store_publications(&mut f, 1, 73);
    for (op, r) in [OutputPreparation, OutputCapture, ReviewResult, Admission]
        .into_iter()
        .zip(&refs)
    {
        returned(&mut w, "accepted", op, r);
    }
    synthetic_completion(&f, &mut w, "rejected", Some("accepted"));
    let rejected = store_publications(&mut f, 2, 74);
    for (op, r) in [OutputPreparation, OutputCapture, ReviewResult, Admission]
        .into_iter()
        .zip(&rejected)
    {
        returned(&mut w, "rejected", op, r);
    }
    drop(w);
    let j = serde_json::to_value(inspect(f.base.join("store"), f.base.join("ledger")).unwrap())
        .unwrap();
    assert_eq!(j["counts"]["attempts"], 2);
    assert_eq!(j["counts"]["store_admissions_accepted"], 1);
    assert_eq!(j["counts"]["store_admissions_rejected"], 1);
    assert_eq!(j["counts"]["store_results"], 2);
    assert_eq!(j["unmatched_store_results"], serde_json::json!([]));
    for a in j["attempts"].as_array().unwrap() {
        assert_eq!(a["status"], "HistoricalReferencesValidated");
        assert_eq!(a["live_witness"], false);
        assert_eq!(a["historical_receipt_recovered"], false);
        for p in a["publications"].as_array().unwrap() {
            assert_eq!(p["ledger_status"], "KnownReferences");
            assert_eq!(p["references"][0]["validation"]["status"], "Validated");
        }
    }
    let events = j["store"]["events"].as_array().unwrap();
    for r in refs.iter().chain(&rejected) {
        assert!(events
            .iter()
            .any(|e| e["reference"] == serde_json::to_value(binding(r)).unwrap()));
    }
}

#[test]
fn returned_start_is_resolved_without_claiming_committed_capture() {
    use ai_agent_evidence_binder::ledger::PublicationKind;
    let mut f = Fixture::new();
    let mut w = LedgerWriter::open(f.base.join("ledger")).unwrap();
    synthetic_completion(&f, &mut w, "prepared", None);
    fs::write(f.base.join("out/submission.json"), b"fake output").unwrap();
    let prepared = f
        .store
        .prepare_selected_embedded_freeze(SelectedEmbeddedFreezePreparationInput {
            source_root: f.base.join("out"),
            freeze_attempt_id: FreezeAttemptId::try_from([77; 32].as_slice()).unwrap(),
            policy_record_id: f.freeze_policy,
        })
        .unwrap();
    returned(
        &mut w,
        "prepared",
        PublicationKind::OutputPreparation,
        prepared.start_event_reference(),
    );
    drop(w);
    let j = serde_json::to_value(inspect(f.base.join("store"), f.base.join("ledger")).unwrap())
        .unwrap();
    assert_eq!(
        j["attempts"][0]["publications"][0]["references"][0]["validation"]["status"],
        "Validated"
    );
    assert_eq!(j["attempts"][0]["status"], "Unresolved");
    assert_eq!(j["attempts"][0]["historical_receipt_recovered"], false);
}

#[test]
fn uncertain_preparation_keeps_every_possible_start_without_attribution_or_repair() {
    use ai_agent_evidence_binder::ledger::{Publication, PublicationKind};
    let mut f = Fixture::new();
    let mut w = LedgerWriter::open(f.base.join("ledger")).unwrap();
    synthetic_completion(&f, &mut w, "uncertain", None);
    w.append_publication(
        "uncertain",
        Publication::Intent {
            operation: PublicationKind::OutputPreparation,
        },
    )
    .unwrap();
    w.append_publication(
        "uncertain",
        Publication::Uncertain {
            operation: PublicationKind::OutputPreparation,
            code: "lost_return".into(),
        },
    )
    .unwrap();
    let mut starts = vec![];
    fs::write(f.base.join("out/submission.json"), b"opaque fixture").unwrap();
    for nonce in [78, 79] {
        fs::write(f.base.join("out/submission.json"), [nonce]).unwrap();
        let p = f
            .store
            .prepare_selected_embedded_freeze(SelectedEmbeddedFreezePreparationInput {
                source_root: f.base.join("out"),
                freeze_attempt_id: FreezeAttemptId::try_from([nonce; 32].as_slice()).unwrap(),
                policy_record_id: f.freeze_policy,
            })
            .unwrap();
        starts.push(binding(p.start_event_reference()));
    }
    drop(w);
    let before = fs::read(f.base.join("ledger")).unwrap();
    let j = serde_json::to_value(inspect(f.base.join("store"), f.base.join("ledger")).unwrap())
        .unwrap();
    let p = &j["attempts"][0]["publications"][0];
    assert_eq!(p["ledger_status"], "Uncertain");
    assert_eq!(p["uncertainty_code"], "lost_return");
    assert_eq!(p["candidates"], serde_json::to_value(starts).unwrap());
    assert_eq!(p["candidate_count"], 2);
    assert_eq!(p["candidate_attribution_established"], false);
    assert_eq!(j["attempts"][0]["status"], "Unresolved");
    assert_eq!(fs::read(f.base.join("ledger")).unwrap(), before);
}

#[test]
fn corrupt_tail_keeps_prefix_boundary_and_exact_request_mismatch_without_mutation() {
    let f = Fixture::new();
    let mut w = LedgerWriter::open(f.base.join("ledger")).unwrap();
    let mut wrong = binding(f.request.request_event_reference());
    wrong.entry_hash = "ab".repeat(32);
    w.append_intent(wrong, "mismatch", f.intent(None))
        .unwrap()
        .consume();
    drop(w);
    let good = fs::read(f.base.join("ledger")).unwrap();
    let before_report =
        serde_json::to_value(inspect(f.base.join("store"), f.base.join("ledger")).unwrap())
            .unwrap();
    assert_eq!(before_report["attempts"][0]["status"], "ReferenceMismatch");
    assert_eq!(
        before_report["attempts"][0]["request_validation"]["status"],
        "Invalid"
    );
    let mut torn = good.clone();
    torn.extend_from_slice(&[1, 2]);
    fs::write(f.base.join("ledger"), &torn).unwrap();
    let j = serde_json::to_value(inspect(f.base.join("store"), f.base.join("ledger")).unwrap())
        .unwrap();
    assert_eq!(j["ledger"]["status"], "CorruptPrefix");
    assert_eq!(j["ledger"]["boundary"]["offset"], good.len());
    assert_eq!(j["ledger"]["boundary"]["kind"], "Torn");
    assert_eq!(j["counts"]["attempts"], 1);
    assert_eq!(j["attempts"][0]["ledger_history_complete"], false);
    assert_eq!(j["attempts"][0]["historical_receipt_recovered"], false);
    assert_eq!(fs::read(f.base.join("ledger")).unwrap(), torn);
}

fn result_only(f: &mut Fixture, status: u64) -> JournalReference {
    f.store
        .record_selected_review_result(SelectedReviewResultInput {
            request_event_reference: f.request.request_event_reference().clone(),
            method_status: status,
            finding_state: 1,
            reason_codes: vec![],
            findings: vec![],
            reviewer_metadata: Some("FAKE reviewer; synthetic ledger history; not AI".into()),
        })
        .unwrap()
        .result_event_reference()
        .clone()
}
#[test]
fn matching_result_claims_are_not_independent_reviews_and_unmatched_results_stay_visible() {
    let mut f = Fixture::new();
    let r1 = result_only(&mut f, 1);
    let r2 = result_only(&mut f, 1);
    let r3 = result_only(&mut f, 2);
    assert_ne!(r1, r2, "Store chronology keeps deliveries distinct");
    drop(LedgerWriter::open(f.base.join("ledger")).unwrap());
    let j = serde_json::to_value(inspect(f.base.join("store"), f.base.join("ledger")).unwrap())
        .unwrap();
    assert_eq!(j["counts"]["store_results"], 3);
    assert_eq!(j["counts"]["independent_reviews_established"], 0);
    assert_eq!(j["counts"]["distinct_result_claim_sets"], 2);
    assert_eq!(j["counts"]["matching_result_claim_groups"], 1);
    assert_eq!(
        j["matching_result_claims"][0]["references"],
        serde_json::to_value(vec![binding(&r1), binding(&r2)]).unwrap()
    );
    assert_eq!(
        j["unmatched_store_results"],
        serde_json::to_value(vec![binding(&r1), binding(&r2), binding(&r3)]).unwrap()
    );
}

#[test]
fn returned_admission_must_match_exact_ledger_result_not_just_same_request() {
    use ai_agent_evidence_binder::ledger::PublicationKind::*;
    let mut f = Fixture::new();
    let first = result_only(&mut f, 1);
    let published = f.store.complete_selected_review_admission(first).unwrap();
    let AuthoritativeReviewAdmissionRuntimeOutcome::Published(published) = published else {
        panic!("expected actual Store fixture publication")
    };
    let second = result_only(&mut f, 1);
    let mut w = LedgerWriter::open(f.base.join("ledger")).unwrap();
    synthetic_completion(&f, &mut w, "mismatched_join", None);
    // Explicit negative ledger fixture: the old target is NOT an output capture.
    returned(
        &mut w,
        "mismatched_join",
        OutputCapture,
        f.request.freeze_authority().committed_event_reference(),
    );
    returned(&mut w, "mismatched_join", ReviewResult, &second);
    returned(
        &mut w,
        "mismatched_join",
        Admission,
        published.journal_reference(),
    );
    drop(w);
    let j = serde_json::to_value(inspect(f.base.join("store"), f.base.join("ledger")).unwrap())
        .unwrap();
    assert_eq!(j["attempts"][0]["status"], "ReferenceMismatch");
    assert_eq!(
        j["attempts"][0]["publications"][2]["references"][0]["validation"]["error"],
        "AdmissionResultMismatch"
    );
}

fn snapshot(root: &std::path::Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    fn visit(
        root: &std::path::Path,
        dir: &std::path::Path,
        out: &mut std::collections::BTreeMap<PathBuf, Vec<u8>>,
    ) {
        for item in fs::read_dir(dir).unwrap() {
            let item = item.unwrap();
            let path = item.path();
            if item.file_type().unwrap().is_dir() {
                visit(root, &path, out);
            } else {
                assert!(item.file_type().unwrap().is_file());
                out.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut out = std::collections::BTreeMap::new();
    visit(root, root, &mut out);
    out
}
#[test]
fn current_payload_corruption_is_visible_and_inspection_changes_no_store_bytes() {
    let f = Fixture::new();
    let mut w = LedgerWriter::open(f.base.join("ledger")).unwrap();
    w.append_intent(
        binding(f.request.request_event_reference()),
        "payload",
        f.intent(None),
    )
    .unwrap()
    .consume();
    drop(w);
    fs::write(
        f.target_payload.join("target.txt"),
        b"intentionally corrupt retained fixture bytes",
    )
    .unwrap();
    let before = snapshot(&f.base);
    let j = serde_json::to_value(inspect(f.base.join("store"), f.base.join("ledger")).unwrap())
        .unwrap();
    assert_eq!(j["store"]["status"], "CapturedViewWithValidationFailures");
    assert_eq!(j["attempts"][0]["request_validation"]["status"], "Invalid");
    assert_eq!(j["attempts"][0]["status"], "ReferenceMismatch");
    assert_eq!(snapshot(&f.base), before);
}
#[test]
fn store_unavailable_does_not_discard_ledger_attempts() {
    let f = Fixture::new();
    let mut w = LedgerWriter::open(f.base.join("ledger")).unwrap();
    w.append_intent(
        binding(f.request.request_event_reference()),
        "missing_store",
        f.intent(None),
    )
    .unwrap()
    .consume();
    drop(w);
    let j =
        serde_json::to_value(inspect(f.base.join("absent-store"), f.base.join("ledger")).unwrap())
            .unwrap();
    assert_eq!(j["store"]["status"], "Unavailable");
    assert_eq!(j["counts"]["attempts"], 1);
    assert_eq!(j["attempts"][0]["status"], "StoreUnavailable");
    assert_eq!(
        j["attempts"][0]["request_validation"]["status"],
        "Unavailable"
    );
    assert!(!f.base.join("absent-store").exists());
}

#[test]
fn uncertain_result_candidates_preserve_all_exact_instances_even_when_payload_invalid() {
    use ai_agent_evidence_binder::ledger::{Publication, PublicationKind};
    let mut f = Fixture::new();
    let r1 = result_only(&mut f, 1);
    let r2 = result_only(&mut f, 1);
    let mut w = LedgerWriter::open(f.base.join("ledger")).unwrap();
    synthetic_completion(&f, &mut w, "uncertain_results", None);
    // Negative ledger-only binding; not an output capture or execution fixture.
    returned(
        &mut w,
        "uncertain_results",
        PublicationKind::OutputCapture,
        f.request.freeze_authority().committed_event_reference(),
    );
    w.append_publication(
        "uncertain_results",
        Publication::Intent {
            operation: PublicationKind::ReviewResult,
        },
    )
    .unwrap();
    w.append_publication(
        "uncertain_results",
        Publication::Uncertain {
            operation: PublicationKind::ReviewResult,
            code: "lost_result_return".into(),
        },
    )
    .unwrap();
    drop(w);
    fs::write(
        f.target_payload.join("target.txt"),
        b"intentional negative payload",
    )
    .unwrap();
    let j = serde_json::to_value(inspect(f.base.join("store"), f.base.join("ledger")).unwrap())
        .unwrap();
    assert_eq!(
        j["attempts"][0]["publications"][1]["candidates"],
        serde_json::to_value(vec![binding(&r1), binding(&r2)]).unwrap()
    );
    assert_eq!(
        j["attempts"][0]["publications"][1]["ledger_status"],
        "Uncertain"
    );
    assert_eq!(j["counts"]["store_results"], 2);
    assert_eq!(j["unmatched_store_results"].as_array().unwrap().len(), 2);
    for e in j["store"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["reference"]["event_type_id"] == 301)
    {
        assert_eq!(e["validation"]["status"], "Invalid");
    }
}

#[test]
fn unresolved_admission_exposes_exact_request_candidates_and_all_admissions() {
    use ai_agent_evidence_binder::ledger::{Publication, PublicationKind};
    let mut f = Fixture::new();
    let r1 = result_only(&mut f, 1);
    let a1 = f
        .store
        .complete_selected_review_admission(r1.clone())
        .unwrap();
    let AuthoritativeReviewAdmissionRuntimeOutcome::Published(a1) = a1 else {
        panic!("expected first Store publication")
    };
    let original_request = f.request.clone();
    f.request = f
        .store
        .record_selected_review_request(SelectedReviewRequestInput {
            freeze_authority: f.request.freeze_authority().clone(),
            review_policy_record_id: record_id(f.request.policy_authority_ref()),
            review_role_id: 1,
        })
        .unwrap();
    let r2 = result_only(&mut f, 2);
    let a2 = f.store.complete_selected_review_admission(r2).unwrap();
    f.request = original_request;
    let AuthoritativeReviewAdmissionRuntimeOutcome::Published(a2) = a2 else {
        panic!("expected second Store publication")
    };
    assert_eq!(a1.journal_reference().event_type_id().value(), 302);
    assert_eq!(a2.journal_reference().event_type_id().value(), 303);
    let mut w = LedgerWriter::open(f.base.join("ledger")).unwrap();
    synthetic_completion(&f, &mut w, "pending_admission", None);
    // Negative historical local join, never execution or valid output capture.
    returned(
        &mut w,
        "pending_admission",
        PublicationKind::OutputCapture,
        f.request.freeze_authority().committed_event_reference(),
    );
    returned(
        &mut w,
        "pending_admission",
        PublicationKind::ReviewResult,
        &r1,
    );
    w.append_publication(
        "pending_admission",
        Publication::Intent {
            operation: PublicationKind::Admission,
        },
    )
    .unwrap();
    drop(w);
    let before = snapshot(&f.base);
    let j = serde_json::to_value(inspect(f.base.join("store"), f.base.join("ledger")).unwrap())
        .unwrap();
    assert_eq!(j["counts"]["store_admissions_accepted"], 1);
    assert_eq!(j["counts"]["store_admissions_rejected"], 1);
    let p = &j["attempts"][0]["publications"][2];
    assert_eq!(p["ledger_status"], "Unresolved");
    assert_eq!(
        p["candidates"],
        serde_json::to_value(vec![binding(a1.journal_reference())]).unwrap()
    );
    assert_eq!(p["candidate_count"], 1);
    assert_eq!(j["attempts"][0]["historical_receipt_recovered"], false);
    assert_eq!(snapshot(&f.base), before);
}
#[test]
fn a_fault_after_returned_preparation_preserves_both_observations() {
    use ai_agent_evidence_binder::ledger::{FaultKind, Observation, PublicationKind};
    let mut f = Fixture::new();
    let mut w = LedgerWriter::open(f.base.join("ledger")).unwrap();
    synthetic_completion(&f, &mut w, "faulted", None);
    fs::write(f.base.join("out/submission.json"), b"fake output").unwrap();
    let prepared = f
        .store
        .prepare_selected_embedded_freeze(SelectedEmbeddedFreezePreparationInput {
            source_root: f.base.join("out"),
            freeze_attempt_id: FreezeAttemptId::try_from([80; 32].as_slice()).unwrap(),
            policy_record_id: f.freeze_policy,
        })
        .unwrap();
    returned(
        &mut w,
        "faulted",
        PublicationKind::OutputPreparation,
        prepared.start_event_reference(),
    );
    w.append_observation(
        "faulted",
        Observation::Fault {
            fault: FaultKind::InvalidSubmission,
            detail: "synthetic fault after retained start".into(),
        },
    )
    .unwrap();
    drop(w);
    let before = snapshot(&f.base);
    let j = serde_json::to_value(inspect(f.base.join("store"), f.base.join("ledger")).unwrap())
        .unwrap();
    assert_eq!(j["attempts"][0]["status"], "ObservedFailure");
    assert_eq!(
        j["attempts"][0]["publications"][0]["ledger_status"],
        "KnownReferences"
    );
    assert_eq!(
        j["attempts"][0]["publications"][0]["references"][0]["reference"],
        serde_json::to_value(binding(prepared.start_event_reference())).unwrap()
    );
    assert_eq!(
        j["ledger"]["frames"].as_array().unwrap().last().unwrap()["event"]["payload"]["fault"],
        "InvalidSubmission"
    );
    assert_eq!(snapshot(&f.base), before);
}

#[test]
fn rejected_store_admission_is_reported_without_success_promotion() {
    let mut f = Fixture::new();
    let r = result_only(&mut f, 2);
    let publication = f.store.complete_selected_review_admission(r).unwrap();
    let AuthoritativeReviewAdmissionRuntimeOutcome::Published(p) = publication else {
        panic!("expected rejected Store fixture publication")
    };
    assert_eq!(p.journal_reference().event_type_id().value(), 303);
    drop(LedgerWriter::open(f.base.join("ledger")).unwrap());
    let j = serde_json::to_value(inspect(f.base.join("store"), f.base.join("ledger")).unwrap())
        .unwrap();
    assert_eq!(j["counts"]["store_admissions_rejected"], 1);
    assert_eq!(j["counts"]["store_admissions_accepted"], 0);
}

#[test]
fn failed_and_unresolved_attempts_survive_reopen_without_live_witness() {
    use ai_agent_evidence_binder::ledger::Observation;
    let f = Fixture::new();
    let mut w = LedgerWriter::open(f.base.join("ledger")).unwrap();
    w.append_intent(
        binding(f.request.request_event_reference()),
        "failed",
        f.intent(None),
    )
    .unwrap()
    .consume();
    w.append_observation(
        "failed",
        Observation::SpawnFailed {
            code: "synthetic_failure".into(),
        },
    )
    .unwrap();
    w.append_intent(
        binding(f.request.request_event_reference()),
        "pending",
        f.intent(Some("failed")),
    )
    .unwrap()
    .consume();
    drop(w);
    let before = fs::read(f.base.join("ledger")).unwrap();
    let j = serde_json::to_value(inspect(f.base.join("store"), f.base.join("ledger")).unwrap())
        .unwrap();
    assert_eq!(j["counts"]["attempts"], 2);
    assert_eq!(j["attempts"][0]["status"], "ObservedFailure");
    assert_eq!(j["attempts"][1]["status"], "Unresolved");
    assert_eq!(j["attempts"][1]["predecessor"], "failed");
    for a in j["attempts"].as_array().unwrap() {
        assert_eq!(a["live_witness"], false);
        assert_eq!(a["request_validation"]["status"], "Validated");
    }
    assert_eq!(j["unmatched_store_requests"], serde_json::json!([]));
    assert_eq!(j["counts"]["observed_failure_attempts"], 1);
    assert_eq!(j["counts"]["unresolved_attempts"], 1);
    assert_eq!(
        j["counts"]["ledger_frames"],
        j["ledger"]["frames"].as_array().unwrap().len()
    );
    assert_eq!(
        j["counts"]["store_events"],
        j["store"]["events"].as_array().unwrap().len()
    );
    assert_eq!(j["counts"]["attempt_statuses"]["ObservedFailure"], 1);
    assert_eq!(j["counts"]["attempt_statuses"]["Unresolved"], 1);
    assert_eq!(j["ledger_join_authoritative"], false);
    assert_eq!(fs::read(f.base.join("ledger")).unwrap(), before);
}

#[test]
fn empty_ledger_exposes_unmanaged_store_request_in_serializable_read_only_report() {
    let f = Fixture::new();
    drop(LedgerWriter::open(f.base.join("ledger")).unwrap());
    let before = fs::read(f.base.join("ledger")).unwrap();
    let report = inspect(f.base.join("store"), f.base.join("ledger")).unwrap();
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["schema"], 1);
    assert_eq!(json["read_only"], true);
    assert_eq!(json["store"]["status"], "ValidatedCapturedView");
    assert_eq!(json["counts"]["attempts"], 0);
    assert_eq!(json["counts"]["store_requests"], 1);
    assert_eq!(
        json["unmatched_store_requests"][0],
        serde_json::to_value(binding(f.request.request_event_reference())).unwrap()
    );
    assert_eq!(json["counts"]["independent_reviews_established"], 0);
    assert_eq!(fs::read(f.base.join("ledger")).unwrap(), before);
}
