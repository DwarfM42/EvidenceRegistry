use super::*;
use std::fs;
use std::path::{Path, PathBuf};

/// A fail-closed error while opening the authoritative on-disk Registry namespaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthoritativeRegistryStoreOpenError {
    Io,
    RootNamespaceInvalid,
    RegistryNamespaceInvalid,
    JournalNamespaceInvalid,
    JournalSlotNameInvalid,
    JournalSlotSequenceInvalid,
    RecordNamespaceInvalid,
    RecordFilenameInvalid,
    RecordDecode,
    RecordIdentityMismatch,
    GenesisRecordDecode,
    GenesisRecordIdentityMismatch,
    GenesisRegistryMismatch,
    GenesisProfileMismatch,
    GenesisCapabilityMismatch,
    GenesisEnvironmentMismatch,
    RetainedJournal(RetainedJournalError),
}

/// An authoritative Registry store opened from its exact retained Journal and Record namespaces.
///
/// The root path is only a locator. Registry identity, current head, Journal history, and Record
/// identities are derived from strict retained bytes. No caller-selected Journal head, Record body,
/// or prevalidated authority token participates in resolution.
#[derive(Debug)]
pub struct AuthoritativeRegistryStore {
    root: PathBuf,
    retained_journal: RetainedJournal,
    records: Vec<(RecordId, Vec<u8>)>,
    live_instance_identity: Arc<AuthoritativeRegistryStoreInstanceIdentity>,
}

#[derive(Debug)]
struct AuthoritativeRegistryStoreInstanceIdentity {
    outstanding_review_admissions: AtomicUsize,
}

static NEXT_AUTHORITATIVE_REVIEW_ADMISSION_TOKEN: AtomicU64 = AtomicU64::new(1);

/// One bounded, process-local authoritative Review Admission acceptance.
///
/// The exact operation-start head is captured from the opened store before any Request, Result,
/// or Policy decode. The value is non-cloneable and has no public constructor.
pub struct AcceptedAuthoritativeReviewAdmission {
    acceptance_token: u64,
    origin_store_instance_identity: Arc<AuthoritativeRegistryStoreInstanceIdentity>,
    request_event_reference: JournalReference,
    request_bytes: Vec<u8>,
    result_event_reference: JournalReference,
    result_bytes: Vec<u8>,
    operation_start_journal_ref: JournalReference,
}

impl AcceptedAuthoritativeReviewAdmission {
    pub fn operation_start_journal_ref(&self) -> &JournalReference {
        &self.operation_start_journal_ref
    }
}

impl Drop for AcceptedAuthoritativeReviewAdmission {
    fn drop(&mut self) {
        let previous = self
            .origin_store_instance_identity
            .outstanding_review_admissions
            .fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0);
    }
}

/// A fail-closed failure before an authoritative acceptance reaches `L(A)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthoritativeReviewAdmissionAcceptanceError {
    Input(ReviewAdmissionAcceptanceError),
    Store(AuthoritativeRegistryStoreOpenError),
    RegistryIdentityChanged,
}

/// A fail-closed authoritative §82 outcome before Policy evaluator dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthoritativeReviewAdmissionSection82Error {
    AcceptanceStoreMismatch,
    Store(AuthoritativeRegistryStoreOpenError),
    RegistryIdentityChanged,
    RequestPayloadUnavailable,
    ResultPayloadUnavailable,
    PresentedRequestMismatch,
    PresentedResultMismatch,
    Structural(ReviewAdmissionSection82AuthorityError),
    OperationStartReference(RetainedJournalError),
    RequestAuthorityAfterOperationStart,
    ResultAuthorityAfterOperationStart,
    FreezeAuthority(AuthoritativeFreezeCommittedBindingError),
}

/// Every authoritative input established at the successful §82 boundary.
///
/// The witness is derived from exact retained store bytes and has no public constructor. It is a
/// Policy-route input only: it is not a completed §46 result, §83 disposition, Admission Record,
/// terminal event, or publication effect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthoritativeReviewAdmissionSection82 {
    operation_start_journal_ref: JournalReference,
    request_event_reference: JournalReference,
    result_event_reference: JournalReference,
    request: ReviewRequestRecord,
    result: ReviewResultRecord,
    policy: ReviewAdmissionPolicyRecord,
    policy_bytes: Vec<u8>,
    returned_anchor: JournalAnchor,
    anchor_comparison: JournalAnchorHistoryComparison,
    policy_context_prerequisites: ReviewAdmissionPolicyContextPrerequisites,
    freeze_authority: AuthoritativeFreezeCommittedBinding,
}

impl AuthoritativeReviewAdmissionSection82 {
    pub fn operation_start_journal_ref(&self) -> &JournalReference {
        &self.operation_start_journal_ref
    }

    pub fn request_event_reference(&self) -> &JournalReference {
        &self.request_event_reference
    }

    pub fn result_event_reference(&self) -> &JournalReference {
        &self.result_event_reference
    }

    pub fn policy_authority_ref(&self) -> &JournalReference {
        self.request.policy_authority_ref()
    }

    pub fn common_review_scope_ref(&self) -> RecordId {
        self.policy_context_prerequisites.common_review_scope_ref()
    }

    pub fn freeze_authority(&self) -> &AuthoritativeFreezeCommittedBinding {
        &self.freeze_authority
    }

    pub fn request(&self) -> &ReviewRequestRecord {
        &self.request
    }

    pub fn result(&self) -> &ReviewResultRecord {
        &self.result
    }

    pub fn policy(&self) -> &ReviewAdmissionPolicyRecord {
        &self.policy
    }

    pub fn policy_bytes(&self) -> &[u8] {
        &self.policy_bytes
    }

    pub fn returned_anchor(&self) -> &JournalAnchor {
        &self.returned_anchor
    }

    pub fn anchor_comparison(&self) -> JournalAnchorHistoryComparison {
        self.anchor_comparison
    }

    pub fn policy_context_prerequisites(&self) -> ReviewAdmissionPolicyContextPrerequisites {
        self.policy_context_prerequisites
    }
}

/// A positive Freeze-authority witness derived only from one opened authoritative Registry store.
///
/// This type has no public constructor. Its exact START, Receipt, Manifest, and terminal event
/// identities have already passed retained-Journal, strict Record, identity, dependency, and
/// continuity validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthoritativeFreezeCommittedBinding {
    registry_id: RegistryId,
    committed_event_reference: JournalReference,
    start_event_reference: JournalReference,
    receipt_record_id: RecordId,
    manifest_record_id: RecordId,
    start_record_id: RecordId,
    freeze_attempt_id: FreezeAttemptId,
    freeze_id: [u8; ID_LENGTH],
    subject_id: [u8; ID_LENGTH],
    policy_record_id: RecordId,
}

impl AuthoritativeFreezeCommittedBinding {
    pub fn registry_id(&self) -> RegistryId {
        self.registry_id
    }

    pub fn committed_event_reference(&self) -> &JournalReference {
        &self.committed_event_reference
    }

    pub fn start_event_reference(&self) -> &JournalReference {
        &self.start_event_reference
    }

    pub fn receipt_record_id(&self) -> RecordId {
        self.receipt_record_id
    }

    pub fn manifest_record_id(&self) -> RecordId {
        self.manifest_record_id
    }

    pub fn start_record_id(&self) -> RecordId {
        self.start_record_id
    }

    pub fn freeze_attempt_id(&self) -> FreezeAttemptId {
        self.freeze_attempt_id
    }

    pub fn freeze_id(&self) -> [u8; ID_LENGTH] {
        self.freeze_id
    }

    pub fn subject_id(&self) -> [u8; ID_LENGTH] {
        self.subject_id
    }

    pub fn policy_record_id(&self) -> RecordId {
        self.policy_record_id
    }
}

/// A fail-closed error before positive Freeze authority can be established.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthoritativeFreezeCommittedBindingError {
    Structural(ResolvedFreezeCommittedBindingError),
}

impl AuthoritativeRegistryStore {
    /// Opens one authoritative Registry from exact namespace bytes.
    ///
    /// Journal slots must be a contiguous sequence of exact twenty-digit names. Every retained
    /// Entry is strictly decoded and replayed. Every Record namespace object must have the exact
    /// lowercase content-addressed filename and strict self-hash identity required by its bytes.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, AuthoritativeRegistryStoreOpenError> {
        let root = root.as_ref();
        ensure_real_directory(root)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
        let root = fs::canonicalize(root).map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        let registry_dir = root.join("registry");
        let journal_dir = root.join("journal");
        let records_dir = root.join("records");
        ensure_real_directory(&registry_dir)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RegistryNamespaceInvalid)?;
        ensure_real_directory(&journal_dir)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::JournalNamespaceInvalid)?;
        ensure_real_directory(&records_dir)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RecordNamespaceInvalid)?;

        let genesis_record_bytes = read_regular_file(&registry_dir.join("genesis.cbor"))
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RegistryNamespaceInvalid)?;
        let genesis_record = GenesisRecord::decode_authoritative(&genesis_record_bytes)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::GenesisRecordDecode)?;

        let mut records = load_record_namespace(&records_dir)?;
        match records
            .iter()
            .find(|(record_id, _)| *record_id == genesis_record.record_id())
        {
            Some((_, bytes)) if bytes != &genesis_record_bytes => {
                return Err(AuthoritativeRegistryStoreOpenError::GenesisRecordIdentityMismatch)
            }
            Some(_) => {}
            None => records.push((genesis_record.record_id(), genesis_record_bytes)),
        }
        records.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));

        let journal_slots = load_journal_slots(&journal_dir)?;
        let genesis_entry =
            GenesisJournalEntry::decode_authoritative(&journal_slots[0]).map_err(|_| {
                AuthoritativeRegistryStoreOpenError::RetainedJournal(
                    RetainedJournalError::DecodeError,
                )
            })?;
        if genesis_entry.event_record_id.as_bytes() != genesis_record.record_id().as_bytes() {
            return Err(AuthoritativeRegistryStoreOpenError::GenesisRecordIdentityMismatch);
        }
        if genesis_entry.registry_id != genesis_record.input.registry_id {
            return Err(AuthoritativeRegistryStoreOpenError::GenesisRegistryMismatch);
        }
        if genesis_record.input.journal_format_version != 1
            || genesis_record.input.record_identity_profile_id != 1
        {
            return Err(AuthoritativeRegistryStoreOpenError::GenesisProfileMismatch);
        }
        if genesis_entry.storage_capability_class_id
            != genesis_record.input.storage_capability_class_id
        {
            return Err(AuthoritativeRegistryStoreOpenError::GenesisCapabilityMismatch);
        }
        if genesis_entry.environment_observation_id
            != genesis_record.input.environment_observation_id
        {
            return Err(AuthoritativeRegistryStoreOpenError::GenesisEnvironmentMismatch);
        }

        let mut retained_journal = RetainedJournal::from_genesis(genesis_entry)
            .map_err(AuthoritativeRegistryStoreOpenError::RetainedJournal)?;
        for entry_bytes in journal_slots.iter().skip(1) {
            retained_journal
                .append_strict_entry(entry_bytes)
                .map_err(AuthoritativeRegistryStoreOpenError::RetainedJournal)?;
        }

        Ok(Self {
            root,
            retained_journal,
            records,
            live_instance_identity: Arc::new(AuthoritativeRegistryStoreInstanceIdentity {
                outstanding_review_admissions: AtomicUsize::new(0),
            }),
        })
    }

    /// The canonical root locator from which this store's authority namespaces were opened.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The exact retained Journal reconstructed from the authoritative Journal namespace.
    pub fn retained_journal(&self) -> &RetainedJournal {
        &self.retained_journal
    }

    /// Establishes positive Freeze authority from this store's exact retained namespaces.
    pub fn validate_freeze_committed_authority(
        &self,
        committed_event_reference: JournalReference,
    ) -> Result<AuthoritativeFreezeCommittedBinding, AuthoritativeFreezeCommittedBindingError> {
        match validate_resolved_freeze_committed_binding(
            &self.retained_journal,
            committed_event_reference.clone(),
            self,
        )
        .map_err(AuthoritativeFreezeCommittedBindingError::Structural)?
        {
            ResolvedFreezeCommittedBindingOutcome::AuthorityEvidenceUnavailable => {}
        }

        let receipt_record_id = RecordId::try_from(
            committed_event_reference
                .event_record_id()
                .as_bytes()
                .as_slice(),
        )
        .expect("EventRecordId has RecordId width");
        let receipt_bytes = self
            .resolve(receipt_record_id)
            .expect("the structural binding resolved the exact Receipt bytes");
        let receipt = FreezeReceiptRecord::decode_authoritative(receipt_bytes)
            .expect("the structural binding strictly decoded the Receipt");
        let start_event_reference = receipt.input().attempt_start_journal_ref.clone();
        let start_record_id = RecordId::try_from(
            start_event_reference
                .event_record_id()
                .as_bytes()
                .as_slice(),
        )
        .expect("EventRecordId has RecordId width");

        Ok(AuthoritativeFreezeCommittedBinding {
            registry_id: self.retained_journal.registry_id,
            committed_event_reference,
            start_event_reference,
            receipt_record_id,
            manifest_record_id: receipt.input().manifest_id,
            start_record_id,
            freeze_attempt_id: receipt.input().freeze_attempt_id,
            freeze_id: receipt.input().freeze_id,
            subject_id: receipt.input().subject_id,
            policy_record_id: receipt.input().policy_record_id,
        })
    }

    /// Accepts one bounded opaque Request/Result presentation and binds the exact authoritative
    /// store head at the same successful acceptance boundary.
    pub fn accept_authoritative_review_admission(
        &mut self,
        request_event_reference: JournalReference,
        request_bytes: &[u8],
        result_event_reference: JournalReference,
        result_bytes: &[u8],
    ) -> Result<AcceptedAuthoritativeReviewAdmission, AuthoritativeReviewAdmissionAcceptanceError>
    {
        if request_bytes
            .len()
            .checked_add(result_bytes.len())
            .is_none_or(|length| length > REVIEW_ADMISSION_MAX_OPAQUE_INPUT_BYTES)
        {
            return Err(AuthoritativeReviewAdmissionAcceptanceError::Input(
                ReviewAdmissionAcceptanceError::InputTooLarge,
            ));
        }
        self.live_instance_identity
            .outstanding_review_admissions
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                (current < REVIEW_ADMISSION_MAX_OUTSTANDING_ACCEPTANCES).then_some(current + 1)
            })
            .map_err(|_| {
                AuthoritativeReviewAdmissionAcceptanceError::Input(
                    ReviewAdmissionAcceptanceError::TooManyOutstandingAcceptances,
                )
            })?;
        let acceptance_token = NEXT_AUTHORITATIVE_REVIEW_ADMISSION_TOKEN
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| {
                self.release_authoritative_acceptance_capacity();
                AuthoritativeReviewAdmissionAcceptanceError::Input(
                    ReviewAdmissionAcceptanceError::AcceptanceTokenExhausted,
                )
            })?;

        let mut owned_request_bytes = Vec::new();
        if owned_request_bytes
            .try_reserve_exact(request_bytes.len())
            .is_err()
        {
            self.release_authoritative_acceptance_capacity();
            return Err(AuthoritativeReviewAdmissionAcceptanceError::Input(
                ReviewAdmissionAcceptanceError::InputAllocationFailed,
            ));
        }
        owned_request_bytes.extend_from_slice(request_bytes);
        let mut owned_result_bytes = Vec::new();
        if owned_result_bytes
            .try_reserve_exact(result_bytes.len())
            .is_err()
        {
            self.release_authoritative_acceptance_capacity();
            return Err(AuthoritativeReviewAdmissionAcceptanceError::Input(
                ReviewAdmissionAcceptanceError::InputAllocationFailed,
            ));
        }
        owned_result_bytes.extend_from_slice(result_bytes);

        if let Err(error) = self.reload_authoritative_namespaces() {
            self.release_authoritative_acceptance_capacity();
            return Err(error);
        }
        let operation_start_journal_ref = self.retained_journal.current_head_reference();
        Ok(AcceptedAuthoritativeReviewAdmission {
            acceptance_token,
            origin_store_instance_identity: Arc::clone(&self.live_instance_identity),
            request_event_reference,
            request_bytes: owned_request_bytes,
            result_event_reference,
            result_bytes: owned_result_bytes,
            operation_start_journal_ref,
        })
    }

    /// Consumes one authoritative acceptance through the complete §82 authority boundary.
    pub fn complete_authoritative_review_admission_section_82(
        &mut self,
        accepted: AcceptedAuthoritativeReviewAdmission,
    ) -> Result<AuthoritativeReviewAdmissionSection82, AuthoritativeReviewAdmissionSection82Error>
    {
        if !Arc::ptr_eq(
            &accepted.origin_store_instance_identity,
            &self.live_instance_identity,
        ) {
            return Err(AuthoritativeReviewAdmissionSection82Error::AcceptanceStoreMismatch);
        }
        let _consumed_acceptance_token = accepted.acceptance_token;
        self.reload_authoritative_namespaces()
            .map_err(|error| match error {
                AuthoritativeReviewAdmissionAcceptanceError::Store(error) => {
                    AuthoritativeReviewAdmissionSection82Error::Store(error)
                }
                AuthoritativeReviewAdmissionAcceptanceError::RegistryIdentityChanged => {
                    AuthoritativeReviewAdmissionSection82Error::RegistryIdentityChanged
                }
                AuthoritativeReviewAdmissionAcceptanceError::Input(_) => {
                    unreachable!("namespace reload does not inspect opaque input allocation")
                }
            })?;

        let request_record_id = record_id_from_event_reference(&accepted.request_event_reference);
        let result_record_id = record_id_from_event_reference(&accepted.result_event_reference);
        let request_bytes = self
            .resolve(request_record_id)
            .ok_or(AuthoritativeReviewAdmissionSection82Error::RequestPayloadUnavailable)?
            .to_vec();
        let result_bytes = self
            .resolve(result_record_id)
            .ok_or(AuthoritativeReviewAdmissionSection82Error::ResultPayloadUnavailable)?
            .to_vec();
        if accepted.request_bytes != request_bytes {
            return Err(AuthoritativeReviewAdmissionSection82Error::PresentedRequestMismatch);
        }
        if accepted.result_bytes != result_bytes {
            return Err(AuthoritativeReviewAdmissionSection82Error::PresentedResultMismatch);
        }

        validate_review_admission_section_82_structural_inputs(
            &self.retained_journal,
            &accepted.request_event_reference,
            &request_bytes,
            &accepted.result_event_reference,
            &result_bytes,
            self,
        )
        .map_err(AuthoritativeReviewAdmissionSection82Error::Structural)?;
        self.retained_journal
            .resolve_reference(&accepted.operation_start_journal_ref)
            .map_err(AuthoritativeReviewAdmissionSection82Error::OperationStartReference)?;
        if accepted.request_event_reference.entry_index().value()
            > accepted.operation_start_journal_ref.entry_index().value()
        {
            return Err(
                AuthoritativeReviewAdmissionSection82Error::RequestAuthorityAfterOperationStart,
            );
        }
        if accepted.result_event_reference.entry_index().value()
            > accepted.operation_start_journal_ref.entry_index().value()
        {
            return Err(
                AuthoritativeReviewAdmissionSection82Error::ResultAuthorityAfterOperationStart,
            );
        }

        let request = ReviewRequestRecord::decode_authoritative(&request_bytes)
            .expect("the structural §82 validator strictly decoded the Request");
        let result = ReviewResultRecord::decode_authoritative(&result_bytes)
            .expect("the structural §82 validator strictly decoded the Result");
        let returned_anchor = resolve_retained_review_package_anchor_input(
            &self.retained_journal,
            &accepted.request_event_reference,
            &request_bytes,
            &accepted.result_event_reference,
            &result_bytes,
        )
        .expect("the structural §82 validator resolved the returned Anchor");
        let anchor_comparison =
            compare_retained_journal_anchor_history(&self.retained_journal, &returned_anchor);
        let policy_record_id = record_id_from_event_reference(request.policy_authority_ref());
        let policy_bytes = self
            .resolve(policy_record_id)
            .expect("the structural §82 validator resolved the authoritative Policy")
            .to_vec();
        let policy = ReviewAdmissionPolicyRecord::decode_authoritative(&policy_bytes)
            .expect("the structural §82 validator strictly decoded the Policy");
        let policy_context_prerequisites = validate_review_admission_policy_context_prerequisites(
            &self.retained_journal,
            request.policy_authority_ref(),
            &policy_bytes,
            &request_bytes,
            &result_bytes,
            self,
        )
        .expect("the structural §82 validator established Policy context prerequisites");
        let freeze_authority = self
            .validate_freeze_committed_authority(request.freeze_authority_ref().clone())
            .map_err(AuthoritativeReviewAdmissionSection82Error::FreezeAuthority)?;

        Ok(AuthoritativeReviewAdmissionSection82 {
            operation_start_journal_ref: accepted.operation_start_journal_ref.clone(),
            request_event_reference: accepted.request_event_reference.clone(),
            result_event_reference: accepted.result_event_reference.clone(),
            request,
            result,
            policy,
            policy_bytes,
            returned_anchor,
            anchor_comparison,
            policy_context_prerequisites,
            freeze_authority,
        })
    }

    fn reload_authoritative_namespaces(
        &mut self,
    ) -> Result<(), AuthoritativeReviewAdmissionAcceptanceError> {
        let reloaded =
            Self::open(&self.root).map_err(AuthoritativeReviewAdmissionAcceptanceError::Store)?;
        if reloaded.retained_journal.registry_id != self.retained_journal.registry_id {
            return Err(AuthoritativeReviewAdmissionAcceptanceError::RegistryIdentityChanged);
        }
        self.retained_journal = reloaded.retained_journal;
        self.records = reloaded.records;
        Ok(())
    }

    fn release_authoritative_acceptance_capacity(&self) {
        let previous = self
            .live_instance_identity
            .outstanding_review_admissions
            .fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0);
    }
}

/// Executes every Policy requirement mechanically applicable to an authoritative §82 witness.
///
/// Evaluator 1001 emits exactly one result over the complete Review selector set: any exact
/// selector match passes Admission compatibility. Evaluator 1015 remains a distinct mandatory
/// profile-specific result. Individual results are retained and only this §46 composition step
/// creates a completed Policy result.
pub fn evaluate_authoritative_review_admission_policy_46(
    section_82: AuthoritativeReviewAdmissionSection82,
) -> ReviewAdmissionPolicy46Completion {
    let policy = section_82.policy();
    let request = section_82.request();
    let result = section_82.result();
    let mut evaluator_results = Vec::with_capacity(5);

    let selector_matches = policy.review_requirements().iter().any(|requirement| {
        requirement.review_role_id() == request.review_role_id()
            && requirement.review_scope_ref() == request.review_scope_ref()
            && requirement.review_method_ref() == request.review_method_ref()
            && requirement.required_checks_ref() == request.required_checks_ref()
    });
    evaluator_results.push(ReviewAdmissionIndividualEvaluatorResult {
        evaluator_id: 1001,
        outcome: pass_or_fail(selector_matches),
    });

    if !policy.required_method_statuses().is_empty() {
        evaluator_results.push(ReviewAdmissionIndividualEvaluatorResult {
            evaluator_id: 1003,
            outcome: pass_or_fail(
                policy
                    .required_method_statuses()
                    .contains(&result.method_status()),
            ),
        });
    }
    if !policy.allowed_finding_states().is_empty() {
        evaluator_results.push(ReviewAdmissionIndividualEvaluatorResult {
            evaluator_id: 1004,
            outcome: pass_or_fail(
                policy
                    .allowed_finding_states()
                    .contains(&result.finding_state()),
            ),
        });
    }
    if !policy.acceptable_anchor_relation_ids().is_empty() {
        evaluator_results.push(ReviewAdmissionIndividualEvaluatorResult {
            evaluator_id: 1009,
            outcome: pass_or_fail(
                policy
                    .acceptable_anchor_relation_ids()
                    .contains(&anchor_relation_id(section_82.anchor_comparison())),
            ),
        });
    }
    evaluator_results.push(ReviewAdmissionIndividualEvaluatorResult {
        evaluator_id: 1015,
        outcome: match evaluate_review_admission_gate_scope_1015(
            section_82.policy_context_prerequisites(),
        ) {
            ReviewAdmissionGateScope1015Result::Pass => {
                ReviewAdmissionIndividualEvaluatorOutcome::Pass
            }
            ReviewAdmissionGateScope1015Result::Fail => {
                ReviewAdmissionIndividualEvaluatorOutcome::Fail
            }
        },
    });

    let result = if evaluator_results
        .iter()
        .any(|result| result.outcome == ReviewAdmissionIndividualEvaluatorOutcome::Fail)
    {
        ReviewAdmissionCompletedPolicyResult::GateUnsatisfied
    } else if evaluator_results
        .iter()
        .any(|result| result.outcome == ReviewAdmissionIndividualEvaluatorOutcome::Indeterminate)
    {
        ReviewAdmissionCompletedPolicyResult::GateIndeterminate
    } else {
        ReviewAdmissionCompletedPolicyResult::Satisfied
    };
    ReviewAdmissionPolicy46Completion {
        result,
        evaluator_results,
    }
}

fn pass_or_fail(condition: bool) -> ReviewAdmissionIndividualEvaluatorOutcome {
    if condition {
        ReviewAdmissionIndividualEvaluatorOutcome::Pass
    } else {
        ReviewAdmissionIndividualEvaluatorOutcome::Fail
    }
}

fn anchor_relation_id(comparison: JournalAnchorHistoryComparison) -> u64 {
    match comparison {
        JournalAnchorHistoryComparison::AnchorEqualsCurrentHead => 1,
        JournalAnchorHistoryComparison::AnchorIsValidAncestor => 2,
        JournalAnchorHistoryComparison::JournalDivergence => 3,
        JournalAnchorHistoryComparison::JournalHistoryBehindAnchor => 4,
        JournalAnchorHistoryComparison::AnchorFromDifferentRegistry => 5,
        JournalAnchorHistoryComparison::AnchorInvalid => 6,
    }
}

impl ExactRecordByteResolver for AuthoritativeRegistryStore {
    fn resolve(&self, record_id: RecordId) -> Option<&[u8]> {
        self.records
            .binary_search_by(|(stored_id, _)| stored_id.as_bytes().cmp(record_id.as_bytes()))
            .ok()
            .map(|index| self.records[index].1.as_slice())
    }
}

fn ensure_real_directory(path: &Path) -> Result<(), ()> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(());
    }
    Ok(())
}

fn read_regular_file(path: &Path) -> Result<Vec<u8>, ()> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(());
    }
    fs::read(path).map_err(|_| ())
}

fn load_journal_slots(
    journal_dir: &Path,
) -> Result<Vec<Vec<u8>>, AuthoritativeRegistryStoreOpenError> {
    let mut slots = Vec::new();
    for entry in fs::read_dir(journal_dir).map_err(|_| AuthoritativeRegistryStoreOpenError::Io)? {
        let entry = entry.map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        let file_type = entry
            .file_type()
            .map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        if !file_type.is_file() || file_type.is_symlink() {
            return Err(AuthoritativeRegistryStoreOpenError::JournalNamespaceInvalid);
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| AuthoritativeRegistryStoreOpenError::JournalSlotNameInvalid)?;
        let index = parse_journal_slot_name(&name)
            .ok_or(AuthoritativeRegistryStoreOpenError::JournalSlotNameInvalid)?;
        let bytes = read_regular_file(&entry.path())
            .map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        slots.push((index, bytes));
    }
    slots.sort_by_key(|(index, _)| *index);
    if slots.is_empty()
        || slots
            .iter()
            .enumerate()
            .any(|(expected, (actual, _))| u64::try_from(expected).ok() != Some(*actual))
    {
        return Err(AuthoritativeRegistryStoreOpenError::JournalSlotSequenceInvalid);
    }
    Ok(slots.into_iter().map(|(_, bytes)| bytes).collect())
}

fn parse_journal_slot_name(name: &str) -> Option<u64> {
    let digits = name.strip_suffix(".cbor")?;
    if digits.len() != 20 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

fn load_record_namespace(
    records_dir: &Path,
) -> Result<Vec<(RecordId, Vec<u8>)>, AuthoritativeRegistryStoreOpenError> {
    let mut records = Vec::new();
    for entry in fs::read_dir(records_dir).map_err(|_| AuthoritativeRegistryStoreOpenError::Io)? {
        let entry = entry.map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        let file_type = entry
            .file_type()
            .map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        if !file_type.is_file() || file_type.is_symlink() {
            return Err(AuthoritativeRegistryStoreOpenError::RecordNamespaceInvalid);
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RecordFilenameInvalid)?;
        let expected_id = parse_record_filename(&name)
            .ok_or(AuthoritativeRegistryStoreOpenError::RecordFilenameInvalid)?;
        let bytes = read_regular_file(&entry.path())
            .map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        let frame = StrictRecordFrame::decode_authoritative(&bytes)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RecordDecode)?;
        if frame.record_id() != expected_id {
            return Err(AuthoritativeRegistryStoreOpenError::RecordIdentityMismatch);
        }
        records.push((expected_id, bytes));
    }
    records.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    if records
        .windows(2)
        .any(|pair| pair[0].0.as_bytes() == pair[1].0.as_bytes())
    {
        return Err(AuthoritativeRegistryStoreOpenError::RecordNamespaceInvalid);
    }
    Ok(records)
}

fn parse_record_filename(name: &str) -> Option<RecordId> {
    let hex = name.strip_suffix(".cbor")?;
    if hex.len() != ID_LENGTH * 2
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    let mut bytes = [0_u8; ID_LENGTH];
    for (index, slot) in bytes.iter_mut().enumerate() {
        let offset = index * 2;
        *slot = u8::from_str_radix(&hex[offset..offset + 2], 16).ok()?;
    }
    RecordId::try_from(bytes.as_slice()).ok()
}

fn record_id_from_event_reference(reference: &JournalReference) -> RecordId {
    RecordId::try_from(reference.event_record_id().as_bytes().as_slice())
        .expect("EventRecordId has RecordId width")
}
