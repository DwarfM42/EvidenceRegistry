//! One live managed dispatch → independent Store output Freeze → exact Result → Admission.
//!
//! This is not an old-log import API. Only the private function consuming the
//! non-Clone managed completion can publish. Output bytes are capture-time evidence,
//! not authenticated authorship or equality with bytes at process exit. The ledger
//! join does not add a Core supporting-evidence relation or a Policy evaluator.
use crate::{
    ledger::{
        DispatchIntent, FaultKind, LedgerError, LedgerWriter, Observation, Publication,
        PublicationKind, RetainedReference,
    },
    parse_submission,
    process::{run_managed, ManagedCompletion, ProcessError},
    RequestBinding, SubmissionError,
};
use evidence_registry::*;
#[cfg(test)]
mod crash_tests;
use std::{
    collections::BTreeSet,
    path::{Component, Path},
    sync::atomic::AtomicBool,
};

#[derive(Clone, Debug)]
pub struct OutputCapturePlan {
    pub freeze_attempt_id: FreezeAttemptId,
    pub policy_record_id: RecordId,
}
/// The Store alone derives the disposition. `Ok` is not synonymous with accepted.
#[derive(Debug)]
pub struct BridgeOutcome {
    pub output_freeze: AuthoritativeFreezeCommittedBinding,
    pub result: RecordedSelectedReviewResult,
    pub admission: AuthoritativeReviewAdmissionRuntimeOutcome,
}
#[derive(Debug)]
pub enum BridgeError {
    Preflight(&'static str),
    Io(std::io::Error),
    Ledger(LedgerError),
    Process(ProcessError),
    Request(SelectedReviewRequestError),
    Preparation(SelectedEmbeddedFreezePreparationError),
    Capture(SelectedEmbeddedFreezeCommitError),
    Readback(AuthoritativeFreezeCommittedBindingError),
    CaptureSet(&'static str),
    Submission(SubmissionError),
    Result(SelectedReviewResultError),
    Section82(SelectedReviewAdmissionSection82Error),
    Admission(SelectedReviewAdmissionCompletionError),
    FailureRecording {
        original: String,
        ledger: LedgerError,
    },
}
impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for BridgeError {}
impl From<LedgerError> for BridgeError {
    fn from(e: LedgerError) -> Self {
        Self::Ledger(e)
    }
}
/// Read-only serialization of an exact reference; never an ingestion capability.
pub fn request_binding(r: &JournalReference) -> RequestBinding {
    RequestBinding {
        registry_id: hex(r.registry_id().as_bytes()),
        entry_index: r.entry_index().value(),
        entry_hash: hex(r.entry_hash().as_bytes()),
        event_type_id: r.event_type_id().value().into(),
        event_record_id: hex(r.event_record_id().as_bytes()),
    }
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|b| format!("{b:02x}")).collect()
}
fn retained(r: &JournalReference) -> RetainedReference {
    let b = request_binding(r);
    RetainedReference {
        registry_id: b.registry_id,
        entry_index: b.entry_index,
        entry_hash: b.entry_hash,
        event_type_id: b.event_type_id,
        event_record_id: b.event_record_id,
    }
}
/// Safe complete entry. Validates current Store Q before durable intent and spawn,
/// checks namespace separation, and never retries any dispatch or Store mutation.
/// Path comparison is an overlap guard, not a retained-root capability or custody.
pub fn run_to_admission(
    store: &mut AuthoritativeRegistryStore,
    writer: &mut LedgerWriter,
    request: JournalReference,
    attempt_id: &str,
    intent: DispatchIntent,
    capture: OutputCapturePlan,
    cancellation: &AtomicBool,
) -> Result<BridgeOutcome, BridgeError> {
    if ["submission.json", "binder-stdout.bin", "binder-stderr.bin"]
        .iter()
        .any(|required| !intent.mandatory_files.iter().any(|name| name == required))
    {
        return Err(BridgeError::Preflight("missing_fixed_declaration"));
    }
    check_separation(
        store.root(),
        writer.path(),
        Path::new(&intent.approved_output_root),
    )?;
    let request = store
        .validate_selected_review_request(request)
        .map_err(BridgeError::Request)?;
    let completion = run_managed(
        writer,
        request_binding(request.request_event_reference()),
        attempt_id,
        intent,
        cancellation,
    )
    .map_err(BridgeError::Process)?;
    ingest_completion(store, writer, completion, request, capture)
}
fn check_separation(store: &Path, ledger: &Path, output: &Path) -> Result<(), BridgeError> {
    // This selects current canonical names only for overlap rejection. Store still
    // binds and verifies its own source handles; no bytes are read by this guard.
    let key = |p: &Path| -> Result<std::path::PathBuf, BridgeError> {
        if p.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(BridgeError::Preflight("parent_component"));
        }
        let p = std::fs::canonicalize(p).map_err(BridgeError::Io)?;
        #[cfg(windows)]
        let p = std::path::PathBuf::from(
            p.to_str()
                .ok_or(BridgeError::Preflight("path_encoding"))?
                .to_lowercase(),
        );
        Ok(p)
    };
    let store = key(store)?;
    let ledger = key(ledger)?;
    let output = key(output)?;
    if ledger.starts_with(&store)
        || ledger.starts_with(&output)
        || output.starts_with(&store)
        || store.starts_with(&output)
    {
        return Err(BridgeError::Preflight("namespace_overlap"));
    }
    Ok(())
}
fn intent(w: &mut LedgerWriter, a: &str, operation: PublicationKind) -> Result<(), BridgeError> {
    w.append_publication(a, Publication::Intent { operation })?;
    Ok(())
}
fn returned(
    w: &mut LedgerWriter,
    a: &str,
    operation: PublicationKind,
    refs: &[&JournalReference],
    receipt: &str,
) -> Result<(), BridgeError> {
    w.append_publication(
        a,
        Publication::Returned {
            operation,
            references: refs.iter().map(|r| retained(r)).collect(),
            receipt: receipt.into(),
        },
    )?;
    Ok(())
}
fn uncertain(
    w: &mut LedgerWriter,
    a: &str,
    operation: PublicationKind,
    code: &str,
    error: BridgeError,
) -> BridgeError {
    #[cfg(test)]
    crash_tests::fire("publication_error");
    match w.append_publication(
        a,
        Publication::Uncertain {
            operation,
            code: code.into(),
        },
    ) {
        Ok(_) => error,
        Err(ledger) => BridgeError::FailureRecording {
            original: error.to_string(),
            ledger,
        },
    }
}
fn fault(
    w: &mut LedgerWriter,
    a: &str,
    kind: FaultKind,
    code: &str,
    error: BridgeError,
) -> BridgeError {
    match w.append_observation(
        a,
        Observation::Fault {
            fault: kind,
            detail: code.into(),
        },
    ) {
        Ok(_) => error,
        Err(ledger) => BridgeError::FailureRecording {
            original: error.to_string(),
            ledger,
        },
    }
}
// Private, by-value completion ownership is kept through all capture validation.
// There is deliberately no public endpoint accepting a ledger or historical IDs.
fn ingest_completion(
    store: &mut AuthoritativeRegistryStore,
    writer: &mut LedgerWriter,
    completion: ManagedCompletion,
    request: RecordedSelectedReviewRequest,
    capture: OutputCapturePlan,
) -> Result<BridgeOutcome, BridgeError> {
    let a = completion.attempt_id();
    let declaration = completion.intent();
    intent(writer, a, PublicationKind::OutputPreparation)?;
    #[cfg(test)]
    crash_tests::fire("preparation_enter");
    let prepared = store
        .prepare_selected_embedded_freeze_with_limits(
            SelectedEmbeddedFreezePreparationInput {
                source_root: declaration.approved_output_root.clone().into(),
                freeze_attempt_id: capture.freeze_attempt_id,
                policy_record_id: capture.policy_record_id,
            },
            SelectedEmbeddedCaptureLimits {
                max_files: declaration.limits.output_files as usize,
                max_content_bytes: declaration.limits.output_bytes as usize,
            },
        )
        .map_err(|e| {
            uncertain(
                writer,
                a,
                PublicationKind::OutputPreparation,
                "output_preparation_failed",
                BridgeError::Preparation(e),
            )
        })?;
    #[cfg(test)]
    crash_tests::fire("preparation_visible");
    returned(
        writer,
        a,
        PublicationKind::OutputPreparation,
        &[prepared.start_event_reference()],
        "Store returned preparation; START only; no Freeze receipt",
    )?;
    intent(writer, a, PublicationKind::OutputCapture)?;
    #[cfg(test)]
    crash_tests::fire("capture_enter");
    let output_freeze = store
        .commit_prepared_selected_embedded_freeze(prepared)
        .map_err(|e| {
            uncertain(
                writer,
                a,
                PublicationKind::OutputCapture,
                "output_commit_failed",
                BridgeError::Capture(e),
            )
        })?;
    #[cfg(test)]
    crash_tests::fire("capture_visible");
    returned(
        writer,
        a,
        PublicationKind::OutputCapture,
        &[output_freeze.committed_event_reference()],
        "Store returned committed Freeze binding; capture-time evidence",
    )?;
    #[cfg(test)]
    crash_tests::fire("capture_recorded");
    let snapshot = store
        .read_selected_embedded_payload(output_freeze.committed_event_reference().clone())
        .map_err(|e| {
            fault(
                writer,
                a,
                FaultKind::CaptureFailed,
                "output_readback_failed",
                BridgeError::Readback(e),
            )
        })?;
    let manifest = snapshot.manifest().input();
    let mut names = BTreeSet::new();
    let mut submission_index = None;
    for (i, artifact) in manifest.artifacts.iter().enumerate() {
        let name = artifact
            .path_components
            .iter()
            .map(|c| std::str::from_utf8(c))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| {
                fault(
                    writer,
                    a,
                    FaultKind::CaptureFailed,
                    "nonportable_output",
                    BridgeError::CaptureSet("nonportable_output"),
                )
            })?
            .join("/");
        if name == "submission.json" {
            submission_index = Some(i);
        }
        names.insert(name);
    }
    let i = submission_index.ok_or_else(|| {
        fault(
            writer,
            a,
            FaultKind::InvalidSubmission,
            "missing_submission",
            BridgeError::CaptureSet("missing_submission"),
        )
    })?;
    // Completeness is relative to the predispatch declaration, not the Agent's
    // optional candidate list. Compare only Store-captured manifest members.
    if declaration
        .mandatory_files
        .iter()
        .any(|name| !names.contains(name))
    {
        return Err(fault(
            writer,
            a,
            FaultKind::CaptureFailed,
            "missing_mandatory",
            BridgeError::CaptureSet("missing_mandatory"),
        ));
    }
    let submission = parse_submission(&snapshot.artifact_bytes()[i], completion.request())
        .map_err(|e| {
            fault(
                writer,
                a,
                FaultKind::InvalidSubmission,
                "invalid_submission",
                BridgeError::Submission(e),
            )
        })?;
    if submission
        .artifacts
        .iter()
        .any(|name| !names.contains(name))
    {
        return Err(fault(
            writer,
            a,
            FaultKind::InvalidSubmission,
            "candidate_not_captured",
            BridgeError::CaptureSet("candidate_not_captured"),
        ));
    }
    let findings = submission
        .findings
        .iter()
        .map(|s| {
            let bytes: Vec<u8> = s
                .as_bytes()
                .chunks_exact(2)
                .map(|b| {
                    u8::from_str_radix(std::str::from_utf8(b).expect("parser ASCII hex"), 16)
                        .expect("parser hex")
                })
                .collect();
            RecordId::try_from(bytes.as_slice()).expect("parser exact ID length")
        })
        .collect();
    intent(writer, a, PublicationKind::ReviewResult)?;
    #[cfg(test)]
    crash_tests::fire("result_enter");
    let result = store
        .record_selected_review_result(SelectedReviewResultInput {
            request_event_reference: request.request_event_reference().clone(),
            method_status: submission.method_status,
            finding_state: submission.finding_state,
            reason_codes: submission.reason_codes,
            findings,
            reviewer_metadata: submission.reviewer_metadata,
        })
        .map_err(|e| {
            uncertain(
                writer,
                a,
                PublicationKind::ReviewResult,
                "result_failed",
                BridgeError::Result(e),
            )
        })?;
    #[cfg(test)]
    crash_tests::fire("result_visible");
    returned(
        writer,
        a,
        PublicationKind::ReviewResult,
        &[result.result_event_reference()],
        "Store returned exact selected Result; Agent claims not semantic truth",
    )?;
    store
        .derive_selected_review_admission_section_82(result.result_event_reference().clone())
        .map_err(|e| {
            fault(
                writer,
                a,
                FaultKind::Unavailable,
                "section82_failed",
                BridgeError::Section82(e),
            )
        })?;
    intent(writer, a, PublicationKind::Admission)?;
    #[cfg(test)]
    crash_tests::fire("admission_enter");
    let admission = store
        .complete_selected_review_admission(result.result_event_reference().clone())
        .map_err(|e| {
            uncertain(
                writer,
                a,
                PublicationKind::Admission,
                "admission_failed",
                BridgeError::Admission(e),
            )
        })?;
    #[cfg(test)]
    crash_tests::fire("admission_visible");
    match &admission {
        AuthoritativeReviewAdmissionRuntimeOutcome::Published(p) => returned(
            writer,
            a,
            PublicationKind::Admission,
            &[p.journal_reference()],
            &format!("Store returned publication: {:?}", p.durability()),
        )?,
        AuthoritativeReviewAdmissionRuntimeOutcome::PublishedReceiptUncertain(_) => {
            writer.append_publication(
                a,
                Publication::Uncertain {
                    operation: PublicationKind::Admission,
                    code: "admission_receipt_uncertain".into(),
                },
            )?;
        }
        AuthoritativeReviewAdmissionRuntimeOutcome::PreTerminal(_) => {
            writer.append_publication(
                a,
                Publication::Uncertain {
                    operation: PublicationKind::Admission,
                    code: "admission_preterminal".into(),
                },
            )?;
        }
    }
    drop(completion);
    Ok(BridgeOutcome {
        output_freeze,
        result,
        admission,
    })
}
