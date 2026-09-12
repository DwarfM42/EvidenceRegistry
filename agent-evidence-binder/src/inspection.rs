//! Read-only reconciliation of nonauthoritative Binder history with one cold Store view.
//!
//! Inspection never recreates execution witnesses or publication durability receipts.
//! Its Q/output/R joins are ledger claims, not a new Core supporting-evidence relation.
use crate::ledger::{
    self, Event, Frame, Observation, PublicationKind, PublicationStatus, RetainedReference,
};
use crate::RequestBinding;
use evidence_registry::*;
use serde::Serialize;
use std::{collections::BTreeMap, path::Path};

/// Bounds the ledger-attempt × Store-event cross product retained as candidate
/// references. The global exact Store event inventory is never truncated.
pub const MAX_CANDIDATE_REFERENCES: usize = 65_536;

#[derive(Clone, Debug, Serialize)]
pub struct InspectionReport {
    pub schema: u32,
    pub read_only: bool,
    pub ledger_join_authoritative: bool,
    pub candidate_enumeration_complete: bool,
    pub ledger: LedgerInspection,
    pub store: StoreInspection,
    pub attempts: Vec<AttemptInspection>,
    pub unmatched_store_requests: Vec<RetainedReference>,
    pub unmatched_store_results: Vec<RetainedReference>,
    pub counts: Counts,
    pub matching_result_claims: Vec<MatchingResultClaims>,
}
#[derive(Clone, Debug, Serialize)]
pub struct LedgerInspection {
    pub status: String,
    pub valid_bytes: u64,
    pub last_digest: String,
    pub boundary: Option<CorruptionBoundary>,
    /// Validated prefix only. Observations/receipt text remain ledger assertions.
    pub frames: Vec<Frame>,
}
#[derive(Clone, Debug, Serialize)]
pub struct CorruptionBoundary {
    pub offset: u64,
    pub kind: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct StoreInspection {
    pub status: String,
    pub error: Option<String>,
    pub captured_head: Option<RetainedReference>,
    /// Ordered exact retained instances, never deduplicated by Record identity.
    pub events: Vec<StoreEvent>,
}
#[derive(Clone, Debug, Serialize)]
pub struct StoreEvent {
    pub reference: RetainedReference,
    pub validation: Validation,
    pub validation_scope: String,
    /// Record-declared relation for reconciliation, even when current validation fails.
    pub request: Option<RetainedReference>,
    pub result: Option<RetainedReference>,
    pub freeze_start: Option<RetainedReference>,
    pub target_freeze: Option<RetainedReference>,
    pub disposition_id: Option<u64>,
    pub result_claims: Option<ResultClaims>,
}
/// Equality here compares only Core-mapped submitted claims for one exact Q;
/// not whole JSON, artifact selections, authorship, or independent reasoning.
#[derive(Clone, Debug, Serialize)]
pub struct ResultClaims {
    pub method_status: u64,
    pub finding_state: u64,
    pub reason_codes: Vec<String>,
    pub findings: Vec<String>,
    pub reviewer_metadata: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct MatchingResultClaims {
    pub references: Vec<RetainedReference>,
}
#[derive(Clone, Debug, Serialize)]
pub struct AttemptInspection {
    pub attempt_id: String,
    pub request: RequestBinding,
    pub predecessor: Option<String>,
    pub launch: String,
    pub live_witness: bool,
    pub historical_receipt_recovered: bool,
    pub ledger_history_complete: bool,
    pub publications: Vec<PublicationInspection>,
    pub status: String,
    pub request_validation: Validation,
    pub sequences: Vec<u64>,
}
#[derive(Clone, Debug, Serialize)]
pub struct PublicationInspection {
    pub operation: PublicationKind,
    pub ledger_status: String,
    pub uncertainty_code: Option<String>,
    /// Possible retained instances, NOT proof this attempt produced them. Includes
    /// invalid current candidates; consult the exact Store event validation.
    pub candidates: Vec<RetainedReference>,
    /// Exactly the number enumerated in `candidates` (not the total matches).
    pub candidate_count: usize,
    pub candidate_total_matches: Option<usize>,
    /// Complete, ResourceLimited, Unavailable, or NotApplicable (known return).
    pub candidate_enumeration_status: String,
    pub candidate_attribution_established: bool,
    pub references: Vec<ReferenceInspection>,
}
#[derive(Clone, Debug, Serialize)]
pub struct ReferenceInspection {
    pub reference: RetainedReference,
    pub validation: Validation,
}
#[derive(Clone, Debug, Serialize)]
pub struct Validation {
    pub status: String,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct Counts {
    pub attempts: usize,
    pub attempt_statuses: BTreeMap<String, usize>,
    pub ledger_frames: usize,
    pub store_events: usize,
    pub store_freeze_starts: usize,
    pub store_freezes: usize,
    pub unmatched_store_requests: usize,
    pub unmatched_store_results: usize,
    pub observed_failure_attempts: usize,
    pub unresolved_attempts: usize,
    pub store_requests: usize,
    pub store_results: usize,
    pub store_admissions_accepted: usize,
    pub store_admissions_rejected: usize,
    pub distinct_result_claim_sets: usize,
    pub matching_result_claim_groups: usize,
    /// No independence/authorship evidence is established by this inspection.
    pub independent_reviews_established: usize,
}

/// Opens neither a writer nor an execution/Admission route. A corrupt ledger
/// returns its valid prefix and boundary. A Store open failure remains visible
/// alongside that history. Busy/unreadable ledgers return the original error.
pub fn inspect(
    store_root: impl AsRef<Path>,
    ledger_path: impl AsRef<Path>,
) -> ledger::Result<InspectionReport> {
    let read = ledger::read_ledger(ledger_path)?;
    let opened = AuthoritativeRegistryStore::open_selected_profile(store_root);
    let mut store = match &opened {
        Ok(store) => StoreInspection {
            status: "ValidatedCapturedView".into(),
            error: None,
            captured_head: Some(retained(&store.retained_journal().current_head_reference())),
            events: store
                .retained_journal()
                .references()
                .filter(|r| matches!(r.event_type_id().value(), 100 | 101 | 300 | 301 | 302 | 303))
                .map(|r| inspect_event(store, r))
                .collect(),
        },
        Err(error) => StoreInspection {
            status: "Unavailable".into(),
            error: Some(format!("{error:?}")),
            captured_head: None,
            events: vec![],
        },
    };
    if store
        .events
        .iter()
        .any(|e| e.validation.status != "Validated")
    {
        store.status = "CapturedViewWithValidationFailures".into();
    }
    let mut candidate_budget = MAX_CANDIDATE_REFERENCES;
    let attempts = read
        .attempts()
        .into_iter()
        .map(|a| inspect_attempt(&store, &read, a, &mut candidate_budget))
        .collect::<Vec<_>>();
    let unmatched_store_requests = store
        .events
        .iter()
        .filter(|e| {
            e.reference.event_type_id == 300
                && !attempts
                    .iter()
                    .any(|a| request_reference(&a.request) == e.reference)
        })
        .map(|e| e.reference.clone())
        .collect::<Vec<_>>();
    let unmatched_store_results = store
        .events
        .iter()
        .filter(|e| {
            e.reference.event_type_id == 301
                && !attempts.iter().any(|a| {
                    a.publications.iter().any(|p| {
                        p.operation == PublicationKind::ReviewResult
                            && p.references.iter().any(|r| {
                                r.reference == e.reference && r.validation.status == "Validated"
                            })
                    })
                })
        })
        .map(|e| e.reference.clone())
        .collect::<Vec<_>>();
    let mut claim_groups: BTreeMap<String, Vec<RetainedReference>> = BTreeMap::new();
    for e in &store.events {
        if let (Some(q), Some(claims)) = (&e.request, &e.result_claims) {
            let key =
                serde_json::to_string(&(q, claims)).expect("finite scalar and string projection");
            claim_groups
                .entry(key)
                .or_default()
                .push(e.reference.clone());
        }
    }
    let distinct_result_claim_sets = claim_groups.len();
    let mut matching_result_claims = claim_groups
        .into_values()
        .filter(|v| v.len() > 1)
        .map(|references| MatchingResultClaims { references })
        .collect::<Vec<_>>();
    matching_result_claims.sort_by_key(|g| g.references[0].entry_index);
    let mut attempt_statuses = BTreeMap::new();
    for a in &attempts {
        *attempt_statuses.entry(a.status.clone()).or_default() += 1;
    }
    let counts = Counts {
        attempt_statuses,
        ledger_frames: read.frames.len(),
        store_events: store.events.len(),
        store_freeze_starts: store
            .events
            .iter()
            .filter(|e| e.reference.event_type_id == 100)
            .count(),
        store_freezes: store
            .events
            .iter()
            .filter(|e| e.reference.event_type_id == 101)
            .count(),
        unmatched_store_requests: unmatched_store_requests.len(),
        unmatched_store_results: unmatched_store_results.len(),
        attempts: attempts.len(),
        observed_failure_attempts: attempts
            .iter()
            .filter(|a| a.status == "ObservedFailure")
            .count(),
        unresolved_attempts: attempts.iter().filter(|a| a.status == "Unresolved").count(),
        store_requests: store
            .events
            .iter()
            .filter(|e| e.reference.event_type_id == 300)
            .count(),
        store_results: store
            .events
            .iter()
            .filter(|e| e.reference.event_type_id == 301)
            .count(),
        store_admissions_accepted: store
            .events
            .iter()
            .filter(|e| e.reference.event_type_id == 302 && e.validation.status == "Validated")
            .count(),
        store_admissions_rejected: store
            .events
            .iter()
            .filter(|e| e.reference.event_type_id == 303 && e.validation.status == "Validated")
            .count(),
        distinct_result_claim_sets,
        matching_result_claim_groups: matching_result_claims.len(),
        independent_reviews_established: 0,
    };
    Ok(InspectionReport {
        schema: 1,
        read_only: true,
        ledger_join_authoritative: false,
        candidate_enumeration_complete: store.status != "Unavailable"
            && attempts.iter().flat_map(|a| &a.publications).all(|p| {
                matches!(
                    p.candidate_enumeration_status.as_str(),
                    "Complete" | "NotApplicable"
                )
            }),
        ledger: LedgerInspection {
            status: if read.boundary.is_some() {
                "CorruptPrefix"
            } else {
                "ValidPrefix"
            }
            .into(),
            valid_bytes: read.valid_bytes,
            last_digest: read.last_digest,
            boundary: read.boundary.map(|b| CorruptionBoundary {
                offset: b.offset,
                kind: format!("{:?}", b.kind),
            }),
            frames: read.frames,
        },
        store,
        attempts,
        unmatched_store_requests,
        unmatched_store_results,
        counts,
        matching_result_claims,
    })
}
fn inspect_attempt(
    store: &StoreInspection,
    read: &ledger::ReadReport,
    a: ledger::AttemptProjection,
    candidate_budget: &mut usize,
) -> AttemptInspection {
    let reference = request_reference(&a.request);
    let request_validation = if store.status == "Unavailable" {
        Validation {
            status: "Unavailable".into(),
            error: store.error.clone(),
        }
    } else {
        lookup_event(store, &reference)
            .map(|e| e.validation.clone())
            .unwrap_or_else(|| invalid("ExactReferenceNotRetained"))
    };
    let publications = a
        .publications
        .iter()
        .map(|p| {
            let (set, candidate_enumeration_status) =
                if p.status == PublicationStatus::KnownReferences {
                    (CandidateSet::default(), "NotApplicable")
                } else if store.status == "Unavailable" {
                    (CandidateSet::default(), "Unavailable")
                } else {
                    let set = collect_candidates(store, &reference, p.operation, candidate_budget);
                    let status = if set.complete {
                        "Complete"
                    } else {
                        "ResourceLimited"
                    };
                    (set, status)
                };
            PublicationInspection {
                candidate_count: set.references.len(),
                candidates: set.references,
                candidate_total_matches: matches!(
                    candidate_enumeration_status,
                    "Complete" | "ResourceLimited"
                )
                .then_some(set.total_matches),
                candidate_enumeration_status: candidate_enumeration_status.into(),
                candidate_attribution_established: false,
                operation: p.operation,
                ledger_status: match p.status {
                    PublicationStatus::Unresolved => "Unresolved",
                    PublicationStatus::KnownReferences => "KnownReferences",
                    PublicationStatus::Uncertain(_) => "Uncertain",
                }
                .into(),
                uncertainty_code: match &p.status {
                    PublicationStatus::Uncertain(code) => Some(code.clone()),
                    _ => None,
                },
                references: p
                    .references
                    .iter()
                    .map(|r| ReferenceInspection {
                        reference: r.clone(),
                        validation: known_reference(
                            store,
                            &reference,
                            &a.publications,
                            p.operation,
                            r,
                        ),
                    })
                    .collect(),
            }
        })
        .collect::<Vec<_>>();
    let complete = publications
        .last()
        .is_some_and(|p| p.operation == PublicationKind::Admission)
        && publications.iter().all(|p| {
            p.ledger_status == "KnownReferences"
                && p.references
                    .iter()
                    .all(|r| r.validation.status == "Validated")
        });
    let failed = read
        .frames
        .iter()
        .filter(|f| f.attempt_id == a.attempt_id)
        .any(|f| match f.event {
            Event::Observation(
                Observation::SpawnFailed { .. }
                | Observation::Fault { .. }
                | Observation::Exited { code: None },
            ) => true,
            Event::Observation(Observation::Exited { code: Some(code) }) => code != 0,
            _ => false,
        });
    let status = if failed {
        "ObservedFailure"
    } else if read.boundary.is_some() {
        "CorruptHistory"
    } else if request_validation.status == "Unavailable" {
        "StoreUnavailable"
    } else if request_validation.status != "Validated"
        || publications.iter().any(|p| {
            p.references
                .iter()
                .any(|r| r.validation.status != "Validated")
        })
    {
        "ReferenceMismatch"
    } else if complete {
        "HistoricalReferencesValidated"
    } else {
        "Unresolved"
    };
    AttemptInspection {
        attempt_id: a.attempt_id,
        request: a.request,
        predecessor: a.predecessor,
        launch: format!("{:?}", a.launch),
        live_witness: false,
        historical_receipt_recovered: false,
        ledger_history_complete: read.boundary.is_none(),
        status: status.into(),
        publications,
        request_validation,
        sequences: a.sequences,
    }
}

fn inspect_event(store: &AuthoritativeRegistryStore, r: JournalReference) -> StoreEvent {
    let mut event = StoreEvent {
        reference: retained(&r),
        validation: invalid("Unvalidated"),
        validation_scope: String::new(),
        request: None,
        result: None,
        freeze_start: None,
        target_freeze: None,
        disposition_id: None,
        result_claims: None,
    };
    match r.event_type_id().value() {
        100 => {
            event.validation_scope = "RetainedStartOnlyNotCommittedCapture".into();
            event.validation = validation(
                store
                    .resolve(record_id(&r))
                    .ok_or("RecordUnavailable")
                    .and_then(|b| {
                        FreezeAttemptStartRecord::decode_authoritative(b).map_err(|_| "StartDecode")
                    }),
            );
        }
        101 => {
            event.validation_scope = "SelectedFreezeAndCurrentPayload".into();
            let checked = store.validate_freeze_committed_authority(r);
            if let Ok(f) = &checked {
                event.freeze_start = Some(retained(f.start_event_reference()));
            }
            event.validation = validation(checked);
        }
        300 => {
            event.validation_scope = "SelectedRequestAndPrerequisites".into();
            let checked = store.validate_selected_review_request(r);
            if let Ok(q) = &checked {
                event.target_freeze =
                    Some(retained(q.freeze_authority().committed_event_reference()));
            }
            event.validation = validation(checked);
        }
        301 => {
            event.validation_scope = "SelectedResultTransportNotAdmission".into();
            // Keep the exact Record-declared Q for diagnostic matching even if
            // current payload validation fails. It cannot authorize ingestion.
            if let Some(decoded) = store
                .resolve(record_id(&r))
                .and_then(|b| ReviewResultRecord::decode_authoritative(b).ok())
            {
                event.request = Some(retained(decoded.review_request_authority_ref()));
            }
            let checked = store.validate_selected_review_result(r);
            if let Ok(result) = &checked {
                event.request = Some(retained(result.request().request_event_reference()));
                let claims = result.result();
                event.result_claims = Some(ResultClaims {
                    method_status: claims.method_status(),
                    finding_state: claims.finding_state(),
                    reason_codes: claims.reason_codes().to_vec(),
                    findings: claims
                        .findings()
                        .iter()
                        .map(|r| hex(r.as_bytes()))
                        .collect(),
                    reviewer_metadata: claims.reviewer_metadata().map(str::to_owned),
                });
            }
            event.validation = validation(checked);
        }
        302 | 303 => {
            // Selected cold open replays EVERY selected terminal through Core's
            // exact Request/Result, §82, §46, §83 and Record reconstruction.
            // A legacy marker is not promoted to selected semantic authority.
            event.validation_scope = "SelectedTerminalColdReplay".into();
            let checked = store
                .resolve(record_id(&r))
                .ok_or("RecordUnavailable")
                .and_then(|b| {
                    ReviewAdmissionRecord::decode_authoritative(b).map_err(|_| "AdmissionDecode")
                })
                .and_then(|a| {
                    if a.terminal_authority_closure_sha256()
                        == Some(&TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256)
                    {
                        Ok(a)
                    } else {
                        Err("SelectedAuthorityUnavailable")
                    }
                });
            if let Ok(a) = &checked {
                event.request = a.review_request_ref().map(retained);
                event.result = a.review_result_ref().map(retained);
                event.disposition_id = Some(a.disposition_id());
            }
            event.validation = validation(checked);
        }
        _ => unreachable!("filtered event inventory"),
    }
    event
}
fn record_id(r: &JournalReference) -> RecordId {
    RecordId::try_from(r.event_record_id().as_bytes().as_slice())
        .expect("Core exact identity width")
}
fn lookup_event<'a>(store: &'a StoreInspection, r: &RetainedReference) -> Option<&'a StoreEvent> {
    let index = store
        .events
        .binary_search_by_key(&r.entry_index, |e| e.reference.entry_index)
        .ok()?;
    let event = &store.events[index];
    (event.reference == *r).then_some(event)
}
#[derive(Default)]
struct CandidateSet {
    references: Vec<RetainedReference>,
    total_matches: usize,
    complete: bool,
}
fn collect_candidates(
    store: &StoreInspection,
    q: &RetainedReference,
    op: PublicationKind,
    budget: &mut usize,
) -> CandidateSet {
    let mut set = CandidateSet::default();
    for event in store.events.iter().filter(|e| candidate(e, q, op)) {
        set.total_matches += 1;
        if *budget > 0 {
            set.references.push(event.reference.clone());
            *budget -= 1;
        }
    }
    set.complete = set.references.len() == set.total_matches;
    set
}
fn candidate(e: &StoreEvent, q: &RetainedReference, op: PublicationKind) -> bool {
    if e.reference.registry_id != q.registry_id || e.reference.entry_index <= q.entry_index {
        return false;
    }
    match op {
        // The ledger does not retain a Core output-attempt selector in the intent.
        // Keep all later instances; neither roots nor timestamps prove attribution.
        PublicationKind::OutputPreparation => matches!(e.reference.event_type_id, 100 | 101),
        PublicationKind::OutputCapture => e.reference.event_type_id == 101,
        PublicationKind::ReviewResult => {
            e.reference.event_type_id == 301 && e.request.as_ref() == Some(q)
        }
        PublicationKind::Admission => {
            matches!(e.reference.event_type_id, 302 | 303) && e.request.as_ref() == Some(q)
        }
    }
}
fn known_reference(
    store: &StoreInspection,
    q: &RetainedReference,
    publications: &[ledger::PublicationProjection],
    op: PublicationKind,
    r: &RetainedReference,
) -> Validation {
    if store.status == "Unavailable" {
        return Validation {
            status: "Unavailable".into(),
            error: store.error.clone(),
        };
    }
    let Some(event) = lookup_event(store, r) else {
        return invalid("ExactReferenceNotRetained");
    };
    let expected = match op {
        PublicationKind::OutputPreparation => r.event_type_id == 100,
        PublicationKind::OutputCapture => r.event_type_id == 101,
        PublicationKind::ReviewResult => r.event_type_id == 301,
        PublicationKind::Admission => matches!(r.event_type_id, 302 | 303),
    };
    if !expected {
        return invalid("PublicationEventKindMismatch");
    }
    if r.entry_index <= q.entry_index {
        return invalid("PublicationNotAfterRequest");
    }
    if matches!(
        op,
        PublicationKind::ReviewResult | PublicationKind::Admission
    ) && event.request.as_ref() != Some(q)
    {
        return invalid("PublicationRequestMismatch");
    }
    if op == PublicationKind::OutputCapture {
        if let Some(preparation) = publications
            .iter()
            .find(|p| p.operation == PublicationKind::OutputPreparation)
        {
            if !event
                .freeze_start
                .as_ref()
                .is_some_and(|start| preparation.references.contains(start))
            {
                return invalid("CaptureStartMismatch");
            }
        }
    }
    if op == PublicationKind::Admission
        && !publications.iter().any(|p| {
            p.operation == PublicationKind::ReviewResult
                && event
                    .result
                    .as_ref()
                    .is_some_and(|r| p.references.contains(r))
        })
    {
        return invalid("AdmissionResultMismatch");
    }
    let predecessor = match op {
        PublicationKind::OutputPreparation => None,
        PublicationKind::OutputCapture => Some(PublicationKind::OutputPreparation),
        PublicationKind::ReviewResult => Some(PublicationKind::OutputCapture),
        PublicationKind::Admission => Some(PublicationKind::ReviewResult),
    };
    if publications
        .iter()
        .filter(|p| Some(p.operation) == predecessor)
        .flat_map(|p| &p.references)
        .any(|prior| prior.entry_index >= r.entry_index)
    {
        return invalid("PublicationStageChronologyMismatch");
    }
    event.validation.clone()
}
fn request_reference(r: &RequestBinding) -> RetainedReference {
    RetainedReference {
        registry_id: r.registry_id.clone(),
        entry_index: r.entry_index,
        entry_hash: r.entry_hash.clone(),
        event_type_id: r.event_type_id,
        event_record_id: r.event_record_id.clone(),
    }
}
fn validation<T, E: std::fmt::Debug>(r: Result<T, E>) -> Validation {
    match r {
        Ok(_) => Validation {
            status: "Validated".into(),
            error: None,
        },
        Err(e) => invalid(&format!("{e:?}")),
    }
}
fn invalid(error: &str) -> Validation {
    Validation {
        status: "Invalid".into(),
        error: Some(error.into()),
    }
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn retained(r: &JournalReference) -> RetainedReference {
    RetainedReference {
        registry_id: hex(r.registry_id().as_bytes()),
        entry_index: r.entry_index().value(),
        entry_hash: hex(r.entry_hash().as_bytes()),
        event_type_id: r.event_type_id().value().into(),
        event_record_id: hex(r.event_record_id().as_bytes()),
    }
}

#[cfg(test)]
mod projection_tests {
    use super::*;
    // Pure local-join unit fixture: these are projected labels, NOT Records,
    // Journal bytes, Store receipts, or evidence of execution/authority.
    fn reference(index: u64, event: u64) -> RetainedReference {
        RetainedReference {
            registry_id: "01".repeat(32),
            entry_index: index,
            entry_hash: format!("{index:064x}"),
            event_type_id: event,
            event_record_id: format!("{event:064x}"),
        }
    }
    #[test]
    fn candidate_cross_product_has_explicit_bounded_enumeration_not_silent_loss() {
        let q = reference(2, 300);
        let events = [3, 4]
            .into_iter()
            .map(|i| StoreEvent {
                reference: reference(i, 100),
                validation: validation(Ok::<(), ()>(())),
                validation_scope: "synthetic projection only".into(),
                request: None,
                result: None,
                freeze_start: None,
                target_freeze: None,
                disposition_id: None,
                result_claims: None,
            })
            .collect();
        let store = StoreInspection {
            status: "ValidatedCapturedView".into(),
            error: None,
            captured_head: None,
            events,
        };
        let mut budget = 1;
        let set = collect_candidates(&store, &q, PublicationKind::OutputPreparation, &mut budget);
        assert_eq!(set.references.len(), 1);
        assert_eq!(set.total_matches, 2);
        assert!(!set.complete);
        assert_eq!(budget, 0);
        assert_eq!(
            store.events.len(),
            2,
            "global exact inventory remains complete"
        );
    }
    #[test]
    fn a_later_capture_cannot_be_ledger_bound_as_preceding_result() {
        let q = reference(2, 300);
        let result = reference(4, 301);
        let output = reference(5, 101);
        let store = StoreInspection {
            status: "ValidatedCapturedView".into(),
            error: None,
            captured_head: Some(output.clone()),
            events: vec![StoreEvent {
                reference: result.clone(),
                validation: validation(Ok::<(), ()>(())),
                validation_scope: "synthetic projection only".into(),
                request: Some(q.clone()),
                result: None,
                freeze_start: None,
                target_freeze: None,
                disposition_id: None,
                result_claims: None,
            }],
        };
        let prior = vec![ledger::PublicationProjection {
            operation: PublicationKind::OutputCapture,
            status: PublicationStatus::KnownReferences,
            references: vec![output],
        }];
        let checked = known_reference(&store, &q, &prior, PublicationKind::ReviewResult, &result);
        assert_eq!(
            checked.error.as_deref(),
            Some("PublicationStageChronologyMismatch")
        );
    }
    #[test]
    fn contradictory_output_start_join_is_reported_even_if_each_projection_is_valid() {
        let q = reference(2, 300);
        let start = reference(3, 100);
        let output = reference(4, 101);
        let store = StoreInspection {
            status: "ValidatedCapturedView".into(),
            error: None,
            captured_head: Some(output.clone()),
            events: vec![StoreEvent {
                reference: output.clone(),
                validation: validation(Ok::<(), ()>(())),
                validation_scope: "synthetic projection only".into(),
                request: None,
                result: None,
                freeze_start: Some(reference(1, 100)),
                target_freeze: None,
                disposition_id: None,
                result_claims: None,
            }],
        };
        let prior = vec![ledger::PublicationProjection {
            operation: PublicationKind::OutputPreparation,
            status: PublicationStatus::KnownReferences,
            references: vec![start],
        }];
        let checked = known_reference(&store, &q, &prior, PublicationKind::OutputCapture, &output);
        assert_eq!(checked.status, "Invalid");
        assert_eq!(checked.error.as_deref(), Some("CaptureStartMismatch"));
    }
}
