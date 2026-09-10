use super::*;
use std::fs;
#[cfg(not(target_os = "macos"))]
use std::fs::OpenOptions;
use std::io::{Read, Write};
#[cfg(not(target_os = "macos"))]
use std::io::{Seek, SeekFrom};
#[cfg(target_os = "linux")]
use std::path::Component;
use std::path::{Path, PathBuf};

/// Maximum bytes admitted from any single authoritative namespace object.
pub const AUTHORITATIVE_STORE_MAX_OBJECT_BYTES: usize = 1_048_576;
/// Maximum regular objects admitted across one authoritative store opening.
pub const AUTHORITATIVE_STORE_MAX_OBJECTS: usize = 4_096;
/// Maximum aggregate bytes retained across one authoritative store opening.
pub const AUTHORITATIVE_STORE_MAX_NAMESPACE_BYTES: usize = 64 * 1_048_576;

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
    NamespaceResourceLimit,
    GenesisRecordDecode,
    GenesisRecordIdentityMismatch,
    GenesisRegistryMismatch,
    GenesisProfileMismatch,
    GenesisCapabilityMismatch,
    GenesisEnvironmentMismatch,
    GenesisCapabilityObservationInvalid,
    GenesisEnvironmentObservationInvalid,
    EventRecordUnavailable,
    EventRecordBinding(EventRecordStructuralBindingError),
    EventRecordDecode,
    /// Selected terminal cold replay exceeded its explicit retained-payload resource budget.
    SelectedTerminalReplayResourceLimit,
    /// Frozen-valid structure requires contextual authority semantics not supplied by this runtime.
    EventSemanticAuthorityUnavailable,
    /// This platform lacks an implemented retained-generation protection adapter.
    RetainedGenerationProtectionUnavailable,
    RetainedGenerationChanged,
    RetainedJournal(RetainedJournalError),
}

/// A read-only view of the exact retained head inspected through the selected
/// terminal-closure replay path.
///
/// This reports retained semantic consistency at `captured_head_reference`.
/// It is not a live publication receipt and cannot attest historical flushes,
/// producer identity, or the state of the filesystem after this inspection
/// returned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedTerminalInspection {
    captured_head_reference: JournalReference,
    terminal_admission_event_reference: Option<JournalReference>,
    terminal_disposition_id: Option<u64>,
}

impl SelectedTerminalInspection {
    /// The exact Journal head captured by the selected cold reopen.
    pub fn captured_head_reference(&self) -> &JournalReference {
        &self.captured_head_reference
    }

    /// The latest retained selected terminal Admission, if this captured prefix contains one.
    pub fn terminal_admission_event_reference(&self) -> Option<&JournalReference> {
        self.terminal_admission_event_reference.as_ref()
    }

    /// The retained Admission disposition ID, if this captured prefix contains one.
    pub fn terminal_disposition_id(&self) -> Option<u64> {
        self.terminal_disposition_id
    }
}

/// A fail-closed outcome while creating a selected Store from an absent root.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthoritativeRegistryStoreInitializeError {
    RootAlreadyExists,
    RootCreation,
    NamespaceCreation,
    CapabilityProbe,
    ObservationRecord,
    Publication,
    Reopen(AuthoritativeRegistryStoreOpenError),
}

/// Minimal caller-supplied material for one selected EMBEDDED Freeze preparation.
///
/// Every Record, Journal, root, payload, Subject, and event identity is derived by the Store.
/// `source_root` is an untrusted source locator, not a retained or publishable destination.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedEmbeddedFreezePreparationInput {
    pub source_root: PathBuf,
    pub freeze_attempt_id: FreezeAttemptId,
    pub policy_record_id: RecordId,
}

/// Exact Store-derived artifacts from a successful selected EMBEDDED Freeze preparation.
///
/// This is a START-and-payload preparation result only. It is not a Receipt, event 101, positive
/// Freeze authority, Policy completion, review admission, or terminal publication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedSelectedEmbeddedFreeze {
    start_record: FreezeAttemptStartRecord,
    start_event_reference: JournalReference,
    manifest_record_id: RecordId,
    payload_directory: PathBuf,
}

impl PreparedSelectedEmbeddedFreeze {
    pub fn start_record(&self) -> &FreezeAttemptStartRecord {
        &self.start_record
    }

    pub fn start_event_reference(&self) -> &JournalReference {
        &self.start_event_reference
    }

    /// The exact Store-derived MANIFEST identity for the retained payload inventory.
    pub fn manifest_record_id(&self) -> RecordId {
        self.manifest_record_id
    }

    /// The Store-owned retained payload directory for diagnostic local access.
    pub fn payload_directory(&self) -> &Path {
        &self.payload_directory
    }
}

/// A fail-closed failure while preparing the selected EMBEDDED Freeze START and retained payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectedEmbeddedFreezePreparationError {
    SelectedProfileRequired,
    PublicationLockUnavailable,
    Store(AuthoritativeRegistryStoreOpenError),
    PolicyUnavailable,
    PolicyInvalid,
    PolicyUnsupported,
    SourceInvalid,
    SourceResourceLimit,
    EntryIndexExhausted,
    StartJournalEntry,
    StartRecordPublication,
    StartJournalPublication,
    StartPublicationConflict,
    RootConflict,
    RootCreation,
    PayloadPublication,
    ManifestConstruction,
    ManifestPublication,
    SourceChanged,
    ReplayMismatch,
    RetainedGenerationChanged,
}

/// A fail-closed failure while turning one Store-derived selected payload preparation into
/// FREEZE_COMMITTED. The input is opaque: callers cannot supply a Receipt, Manifest, root,
/// Subject, policy conclusion, or terminal Journal state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectedEmbeddedFreezeCommitError {
    SelectedProfileRequired,
    PublicationLockUnavailable,
    Store(AuthoritativeRegistryStoreOpenError),
    PreparationMismatch,
    RetainedGenerationChanged,
    PayloadInvalid,
    PolicyInvalid,
    CreationProfilePublication,
    ReceiptConstruction,
    ReceiptPublication,
    JournalConstruction,
    JournalPublication,
    JournalConflict,
    ReplayMismatch,
    PositiveReplay(AuthoritativeFreezeCommittedBindingError),
}

/// Minimal semantic input for a Store-owned selected REVIEW_REQUEST producer.
///
/// The Freeze witness is opaque and revalidated against current retained Store
/// state. The caller can select only a retained Review Policy identity and one
/// registered role; the Store derives its exact Policy event, selector
/// requirement, package Anchor, Request bytes, and Journal state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedReviewRequestInput {
    pub freeze_authority: AuthoritativeFreezeCommittedBinding,
    pub review_policy_record_id: RecordId,
    pub review_role_id: u64,
}

/// Exact Store-derived output from a selected REVIEW_REQUEST_RECORDED append.
///
/// This is Request transport/binding evidence only. It is not a Result,
/// §82 completion, Policy satisfaction, Admission, or terminal publication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedSelectedReviewRequest {
    request: ReviewRequestRecord,
    request_event_reference: JournalReference,
    policy_authority_ref: JournalReference,
    freeze_authority: AuthoritativeFreezeCommittedBinding,
}

impl RecordedSelectedReviewRequest {
    pub fn request(&self) -> &ReviewRequestRecord {
        &self.request
    }

    pub fn request_event_reference(&self) -> &JournalReference {
        &self.request_event_reference
    }

    pub fn policy_authority_ref(&self) -> &JournalReference {
        &self.policy_authority_ref
    }

    pub fn freeze_authority(&self) -> &AuthoritativeFreezeCommittedBinding {
        &self.freeze_authority
    }
}

/// A fail-closed failure while producing or revalidating a selected Request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectedReviewRequestError {
    SelectedProfileRequired,
    PublicationLockUnavailable,
    Store(AuthoritativeRegistryStoreOpenError),
    RetainedGenerationChanged,
    FreezeAuthority(AuthoritativeFreezeCommittedBindingError),
    FreezeWitnessMismatch,
    PolicyUnavailable,
    PolicyInvalid,
    PolicyUnsupported,
    PolicyAuthorityMissing,
    PolicyAuthorityAmbiguous,
    SelectorUnavailable,
    SelectorAmbiguous,
    AnchorConstruction,
    RequestConstruction,
    RequestPublication,
    JournalConstruction,
    JournalPublication,
    JournalConflict,
    ReplayResourceLimit,
    ReplayMismatch,
}

/// Minimal semantic submission for a Store-owned selected REVIEW_RESULT producer.
/// The Request reference is only a retained selector; the Store derives every
/// Freeze, Manifest, Scope, Method, Anchor, operation-start, Record, and slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedReviewResultInput {
    pub request_event_reference: JournalReference,
    pub method_status: u64,
    pub finding_state: u64,
    pub reason_codes: Vec<String>,
    pub findings: Vec<RecordId>,
    pub reviewer_metadata: Option<String>,
}

/// Exact Store-derived output from a selected REVIEW_RESULT_RECORDED append.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedSelectedReviewResult {
    result: ReviewResultRecord,
    result_event_reference: JournalReference,
    request: RecordedSelectedReviewRequest,
}

impl RecordedSelectedReviewResult {
    pub fn result(&self) -> &ReviewResultRecord {
        &self.result
    }
    pub fn result_event_reference(&self) -> &JournalReference {
        &self.result_event_reference
    }
    pub fn request(&self) -> &RecordedSelectedReviewRequest {
        &self.request
    }
}

/// A fail-closed failure while producing or revalidating a selected Result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectedReviewResultError {
    SelectedProfileRequired,
    PublicationLockUnavailable,
    Store(AuthoritativeRegistryStoreOpenError),
    RetainedGenerationChanged,
    Request(SelectedReviewRequestError),
    FindingUnavailable,
    ResultConstruction,
    ResultPublication,
    JournalConstruction,
    JournalPublication,
    JournalConflict,
    ReplayMismatch,
}

/// A fail-closed failure while deriving the selected nonpublishing §82 witness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectedReviewAdmissionSection82Error {
    SelectedProfileRequired,
    Result(SelectedReviewResultError),
    Acceptance(AuthoritativeReviewAdmissionAcceptanceError),
    Section82(AuthoritativeReviewAdmissionSection82Error),
}

/// A fail-closed failure while completing and publishing the selected
/// REVIEW_ADMISSION route. The selected route is the only runtime surface
/// permitted to interpret the frozen profile-1 exact gate Scope relation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectedReviewAdmissionCompletionError {
    SelectedProfileRequired,
    PublicationLockUnavailable,
    Store(AuthoritativeRegistryStoreOpenError),
    RetainedGenerationChanged,
    Reload(AuthoritativeReviewAdmissionAcceptanceError),
    Section82(SelectedReviewAdmissionSection82Error),
    Runtime(AuthoritativeReviewAdmissionRuntimeError),
}

/// An authoritative Registry store opened from its exact retained Journal and Record namespaces.
///
/// The root path is only a locator. Registry identity, current head, Journal history, and Record
/// identities are derived from strict retained bytes. No caller-selected Journal head, Record body,
/// or prevalidated authority token participates in resolution.
#[derive(Debug)]
pub struct AuthoritativeRegistryStore {
    root: PathBuf,
    open_profile: AuthoritativeRegistryStoreOpenProfile,
    retained_journal: RetainedJournal,
    records: Vec<(RecordId, Vec<u8>)>,
    namespace_holds: Vec<fs::File>,
    retained_file_witnesses: Vec<RetainedFileWitness>,
    live_instance_identity: Arc<AuthoritativeRegistryStoreInstanceIdentity>,
}

/// Distinguishes the bounded predecessor reader from the explicitly selected Store profile.
/// A GENESIS scalar value alone never upgrades a legacy opening into the selected profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuthoritativeRegistryStoreOpenProfile {
    LegacyThreeNamespace,
    SelectedTerminalAuthorityClosure,
}

#[derive(Debug)]
struct RetainedFileWitness {
    path: PathBuf,
    file: fs::File,
    expected_length: usize,
    expected_sha256: [u8; ID_LENGTH],
}

struct RetainedFileRead {
    bytes: Vec<u8>,
    witness: RetainedFileWitness,
}

type LoadedRecordNamespace = (Vec<(RecordId, Vec<u8>)>, Vec<RetainedFileWitness>);

struct NamespaceBudget {
    objects: usize,
    bytes: usize,
}

impl NamespaceBudget {
    fn new() -> Self {
        Self {
            objects: 0,
            bytes: 0,
        }
    }

    fn reserve(&mut self, length: u64) -> Result<usize, NamespaceReadError> {
        let length = usize::try_from(length).map_err(|_| NamespaceReadError::ResourceLimit)?;
        if length > AUTHORITATIVE_STORE_MAX_OBJECT_BYTES {
            return Err(NamespaceReadError::ResourceLimit);
        }
        self.objects = self
            .objects
            .checked_add(1)
            .filter(|count| *count <= AUTHORITATIVE_STORE_MAX_OBJECTS)
            .ok_or(NamespaceReadError::ResourceLimit)?;
        self.bytes = self
            .bytes
            .checked_add(length)
            .filter(|bytes| *bytes <= AUTHORITATIVE_STORE_MAX_NAMESPACE_BYTES)
            .ok_or(NamespaceReadError::ResourceLimit)?;
        Ok(length)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NamespaceReadError {
    Io,
    Invalid,
    ResourceLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectedTerminalReplayPayloadBudgetError {
    FreezePrerequisiteInvalid,
    ResourceLimit,
}

#[derive(Debug)]
struct AuthoritativeRegistryStoreInstanceIdentity {
    outstanding_review_admissions: AtomicUsize,
}

static NEXT_AUTHORITATIVE_REVIEW_ADMISSION_TOKEN: AtomicU64 = AtomicU64::new(1);
#[cfg(any(windows, target_os = "macos", test))]
static NEXT_AUTHORITATIVE_PUBLICATION_TEMP: AtomicU64 = AtomicU64::new(1);
const MAX_AUTHORITATIVE_PUBLICATION_RETRIES: usize = 8;

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
    /// Frozen authority does not define the generic Policy gate-Scope relation needed for §46.
    PolicyScopeApplicabilityUnavailable,
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

/// One exact observed durability action for terminal local publication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DurabilityActionState {
    Performed,
    PreviouslyEstablished,
    Unsupported,
}

/// Exact publication-time durability facts retained with a successful publication receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthoritativePublicationDurability {
    record_content_flush: DurabilityActionState,
    journal_content_flush: DurabilityActionState,
    record_atomic_publish_no_replace: DurabilityActionState,
    journal_atomic_publish_no_replace: DurabilityActionState,
    record_parent_directory_flush: DurabilityActionState,
    journal_parent_directory_flush: DurabilityActionState,
    platform_strongest_available: bool,
}

impl AuthoritativePublicationDurability {
    pub fn record_content_flush(&self) -> DurabilityActionState {
        self.record_content_flush
    }

    pub fn journal_content_flush(&self) -> DurabilityActionState {
        self.journal_content_flush
    }

    pub fn record_atomic_publish_no_replace(&self) -> DurabilityActionState {
        self.record_atomic_publish_no_replace
    }

    pub fn journal_atomic_publish_no_replace(&self) -> DurabilityActionState {
        self.journal_atomic_publish_no_replace
    }

    pub fn record_parent_directory_flush(&self) -> DurabilityActionState {
        self.record_parent_directory_flush
    }

    pub fn journal_parent_directory_flush(&self) -> DurabilityActionState {
        self.journal_parent_directory_flush
    }

    pub fn platform_strongest_available(&self) -> bool {
        self.platform_strongest_available
    }
}

/// A successful exact terminal Record and Journal publication receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthoritativeReviewAdmissionPublication {
    policy_completion: ReviewAdmissionPolicy46Completion,
    disposition: ReviewAdmissionSection83Disposition,
    admission_record: ReviewAdmissionRecord,
    admission_record_bytes: Vec<u8>,
    journal_entry: ReviewAdmissionJournalEntry,
    journal_entry_bytes: Vec<u8>,
    journal_reference: JournalReference,
    operation_start_journal_ref: JournalReference,
    freeze_authority: AuthoritativeFreezeCommittedBinding,
    durability: AuthoritativePublicationDurability,
}

impl AuthoritativeReviewAdmissionPublication {
    pub fn policy_completion(&self) -> &ReviewAdmissionPolicy46Completion {
        &self.policy_completion
    }

    pub fn disposition(&self) -> ReviewAdmissionSection83Disposition {
        self.disposition
    }

    pub fn admission_record(&self) -> &ReviewAdmissionRecord {
        &self.admission_record
    }

    pub fn admission_record_bytes(&self) -> &[u8] {
        &self.admission_record_bytes
    }

    pub fn journal_entry(&self) -> &ReviewAdmissionJournalEntry {
        &self.journal_entry
    }

    pub fn journal_entry_bytes(&self) -> &[u8] {
        &self.journal_entry_bytes
    }

    pub fn journal_reference(&self) -> &JournalReference {
        &self.journal_reference
    }

    pub fn operation_start_journal_ref(&self) -> &JournalReference {
        &self.operation_start_journal_ref
    }

    pub fn freeze_authority(&self) -> &AuthoritativeFreezeCommittedBinding {
        &self.freeze_authority
    }

    pub fn durability(&self) -> AuthoritativePublicationDurability {
        self.durability
    }
}

/// A terminal Journal name became or may have become visible, but the runtime could not
/// authenticate a complete publication receipt. Callers must recover by reopening and replaying
/// the exact Journal reference and Record bytes; this value is not a durability receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthoritativeReviewAdmissionPublishedReceiptUncertain {
    admission_record_id: RecordId,
    admission_record_bytes: Vec<u8>,
    journal_reference: JournalReference,
    journal_entry_bytes: Vec<u8>,
    operation_start_journal_ref: JournalReference,
}

impl AuthoritativeReviewAdmissionPublishedReceiptUncertain {
    pub fn admission_record_id(&self) -> RecordId {
        self.admission_record_id
    }

    pub fn admission_record_bytes(&self) -> &[u8] {
        &self.admission_record_bytes
    }

    pub fn journal_reference(&self) -> &JournalReference {
        &self.journal_reference
    }

    pub fn journal_entry_bytes(&self) -> &[u8] {
        &self.journal_entry_bytes
    }

    pub fn operation_start_journal_ref(&self) -> &JournalReference {
        &self.operation_start_journal_ref
    }
}

/// The production authoritative Review Admission runtime result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthoritativeReviewAdmissionRuntimeOutcome {
    PreTerminal(AuthoritativeReviewAdmissionSection82Error),
    Published(Box<AuthoritativeReviewAdmissionPublication>),
    PublishedReceiptUncertain(Box<AuthoritativeReviewAdmissionPublishedReceiptUncertain>),
}

/// A failure after a terminal disposition was derived but before a durable publication receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthoritativeReviewAdmissionPublicationError {
    Store(AuthoritativeRegistryStoreOpenError),
    RegistryIdentityChanged,
    OperationStartReference(RetainedJournalError),
    EntryIndexExhausted,
    AdmissionRecord,
    JournalEntry,
    LifecyclePreflight(RetainedJournalError),
    /// The complete selected candidate prefix did not pass the same semantic
    /// replay validation required after Journal visibility.
    SemanticPreflight,
    /// No platform primitive established the required mandatory runtime-publication lock.
    PublicationLockUnavailable,
    RetainedGenerationChanged,
    RecordPublication,
    JournalPublication,
    PublicationConflictExhausted,
    ReplayMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthoritativeReviewAdmissionRuntimeError {
    Publication(AuthoritativeReviewAdmissionPublicationError),
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
    RetainedGenerationChanged,
    /// A selected Receipt did not satisfy the adopted selected Store, profile, policy, or
    /// currently retained EMBEDDED-payload prerequisites.
    SelectedPrerequisiteInvalid,
    /// Revalidating a selected retained EMBEDDED payload exceeded the bounded resource envelope.
    ResourceLimit,
    /// Frozen authority does not assign the generic gate-Scope and selected Manifest-profile
    /// semantics needed to promote exact structural bindings into positive Freeze authority.
    SemanticAuthorityUnavailable,
}

impl AuthoritativeRegistryStore {
    /// Opens one authoritative Registry from exact namespace bytes.
    ///
    /// Journal slots must be a contiguous sequence of exact twenty-digit names. Every retained
    /// Entry is strictly decoded and replayed. Every Record namespace object must have the exact
    /// lowercase content-addressed filename and strict self-hash identity required by its bytes.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, AuthoritativeRegistryStoreOpenError> {
        Self::open_with_generation_hook(
            root.as_ref(),
            AuthoritativeRegistryStoreOpenProfile::LegacyThreeNamespace,
            || {},
        )
    }

    /// Opens the adopted selected Store profile without upgrading predecessor histories.
    ///
    /// This boundary requires the complete canonical retained and operational namespace. It never
    /// initializes or repairs an incomplete root.
    pub fn open_selected_profile(
        root: impl AsRef<Path>,
    ) -> Result<Self, AuthoritativeRegistryStoreOpenError> {
        Self::open_with_generation_hook(
            root.as_ref(),
            AuthoritativeRegistryStoreOpenProfile::SelectedTerminalAuthorityClosure,
            || {},
        )
    }

    /// Performs a separate, read-only selected cold reopen and reports the
    /// resulting exact retained view. The Journal-only CLI remains unchanged.
    pub fn inspect_selected_terminal(
        root: impl AsRef<Path>,
    ) -> Result<SelectedTerminalInspection, AuthoritativeRegistryStoreOpenError> {
        let store = Self::open_selected_profile(root)?;
        let captured_head_reference = store.retained_journal.current_head_reference();
        let mut terminal_admission_event_reference = None;
        let mut terminal_disposition_id = None;
        for entry in store.retained_journal.entries.iter().rev() {
            if !matches!(entry.event_type_id().value(), 302 | 303) {
                continue;
            }
            let event_reference = JournalReference::new(
                entry.registry_id(),
                entry.entry_index(),
                entry.entry_hash(),
                entry.event_type_id(),
                entry.event_record_id(),
            );
            let admission_bytes = store
                .resolve(record_id_from_event_reference(&event_reference))
                .ok_or(AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            let admission = ReviewAdmissionRecord::decode_authoritative(admission_bytes)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            if admission.terminal_authority_closure_sha256()
                != Some(&TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256)
            {
                continue;
            }
            terminal_admission_event_reference = Some(event_reference);
            terminal_disposition_id = Some(admission.disposition_id());
            break;
        }
        Ok(SelectedTerminalInspection {
            captured_head_reference,
            terminal_admission_event_reference,
            terminal_disposition_id,
        })
    }

    /// Creates a selected Store only at an absent root, publishes initial
    /// Records and slot zero, then returns only an exact selected reopen.
    ///
    /// Capability dimensions are marked PRESENT only after an operation-scoped
    /// probe at this exact selected root. Unsupported dimensions remain
    /// NOT_PROBED and cannot authorize a later selected operation.
    pub fn initialize_selected_profile(
        root: impl AsRef<Path>,
        registry_id: RegistryId,
        created_by_tool_version: impl Into<String>,
    ) -> Result<Self, AuthoritativeRegistryStoreInitializeError> {
        let root = root.as_ref();
        match fs::create_dir(root) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(AuthoritativeRegistryStoreInitializeError::RootAlreadyExists)
            }
            Err(_) => return Err(AuthoritativeRegistryStoreInitializeError::RootCreation),
        }
        for relative in [
            "registry",
            "journal",
            "records",
            "roots",
            "coordination",
            "coordination/freeze",
            "coordination/staging",
        ] {
            fs::create_dir(root.join(relative))
                .map_err(|_| AuthoritativeRegistryStoreInitializeError::NamespaceCreation)?;
        }
        let capability_input = probe_selected_initial_storage_capabilities(root)
            .map_err(|_| AuthoritativeRegistryStoreInitializeError::CapabilityProbe)?;
        let capability = StorageCapabilityClassRecord::new(capability_input)
            .map_err(|_| AuthoritativeRegistryStoreInitializeError::ObservationRecord)?;
        let environment = EnvironmentObservationRecord::new(EnvironmentObservationRecordInput {
            os_name: std::env::consts::OS.to_owned(),
            os_version: None,
            filesystem_reported_name: None,
            driver_details: None,
            mount_identity: None,
            volume_identity: None,
            resolved_registry_storage_identity: None,
            probe_tool_version: created_by_tool_version.into(),
            observation_limitations: vec![
                "atomic-rename-not-probed".to_owned(),
                "no-hostile-attestation".to_owned(),
                "no-replace-publication-not-probed".to_owned(),
                "placeholder-capability-not-probed".to_owned(),
                "writer-lock-not-probed".to_owned(),
            ],
        })
        .map_err(|_| AuthoritativeRegistryStoreInitializeError::ObservationRecord)?;
        let genesis = GenesisRecord::new(GenesisRecordInput {
            registry_id,
            journal_format_version: 1,
            record_identity_profile_id: 1,
            storage_capability_class_id: capability.record_id(),
            environment_observation_id: environment.record_id(),
            created_by_tool_version: environment.input().probe_tool_version.clone(),
        })
        .map_err(|_| AuthoritativeRegistryStoreInitializeError::ObservationRecord)?;
        let entry = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from(genesis.record_id().as_bytes().as_slice())
                .expect("RecordId has exact EventRecordId width"),
            capability.record_id(),
            environment.record_id(),
        );
        let root_hold = open_real_directory_hold(root)
            .map_err(|_| AuthoritativeRegistryStoreInitializeError::NamespaceCreation)?;
        let records_hold = open_child_directory_hold(&root_hold, root, "records")
            .map_err(|_| AuthoritativeRegistryStoreInitializeError::NamespaceCreation)?;
        let registry_hold = open_child_directory_hold(&root_hold, root, "registry")
            .map_err(|_| AuthoritativeRegistryStoreInitializeError::NamespaceCreation)?;
        let journal_hold = open_child_directory_hold(&root_hold, root, "journal")
            .map_err(|_| AuthoritativeRegistryStoreInitializeError::NamespaceCreation)?;
        let records_publication_hold =
            open_publication_child_directory_hold(&root_hold, root, &records_hold, "records")
                .map_err(|_| AuthoritativeRegistryStoreInitializeError::NamespaceCreation)?;
        let registry_publication_hold =
            open_publication_child_directory_hold(&root_hold, root, &registry_hold, "registry")
                .map_err(|_| AuthoritativeRegistryStoreInitializeError::NamespaceCreation)?;
        let journal_publication_hold =
            open_publication_child_directory_hold(&root_hold, root, &journal_hold, "journal")
                .map_err(|_| AuthoritativeRegistryStoreInitializeError::NamespaceCreation)?;
        for (record_id, bytes) in [
            (capability.record_id(), capability.authoritative_cbor()),
            (environment.record_id(), environment.authoritative_cbor()),
            (genesis.record_id(), genesis.authoritative_cbor()),
        ] {
            publish_selected_initial_file(
                &root.join("records"),
                &records_publication_hold,
                Path::new(&record_filename(record_id)),
                &bytes,
            )?;
        }
        publish_selected_initial_file(
            &root.join("registry"),
            &registry_publication_hold,
            Path::new("genesis.cbor"),
            &genesis.authoritative_cbor(),
        )?;
        publish_selected_initial_file(
            &root.join("journal"),
            &journal_publication_hold,
            Path::new("00000000000000000000.cbor"),
            &entry.authoritative_cbor(),
        )?;
        Self::open_selected_profile(root).map_err(AuthoritativeRegistryStoreInitializeError::Reopen)
    }

    fn open_with_generation_hook<F>(
        root: &Path,
        open_profile: AuthoritativeRegistryStoreOpenProfile,
        before_generation_guard: F,
    ) -> Result<Self, AuthoritativeRegistryStoreOpenError>
    where
        F: FnOnce(),
    {
        let root_hold = open_real_directory_hold(root)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
        let root = fs::canonicalize(root).map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        ensure_path_matches_handle(&root, &root_hold)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
        let registry_dir = root.join("registry");
        let journal_dir = root.join("journal");
        let records_dir = root.join("records");
        let registry_hold = open_child_directory_hold(&root_hold, &root, "registry")
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RegistryNamespaceInvalid)?;
        let journal_hold = open_child_directory_hold(&root_hold, &root, "journal")
            .map_err(|_| AuthoritativeRegistryStoreOpenError::JournalNamespaceInvalid)?;
        let records_hold = open_child_directory_hold(&root_hold, &root, "records")
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RecordNamespaceInvalid)?;
        ensure_path_matches_handle(&registry_dir, &registry_hold)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RegistryNamespaceInvalid)?;
        ensure_path_matches_handle(&journal_dir, &journal_hold)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::JournalNamespaceInvalid)?;
        ensure_path_matches_handle(&records_dir, &records_hold)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RecordNamespaceInvalid)?;
        let selected_namespace_holds = if open_profile
            == AuthoritativeRegistryStoreOpenProfile::SelectedTerminalAuthorityClosure
        {
            let roots_dir = root.join("roots");
            let coordination_dir = root.join("coordination");
            let roots_hold = open_child_directory_hold(&root_hold, &root, "roots")
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
            let coordination_hold = open_child_directory_hold(&root_hold, &root, "coordination")
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
            let freeze_dir = coordination_dir.join("freeze");
            let staging_dir = coordination_dir.join("staging");
            let freeze_hold =
                open_child_directory_hold(&coordination_hold, &coordination_dir, "freeze")
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
            let staging_hold =
                open_child_directory_hold(&coordination_hold, &coordination_dir, "staging")
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
            ensure_path_matches_handle(&roots_dir, &roots_hold)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
            ensure_path_matches_handle(&coordination_dir, &coordination_hold)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
            ensure_path_matches_handle(&freeze_dir, &freeze_hold)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
            ensure_path_matches_handle(&staging_dir, &staging_hold)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
            Some((roots_hold, coordination_hold, freeze_hold, staging_hold))
        } else {
            None
        };
        if selected_namespace_holds.is_some() {
            validate_selected_profile_namespace_layout(&root)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
        }
        let registry_contents = namespace_contents_path(&registry_dir, &registry_hold)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RegistryNamespaceInvalid)?;
        let journal_contents = namespace_contents_path(&journal_dir, &journal_hold)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::JournalNamespaceInvalid)?;
        let records_contents = namespace_contents_path(&records_dir, &records_hold)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RecordNamespaceInvalid)?;
        if selected_namespace_holds.is_some() {
            validate_selected_retained_object_names(&journal_contents, &records_contents)?;
        }
        let mut budget = NamespaceBudget::new();

        let genesis_record_read =
            read_regular_file(&registry_contents.join("genesis.cbor"), &mut budget).map_err(
                |error| match error {
                    NamespaceReadError::ResourceLimit => {
                        AuthoritativeRegistryStoreOpenError::NamespaceResourceLimit
                    }
                    NamespaceReadError::Io | NamespaceReadError::Invalid => {
                        AuthoritativeRegistryStoreOpenError::RegistryNamespaceInvalid
                    }
                },
            )?;
        let genesis_record_bytes = genesis_record_read.bytes;
        let genesis_record = GenesisRecord::decode_authoritative(&genesis_record_bytes)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::GenesisRecordDecode)?;

        let (mut records, mut record_witnesses) =
            load_record_namespace(&records_contents, &mut budget)?;
        match records
            .iter()
            .find(|(record_id, _)| *record_id == genesis_record.record_id())
        {
            Some((_, bytes)) if bytes != &genesis_record_bytes => {
                return Err(AuthoritativeRegistryStoreOpenError::GenesisRecordIdentityMismatch)
            }
            Some(_) => {}
            None if open_profile
                == AuthoritativeRegistryStoreOpenProfile::SelectedTerminalAuthorityClosure =>
            {
                return Err(AuthoritativeRegistryStoreOpenError::GenesisRecordIdentityMismatch)
            }
            None => records.push((genesis_record.record_id(), genesis_record_bytes)),
        }
        records.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
        if open_profile == AuthoritativeRegistryStoreOpenProfile::SelectedTerminalAuthorityClosure {
            validate_selected_genesis_observation_records(&records, &genesis_record)?;
        }

        let (journal_slots, mut journal_witnesses) =
            load_journal_slots(&journal_contents, &mut budget)?;
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
        if selected_namespace_holds.is_some() {
            validate_selected_retained_root_locators(&root, &retained_journal)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
        }
        validate_authoritative_event_records(&retained_journal, &records)?;
        if open_profile == AuthoritativeRegistryStoreOpenProfile::LegacyThreeNamespace
            && retained_journal.entries.iter().any(|entry| {
                matches!(entry.event_type_id().value(), 302 | 303)
                    && records
                        .binary_search_by(|(record_id, _)| {
                            record_id.as_bytes().cmp(entry.event_record_id().as_bytes())
                        })
                        .ok()
                        .and_then(|index| {
                            ReviewAdmissionRecord::decode_authoritative(&records[index].1).ok()
                        })
                        .is_some_and(|admission| {
                            admission.terminal_authority_closure_sha256()
                                == Some(&TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256)
                        })
            })
        {
            return Err(AuthoritativeRegistryStoreOpenError::EventSemanticAuthorityUnavailable);
        }

        let mut retained_file_witnesses = Vec::with_capacity(
            1_usize
                .checked_add(record_witnesses.len())
                .and_then(|count| count.checked_add(journal_witnesses.len()))
                .ok_or(AuthoritativeRegistryStoreOpenError::NamespaceResourceLimit)?,
        );
        retained_file_witnesses.push(genesis_record_read.witness);
        retained_file_witnesses.append(&mut record_witnesses);
        retained_file_witnesses.append(&mut journal_witnesses);
        before_generation_guard();
        if !cfg!(any(windows, target_os = "linux", target_os = "macos")) {
            return Err(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationProtectionUnavailable,
            );
        }

        let mut retained_generation_guards = Vec::new();
        retained_generation_guards
            .try_reserve_exact(retained_file_witnesses.len())
            .map_err(|_| AuthoritativeRegistryStoreOpenError::NamespaceResourceLimit)?;
        for witness in retained_file_witnesses {
            retained_generation_guards.push(
                into_retained_file_guard(witness)
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged)?,
            );
        }

        ensure_path_matches_handle(&root, &root_hold)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
        ensure_path_matches_handle(&registry_dir, &registry_hold)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RegistryNamespaceInvalid)?;
        ensure_path_matches_handle(&journal_dir, &journal_hold)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::JournalNamespaceInvalid)?;
        ensure_path_matches_handle(&records_dir, &records_hold)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RecordNamespaceInvalid)?;
        if let Some((roots_hold, coordination_hold, freeze_hold, staging_hold)) =
            &selected_namespace_holds
        {
            ensure_path_matches_handle(&root.join("roots"), roots_hold)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
            ensure_path_matches_handle(&root.join("coordination"), coordination_hold)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
            ensure_path_matches_handle(&root.join("coordination/freeze"), freeze_hold)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
            ensure_path_matches_handle(&root.join("coordination/staging"), staging_hold)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
            validate_selected_profile_namespace_layout(&root)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
            validate_selected_retained_root_locators(&root, &retained_journal)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
            validate_selected_retained_object_names(&journal_contents, &records_contents)?;
        }
        for witness in &mut retained_generation_guards {
            revalidate_retained_file_witness(witness)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged)?;
        }

        let mut namespace_holds = vec![root_hold, registry_hold, journal_hold, records_hold];
        if let Some((roots_hold, coordination_hold, freeze_hold, staging_hold)) =
            selected_namespace_holds
        {
            namespace_holds.extend([roots_hold, coordination_hold, freeze_hold, staging_hold]);
        }

        let store = Self {
            root,
            open_profile,
            retained_journal,
            records,
            namespace_holds,
            retained_file_witnesses: retained_generation_guards,
            live_instance_identity: Arc::new(AuthoritativeRegistryStoreInstanceIdentity {
                outstanding_review_admissions: AtomicUsize::new(0),
            }),
        };
        if store.open_profile
            == AuthoritativeRegistryStoreOpenProfile::SelectedTerminalAuthorityClosure
        {
            store.validate_selected_terminal_admission_replay()?;
        }
        Ok(store)
    }

    /// The canonical root locator from which this store's authority namespaces were opened.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The exact retained Journal reconstructed from the authoritative Journal namespace.
    pub fn retained_journal(&self) -> &RetainedJournal {
        &self.retained_journal
    }

    /// Records a selected event-100 START and creates its Store-owned EMBEDDED payload copy.
    ///
    /// This bounded producer deliberately stops before Receipt/event-101 construction. Its
    /// successful return preserves structural facts only and establishes no positive authority.
    pub fn prepare_selected_embedded_freeze(
        &mut self,
        input: SelectedEmbeddedFreezePreparationInput,
    ) -> Result<PreparedSelectedEmbeddedFreeze, SelectedEmbeddedFreezePreparationError> {
        if self.open_profile
            != AuthoritativeRegistryStoreOpenProfile::SelectedTerminalAuthorityClosure
        {
            return Err(SelectedEmbeddedFreezePreparationError::SelectedProfileRequired);
        }
        let root_hold = self
            .namespace_holds
            .first()
            .ok_or(SelectedEmbeddedFreezePreparationError::RetainedGenerationChanged)?;
        let _publication_lock =
            acquire_selected_authoritative_publication_lock(&self.root, root_hold)
                .map_err(|_| SelectedEmbeddedFreezePreparationError::PublicationLockUnavailable)?;
        self.reload_authoritative_namespaces()
            .map_err(map_selected_embedded_preparation_reload_error)?;

        let policy_bytes = self
            .resolve(input.policy_record_id)
            .ok_or(SelectedEmbeddedFreezePreparationError::PolicyUnavailable)?;
        let policy = MinimalPolicyRecord::decode_authoritative(policy_bytes)
            .map_err(|_| SelectedEmbeddedFreezePreparationError::PolicyInvalid)?;
        if policy.record_id() != input.policy_record_id {
            return Err(SelectedEmbeddedFreezePreparationError::PolicyInvalid);
        }
        if !selected_freeze_policy_is_supported(&policy, self) {
            return Err(SelectedEmbeddedFreezePreparationError::PolicyUnsupported);
        }
        let source = collect_selected_embedded_source(&input.source_root)?;
        let subject_id = derive_selected_regular_file_subject(&source.artifacts)
            .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceInvalid)?;
        let current_head = self.retained_journal.current_head_reference();
        let current_context = self
            .retained_journal
            .resolve_reference(&current_head)
            .map_err(|_| SelectedEmbeddedFreezePreparationError::ReplayMismatch)?;
        let next_index = current_head
            .entry_index()
            .value()
            .checked_add(1)
            .and_then(|value| JournalEntryIndex::try_from(value).ok())
            .ok_or(SelectedEmbeddedFreezePreparationError::EntryIndexExhausted)?;
        let start_record = FreezeAttemptStartRecord::new(FreezeAttemptStartRecordInput {
            freeze_attempt_id: input.freeze_attempt_id,
            intended_root_id: derive_freeze_root(
                current_head.registry_id(),
                input.freeze_attempt_id,
            )
            .intended_root_id(),
            subject_id,
            policy_record_id: input.policy_record_id,
        });
        let start_entry =
            FreezeAttemptStartJournalEntry::new(FreezeAttemptStartJournalEntryInput {
                registry_id: current_head.registry_id(),
                entry_index: next_index,
                previous_entry_hash: current_head.entry_hash(),
                start_record: start_record.clone(),
                storage_capability_class_id: current_context.storage_capability_class_id(),
                environment_observation_id: current_context.environment_observation_id(),
            })
            .map_err(|_| SelectedEmbeddedFreezePreparationError::StartJournalEntry)?;
        let start_entry_bytes = start_entry.authoritative_cbor();
        let expected_start_reference = JournalReference::new(
            current_head.registry_id(),
            next_index,
            JournalEntryHash::try_from(Sha256::digest(&start_entry_bytes).as_slice())
                .expect("SHA-256 has exact Journal Entry hash width"),
            start_entry.event_type_id(),
            start_entry.event_record_id(),
        );

        publish_record_bytes(
            self,
            start_record.record_id(),
            &start_record.authoritative_cbor(),
        )
        .map_err(|_| SelectedEmbeddedFreezePreparationError::StartRecordPublication)?;
        let journal_dir = self.root.join("journal");
        let journal_hold = self
            .acquire_publication_directory_hold(2, "journal")
            .map_err(|_| SelectedEmbeddedFreezePreparationError::RetainedGenerationChanged)?;
        let journal_contents = namespace_contents_path(&journal_dir, &journal_hold)
            .map_err(|_| SelectedEmbeddedFreezePreparationError::RetainedGenerationChanged)?;
        let journal_name = format!("{:020}.cbor", next_index.value());
        match publish_journal_slot(
            &journal_contents,
            &journal_hold,
            Path::new(&journal_name),
            &start_entry_bytes,
        )
        .map_err(|_| SelectedEmbeddedFreezePreparationError::StartJournalPublication)?
        {
            JournalSlotPublication::Published(_) => {}
            JournalSlotPublication::Conflict => {
                return Err(SelectedEmbeddedFreezePreparationError::StartPublicationConflict)
            }
            JournalSlotPublication::VisibleReceiptUncertain => {
                return Err(SelectedEmbeddedFreezePreparationError::StartJournalPublication)
            }
        }
        self.reload_authoritative_namespaces()
            .map_err(map_selected_embedded_preparation_reload_error)?;
        let start_event_reference = self.retained_journal.current_head_reference();
        if start_event_reference != expected_start_reference
            || start_event_reference.event_type_id().value() != 100
        {
            return Err(SelectedEmbeddedFreezePreparationError::ReplayMismatch);
        }

        let (payload_directory, payload_hold) = create_selected_embedded_payload_root(
            self,
            input.freeze_attempt_id,
            &start_event_reference,
        )?;
        publish_selected_embedded_payload(&payload_directory, &payload_hold, &source)?;
        let retained = collect_selected_embedded_source(&payload_directory)?;
        let retained_subject_id = derive_selected_regular_file_subject(&retained.artifacts)
            .map_err(|_| SelectedEmbeddedFreezePreparationError::ReplayMismatch)?;
        if retained.artifacts != source.artifacts
            || retained_subject_id != subject_id
            || collect_selected_embedded_source(&input.source_root)? != source
        {
            return Err(SelectedEmbeddedFreezePreparationError::SourceChanged);
        }
        let manifest = ManifestRecord::new(ManifestRecordInput {
            subject_id: retained_subject_id,
            artifact_count: u64::try_from(retained.artifacts.len())
                .map_err(|_| SelectedEmbeddedFreezePreparationError::ManifestConstruction)?,
            path_identity_profile_id: 1,
            digest_profile_id: 1,
            artifacts: retained.artifacts,
        })
        .map_err(|_| SelectedEmbeddedFreezePreparationError::ManifestConstruction)?;
        publish_record_bytes(self, manifest.record_id(), &manifest.authoritative_cbor())
            .map_err(|_| SelectedEmbeddedFreezePreparationError::ManifestPublication)?;
        self.revalidate_retained_generation()
            .map_err(|_| SelectedEmbeddedFreezePreparationError::RetainedGenerationChanged)?;
        Ok(PreparedSelectedEmbeddedFreeze {
            start_record,
            start_event_reference,
            manifest_record_id: manifest.record_id(),
            payload_directory,
        })
    }

    /// Publishes a selected FREEZE_RECEIPT and event 101 from an opaque Store-produced
    /// EMBEDDED preparation, then requires an exact selected cold replay before returning.
    pub fn commit_prepared_selected_embedded_freeze(
        &mut self,
        prepared: PreparedSelectedEmbeddedFreeze,
    ) -> Result<AuthoritativeFreezeCommittedBinding, SelectedEmbeddedFreezeCommitError> {
        if self.open_profile
            != AuthoritativeRegistryStoreOpenProfile::SelectedTerminalAuthorityClosure
        {
            return Err(SelectedEmbeddedFreezeCommitError::SelectedProfileRequired);
        }
        let root_hold = self
            .namespace_holds
            .first()
            .ok_or(SelectedEmbeddedFreezeCommitError::RetainedGenerationChanged)?;
        let _publication_lock =
            acquire_selected_authoritative_publication_lock(&self.root, root_hold)
                .map_err(|_| SelectedEmbeddedFreezeCommitError::PublicationLockUnavailable)?;
        self.reload_authoritative_namespaces()
            .map_err(map_selected_embedded_commit_reload_error)?;

        let start_reference = prepared.start_event_reference.clone();
        if self.retained_journal.current_head_reference() != start_reference
            || start_reference.event_type_id().value() != 100
            || self
                .resolve(prepared.start_record.record_id())
                .is_none_or(|bytes| bytes != prepared.start_record.authoritative_cbor())
        {
            return Err(SelectedEmbeddedFreezeCommitError::PreparationMismatch);
        }
        let manifest_bytes = self
            .resolve(prepared.manifest_record_id)
            .ok_or(SelectedEmbeddedFreezeCommitError::PreparationMismatch)?;
        let manifest = ManifestRecord::decode_authoritative(manifest_bytes)
            .map_err(|_| SelectedEmbeddedFreezeCommitError::PreparationMismatch)?;
        if manifest.record_id() != prepared.manifest_record_id
            || manifest.input().subject_id != prepared.start_record.input().subject_id
            || validate_selected_embedded_payload(
                &self.root,
                prepared.start_record.input().freeze_attempt_id,
                &manifest,
            )
            .is_err()
        {
            return Err(SelectedEmbeddedFreezeCommitError::PayloadInvalid);
        }
        let policy = self
            .resolve(prepared.start_record.input().policy_record_id)
            .and_then(|bytes| MinimalPolicyRecord::decode_authoritative(bytes).ok())
            .filter(|policy| {
                policy.record_id() == prepared.start_record.input().policy_record_id
                    && selected_freeze_policy_is_supported(policy, self)
            })
            .ok_or(SelectedEmbeddedFreezeCommitError::PolicyInvalid)?;
        let creation_profile = FreezeCreationProfileRecord::new()
            .map_err(|_| SelectedEmbeddedFreezeCommitError::ReceiptConstruction)?;
        publish_record_bytes(
            self,
            creation_profile.record_id(),
            &creation_profile.authoritative_cbor(),
        )
        .map_err(|_| SelectedEmbeddedFreezeCommitError::CreationProfilePublication)?;
        self.revalidate_retained_generation()
            .map_err(|_| SelectedEmbeddedFreezeCommitError::RetainedGenerationChanged)?;

        let head = self.retained_journal.current_head_reference();
        let context = self
            .retained_journal
            .resolve_reference(&head)
            .map_err(|_| SelectedEmbeddedFreezeCommitError::PreparationMismatch)?;
        let entry_index = head
            .entry_index()
            .value()
            .checked_add(1)
            .and_then(|value| JournalEntryIndex::try_from(value).ok())
            .ok_or(SelectedEmbeddedFreezeCommitError::JournalConstruction)?;
        let receipt = FreezeReceiptRecord::new(FreezeReceiptRecordInput {
            freeze_attempt_id: prepared.start_record.input().freeze_attempt_id,
            freeze_id: *prepared.start_record.input().freeze_attempt_id.as_bytes(),
            attempt_start_journal_ref: start_reference.clone(),
            subject_id: prepared.start_record.input().subject_id,
            manifest_id: prepared.manifest_record_id,
            custody_mode_id: 1,
            creation_profile_ref: creation_profile.record_id(),
            path_identity_profile_id: 1,
            filesystem_profile_ref: context.environment_observation_id(),
            policy_record_id: policy.record_id(),
            file_content_flush_state: 1,
            atomic_publish_no_replace_state: 1,
            parent_directory_flush_state: 1,
            platform_strongest_available: false,
            requested_commit_durability_ref: None,
            terminal_authority_closure_sha256: Some(TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256),
            created_by_tool_version: "evidence-registry-selected-freeze-v1".to_owned(),
        })
        .map_err(|_| SelectedEmbeddedFreezeCommitError::ReceiptConstruction)?;
        let entry = FreezeCommittedJournalEntry::new(FreezeCommittedJournalEntryInput {
            registry_id: head.registry_id(),
            entry_index,
            previous_entry_hash: head.entry_hash(),
            receipt: receipt.clone(),
            storage_capability_class_id: context.storage_capability_class_id(),
            environment_observation_id: context.environment_observation_id(),
        })
        .map_err(|_| SelectedEmbeddedFreezeCommitError::JournalConstruction)?;
        let entry_bytes = entry.authoritative_cbor();
        let reference = JournalReference::new(
            head.registry_id(),
            entry_index,
            JournalEntryHash::try_from(Sha256::digest(&entry_bytes).as_slice())
                .expect("SHA-256 has exact Journal Entry hash width"),
            entry.event_type_id(),
            entry.event_record_id(),
        );
        publish_record_bytes(self, receipt.record_id(), &receipt.authoritative_cbor())
            .map_err(|_| SelectedEmbeddedFreezeCommitError::ReceiptPublication)?;
        self.revalidate_retained_generation()
            .map_err(|_| SelectedEmbeddedFreezeCommitError::RetainedGenerationChanged)?;
        let journal_dir = self.root.join("journal");
        let journal_hold = self
            .acquire_publication_directory_hold(2, "journal")
            .map_err(|_| SelectedEmbeddedFreezeCommitError::RetainedGenerationChanged)?;
        let journal_contents = namespace_contents_path(&journal_dir, &journal_hold)
            .map_err(|_| SelectedEmbeddedFreezeCommitError::RetainedGenerationChanged)?;
        match publish_journal_slot(
            &journal_contents,
            &journal_hold,
            Path::new(&format!("{:020}.cbor", entry_index.value())),
            &entry_bytes,
        )
        .map_err(|_| SelectedEmbeddedFreezeCommitError::JournalPublication)?
        {
            JournalSlotPublication::Published(_) => {}
            JournalSlotPublication::Conflict => {
                return Err(SelectedEmbeddedFreezeCommitError::JournalConflict)
            }
            JournalSlotPublication::VisibleReceiptUncertain => {
                return Err(SelectedEmbeddedFreezeCommitError::JournalPublication)
            }
        }
        self.reload_authoritative_namespaces()
            .map_err(map_selected_embedded_commit_reload_error)?;
        if self.retained_journal.current_head_reference() != reference {
            return Err(SelectedEmbeddedFreezeCommitError::ReplayMismatch);
        }
        self.validate_freeze_committed_authority(reference)
            .map_err(SelectedEmbeddedFreezeCommitError::PositiveReplay)
    }

    /// Validates selected Receipt/event-101 replay and current embedded payload bytes before
    /// returning a positive selected Freeze witness. Unselected Receipt forms remain unavailable.
    pub fn validate_freeze_committed_authority(
        &self,
        committed_event_reference: JournalReference,
    ) -> Result<AuthoritativeFreezeCommittedBinding, AuthoritativeFreezeCommittedBindingError> {
        self.revalidate_retained_generation()
            .map_err(|()| AuthoritativeFreezeCommittedBindingError::RetainedGenerationChanged)?;
        self.validate_freeze_committed_authority_from_retained_snapshot(committed_event_reference)
    }

    /// Validates a selected Freeze against one already generation-validated retained snapshot.
    ///
    /// This is intentionally private: callers must use the public validator above. Selected
    /// terminal replay uses it only after `open_selected_profile` has captured and retained one
    /// exact namespace generation, so repeated terminal records cannot turn a bounded open into
    /// repeated full namespace revalidation.
    fn validate_freeze_committed_authority_from_retained_snapshot(
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
        let committed = self
            .retained_journal
            .resolve_reference(&committed_event_reference)
            .map_err(|error| {
                AuthoritativeFreezeCommittedBindingError::Structural(
                    ResolvedFreezeCommittedBindingError::RetainedReference(error),
                )
            })?;
        let receipt_record_id = record_id_from_event_reference(&committed_event_reference);
        let receipt = self
            .resolve(receipt_record_id)
            .and_then(|bytes| FreezeReceiptRecord::decode_authoritative(bytes).ok())
            .filter(|receipt| receipt.record_id() == receipt_record_id)
            .ok_or(AuthoritativeFreezeCommittedBindingError::SelectedPrerequisiteInvalid)?;
        if receipt.terminal_authority_closure_sha256()
            != Some(&TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256)
        {
            return Err(AuthoritativeFreezeCommittedBindingError::SemanticAuthorityUnavailable);
        }
        validate_selected_freeze_committed_prerequisites(
            &self.root,
            &self.retained_journal,
            self,
            &committed,
            &receipt,
        )
        .map_err(|error| match error {
            SelectedEmbeddedFreezePreparationError::SourceResourceLimit => {
                AuthoritativeFreezeCommittedBindingError::ResourceLimit
            }
            _ => AuthoritativeFreezeCommittedBindingError::SelectedPrerequisiteInvalid,
        })?;
        let start_record_id =
            record_id_from_event_reference(&receipt.input().attempt_start_journal_ref);
        Ok(AuthoritativeFreezeCommittedBinding {
            registry_id: committed_event_reference.registry_id(),
            committed_event_reference,
            start_event_reference: receipt.input().attempt_start_journal_ref.clone(),
            receipt_record_id,
            manifest_record_id: receipt.input().manifest_id,
            start_record_id,
            freeze_attempt_id: receipt.input().freeze_attempt_id,
            freeze_id: receipt.input().freeze_id,
            subject_id: receipt.input().subject_id,
            policy_record_id: receipt.input().policy_record_id,
        })
    }

    /// Revalidates the live selected payload for an already record-validated Freeze binding.
    /// The binding cache deliberately never stands in for this check: selected replay is only
    /// positive while the payload currently matches the retained manifest.
    fn validate_selected_freeze_payload_from_retained_snapshot(
        &self,
        freeze_authority: &AuthoritativeFreezeCommittedBinding,
    ) -> Result<(), AuthoritativeFreezeCommittedBindingError> {
        let manifest = self
            .resolve(freeze_authority.manifest_record_id())
            .and_then(|bytes| ManifestRecord::decode_authoritative(bytes).ok())
            .filter(|manifest| manifest.record_id() == freeze_authority.manifest_record_id())
            .ok_or(AuthoritativeFreezeCommittedBindingError::SelectedPrerequisiteInvalid)?;
        validate_selected_embedded_payload(
            &self.root,
            freeze_authority.freeze_attempt_id(),
            &manifest,
        )
        .map_err(|error| match error {
            SelectedEmbeddedFreezePreparationError::SourceResourceLimit => {
                AuthoritativeFreezeCommittedBindingError::ResourceLimit
            }
            _ => AuthoritativeFreezeCommittedBindingError::SelectedPrerequisiteInvalid,
        })
    }

    /// Records one selected REVIEW_REQUEST from Store-resolved Freeze and Policy state.
    ///
    /// The caller cannot supply a Request body, Policy event reference, selector
    /// fields, Anchor, operation-start reference, or Journal slot. Successful
    /// recording establishes only the bounded selected Request transport and
    /// retained binding; it does not ingest a Result or complete Admission.
    pub fn record_selected_review_request(
        &mut self,
        input: SelectedReviewRequestInput,
    ) -> Result<RecordedSelectedReviewRequest, SelectedReviewRequestError> {
        if self.open_profile
            != AuthoritativeRegistryStoreOpenProfile::SelectedTerminalAuthorityClosure
        {
            return Err(SelectedReviewRequestError::SelectedProfileRequired);
        }
        let root_hold = self
            .namespace_holds
            .first()
            .ok_or(SelectedReviewRequestError::RetainedGenerationChanged)?;
        let _publication_lock =
            acquire_selected_authoritative_publication_lock(&self.root, root_hold)
                .map_err(|_| SelectedReviewRequestError::PublicationLockUnavailable)?;
        self.reload_authoritative_namespaces()
            .map_err(map_selected_review_request_reload_error)?;

        let freeze_authority = self
            .validate_freeze_committed_authority(
                input.freeze_authority.committed_event_reference().clone(),
            )
            .map_err(SelectedReviewRequestError::FreezeAuthority)?;
        if freeze_authority != input.freeze_authority {
            return Err(SelectedReviewRequestError::FreezeWitnessMismatch);
        }
        let policy_reference = selected_policy_authority_reference(
            &self.retained_journal,
            input.review_policy_record_id,
        )?;
        let policy_bytes = self
            .resolve(input.review_policy_record_id)
            .ok_or(SelectedReviewRequestError::PolicyUnavailable)?;
        let policy = ReviewAdmissionPolicyRecord::decode_authoritative(policy_bytes)
            .map_err(|_| SelectedReviewRequestError::PolicyInvalid)?;
        if policy.record_id() != input.review_policy_record_id
            || policy.operation_start_journal_ref() != freeze_authority.start_event_reference()
            || !selected_review_policy_is_supported(&policy, self)
        {
            return Err(SelectedReviewRequestError::PolicyUnsupported);
        }
        let requirement = selected_review_policy_requirement(&policy, input.review_role_id)?;
        for record_id in [
            requirement.required_checks_ref(),
            requirement.review_scope_ref(),
            requirement.review_method_ref(),
        ] {
            let bytes = self
                .resolve(record_id)
                .ok_or(SelectedReviewRequestError::SelectorUnavailable)?;
            StrictRecordFrame::decode_authoritative(bytes)
                .map_err(|_| SelectedReviewRequestError::SelectorUnavailable)?;
        }
        let head = self.retained_journal.current_head_reference();
        let context = self
            .retained_journal
            .resolve_reference(&head)
            .map_err(|_| SelectedReviewRequestError::ReplayMismatch)?;
        let anchor =
            JournalAnchor::new(head.registry_id(), head.entry_index(), head.entry_hash(), 1)
                .map_err(|_| SelectedReviewRequestError::AnchorConstruction)?;
        let entry_index = head
            .entry_index()
            .value()
            .checked_add(1)
            .and_then(|value| JournalEntryIndex::try_from(value).ok())
            .ok_or(SelectedReviewRequestError::JournalConstruction)?;
        let request = ReviewRequestRecord::new_selected(ReviewRequestRecordInput {
            freeze_authority_ref: freeze_authority.committed_event_reference().clone(),
            manifest_id: freeze_authority.manifest_record_id(),
            review_role_id: input.review_role_id,
            required_checks_ref: requirement.required_checks_ref(),
            policy_authority_ref: policy_reference.clone(),
            review_scope_ref: requirement.review_scope_ref(),
            review_method_ref: requirement.review_method_ref(),
            review_package_anchor_id: anchor.anchor_id(),
            operation_start_journal_ref: freeze_authority.start_event_reference().clone(),
        })
        .map_err(|_| SelectedReviewRequestError::RequestConstruction)?;
        let entry = ReviewRequestJournalEntry::new(ReviewRequestJournalEntryInput {
            registry_id: head.registry_id(),
            entry_index,
            previous_entry_hash: head.entry_hash(),
            request: request.clone(),
            storage_capability_class_id: context.storage_capability_class_id(),
            environment_observation_id: context.environment_observation_id(),
        })
        .map_err(|_| SelectedReviewRequestError::JournalConstruction)?;
        let entry_bytes = entry.authoritative_cbor();
        let reference = JournalReference::new(
            head.registry_id(),
            entry_index,
            JournalEntryHash::try_from(Sha256::digest(&entry_bytes).as_slice())
                .expect("SHA-256 has the exact JournalEntryHash width"),
            entry.event_type_id(),
            entry.event_record_id(),
        );
        publish_record_bytes(self, request.record_id(), &request.authoritative_cbor())
            .map_err(|_| SelectedReviewRequestError::RequestPublication)?;
        self.revalidate_retained_generation()
            .map_err(|_| SelectedReviewRequestError::RetainedGenerationChanged)?;
        let journal_dir = self.root.join("journal");
        let journal_hold = self
            .acquire_publication_directory_hold(2, "journal")
            .map_err(|_| SelectedReviewRequestError::RetainedGenerationChanged)?;
        let journal_contents = namespace_contents_path(&journal_dir, &journal_hold)
            .map_err(|_| SelectedReviewRequestError::RetainedGenerationChanged)?;
        match publish_journal_slot(
            &journal_contents,
            &journal_hold,
            Path::new(&format!("{:020}.cbor", entry_index.value())),
            &entry_bytes,
        )
        .map_err(|_| SelectedReviewRequestError::JournalPublication)?
        {
            JournalSlotPublication::Published(_) => {}
            JournalSlotPublication::Conflict => {
                return Err(SelectedReviewRequestError::JournalConflict)
            }
            JournalSlotPublication::VisibleReceiptUncertain => {
                return Err(SelectedReviewRequestError::JournalPublication)
            }
        }
        self.reload_authoritative_namespaces()
            .map_err(map_selected_review_request_reload_error)?;
        if self.retained_journal.current_head_reference() != reference {
            return Err(SelectedReviewRequestError::ReplayMismatch);
        }
        self.validate_selected_review_request(reference)
    }

    /// Records one selected Result from a retained selected Request. Callers
    /// select only the retained Request and bounded semantic submission; all
    /// authority and transport bindings are derived by the Store.
    pub fn record_selected_review_result(
        &mut self,
        input: SelectedReviewResultInput,
    ) -> Result<RecordedSelectedReviewResult, SelectedReviewResultError> {
        if self.open_profile
            != AuthoritativeRegistryStoreOpenProfile::SelectedTerminalAuthorityClosure
        {
            return Err(SelectedReviewResultError::SelectedProfileRequired);
        }
        let root_hold = self
            .namespace_holds
            .first()
            .ok_or(SelectedReviewResultError::RetainedGenerationChanged)?;
        let _publication_lock =
            acquire_selected_authoritative_publication_lock(&self.root, root_hold)
                .map_err(|_| SelectedReviewResultError::PublicationLockUnavailable)?;
        self.reload_authoritative_namespaces()
            .map_err(map_selected_review_result_reload_error)?;
        let request = self
            .validate_selected_review_request(input.request_event_reference.clone())
            .map_err(SelectedReviewResultError::Request)?;
        for finding in &input.findings {
            let bytes = self
                .resolve(*finding)
                .ok_or(SelectedReviewResultError::FindingUnavailable)?;
            StrictRecordFrame::decode_authoritative(bytes)
                .map_err(|_| SelectedReviewResultError::FindingUnavailable)?;
        }
        let head = self.retained_journal.current_head_reference();
        let context = self
            .retained_journal
            .resolve_reference(&head)
            .map_err(|_| SelectedReviewResultError::ReplayMismatch)?;
        let entry_index = head
            .entry_index()
            .value()
            .checked_add(1)
            .and_then(|value| JournalEntryIndex::try_from(value).ok())
            .ok_or(SelectedReviewResultError::JournalConstruction)?;
        let result = ReviewResultRecord::new_selected(ReviewResultRecordInput {
            review_request_authority_ref: input.request_event_reference,
            freeze_authority_ref: request.request().freeze_authority_ref().clone(),
            manifest_id: request.request().manifest_id(),
            review_role_id: request.request().review_role_id(),
            review_scope_ref: request.request().review_scope_ref(),
            review_method_ref: request.request().review_method_ref(),
            method_status: input.method_status,
            finding_state: input.finding_state,
            reason_codes: input.reason_codes,
            findings: input.findings,
            reviewer_metadata: input.reviewer_metadata,
            review_package_anchor_id: request.request().review_package_anchor_id(),
            operation_start_journal_ref: request.request().operation_start_journal_ref().clone(),
        })
        .map_err(|_| SelectedReviewResultError::ResultConstruction)?;
        let entry = ReviewResultJournalEntry::new(ReviewResultJournalEntryInput {
            registry_id: head.registry_id(),
            entry_index,
            previous_entry_hash: head.entry_hash(),
            result: result.clone(),
            storage_capability_class_id: context.storage_capability_class_id(),
            environment_observation_id: context.environment_observation_id(),
        })
        .map_err(|_| SelectedReviewResultError::JournalConstruction)?;
        let entry_bytes = entry.authoritative_cbor();
        let reference = JournalReference::new(
            head.registry_id(),
            entry_index,
            JournalEntryHash::try_from(Sha256::digest(&entry_bytes).as_slice())
                .expect("SHA-256 has exact JournalEntryHash width"),
            entry.event_type_id(),
            entry.event_record_id(),
        );
        publish_record_bytes(self, result.record_id(), &result.authoritative_cbor())
            .map_err(|_| SelectedReviewResultError::ResultPublication)?;
        self.revalidate_retained_generation()
            .map_err(|_| SelectedReviewResultError::RetainedGenerationChanged)?;
        let journal_dir = self.root.join("journal");
        let journal_hold = self
            .acquire_publication_directory_hold(2, "journal")
            .map_err(|_| SelectedReviewResultError::RetainedGenerationChanged)?;
        let journal_contents = namespace_contents_path(&journal_dir, &journal_hold)
            .map_err(|_| SelectedReviewResultError::RetainedGenerationChanged)?;
        match publish_journal_slot(
            &journal_contents,
            &journal_hold,
            Path::new(&format!("{:020}.cbor", entry_index.value())),
            &entry_bytes,
        )
        .map_err(|_| SelectedReviewResultError::JournalPublication)?
        {
            JournalSlotPublication::Published(_) => {}
            JournalSlotPublication::Conflict => {
                return Err(SelectedReviewResultError::JournalConflict)
            }
            JournalSlotPublication::VisibleReceiptUncertain => {
                return Err(SelectedReviewResultError::JournalPublication)
            }
        }
        self.reload_authoritative_namespaces()
            .map_err(map_selected_review_result_reload_error)?;
        if self.retained_journal.current_head_reference() != reference {
            return Err(SelectedReviewResultError::ReplayMismatch);
        }
        self.validate_selected_review_result(reference)
    }

    /// Revalidates a retained selected Request and its Store-owned Freeze/Policy inputs.
    pub fn validate_selected_review_request(
        &self,
        request_event_reference: JournalReference,
    ) -> Result<RecordedSelectedReviewRequest, SelectedReviewRequestError> {
        self.revalidate_retained_generation()
            .map_err(|_| SelectedReviewRequestError::RetainedGenerationChanged)?;
        let mut freeze_cache = Vec::new();
        let mut replay_work_bytes = 0;
        self.validate_selected_review_request_from_retained_snapshot(
            request_event_reference,
            &mut freeze_cache,
            &mut replay_work_bytes,
            u64::MAX,
        )
    }

    fn validate_selected_review_request_from_retained_snapshot(
        &self,
        request_event_reference: JournalReference,
        freeze_cache: &mut Vec<(JournalReference, AuthoritativeFreezeCommittedBinding)>,
        replay_work_bytes: &mut u64,
        max_replay_work_bytes: u64,
    ) -> Result<RecordedSelectedReviewRequest, SelectedReviewRequestError> {
        let request_record_id = record_id_from_event_reference(&request_event_reference);
        let request = self
            .resolve(request_record_id)
            .and_then(|bytes| ReviewRequestRecord::decode_authoritative(bytes).ok())
            .filter(|request| request.record_id() == request_record_id)
            .ok_or(SelectedReviewRequestError::ReplayMismatch)?;
        if request.terminal_authority_closure_sha256()
            != Some(&TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256)
        {
            return Err(SelectedReviewRequestError::PolicyUnsupported);
        }
        validate_review_request_recorded_binding(
            &self.retained_journal,
            &request_event_reference,
            &request.authoritative_cbor(),
        )
        .map_err(|_| SelectedReviewRequestError::ReplayMismatch)?;
        self.reserve_selected_terminal_replay_payload_budget(
            request.freeze_authority_ref(),
            replay_work_bytes,
            max_replay_work_bytes,
        )
        .map_err(|error| match error {
            SelectedTerminalReplayPayloadBudgetError::FreezePrerequisiteInvalid => {
                SelectedReviewRequestError::FreezeAuthority(
                    AuthoritativeFreezeCommittedBindingError::SelectedPrerequisiteInvalid,
                )
            }
            SelectedTerminalReplayPayloadBudgetError::ResourceLimit => {
                SelectedReviewRequestError::ReplayResourceLimit
            }
        })?;
        let freeze_authority = if let Some((_, freeze_authority)) = freeze_cache
            .iter()
            .find(|(reference, _)| reference == request.freeze_authority_ref())
        {
            self.validate_selected_freeze_payload_from_retained_snapshot(freeze_authority)
                .map_err(|error| match error {
                    AuthoritativeFreezeCommittedBindingError::ResourceLimit => {
                        SelectedReviewRequestError::ReplayResourceLimit
                    }
                    error => SelectedReviewRequestError::FreezeAuthority(error),
                })?;
            freeze_authority.clone()
        } else {
            let freeze_authority = self
                .validate_freeze_committed_authority_from_retained_snapshot(
                    request.freeze_authority_ref().clone(),
                )
                .map_err(SelectedReviewRequestError::FreezeAuthority)?;
            freeze_cache
                .try_reserve(1)
                .map_err(|_| SelectedReviewRequestError::ReplayResourceLimit)?;
            freeze_cache.push((
                request.freeze_authority_ref().clone(),
                freeze_authority.clone(),
            ));
            freeze_authority
        };
        if request.manifest_id() != freeze_authority.manifest_record_id()
            || request.operation_start_journal_ref() != freeze_authority.start_event_reference()
        {
            return Err(SelectedReviewRequestError::ReplayMismatch);
        }
        let policy_record_id = record_id_from_event_reference(request.policy_authority_ref());
        let policy = self
            .resolve(policy_record_id)
            .and_then(|bytes| ReviewAdmissionPolicyRecord::decode_authoritative(bytes).ok())
            .filter(|policy| policy.record_id() == policy_record_id)
            .ok_or(SelectedReviewRequestError::PolicyInvalid)?;
        if policy.operation_start_journal_ref() != freeze_authority.start_event_reference()
            || !selected_review_policy_is_supported(&policy, self)
        {
            return Err(SelectedReviewRequestError::PolicyUnsupported);
        }
        let requirement = selected_review_policy_requirement(&policy, request.review_role_id())?;
        if request.required_checks_ref() != requirement.required_checks_ref()
            || request.review_scope_ref() != requirement.review_scope_ref()
            || request.review_method_ref() != requirement.review_method_ref()
        {
            return Err(SelectedReviewRequestError::ReplayMismatch);
        }
        Ok(RecordedSelectedReviewRequest {
            policy_authority_ref: request.policy_authority_ref().clone(),
            request,
            request_event_reference,
            freeze_authority,
        })
    }

    /// Revalidates one retained selected Result and its selected Request/Freeze
    /// transport bindings. It establishes neither §82 nor Admission authority.
    pub fn validate_selected_review_result(
        &self,
        result_event_reference: JournalReference,
    ) -> Result<RecordedSelectedReviewResult, SelectedReviewResultError> {
        self.revalidate_retained_generation()
            .map_err(|_| SelectedReviewResultError::RetainedGenerationChanged)?;
        let mut freeze_cache = Vec::new();
        let mut replay_work_bytes = 0;
        self.validate_selected_review_result_from_retained_snapshot(
            result_event_reference,
            &mut freeze_cache,
            &mut replay_work_bytes,
            u64::MAX,
        )
    }

    fn validate_selected_review_result_from_retained_snapshot(
        &self,
        result_event_reference: JournalReference,
        freeze_cache: &mut Vec<(JournalReference, AuthoritativeFreezeCommittedBinding)>,
        replay_work_bytes: &mut u64,
        max_replay_work_bytes: u64,
    ) -> Result<RecordedSelectedReviewResult, SelectedReviewResultError> {
        let result_record_id = record_id_from_event_reference(&result_event_reference);
        let result = self
            .resolve(result_record_id)
            .and_then(|bytes| ReviewResultRecord::decode_authoritative(bytes).ok())
            .filter(|result| {
                result.record_id() == result_record_id
                    && result.review_package_anchor_binding_version() == Some(1)
            })
            .ok_or(SelectedReviewResultError::ReplayMismatch)?;
        validate_review_result_recorded_binding(
            &self.retained_journal,
            &result_event_reference,
            &result.authoritative_cbor(),
        )
        .map_err(|_| SelectedReviewResultError::ReplayMismatch)?;
        let request = self
            .validate_selected_review_request_from_retained_snapshot(
                result.review_request_authority_ref().clone(),
                freeze_cache,
                replay_work_bytes,
                max_replay_work_bytes,
            )
            .map_err(SelectedReviewResultError::Request)?;
        if result.freeze_authority_ref() != request.request().freeze_authority_ref()
            || result.manifest_id() != request.request().manifest_id()
            || result.review_role_id() != request.request().review_role_id()
            || result.review_scope_ref() != request.request().review_scope_ref()
            || result.review_method_ref() != request.request().review_method_ref()
            || result.review_package_anchor_id() != request.request().review_package_anchor_id()
            || result.operation_start_journal_ref()
                != request.request().operation_start_journal_ref()
        {
            return Err(SelectedReviewResultError::ReplayMismatch);
        }
        Ok(RecordedSelectedReviewResult {
            result,
            result_event_reference,
            request,
        })
    }

    /// Derives the exact selected §82 witness from one retained Result event.
    ///
    /// The caller supplies no Request/Result bytes, Freeze/Policy/Anchor references, evaluator
    /// outcome, Admission body, or publication target. This method never evaluates §46, derives
    /// §83, constructs an Admission Record, or appends events 302/303.
    pub fn derive_selected_review_admission_section_82(
        &mut self,
        result_event_reference: JournalReference,
    ) -> Result<AuthoritativeReviewAdmissionSection82, SelectedReviewAdmissionSection82Error> {
        if self.open_profile
            != AuthoritativeRegistryStoreOpenProfile::SelectedTerminalAuthorityClosure
        {
            return Err(SelectedReviewAdmissionSection82Error::SelectedProfileRequired);
        }
        let result = self
            .validate_selected_review_result(result_event_reference)
            .map_err(SelectedReviewAdmissionSection82Error::Result)?;
        let request_bytes = result.request().request().authoritative_cbor();
        let result_bytes = result.result().authoritative_cbor();
        let accepted = self
            .accept_authoritative_review_admission(
                result.request().request_event_reference().clone(),
                &request_bytes,
                result.result_event_reference().clone(),
                &result_bytes,
            )
            .map_err(SelectedReviewAdmissionSection82Error::Acceptance)?;
        self.complete_authoritative_review_admission_section_82(accepted)
            .map_err(SelectedReviewAdmissionSection82Error::Section82)
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

    pub fn complete_authoritative_review_admission(
        &mut self,
        accepted: AcceptedAuthoritativeReviewAdmission,
    ) -> Result<AuthoritativeReviewAdmissionRuntimeOutcome, AuthoritativeReviewAdmissionRuntimeError>
    {
        let section_82 = match self.complete_authoritative_review_admission_section_82(accepted) {
            Ok(section_82) => section_82,
            Err(error) => {
                return Ok(AuthoritativeReviewAdmissionRuntimeOutcome::PreTerminal(
                    error,
                ));
            }
        };
        self.complete_review_admission_after_section_82(section_82, false, false)
    }

    /// Completes the selected profile-1 exact-Scope terminal route from one
    /// retained selected REVIEW_RESULT event. The Store holds the publication
    /// lock while it reloads, revalidates §82, evaluates §46, derives §83, and
    /// constructs and appends the selected Admission event.
    pub fn complete_selected_review_admission(
        &mut self,
        result_event_reference: JournalReference,
    ) -> Result<AuthoritativeReviewAdmissionRuntimeOutcome, SelectedReviewAdmissionCompletionError>
    {
        if self.open_profile
            != AuthoritativeRegistryStoreOpenProfile::SelectedTerminalAuthorityClosure
        {
            return Err(SelectedReviewAdmissionCompletionError::SelectedProfileRequired);
        }
        let root_hold = self
            .namespace_holds
            .first()
            .ok_or(SelectedReviewAdmissionCompletionError::RetainedGenerationChanged)?;
        let _publication_lock =
            acquire_selected_authoritative_publication_lock(&self.root, root_hold)
                .map_err(|()| SelectedReviewAdmissionCompletionError::PublicationLockUnavailable)?;
        self.reload_authoritative_namespaces()
            .map_err(SelectedReviewAdmissionCompletionError::Reload)?;
        let section_82 = self
            .derive_selected_review_admission_section_82(result_event_reference)
            .map_err(SelectedReviewAdmissionCompletionError::Section82)?;
        self.complete_review_admission_after_section_82(section_82, true, true)
            .map_err(SelectedReviewAdmissionCompletionError::Runtime)
    }

    /// Executes authoritative §46, §83, terminal construction, and durable publication after
    /// the caller has established an exact §82 witness. `selected_profile` is only supplied by
    /// the locked selected entry point above; the generic path cannot select profile semantics.
    ///
    /// Terminal publication requires cross-process publisher serialization and retained-object
    /// revalidation through Record and Journal publication. Windows uses deny-share guards; Linux
    /// and macOS use cooperative `flock` serialization plus exact identity and byte revalidation.
    /// Unix adapters do not constrain a same-principal writer that ignores the protocol. Linux
    /// explicitly unlocks on orderly owner drop; macOS retains the bare File/O_CLOEXEC lifecycle
    /// and does not promise release while a fork-without-exec child retains its descriptor.
    fn complete_review_admission_after_section_82(
        &mut self,
        section_82: AuthoritativeReviewAdmissionSection82,
        selected_profile: bool,
        publication_lock_held: bool,
    ) -> Result<AuthoritativeReviewAdmissionRuntimeOutcome, AuthoritativeReviewAdmissionRuntimeError>
    {
        if !selected_profile && frozen_generic_policy_scope_applicability_unavailable() {
            return Ok(AuthoritativeReviewAdmissionRuntimeOutcome::PreTerminal(
                AuthoritativeReviewAdmissionSection82Error::PolicyScopeApplicabilityUnavailable,
            ));
        }
        let policy_completion = if selected_profile {
            evaluate_selected_review_admission_policy_profile(&section_82)
        } else {
            evaluate_unfrozen_review_admission_policy_profile(&section_82)
        };
        let policy_route =
            ReviewAdmissionPolicy46RouteOutcome::Completed(policy_completion.clone());
        let disposition = derive_review_admission_section_83_disposition(&policy_route);
        let disposition_id = match disposition {
            ReviewAdmissionSection83Disposition::ReviewAdmissionAccepted => 1,
            ReviewAdmissionSection83Disposition::ReviewAdmissionRejected => 2,
            ReviewAdmissionSection83Disposition::PreTerminal => {
                unreachable!("a completed §46 result always selects a terminal §83 disposition")
            }
        };
        let mut reason_codes = section_82.result().reason_codes().to_vec();
        reason_codes.push(
            match policy_completion.result() {
                ReviewAdmissionCompletedPolicyResult::Satisfied => "POLICY_SATISFIED",
                ReviewAdmissionCompletedPolicyResult::GateUnsatisfied => "POLICY_GATE_UNSATISFIED",
                ReviewAdmissionCompletedPolicyResult::GateIndeterminate => {
                    "POLICY_GATE_INDETERMINATE"
                }
            }
            .to_owned(),
        );
        reason_codes.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        reason_codes.dedup();

        let admission_input = ReviewAdmissionRecordInput {
            disposition_id,
            review_request_ref: section_82.request_event_reference().clone(),
            review_result_ref: section_82.result_event_reference().clone(),
            policy_authority_ref: section_82.policy_authority_ref().clone(),
            reason_codes,
            operation_start_journal_ref: section_82.operation_start_journal_ref().clone(),
        };
        let admission_record = if selected_profile {
            ReviewAdmissionRecord::new_selected(admission_input)
        } else {
            ReviewAdmissionRecord::new(admission_input)
        }
        .map_err(|_| {
            AuthoritativeReviewAdmissionRuntimeError::Publication(
                AuthoritativeReviewAdmissionPublicationError::AdmissionRecord,
            )
        })?;
        let admission_record_bytes = admission_record.authoritative_cbor();
        let root_hold = self.namespace_holds.first().ok_or(
            AuthoritativeReviewAdmissionRuntimeError::Publication(
                AuthoritativeReviewAdmissionPublicationError::RetainedGenerationChanged,
            ),
        )?;
        let _publication_lock = if publication_lock_held {
            None
        } else {
            Some(
                acquire_authoritative_publication_lock(&self.root, root_hold).map_err(|()| {
                    AuthoritativeReviewAdmissionRuntimeError::Publication(
                        AuthoritativeReviewAdmissionPublicationError::PublicationLockUnavailable,
                    )
                })?,
            )
        };

        let publication_attempts = if selected_profile {
            1
        } else {
            MAX_AUTHORITATIVE_PUBLICATION_RETRIES
        };
        for _ in 0..publication_attempts {
            if selected_profile {
                self.revalidate_retained_generation().map_err(|_| {
                    AuthoritativeReviewAdmissionRuntimeError::Publication(
                        AuthoritativeReviewAdmissionPublicationError::RetainedGenerationChanged,
                    )
                })?;
            } else {
                self.reload_authoritative_namespaces()
                    .map_err(map_reload_to_runtime_error)?;
            }
            self.retained_journal
                .resolve_reference(section_82.operation_start_journal_ref())
                .map_err(|error| {
                    AuthoritativeReviewAdmissionRuntimeError::Publication(
                        AuthoritativeReviewAdmissionPublicationError::OperationStartReference(
                            error,
                        ),
                    )
                })?;
            let current_head = self.retained_journal.current_head_reference();
            let current_context = self
                .retained_journal
                .resolve_reference(&current_head)
                .expect("the current retained head resolves against its own exact bytes");
            let next_index = current_head
                .entry_index()
                .value()
                .checked_add(1)
                .and_then(|value| JournalEntryIndex::try_from(value).ok())
                .ok_or(AuthoritativeReviewAdmissionRuntimeError::Publication(
                    AuthoritativeReviewAdmissionPublicationError::EntryIndexExhausted,
                ))?;
            let journal_entry =
                ReviewAdmissionJournalEntry::new(ReviewAdmissionJournalEntryInput {
                    registry_id: current_head.registry_id(),
                    entry_index: next_index,
                    previous_entry_hash: current_head.entry_hash(),
                    admission: admission_record.clone(),
                    storage_capability_class_id: current_context.storage_capability_class_id(),
                    environment_observation_id: current_context.environment_observation_id(),
                })
                .map_err(|_| {
                    AuthoritativeReviewAdmissionRuntimeError::Publication(
                        AuthoritativeReviewAdmissionPublicationError::JournalEntry,
                    )
                })?;
            let journal_entry_bytes = journal_entry.authoritative_cbor();
            let mut preflight_store =
                Self::open_for_review_admission_runtime(&self.root, selected_profile).map_err(
                    |error| {
                        AuthoritativeReviewAdmissionRuntimeError::Publication(
                            AuthoritativeReviewAdmissionPublicationError::Store(error),
                        )
                    },
                )?;
            if preflight_store.retained_journal.current_head_reference() != current_head {
                if selected_profile {
                    return Err(AuthoritativeReviewAdmissionRuntimeError::Publication(
                        AuthoritativeReviewAdmissionPublicationError::RetainedGenerationChanged,
                    ));
                }
                continue;
            }
            preflight_store
                .retained_journal
                .append_strict_entry(&journal_entry_bytes)
                .map_err(|error| {
                    AuthoritativeReviewAdmissionRuntimeError::Publication(
                        AuthoritativeReviewAdmissionPublicationError::LifecyclePreflight(error),
                    )
                })?;
            match preflight_store.records.binary_search_by(|(stored_id, _)| {
                stored_id
                    .as_bytes()
                    .cmp(admission_record.record_id().as_bytes())
            }) {
                Ok(index) if preflight_store.records[index].1 == admission_record_bytes => {}
                Ok(_) => {
                    return Err(AuthoritativeReviewAdmissionRuntimeError::Publication(
                        AuthoritativeReviewAdmissionPublicationError::SemanticPreflight,
                    ));
                }
                Err(index) => preflight_store.records.insert(
                    index,
                    (admission_record.record_id(), admission_record_bytes.clone()),
                ),
            }
            validate_authoritative_event_records(
                &preflight_store.retained_journal,
                &preflight_store.records,
            )
            .map_err(|_| {
                AuthoritativeReviewAdmissionRuntimeError::Publication(
                    AuthoritativeReviewAdmissionPublicationError::SemanticPreflight,
                )
            })?;
            if selected_profile {
                preflight_store
                    .validate_selected_terminal_admission_replay()
                    .map_err(|_| {
                        AuthoritativeReviewAdmissionRuntimeError::Publication(
                            AuthoritativeReviewAdmissionPublicationError::SemanticPreflight,
                        )
                    })?;
            }
            let mut publication_store =
                Self::open_for_review_admission_runtime(&self.root, selected_profile).map_err(
                    |error| {
                        AuthoritativeReviewAdmissionRuntimeError::Publication(
                            AuthoritativeReviewAdmissionPublicationError::Store(error),
                        )
                    },
                )?;
            if publication_store.retained_journal.current_head_reference() != current_head {
                if selected_profile {
                    return Err(AuthoritativeReviewAdmissionRuntimeError::Publication(
                        AuthoritativeReviewAdmissionPublicationError::RetainedGenerationChanged,
                    ));
                }
                continue;
            }
            for witness in &mut publication_store.retained_file_witnesses {
                revalidate_retained_file_witness(witness).map_err(|()| {
                    AuthoritativeReviewAdmissionRuntimeError::Publication(
                        AuthoritativeReviewAdmissionPublicationError::RetainedGenerationChanged,
                    )
                })?;
            }
            let mut publication = publish_record_bytes(
                &mut publication_store,
                admission_record.record_id(),
                &admission_record_bytes,
            )
            .map_err(|_| {
                AuthoritativeReviewAdmissionRuntimeError::Publication(
                    AuthoritativeReviewAdmissionPublicationError::RecordPublication,
                )
            })?;
            publication_store
                .retained_journal
                .resolve_reference(&current_head)
                .map_err(|error| {
                    AuthoritativeReviewAdmissionRuntimeError::Publication(
                        AuthoritativeReviewAdmissionPublicationError::OperationStartReference(
                            error,
                        ),
                    )
                })?;
            let mut retained_generation_guard = publication_store
                .acquire_retained_generation_guard(Some(&publication.retained_witness))
                .map_err(|()| {
                    AuthoritativeReviewAdmissionRuntimeError::Publication(
                        AuthoritativeReviewAdmissionPublicationError::RetainedGenerationChanged,
                    )
                })?;
            let journal_hold = publication_store
                .acquire_publication_directory_hold(2, "journal")
                .map_err(|()| {
                    AuthoritativeReviewAdmissionRuntimeError::Publication(
                        AuthoritativeReviewAdmissionPublicationError::RetainedGenerationChanged,
                    )
                })?;
            for witness in &mut retained_generation_guard {
                revalidate_retained_file_witness(witness).map_err(|()| {
                    AuthoritativeReviewAdmissionRuntimeError::Publication(
                        AuthoritativeReviewAdmissionPublicationError::RetainedGenerationChanged,
                    )
                })?;
            }
            let journal_dir = self.root.join("journal");
            let journal_name = format!("{:020}.cbor", next_index.value());
            let entry_hash =
                JournalEntryHash::try_from(Sha256::digest(&journal_entry_bytes).as_slice())
                    .expect("SHA-256 has JournalEntryHash width");
            let journal_reference = JournalReference::new(
                current_head.registry_id(),
                next_index,
                entry_hash,
                journal_entry.event_type_id(),
                journal_entry.event_record_id(),
            );
            let uncertain_publication = AuthoritativeReviewAdmissionPublishedReceiptUncertain {
                admission_record_id: admission_record.record_id(),
                admission_record_bytes: admission_record_bytes.clone(),
                journal_reference: journal_reference.clone(),
                journal_entry_bytes: journal_entry_bytes.clone(),
                operation_start_journal_ref: section_82.operation_start_journal_ref().clone(),
            };
            let mut journal_publication = match publish_journal_slot(
                &journal_dir,
                &journal_hold,
                Path::new(&journal_name),
                &journal_entry_bytes,
            )
            .map_err(|_| {
                AuthoritativeReviewAdmissionRuntimeError::Publication(
                    AuthoritativeReviewAdmissionPublicationError::JournalPublication,
                )
            })? {
                JournalSlotPublication::Conflict => {
                    if selected_profile {
                        return Err(AuthoritativeReviewAdmissionRuntimeError::Publication(
                            AuthoritativeReviewAdmissionPublicationError::RetainedGenerationChanged,
                        ));
                    }
                    continue;
                }
                JournalSlotPublication::Published(publication) => publication,
                JournalSlotPublication::VisibleReceiptUncertain => {
                    return Ok(
                        AuthoritativeReviewAdmissionRuntimeOutcome::PublishedReceiptUncertain(
                            Box::new(uncertain_publication),
                        ),
                    );
                }
            };
            macro_rules! post_visibility_try {
                ($result:expr) => {
                    match $result {
                        Ok(value) => value,
                        Err(_) => {
                            return Ok(AuthoritativeReviewAdmissionRuntimeOutcome::PublishedReceiptUncertain(
                                Box::new(uncertain_publication.clone()),
                            ));
                        }
                    }
                };
            }
            post_visibility_try!(revalidate_retained_file_witness(
                &mut publication.retained_witness
            ));
            post_visibility_try!(revalidate_retained_file_witness(
                &mut journal_publication.retained_witness
            ));
            match publication_store
                .records
                .binary_search_by(|(stored_id, _)| {
                    stored_id
                        .as_bytes()
                        .cmp(admission_record.record_id().as_bytes())
                }) {
                Ok(index) if publication_store.records[index].1 == admission_record_bytes => {}
                Ok(_) => {
                    return Ok(
                        AuthoritativeReviewAdmissionRuntimeOutcome::PublishedReceiptUncertain(
                            Box::new(uncertain_publication.clone()),
                        ),
                    );
                }
                Err(index) => publication_store.records.insert(
                    index,
                    (admission_record.record_id(), admission_record_bytes.clone()),
                ),
            }
            post_visibility_try!(publication_store
                .retained_journal
                .append_strict_entry(&journal_entry_bytes));
            post_visibility_try!(validate_authoritative_event_records(
                &publication_store.retained_journal,
                &publication_store.records,
            ));
            if selected_profile {
                post_visibility_try!(
                    publication_store.validate_selected_terminal_admission_replay()
                );
            }
            if authenticate_published_journal_reference(
                &publication_store.retained_journal,
                &journal_reference,
            )
            .is_err()
            {
                return Ok(
                    AuthoritativeReviewAdmissionRuntimeOutcome::PublishedReceiptUncertain(
                        Box::new(uncertain_publication.clone()),
                    ),
                );
            }
            let journal_witness = post_visibility_try!(downgrade_publication_witness(
                journal_publication.retained_witness,
            ));
            let mut post_publication_guards = Vec::with_capacity(2);
            if publication.facts.atomic_no_replace == DurabilityActionState::Performed {
                let record_witness = post_visibility_try!(downgrade_publication_witness(
                    publication.retained_witness
                ));
                post_publication_guards.push(post_visibility_try!(open_retained_file_guard(
                    &record_witness
                )));
                publication_store
                    .retained_file_witnesses
                    .push(record_witness);
            }
            post_publication_guards.push(post_visibility_try!(open_retained_file_guard(
                &journal_witness
            )));
            publication_store
                .retained_file_witnesses
                .push(journal_witness);
            let replayed = post_visibility_try!(Self::open_for_review_admission_runtime(
                &self.root,
                selected_profile,
            ));
            if replayed.namespace_holds.len() != publication_store.namespace_holds.len()
                || replayed
                    .namespace_holds
                    .iter()
                    .zip(&publication_store.namespace_holds)
                    .any(|(replayed, retained)| !handles_identify_same_object(replayed, retained))
            {
                return Ok(
                    AuthoritativeReviewAdmissionRuntimeOutcome::PublishedReceiptUncertain(
                        Box::new(uncertain_publication.clone()),
                    ),
                );
            }
            if replayed.retained_journal.current_head_reference() != journal_reference
                || replayed.resolve(admission_record.record_id())
                    != Some(admission_record_bytes.as_slice())
            {
                return Ok(
                    AuthoritativeReviewAdmissionRuntimeOutcome::PublishedReceiptUncertain(
                        Box::new(uncertain_publication),
                    ),
                );
            }
            *self = replayed;
            return Ok(AuthoritativeReviewAdmissionRuntimeOutcome::Published(
                Box::new(AuthoritativeReviewAdmissionPublication {
                    policy_completion,
                    disposition,
                    admission_record,
                    admission_record_bytes,
                    journal_entry,
                    journal_entry_bytes,
                    journal_reference,
                    operation_start_journal_ref: section_82.operation_start_journal_ref().clone(),
                    freeze_authority: section_82.freeze_authority().clone(),
                    durability: AuthoritativePublicationDurability {
                        record_content_flush: publication.facts.content_flush,
                        journal_content_flush: journal_publication.facts.content_flush,
                        record_atomic_publish_no_replace: publication.facts.atomic_no_replace,
                        journal_atomic_publish_no_replace: journal_publication
                            .facts
                            .atomic_no_replace,
                        record_parent_directory_flush: publication.facts.parent_directory_flush,
                        journal_parent_directory_flush: journal_publication
                            .facts
                            .parent_directory_flush,
                        // The exact completed actions above are receipt facts. This runtime has no
                        // independently authenticated, operation-scoped storage-capability basis
                        // from which to promote them into a strongest-available platform claim.
                        platform_strongest_available: false,
                    },
                }),
            ));
        }
        Err(AuthoritativeReviewAdmissionRuntimeError::Publication(
            AuthoritativeReviewAdmissionPublicationError::PublicationConflictExhausted,
        ))
    }

    /// Replays every retained selected terminal Admission from exact retained inputs.
    ///
    /// A terminal marker is only a format selector. It cannot replace the
    /// selected Request/Result, §82, §46, §83, and exact-record reconstruction
    /// checks performed here after the selected Store has retained all current
    /// namespace witnesses.
    fn validate_selected_terminal_admission_replay(
        &self,
    ) -> Result<(), AuthoritativeRegistryStoreOpenError> {
        let mut freeze_cache = Vec::new();
        let mut replay_work_bytes = 0;
        for (position, entry) in self.retained_journal.entries.iter().enumerate() {
            if !matches!(entry.event_type_id().value(), 302 | 303) {
                continue;
            }
            let event_reference = JournalReference::new(
                entry.registry_id(),
                entry.entry_index(),
                entry.entry_hash(),
                entry.event_type_id(),
                entry.event_record_id(),
            );
            let record_id = record_id_from_event_reference(&event_reference);
            let admission_bytes = self
                .resolve(record_id)
                .ok_or(AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            let admission = ReviewAdmissionRecord::decode_authoritative(admission_bytes)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            if admission.terminal_authority_closure_sha256()
                != Some(&TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256)
            {
                continue;
            }
            let predecessor = position
                .checked_sub(1)
                .and_then(|index| self.retained_journal.entries.get(index))
                .ok_or(AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            let operation_start = JournalReference::new(
                predecessor.registry_id(),
                predecessor.entry_index(),
                predecessor.entry_hash(),
                predecessor.event_type_id(),
                predecessor.event_record_id(),
            );
            if admission.operation_start_journal_ref() != &operation_start {
                return Err(AuthoritativeRegistryStoreOpenError::EventRecordDecode);
            }

            let recorded_result = self
                .validate_selected_review_result_from_retained_snapshot(
                    admission
                        .review_result_ref()
                        .ok_or(AuthoritativeRegistryStoreOpenError::EventRecordDecode)?
                        .clone(),
                    &mut freeze_cache,
                    &mut replay_work_bytes,
                    AUTHORITATIVE_STORE_MAX_NAMESPACE_BYTES as u64,
                )
                .map_err(|error| match error {
                    SelectedReviewResultError::Request(
                        SelectedReviewRequestError::ReplayResourceLimit,
                    )
                    | SelectedReviewResultError::Request(
                        SelectedReviewRequestError::FreezeAuthority(
                            AuthoritativeFreezeCommittedBindingError::ResourceLimit,
                        ),
                    ) => AuthoritativeRegistryStoreOpenError::SelectedTerminalReplayResourceLimit,
                    _ => AuthoritativeRegistryStoreOpenError::EventRecordDecode,
                })?;
            let request = recorded_result.request().request();
            let result = recorded_result.result();
            if admission.review_request_ref()
                != Some(recorded_result.request().request_event_reference())
                || admission.policy_authority_ref()
                    != Some(recorded_result.request().policy_authority_ref())
            {
                return Err(AuthoritativeRegistryStoreOpenError::EventRecordDecode);
            }
            let request_bytes = request.authoritative_cbor();
            let result_bytes = result.authoritative_cbor();
            let returned_anchor = resolve_retained_review_package_anchor_input(
                &self.retained_journal,
                recorded_result.request().request_event_reference(),
                &request_bytes,
                recorded_result.result_event_reference(),
                &result_bytes,
            )
            .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            let predecessor_anchor = JournalAnchor::new(
                operation_start.registry_id(),
                operation_start.entry_index(),
                operation_start.entry_hash(),
                1,
            )
            .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            let anchor_comparison = if returned_anchor == predecessor_anchor {
                JournalAnchorHistoryComparison::AnchorEqualsCurrentHead
            } else {
                match compare_retained_journal_anchor_history(
                    &self.retained_journal,
                    &returned_anchor,
                ) {
                    JournalAnchorHistoryComparison::AnchorIsValidAncestor
                    | JournalAnchorHistoryComparison::AnchorEqualsCurrentHead => {
                        JournalAnchorHistoryComparison::AnchorIsValidAncestor
                    }
                    _ => return Err(AuthoritativeRegistryStoreOpenError::EventRecordDecode),
                }
            };
            let policy_bytes = self
                .resolve(record_id_from_event_reference(
                    request.policy_authority_ref(),
                ))
                .ok_or(AuthoritativeRegistryStoreOpenError::EventRecordDecode)?
                .to_vec();
            let policy = ReviewAdmissionPolicyRecord::decode_authoritative(&policy_bytes)
                .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            if !selected_review_policy_is_supported(&policy, self) {
                return Err(AuthoritativeRegistryStoreOpenError::EventRecordDecode);
            }
            let policy_context_prerequisites =
                validate_review_admission_policy_context_prerequisites(
                    &self.retained_journal,
                    request.policy_authority_ref(),
                    &policy_bytes,
                    &request_bytes,
                    &result_bytes,
                    self,
                )
                .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            let section_82 = AuthoritativeReviewAdmissionSection82 {
                operation_start_journal_ref: operation_start,
                request_event_reference: recorded_result
                    .request()
                    .request_event_reference()
                    .clone(),
                result_event_reference: recorded_result.result_event_reference().clone(),
                request: request.clone(),
                result: result.clone(),
                policy,
                policy_bytes,
                returned_anchor,
                anchor_comparison,
                policy_context_prerequisites,
                freeze_authority: recorded_result.request().freeze_authority().clone(),
            };
            let completion = evaluate_selected_review_admission_policy_profile(&section_82);
            let disposition_id = match completion.result() {
                ReviewAdmissionCompletedPolicyResult::Satisfied => 1,
                ReviewAdmissionCompletedPolicyResult::GateUnsatisfied
                | ReviewAdmissionCompletedPolicyResult::GateIndeterminate => 2,
            };
            let mut reason_codes = result.reason_codes().to_vec();
            reason_codes.push(
                match completion.result() {
                    ReviewAdmissionCompletedPolicyResult::Satisfied => "POLICY_SATISFIED",
                    ReviewAdmissionCompletedPolicyResult::GateUnsatisfied => {
                        "POLICY_GATE_UNSATISFIED"
                    }
                    ReviewAdmissionCompletedPolicyResult::GateIndeterminate => {
                        "POLICY_GATE_INDETERMINATE"
                    }
                }
                .to_owned(),
            );
            reason_codes.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
            reason_codes.dedup();
            let expected = ReviewAdmissionRecord::new_selected(ReviewAdmissionRecordInput {
                disposition_id,
                review_request_ref: section_82.request_event_reference().clone(),
                review_result_ref: section_82.result_event_reference().clone(),
                policy_authority_ref: section_82.policy_authority_ref().clone(),
                reason_codes,
                operation_start_journal_ref: section_82.operation_start_journal_ref().clone(),
            })
            .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            if expected.authoritative_cbor() != admission_bytes {
                return Err(AuthoritativeRegistryStoreOpenError::EventRecordDecode);
            }
        }
        Ok(())
    }

    /// Reserves one complete bounded traversal allowance before every live selected Freeze walk
    /// during one selected terminal replay. Cache hits retain a record-derived binding, not a
    /// stable payload-generation witness, so they must revalidate the live payload too. The
    /// conservative allowance covers empty directories, names, metadata, cloned paths, and
    /// content—not only manifest-declared file bytes—so zero-byte payloads cannot evade the
    /// aggregate replay-work envelope.
    fn reserve_selected_terminal_replay_payload_budget(
        &self,
        committed_event_reference: &JournalReference,
        replay_work_bytes: &mut u64,
        max_replay_work_bytes: u64,
    ) -> Result<(), SelectedTerminalReplayPayloadBudgetError> {
        let receipt_record_id = record_id_from_event_reference(committed_event_reference);
        let receipt = self
            .resolve(receipt_record_id)
            .and_then(|bytes| FreezeReceiptRecord::decode_authoritative(bytes).ok())
            .filter(|receipt| receipt.record_id() == receipt_record_id)
            .ok_or(SelectedTerminalReplayPayloadBudgetError::FreezePrerequisiteInvalid)?;
        let _manifest = self
            .resolve(receipt.input().manifest_id)
            .and_then(|bytes| ManifestRecord::decode_authoritative(bytes).ok())
            .filter(|manifest| manifest.record_id() == receipt.input().manifest_id)
            .ok_or(SelectedTerminalReplayPayloadBudgetError::FreezePrerequisiteInvalid)?;
        let traversal_work_bytes = u64::try_from(AUTHORITATIVE_STORE_MAX_NAMESPACE_BYTES)
            .map_err(|_| SelectedTerminalReplayPayloadBudgetError::ResourceLimit)?;
        let next = replay_work_bytes
            .checked_add(traversal_work_bytes)
            .ok_or(SelectedTerminalReplayPayloadBudgetError::ResourceLimit)?;
        if next > max_replay_work_bytes {
            return Err(SelectedTerminalReplayPayloadBudgetError::ResourceLimit);
        }
        *replay_work_bytes = next;
        Ok(())
    }

    fn open_for_review_admission_runtime(
        root: &Path,
        selected_profile: bool,
    ) -> Result<Self, AuthoritativeRegistryStoreOpenError> {
        if selected_profile {
            Self::open_selected_profile(root)
        } else {
            Self::open(root)
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn reload_authoritative_namespaces(
        &mut self,
    ) -> Result<(), AuthoritativeReviewAdmissionAcceptanceError> {
        self.reload_authoritative_namespaces_with_hook(|| {})
    }

    #[cfg(not(target_os = "macos"))]
    fn reload_authoritative_namespaces_with_hook<F>(
        &mut self,
        after_precheck: F,
    ) -> Result<(), AuthoritativeReviewAdmissionAcceptanceError>
    where
        F: FnOnce(),
    {
        self.reload_authoritative_namespaces_with_hooks(after_precheck, || {})
    }

    #[cfg(not(target_os = "macos"))]
    fn reload_authoritative_namespaces_with_hooks<F, G>(
        &mut self,
        after_precheck: F,
        after_reopen: G,
    ) -> Result<(), AuthoritativeReviewAdmissionAcceptanceError>
    where
        F: FnOnce(),
        G: FnOnce(),
    {
        self.revalidate_retained_generation().map_err(|_| {
            AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged,
            )
        })?;
        after_precheck();
        let mut reloaded = Self::open_with_generation_hook(&self.root, self.open_profile, || {})
            .map_err(AuthoritativeReviewAdmissionAcceptanceError::Store)?;
        after_reopen();
        self.revalidate_retained_generation().map_err(|_| {
            AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged,
            )
        })?;
        if reloaded.namespace_holds.len() != self.namespace_holds.len()
            || reloaded
                .namespace_holds
                .iter()
                .zip(&self.namespace_holds)
                .any(|(reloaded, retained)| !handles_identify_same_object(reloaded, retained))
        {
            return Err(AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged,
            ));
        }
        Self::rebind_reloaded_witness_paths_to_original_holds(self, &mut reloaded).map_err(
            |_| {
                AuthoritativeReviewAdmissionAcceptanceError::Store(
                    AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged,
                )
            },
        )?;
        if self.retained_file_witnesses.iter().any(|retained| {
            !reloaded.retained_file_witnesses.iter().any(|reloaded| {
                reloaded.path == retained.path
                    && handles_identify_same_object(&reloaded.file, &retained.file)
                    && reloaded.expected_length == retained.expected_length
                    && reloaded.expected_sha256 == retained.expected_sha256
            })
        }) {
            return Err(AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged,
            ));
        }
        if reloaded.retained_journal.registry_id != self.retained_journal.registry_id {
            return Err(AuthoritativeReviewAdmissionAcceptanceError::RegistryIdentityChanged);
        }
        self.retained_journal = reloaded.retained_journal;
        self.records = reloaded.records;
        self.retained_file_witnesses = reloaded.retained_file_witnesses;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn reload_authoritative_namespaces(
        &mut self,
    ) -> Result<(), AuthoritativeReviewAdmissionAcceptanceError> {
        self.reload_authoritative_namespaces_with_hook(|| {})
    }

    #[cfg(test)]
    #[cfg(target_os = "macos")]
    fn reload_authoritative_namespaces_with_hook<F>(
        &mut self,
        before_reopen: F,
    ) -> Result<(), AuthoritativeReviewAdmissionAcceptanceError>
    where
        F: FnOnce(),
    {
        self.reload_authoritative_namespaces_with_reopen_hook(before_reopen)
    }

    #[cfg(not(test))]
    #[cfg(target_os = "macos")]
    fn reload_authoritative_namespaces_with_hook<F>(
        &mut self,
        before_reopen: F,
    ) -> Result<(), AuthoritativeReviewAdmissionAcceptanceError>
    where
        F: FnOnce(),
    {
        self.reload_authoritative_namespaces_with_reopen_hook(before_reopen)
    }

    #[cfg(target_os = "macos")]
    fn reload_authoritative_namespaces_with_reopen_hook<F>(
        &mut self,
        before_reopen: F,
    ) -> Result<(), AuthoritativeReviewAdmissionAcceptanceError>
    where
        F: FnOnce(),
    {
        // Profile L coordinates ordinary writers; exact retained identities and bytes, not
        // timestamps, keep an old authoritative generation continuous through this reopen.
        self.revalidate_retained_generation().map_err(|_| {
            AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged,
            )
        })?;
        before_reopen();
        self.revalidate_retained_generation().map_err(|_| {
            AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged,
            )
        })?;
        let reloaded = Self::open_with_generation_hook(&self.root, self.open_profile, || {})
            .map_err(AuthoritativeReviewAdmissionAcceptanceError::Store)?;
        self.revalidate_retained_generation().map_err(|_| {
            AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged,
            )
        })?;
        self.ensure_candidate_continues_retained_generation(&reloaded)
            .map_err(|_| {
                AuthoritativeReviewAdmissionAcceptanceError::Store(
                    AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged,
                )
            })?;
        if reloaded.retained_journal.registry_id != self.retained_journal.registry_id {
            return Err(AuthoritativeReviewAdmissionAcceptanceError::RegistryIdentityChanged);
        }
        self.retained_journal = reloaded.retained_journal;
        self.records = reloaded.records;
        self.namespace_holds = reloaded.namespace_holds;
        self.retained_file_witnesses = reloaded.retained_file_witnesses;
        Ok(())
    }

    #[cfg(all(test, target_os = "macos"))]
    fn reload_authoritative_namespaces_with_candidate_loader<F>(
        &mut self,
        loader: F,
    ) -> Result<(), AuthoritativeReviewAdmissionAcceptanceError>
    where
        F: FnOnce(&Path) -> Result<Self, AuthoritativeRegistryStoreOpenError>,
    {
        self.revalidate_retained_generation().map_err(|_| {
            AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged,
            )
        })?;
        let reloaded =
            loader(&self.root).map_err(AuthoritativeReviewAdmissionAcceptanceError::Store)?;
        self.revalidate_retained_generation().map_err(|_| {
            AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged,
            )
        })?;
        self.ensure_candidate_continues_retained_generation(&reloaded)
            .map_err(|_| {
                AuthoritativeReviewAdmissionAcceptanceError::Store(
                    AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged,
                )
            })?;
        if reloaded.retained_journal.registry_id != self.retained_journal.registry_id {
            return Err(AuthoritativeReviewAdmissionAcceptanceError::RegistryIdentityChanged);
        }
        self.retained_journal = reloaded.retained_journal;
        self.records = reloaded.records;
        self.namespace_holds = reloaded.namespace_holds;
        self.retained_file_witnesses = reloaded.retained_file_witnesses;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn ensure_candidate_continues_retained_generation(&self, candidate: &Self) -> Result<(), ()> {
        if self.namespace_holds.len() != candidate.namespace_holds.len()
            || !self
                .namespace_holds
                .iter()
                .zip(&candidate.namespace_holds)
                .all(|(old, fresh)| handles_identify_same_object(old, fresh))
        {
            return Err(());
        }
        for old in &self.retained_file_witnesses {
            if !candidate.retained_file_witnesses.iter().any(|fresh| {
                old.path == fresh.path
                    && old.expected_length == fresh.expected_length
                    && old.expected_sha256 == fresh.expected_sha256
                    && handles_identify_same_object(&old.file, &fresh.file)
            }) {
                return Err(());
            }
        }
        Ok(())
    }

    fn acquire_retained_generation_guard(
        &self,
        already_guarded: Option<&RetainedFileWitness>,
    ) -> Result<Vec<RetainedFileWitness>, ()> {
        let mut guards = Vec::new();
        guards
            .try_reserve_exact(self.retained_file_witnesses.len())
            .map_err(|_| ())?;
        for witness in &self.retained_file_witnesses {
            if already_guarded
                .is_some_and(|guard| handles_identify_same_object(&witness.file, &guard.file))
            {
                continue;
            }
            guards.push(open_retained_file_guard(witness)?);
        }
        Ok(guards)
    }

    fn revalidate_retained_generation(&self) -> Result<(), ()> {
        self.revalidate_retained_namespace_holds()?;
        for witness in &self.retained_file_witnesses {
            let mut guard = open_retained_file_guard(witness)?;
            revalidate_retained_file_witness(&mut guard)?;
        }
        self.revalidate_retained_namespace_holds()?;
        Ok(())
    }

    fn revalidate_retained_namespace_holds(&self) -> Result<(), ()> {
        let [root_hold, registry_hold, journal_hold, records_hold, remainder @ ..] =
            self.namespace_holds.as_slice()
        else {
            return Err(());
        };
        ensure_path_matches_handle(&self.root, root_hold)?;
        ensure_path_matches_handle(&self.root.join("registry"), registry_hold)?;
        ensure_path_matches_handle(&self.root.join("journal"), journal_hold)?;
        ensure_path_matches_handle(&self.root.join("records"), records_hold)?;
        match self.open_profile {
            AuthoritativeRegistryStoreOpenProfile::LegacyThreeNamespace if remainder.is_empty() => {
                Ok(())
            }
            AuthoritativeRegistryStoreOpenProfile::SelectedTerminalAuthorityClosure => {
                let [roots_hold, coordination_hold, freeze_hold, staging_hold] = remainder else {
                    return Err(());
                };
                ensure_path_matches_handle(&self.root.join("roots"), roots_hold)?;
                ensure_path_matches_handle(&self.root.join("coordination"), coordination_hold)?;
                ensure_path_matches_handle(&self.root.join("coordination/freeze"), freeze_hold)?;
                ensure_path_matches_handle(&self.root.join("coordination/staging"), staging_hold)
            }
            _ => Err(()),
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn rebind_reloaded_witness_paths_to_original_holds(
        original: &Self,
        reloaded: &mut Self,
    ) -> Result<(), ()> {
        let names = [None, Some("registry"), Some("journal"), Some("records")];
        let mut old_bases = Vec::with_capacity(names.len());
        let mut new_bases = Vec::with_capacity(names.len());
        for (index, name) in names.into_iter().enumerate() {
            let old_path =
                name.map_or_else(|| original.root.clone(), |name| original.root.join(name));
            let new_path =
                name.map_or_else(|| reloaded.root.clone(), |name| reloaded.root.join(name));
            old_bases.push(namespace_contents_path(
                &old_path,
                original.namespace_holds.get(index).ok_or(())?,
            )?);
            new_bases.push(namespace_contents_path(
                &new_path,
                reloaded.namespace_holds.get(index).ok_or(())?,
            )?);
        }
        for witness in &mut reloaded.retained_file_witnesses {
            let parent = witness.path.parent().ok_or(())?;
            let leaf = witness.path.file_name().ok_or(())?;
            let matches: Vec<_> = new_bases
                .iter()
                .enumerate()
                .filter_map(|(index, base)| (parent == base).then_some(index))
                .collect();
            let [index] = matches.as_slice() else {
                return Err(());
            };
            witness.path = old_bases[*index].join(leaf);
        }
        Ok(())
    }

    fn acquire_publication_directory_hold(
        &self,
        retained_index: usize,
        name: &str,
    ) -> Result<fs::File, ()> {
        open_publication_child_directory_hold(
            self.namespace_holds.first().ok_or(())?,
            &self.root,
            self.namespace_holds.get(retained_index).ok_or(())?,
            name,
        )
    }

    fn release_authoritative_acceptance_capacity(&self) {
        let previous = self
            .live_instance_identity
            .outstanding_review_admissions
            .fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0);
    }
}

/// Observes only capabilities that can be exercised safely in the selected
/// nonauthoritative staging namespace. It deliberately leaves unrelated
/// dimensions unprobed instead of promoting implementation assumptions.
fn probe_selected_initial_storage_capabilities(
    root: &Path,
) -> Result<StorageCapabilityClassRecordInput, ()> {
    let root_hold = open_real_directory_hold(root)?;
    let coordination = root.join("coordination");
    let coordination_hold = open_child_directory_hold(&root_hold, root, "coordination")?;
    let staging = coordination.join("staging");
    let staging_hold = open_child_directory_hold(&coordination_hold, &coordination, "staging")?;
    let staging_publication_hold = open_publication_child_directory_hold(
        &coordination_hold,
        &coordination,
        &staging_hold,
        "staging",
    )?;
    ensure_path_matches_handle(&staging, &staging_publication_hold)?;

    let temporary = staging.join(format!("{}.tmp", selected_staging_probe_token()?));
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|_| ())?;
    let write_result = file
        .write_all(b"EvidenceRegistry.selected-staging-capability-probe.v1")
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all());
    if write_result.is_err() {
        drop(file);
        return Err(());
    }
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
    {
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        _ => return Err(()),
    }
    drop(file);
    sync_retained_directory(&staging, &staging_publication_hold)?;
    fs::remove_file(&temporary).map_err(|_| ())?;
    sync_retained_directory(&staging, &staging_publication_hold)?;

    Ok(StorageCapabilityClassRecordInput {
        filesystem_transport: std::env::consts::FAMILY.to_owned(),
        sync_management: "std::fs::File::sync_all".to_owned(),
        placeholder_capability: 3,
        exclusive_create_capability: 1,
        no_replace_publication_capability: 3,
        locking_capability: 3,
        atomic_rename_capability: 3,
        file_flush_capability: 1,
        directory_flush_capability: 1,
    })
}

fn selected_staging_probe_token() -> Result<String, ()> {
    let mut bytes = [0_u8; 16];
    #[cfg(windows)]
    {
        #[link(name = "bcrypt")]
        unsafe extern "system" {
            fn BCryptGenRandom(
                algorithm: isize,
                buffer: *mut u8,
                buffer_length: u32,
                flags: u32,
            ) -> i32;
        }
        const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x0000_0002;
        if unsafe {
            BCryptGenRandom(
                0,
                bytes.as_mut_ptr(),
                u32::try_from(bytes.len()).map_err(|_| ())?,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        } < 0
        {
            return Err(());
        }
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        fs::File::open("/dev/urandom")
            .and_then(|mut random| random.read_exact(&mut bytes))
            .map_err(|_| ())?;
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        return Err(());
    }
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn validate_selected_genesis_observation_records(
    records: &[(RecordId, Vec<u8>)],
    genesis: &GenesisRecord,
) -> Result<(), AuthoritativeRegistryStoreOpenError> {
    let resolve = |record_id: RecordId| {
        records
            .binary_search_by(|(stored_id, _)| stored_id.as_bytes().cmp(record_id.as_bytes()))
            .ok()
            .map(|index| records[index].1.as_slice())
    };
    let capability = resolve(genesis.input.storage_capability_class_id)
        .ok_or(AuthoritativeRegistryStoreOpenError::GenesisCapabilityObservationInvalid)?;
    let decoded_capability = StorageCapabilityClassRecord::decode_authoritative(capability)
        .map_err(|_| AuthoritativeRegistryStoreOpenError::GenesisCapabilityObservationInvalid)?;
    if decoded_capability.record_id() != genesis.input.storage_capability_class_id {
        return Err(AuthoritativeRegistryStoreOpenError::GenesisCapabilityObservationInvalid);
    }
    let environment = resolve(genesis.input.environment_observation_id)
        .ok_or(AuthoritativeRegistryStoreOpenError::GenesisEnvironmentObservationInvalid)?;
    let decoded_environment = EnvironmentObservationRecord::decode_authoritative(environment)
        .map_err(|_| AuthoritativeRegistryStoreOpenError::GenesisEnvironmentObservationInvalid)?;
    if decoded_environment.record_id() != genesis.input.environment_observation_id {
        return Err(AuthoritativeRegistryStoreOpenError::GenesisEnvironmentObservationInvalid);
    }
    Ok(())
}

fn frozen_generic_policy_scope_applicability_unavailable() -> bool {
    // The governing frozen authorities assign no generic operation/subject-to-Scope predicate.
    // This fixed gate must remain fail-closed unless a frozen successor supplies that relation.
    true
}

/// Retains the previously implemented profile-specific composition behind the fixed frozen-Scope
/// fail-closed gate. Evaluator 1015 is not assigned by the governing frozen v0.3 authorities, so
/// this helper is private and cannot authorize §46 completion or terminal publication.
fn evaluate_unfrozen_review_admission_policy_profile(
    section_82: &AuthoritativeReviewAdmissionSection82,
) -> ReviewAdmissionPolicy46Completion {
    evaluate_profile_requirements_without_generic_scope(
        section_82.policy(),
        section_82.request(),
        section_82.result(),
        section_82.anchor_comparison(),
    )
}

/// Evaluates the frozen selected profile's exact gate Scope rule. The selected
/// entry point has already established the exact `[2, 5]` Policy context and
/// profile-specific Request/Result carriers before invoking this evaluator.
fn evaluate_selected_review_admission_policy_profile(
    section_82: &AuthoritativeReviewAdmissionSection82,
) -> ReviewAdmissionPolicy46Completion {
    let mut completion = evaluate_profile_requirements_without_generic_scope(
        section_82.policy(),
        section_82.request(),
        section_82.result(),
        section_82.anchor_comparison(),
    );
    let prerequisites = section_82.policy_context_prerequisites();
    completion
        .evaluator_results
        .push(ReviewAdmissionIndividualEvaluatorResult {
            evaluator_id: 1015,
            outcome: pass_or_fail(
                prerequisites.gate_scope_ref() == prerequisites.common_review_scope_ref(),
            ),
        });
    completion.result =
        if completion
            .evaluator_results
            .iter()
            .any(|result| result.outcome() == ReviewAdmissionIndividualEvaluatorOutcome::Fail)
        {
            ReviewAdmissionCompletedPolicyResult::GateUnsatisfied
        } else if completion.evaluator_results.iter().any(|result| {
            result.outcome() == ReviewAdmissionIndividualEvaluatorOutcome::Indeterminate
        }) {
            ReviewAdmissionCompletedPolicyResult::GateIndeterminate
        } else {
            ReviewAdmissionCompletedPolicyResult::Satisfied
        };
    completion
}

fn evaluate_profile_requirements_without_generic_scope(
    policy: &ReviewAdmissionPolicyRecord,
    request: &ReviewRequestRecord,
    result: &ReviewResultRecord,
    anchor_comparison: JournalAnchorHistoryComparison,
) -> ReviewAdmissionPolicy46Completion {
    let mut evaluator_results = Vec::with_capacity(4);

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
                    .contains(&anchor_relation_id(anchor_comparison)),
            ),
        });
    }

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

fn map_selected_embedded_preparation_reload_error(
    error: AuthoritativeReviewAdmissionAcceptanceError,
) -> SelectedEmbeddedFreezePreparationError {
    match error {
        AuthoritativeReviewAdmissionAcceptanceError::Store(error) => {
            SelectedEmbeddedFreezePreparationError::Store(error)
        }
        AuthoritativeReviewAdmissionAcceptanceError::RegistryIdentityChanged => {
            SelectedEmbeddedFreezePreparationError::ReplayMismatch
        }
        AuthoritativeReviewAdmissionAcceptanceError::Input(_) => {
            SelectedEmbeddedFreezePreparationError::ReplayMismatch
        }
    }
}

fn map_selected_embedded_commit_reload_error(
    error: AuthoritativeReviewAdmissionAcceptanceError,
) -> SelectedEmbeddedFreezeCommitError {
    match error {
        AuthoritativeReviewAdmissionAcceptanceError::Store(error) => {
            SelectedEmbeddedFreezeCommitError::Store(error)
        }
        AuthoritativeReviewAdmissionAcceptanceError::RegistryIdentityChanged => {
            SelectedEmbeddedFreezeCommitError::ReplayMismatch
        }
        AuthoritativeReviewAdmissionAcceptanceError::Input(_) => {
            SelectedEmbeddedFreezeCommitError::ReplayMismatch
        }
    }
}

fn map_selected_review_request_reload_error(
    error: AuthoritativeReviewAdmissionAcceptanceError,
) -> SelectedReviewRequestError {
    match error {
        AuthoritativeReviewAdmissionAcceptanceError::Store(error) => {
            SelectedReviewRequestError::Store(error)
        }
        AuthoritativeReviewAdmissionAcceptanceError::RegistryIdentityChanged => {
            SelectedReviewRequestError::ReplayMismatch
        }
        AuthoritativeReviewAdmissionAcceptanceError::Input(_) => {
            SelectedReviewRequestError::ReplayMismatch
        }
    }
}

fn map_selected_review_result_reload_error(
    error: AuthoritativeReviewAdmissionAcceptanceError,
) -> SelectedReviewResultError {
    match error {
        AuthoritativeReviewAdmissionAcceptanceError::Store(error) => {
            SelectedReviewResultError::Store(error)
        }
        AuthoritativeReviewAdmissionAcceptanceError::RegistryIdentityChanged
        | AuthoritativeReviewAdmissionAcceptanceError::Input(_) => {
            SelectedReviewResultError::ReplayMismatch
        }
    }
}

fn selected_policy_authority_reference(
    journal: &RetainedJournal,
    policy_record_id: RecordId,
) -> Result<JournalReference, SelectedReviewRequestError> {
    let mut selected = None;
    for entry in &journal.entries {
        if entry.event_type_id().value() != 400
            || entry.event_record_id().as_bytes() != policy_record_id.as_bytes()
        {
            continue;
        }
        let reference = JournalReference::new(
            entry.registry_id(),
            entry.entry_index(),
            entry.entry_hash(),
            entry.event_type_id(),
            entry.event_record_id(),
        );
        if selected.replace(reference).is_some() {
            return Err(SelectedReviewRequestError::PolicyAuthorityAmbiguous);
        }
    }
    selected.ok_or(SelectedReviewRequestError::PolicyAuthorityMissing)
}

fn selected_review_policy_is_supported(
    policy: &ReviewAdmissionPolicyRecord,
    resolver: &impl ExactRecordByteResolver,
) -> bool {
    policy.supported_context_ids() == [2, 5]
        && resolver
            .resolve(policy.gate_scope_ref())
            .and_then(|bytes| ScopeRecord::decode_authoritative(bytes).ok())
            .is_some_and(|scope| {
                scope.record_id() == policy.gate_scope_ref()
                    && scope.input().scope_profile_id == 1
                    && scope.input().scope_profile_version == 1
                    && scope.input().scope_payload.is_empty()
            })
}

fn selected_review_policy_requirement(
    policy: &ReviewAdmissionPolicyRecord,
    review_role_id: u64,
) -> Result<ReviewAdmissionReviewRequirement, SelectedReviewRequestError> {
    let mut selected = None;
    for requirement in policy
        .review_requirements()
        .iter()
        .filter(|requirement| requirement.review_role_id() == review_role_id)
    {
        if selected.replace(requirement.clone()).is_some() {
            return Err(SelectedReviewRequestError::SelectorAmbiguous);
        }
    }
    selected.ok_or(SelectedReviewRequestError::SelectorUnavailable)
}

fn selected_freeze_policy_is_supported(
    policy: &MinimalPolicyRecord,
    resolver: &impl ExactRecordByteResolver,
) -> bool {
    policy.declares_freeze_commit_support()
        && resolver
            .resolve(policy.gate_scope_ref())
            .and_then(|bytes| ScopeRecord::decode_authoritative(bytes).ok())
            .is_some_and(|scope| {
                scope.record_id() == policy.gate_scope_ref()
                    && scope.input().scope_profile_id == 2
                    && scope.input().scope_profile_version == 1
                    && scope.input().scope_payload.is_empty()
                    && scope.input().scope_label.is_none()
            })
}

fn validate_selected_embedded_payload(
    store_root: &Path,
    attempt: FreezeAttemptId,
    manifest: &ManifestRecord,
) -> Result<(), SelectedEmbeddedFreezePreparationError> {
    if manifest.input().path_identity_profile_id != 1 || manifest.input().digest_profile_id != 1 {
        return Err(SelectedEmbeddedFreezePreparationError::SourceInvalid);
    }
    let root = store_root
        .join("roots")
        .join(selected_attempt_directory_name(attempt));
    validate_selected_exact_directory_entries(&root, &[("payload", true)])
        .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceInvalid)?;
    let payload = root.join("payload");
    let retained = collect_selected_embedded_source(&payload)?;
    if retained.artifacts != manifest.input().artifacts
        || u64::try_from(retained.artifacts.len())
            .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceResourceLimit)?
            != manifest.input().artifact_count
        || derive_selected_regular_file_subject(&retained.artifacts)
            .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceInvalid)?
            != manifest.input().subject_id
    {
        return Err(SelectedEmbeddedFreezePreparationError::SourceInvalid);
    }
    Ok(())
}

fn validate_selected_freeze_committed_prerequisites(
    store_root: &Path,
    retained_journal: &RetainedJournal,
    resolver: &impl ExactRecordByteResolver,
    committed: &ResolvedJournalReference,
    receipt: &FreezeReceiptRecord,
) -> Result<(), SelectedEmbeddedFreezePreparationError> {
    let invalid = || SelectedEmbeddedFreezePreparationError::SourceInvalid;
    if committed.event_type_id().value() != 101
        || receipt.input().freeze_id != *receipt.input().freeze_attempt_id.as_bytes()
        || receipt.input().custody_mode_id != 1
        || receipt.input().path_identity_profile_id != 1
        || receipt.input().requested_commit_durability_ref.is_some()
        || receipt.input().platform_strongest_available
        || receipt.input().file_content_flush_state != 1
        || receipt.input().atomic_publish_no_replace_state != 1
        || receipt.input().parent_directory_flush_state != 1
        || receipt.input().filesystem_profile_ref != committed.environment_observation_id()
    {
        return Err(invalid());
    }
    let creation_profile = resolver
        .resolve(receipt.input().creation_profile_ref)
        .ok_or_else(invalid)?;
    let creation_profile = FreezeCreationProfileRecord::decode_authoritative(creation_profile)
        .map_err(|_| invalid())?;
    if creation_profile.record_id() != receipt.input().creation_profile_ref {
        return Err(invalid());
    }
    let environment = resolver
        .resolve(receipt.input().filesystem_profile_ref)
        .ok_or_else(invalid)?;
    let environment =
        EnvironmentObservationRecord::decode_authoritative(environment).map_err(|_| invalid())?;
    if environment.record_id() != receipt.input().filesystem_profile_ref {
        return Err(invalid());
    }
    let capability = resolver
        .resolve(committed.storage_capability_class_id())
        .ok_or_else(invalid)?;
    let capability =
        StorageCapabilityClassRecord::decode_authoritative(capability).map_err(|_| invalid())?;
    if capability.record_id() != committed.storage_capability_class_id() {
        return Err(invalid());
    }
    let policy = resolver
        .resolve(receipt.input().policy_record_id)
        .and_then(|bytes| MinimalPolicyRecord::decode_authoritative(bytes).ok())
        .filter(|policy| {
            policy.record_id() == receipt.input().policy_record_id
                && policy.declares_freeze_commit_support()
        })
        .ok_or_else(invalid)?;
    if !selected_freeze_policy_is_supported(&policy, resolver) {
        return Err(invalid());
    }
    let start = retained_journal
        .resolve_reference(&receipt.input().attempt_start_journal_ref)
        .map_err(|_| invalid())?;
    if start.event_type_id().value() != 100
        || start.lifecycle_object_id() != *receipt.input().freeze_attempt_id.as_bytes()
        || derive_freeze_root(start.registry_id(), receipt.input().freeze_attempt_id)
            .intended_root_id()
            != start
                .freeze_attempt_intended_root_id()
                .ok_or_else(invalid)?
    {
        return Err(invalid());
    }
    let manifest = resolver
        .resolve(receipt.input().manifest_id)
        .and_then(|bytes| ManifestRecord::decode_authoritative(bytes).ok())
        .filter(|manifest| manifest.record_id() == receipt.input().manifest_id)
        .ok_or_else(invalid)?;
    if manifest.input().subject_id != receipt.input().subject_id
        || policy.record_id() != receipt.input().policy_record_id
    {
        return Err(invalid());
    }
    validate_selected_embedded_payload(store_root, receipt.input().freeze_attempt_id, &manifest)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectedEmbeddedSource {
    artifacts: Vec<ManifestArtifactEntry>,
    bytes: Vec<Vec<u8>>,
}

/// Bounds metadata retained while recursively traversing a selected payload, including empty
/// directories which do not contribute manifest artifact bytes.
struct SelectedEmbeddedTraversalBudget {
    entries: usize,
    path_bytes: usize,
    content_bytes: usize,
    retained_path_bytes: usize,
}

impl SelectedEmbeddedTraversalBudget {
    fn reserve_child(
        &mut self,
        name: &str,
        depth: usize,
    ) -> Result<(), SelectedEmbeddedFreezePreparationError> {
        self.entries = self
            .entries
            .checked_add(1)
            .filter(|entries| *entries <= AUTHORITATIVE_STORE_MAX_OBJECTS)
            .ok_or(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)?;
        self.path_bytes = self
            .path_bytes
            .checked_add(name.len())
            .filter(|bytes| *bytes <= AUTHORITATIVE_STORE_MAX_NAMESPACE_BYTES)
            .ok_or(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)?;
        if depth > AUTHORITATIVE_STORE_MAX_OBJECTS {
            return Err(SelectedEmbeddedFreezePreparationError::SourceResourceLimit);
        }
        Ok(())
    }

    fn reserve_content(
        &mut self,
        length: u64,
    ) -> Result<usize, SelectedEmbeddedFreezePreparationError> {
        let length = usize::try_from(length)
            .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceResourceLimit)?;
        self.content_bytes = self
            .content_bytes
            .checked_add(length)
            .filter(|bytes| *bytes <= AUTHORITATIVE_STORE_MAX_NAMESPACE_BYTES)
            .ok_or(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)?;
        Ok(length)
    }

    fn reserve_retained_artifact_path(
        &mut self,
        path: &[Vec<u8>],
    ) -> Result<(), SelectedEmbeddedFreezePreparationError> {
        let component_bytes = path.iter().try_fold(0_usize, |total, component| {
            total
                .checked_add(component.len())
                .ok_or(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)
        })?;
        let component_vector_bytes = path
            .len()
            .checked_mul(std::mem::size_of::<Vec<u8>>())
            .ok_or(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)?;
        let retained_bytes = component_bytes
            .checked_add(component_vector_bytes)
            .ok_or(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)?;
        self.retained_path_bytes = self
            .retained_path_bytes
            .checked_add(retained_bytes)
            .filter(|bytes| *bytes <= AUTHORITATIVE_STORE_MAX_NAMESPACE_BYTES)
            .ok_or(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)?;
        Ok(())
    }
}

fn collect_selected_embedded_source(
    root: &Path,
) -> Result<SelectedEmbeddedSource, SelectedEmbeddedFreezePreparationError> {
    let root_metadata = fs::symlink_metadata(root)
        .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceInvalid)?;
    if !root_metadata.is_dir() || metadata_is_reparse(&root_metadata) {
        return Err(SelectedEmbeddedFreezePreparationError::SourceInvalid);
    }
    let mut entries = Vec::new();
    let mut traversal_budget = SelectedEmbeddedTraversalBudget {
        entries: 0,
        path_bytes: 0,
        content_bytes: 0,
        retained_path_bytes: 0,
    };
    collect_selected_embedded_source_descendants(
        root,
        &mut Vec::new(),
        &mut entries,
        &mut traversal_budget,
    )?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let mut artifacts = Vec::with_capacity(entries.len());
    let mut bytes = Vec::with_capacity(entries.len());
    let mut total_bytes = 0_usize;
    for (path_components, file_bytes) in entries {
        total_bytes = total_bytes
            .checked_add(file_bytes.len())
            .filter(|total| *total <= AUTHORITATIVE_STORE_MAX_NAMESPACE_BYTES)
            .ok_or(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)?;
        artifacts.push(ManifestArtifactEntry {
            artifact_kind_id: 1,
            path_components,
            size_bytes: u64::try_from(file_bytes.len())
                .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceResourceLimit)?,
            digest_algorithm_id: 1,
            digest_bytes: Sha256::digest(&file_bytes).into(),
        });
        bytes.push(file_bytes);
    }
    Ok(SelectedEmbeddedSource { artifacts, bytes })
}

fn collect_selected_embedded_source_descendants(
    directory: &Path,
    prefix: &mut Vec<Vec<u8>>,
    output: &mut Vec<(Vec<Vec<u8>>, Vec<u8>)>,
    traversal_budget: &mut SelectedEmbeddedTraversalBudget,
) -> Result<(), SelectedEmbeddedFreezePreparationError> {
    let mut children = Vec::new();
    for entry in fs::read_dir(directory)
        .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceInvalid)?
    {
        let entry = entry.map_err(|_| SelectedEmbeddedFreezePreparationError::SourceInvalid)?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceInvalid)?;
        if name.is_empty() || matches!(name.as_str(), "." | "..") {
            return Err(SelectedEmbeddedFreezePreparationError::SourceInvalid);
        }
        traversal_budget.reserve_child(&name, prefix.len().saturating_add(1))?;
        children.push((name, entry.path()));
    }
    children.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    for (name, path) in children {
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceInvalid)?;
        if metadata_is_reparse(&metadata) {
            return Err(SelectedEmbeddedFreezePreparationError::SourceInvalid);
        }
        prefix.push(name.into_bytes());
        if metadata.is_dir() {
            collect_selected_embedded_source_descendants(&path, prefix, output, traversal_budget)?;
        } else if metadata.is_file() {
            if output.len() >= AUTHORITATIVE_STORE_MAX_OBJECTS
                || metadata.len() > AUTHORITATIVE_STORE_MAX_OBJECT_BYTES as u64
            {
                return Err(SelectedEmbeddedFreezePreparationError::SourceResourceLimit);
            }
            traversal_budget.reserve_retained_artifact_path(prefix)?;
            output
                .try_reserve(1)
                .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceResourceLimit)?;
            let mut file = open_identity_handle_for_regular_file(&path)
                .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceInvalid)?;
            let opened_metadata = file
                .metadata()
                .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceInvalid)?;
            if !opened_metadata.is_file()
                || metadata_is_reparse(&opened_metadata)
                || metadata_link_count(&opened_metadata, &file) != Some(1)
                || opened_metadata.len() != metadata.len()
            {
                return Err(SelectedEmbeddedFreezePreparationError::SourceInvalid);
            }
            ensure_path_matches_handle(&path, &file)
                .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceInvalid)?;
            let expected_length = traversal_budget.reserve_content(opened_metadata.len())?;
            let maximum_read_length = u64::try_from(expected_length)
                .ok()
                .and_then(|length| length.checked_add(1))
                .ok_or(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)?;
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(expected_length.saturating_add(1))
                .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceResourceLimit)?;
            Read::by_ref(&mut file)
                .take(maximum_read_length)
                .read_to_end(&mut bytes)
                .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceInvalid)?;
            if bytes.len() != opened_metadata.len() as usize
                || file
                    .metadata()
                    .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceInvalid)?
                    .len()
                    != opened_metadata.len()
            {
                return Err(SelectedEmbeddedFreezePreparationError::SourceInvalid);
            }
            ensure_path_matches_handle(&path, &file)
                .map_err(|_| SelectedEmbeddedFreezePreparationError::SourceInvalid)?;
            output.push((prefix.clone(), bytes));
        } else {
            return Err(SelectedEmbeddedFreezePreparationError::SourceInvalid);
        }
        prefix.pop();
    }
    Ok(())
}

fn selected_attempt_directory_name(attempt: FreezeAttemptId) -> String {
    attempt
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn create_selected_embedded_payload_root(
    store: &AuthoritativeRegistryStore,
    attempt: FreezeAttemptId,
    start_reference: &JournalReference,
) -> Result<(PathBuf, fs::File), SelectedEmbeddedFreezePreparationError> {
    if start_reference.registry_id() != store.retained_journal.registry_id
        || start_reference.entry_index().value() == 0
        || start_reference.event_type_id().value() != 100
    {
        return Err(SelectedEmbeddedFreezePreparationError::ReplayMismatch);
    }
    let roots_path = store.root.join("roots");
    let roots_hold = store
        .acquire_publication_directory_hold(4, "roots")
        .map_err(|_| SelectedEmbeddedFreezePreparationError::RetainedGenerationChanged)?;
    let roots_contents = namespace_contents_path(&roots_path, &roots_hold)
        .map_err(|_| SelectedEmbeddedFreezePreparationError::RetainedGenerationChanged)?;
    let attempt_name = selected_attempt_directory_name(attempt);
    match fs::create_dir(roots_contents.join(&attempt_name)) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(SelectedEmbeddedFreezePreparationError::RootConflict)
        }
        Err(_) => return Err(SelectedEmbeddedFreezePreparationError::RootCreation),
    }
    let root_hold = open_child_directory_hold(&roots_hold, &roots_contents, &attempt_name)
        .map_err(|_| SelectedEmbeddedFreezePreparationError::RootCreation)?;
    let root_contents = namespace_contents_path(&roots_contents.join(&attempt_name), &root_hold)
        .map_err(|_| SelectedEmbeddedFreezePreparationError::RootCreation)?;
    let root_publication_hold = open_publication_child_directory_hold(
        &roots_hold,
        &roots_contents,
        &root_hold,
        &attempt_name,
    )
    .map_err(|_| SelectedEmbeddedFreezePreparationError::RootCreation)?;
    fs::create_dir(root_contents.join("payload"))
        .map_err(|_| SelectedEmbeddedFreezePreparationError::RootCreation)?;
    let payload_hold = open_child_directory_hold(&root_hold, &root_contents, "payload")
        .map_err(|_| SelectedEmbeddedFreezePreparationError::RootCreation)?;
    let payload_publication_hold = open_publication_child_directory_hold(
        &root_publication_hold,
        &root_contents,
        &payload_hold,
        "payload",
    )
    .map_err(|_| SelectedEmbeddedFreezePreparationError::RootCreation)?;
    sync_retained_directory(&root_contents, &root_publication_hold)
        .map_err(|_| SelectedEmbeddedFreezePreparationError::RootCreation)?;
    sync_retained_directory(&roots_contents, &roots_hold)
        .map_err(|_| SelectedEmbeddedFreezePreparationError::RootCreation)?;
    sync_retained_directory(&root_contents.join("payload"), &payload_publication_hold)
        .map_err(|_| SelectedEmbeddedFreezePreparationError::RootCreation)?;
    Ok((
        store.root.join("roots").join(attempt_name).join("payload"),
        payload_publication_hold,
    ))
}

fn publish_selected_embedded_payload(
    payload_directory: &Path,
    payload_hold: &fs::File,
    source: &SelectedEmbeddedSource,
) -> Result<(), SelectedEmbeddedFreezePreparationError> {
    let mut directories = BTreeSet::new();
    for artifact in &source.artifacts {
        for length in 1..artifact.path_components.len() {
            directories.insert(artifact.path_components[..length].to_vec());
        }
    }
    for directory in directories {
        let mut parent_path = namespace_contents_path(payload_directory, payload_hold)
            .map_err(|_| SelectedEmbeddedFreezePreparationError::PayloadPublication)?;
        let mut parent_hold = payload_hold
            .try_clone()
            .map_err(|_| SelectedEmbeddedFreezePreparationError::PayloadPublication)?;
        for (index, component) in directory.iter().enumerate() {
            let name = std::str::from_utf8(component)
                .map_err(|_| SelectedEmbeddedFreezePreparationError::PayloadPublication)?;
            let child_path = parent_path.join(name);
            if index + 1 == directory.len() {
                fs::create_dir(&child_path)
                    .map_err(|_| SelectedEmbeddedFreezePreparationError::PayloadPublication)?;
            }
            let child_hold = open_child_directory_hold(&parent_hold, &parent_path, name)
                .map_err(|_| SelectedEmbeddedFreezePreparationError::PayloadPublication)?;
            parent_path = namespace_contents_path(&child_path, &child_hold)
                .map_err(|_| SelectedEmbeddedFreezePreparationError::PayloadPublication)?;
            parent_hold = child_hold;
        }
        sync_retained_directory(&parent_path, &parent_hold)
            .map_err(|_| SelectedEmbeddedFreezePreparationError::PayloadPublication)?;
    }
    for (artifact, bytes) in source.artifacts.iter().zip(&source.bytes) {
        let mut parent_path = namespace_contents_path(payload_directory, payload_hold)
            .map_err(|_| SelectedEmbeddedFreezePreparationError::PayloadPublication)?;
        let mut parent_hold = payload_hold
            .try_clone()
            .map_err(|_| SelectedEmbeddedFreezePreparationError::PayloadPublication)?;
        for component in &artifact.path_components[..artifact.path_components.len() - 1] {
            let name = std::str::from_utf8(component)
                .map_err(|_| SelectedEmbeddedFreezePreparationError::PayloadPublication)?;
            let child_path = parent_path.join(name);
            let child_hold = open_child_directory_hold(&parent_hold, &parent_path, name)
                .map_err(|_| SelectedEmbeddedFreezePreparationError::PayloadPublication)?;
            parent_path = namespace_contents_path(&child_path, &child_hold)
                .map_err(|_| SelectedEmbeddedFreezePreparationError::PayloadPublication)?;
            parent_hold = child_hold;
        }
        let name = std::str::from_utf8(
            artifact
                .path_components
                .last()
                .ok_or(SelectedEmbeddedFreezePreparationError::PayloadPublication)?,
        )
        .map_err(|_| SelectedEmbeddedFreezePreparationError::PayloadPublication)?;
        match publish_new_immutable_file(&parent_path, &parent_hold, Path::new(name), bytes)
            .map_err(|_| SelectedEmbeddedFreezePreparationError::PayloadPublication)?
        {
            ImmutablePublicationOutcome::Published(_) => {}
            ImmutablePublicationOutcome::Conflict
            | ImmutablePublicationOutcome::VisibleReceiptUncertain => {
                return Err(SelectedEmbeddedFreezePreparationError::PayloadPublication)
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct ImmutablePublicationFacts {
    content_flush: DurabilityActionState,
    atomic_no_replace: DurabilityActionState,
    parent_directory_flush: DurabilityActionState,
}

struct ImmutablePublication {
    facts: ImmutablePublicationFacts,
    retained_witness: RetainedFileWitness,
}

enum JournalSlotPublication {
    Published(ImmutablePublication),
    Conflict,
    VisibleReceiptUncertain,
}

enum ImmutablePublicationOutcome {
    Published(ImmutablePublication),
    Conflict,
    VisibleReceiptUncertain,
}

fn map_reload_to_runtime_error(
    error: AuthoritativeReviewAdmissionAcceptanceError,
) -> AuthoritativeReviewAdmissionRuntimeError {
    AuthoritativeReviewAdmissionRuntimeError::Publication(match error {
        AuthoritativeReviewAdmissionAcceptanceError::Store(error) => {
            AuthoritativeReviewAdmissionPublicationError::Store(error)
        }
        AuthoritativeReviewAdmissionAcceptanceError::RegistryIdentityChanged => {
            AuthoritativeReviewAdmissionPublicationError::RegistryIdentityChanged
        }
        AuthoritativeReviewAdmissionAcceptanceError::Input(_) => {
            unreachable!("namespace reload does not inspect opaque input allocation")
        }
    })
}

fn publish_record_bytes(
    store: &mut AuthoritativeRegistryStore,
    record_id: RecordId,
    bytes: &[u8],
) -> Result<ImmutablePublication, ()> {
    for (candidate_id, candidate_bytes) in &store.records {
        if *candidate_id == record_id {
            continue;
        }
        let frame = StrictRecordFrame::decode_authoritative(candidate_bytes).map_err(|_| ())?;
        if frame.record_type_id().value() == 32
            && !store
                .retained_journal
                .entries
                .iter()
                .any(|entry| entry.event_record_id().as_bytes() == candidate_id.as_bytes())
        {
            return Err(());
        }
    }
    let records_dir = store.root.join("records");
    let records_hold = store.acquire_publication_directory_hold(3, "records")?;
    ensure_path_matches_handle(&records_dir, &records_hold)?;
    let records_contents = namespace_contents_path(&records_dir, &records_hold)?;
    let final_name = record_filename(record_id);
    let final_path = records_contents.join(&final_name);
    if final_path.exists() {
        let mut guard = if let Some(position) = store
            .retained_file_witnesses
            .iter()
            .position(|witness| witness.path == final_path)
        {
            open_retained_file_guard(&store.retained_file_witnesses[position])?
        } else {
            let read =
                read_regular_file(&final_path, &mut NamespaceBudget::new()).map_err(|_| ())?;
            if read.bytes != bytes {
                return Err(());
            }
            store.retained_file_witnesses.push(read.witness);
            open_retained_file_guard(store.retained_file_witnesses.last().ok_or(())?)?
        };
        revalidate_retained_file_witness(&mut guard)?;
        if guard.expected_length != bytes.len()
            || guard.expected_sha256 != <[u8; ID_LENGTH]>::from(Sha256::digest(bytes))
        {
            return Err(());
        }
        // A same-byte reuse is still an operation-local durable-publication
        // dependency. Flush the exact retained handle before its Journal event
        // can become visible; a previous operation's flush is not reused as
        // this operation's receipt fact.
        guard.file.sync_all().map_err(|_| ())?;
        return Ok(ImmutablePublication {
            facts: ImmutablePublicationFacts {
                content_flush: DurabilityActionState::Performed,
                atomic_no_replace: DurabilityActionState::PreviouslyEstablished,
                parent_directory_flush: sync_retained_directory(&records_dir, &records_hold)?,
            },
            retained_witness: guard,
        });
    }
    match publish_new_immutable_file(&records_dir, &records_hold, Path::new(&final_name), bytes)? {
        ImmutablePublicationOutcome::Published(publication) => Ok(publication),
        ImmutablePublicationOutcome::Conflict
        | ImmutablePublicationOutcome::VisibleReceiptUncertain => Err(()),
    }
}

fn publish_journal_slot(
    journal_dir: &Path,
    journal_hold: &fs::File,
    final_name: &Path,
    bytes: &[u8],
) -> Result<JournalSlotPublication, ()> {
    match publish_new_immutable_file(journal_dir, journal_hold, final_name, bytes)? {
        ImmutablePublicationOutcome::Published(publication) => {
            Ok(JournalSlotPublication::Published(publication))
        }
        ImmutablePublicationOutcome::Conflict => Ok(JournalSlotPublication::Conflict),
        ImmutablePublicationOutcome::VisibleReceiptUncertain => {
            Ok(JournalSlotPublication::VisibleReceiptUncertain)
        }
    }
}

fn publish_new_immutable_file(
    parent: &Path,
    parent_hold: &fs::File,
    final_name: &Path,
    bytes: &[u8],
) -> Result<ImmutablePublicationOutcome, ()> {
    publish_new_immutable_file_with_hook(parent, parent_hold, final_name, bytes, || {})
}

fn publish_new_immutable_file_with_hook<F>(
    parent: &Path,
    parent_hold: &fs::File,
    final_name: &Path,
    bytes: &[u8],
    before_promotion: F,
) -> Result<ImmutablePublicationOutcome, ()>
where
    F: FnOnce(),
{
    publish_new_immutable_file_with_hooks(
        parent,
        parent_hold,
        final_name,
        bytes,
        before_promotion,
        || Ok(()),
    )
}

fn publish_selected_initial_file(
    parent: &Path,
    parent_hold: &fs::File,
    final_name: &Path,
    bytes: &[u8],
) -> Result<(), AuthoritativeRegistryStoreInitializeError> {
    match publish_new_immutable_file(parent, parent_hold, final_name, bytes) {
        Ok(ImmutablePublicationOutcome::Published(_)) => Ok(()),
        Ok(
            ImmutablePublicationOutcome::Conflict
            | ImmutablePublicationOutcome::VisibleReceiptUncertain,
        )
        | Err(()) => Err(AuthoritativeRegistryStoreInitializeError::Publication),
    }
}

fn publish_new_immutable_file_with_hooks<F, G>(
    parent: &Path,
    parent_hold: &fs::File,
    final_name: &Path,
    bytes: &[u8],
    before_promotion: F,
    after_visibility: G,
) -> Result<ImmutablePublicationOutcome, ()>
where
    F: FnOnce(),
    G: FnOnce() -> Result<(), ()>,
{
    if final_name.parent() != Some(Path::new("")) || final_name.file_name().is_none() {
        return Err(());
    }
    ensure_path_matches_handle(parent, parent_hold)?;

    #[cfg(windows)]
    {
        let final_path = parent.join(final_name);
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| ())?;
        let temp_path = parent.join(format!(
            ".evidence-registry-publish-{}-{sequence}.tmp",
            std::process::id()
        ));
        let mut file = create_owned_publication_temp(&temp_path)?;
        let write_result = file
            .write_all(bytes)
            .and_then(|()| file.flush())
            .and_then(|()| file.sync_all());
        if write_result.is_err() {
            cleanup_owned_publication_temp_after_failure(file, &temp_path, parent, parent_hold)?;
            return Err(());
        }
        before_promotion();
        ensure_path_matches_handle(parent, parent_hold)?;
        let published = match promote_held_file_no_replace(&file, parent_hold, &final_path) {
            Ok(HeldFilePromotion::Published) => true,
            Ok(HeldFilePromotion::Conflict) => false,
            Ok(HeldFilePromotion::VisibilityUncertain) => {
                return Ok(ImmutablePublicationOutcome::VisibleReceiptUncertain);
            }
            Err(_) => {
                if temp_path.exists() {
                    cleanup_owned_publication_temp_after_failure(
                        file,
                        &temp_path,
                        parent,
                        parent_hold,
                    )?;
                } else {
                    drop(file);
                }
                return Err(());
            }
        };
        if !published {
            if !temp_path.exists() {
                drop(file);
                return Err(());
            }
            remove_owned_publication_temp(file, &temp_path)?;
            sync_retained_directory(parent, parent_hold)?;
            return Ok(ImmutablePublicationOutcome::Conflict);
        }
        if after_visibility().is_err() || file.sync_all().is_err() || temp_path.exists() {
            return Ok(ImmutablePublicationOutcome::VisibleReceiptUncertain);
        }
        let parent_directory_flush = match sync_retained_directory(parent, parent_hold) {
            Ok(state) => state,
            Err(()) => return Ok(ImmutablePublicationOutcome::VisibleReceiptUncertain),
        };
        Ok(ImmutablePublicationOutcome::Published(
            ImmutablePublication {
                facts: ImmutablePublicationFacts {
                    content_flush: DurabilityActionState::Performed,
                    atomic_no_replace: DurabilityActionState::Performed,
                    parent_directory_flush,
                },
                retained_witness: RetainedFileWitness {
                    path: final_path,
                    file,
                    expected_length: bytes.len(),
                    expected_sha256: Sha256::digest(bytes).into(),
                },
            },
        ))
    }

    #[cfg(target_os = "linux")]
    {
        use std::ffi::CString;
        use std::os::fd::{AsRawFd, FromRawFd};
        use std::os::unix::ffi::OsStrExt;

        unsafe extern "C" {
            fn openat(directory: i32, path: *const i8, flags: i32, ...) -> i32;
            fn linkat(
                old_directory: i32,
                old_path: *const i8,
                new_directory: i32,
                new_path: *const i8,
                flags: i32,
            ) -> i32;
        }

        const O_RDWR: i32 = 0x0000_0002;
        const O_CLOEXEC: i32 = 0x0008_0000;
        const O_TMPFILE: i32 = 0x0041_0000;
        const AT_EMPTY_PATH: i32 = 0x0000_1000;
        const AT_SYMLINK_FOLLOW: i32 = 0x0000_0400;
        const AT_FDCWD: i32 = -100;
        const EEXIST: i32 = 17;

        let dot = CString::new(".").expect("literal path has no NUL");
        let file_descriptor = unsafe {
            openat(
                parent_hold.as_raw_fd(),
                dot.as_ptr(),
                O_RDWR | O_CLOEXEC | O_TMPFILE,
                0o600,
            )
        };
        if file_descriptor < 0 {
            return Err(());
        }
        let mut file = unsafe { fs::File::from_raw_fd(file_descriptor) };
        file.write_all(bytes).map_err(|_| ())?;
        file.flush().map_err(|_| ())?;
        file.sync_all().map_err(|_| ())?;
        before_promotion();
        ensure_path_matches_handle(parent, parent_hold)?;

        let final_path = parent.join(final_name);
        let final_name_c = CString::new(final_name.as_os_str().as_bytes()).map_err(|_| ())?;
        let empty = CString::new("").expect("empty path has no NUL");
        let mut linked = unsafe {
            linkat(
                file.as_raw_fd(),
                empty.as_ptr(),
                parent_hold.as_raw_fd(),
                final_name_c.as_ptr(),
                AT_EMPTY_PATH,
            )
        };
        if linked != 0 {
            let source =
                CString::new(format!("/proc/self/fd/{}", file.as_raw_fd())).map_err(|_| ())?;
            linked = unsafe {
                linkat(
                    AT_FDCWD,
                    source.as_ptr(),
                    parent_hold.as_raw_fd(),
                    final_name_c.as_ptr(),
                    AT_SYMLINK_FOLLOW,
                )
            };
        }
        let published = if linked == 0 {
            true
        } else if std::io::Error::last_os_error().raw_os_error() == Some(EEXIST) {
            false
        } else {
            return Err(());
        };
        if !published {
            return Ok(ImmutablePublicationOutcome::Conflict);
        }
        if after_visibility().is_err() {
            return Ok(ImmutablePublicationOutcome::VisibleReceiptUncertain);
        }
        let parent_directory_flush = match sync_retained_directory(parent, parent_hold) {
            Ok(state) => state,
            Err(()) => return Ok(ImmutablePublicationOutcome::VisibleReceiptUncertain),
        };
        Ok(ImmutablePublicationOutcome::Published(
            ImmutablePublication {
                facts: ImmutablePublicationFacts {
                    content_flush: DurabilityActionState::Performed,
                    atomic_no_replace: DurabilityActionState::Performed,
                    parent_directory_flush,
                },
                retained_witness: RetainedFileWitness {
                    path: final_path,
                    file,
                    expected_length: bytes.len(),
                    expected_sha256: Sha256::digest(bytes).into(),
                },
            },
        ))
    }

    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;
        use std::os::fd::{AsRawFd, FromRawFd};

        unsafe extern "C" {
            fn openat(directory: i32, path: *const i8, flags: i32, ...) -> i32;
            fn renameatx_np(
                from: i32,
                from_name: *const i8,
                to: i32,
                to_name: *const i8,
                flags: u32,
            ) -> i32;
            fn unlinkat(directory: i32, path: *const i8, flags: i32) -> i32;
        }

        const O_RDWR: i32 = 2;
        const O_CREAT: i32 = 0x0000_0200;
        const O_EXCL: i32 = 0x0000_0800;
        const O_NOFOLLOW: i32 = 0x0000_0100;
        const O_CLOEXEC: i32 = 0x0100_0000;
        const RENAME_EXCL: u32 = 0x0000_0004;
        const EEXIST: i32 = 17;

        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| ())?;
        let temp_name = format!(
            ".evidence-registry-publish-{}-{sequence}.tmp",
            std::process::id()
        );
        let temp_name_c = CString::new(temp_name.as_bytes()).map_err(|_| ())?;
        let final_name_c =
            CString::new(final_name.as_os_str().as_encoded_bytes()).map_err(|_| ())?;
        let descriptor = unsafe {
            openat(
                parent_hold.as_raw_fd(),
                temp_name_c.as_ptr(),
                O_RDWR | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC,
                0o600,
            )
        };
        if descriptor < 0 {
            return Err(());
        }
        let mut file = unsafe { fs::File::from_raw_fd(descriptor) };
        let temp_path = parent.join(&temp_name);
        let cleanup = |file: &fs::File| -> Result<(), ()> {
            ensure_path_matches_handle(&temp_path, file)?;
            if unsafe { unlinkat(parent_hold.as_raw_fd(), temp_name_c.as_ptr(), 0) } != 0 {
                return Err(());
            }
            Ok(())
        };
        if file
            .write_all(bytes)
            .and_then(|()| file.flush())
            .and_then(|()| file.sync_all())
            .is_err()
        {
            cleanup(&file)?;
            return Err(());
        }
        before_promotion();
        ensure_path_matches_handle(parent, parent_hold)?;
        ensure_path_matches_handle(&temp_path, &file)?;
        let renamed = unsafe {
            renameatx_np(
                parent_hold.as_raw_fd(),
                temp_name_c.as_ptr(),
                parent_hold.as_raw_fd(),
                final_name_c.as_ptr(),
                RENAME_EXCL,
            )
        };
        if renamed != 0 {
            if std::io::Error::last_os_error().raw_os_error() == Some(EEXIST) {
                cleanup(&file)?;
                return Ok(ImmutablePublicationOutcome::Conflict);
            }
            cleanup(&file)?;
            return Err(());
        }
        let final_path = parent.join(final_name);
        if ensure_path_matches_handle(&final_path, &file).is_err()
            || after_visibility().is_err()
            || file.sync_all().is_err()
        {
            return Ok(ImmutablePublicationOutcome::VisibleReceiptUncertain);
        }
        let parent_directory_flush = match sync_retained_directory(parent, parent_hold) {
            Ok(state) => state,
            Err(()) => return Ok(ImmutablePublicationOutcome::VisibleReceiptUncertain),
        };
        let mut witness = RetainedFileWitness {
            path: final_path,
            file,
            expected_length: bytes.len(),
            expected_sha256: Sha256::digest(bytes).into(),
        };
        if revalidate_retained_file_witness(&mut witness).is_err() {
            return Ok(ImmutablePublicationOutcome::VisibleReceiptUncertain);
        }
        Ok(ImmutablePublicationOutcome::Published(
            ImmutablePublication {
                facts: ImmutablePublicationFacts {
                    content_flush: DurabilityActionState::Performed,
                    atomic_no_replace: DurabilityActionState::Performed,
                    parent_directory_flush,
                },
                retained_witness: witness,
            },
        ))
    }

    #[cfg(all(not(windows), not(target_os = "linux"), not(target_os = "macos")))]
    {
        let _ = (bytes, before_promotion, after_visibility);
        Err(())
    }
}

#[cfg(windows)]
fn cleanup_owned_publication_temp_after_failure(
    file: fs::File,
    temp_path: &Path,
    parent_path: &Path,
    parent_hold: &fs::File,
) -> Result<(), ()> {
    remove_owned_publication_temp(file, temp_path)?;
    sync_retained_directory(parent_path, parent_hold)?;
    Ok(())
}

#[cfg(windows)]
fn remove_owned_publication_temp(file: fs::File, path: &Path) -> Result<(), ()> {
    use std::os::windows::io::AsRawHandle;
    #[repr(C)]
    struct FileDispositionInfo {
        delete_file: u8,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetFileInformationByHandle(
            file: *mut core::ffi::c_void,
            information_class: u32,
            information: *const core::ffi::c_void,
            information_size: u32,
        ) -> i32;
    }
    let disposition = FileDispositionInfo { delete_file: 1 };
    let result = unsafe {
        SetFileInformationByHandle(
            file.as_raw_handle(),
            4,
            (&raw const disposition).cast(),
            u32::try_from(std::mem::size_of::<FileDispositionInfo>()).map_err(|_| ())?,
        )
    };
    if result == 0 {
        return Err(());
    }
    drop(file);
    if path.exists() {
        return Err(());
    }
    Ok(())
}

#[cfg(windows)]
fn create_owned_publication_temp(path: &Path) -> Result<fs::File, ()> {
    use std::os::windows::fs::OpenOptionsExt;
    const DELETE: u32 = 0x0001_0000;
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const GENERIC_READ: u32 = 0x8000_0000;
    const GENERIC_WRITE: u32 = 0x4000_0000;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .access_mode(GENERIC_READ | GENERIC_WRITE | DELETE)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|_| ())
}

#[cfg(windows)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeldFilePromotion {
    Published,
    Conflict,
    VisibilityUncertain,
}

#[cfg(windows)]
fn promote_held_file_no_replace(
    file: &fs::File,
    parent_handle: &fs::File,
    final_path: &Path,
) -> Result<HeldFilePromotion, u32> {
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::AsRawHandle;

    #[repr(C)]
    struct IoStatusBlock {
        status: isize,
        information: usize,
    }

    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtSetInformationFile(
            file: *mut core::ffi::c_void,
            io_status: *mut IoStatusBlock,
            information: *const core::ffi::c_void,
            information_size: u32,
            information_class: u32,
        ) -> i32;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn WaitForSingleObject(handle: *mut core::ffi::c_void, milliseconds: u32) -> u32;
    }

    const FILE_RENAME_INFORMATION_CLASS: u32 = 10;
    let layout = file_rename_information_layout(std::mem::size_of::<usize>()).ok_or(87_u32)?;
    let name: Vec<u16> = final_path
        .file_name()
        .ok_or(87_u32)?
        .encode_wide()
        .collect();
    let name_bytes = name.len().checked_mul(2).ok_or(87_u32)?;
    let information_bytes = layout
        .file_name_offset
        .checked_add(name_bytes)
        .ok_or(87_u32)?;
    let allocation_bytes = layout
        .allocation_header_size
        .checked_add(name_bytes)
        .ok_or(87_u32)?;
    let words = allocation_bytes
        .checked_add(layout.alignment - 1)
        .ok_or(87_u32)?
        / layout.alignment;
    let mut storage = vec![0_usize; words];
    let bytes = unsafe {
        std::slice::from_raw_parts_mut(storage.as_mut_ptr().cast::<u8>(), words * layout.alignment)
    };
    bytes[0] = 0;
    bytes
        [layout.root_directory_offset..layout.root_directory_offset + std::mem::size_of::<usize>()]
        .copy_from_slice(&(parent_handle.as_raw_handle() as usize).to_ne_bytes());
    bytes[layout.file_name_length_offset..layout.file_name_length_offset + 4]
        .copy_from_slice(&u32::try_from(name_bytes).map_err(|_| 87_u32)?.to_ne_bytes());
    for (index, unit) in name.iter().enumerate() {
        let offset = layout.file_name_offset + index * 2;
        bytes[offset..offset + 2].copy_from_slice(&unit.to_ne_bytes());
    }
    let information_size =
        u32::try_from(information_bytes.max(layout.allocation_header_size)).map_err(|_| 87_u32)?;
    let mut io_status = IoStatusBlock {
        status: 0,
        information: 0,
    };
    let mut status = unsafe {
        NtSetInformationFile(
            file.as_raw_handle(),
            &mut io_status,
            storage.as_ptr().cast(),
            information_size,
            FILE_RENAME_INFORMATION_CLASS,
        )
    };
    const STATUS_PENDING: i32 = 0x103;
    const STATUS_OBJECT_NAME_COLLISION: i32 = 0xc000_0035_u32 as i32;
    const INFINITE: u32 = 0xffff_ffff;
    if status == STATUS_PENDING {
        if unsafe { WaitForSingleObject(file.as_raw_handle(), INFINITE) } != 0 {
            return Ok(HeldFilePromotion::VisibilityUncertain);
        }
        status = io_status.status as i32;
    }
    if status >= 0 {
        if ensure_path_matches_handle(final_path, file).is_err() {
            return Ok(HeldFilePromotion::VisibilityUncertain);
        }
        return Ok(HeldFilePromotion::Published);
    }
    if status == STATUS_OBJECT_NAME_COLLISION {
        Ok(HeldFilePromotion::Conflict)
    } else {
        Err(u32::from_ne_bytes(status.to_ne_bytes()))
    }
}

fn sync_retained_directory(path: &Path, directory: &fs::File) -> Result<DurabilityActionState, ()> {
    ensure_path_matches_handle(path, directory)?;
    directory.sync_all().map_err(|_| ())?;
    ensure_path_matches_handle(path, directory)?;
    Ok(DurabilityActionState::Performed)
}

#[cfg(windows)]
fn acquire_selected_authoritative_publication_lock(
    root: &Path,
    root_hold: &fs::File,
) -> Result<fs::File, ()> {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;

    ensure_path_matches_handle(root, root_hold)?;
    let coordination = root.join("coordination");
    let coordination_hold = open_child_directory_hold(root_hold, root, "coordination")?;
    ensure_path_matches_handle(&coordination, &coordination_hold)?;
    let path = coordination.join("publication.lock");
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .share_mode(0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&path)
        .map_err(|_| ())?;
    let metadata = file.metadata().map_err(|_| ())?;
    if !metadata.is_file()
        || metadata_is_reparse(&metadata)
        || metadata_link_count(&metadata, &file) != Some(1)
    {
        return Err(());
    }
    ensure_path_matches_handle(root, root_hold)?;
    ensure_path_matches_handle(&coordination, &coordination_hold)?;
    Ok(file)
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn acquire_selected_authoritative_publication_lock(
    root: &Path,
    root_hold: &fs::File,
) -> Result<impl std::ops::Drop, ()> {
    acquire_authoritative_publication_lock(root, root_hold)
}

#[cfg(target_os = "macos")]
fn acquire_selected_authoritative_publication_lock(
    root: &Path,
    root_hold: &fs::File,
) -> Result<fs::File, ()> {
    acquire_authoritative_publication_lock(root, root_hold)
}

#[cfg(windows)]
fn acquire_authoritative_publication_lock(
    root: &Path,
    root_hold: &fs::File,
) -> Result<fs::File, ()> {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;

    ensure_path_matches_handle(root, root_hold)?;
    let path = root.join(".evidence-registry-publication.lock");
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .share_mode(0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&path)
        .map_err(|_| ())?;
    let metadata = file.metadata().map_err(|_| ())?;
    if !metadata.is_file()
        || metadata_is_reparse(&metadata)
        || metadata_link_count(&metadata, &file) != Some(1)
    {
        return Err(());
    }
    ensure_path_matches_handle(root, root_hold)?;
    Ok(file)
}

#[cfg(target_os = "linux")]
struct LinuxPublicationLock {
    file: fs::File,
    owner_pid: u32,
}
#[cfg(target_os = "linux")]
impl std::os::fd::AsRawFd for LinuxPublicationLock {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        std::os::fd::AsRawFd::as_raw_fd(&self.file)
    }
}
#[cfg(target_os = "linux")]
impl Drop for LinuxPublicationLock {
    fn drop(&mut self) {
        unsafe extern "C" {
            fn getpid() -> i32;
        }
        if unsafe { getpid() } == self.owner_pid as i32 && self.unlock().is_err() {
            eprintln!("linux publication lock unlock failed");
        }
    }
}
#[cfg(target_os = "linux")]
impl LinuxPublicationLock {
    fn unlock(&self) -> Result<(), ()> {
        unsafe extern "C" {
            fn flock(fd: i32, operation: i32) -> i32;
        }
        loop {
            if unsafe { flock(std::os::fd::AsRawFd::as_raw_fd(&self.file), 8) } == 0 {
                return Ok(());
            }
            if std::io::Error::last_os_error().raw_os_error() != Some(4) {
                return Err(());
            }
        }
    }
}
#[cfg(target_os = "linux")]
fn acquire_authoritative_publication_lock(
    root: &Path,
    root_hold: &fs::File,
) -> Result<LinuxPublicationLock, ()> {
    #[cfg(test)]
    {
        acquire_authoritative_publication_lock_impl(root, root_hold, None)
    }
    #[cfg(not(test))]
    {
        acquire_authoritative_publication_lock_impl(root, root_hold)
    }
}

#[cfg(all(test, target_os = "linux"))]
type PostLockHook<'a> = dyn FnMut(&fs::File) -> Result<(), ()> + 'a;
#[cfg(all(test, target_os = "linux"))]
fn acquire_authoritative_publication_lock_with_post_lock_hook<F>(
    root: &Path,
    root_hold: &fs::File,
    mut post_lock_hook: F,
) -> Result<LinuxPublicationLock, ()>
where
    F: FnMut(&fs::File) -> Result<(), ()>,
{
    acquire_authoritative_publication_lock_impl(root, root_hold, Some(&mut post_lock_hook))
}

#[cfg(target_os = "linux")]
fn acquire_authoritative_publication_lock_impl(
    root: &Path,
    root_hold: &fs::File,
    #[cfg(test)] mut post_lock_hook: Option<&mut PostLockHook<'_>>,
) -> Result<LinuxPublicationLock, ()> {
    use std::ffi::CString;
    use std::os::fd::{AsRawFd, FromRawFd};

    unsafe extern "C" {
        fn flock(fd: i32, operation: i32) -> i32;
        fn openat(dirfd: i32, pathname: *const std::ffi::c_char, flags: i32, ...) -> i32;
    }

    const O_DIRECTORY: i32 = 0x0001_0000;
    const O_NOFOLLOW: i32 = 0x0002_0000;
    const O_CLOEXEC: i32 = 0x0008_0000;
    const LOCK_EX: i32 = 2;
    const LOCK_NB: i32 = 4;
    const EINTR: i32 = 4;

    ensure_path_matches_handle(root, root_hold)?;
    let dot = CString::new(".").expect("literal path has no NUL");
    let fd = unsafe {
        openat(
            root_hold.as_raw_fd(),
            dot.as_ptr(),
            O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC,
            0,
        )
    };
    if fd < 0 {
        return Err(());
    }
    let file = unsafe { fs::File::from_raw_fd(fd) };
    let metadata = file.metadata().map_err(|_| ())?;
    if !metadata.is_dir()
        || metadata_is_reparse(&metadata)
        || !handles_identify_same_object(&file, root_hold)
    {
        return Err(());
    }
    loop {
        if unsafe { flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) } == 0 {
            break;
        }
        if std::io::Error::last_os_error().raw_os_error() != Some(EINTR) {
            return Err(());
        }
    }
    let lock = LinuxPublicationLock {
        file,
        owner_pid: std::process::id(),
    };
    #[cfg(test)]
    if let Some(hook) = post_lock_hook.as_mut() {
        hook(&lock.file)?;
    }
    ensure_path_matches_handle(root, root_hold)?;
    if !handles_identify_same_object(&lock.file, root_hold) {
        return Err(());
    }
    Ok(lock)
}

#[cfg(target_os = "macos")]
fn acquire_authoritative_publication_lock(
    root: &Path,
    root_hold: &fs::File,
) -> Result<fs::File, ()> {
    use std::ffi::CString;
    use std::os::fd::{AsRawFd, FromRawFd};

    unsafe extern "C" {
        fn flock(fd: i32, operation: i32) -> i32;
        fn openat(directory: i32, path: *const i8, flags: i32, ...) -> i32;
    }

    const LOCK_EX: i32 = 2;
    const LOCK_NB: i32 = 4;
    const O_RDONLY: i32 = 0;
    const O_CLOEXEC: i32 = 0x0100_0000;
    const O_DIRECTORY: i32 = 0x0010_0000;

    ensure_path_matches_handle(root, root_hold)?;
    let dot = CString::new(".").expect("literal path has no NUL");
    let descriptor = unsafe {
        openat(
            root_hold.as_raw_fd(),
            dot.as_ptr(),
            O_RDONLY | O_CLOEXEC | O_DIRECTORY,
            0,
        )
    };
    if descriptor < 0 {
        return Err(());
    }
    let file = unsafe { fs::File::from_raw_fd(descriptor) };
    let metadata = file.metadata().map_err(|_| ())?;
    if !metadata.is_dir()
        || metadata_is_reparse(&metadata)
        || unsafe { flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) } != 0
    {
        return Err(());
    }
    ensure_path_matches_handle(root, root_hold)?;
    Ok(file)
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn acquire_authoritative_publication_lock(
    _root: &Path,
    _root_hold: &fs::File,
) -> Result<fs::File, ()> {
    Err(())
}

#[cfg(all(test, any(windows, target_os = "linux")))]
fn open_publication_directory_hold(path: &Path) -> Result<fs::File, ()> {
    #[cfg(windows)]
    let directory = {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_SHARE_READ: u32 = 0x0000_0001;
        const FILE_SHARE_WRITE: u32 = 0x0000_0002;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
        OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)
            .map_err(|_| ())?
    };
    #[cfg(not(windows))]
    let directory = fs::File::open(path).map_err(|_| ())?;
    let metadata = directory.metadata().map_err(|_| ())?;
    if !metadata.is_dir() || metadata_is_reparse(&metadata) {
        return Err(());
    }
    ensure_path_matches_handle(path, &directory)?;
    Ok(directory)
}

#[cfg(any(windows, test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileRenameInformationLayout {
    root_directory_offset: usize,
    file_name_length_offset: usize,
    file_name_offset: usize,
    allocation_header_size: usize,
    alignment: usize,
}

#[cfg(any(windows, test))]
fn file_rename_information_layout(pointer_size: usize) -> Option<FileRenameInformationLayout> {
    if !matches!(pointer_size, 4 | 8) {
        return None;
    }
    let root_directory_offset = pointer_size;
    let file_name_length_offset = root_directory_offset.checked_add(pointer_size)?;
    let file_name_offset = file_name_length_offset.checked_add(4)?;
    let allocation_header_size = file_name_offset
        .checked_add(2)?
        .checked_add(pointer_size - 1)?
        / pointer_size
        * pointer_size;
    Some(FileRenameInformationLayout {
        root_directory_offset,
        file_name_length_offset,
        file_name_offset,
        allocation_header_size,
        alignment: pointer_size,
    })
}

fn authenticate_published_journal_reference(
    journal: &RetainedJournal,
    published: &JournalReference,
) -> Result<(), ()> {
    journal
        .resolve_reference(published)
        .map(|_| ())
        .map_err(|_| ())
}

fn admit_bounded_publication_temp_residue(
    path: &Path,
    budget: &mut NamespaceBudget,
) -> Result<(), AuthoritativeRegistryStoreOpenError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
    budget
        .reserve(metadata.len())
        .map_err(|error| match error {
            NamespaceReadError::ResourceLimit => {
                AuthoritativeRegistryStoreOpenError::NamespaceResourceLimit
            }
            NamespaceReadError::Io | NamespaceReadError::Invalid => {
                AuthoritativeRegistryStoreOpenError::Io
            }
        })?;
    if !metadata.is_file() || metadata_is_reparse(&metadata) {
        return Err(AuthoritativeRegistryStoreOpenError::Io);
    }
    Ok(())
}

fn is_publication_temp_name(name: &str) -> bool {
    let Some(body) = name
        .strip_prefix(".evidence-registry-publish-")
        .and_then(|name| name.strip_suffix(".tmp"))
    else {
        return false;
    };
    let Some((process, sequence)) = body.split_once('-') else {
        return false;
    };
    !process.is_empty()
        && !sequence.is_empty()
        && process.bytes().all(|byte| byte.is_ascii_digit())
        && sequence.bytes().all(|byte| byte.is_ascii_digit())
}

fn record_filename(record_id: RecordId) -> String {
    let mut name = String::with_capacity(ID_LENGTH * 2 + 5);
    for byte in record_id.as_bytes() {
        use std::fmt::Write as _;
        write!(&mut name, "{byte:02x}").expect("writing to String cannot fail");
    }
    name.push_str(".cbor");
    name
}

impl ExactRecordByteResolver for AuthoritativeRegistryStore {
    fn resolve(&self, record_id: RecordId) -> Option<&[u8]> {
        self.records
            .binary_search_by(|(stored_id, _)| stored_id.as_bytes().cmp(record_id.as_bytes()))
            .ok()
            .map(|index| self.records[index].1.as_slice())
    }
}

fn open_real_directory_hold(path: &Path) -> Result<fs::File, ()> {
    #[cfg(windows)]
    let file = {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_SHARE_READ: u32 = 0x0000_0001;
        const FILE_SHARE_WRITE: u32 = 0x0000_0002;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)
            .map_err(|_| ())?
    };
    #[cfg(target_os = "linux")]
    let file = {
        use std::os::unix::fs::OpenOptionsExt;
        const O_DIRECTORY: i32 = 0x0001_0000;
        const O_NOFOLLOW: i32 = 0x0002_0000;
        const O_CLOEXEC: i32 = 0x0008_0000;
        OpenOptions::new()
            .read(true)
            .custom_flags(O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC)
            .open(path)
            .map_err(|_| ())?
    };
    #[cfg(target_os = "macos")]
    let file = {
        use std::ffi::CString;
        use std::os::fd::FromRawFd;
        use std::os::unix::ffi::OsStrExt;

        unsafe extern "C" {
            fn open(path: *const i8, flags: i32, ...) -> i32;
        }

        const O_RDONLY: i32 = 0;
        const O_NONBLOCK: i32 = 0x0000_0004;
        const O_NOFOLLOW: i32 = 0x0000_0100;
        const O_CLOEXEC: i32 = 0x0100_0000;
        const O_DIRECTORY: i32 = 0x0010_0000;

        let path = CString::new(path.as_os_str().as_bytes()).map_err(|_| ())?;
        let descriptor = unsafe {
            open(
                path.as_ptr(),
                O_RDONLY | O_NONBLOCK | O_NOFOLLOW | O_CLOEXEC | O_DIRECTORY,
                0,
            )
        };
        if descriptor < 0 {
            return Err(());
        }
        unsafe { fs::File::from_raw_fd(descriptor) }
    };
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    let file = fs::File::open(path).map_err(|_| ())?;
    let metadata = file.metadata().map_err(|_| ())?;
    if !metadata.is_dir() || metadata_is_reparse(&metadata) {
        return Err(());
    }
    ensure_path_matches_handle(path, &file)?;
    Ok(file)
}

#[cfg(windows)]
fn open_windows_child_directory_hold(
    parent: &fs::File,
    name: &str,
    write_access: bool,
) -> Result<fs::File, ()> {
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::{AsRawHandle, FromRawHandle};

    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains(['/', '\\', ':'])
        || name.ends_with([' ', '.'])
    {
        return Err(());
    }

    #[repr(C)]
    struct UnicodeString {
        length: u16,
        maximum_length: u16,
        buffer: *mut u16,
    }
    #[repr(C)]
    struct ObjectAttributes {
        length: u32,
        root_directory: *mut core::ffi::c_void,
        object_name: *mut UnicodeString,
        attributes: u32,
        security_descriptor: *mut core::ffi::c_void,
        security_quality_of_service: *mut core::ffi::c_void,
    }
    #[repr(C)]
    struct IoStatusBlock {
        status: isize,
        information: usize,
    }
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtCreateFile(
            file_handle: *mut *mut core::ffi::c_void,
            desired_access: u32,
            object_attributes: *mut ObjectAttributes,
            io_status: *mut IoStatusBlock,
            allocation_size: *const i64,
            file_attributes: u32,
            share_access: u32,
            create_disposition: u32,
            create_options: u32,
            ea_buffer: *const core::ffi::c_void,
            ea_length: u32,
        ) -> i32;
    }

    let mut wide: Vec<u16> = std::ffi::OsStr::new(name).encode_wide().collect();
    let byte_length = wide.len().checked_mul(2).ok_or(())?;
    let mut unicode = UnicodeString {
        length: u16::try_from(byte_length).map_err(|_| ())?,
        maximum_length: u16::try_from(byte_length).map_err(|_| ())?,
        buffer: wide.as_mut_ptr(),
    };
    let mut attributes = ObjectAttributes {
        length: u32::try_from(std::mem::size_of::<ObjectAttributes>()).map_err(|_| ())?,
        root_directory: parent.as_raw_handle(),
        object_name: &mut unicode,
        attributes: 0x0000_0040,
        security_descriptor: std::ptr::null_mut(),
        security_quality_of_service: std::ptr::null_mut(),
    };
    let mut io_status = IoStatusBlock {
        status: 0,
        information: 0,
    };
    let mut raw = std::ptr::null_mut();
    let desired_access = if write_access {
        0x0010_0183
    } else {
        0x0010_0081
    };
    let status = unsafe {
        NtCreateFile(
            &mut raw,
            desired_access,
            &mut attributes,
            &mut io_status,
            std::ptr::null(),
            0,
            0x0000_0001 | 0x0000_0002,
            1,
            0x0000_0001 | 0x0000_0020 | 0x0020_0000,
            std::ptr::null(),
            0,
        )
    };
    if status < 0 || raw.is_null() {
        return Err(());
    }
    let file = unsafe { fs::File::from_raw_handle(raw) };
    let metadata = file.metadata().map_err(|_| ())?;
    if !metadata.is_dir() || metadata_is_reparse(&metadata) {
        return Err(());
    }
    Ok(file)
}

#[cfg(windows)]
fn open_child_directory_hold(
    parent: &fs::File,
    _parent_path: &Path,
    name: &str,
) -> Result<fs::File, ()> {
    open_windows_child_directory_hold(parent, name, false)
}

#[cfg(windows)]
fn open_publication_child_directory_hold(
    parent: &fs::File,
    parent_path: &Path,
    retained_child: &fs::File,
    name: &str,
) -> Result<fs::File, ()> {
    let child = open_windows_child_directory_hold(parent, name, true)?;
    if !handles_identify_same_object(&child, retained_child) {
        return Err(());
    }
    ensure_path_matches_handle(&parent_path.join(name), &child)?;
    Ok(child)
}

#[cfg(target_os = "linux")]
fn open_child_directory_hold(
    parent: &fs::File,
    _parent_path: &Path,
    name: &str,
) -> Result<fs::File, ()> {
    use std::ffi::CString;
    use std::os::fd::{AsRawFd, FromRawFd};

    unsafe extern "C" {
        fn openat(directory: i32, path: *const i8, flags: i32, ...) -> i32;
    }

    const O_RDONLY: i32 = 0;
    const O_CLOEXEC: i32 = 0x0008_0000;
    const O_DIRECTORY: i32 = 0x0001_0000;
    const O_NOFOLLOW: i32 = 0x0002_0000;

    let name = CString::new(name.as_bytes()).map_err(|_| ())?;
    let descriptor = unsafe {
        openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
            0,
        )
    };
    if descriptor < 0 {
        return Err(());
    }
    let child = unsafe { fs::File::from_raw_fd(descriptor) };
    let metadata = child.metadata().map_err(|_| ())?;
    if !metadata.is_dir() || metadata_is_reparse(&metadata) {
        return Err(());
    }
    Ok(child)
}

#[cfg(target_os = "macos")]
fn open_child_directory_hold(
    parent: &fs::File,
    parent_path: &Path,
    name: &str,
) -> Result<fs::File, ()> {
    use std::ffi::CString;
    use std::os::fd::{AsRawFd, FromRawFd};

    unsafe extern "C" {
        fn openat(directory: i32, path: *const i8, flags: i32, ...) -> i32;
    }

    const O_RDONLY: i32 = 0;
    const O_CLOEXEC: i32 = 0x0100_0000;
    const O_DIRECTORY: i32 = 0x0010_0000;
    const O_NOFOLLOW: i32 = 0x0000_0100;

    ensure_path_matches_handle(parent_path, parent)?;
    let child_path = parent_path.join(name);
    let name = CString::new(name.as_bytes()).map_err(|_| ())?;
    let descriptor = unsafe {
        openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
            0,
        )
    };
    if descriptor < 0 {
        return Err(());
    }
    let child = unsafe { fs::File::from_raw_fd(descriptor) };
    let metadata = child.metadata().map_err(|_| ())?;
    if !metadata.is_dir() || metadata_is_reparse(&metadata) {
        return Err(());
    }
    ensure_path_matches_handle(parent_path, parent)?;
    ensure_path_matches_handle(&child_path, &child)?;
    Ok(child)
}

#[cfg(all(not(windows), not(target_os = "linux"), not(target_os = "macos")))]
fn open_child_directory_hold(
    _parent: &fs::File,
    _parent_path: &Path,
    _name: &str,
) -> Result<fs::File, ()> {
    Err(())
}

#[cfg(target_os = "linux")]
fn namespace_contents_path(_path: &Path, hold: &fs::File) -> Result<PathBuf, ()> {
    use std::os::fd::AsRawFd;

    let path = PathBuf::from(format!("/proc/self/fd/{}", hold.as_raw_fd()));
    Ok(path)
}

#[cfg(windows)]
fn namespace_contents_path(path: &Path, hold: &fs::File) -> Result<PathBuf, ()> {
    ensure_path_matches_handle(path, hold)?;
    Ok(path.to_path_buf())
}

#[cfg(target_os = "macos")]
fn namespace_contents_path(path: &Path, hold: &fs::File) -> Result<PathBuf, ()> {
    ensure_path_matches_handle(path, hold)?;
    Ok(path.to_path_buf())
}

#[cfg(all(not(windows), not(target_os = "linux"), not(target_os = "macos")))]
fn namespace_contents_path(_path: &Path, _hold: &fs::File) -> Result<PathBuf, ()> {
    Err(())
}

#[cfg(not(windows))]
fn open_publication_child_directory_hold(
    _parent: &fs::File,
    parent_path: &Path,
    retained_child: &fs::File,
    name: &str,
) -> Result<fs::File, ()> {
    let child = retained_child.try_clone().map_err(|_| ())?;
    ensure_path_matches_handle(&parent_path.join(name), &child)?;
    Ok(child)
}

#[cfg(windows)]
fn ensure_path_matches_handle(path: &Path, file: &fs::File) -> Result<(), ()> {
    let path_metadata = fs::symlink_metadata(path).map_err(|_| ())?;
    if path_metadata.file_type().is_symlink() || metadata_is_reparse(&path_metadata) {
        return Err(());
    }
    let comparison = open_windows_identity_handle(path, path_metadata.is_dir())?;
    if windows_file_identity(file)? != windows_file_identity(&comparison)? {
        return Err(());
    }
    Ok(())
}

#[cfg(not(windows))]
fn ensure_path_matches_handle(path: &Path, file: &fs::File) -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        let proc_fd_root = Path::new("/proc/self/fd");
        if let Ok(relative) = path.strip_prefix(proc_fd_root) {
            let components = relative.components().collect::<Vec<_>>();
            if components.len() == 1 {
                if let Component::Normal(descriptor) = components[0] {
                    if !descriptor.is_empty()
                        && descriptor.as_encoded_bytes().iter().all(u8::is_ascii_digit)
                    {
                        // Linux namespace traversal is rooted through a live
                        // `/proc/self/fd/<n>` descriptor. That proc entry is
                        // necessarily a symlink, so authenticate its resolved
                        // target against the supplied held handle instead.
                        let path_metadata = fs::metadata(path).map_err(|_| ())?;
                        let handle_metadata = file.metadata().map_err(|_| ())?;
                        if metadata_is_reparse(&path_metadata)
                            || !metadata_identity_matches(&path_metadata, &handle_metadata)
                        {
                            return Err(());
                        }
                        return Ok(());
                    }
                }
            }
        }
    }
    let path_metadata = fs::symlink_metadata(path).map_err(|_| ())?;
    let handle_metadata = file.metadata().map_err(|_| ())?;
    if path_metadata.file_type().is_symlink()
        || metadata_is_reparse(&path_metadata)
        || !metadata_identity_matches(&path_metadata, &handle_metadata)
    {
        return Err(());
    }
    Ok(())
}

fn read_regular_file(
    path: &Path,
    budget: &mut NamespaceBudget,
) -> Result<RetainedFileRead, NamespaceReadError> {
    #[cfg(windows)]
    let mut file = {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_SHARE_READ: u32 = 0x0000_0001;
        const FILE_SHARE_WRITE: u32 = 0x0000_0002;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|_| NamespaceReadError::Io)?
    };
    #[cfg(target_os = "linux")]
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt;
        const O_NONBLOCK: i32 = 0x0000_0800;
        const O_NOFOLLOW: i32 = 0x0002_0000;
        OpenOptions::new()
            .read(true)
            .custom_flags(O_NONBLOCK | O_NOFOLLOW)
            .open(path)
            .map_err(|_| NamespaceReadError::Io)?
    };
    #[cfg(target_os = "macos")]
    let mut file =
        open_identity_handle_for_regular_file(path).map_err(|_| NamespaceReadError::Io)?;
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|_| NamespaceReadError::Io)?;
    let metadata = file.metadata().map_err(|_| NamespaceReadError::Io)?;
    if !metadata.is_file()
        || metadata_is_reparse(&metadata)
        || !metadata_has_admitted_link_count(path, &metadata, &file)
    {
        return Err(NamespaceReadError::Invalid);
    }
    ensure_path_matches_handle(path, &file).map_err(|_| NamespaceReadError::Invalid)?;
    let expected_length = budget.reserve(metadata.len())?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(expected_length.saturating_add(1))
        .map_err(|_| NamespaceReadError::ResourceLimit)?;
    Read::by_ref(&mut file)
        .take(u64::try_from(AUTHORITATIVE_STORE_MAX_OBJECT_BYTES).unwrap() + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| NamespaceReadError::Io)?;
    if bytes.len() != expected_length || bytes.len() > AUTHORITATIVE_STORE_MAX_OBJECT_BYTES {
        return Err(NamespaceReadError::Invalid);
    }
    ensure_path_matches_handle(path, &file).map_err(|_| NamespaceReadError::Invalid)?;
    let expected_sha256 = Sha256::digest(&bytes).into();
    Ok(RetainedFileRead {
        witness: RetainedFileWitness {
            path: path.to_path_buf(),
            file,
            expected_length: bytes.len(),
            expected_sha256,
        },
        bytes,
    })
}

fn revalidate_retained_file_witness(witness: &mut RetainedFileWitness) -> Result<(), ()> {
    let metadata = witness.file.metadata().map_err(|_| ())?;
    if !metadata.is_file()
        || metadata_is_reparse(&metadata)
        || !metadata_has_admitted_link_count(&witness.path, &metadata, &witness.file)
        || usize::try_from(metadata.len()).map_err(|_| ())? != witness.expected_length
    {
        return Err(());
    }
    ensure_path_matches_handle(&witness.path, &witness.file)?;
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::FileExt;

        let read_limit = witness.expected_length.checked_add(1).ok_or(())?;
        let mut bytes = vec![0; read_limit];
        let mut offset = 0usize;
        while offset < read_limit {
            match witness.file.read_at(&mut bytes[offset..], offset as u64) {
                Ok(0) => break,
                Ok(read) => offset += read,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => return Err(()),
            }
        }
        bytes.truncate(offset);
        let metadata_after = witness.file.metadata().map_err(|_| ())?;
        if !metadata_after.is_file()
            || metadata_is_reparse(&metadata_after)
            || !metadata_has_admitted_link_count(&witness.path, &metadata_after, &witness.file)
            || usize::try_from(metadata_after.len()).map_err(|_| ())? != witness.expected_length
        {
            return Err(());
        }
        ensure_path_matches_handle(&witness.path, &witness.file)?;
        if bytes.len() == witness.expected_length
            && <[u8; ID_LENGTH]>::from(Sha256::digest(&bytes)) == witness.expected_sha256
        {
            Ok(())
        } else {
            Err(())
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        witness.file.seek(SeekFrom::Start(0)).map_err(|_| ())?;
        let read_limit = u64::try_from(witness.expected_length)
            .map_err(|_| ())?
            .checked_add(1)
            .ok_or(())?;
        let mut bytes = Vec::new();
        Read::by_ref(&mut witness.file)
            .take(read_limit)
            .read_to_end(&mut bytes)
            .map_err(|_| ())?;
        if bytes.len() != witness.expected_length
            || <[u8; ID_LENGTH]>::from(Sha256::digest(&bytes)) != witness.expected_sha256
        {
            return Err(());
        }
        Ok(())
    }
}

#[cfg(windows)]
fn open_retained_file_guard(source: &RetainedFileWitness) -> Result<RetainedFileWitness, ()> {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&source.path)
        .map_err(|_| ())?;
    if !handles_identify_same_object(&source.file, &file) {
        return Err(());
    }
    let mut guard = RetainedFileWitness {
        path: source.path.clone(),
        file,
        expected_length: source.expected_length,
        expected_sha256: source.expected_sha256,
    };
    revalidate_retained_file_witness(&mut guard)?;
    Ok(guard)
}

#[cfg(target_os = "linux")]
fn open_retained_file_guard(source: &RetainedFileWitness) -> Result<RetainedFileWitness, ()> {
    use std::os::unix::fs::OpenOptionsExt;

    const O_NONBLOCK: i32 = 0x0000_0800;
    const O_NOFOLLOW: i32 = 0x0002_0000;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(O_NONBLOCK | O_NOFOLLOW)
        .open(&source.path)
        .map_err(|_| ())?;
    if !handles_identify_same_object(&source.file, &file) {
        return Err(());
    }
    let mut guard = RetainedFileWitness {
        path: source.path.clone(),
        file,
        expected_length: source.expected_length,
        expected_sha256: source.expected_sha256,
    };
    revalidate_retained_file_witness(&mut guard)?;
    Ok(guard)
}

#[cfg(target_os = "macos")]
fn open_retained_file_guard(source: &RetainedFileWitness) -> Result<RetainedFileWitness, ()> {
    use std::os::fd::AsRawFd;

    unsafe extern "C" {
        fn flock(fd: i32, operation: i32) -> i32;
    }

    const LOCK_SH: i32 = 1;
    const LOCK_NB: i32 = 4;

    let mut guard = RetainedFileWitness {
        path: source.path.clone(),
        file: source.file.try_clone().map_err(|_| ())?,
        expected_length: source.expected_length,
        expected_sha256: source.expected_sha256,
    };
    if unsafe { flock(guard.file.as_raw_fd(), LOCK_SH | LOCK_NB) } != 0 {
        return Err(());
    }
    if !handles_identify_same_object(&source.file, &guard.file) {
        return Err(());
    }
    revalidate_retained_file_witness(&mut guard)?;
    Ok(guard)
}

#[cfg(target_os = "macos")]
fn into_retained_file_guard(mut witness: RetainedFileWitness) -> Result<RetainedFileWitness, ()> {
    use std::os::fd::AsRawFd;

    unsafe extern "C" {
        fn flock(fd: i32, operation: i32) -> i32;
    }

    const LOCK_SH: i32 = 1;
    const LOCK_NB: i32 = 4;

    if unsafe { flock(witness.file.as_raw_fd(), LOCK_SH | LOCK_NB) } != 0 {
        return Err(());
    }
    revalidate_retained_file_witness(&mut witness)?;
    Ok(witness)
}

#[cfg(not(target_os = "macos"))]
fn into_retained_file_guard(witness: RetainedFileWitness) -> Result<RetainedFileWitness, ()> {
    open_retained_file_guard(&witness)
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn open_retained_file_guard(_source: &RetainedFileWitness) -> Result<RetainedFileWitness, ()> {
    Err(())
}

#[cfg(windows)]
fn downgrade_publication_witness(source: RetainedFileWitness) -> Result<RetainedFileWitness, ()> {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_SHARE_WRITE: u32 = 0x0000_0002;
    const FILE_SHARE_DELETE: u32 = 0x0000_0004;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;

    let path = source.path.clone();
    let mut file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&source.path)
        .map_err(|_| ())?;
    if !handles_identify_same_object(&source.file, &file) {
        return Err(());
    }
    let metadata = file.metadata().map_err(|_| ())?;
    if !metadata.is_file()
        || metadata_is_reparse(&metadata)
        || metadata_link_count(&metadata, &file) != Some(1)
        || usize::try_from(metadata.len()).map_err(|_| ())? != source.expected_length
    {
        return Err(());
    }
    let read_limit = u64::try_from(source.expected_length)
        .map_err(|_| ())?
        .checked_add(1)
        .ok_or(())?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    if bytes.len() != source.expected_length
        || <[u8; ID_LENGTH]>::from(Sha256::digest(&bytes)) != source.expected_sha256
    {
        return Err(());
    }
    drop(source);
    let retained = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&path)
        .map_err(|_| ())?;
    if !handles_identify_same_object(&file, &retained) {
        return Err(());
    }
    drop(file);
    let mut witness = RetainedFileWitness {
        path,
        file: retained,
        expected_length: bytes.len(),
        expected_sha256: Sha256::digest(&bytes).into(),
    };
    revalidate_retained_file_witness(&mut witness)?;
    Ok(witness)
}

#[cfg(target_os = "linux")]
fn downgrade_publication_witness(source: RetainedFileWitness) -> Result<RetainedFileWitness, ()> {
    use std::os::unix::fs::OpenOptionsExt;

    const O_NONBLOCK: i32 = 0x0000_0800;
    const O_NOFOLLOW: i32 = 0x0002_0000;
    let path = source.path.clone();
    let expected_length = source.expected_length;
    let expected_sha256 = source.expected_sha256;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(O_NONBLOCK | O_NOFOLLOW)
        .open(&path)
        .map_err(|_| ())?;
    if !handles_identify_same_object(&source.file, &file) {
        return Err(());
    }
    let metadata = file.metadata().map_err(|_| ())?;
    if !metadata.is_file()
        || metadata_is_reparse(&metadata)
        || metadata_link_count(&metadata, &file) != Some(1)
        || usize::try_from(metadata.len()).map_err(|_| ())? != expected_length
    {
        return Err(());
    }
    let read_limit = u64::try_from(expected_length)
        .map_err(|_| ())?
        .checked_add(1)
        .ok_or(())?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    if bytes.len() != expected_length
        || <[u8; ID_LENGTH]>::from(Sha256::digest(&bytes)) != expected_sha256
    {
        return Err(());
    }
    drop(source);
    let mut witness = RetainedFileWitness {
        path,
        file,
        expected_length,
        expected_sha256,
    };
    revalidate_retained_file_witness(&mut witness)?;
    Ok(witness)
}

#[cfg(target_os = "macos")]
fn downgrade_publication_witness(source: RetainedFileWitness) -> Result<RetainedFileWitness, ()> {
    let file = open_identity_handle_for_regular_file(&source.path)?;
    if !handles_identify_same_object(&source.file, &file) {
        return Err(());
    }
    let metadata = file.metadata().map_err(|_| ())?;
    if !metadata.is_file()
        || metadata_is_reparse(&metadata)
        || !metadata_has_admitted_link_count(&source.path, &metadata, &file)
        || usize::try_from(metadata.len()).map_err(|_| ())? != source.expected_length
    {
        return Err(());
    }
    let mut witness = RetainedFileWitness {
        path: source.path.clone(),
        file,
        expected_length: source.expected_length,
        expected_sha256: source.expected_sha256,
    };
    revalidate_retained_file_witness(&mut witness)?;
    Ok(witness)
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn downgrade_publication_witness(_source: RetainedFileWitness) -> Result<RetainedFileWitness, ()> {
    Err(())
}

fn metadata_has_admitted_link_count(path: &Path, metadata: &fs::Metadata, file: &fs::File) -> bool {
    let Some(link_count) = metadata_link_count(metadata, file) else {
        return false;
    };
    if link_count == 1 {
        return true;
    }
    if link_count != 2 {
        return false;
    }
    let Some(parent) = path.parent() else {
        return false;
    };
    let Ok(entries) = fs::read_dir(parent) else {
        return false;
    };
    let mut matching_temps = 0_u8;
    for entry in entries {
        let Ok(entry) = entry else {
            return false;
        };
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            return false;
        };
        if !is_publication_temp_name(&name) {
            continue;
        }
        let Ok(candidate) = open_identity_handle_for_regular_file(&entry.path()) else {
            return false;
        };
        if handles_identify_same_object(file, &candidate) {
            matching_temps = match matching_temps.checked_add(1) {
                Some(count) => count,
                None => return false,
            };
        }
    }
    matching_temps == 1
}

#[cfg(windows)]
fn metadata_link_count(_metadata: &fs::Metadata, file: &fs::File) -> Option<u64> {
    windows_file_identity(file)
        .ok()
        .map(|identity| u64::from(identity.link_count))
}

#[cfg(unix)]
fn metadata_link_count(metadata: &fs::Metadata, _file: &fs::File) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    Some(metadata.nlink())
}

#[cfg(not(any(windows, unix)))]
fn metadata_link_count(_metadata: &fs::Metadata, _file: &fs::File) -> Option<u64> {
    None
}

fn open_identity_handle_for_regular_file(path: &Path) -> Result<fs::File, ()> {
    #[cfg(windows)]
    {
        open_windows_identity_handle(path, false)
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;

        const O_NONBLOCK: i32 = 0x0000_0800;
        const O_NOFOLLOW: i32 = 0x0002_0000;
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(O_NONBLOCK | O_NOFOLLOW)
            .open(path)
            .map_err(|_| ())?;
        let metadata = file.metadata().map_err(|_| ())?;
        (metadata.is_file() && !metadata_is_reparse(&metadata))
            .then_some(file)
            .ok_or(())
    }
    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;
        use std::os::fd::FromRawFd;
        use std::os::unix::ffi::OsStrExt;

        unsafe extern "C" {
            fn open(path: *const i8, flags: i32, ...) -> i32;
        }
        const O_RDONLY: i32 = 0;
        const O_NONBLOCK: i32 = 0x0000_0004;
        const O_NOFOLLOW: i32 = 0x0000_0100;
        const O_CLOEXEC: i32 = 0x0100_0000;
        let path = CString::new(path.as_os_str().as_bytes()).map_err(|_| ())?;
        let descriptor = unsafe {
            open(
                path.as_ptr(),
                O_RDONLY | O_NONBLOCK | O_NOFOLLOW | O_CLOEXEC,
            )
        };
        if descriptor < 0 {
            return Err(());
        }
        let file = unsafe { fs::File::from_raw_fd(descriptor) };
        if !file.metadata().map_err(|_| ())?.is_file() {
            return Err(());
        }
        Ok(file)
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        fs::File::open(path).map_err(|_| ())
    }
}

#[cfg(windows)]
fn handles_identify_same_object(left: &fs::File, right: &fs::File) -> bool {
    windows_file_identity(left)
        .ok()
        .zip(windows_file_identity(right).ok())
        .is_some_and(|(left, right)| {
            left.volume_serial_number == right.volume_serial_number
                && left.file_index == right.file_index
        })
}

#[cfg(unix)]
fn handles_identify_same_object(left: &fs::File, right: &fs::File) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.metadata()
        .ok()
        .zip(right.metadata().ok())
        .is_some_and(|(left, right)| left.dev() == right.dev() && left.ino() == right.ino())
}

#[cfg(not(any(windows, unix)))]
fn handles_identify_same_object(_left: &fs::File, _right: &fs::File) -> bool {
    false
}

#[cfg(windows)]
fn metadata_is_reparse(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn metadata_is_reparse(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(windows)]
fn open_windows_identity_handle(path: &Path, directory: bool) -> Result<fs::File, ()> {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_SHARE_WRITE: u32 = 0x0000_0002;
    const FILE_SHARE_DELETE: u32 = 0x0000_0004;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(
            FILE_FLAG_OPEN_REPARSE_POINT
                | if directory {
                    FILE_FLAG_BACKUP_SEMANTICS
                } else {
                    0
                },
        )
        .open(path)
        .map_err(|_| ())
}

#[cfg(windows)]
#[derive(Clone, Copy, PartialEq, Eq)]
struct WindowsFileIdentity {
    volume_serial_number: u32,
    file_index: u64,
    link_count: u32,
}

#[cfg(windows)]
#[repr(C)]
struct WindowsFileTime {
    low_date_time: u32,
    high_date_time: u32,
}

#[cfg(windows)]
#[repr(C)]
struct WindowsByHandleFileInformation {
    file_attributes: u32,
    creation_time: WindowsFileTime,
    last_access_time: WindowsFileTime,
    last_write_time: WindowsFileTime,
    volume_serial_number: u32,
    file_size_high: u32,
    file_size_low: u32,
    number_of_links: u32,
    file_index_high: u32,
    file_index_low: u32,
}

#[cfg(windows)]
fn windows_file_identity(file: &fs::File) -> Result<WindowsFileIdentity, ()> {
    use std::ffi::c_void;
    use std::os::windows::io::AsRawHandle;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        #[link_name = "GetFileInformationByHandle"]
        fn get_file_information_by_handle(
            file: *mut c_void,
            information: *mut WindowsByHandleFileInformation,
        ) -> i32;
    }

    let mut information = std::mem::MaybeUninit::<WindowsByHandleFileInformation>::uninit();
    // SAFETY: `file` is a live owned handle and `information` points to writable storage with the
    // exact Win32 BY_HANDLE_FILE_INFORMATION layout for the duration of the call.
    let succeeded =
        unsafe { get_file_information_by_handle(file.as_raw_handle(), information.as_mut_ptr()) };
    if succeeded == 0 {
        return Err(());
    }
    // SAFETY: a nonzero Win32 result initializes the complete output structure.
    let information = unsafe { information.assume_init() };
    Ok(WindowsFileIdentity {
        volume_serial_number: information.volume_serial_number,
        file_index: (u64::from(information.file_index_high) << 32)
            | u64::from(information.file_index_low),
        link_count: information.number_of_links,
    })
}

#[cfg(unix)]
fn metadata_identity_matches(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(not(any(windows, unix)))]
fn metadata_identity_matches(_left: &fs::Metadata, _right: &fs::Metadata) -> bool {
    false
}

fn load_journal_slots(
    journal_dir: &Path,
    budget: &mut NamespaceBudget,
) -> Result<(Vec<Vec<u8>>, Vec<RetainedFileWitness>), AuthoritativeRegistryStoreOpenError> {
    let mut slots = Vec::new();
    let mut witnesses = Vec::new();
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
        if is_publication_temp_name(&name) {
            admit_bounded_publication_temp_residue(&entry.path(), budget)?;
            continue;
        }
        let index = parse_journal_slot_name(&name)
            .ok_or(AuthoritativeRegistryStoreOpenError::JournalSlotNameInvalid)?;
        let read = read_regular_file(&entry.path(), budget).map_err(|error| match error {
            NamespaceReadError::Io => AuthoritativeRegistryStoreOpenError::Io,
            NamespaceReadError::Invalid => {
                AuthoritativeRegistryStoreOpenError::JournalNamespaceInvalid
            }
            NamespaceReadError::ResourceLimit => {
                AuthoritativeRegistryStoreOpenError::NamespaceResourceLimit
            }
        })?;
        slots.push((index, read.bytes));
        witnesses.push(read.witness);
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
    Ok((
        slots.into_iter().map(|(_, bytes)| bytes).collect(),
        witnesses,
    ))
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
    budget: &mut NamespaceBudget,
) -> Result<LoadedRecordNamespace, AuthoritativeRegistryStoreOpenError> {
    let mut records = Vec::new();
    let mut witnesses = Vec::new();
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
        if is_publication_temp_name(&name) {
            admit_bounded_publication_temp_residue(&entry.path(), budget)?;
            continue;
        }
        let expected_id = parse_record_filename(&name)
            .ok_or(AuthoritativeRegistryStoreOpenError::RecordFilenameInvalid)?;
        let read = read_regular_file(&entry.path(), budget).map_err(|error| match error {
            NamespaceReadError::Io => AuthoritativeRegistryStoreOpenError::Io,
            NamespaceReadError::Invalid => {
                AuthoritativeRegistryStoreOpenError::RecordNamespaceInvalid
            }
            NamespaceReadError::ResourceLimit => {
                AuthoritativeRegistryStoreOpenError::NamespaceResourceLimit
            }
        })?;
        let frame = StrictRecordFrame::decode_authoritative(&read.bytes)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RecordDecode)?;
        if frame.record_id() != expected_id {
            return Err(AuthoritativeRegistryStoreOpenError::RecordIdentityMismatch);
        }
        records.push((expected_id, read.bytes));
        witnesses.push(read.witness);
    }
    records.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    if records
        .windows(2)
        .any(|pair| pair[0].0.as_bytes() == pair[1].0.as_bytes())
    {
        return Err(AuthoritativeRegistryStoreOpenError::RecordNamespaceInvalid);
    }
    Ok((records, witnesses))
}

fn decode_event_record_body<'a>(
    input: &'a [u8],
    expected_record_type: u16,
) -> Result<(CborCursor<'a>, usize), RecordDecodeError> {
    let frame = StrictRecordFrame::decode_authoritative(input)?;
    if frame.record_type_id().value() != expected_record_type {
        return Err(RecordDecodeError);
    }
    let mut cursor = CborCursor::new(input);
    cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
    cursor
        .text_exact(RECORD_DOMAIN)
        .map_err(|_| RecordDecodeError)?;
    if cursor.uint().map_err(|_| RecordDecodeError)? != u64::from(expected_record_type)
        || cursor.uint().map_err(|_| RecordDecodeError)? != 1
    {
        return Err(RecordDecodeError);
    }
    let field_count = cursor.map().map_err(|_| RecordDecodeError)?;
    if field_count < 2 {
        return Err(RecordDecodeError);
    }
    cursor.key(0).map_err(|_| RecordDecodeError)?;
    if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
        return Err(RecordDecodeError);
    }
    cursor.key(1).map_err(|_| RecordDecodeError)?;
    if cursor.uint().map_err(|_| RecordDecodeError)? != u64::from(expected_record_type) {
        return Err(RecordDecodeError);
    }
    Ok((cursor, field_count - 2))
}

fn decode_record_id_value(cursor: &mut CborCursor<'_>) -> Result<(), RecordDecodeError> {
    RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
        .map(|_| ())
        .map_err(|_| RecordDecodeError)
}

fn decode_journal_reference_value(cursor: &mut CborCursor<'_>) -> Result<(), RecordDecodeError> {
    decode_journal_reference(cursor)
        .map(|_| ())
        .map_err(|_| RecordDecodeError)
}

fn decode_sorted_text_set(cursor: &mut CborCursor<'_>) -> Result<(), RecordDecodeError> {
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count > cursor.remaining() {
        return Err(RecordDecodeError);
    }
    let mut previous: Option<Vec<u8>> = None;
    for _ in 0..count {
        let value = cursor.text().map_err(|_| RecordDecodeError)?;
        if previous
            .as_ref()
            .is_some_and(|prior| prior.as_slice() >= value.as_bytes())
        {
            return Err(RecordDecodeError);
        }
        previous = Some(value.into_bytes());
    }
    Ok(())
}

fn decode_sorted_uint_set(cursor: &mut CborCursor<'_>) -> Result<(), RecordDecodeError> {
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count > cursor.remaining() {
        return Err(RecordDecodeError);
    }
    let mut previous = None;
    for _ in 0..count {
        let value = cursor.uint().map_err(|_| RecordDecodeError)?;
        if previous.is_some_and(|prior| prior >= value) {
            return Err(RecordDecodeError);
        }
        previous = Some(value);
    }
    Ok(())
}

fn decode_sorted_event_type_id_set(cursor: &mut CborCursor<'_>) -> Result<(), RecordDecodeError> {
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count > cursor.remaining() {
        return Err(RecordDecodeError);
    }
    let mut previous = None;
    for _ in 0..count {
        let value = EventTypeId::try_from(cursor.uint().map_err(|_| RecordDecodeError)?)
            .map_err(|_| RecordDecodeError)?;
        if previous.is_some_and(|prior: EventTypeId| prior.value() >= value.value()) {
            return Err(RecordDecodeError);
        }
        previous = Some(value);
    }
    Ok(())
}

fn decode_sorted_record_id_set(cursor: &mut CborCursor<'_>) -> Result<(), RecordDecodeError> {
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count > cursor.remaining() / 34 {
        return Err(RecordDecodeError);
    }
    let mut previous: Option<[u8; ID_LENGTH]> = None;
    for _ in 0..count {
        let value = cursor.bstr_32().map_err(|_| RecordDecodeError)?;
        if previous.is_some_and(|prior| prior >= value) {
            return Err(RecordDecodeError);
        }
        previous = Some(value);
    }
    Ok(())
}

fn decode_sorted_journal_reference_set(
    cursor: &mut CborCursor<'_>,
) -> Result<(), RecordDecodeError> {
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count > cursor.remaining() / 105 {
        return Err(RecordDecodeError);
    }
    let mut previous: Option<(u64, [u8; ID_LENGTH])> = None;
    for _ in 0..count {
        let reference = decode_journal_reference(cursor).map_err(|_| RecordDecodeError)?;
        let ordering_key = (
            reference.entry_index().value(),
            *reference.entry_hash().as_bytes(),
        );
        if previous.is_some_and(|prior| prior >= ordering_key) {
            return Err(RecordDecodeError);
        }
        previous = Some(ordering_key);
    }
    Ok(())
}

fn decode_failed_candidate_record_set(
    cursor: &mut CborCursor<'_>,
    allowed_roles: std::ops::RangeInclusive<u64>,
) -> Result<u16, RecordDecodeError> {
    Ok(decode_failed_candidate_record_set_with_values(cursor, allowed_roles)?.0)
}

fn decode_failed_candidate_record_set_with_values(
    cursor: &mut CborCursor<'_>,
    allowed_roles: std::ops::RangeInclusive<u64>,
) -> Result<(u16, Vec<(u64, RecordId)>), RecordDecodeError> {
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count == 0 || count > cursor.remaining() / 39 {
        return Err(RecordDecodeError);
    }
    let mut previous: Option<(u64, [u8; ID_LENGTH], u64)> = None;
    let mut role_mask = 0_u16;
    let mut candidates = Vec::new();
    candidates
        .try_reserve_exact(count)
        .map_err(|_| RecordDecodeError)?;
    for _ in 0..count {
        cursor.map_exact(3).map_err(|_| RecordDecodeError)?;
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        let role = cursor.uint().map_err(|_| RecordDecodeError)?;
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        let record_id = cursor.bstr_32().map_err(|_| RecordDecodeError)?;
        cursor.key(2).map_err(|_| RecordDecodeError)?;
        let failure = cursor.uint().map_err(|_| RecordDecodeError)?;
        let value = (role, record_id, failure);
        if !allowed_roles.contains(&role)
            || !(20..=25).contains(&failure)
            || previous.as_ref().is_some_and(|prior| prior >= &value)
            || previous
                .as_ref()
                .is_some_and(|prior| prior.0 == role && prior.1 == record_id)
        {
            return Err(RecordDecodeError);
        }
        role_mask |= 1_u16
            .checked_shl(u32::try_from(role).map_err(|_| RecordDecodeError)?)
            .ok_or(RecordDecodeError)?;
        candidates.push((
            role,
            RecordId::try_from(record_id.as_slice()).map_err(|_| RecordDecodeError)?,
        ));
        previous = Some(value);
    }
    Ok((role_mask, candidates))
}

fn decode_failed_candidate_journal_set(
    cursor: &mut CborCursor<'_>,
    allowed_roles: std::ops::RangeInclusive<u64>,
) -> Result<u16, RecordDecodeError> {
    Ok(decode_failed_candidate_journal_set_with_values(cursor, allowed_roles)?.0)
}

fn decode_failed_candidate_journal_set_with_values(
    cursor: &mut CborCursor<'_>,
    allowed_roles: std::ops::RangeInclusive<u64>,
) -> Result<(u16, Vec<(u64, JournalReference)>), RecordDecodeError> {
    type FailedCandidateJournalSortKey = (
        u64,
        [u8; ID_LENGTH],
        u64,
        [u8; ID_LENGTH],
        u16,
        [u8; ID_LENGTH],
        u64,
    );
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count == 0 || count > cursor.remaining() / 110 {
        return Err(RecordDecodeError);
    }
    let mut previous: Option<FailedCandidateJournalSortKey> = None;
    let mut role_mask = 0_u16;
    let mut candidates = Vec::new();
    candidates
        .try_reserve_exact(count)
        .map_err(|_| RecordDecodeError)?;
    for _ in 0..count {
        cursor.map_exact(3).map_err(|_| RecordDecodeError)?;
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        let role = cursor.uint().map_err(|_| RecordDecodeError)?;
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        let reference = decode_journal_reference(cursor).map_err(|_| RecordDecodeError)?;
        cursor.key(2).map_err(|_| RecordDecodeError)?;
        let failure = cursor.uint().map_err(|_| RecordDecodeError)?;
        let value = (
            role,
            *reference.registry_id().as_bytes(),
            reference.entry_index().value(),
            *reference.entry_hash().as_bytes(),
            reference.event_type_id().value(),
            *reference.event_record_id().as_bytes(),
            failure,
        );
        if !allowed_roles.contains(&role)
            || !(1..=9).contains(&failure)
            || previous.as_ref().is_some_and(|prior| prior >= &value)
            || previous.as_ref().is_some_and(|prior| {
                prior.0 == value.0
                    && prior.1 == value.1
                    && prior.2 == value.2
                    && prior.3 == value.3
                    && prior.4 == value.4
                    && prior.5 == value.5
            })
        {
            return Err(RecordDecodeError);
        }
        role_mask |= 1_u16
            .checked_shl(u32::try_from(role).map_err(|_| RecordDecodeError)?)
            .ok_or(RecordDecodeError)?;
        candidates.push((role, reference));
        previous = Some(value);
    }
    Ok((role_mask, candidates))
}

fn decode_formal_support_binding(cursor: &mut CborCursor<'_>) -> Result<(), RecordDecodeError> {
    cursor.map_exact(2).map_err(|_| RecordDecodeError)?;
    cursor.key(0).map_err(|_| RecordDecodeError)?;
    decode_sorted_journal_reference_set(cursor)?;
    cursor.key(1).map_err(|_| RecordDecodeError)?;
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count > cursor.remaining() / 213 {
        return Err(RecordDecodeError);
    }
    let mut previous: Option<Vec<u8>> = None;
    for _ in 0..count {
        let fields = cursor.map().map_err(|_| RecordDecodeError)?;
        if !matches!(fields, 2 | 3) {
            return Err(RecordDecodeError);
        }
        let mut value = Vec::new();
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        let definition = decode_journal_reference(cursor).map_err(|_| RecordDecodeError)?;
        value.extend_from_slice(&definition.authoritative_cbor());
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        let establishment = decode_journal_reference(cursor).map_err(|_| RecordDecodeError)?;
        value.extend_from_slice(&establishment.authoritative_cbor());
        if fields == 3 {
            cursor.key(2).map_err(|_| RecordDecodeError)?;
            let compatibility = decode_journal_reference(cursor).map_err(|_| RecordDecodeError)?;
            value.extend_from_slice(&compatibility.authoritative_cbor());
        }
        if previous.as_ref().is_some_and(|prior| prior >= &value) {
            return Err(RecordDecodeError);
        }
        previous = Some(value);
    }
    Ok(())
}

fn decode_diversity_identity_set(cursor: &mut CborCursor<'_>) -> Result<(), RecordDecodeError> {
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count > cursor.remaining() / 3 {
        return Err(RecordDecodeError);
    }
    let mut previous: Option<(u64, Vec<u8>)> = None;
    for _ in 0..count {
        cursor.array_exact(2).map_err(|_| RecordDecodeError)?;
        let dimension = cursor.uint().map_err(|_| RecordDecodeError)?;
        let identity = cursor.bstr().map_err(|_| RecordDecodeError)?;
        let value = (dimension, identity);
        if !matches!(dimension, 1 | 2)
            || value.1.is_empty()
            || previous.as_ref().is_some_and(|prior| prior >= &value)
        {
            return Err(RecordDecodeError);
        }
        previous = Some(value);
    }
    Ok(())
}

fn validate_remaining_event_record_fields(
    record_type: u16,
    event_type: u16,
    cursor: &mut CborCursor<'_>,
    field_count: usize,
) -> Result<(), RecordDecodeError> {
    let mut present = 0_u64;
    let mut disposition = None;
    let mut target_mode = None;
    let mut terminal_disposition = None;
    let mut observed_eviction_state_present = false;
    let mut formal_outcome = None;
    let mut counterexample_present = false;
    let mut verification_target = None;
    let mut source_binding_present = false;
    let mut failed_candidate_record_roles = 0_u16;
    let mut failed_candidate_journal_roles = 0_u16;
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !(16..=32).contains(&key) {
            return Err(RecordDecodeError);
        }
        present |= 1_u64
            .checked_shl(u32::try_from(key).map_err(|_| RecordDecodeError)?)
            .ok_or(RecordDecodeError)?;
        match (record_type, key) {
            (5, 16) => {
                cursor.bstr_32().map_err(|_| RecordDecodeError)?;
            }
            (5, 17 | 20) => decode_journal_reference_value(cursor)?,
            (5, 18) => decode_record_id_value(cursor)?,
            (5, 19) => decode_sorted_text_set(cursor)?,

            (6, 16) => {
                cursor.bstr_32().map_err(|_| RecordDecodeError)?;
            }
            (6, 17 | 22) => decode_journal_reference_value(cursor)?,
            (6, 18) => cursor
                .text_exact(b"UNKNOWN")
                .map_err(|_| RecordDecodeError)?,
            (6, 19..=21) => {
                cursor.text().map_err(|_| RecordDecodeError)?;
            }

            (7, 16) => {
                cursor.bstr_32().map_err(|_| RecordDecodeError)?;
            }
            (7, 17 | 18) => decode_journal_reference_value(cursor)?,
            (7, 19) => {
                cursor.uint().map_err(|_| RecordDecodeError)?;
            }
            (7, 20) => cursor
                .text_exact(b"ATTEMPT_ALREADY_TERMINAL")
                .map_err(|_| RecordDecodeError)?,

            (12, 16 | 20 | 21 | 29 | 30) => decode_record_id_value(cursor)?,
            (12, 17) => {
                let value = cursor.uint().map_err(|_| RecordDecodeError)?;
                if !matches!(value, 1 | 2) {
                    return Err(RecordDecodeError);
                }
                verification_target = Some(value);
            }
            (12, 18 | 19 | 32) => decode_journal_reference_value(cursor)?,
            (12, 22) => {
                cursor.bstr().map_err(|_| RecordDecodeError)?;
            }
            (12, 23 | 31) => {
                cursor.text().map_err(|_| RecordDecodeError)?;
            }
            (12, 24) => {
                cursor.uint().map_err(|_| RecordDecodeError)?;
            }
            (12, 25 | 26) => {
                if !matches!(cursor.uint().map_err(|_| RecordDecodeError)?, 1..=3) {
                    return Err(RecordDecodeError);
                }
            }
            (12, 27 | 28) => decode_sorted_text_set(cursor)?,

            (50, 16) => {
                let value = cursor.uint().map_err(|_| RecordDecodeError)?;
                if !matches!(value, 1 | 2) {
                    return Err(RecordDecodeError);
                }
                disposition = Some(value);
            }
            (50, 17 | 21 | 22 | 31) => decode_journal_reference_value(cursor)?,
            (50, 18 | 23 | 24) => decode_record_id_value(cursor)?,
            (50, 19 | 20 | 26 | 27) => decode_sorted_journal_reference_set(cursor)?,
            (50, 25) => decode_formal_support_binding(cursor)?,
            (50, 28) => {
                failed_candidate_record_roles = decode_failed_candidate_record_set(cursor, 5..=14)?;
            }
            (50, 29) => {
                failed_candidate_journal_roles =
                    decode_failed_candidate_journal_set(cursor, 5..=14)?;
            }
            (50, 30) => decode_sorted_text_set(cursor)?,

            (62, 16..=19) => decode_record_id_value(cursor)?,
            (62, 20 | 21) => decode_sorted_uint_set(cursor)?,
            (62, 22) => decode_journal_reference_value(cursor)?,

            (71, 16) => {
                cursor.bstr_32().map_err(|_| RecordDecodeError)?;
            }
            (71, 17 | 20) => decode_journal_reference_value(cursor)?,
            (71, 18 | 19) => decode_record_id_value(cursor)?,

            (72, 16) => {
                cursor.bstr_32().map_err(|_| RecordDecodeError)?;
            }
            (72, 17) => decode_journal_reference_value(cursor)?,
            (72, 18) => {
                let value = cursor.uint().map_err(|_| RecordDecodeError)?;
                if !matches!(value, 1..=3) {
                    return Err(RecordDecodeError);
                }
                terminal_disposition = Some(value);
            }
            (72, 19) => {
                if !matches!(cursor.uint().map_err(|_| RecordDecodeError)?, 1..=4) {
                    return Err(RecordDecodeError);
                }
                observed_eviction_state_present = true;
            }
            (72, 20) => decode_sorted_text_set(cursor)?,

            (80, 16 | 18) => {
                cursor.text().map_err(|_| RecordDecodeError)?;
            }
            (80, 17) => {
                cursor.uint().map_err(|_| RecordDecodeError)?;
            }
            (80, 19 | 20) => decode_record_id_value(cursor)?,
            (80, 21) => decode_journal_reference_value(cursor)?,

            (81, 16 | 24 | 28) => decode_journal_reference_value(cursor)?,
            (81, 17 | 18 | 22 | 23 | 25 | 27) => decode_record_id_value(cursor)?,
            (81, 19 | 20) => {
                if !matches!(cursor.uint().map_err(|_| RecordDecodeError)?, 1..=3) {
                    return Err(RecordDecodeError);
                }
            }
            (81, 21) => decode_sorted_record_id_set(cursor)?,
            (81, 26) => decode_diversity_identity_set(cursor)?,
            (81, 29) => decode_sorted_text_set(cursor)?,

            (82 | 85, 16) => {
                let value = cursor.uint().map_err(|_| RecordDecodeError)?;
                if !matches!(value, 1 | 2) {
                    return Err(RecordDecodeError);
                }
                target_mode = Some(value);
            }
            (82 | 85, 17) => decode_sorted_journal_reference_set(cursor)?,
            (82 | 85, 18 | 19) => decode_record_id_value(cursor)?,
            (82 | 85, 20) => decode_sorted_record_id_set(cursor)?,
            (82 | 85, 21) => decode_sorted_text_set(cursor)?,
            (82 | 85, 22) => decode_journal_reference_value(cursor)?,

            (83, 16) => {
                cursor.bstr_32().map_err(|_| RecordDecodeError)?;
            }
            (83, 17 | 18 | 19 | 20 | 21 | 23 | 24 | 27) => decode_record_id_value(cursor)?,
            (83, 22) => {
                let value = cursor.uint().map_err(|_| RecordDecodeError)?;
                if !matches!(value, 1..=7) {
                    return Err(RecordDecodeError);
                }
                formal_outcome = Some(value);
            }
            (83, 25) => decode_sorted_journal_reference_set(cursor)?,
            (83, 26 | 28) => decode_journal_reference_value(cursor)?,
            (83, 29) => decode_sorted_text_set(cursor)?,

            (84, 16 | 17 | 21) => decode_journal_reference_value(cursor)?,
            (84, 18 | 19) => decode_record_id_value(cursor)?,
            (84, 20) => decode_sorted_record_id_set(cursor)?,

            (86, 16) => decode_record_id_value(cursor)?,
            (86, 17 | 18 | 19 | 20 | 21 | 22 | 24) => decode_sorted_record_id_set(cursor)?,
            (86, 23) => decode_sorted_text_set(cursor)?,
            (86, 25 | 26) => decode_journal_reference_value(cursor)?,

            (87, 16) => {
                let value = cursor.uint().map_err(|_| RecordDecodeError)?;
                if !matches!(value, 1 | 2) {
                    return Err(RecordDecodeError);
                }
                target_mode = Some(value);
            }
            (87, 17) => decode_sorted_journal_reference_set(cursor)?,
            (87, 18 | 19) => decode_record_id_value(cursor)?,
            (87, 20) => decode_sorted_record_id_set(cursor)?,
            (87, 21 | 22) => decode_sorted_text_set(cursor)?,
            (87, 23) => decode_journal_reference_value(cursor)?,

            (88, 16 | 25) => decode_journal_reference_value(cursor)?,
            (88, 17) => {
                if !matches!(cursor.uint().map_err(|_| RecordDecodeError)?, 1..=5) {
                    return Err(RecordDecodeError);
                }
            }
            (88, 18 | 21) => decode_record_id_value(cursor)?,
            (88, 19) => {
                if !matches!(cursor.uint().map_err(|_| RecordDecodeError)?, 1..=5) {
                    return Err(RecordDecodeError);
                }
            }
            (88, 20) => {
                if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
                    return Err(RecordDecodeError);
                }
            }
            (88, 22) => decode_sorted_record_id_set(cursor)?,
            (88, 23) => decode_sorted_journal_reference_set(cursor)?,
            (88, 24) => decode_sorted_text_set(cursor)?,
            _ => return Err(RecordDecodeError),
        }
        if record_type == 12 && key == 20 {
            source_binding_present = true;
        }
        if record_type == 83 && key == 23 {
            counterexample_present = true;
        }
    }
    if !cursor.finished() {
        return Err(RecordDecodeError);
    }
    let bit = |key: u32| 1_u64 << key;
    let required = match record_type {
        5 => bit(16) | bit(17) | bit(19) | bit(20),
        6 => bit(16) | bit(17) | bit(18) | bit(19) | bit(21) | bit(22),
        7 => bit(16) | bit(17) | bit(18) | bit(19) | bit(20),
        12 => {
            bit(16)
                | bit(17)
                | bit(18)
                | bit(19)
                | bit(21)
                | bit(22)
                | bit(24)
                | bit(25)
                | bit(26)
                | bit(27)
                | bit(28)
                | bit(29)
                | bit(30)
                | bit(31)
        }
        50 => bit(16) | bit(17) | bit(18) | bit(19) | bit(20) | bit(22) | bit(30) | bit(31),
        62 => (16..=22).fold(0, |mask, key| mask | bit(key)),
        71 => (16..=20).fold(0, |mask, key| mask | bit(key)),
        72 => bit(16) | bit(17) | bit(18) | bit(20),
        80 => (16..=21).fold(0, |mask, key| mask | bit(key)),
        81 => {
            bit(16)
                | bit(17)
                | bit(18)
                | bit(19)
                | bit(20)
                | bit(21)
                | bit(22)
                | bit(23)
                | bit(24)
                | bit(26)
                | bit(28)
                | bit(29)
        }
        82 | 85 => bit(16) | bit(19) | bit(20) | bit(21) | bit(22),
        83 => {
            bit(16)
                | bit(17)
                | bit(18)
                | bit(19)
                | bit(20)
                | bit(21)
                | bit(22)
                | bit(24)
                | bit(25)
                | bit(28)
                | bit(29)
        }
        84 => (16..=21).fold(0, |mask, key| mask | bit(key)),
        86 => (16..=24).fold(0, |mask, key| mask | bit(key)) | bit(26),
        87 => bit(16) | bit(19) | bit(20) | bit(21) | bit(22) | bit(23),
        88 => (16..=22).fold(0, |mask, key| mask | bit(key)) | bit(24) | bit(25),
        _ => return Err(RecordDecodeError),
    };
    if present & required != required {
        return Err(RecordDecodeError);
    }
    if record_type == 12
        && !matches!(
            (verification_target, source_binding_present),
            (Some(1), false) | (Some(2), true)
        )
    {
        return Err(RecordDecodeError);
    }
    if record_type == 50 && !matches!((event_type, disposition), (500, Some(1)) | (501, Some(2))) {
        return Err(RecordDecodeError);
    }
    if record_type == 50
        && present & bit(24) != 0
        && (failed_candidate_record_roles | failed_candidate_journal_roles) & bit(8) as u16 != 0
    {
        return Err(RecordDecodeError);
    }
    if record_type == 72 {
        let expected = match event_type {
            701 => 1,
            702 => 2,
            703 => 3,
            _ => return Err(RecordDecodeError),
        };
        if terminal_disposition != Some(expected)
            || (expected == 2) != observed_eviction_state_present
        {
            return Err(RecordDecodeError);
        }
    }
    if matches!(record_type, 82 | 85 | 87) {
        let explicit_present = present & bit(17) != 0;
        let predicate_present = present & bit(18) != 0;
        if !matches!(
            (target_mode, explicit_present, predicate_present),
            (Some(1), true, false) | (Some(2), false, true)
        ) {
            return Err(RecordDecodeError);
        }
    }
    if record_type == 83 && (formal_outcome == Some(2)) != counterexample_present {
        return Err(RecordDecodeError);
    }
    Ok(())
}

fn validate_review_admission_record_schema(
    event_type: u16,
    record_bytes: &[u8],
) -> Result<(), RecordDecodeError> {
    let (mut cursor, field_count) = decode_event_record_body(record_bytes, 32)?;
    let mut present = 0_u64;
    let mut disposition = None;
    let mut resolved_roles = 0_u16;
    let mut failed_roles = 0_u16;
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !(16..=25).contains(&key) {
            return Err(RecordDecodeError);
        }
        present |= 1_u64
            .checked_shl(u32::try_from(key).map_err(|_| RecordDecodeError)?)
            .ok_or(RecordDecodeError)?;
        match key {
            16 => {
                let value = cursor.uint().map_err(|_| RecordDecodeError)?;
                if !matches!(value, 1 | 2) {
                    return Err(RecordDecodeError);
                }
                disposition = Some(value);
            }
            17..=19 => {
                decode_journal_reference_value(&mut cursor)?;
                resolved_roles |= 1_u16 << (key - 16);
            }
            20 => {
                failed_roles |= decode_failed_candidate_record_set(&mut cursor, 1..=3)?;
            }
            21 => {
                failed_roles |= decode_failed_candidate_journal_set(&mut cursor, 1..=3)?;
            }
            22 => decode_sorted_text_set(&mut cursor)?,
            23 => decode_journal_reference_value(&mut cursor)?,
            24 => {
                if cursor.bstr_32().map_err(|_| RecordDecodeError)?
                    != TERMINAL_REVIEW_ADMISSION_LEGACY_SELECTOR_SHA256
                {
                    return Err(RecordDecodeError);
                }
            }
            25 => {
                if cursor.bstr_32().map_err(|_| RecordDecodeError)?
                    != TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256
                {
                    return Err(RecordDecodeError);
                }
            }
            _ => return Err(RecordDecodeError),
        }
    }
    let required = (1_u64 << 16) | (1_u64 << 22) | (1_u64 << 23);
    let selected_markers = (1_u64 << 24) | (1_u64 << 25);
    if !cursor.finished()
        || present & required != required
        || (present & selected_markers != 0 && present & selected_markers != selected_markers)
        || !matches!((event_type, disposition), (302, Some(1)) | (303, Some(2)))
    {
        return Err(RecordDecodeError);
    }
    if disposition == Some(1) {
        let accepted_references = (1_u64 << 17) | (1_u64 << 18) | (1_u64 << 19);
        if present & accepted_references != accepted_references
            || present & ((1_u64 << 20) | (1_u64 << 21)) != 0
        {
            return Err(RecordDecodeError);
        }
    } else if resolved_roles & failed_roles != 0 {
        return Err(RecordDecodeError);
    }
    Ok(())
}

fn decode_nonempty_policy_verification_requirements(
    cursor: &mut CborCursor<'_>,
) -> Result<(), RecordDecodeError> {
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count == 0 || count > cursor.remaining() / 4 {
        return Err(RecordDecodeError);
    }
    let mut previous = None;
    for _ in 0..count {
        cursor.map_exact(4).map_err(|_| RecordDecodeError)?;
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        let phase = cursor.uint().map_err(|_| RecordDecodeError)?;
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        let target = cursor.uint().map_err(|_| RecordDecodeError)?;
        cursor.key(2).map_err(|_| RecordDecodeError)?;
        let authority = cursor.uint().map_err(|_| RecordDecodeError)?;
        cursor.key(3).map_err(|_| RecordDecodeError)?;
        let required_count = cursor.uint().map_err(|_| RecordDecodeError)?;
        let value = (phase, target, authority, required_count);
        if !matches!(phase, 1 | 2)
            || !matches!(target, 1 | 2)
            || authority != 1
            || previous.is_some_and(|prior| prior >= value)
        {
            return Err(RecordDecodeError);
        }
        previous = Some(value);
    }
    Ok(())
}

fn decode_nonempty_policy_diversity_requirements(
    cursor: &mut CborCursor<'_>,
) -> Result<(), RecordDecodeError> {
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count == 0 || count > cursor.remaining() / 2 {
        return Err(RecordDecodeError);
    }
    let mut previous_dimension = None;
    for _ in 0..count {
        cursor.map_exact(2).map_err(|_| RecordDecodeError)?;
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        let dimension = cursor.uint().map_err(|_| RecordDecodeError)?;
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        let _distinct_count = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !matches!(dimension, 1 | 2) || previous_dimension.is_some_and(|prior| prior >= dimension)
        {
            return Err(RecordDecodeError);
        }
        previous_dimension = Some(dimension);
    }
    Ok(())
}

fn decode_nonempty_policy_closeout_postconditions(
    cursor: &mut CborCursor<'_>,
) -> Result<(), RecordDecodeError> {
    cursor.map_exact(4).map_err(|_| RecordDecodeError)?;
    for (key, expected) in [(0, 1), (1, 1), (2, 2), (3, 2)] {
        cursor.key(key).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != expected {
            return Err(RecordDecodeError);
        }
    }
    Ok(())
}

pub(super) fn validate_policy_record_schema(record_bytes: &[u8]) -> Result<(), RecordDecodeError> {
    let (mut cursor, field_count) = decode_event_record_body(record_bytes, 40)?;
    let mut present = 0_u64;
    let mut supported_contexts = None;
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !(16..=30).contains(&key) {
            return Err(RecordDecodeError);
        }
        present |= 1_u64
            .checked_shl(u32::try_from(key).map_err(|_| RecordDecodeError)?)
            .ok_or(RecordDecodeError)?;
        match key {
            16 | 25 => decode_record_id_value(&mut cursor)?,
            17 => {
                let requirements = decode_review_admission_review_requirements(&mut cursor)?;
                if requirements
                    .iter()
                    .any(|requirement| !(1..=5).contains(&requirement.review_role_id()))
                {
                    return Err(RecordDecodeError);
                }
            }
            18 | 19 => {
                decode_nonempty_sorted_uint_set(&mut cursor, |value| matches!(value, 1..=3))?;
            }
            20 => decode_nonempty_policy_verification_requirements(&mut cursor)?,
            21 => {
                cursor.map_exact(1).map_err(|_| RecordDecodeError)?;
                cursor.key(0).map_err(|_| RecordDecodeError)?;
                decode_sorted_event_type_id_set(&mut cursor)?;
            }
            22 => {
                cursor.uint().map_err(|_| RecordDecodeError)?;
            }
            23 => {
                cursor.map_exact(4).map_err(|_| RecordDecodeError)?;
                for key in 0..=3 {
                    cursor.key(key).map_err(|_| RecordDecodeError)?;
                    cursor.bool().map_err(|_| RecordDecodeError)?;
                }
            }
            24 => {
                cursor.map_exact(1).map_err(|_| RecordDecodeError)?;
                cursor.key(0).map_err(|_| RecordDecodeError)?;
                decode_nonempty_sorted_uint_set(&mut cursor, |value| matches!(value, 1..=6))?;
            }
            26 => {
                let fields = cursor.map().map_err(|_| RecordDecodeError)?;
                if !matches!(fields, 1 | 3) {
                    return Err(RecordDecodeError);
                }
                cursor.key(0).map_err(|_| RecordDecodeError)?;
                let required = cursor.bool().map_err(|_| RecordDecodeError)?;
                if required != (fields == 3) {
                    return Err(RecordDecodeError);
                }
                if required {
                    cursor.key(1).map_err(|_| RecordDecodeError)?;
                    decode_nonempty_sorted_uint_set(&mut cursor, |value| matches!(value, 1..=5))?;
                    cursor.key(2).map_err(|_| RecordDecodeError)?;
                    let count = cursor.array().map_err(|_| RecordDecodeError)?;
                    if count == 0 {
                        return Err(RecordDecodeError);
                    }
                    let mut previous = None;
                    for _ in 0..count {
                        let identity = cursor.bstr_32().map_err(|_| RecordDecodeError)?;
                        if previous.is_some_and(|prior| prior >= identity) {
                            return Err(RecordDecodeError);
                        }
                        previous = Some(identity);
                    }
                }
            }
            27 => decode_nonempty_policy_diversity_requirements(&mut cursor)?,
            28 => decode_nonempty_policy_closeout_postconditions(&mut cursor)?,
            29 => decode_journal_reference_value(&mut cursor)?,
            30 => {
                supported_contexts = Some(decode_nonempty_sorted_uint_set(&mut cursor, |value| {
                    matches!(value, 1..=5)
                })?);
            }
            _ => return Err(RecordDecodeError),
        }
    }
    let required = (1_u64 << 16) | (1_u64 << 29) | (1_u64 << 30);
    let supported_contexts = supported_contexts.ok_or(RecordDecodeError)?;
    if !cursor.finished() || present & required != required {
        return Err(RecordDecodeError);
    }
    let mut context_mask = 0_u64;
    for context in supported_contexts {
        context_mask |= 1_u64 << context;
    }
    let field_contexts = [
        (17, (1_u64 << 2) | (1_u64 << 3) | (1_u64 << 5)),
        (18, (1_u64 << 2) | (1_u64 << 3) | (1_u64 << 4)),
        (19, (1_u64 << 2) | (1_u64 << 3) | (1_u64 << 4)),
        (20, 1_u64 << 3),
        (21, 1_u64 << 3),
        (22, 1_u64 << 3),
        (23, 1_u64 << 1),
        (24, 1_u64 << 2),
        (25, 1_u64 << 3),
        (26, 1_u64 << 3),
        (27, 1_u64 << 3),
        (28, (1_u64 << 3) | (1_u64 << 4)),
    ];
    if field_contexts
        .iter()
        .any(|(key, applicable)| present & (1_u64 << key) != 0 && context_mask & applicable == 0)
    {
        return Err(RecordDecodeError);
    }
    let review_required = context_mask & ((1_u64 << 2) | (1_u64 << 5)) != 0;
    if review_required && present & (1_u64 << 17) == 0
        || context_mask & (1_u64 << 4) != 0 && present & (1_u64 << 28) == 0
        || present & (1_u64 << 28) != 0
            && context_mask & ((1_u64 << 3) | (1_u64 << 4)) != ((1_u64 << 3) | (1_u64 << 4))
    {
        return Err(RecordDecodeError);
    }
    Ok(())
}

fn validate_event_record_type_local_schema(
    event_type: u16,
    record_bytes: &[u8],
) -> Result<(), RecordDecodeError> {
    match event_type {
        1 => GenesisRecord::decode_authoritative(record_bytes).map(|_| ()),
        100 => FreezeAttemptStartRecord::decode_authoritative(record_bytes).map(|_| ()),
        101 => FreezeReceiptRecord::decode_authoritative(record_bytes).map(|_| ()),
        300 => ReviewRequestRecord::decode_authoritative(record_bytes).map(|_| ()),
        301 => ReviewResultRecord::decode_authoritative(record_bytes).map(|_| ()),
        302 | 303 => validate_review_admission_record_schema(event_type, record_bytes),
        400 => validate_policy_record_schema(record_bytes),
        102..=104 | 200 | 500..=501 | 600 | 700..=703 | 800..=808 => {
            let record_type =
                expected_record_type_for_event(event_type).ok_or(RecordDecodeError)?;
            let (mut cursor, field_count) = decode_event_record_body(record_bytes, record_type)?;
            validate_remaining_event_record_fields(
                record_type,
                event_type,
                &mut cursor,
                field_count,
            )
        }
        _ => Err(RecordDecodeError),
    }
}

fn validate_event_record_lifecycle_binding(
    entry: &RetainedJournalEntry,
    record_bytes: &[u8],
) -> Result<(), RecordDecodeError> {
    let event_type = entry.event_type_id().value();
    match event_type {
        102..=104 | 700..=703 => {
            let record_type =
                expected_record_type_for_event(event_type).ok_or(RecordDecodeError)?;
            let (mut cursor, field_count) = decode_event_record_body(record_bytes, record_type)?;
            let mut lifecycle_object_id = None;
            for _ in 0..field_count {
                let key = cursor.uint().map_err(|_| RecordDecodeError)?;
                if key == 16 {
                    lifecycle_object_id = Some(cursor.bstr_32().map_err(|_| RecordDecodeError)?);
                } else {
                    cursor.skip_value().map_err(|_| RecordDecodeError)?;
                }
            }
            if lifecycle_object_id != Some(entry.lifecycle_object_id()) {
                return Err(RecordDecodeError);
            }
        }
        600 => {
            if entry.lifecycle_object_id() != *entry.registry_id().as_bytes() {
                return Err(RecordDecodeError);
            }
        }
        200 | 302..=303 | 400 | 500..=501 | 800..=808
            if entry.lifecycle_object_id() != *entry.event_record_id().as_bytes() =>
        {
            return Err(RecordDecodeError);
        }
        _ => {}
    }
    Ok(())
}

fn validate_redundant_event_specific_bindings(
    entry: &RetainedJournalEntry,
    record_bytes: &[u8],
) -> Result<(), RecordDecodeError> {
    let RetainedJournalEntry::Common(common_entry) = entry else {
        return Ok(());
    };
    match entry.event_type_id().value() {
        104 => {
            let common = decode_journal_entry_with_event_specific_keys(
                &common_entry.authoritative_bytes,
                &[16, 18, 19],
            )
            .map_err(|_| RecordDecodeError)?;
            let [(_, DecodedEventSpecificField::Bytes(journal_attempt)), (_, DecodedEventSpecificField::JournalReference(journal_start)), (_, DecodedEventSpecificField::JournalReference(journal_terminal))] =
                common.event_specific_fields.as_slice()
            else {
                return Err(RecordDecodeError);
            };
            let (mut cursor, field_count) = decode_event_record_body(record_bytes, 7)?;
            let mut record_attempt = None;
            let mut record_start = None;
            let mut record_terminal = None;
            for _ in 0..field_count {
                let key = cursor.uint().map_err(|_| RecordDecodeError)?;
                match key {
                    16 => record_attempt = Some(cursor.bstr_32().map_err(|_| RecordDecodeError)?),
                    17 => {
                        record_start = Some(
                            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?,
                        )
                    }
                    18 => {
                        record_terminal = Some(
                            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?,
                        )
                    }
                    _ => cursor.skip_value().map_err(|_| RecordDecodeError)?,
                }
            }
            if record_attempt.as_ref() != Some(journal_attempt)
                || record_start.as_ref() != Some(journal_start)
                || record_terminal.as_ref() != Some(journal_terminal)
            {
                return Err(RecordDecodeError);
            }
        }
        200 => {
            let journal_bracket = decode_journal_entry_with_event_specific_keys(
                &common_entry.authoritative_bytes,
                &[20],
            )
            .ok()
            .and_then(|common| match common.event_specific_fields.as_slice() {
                [(_, DecodedEventSpecificField::JournalReference(reference))] => {
                    Some(reference.clone())
                }
                _ => None,
            });
            let (mut cursor, field_count) = decode_event_record_body(record_bytes, 12)?;
            let mut record_bracket = None;
            for _ in 0..field_count {
                let key = cursor.uint().map_err(|_| RecordDecodeError)?;
                if key == 32 {
                    record_bracket =
                        Some(decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?);
                } else {
                    cursor.skip_value().map_err(|_| RecordDecodeError)?;
                }
            }
            if record_bracket != journal_bracket {
                return Err(RecordDecodeError);
            }
        }
        700 => {
            let common = decode_journal_entry_with_event_specific_keys(
                &common_entry.authoritative_bytes,
                &[21, 22, 23, 24],
            )
            .map_err(|_| RecordDecodeError)?;
            let [(_, DecodedEventSpecificField::Bytes(journal_attempt)), (_, DecodedEventSpecificField::JournalReference(journal_freeze)), (_, DecodedEventSpecificField::Bytes(journal_manifest)), (_, DecodedEventSpecificField::Bytes(journal_scope))] =
                common.event_specific_fields.as_slice()
            else {
                return Err(RecordDecodeError);
            };
            let (mut cursor, field_count) = decode_event_record_body(record_bytes, 71)?;
            let mut record_attempt = None;
            let mut record_freeze = None;
            let mut record_manifest = None;
            let mut record_scope = None;
            for _ in 0..field_count {
                let key = cursor.uint().map_err(|_| RecordDecodeError)?;
                match key {
                    16 => record_attempt = Some(cursor.bstr_32().map_err(|_| RecordDecodeError)?),
                    17 => {
                        record_freeze = Some(
                            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?,
                        )
                    }
                    18 => record_manifest = Some(cursor.bstr_32().map_err(|_| RecordDecodeError)?),
                    19 => record_scope = Some(cursor.bstr_32().map_err(|_| RecordDecodeError)?),
                    _ => cursor.skip_value().map_err(|_| RecordDecodeError)?,
                }
            }
            if record_attempt.as_ref() != Some(journal_attempt)
                || record_freeze.as_ref() != Some(journal_freeze)
                || record_manifest.as_ref() != Some(journal_manifest)
                || record_scope.as_ref() != Some(journal_scope)
            {
                return Err(RecordDecodeError);
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_exact_authority_dependencies(
    entry: &RetainedJournalEntry,
    expected: &[JournalReference],
) -> Result<(), RecordDecodeError> {
    let actual = entry.authority_dependencies();
    let mut matched = Vec::new();
    matched
        .try_reserve_exact(actual.len())
        .map_err(|_| RecordDecodeError)?;
    matched.resize(actual.len(), false);
    for expected_reference in expected {
        let index = actual
            .binary_search_by(|candidate| {
                (candidate.entry_index(), candidate.entry_hash()).cmp(&(
                    expected_reference.entry_index(),
                    expected_reference.entry_hash(),
                ))
            })
            .map_err(|_| RecordDecodeError)?;
        if actual[index] != *expected_reference {
            return Err(RecordDecodeError);
        }
        matched[index] = true;
    }
    if matched.iter().all(|matched| *matched) {
        Ok(())
    } else {
        Err(RecordDecodeError)
    }
}

fn validate_resolved_prior_reference(
    retained_journal: &RetainedJournal,
    entry: &RetainedJournalEntry,
    reference: &JournalReference,
) -> Result<(), RecordDecodeError> {
    let resolved = retained_journal
        .resolve_reference(reference)
        .map_err(|_| RecordDecodeError)?;
    if resolved.entry_index().value() >= entry.entry_index().value() {
        return Err(RecordDecodeError);
    }
    Ok(())
}

fn decode_record_id_for_binding(
    cursor: &mut CborCursor<'_>,
) -> Result<RecordId, RecordDecodeError> {
    RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
        .map_err(|_| RecordDecodeError)
}

fn decode_record_id_set_for_binding(
    cursor: &mut CborCursor<'_>,
) -> Result<Vec<RecordId>, RecordDecodeError> {
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count > cursor.remaining() / 34 {
        return Err(RecordDecodeError);
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| RecordDecodeError)?;
    for _ in 0..count {
        values.push(decode_record_id_for_binding(cursor)?);
    }
    Ok(values)
}

fn decode_journal_reference_set_for_binding(
    cursor: &mut CborCursor<'_>,
) -> Result<Vec<JournalReference>, RecordDecodeError> {
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count > cursor.remaining() / 105 {
        return Err(RecordDecodeError);
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| RecordDecodeError)?;
    for _ in 0..count {
        values.push(decode_journal_reference(cursor).map_err(|_| RecordDecodeError)?);
    }
    Ok(values)
}

fn require_reference_event_type(
    reference: &JournalReference,
    allowed_event_types: &[u16],
) -> Result<(), RecordDecodeError> {
    if !allowed_event_types.contains(&reference.event_type_id().value()) {
        return Err(RecordDecodeError);
    }
    Ok(())
}

fn decode_typed_journal_reference_for_binding(
    cursor: &mut CborCursor<'_>,
    allowed_event_types: &[u16],
) -> Result<JournalReference, RecordDecodeError> {
    let reference = decode_journal_reference(cursor).map_err(|_| RecordDecodeError)?;
    require_reference_event_type(&reference, allowed_event_types)?;
    Ok(reference)
}

fn decode_typed_journal_reference_set_for_binding(
    cursor: &mut CborCursor<'_>,
    allowed_event_types: &[u16],
) -> Result<Vec<JournalReference>, RecordDecodeError> {
    let references = decode_journal_reference_set_for_binding(cursor)?;
    for reference in &references {
        require_reference_event_type(reference, allowed_event_types)?;
    }
    Ok(references)
}

fn validate_exact_identity_dependencies(
    entry: &RetainedJournalEntry,
    mut expected: Vec<RecordId>,
) -> Result<(), RecordDecodeError> {
    expected.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    expected.dedup_by(|left, right| left.as_bytes() == right.as_bytes());
    if entry.identity_dependencies().len() != expected.len() {
        return Err(RecordDecodeError);
    }
    for dependency in entry.identity_dependencies() {
        let IdentityDependency::RecordId(record_id) = dependency else {
            return Err(RecordDecodeError);
        };
        if expected
            .binary_search_by(|candidate| candidate.as_bytes().cmp(record_id.as_bytes()))
            .is_err()
        {
            return Err(RecordDecodeError);
        }
    }
    Ok(())
}

fn validate_exact_typed_identity_dependencies(
    entry: &RetainedJournalEntry,
    expected: &[IdentityDependency],
) -> Result<(), RecordDecodeError> {
    if expected
        .iter()
        .any(|dependency| !entry.identity_dependencies().contains(dependency))
        || entry
            .identity_dependencies()
            .iter()
            .any(|dependency| !expected.contains(dependency))
    {
        return Err(RecordDecodeError);
    }
    Ok(())
}

fn decode_policy_supported_contexts(record_bytes: &[u8]) -> Result<Vec<u64>, RecordDecodeError> {
    let (mut cursor, field_count) = decode_event_record_body(record_bytes, 40)?;
    let mut supported_contexts = None;
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        if key == 30 {
            let count = cursor.array().map_err(|_| RecordDecodeError)?;
            if count == 0 || count > cursor.remaining() {
                return Err(RecordDecodeError);
            }
            let mut contexts = Vec::with_capacity(count);
            for _ in 0..count {
                contexts.push(cursor.uint().map_err(|_| RecordDecodeError)?);
            }
            supported_contexts = Some(contexts);
        } else {
            cursor.skip_value().map_err(|_| RecordDecodeError)?;
        }
    }
    supported_contexts.ok_or(RecordDecodeError)
}

fn validate_freeze_commit_policy_requirements(
    receipt_record_bytes: &[u8],
    records: &[(RecordId, Vec<u8>)],
) -> Result<(), RecordDecodeError> {
    let receipt = FreezeReceiptRecord::decode_authoritative(receipt_record_bytes)?;
    let policy_record_id = receipt.input().policy_record_id;
    let policy_bytes = records
        .binary_search_by(|(stored_id, _)| stored_id.as_bytes().cmp(policy_record_id.as_bytes()))
        .ok()
        .map(|index| records[index].1.as_slice())
        .ok_or(RecordDecodeError)?;
    let policy_frame = StrictRecordFrame::decode_authoritative(policy_bytes)?;
    if policy_frame.record_id() != policy_record_id {
        return Err(RecordDecodeError);
    }
    validate_policy_record_schema(policy_bytes)?;

    let (mut cursor, field_count) = decode_event_record_body(policy_bytes, 40)?;
    let mut contexts = None;
    let mut minimum_durability = None;
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        match key {
            23 => {
                cursor.map_exact(4).map_err(|_| RecordDecodeError)?;
                let mut requirements = [false; 4];
                for (requirement_key, requirement) in requirements.iter_mut().enumerate() {
                    cursor
                        .key(u64::try_from(requirement_key).map_err(|_| RecordDecodeError)?)
                        .map_err(|_| RecordDecodeError)?;
                    *requirement = cursor.bool().map_err(|_| RecordDecodeError)?;
                }
                minimum_durability = Some(requirements);
            }
            30 => {
                contexts = Some(decode_nonempty_sorted_uint_set(&mut cursor, |value| {
                    matches!(value, 1..=5)
                })?);
            }
            _ => cursor.skip_value().map_err(|_| RecordDecodeError)?,
        }
    }
    if !cursor.finished() || !contexts.ok_or(RecordDecodeError)?.contains(&1) {
        return Err(RecordDecodeError);
    }
    if let Some(
        [require_file_content_flush, require_atomic_publish_no_replace, require_parent_directory_flush, require_platform_strongest_available],
    ) = minimum_durability
    {
        if require_file_content_flush && receipt.input().file_content_flush_state != 1
            || require_atomic_publish_no_replace
                && receipt.input().atomic_publish_no_replace_state != 1
            || require_parent_directory_flush && receipt.input().parent_directory_flush_state != 1
            || require_platform_strongest_available && !receipt.input().platform_strongest_available
        {
            return Err(RecordDecodeError);
        }
    }
    Ok(())
}

fn validate_review_request_replay_contract(
    retained_journal: &RetainedJournal,
    entry: &RetainedJournalEntry,
    request: &ReviewRequestRecord,
    records: &[(RecordId, Vec<u8>)],
) -> Result<(), RecordDecodeError> {
    if request.freeze_authority_ref().event_type_id().value() != 101
        || request.policy_authority_ref().event_type_id().value() != 400
    {
        return Err(RecordDecodeError);
    }
    let authorities = [
        request.freeze_authority_ref().clone(),
        request.policy_authority_ref().clone(),
    ];
    validate_exact_authority_dependencies(entry, &authorities)?;
    let identities = [
        IdentityDependency::RecordId(request.manifest_id()),
        IdentityDependency::RecordId(request.required_checks_ref()),
        IdentityDependency::RecordId(request.review_scope_ref()),
        IdentityDependency::RecordId(request.review_method_ref()),
        IdentityDependency::JournalAnchorId(request.review_package_anchor_id()),
    ];
    validate_exact_typed_identity_dependencies(entry, &identities)?;
    for reference in authorities
        .iter()
        .chain([request.operation_start_journal_ref()])
    {
        validate_resolved_prior_reference(retained_journal, entry, reference)?;
    }
    let policy_contexts = decode_policy_supported_contexts(record_bytes_for_reference(
        records,
        request.policy_authority_ref(),
    )?)?;
    if !policy_contexts.contains(&5) {
        return Err(RecordDecodeError);
    }
    let policy = ReviewAdmissionPolicyRecord::decode_authoritative(record_bytes_for_reference(
        records,
        request.policy_authority_ref(),
    )?)?;
    if !policy.review_requirements().iter().any(|requirement| {
        requirement.review_role_id() == request.review_role_id()
            && requirement.review_scope_ref() == request.review_scope_ref()
            && requirement.review_method_ref() == request.review_method_ref()
            && requirement.required_checks_ref() == request.required_checks_ref()
    }) {
        return Err(RecordDecodeError);
    }
    Ok(())
}

fn validate_review_result_replay_contract(
    retained_journal: &RetainedJournal,
    entry: &RetainedJournalEntry,
    result: &ReviewResultRecord,
    records: &[(RecordId, Vec<u8>)],
) -> Result<(), RecordDecodeError> {
    if result
        .review_request_authority_ref()
        .event_type_id()
        .value()
        != 300
    {
        return Err(RecordDecodeError);
    }
    let authorities = [result.review_request_authority_ref().clone()];
    validate_exact_authority_dependencies(entry, &authorities)?;
    let mut identities = vec![
        IdentityDependency::RecordId(result.manifest_id()),
        IdentityDependency::RecordId(result.review_scope_ref()),
        IdentityDependency::RecordId(result.review_method_ref()),
        IdentityDependency::JournalAnchorId(result.review_package_anchor_id()),
    ];
    identities.extend(
        result
            .findings()
            .iter()
            .copied()
            .map(IdentityDependency::RecordId),
    );
    validate_exact_typed_identity_dependencies(entry, &identities)?;
    for reference in authorities
        .iter()
        .chain([result.operation_start_journal_ref()])
    {
        validate_resolved_prior_reference(retained_journal, entry, reference)?;
    }
    let request = ReviewRequestRecord::decode_authoritative(record_bytes_for_reference(
        records,
        result.review_request_authority_ref(),
    )?)?;
    if result.freeze_authority_ref() != request.freeze_authority_ref()
        || result.manifest_id() != request.manifest_id()
        || result.review_role_id() != request.review_role_id()
        || result.review_scope_ref() != request.review_scope_ref()
        || result.review_method_ref() != request.review_method_ref()
        || result.review_package_anchor_id() != request.review_package_anchor_id()
    {
        return Err(RecordDecodeError);
    }
    Ok(())
}

fn record_bytes_for_reference<'a>(
    records: &'a [(RecordId, Vec<u8>)],
    reference: &JournalReference,
) -> Result<&'a [u8], RecordDecodeError> {
    let record_id = RecordId::try_from(reference.event_record_id().as_bytes().as_slice())
        .map_err(|_| RecordDecodeError)?;
    records
        .binary_search_by(|(stored_id, _)| stored_id.as_bytes().cmp(record_id.as_bytes()))
        .ok()
        .map(|index| records[index].1.as_slice())
        .ok_or(RecordDecodeError)
}

struct RecordNamespaceResolver<'a>(&'a [(RecordId, Vec<u8>)]);

impl ExactRecordByteResolver for RecordNamespaceResolver<'_> {
    fn resolve(&self, record_id: RecordId) -> Option<&[u8]> {
        self.0
            .binary_search_by(|(stored_id, _)| stored_id.as_bytes().cmp(record_id.as_bytes()))
            .ok()
            .map(|index| self.0[index].1.as_slice())
    }
}

fn decode_capability_observation_provenance(
    record_bytes: &[u8],
) -> Result<(RecordId, RecordId, bool), RecordDecodeError> {
    let (mut cursor, field_count) = decode_event_record_body(record_bytes, 63)?;
    if !(4..=8).contains(&field_count) {
        return Err(RecordDecodeError);
    }
    let mut storage = None;
    let mut environment = None;
    let mut cache_reused = None;
    let mut process_scope_present = false;
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        match key {
            16 => storage = Some(decode_record_id_for_binding(&mut cursor)?),
            17 => environment = Some(decode_record_id_for_binding(&mut cursor)?),
            18 => cache_reused = Some(cursor.bool().map_err(|_| RecordDecodeError)?),
            19 => {
                cursor.bstr().map_err(|_| RecordDecodeError)?;
                process_scope_present = true;
            }
            20..=22 => {
                cursor.bstr().map_err(|_| RecordDecodeError)?;
            }
            23 => {
                cursor.text().map_err(|_| RecordDecodeError)?;
            }
            _ => return Err(RecordDecodeError),
        }
    }
    if !cursor.finished() || !process_scope_present {
        return Err(RecordDecodeError);
    }
    Ok((
        storage.ok_or(RecordDecodeError)?,
        environment.ok_or(RecordDecodeError)?,
        cache_reused.ok_or(RecordDecodeError)?,
    ))
}

fn validate_event_record_reference_bindings(
    retained_journal: &RetainedJournal,
    entry: &RetainedJournalEntry,
    record_bytes: &[u8],
    records: &[(RecordId, Vec<u8>)],
) -> Result<(), RecordDecodeError> {
    let event_type = entry.event_type_id().value();
    if !matches!(event_type, 102..=104 | 200 | 600 | 700..=703 | 800..=808) {
        return Ok(());
    }
    let record_type = expected_record_type_for_event(event_type).ok_or(RecordDecodeError)?;
    let (mut cursor, field_count) = decode_event_record_body(record_bytes, record_type)?;
    let mut authorities = Vec::new();
    let mut identities = Vec::new();
    let mut other_references = Vec::new();
    let mut prior_storage = None;
    let mut prior_environment = None;
    let mut new_storage = None;
    let mut new_environment = None;
    let mut capability_provenance = None;
    let mut capability_epoch = None;
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        match (event_type, key) {
            (102 | 103, 17) | (104, 17) => authorities.push(
                decode_typed_journal_reference_for_binding(&mut cursor, &[100])?,
            ),
            (104, 18) => authorities.push(decode_typed_journal_reference_for_binding(
                &mut cursor,
                &[101, 102, 103],
            )?),
            (700, 17) => authorities.push(decode_typed_journal_reference_for_binding(
                &mut cursor,
                &[101],
            )?),
            (701..=703, 17) => authorities.push(decode_typed_journal_reference_for_binding(
                &mut cursor,
                &[700],
            )?),
            (801, 16) => authorities.push(decode_typed_journal_reference_for_binding(
                &mut cursor,
                &[800],
            )?),
            (803, 26) => authorities.push(decode_typed_journal_reference_for_binding(
                &mut cursor,
                &[101],
            )?),
            (804, 16 | 17) => authorities.push(decode_typed_journal_reference_for_binding(
                &mut cursor,
                &[800],
            )?),
            (806, 25) => authorities.push(decode_typed_journal_reference_for_binding(
                &mut cursor,
                &[806],
            )?),
            (808, 16) => authorities.push(decode_typed_journal_reference_for_binding(
                &mut cursor,
                &[803],
            )?),
            (802, 17) => authorities.extend(decode_typed_journal_reference_set_for_binding(
                &mut cursor,
                &[801],
            )?),
            (805, 17) => authorities.extend(decode_typed_journal_reference_set_for_binding(
                &mut cursor,
                &[804],
            )?),
            (807, 17) => authorities.extend(decode_typed_journal_reference_set_for_binding(
                &mut cursor,
                &[806],
            )?),
            (803, 25) => authorities.extend(decode_typed_journal_reference_set_for_binding(
                &mut cursor,
                &[800],
            )?),
            (808, 23) => {
                authorities.extend(decode_journal_reference_set_for_binding(&mut cursor)?);
            }
            (102, 20)
            | (103, 22)
            | (200, 18)
            | (600, 22)
            | (700, 20)
            | (800, 21)
            | (801, 28)
            | (802, 22)
            | (803, 28)
            | (804, 21)
            | (805, 22)
            | (806, 26)
            | (807, 23)
            | (808, 25) => {
                other_references
                    .push(decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?);
            }
            (801, 24) => {
                capability_epoch =
                    Some(decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?);
            }
            (200, 19) => authorities.push(decode_typed_journal_reference_for_binding(
                &mut cursor,
                &[101],
            )?),
            (200, 32) => authorities.push(decode_typed_journal_reference_for_binding(
                &mut cursor,
                &[500],
            )?),
            (102, 18)
            | (200, 20 | 21 | 29 | 30)
            | (700, 18 | 19)
            | (800, 19 | 20)
            | (801, 17 | 18 | 27)
            | (802 | 805, 18 | 19)
            | (803, 17 | 18 | 19 | 20 | 21 | 23 | 24 | 27)
            | (804, 18 | 19)
            | (806, 16)
            | (807, 18 | 19)
            | (808, 18 | 21) => {
                identities.push(decode_record_id_for_binding(&mut cursor)?);
            }
            (801, 21)
            | (802 | 805 | 807, 20)
            | (804, 20)
            | (806, 17 | 18 | 19 | 20 | 21 | 22 | 24)
            | (808, 22) => {
                identities.extend(decode_record_id_set_for_binding(&mut cursor)?);
            }
            (600, 16..=19) => {
                let record_id = decode_record_id_for_binding(&mut cursor)?;
                match key {
                    16 => prior_storage = Some(record_id),
                    17 => new_storage = Some(record_id),
                    18 => prior_environment = Some(record_id),
                    19 => new_environment = Some(record_id),
                    _ => unreachable!(),
                }
                identities.push(record_id);
            }
            (801, 22 | 23) => {
                let record_id = decode_record_id_for_binding(&mut cursor)?;
                if key == 22 {
                    new_storage = Some(record_id);
                } else {
                    new_environment = Some(record_id);
                }
                identities.push(record_id);
            }
            (801, 25) => {
                let record_id = decode_record_id_for_binding(&mut cursor)?;
                capability_provenance = Some(record_id);
                identities.push(record_id);
            }
            _ => cursor.skip_value().map_err(|_| RecordDecodeError)?,
        }
    }
    validate_exact_authority_dependencies(entry, &authorities)?;
    if event_type != 600 || !entry.identity_dependencies().is_empty() {
        validate_exact_identity_dependencies(entry, identities)?;
    }
    for reference in authorities.iter().chain(other_references.iter()) {
        validate_resolved_prior_reference(retained_journal, entry, reference)?;
    }
    if matches!(event_type, 600 | 801)
        && (new_storage != Some(entry.storage_capability_class_id())
            || new_environment != Some(entry.environment_observation_id()))
    {
        return Err(RecordDecodeError);
    }
    if event_type == 600 {
        let predecessor_index = entry
            .entry_index()
            .value()
            .checked_sub(1)
            .ok_or(RecordDecodeError)?;
        let predecessor = retained_journal
            .entries
            .iter()
            .find(|candidate| candidate.entry_index().value() == predecessor_index)
            .ok_or(RecordDecodeError)?;
        if entry.previous_entry_hash() != Some(predecessor.entry_hash())
            || prior_storage != Some(predecessor.storage_capability_class_id())
            || prior_environment != Some(predecessor.environment_observation_id())
        {
            return Err(RecordDecodeError);
        }
    }
    if event_type == 801 {
        let epoch_reference = capability_epoch.as_ref().ok_or(RecordDecodeError)?;
        if !matches!(epoch_reference.event_type_id().value(), 1 | 600) {
            return Err(RecordDecodeError);
        }
        let applicable_epoch = retained_journal
            .entries
            .iter()
            .rev()
            .find(|candidate| {
                candidate.entry_index().value() < entry.entry_index().value()
                    && candidate.event_type_id().value() == 600
            })
            .or_else(|| retained_journal.entries.first())
            .ok_or(RecordDecodeError)?;
        let applicable_epoch_reference = JournalReference::new(
            applicable_epoch.registry_id(),
            applicable_epoch.entry_index(),
            applicable_epoch.entry_hash(),
            applicable_epoch.event_type_id(),
            applicable_epoch.event_record_id(),
        );
        if *epoch_reference != applicable_epoch_reference {
            return Err(RecordDecodeError);
        }
        let epoch = retained_journal
            .resolve_reference(epoch_reference)
            .map_err(|_| RecordDecodeError)?;
        if epoch.entry_index().value() >= entry.entry_index().value()
            || Some(epoch.storage_capability_class_id()) != new_storage
            || Some(epoch.environment_observation_id()) != new_environment
        {
            return Err(RecordDecodeError);
        }
    }
    if let Some(provenance_id) = capability_provenance {
        let provenance_bytes = records
            .binary_search_by(|(stored_id, _)| stored_id.as_bytes().cmp(provenance_id.as_bytes()))
            .ok()
            .map(|index| records[index].1.as_slice())
            .ok_or(RecordDecodeError)?;
        let (storage, environment, cache_reused) =
            decode_capability_observation_provenance(provenance_bytes)?;
        if !cache_reused || Some(storage) != new_storage || Some(environment) != new_environment {
            return Err(RecordDecodeError);
        }
    }
    Ok(())
}

fn validate_policy_event_bindings(
    retained_journal: &RetainedJournal,
    entry: &RetainedJournalEntry,
    record_bytes: &[u8],
) -> Result<(), RecordDecodeError> {
    let (mut cursor, field_count) = decode_event_record_body(record_bytes, 40)?;
    let mut identities = Vec::new();
    let mut operation_start = None;
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        match key {
            16 | 25 => identities.push(decode_record_id_for_binding(&mut cursor)?),
            17 => {
                let count = cursor.array().map_err(|_| RecordDecodeError)?;
                if count == 0 || count > cursor.remaining() / 5 {
                    return Err(RecordDecodeError);
                }
                for _ in 0..count {
                    cursor.map_exact(5).map_err(|_| RecordDecodeError)?;
                    cursor.key(0).map_err(|_| RecordDecodeError)?;
                    cursor.uint().map_err(|_| RecordDecodeError)?;
                    for nested_key in 1..=3 {
                        cursor.key(nested_key).map_err(|_| RecordDecodeError)?;
                        identities.push(decode_record_id_for_binding(&mut cursor)?);
                    }
                    cursor.key(4).map_err(|_| RecordDecodeError)?;
                    cursor.uint().map_err(|_| RecordDecodeError)?;
                }
            }
            26 => {
                let fields = cursor.map().map_err(|_| RecordDecodeError)?;
                cursor.key(0).map_err(|_| RecordDecodeError)?;
                let required = cursor.bool().map_err(|_| RecordDecodeError)?;
                if required && fields == 3 {
                    cursor.key(1).map_err(|_| RecordDecodeError)?;
                    cursor.skip_value().map_err(|_| RecordDecodeError)?;
                    cursor.key(2).map_err(|_| RecordDecodeError)?;
                    identities.extend(decode_record_id_set_for_binding(&mut cursor)?);
                } else if required || fields != 1 {
                    return Err(RecordDecodeError);
                }
            }
            29 => {
                operation_start =
                    Some(decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?);
            }
            _ => cursor.skip_value().map_err(|_| RecordDecodeError)?,
        }
    }
    validate_exact_authority_dependencies(entry, &[])?;
    validate_exact_identity_dependencies(entry, identities)?;
    validate_resolved_prior_reference(
        retained_journal,
        entry,
        operation_start.as_ref().ok_or(RecordDecodeError)?,
    )?;
    Ok(())
}

#[derive(Clone, Copy)]
struct PolicyCloseoutFacts {
    supports_closeout_creation: bool,
    closeout_postconditions_present: bool,
    bootstrap_scope: Option<RecordId>,
}

fn decode_policy_closeout_facts(
    record_bytes: &[u8],
) -> Result<PolicyCloseoutFacts, RecordDecodeError> {
    let (mut cursor, field_count) = decode_event_record_body(record_bytes, 40)?;
    let mut supports_closeout_creation = false;
    let mut closeout_postconditions_present = false;
    let mut bootstrap_scope = None;
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        match key {
            25 => bootstrap_scope = Some(decode_record_id_for_binding(&mut cursor)?),
            28 => {
                closeout_postconditions_present = true;
                cursor.skip_value().map_err(|_| RecordDecodeError)?;
            }
            30 => {
                let count = cursor.array().map_err(|_| RecordDecodeError)?;
                for _ in 0..count {
                    if cursor.uint().map_err(|_| RecordDecodeError)? == 3 {
                        supports_closeout_creation = true;
                    }
                }
            }
            _ => cursor.skip_value().map_err(|_| RecordDecodeError)?,
        }
    }
    Ok(PolicyCloseoutFacts {
        supports_closeout_creation,
        closeout_postconditions_present,
        bootstrap_scope,
    })
}

fn decode_assumption_establishment_definition(
    record_bytes: &[u8],
) -> Result<JournalReference, RecordDecodeError> {
    let (mut cursor, field_count) = decode_event_record_body(record_bytes, 81)?;
    let mut definition = None;
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        if key == 16 {
            definition = Some(decode_typed_journal_reference_for_binding(
                &mut cursor,
                &[800],
            )?);
        } else {
            cursor.skip_value().map_err(|_| RecordDecodeError)?;
        }
    }
    definition.ok_or(RecordDecodeError)
}

fn decode_compatibility_definitions(
    record_bytes: &[u8],
) -> Result<(JournalReference, JournalReference), RecordDecodeError> {
    let (mut cursor, field_count) = decode_event_record_body(record_bytes, 84)?;
    let mut source = None;
    let mut target = None;
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        match key {
            16 => {
                source = Some(decode_typed_journal_reference_for_binding(
                    &mut cursor,
                    &[800],
                )?)
            }
            17 => {
                target = Some(decode_typed_journal_reference_for_binding(
                    &mut cursor,
                    &[800],
                )?)
            }
            _ => cursor.skip_value().map_err(|_| RecordDecodeError)?,
        }
    }
    Ok((
        source.ok_or(RecordDecodeError)?,
        target.ok_or(RecordDecodeError)?,
    ))
}

fn decode_bootstrap_declaration_scope(record_bytes: &[u8]) -> Result<RecordId, RecordDecodeError> {
    let (mut cursor, field_count) = decode_event_record_body(record_bytes, 86)?;
    let mut scope = None;
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        if key == 16 {
            scope = Some(decode_record_id_for_binding(&mut cursor)?);
        } else {
            cursor.skip_value().map_err(|_| RecordDecodeError)?;
        }
    }
    scope.ok_or(RecordDecodeError)
}

fn validate_closeout_formal_binding(
    records: &[(RecordId, Vec<u8>)],
    definition: &JournalReference,
    establishment: &JournalReference,
    compatibility: Option<&JournalReference>,
) -> Result<(), RecordDecodeError> {
    let establishment_definition = decode_assumption_establishment_definition(
        record_bytes_for_reference(records, establishment)?,
    )?;
    if establishment_definition == *definition {
        return compatibility
            .is_none()
            .then_some(())
            .ok_or(RecordDecodeError);
    }
    let compatibility = compatibility.ok_or(RecordDecodeError)?;
    let (source, target) =
        decode_compatibility_definitions(record_bytes_for_reference(records, compatibility)?)?;
    if source != establishment_definition || target != *definition {
        return Err(RecordDecodeError);
    }
    Ok(())
}

fn validate_closeout_bootstrap_scopes(
    records: &[(RecordId, Vec<u8>)],
    declarations: &[JournalReference],
    required_scope: RecordId,
) -> Result<(), RecordDecodeError> {
    if declarations.is_empty() {
        return Err(RecordDecodeError);
    }
    for declaration in declarations {
        if decode_bootstrap_declaration_scope(record_bytes_for_reference(records, declaration)?)?
            != required_scope
        {
            return Err(RecordDecodeError);
        }
    }
    Ok(())
}

fn validate_review_admission_event_bindings(
    retained_journal: &RetainedJournal,
    entry: &RetainedJournalEntry,
    record_bytes: &[u8],
    records: &[(RecordId, Vec<u8>)],
) -> Result<(), RecordDecodeError> {
    let (mut cursor, field_count) = decode_event_record_body(record_bytes, 32)?;
    let mut authorities = Vec::new();
    let identities = Vec::new();
    let mut operation_start = None;
    let mut request_reference = None;
    let mut result_reference = None;
    let mut policy_reference = None;
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        match key {
            17..=19 => {
                let expected_event_type = match key {
                    17 => 300,
                    18 => 301,
                    19 => 400,
                    _ => unreachable!(),
                };
                let reference = decode_typed_journal_reference_for_binding(
                    &mut cursor,
                    &[expected_event_type],
                )?;
                match key {
                    17 => request_reference = Some(reference.clone()),
                    18 => result_reference = Some(reference.clone()),
                    19 => policy_reference = Some(reference.clone()),
                    _ => unreachable!(),
                }
                authorities.push(reference);
            }
            20 => {
                decode_failed_candidate_record_set(&mut cursor, 1..=3)?;
            }
            21 => {
                decode_failed_candidate_journal_set(&mut cursor, 1..=3)?;
            }
            23 => {
                operation_start =
                    Some(decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?);
            }
            _ => cursor.skip_value().map_err(|_| RecordDecodeError)?,
        }
    }
    validate_exact_authority_dependencies(entry, &authorities)?;
    validate_exact_identity_dependencies(entry, identities)?;
    for reference in authorities.iter().chain(operation_start.as_ref()) {
        validate_resolved_prior_reference(retained_journal, entry, reference)?;
    }
    let request = request_reference
        .as_ref()
        .map(|reference| {
            ReviewRequestRecord::decode_authoritative(record_bytes_for_reference(
                records, reference,
            )?)
        })
        .transpose()?;
    let result = result_reference
        .as_ref()
        .map(|reference| {
            ReviewResultRecord::decode_authoritative(record_bytes_for_reference(
                records, reference,
            )?)
        })
        .transpose()?;
    if let (Some(request_reference), Some(request), Some(result)) = (
        request_reference.as_ref(),
        request.as_ref(),
        result.as_ref(),
    ) {
        if result.review_request_authority_ref() != request_reference
            || result.freeze_authority_ref() != request.freeze_authority_ref()
            || result.manifest_id() != request.manifest_id()
            || result.review_role_id() != request.review_role_id()
            || result.review_scope_ref() != request.review_scope_ref()
            || result.review_method_ref() != request.review_method_ref()
            || result.review_package_anchor_id() != request.review_package_anchor_id()
        {
            return Err(RecordDecodeError);
        }
    }
    if let (Some(request), Some(policy_reference)) = (request.as_ref(), policy_reference.as_ref()) {
        if request.policy_authority_ref() != policy_reference {
            return Err(RecordDecodeError);
        }
    }

    if entry.event_type_id().value() == 302 {
        let policy_reference = policy_reference.as_ref().ok_or(RecordDecodeError)?;
        let contexts = decode_policy_supported_contexts(record_bytes_for_reference(
            records,
            policy_reference,
        )?)?;
        if !contexts.contains(&2) {
            return Err(RecordDecodeError);
        }
    }
    Ok(())
}

fn validate_closeout_event_bindings(
    retained_journal: &RetainedJournal,
    entry: &RetainedJournalEntry,
    record_bytes: &[u8],
    records: &[(RecordId, Vec<u8>)],
) -> Result<(), RecordDecodeError> {
    let (mut cursor, field_count) = decode_event_record_body(record_bytes, 50)?;
    let mut authorities = Vec::new();
    let mut identities = Vec::new();
    let mut predecessor_closeout_id = None;
    let mut operation_start = None;
    let mut freeze_reference = None;
    let mut policy_reference = None;
    let mut manifest_id = None;
    let mut pre_verification_present = false;
    let mut bootstrap_scope = None;
    let mut bootstrap_authorities = Vec::new();
    let mut formal_bindings = Vec::new();
    let mut resolved_record_candidates = Vec::new();
    let mut resolved_journal_candidates = Vec::new();
    let mut failed_record_candidates = Vec::new();
    let mut failed_journal_candidates = Vec::new();
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        match key {
            17 | 21 | 22 => {
                let expected_event_type = match key {
                    17 => 101,
                    21 => 200,
                    22 => 400,
                    _ => unreachable!(),
                };
                let reference = decode_typed_journal_reference_for_binding(
                    &mut cursor,
                    &[expected_event_type],
                )?;
                if key == 17 {
                    freeze_reference = Some(reference.clone());
                } else if key == 21 {
                    pre_verification_present = true;
                    resolved_journal_candidates.push((7, reference.clone()));
                    resolved_record_candidates
                        .push((7, record_id_from_event_reference(&reference)));
                } else if key == 22 {
                    policy_reference = Some(reference.clone());
                }
                authorities.push(reference);
            }
            19 | 20 | 26 | 27 => {
                let expected_event_type = match key {
                    19 => 302,
                    20 => 200,
                    26 => 806,
                    27 => 808,
                    _ => unreachable!(),
                };
                let references = decode_typed_journal_reference_set_for_binding(
                    &mut cursor,
                    &[expected_event_type],
                )?;
                if key == 26 {
                    bootstrap_authorities = references.clone();
                }
                let candidate_role = match key {
                    19 => 5,
                    20 => 6,
                    26 => 12,
                    27 => 13,
                    _ => unreachable!(),
                };
                resolved_journal_candidates.extend(
                    references
                        .iter()
                        .cloned()
                        .map(|reference| (candidate_role, reference)),
                );
                resolved_record_candidates.extend(
                    references.iter().map(|reference| {
                        (candidate_role, record_id_from_event_reference(reference))
                    }),
                );
                authorities.extend(references);
            }
            18 | 23 => {
                let record_id = decode_record_id_for_binding(&mut cursor)?;
                if key == 18 {
                    manifest_id = Some(record_id);
                    resolved_record_candidates.push((14, record_id));
                } else {
                    bootstrap_scope = Some(record_id);
                }
                identities.push(record_id);
            }
            24 => {
                let record_id = decode_record_id_for_binding(&mut cursor)?;
                predecessor_closeout_id = Some(record_id);
                resolved_record_candidates.push((8, record_id));
            }
            25 => {
                cursor.map_exact(2).map_err(|_| RecordDecodeError)?;
                cursor.key(0).map_err(|_| RecordDecodeError)?;
                let verification_references =
                    decode_typed_journal_reference_set_for_binding(&mut cursor, &[803])?;
                resolved_journal_candidates.extend(
                    verification_references
                        .iter()
                        .cloned()
                        .map(|reference| (9, reference)),
                );
                resolved_record_candidates.extend(
                    verification_references
                        .iter()
                        .map(|reference| (9, record_id_from_event_reference(reference))),
                );
                authorities.extend(verification_references);
                cursor.key(1).map_err(|_| RecordDecodeError)?;
                let count = cursor.array().map_err(|_| RecordDecodeError)?;
                for _ in 0..count {
                    let fields = cursor.map().map_err(|_| RecordDecodeError)?;
                    if !matches!(fields, 2 | 3) {
                        return Err(RecordDecodeError);
                    }
                    cursor.key(0).map_err(|_| RecordDecodeError)?;
                    let definition =
                        decode_typed_journal_reference_for_binding(&mut cursor, &[800])?;
                    cursor.key(1).map_err(|_| RecordDecodeError)?;
                    let establishment =
                        decode_typed_journal_reference_for_binding(&mut cursor, &[801])?;
                    let compatibility = if fields == 3 {
                        cursor.key(2).map_err(|_| RecordDecodeError)?;
                        Some(decode_typed_journal_reference_for_binding(
                            &mut cursor,
                            &[804],
                        )?)
                    } else {
                        None
                    };
                    resolved_journal_candidates.push((10, establishment.clone()));
                    resolved_record_candidates
                        .push((10, record_id_from_event_reference(&establishment)));
                    resolved_journal_candidates.extend(
                        compatibility
                            .iter()
                            .cloned()
                            .map(|reference| (11, reference)),
                    );
                    resolved_record_candidates.extend(
                        compatibility
                            .iter()
                            .map(|reference| (11, record_id_from_event_reference(reference))),
                    );
                    authorities.push(definition.clone());
                    authorities.push(establishment.clone());
                    authorities.extend(compatibility.iter().cloned());
                    formal_bindings.push((definition, establishment, compatibility));
                }
            }
            28 => {
                failed_record_candidates
                    .extend(decode_failed_candidate_record_set_with_values(&mut cursor, 5..=14)?.1);
            }
            29 => {
                failed_journal_candidates.extend(
                    decode_failed_candidate_journal_set_with_values(&mut cursor, 5..=14)?.1,
                );
            }
            31 => {
                operation_start =
                    Some(decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?);
            }
            _ => cursor.skip_value().map_err(|_| RecordDecodeError)?,
        }
    }
    if let Some(predecessor_id) = predecessor_closeout_id {
        let mut matching = entry.authority_dependencies().iter().filter(|reference| {
            reference.event_type_id().value() == 500
                && reference.event_record_id().as_bytes() == predecessor_id.as_bytes()
        });
        let predecessor = matching.next().ok_or(RecordDecodeError)?;
        if matching.next().is_some() {
            return Err(RecordDecodeError);
        }
        authorities.push(predecessor.clone());
        resolved_journal_candidates.push((8, predecessor.clone()));
        resolved_record_candidates.push((8, record_id_from_event_reference(predecessor)));
    }
    if failed_record_candidates
        .iter()
        .any(|candidate| resolved_record_candidates.contains(candidate))
        || failed_journal_candidates
            .iter()
            .any(|candidate| resolved_journal_candidates.contains(candidate))
    {
        return Err(RecordDecodeError);
    }
    validate_exact_authority_dependencies(entry, &authorities)?;
    validate_exact_identity_dependencies(entry, identities)?;
    for reference in authorities.iter().chain(operation_start.as_ref()) {
        validate_resolved_prior_reference(retained_journal, entry, reference)?;
    }
    let freeze_reference = freeze_reference.as_ref().ok_or(RecordDecodeError)?;
    let freeze = FreezeReceiptRecord::decode_authoritative(record_bytes_for_reference(
        records,
        freeze_reference,
    )?)?;
    if manifest_id != Some(freeze.input().manifest_id) {
        return Err(RecordDecodeError);
    }
    let policy_reference = policy_reference.as_ref().ok_or(RecordDecodeError)?;
    let policy =
        decode_policy_closeout_facts(record_bytes_for_reference(records, policy_reference)?)?;
    if !policy.supports_closeout_creation
        || pre_verification_present != policy.closeout_postconditions_present
        || bootstrap_scope != policy.bootstrap_scope
        || !bootstrap_authorities.is_empty() != policy.bootstrap_scope.is_some()
    {
        return Err(RecordDecodeError);
    }
    if let Some(required_scope) = bootstrap_scope {
        validate_closeout_bootstrap_scopes(records, &bootstrap_authorities, required_scope)?;
    }
    for (definition, establishment, compatibility) in formal_bindings {
        validate_closeout_formal_binding(
            records,
            &definition,
            &establishment,
            compatibility.as_ref(),
        )?;
    }
    Ok(())
}

fn validate_authoritative_event_records(
    retained_journal: &RetainedJournal,
    records: &[(RecordId, Vec<u8>)],
) -> Result<(), AuthoritativeRegistryStoreOpenError> {
    let mut semantic_authority_unavailable = false;
    for entry in &retained_journal.entries {
        let event_reference = JournalReference::new(
            entry.registry_id(),
            entry.entry_index(),
            entry.entry_hash(),
            entry.event_type_id(),
            entry.event_record_id(),
        );
        let record_id = record_id_from_event_reference(&event_reference);
        let record_bytes = records
            .binary_search_by(|(stored_id, _)| stored_id.as_bytes().cmp(record_id.as_bytes()))
            .ok()
            .map(|index| records[index].1.as_slice())
            .ok_or(AuthoritativeRegistryStoreOpenError::EventRecordUnavailable)?;
        validate_event_record_structural_binding(retained_journal, &event_reference, record_bytes)
            .map_err(AuthoritativeRegistryStoreOpenError::EventRecordBinding)?;
        let frame = StrictRecordFrame::decode_authoritative(record_bytes)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
        let expected_record_type = expected_record_type_for_event(entry.event_type_id().value())
            .ok_or(AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
        if frame.record_type_id().value() != expected_record_type {
            return Err(AuthoritativeRegistryStoreOpenError::EventRecordDecode);
        }
        validate_event_record_type_local_schema(entry.event_type_id().value(), record_bytes)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
        validate_event_record_lifecycle_binding(entry, record_bytes)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
        validate_redundant_event_specific_bindings(entry, record_bytes)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
        validate_event_record_reference_bindings(retained_journal, entry, record_bytes, records)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;

        match entry.event_type_id().value() {
            1 => {
                GenesisRecord::decode_authoritative(record_bytes)
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            }
            100 => {
                let record = FreezeAttemptStartRecord::decode_authoritative(record_bytes)
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
                if record.input().freeze_attempt_id.as_bytes() != &entry.lifecycle_object_id()
                    || entry.freeze_attempt_intended_root_id()
                        != Some(record.input().intended_root_id)
                    || derive_freeze_root(entry.registry_id(), record.input().freeze_attempt_id)
                        .intended_root_id()
                        != record.input().intended_root_id
                {
                    return Err(AuthoritativeRegistryStoreOpenError::EventRecordDecode);
                }
                let policy_bytes = records
                    .binary_search_by(|(stored_id, _)| {
                        stored_id
                            .as_bytes()
                            .cmp(record.input().policy_record_id.as_bytes())
                    })
                    .ok()
                    .map(|index| records[index].1.as_slice())
                    .ok_or(AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
                validate_policy_record_schema(policy_bytes)
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
                if !decode_policy_supported_contexts(policy_bytes)
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?
                    .contains(&1)
                {
                    return Err(AuthoritativeRegistryStoreOpenError::EventRecordDecode);
                }
            }
            101 => {
                match validate_resolved_freeze_committed_binding(
                    retained_journal,
                    event_reference.clone(),
                    &RecordNamespaceResolver(records),
                )
                .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?
                {
                    ResolvedFreezeCommittedBindingOutcome::AuthorityEvidenceUnavailable => {}
                }
                validate_freeze_commit_policy_requirements(record_bytes, records)
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
                let record = FreezeReceiptRecord::decode_authoritative(record_bytes)
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
                if record.input().freeze_attempt_id.as_bytes() != &entry.lifecycle_object_id() {
                    return Err(AuthoritativeRegistryStoreOpenError::EventRecordDecode);
                }
            }
            300 => {
                validate_review_request_recorded_binding(
                    retained_journal,
                    &event_reference,
                    record_bytes,
                )
                .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
                let request = ReviewRequestRecord::decode_authoritative(record_bytes)
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
                validate_review_request_replay_contract(retained_journal, entry, &request, records)
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            }
            301 => {
                validate_review_result_recorded_binding(
                    retained_journal,
                    &event_reference,
                    record_bytes,
                )
                .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
                let result = ReviewResultRecord::decode_authoritative(record_bytes)
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
                validate_review_result_replay_contract(retained_journal, entry, &result, records)
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            }
            302 | 303 => {
                validate_review_admission_event_bindings(
                    retained_journal,
                    entry,
                    record_bytes,
                    records,
                )
                .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            }
            400 => {
                validate_policy_event_bindings(retained_journal, entry, record_bytes)
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            }
            500 | 501 => {
                validate_closeout_event_bindings(retained_journal, entry, record_bytes, records)
                    .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?;
            }
            _ => {}
        }
        if event_semantic_authority_is_unavailable(
            entry.event_type_id().value(),
            record_bytes,
            records,
        )
        .map_err(|_| AuthoritativeRegistryStoreOpenError::EventRecordDecode)?
        {
            semantic_authority_unavailable = true;
        }
    }
    if semantic_authority_unavailable {
        Err(AuthoritativeRegistryStoreOpenError::EventSemanticAuthorityUnavailable)
    } else {
        Ok(())
    }
}

fn event_semantic_authority_is_unavailable(
    event_type: u16,
    record_bytes: &[u8],
    records: &[(RecordId, Vec<u8>)],
) -> Result<bool, RecordDecodeError> {
    if event_type == 300 {
        // Only the exact selected marker removes the generic replay gate. The
        // Store-owned selected producer and validator re-establish the full
        // Freeze/Policy/Anchor prerequisites before returning a Request witness.
        // Malformed bytes remain unavailable here so the caller's earlier
        // structural validator retains the more precise decode classification.
        return Ok(ReviewRequestRecord::decode_authoritative(record_bytes)
            .ok()
            .and_then(|request| request.terminal_authority_closure_sha256().copied())
            != Some(TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256));
    }
    if event_type == 301 {
        // This only opens the exact versioned selected Result lane after the
        // preceding structural replay contract has bound it to a marked Request.
        // It is not a generic Result or §82/Admission authority decision.
        let selected = ReviewResultRecord::decode_authoritative(record_bytes)
            .ok()
            .filter(|result| result.review_package_anchor_binding_version() == Some(1))
            .and_then(|result| {
                record_bytes_for_reference(records, result.review_request_authority_ref())
                    .ok()
                    .and_then(|bytes| ReviewRequestRecord::decode_authoritative(bytes).ok())
            })
            .and_then(|request| request.terminal_authority_closure_sha256().copied())
            == Some(TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256);
        return Ok(!selected);
    }
    if matches!(event_type, 302 | 303) {
        // A terminal selected Admission is replayable only when its own marker
        // and the entire selected Result-to-Request carrier chain are present.
        // `validate_review_admission_event_bindings` has already checked the
        // exact event, dependency, and record-local bindings before this gate.
        let selected = ReviewAdmissionRecord::decode_authoritative(record_bytes)
            .ok()
            .and_then(|admission| {
                (admission.terminal_authority_closure_sha256()
                    == Some(&TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256))
                .then_some(admission)
            })
            .and_then(|admission| {
                admission.review_result_ref().and_then(|reference| {
                    record_bytes_for_reference(records, reference)
                        .ok()
                        .and_then(|bytes| ReviewResultRecord::decode_authoritative(bytes).ok())
                })
            })
            .filter(|result| result.review_package_anchor_binding_version() == Some(1))
            .and_then(|result| {
                record_bytes_for_reference(records, result.review_request_authority_ref())
                    .ok()
                    .and_then(|bytes| ReviewRequestRecord::decode_authoritative(bytes).ok())
            })
            .and_then(|request| request.terminal_authority_closure_sha256().copied())
            == Some(TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256);
        return Ok(!selected);
    }
    if matches!(event_type, 200 | 301 | 302 | 303 | 500 | 501 | 600 | 700) {
        // The retained inputs do not uniquely re-establish all frozen contextual semantics for
        // these families: Verification-to-Freeze/Manifest continuity remains incomplete; generic
        // Review Request, Review Result, Review Admission, and Closeout Policy satisfaction require
        // an unassigned generic gate-Scope relation; Review Request also lacks positive Freeze
        // authority; capability transition sets lack a frozen capability-ID mapping; and
        // Manifest-relative eviction Scope validation lacks assigned selected-profile semantics.
        // Type-local, event/body, dependency, and otherwise decidable contextual checks run before
        // this gate so malformed histories retain their more precise classification. A locally
        // valid event in any of these families still cannot enter positive authoritative replay.
        return Ok(true);
    }
    if event_type != 801 {
        return Ok(false);
    }
    let (mut cursor, field_count) = decode_event_record_body(record_bytes, 81)?;
    let mut cached_provenance_present = false;
    for _ in 0..field_count {
        let key = cursor.uint().map_err(|_| RecordDecodeError)?;
        if key == 25 {
            cached_provenance_present = true;
        }
        cursor.skip_value().map_err(|_| RecordDecodeError)?;
    }
    // The frozen bytes provide no independent witness that key-25 omission represents an
    // operation-time fresh observation. Cached observations remain supported through their exact
    // type-63 provenance Record.
    Ok(!cached_provenance_present)
}

fn expected_record_type_for_event(event_type: u16) -> Option<u16> {
    match event_type {
        1 => Some(1),
        100 => Some(3),
        101 => Some(4),
        102 => Some(5),
        103 => Some(6),
        104 => Some(7),
        200 => Some(12),
        300 => Some(30),
        301 => Some(31),
        302 | 303 => Some(32),
        400 => Some(40),
        500 | 501 => Some(50),
        600 => Some(62),
        700 => Some(71),
        701..=703 => Some(72),
        800..=808 => Some(80 + (event_type - 800)),
        _ => None,
    }
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

fn validate_selected_profile_namespace_layout(root: &Path) -> Result<(), ()> {
    validate_selected_exact_directory_entries(
        root,
        &[
            ("registry", true),
            ("journal", true),
            ("records", true),
            ("roots", true),
            ("coordination", true),
        ],
    )?;
    validate_selected_exact_directory_entries(&root.join("registry"), &[("genesis.cbor", false)])?;

    let coordination = root.join("coordination");
    let mut seen_freeze = false;
    let mut seen_staging = false;
    let mut seen_publication_lock = false;
    for entry in fs::read_dir(&coordination).map_err(|_| ())? {
        let entry = entry.map_err(|_| ())?;
        let name = entry.file_name().into_string().map_err(|_| ())?;
        let file_type = entry.file_type().map_err(|_| ())?;
        let expected_directory = match name.as_str() {
            "freeze" => {
                if seen_freeze {
                    return Err(());
                }
                seen_freeze = true;
                true
            }
            "staging" => {
                if seen_staging {
                    return Err(());
                }
                seen_staging = true;
                true
            }
            "publication.lock" => {
                if seen_publication_lock {
                    return Err(());
                }
                seen_publication_lock = true;
                false
            }
            _ => return Err(()),
        };
        if file_type.is_symlink()
            || if expected_directory {
                !file_type.is_dir()
            } else {
                !file_type.is_file()
            }
        {
            return Err(());
        }
    }
    (seen_freeze && seen_staging).then_some(()).ok_or(())
}

fn validate_selected_retained_object_names(
    journal_dir: &Path,
    records_dir: &Path,
) -> Result<(), AuthoritativeRegistryStoreOpenError> {
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
        if parse_journal_slot_name(&name).is_none() {
            return Err(AuthoritativeRegistryStoreOpenError::JournalSlotNameInvalid);
        }
    }
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
        if parse_record_filename(&name).is_none() {
            return Err(AuthoritativeRegistryStoreOpenError::RecordFilenameInvalid);
        }
    }
    Ok(())
}

fn validate_selected_exact_directory_entries(
    directory: &Path,
    expected: &[(&str, bool)],
) -> Result<(), ()> {
    let mut found = vec![false; expected.len()];
    for entry in fs::read_dir(directory).map_err(|_| ())? {
        let entry = entry.map_err(|_| ())?;
        let name = entry.file_name().into_string().map_err(|_| ())?;
        let index = expected
            .iter()
            .position(|(expected_name, _)| *expected_name == name)
            .ok_or(())?;
        if found[index] {
            return Err(());
        }
        let file_type = entry.file_type().map_err(|_| ())?;
        if file_type.is_symlink()
            || if expected[index].1 {
                !file_type.is_dir()
            } else {
                !file_type.is_file()
            }
        {
            return Err(());
        }
        found[index] = true;
    }
    found
        .into_iter()
        .all(|present| present)
        .then_some(())
        .ok_or(())
}

fn validate_selected_retained_root_locators(
    root: &Path,
    retained_journal: &RetainedJournal,
) -> Result<(), ()> {
    for entry in fs::read_dir(root.join("roots")).map_err(|_| ())? {
        let entry = entry.map_err(|_| ())?;
        let file_type = entry.file_type().map_err(|_| ())?;
        if !file_type.is_dir() || file_type.is_symlink() {
            return Err(());
        }
        let name = entry.file_name().into_string().map_err(|_| ())?;
        let attempt_id = parse_lowercase_hex_identity(&name).ok_or(())?;
        if !retained_journal.entries.iter().any(|journal_entry| {
            journal_entry.event_type_id().value() == 100
                && journal_entry.registry_id() == retained_journal.registry_id
                && journal_entry.lifecycle_object_id() == attempt_id
        }) {
            return Err(());
        }
    }
    Ok(())
}

fn parse_lowercase_hex_identity(name: &str) -> Option<[u8; ID_LENGTH]> {
    if name.len() != ID_LENGTH * 2
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    let mut bytes = [0_u8; ID_LENGTH];
    for (index, slot) in bytes.iter_mut().enumerate() {
        let offset = index * 2;
        *slot = u8::from_str_radix(&name[offset..offset + 2], 16).ok()?;
    }
    Some(bytes)
}

fn record_id_from_event_reference(reference: &JournalReference) -> RecordId {
    RecordId::try_from(reference.event_record_id().as_bytes().as_slice())
        .expect("EventRecordId has RecordId width")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_embedded_traversal_bounds_cumulative_retained_artifact_paths() {
        let path = (0..1024).map(|_| vec![b'x'; 1024]).collect::<Vec<_>>();
        let per_path =
            path.iter().map(Vec::len).sum::<usize>() + path.len() * std::mem::size_of::<Vec<u8>>();
        let permitted = AUTHORITATIVE_STORE_MAX_NAMESPACE_BYTES / per_path;
        let mut budget = SelectedEmbeddedTraversalBudget {
            entries: 0,
            path_bytes: 0,
            content_bytes: 0,
            retained_path_bytes: 0,
        };

        for _ in 0..permitted {
            assert_eq!(budget.reserve_retained_artifact_path(&path), Ok(()));
        }
        assert_eq!(
            budget.reserve_retained_artifact_path(&path),
            Err(SelectedEmbeddedFreezePreparationError::SourceResourceLimit),
        );
    }

    #[cfg(target_os = "linux")]
    struct LinuxGenesisFixture {
        root: std::path::PathBuf,
        genesis: std::path::PathBuf,
    }

    #[cfg(target_os = "linux")]
    impl LinuxGenesisFixture {
        fn new(label: &str) -> Self {
            let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "evidence-registry-linux-genesis-{label}-{sequence}-{}",
                std::process::id()
            ));
            fs::create_dir_all(root.join("registry")).unwrap();
            fs::create_dir(root.join("journal")).unwrap();
            fs::create_dir(root.join("records")).unwrap();
            let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
            let storage_capability_class_id =
                RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
            let environment_observation_id =
                RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
            let genesis = GenesisRecord::new(GenesisRecordInput {
                registry_id,
                journal_format_version: 1,
                record_identity_profile_id: 1,
                storage_capability_class_id,
                environment_observation_id,
                created_by_tool_version: "linux-supported-genesis-fixture".to_owned(),
            })
            .unwrap();
            let entry = GenesisJournalEntry::new(
                registry_id,
                EventRecordId::try_from(genesis.record_id().as_bytes().as_slice()).unwrap(),
                storage_capability_class_id,
                environment_observation_id,
            );
            let genesis_path = root.join("registry/genesis.cbor");
            fs::write(&genesis_path, genesis.authoritative_cbor()).unwrap();
            fs::write(
                root.join("journal/00000000000000000000.cbor"),
                entry.authoritative_cbor(),
            )
            .unwrap();
            Self {
                root,
                genesis: genesis_path,
            }
        }
    }

    #[cfg(target_os = "linux")]
    impl Drop for LinuxGenesisFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[cfg(target_os = "macos")]
    const CHILD_DEADLINE: std::time::Duration = std::time::Duration::from_secs(2);
    #[cfg(target_os = "macos")]
    const CHILD_OUTPUT_MAX: usize = 4096;
    #[cfg(target_os = "macos")]
    static MACOS_PROCESS_SPAWN_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[cfg(target_os = "macos")]
    struct BoundedChild {
        child: std::process::Child,
        stdout: Option<std::process::ChildStdout>,
        stderr: Option<std::process::ChildStderr>,
        stdout_bytes: Vec<u8>,
        stderr_bytes: Vec<u8>,
        deadline: std::time::Instant,
    }

    #[cfg(target_os = "macos")]
    impl BoundedChild {
        fn new(child: std::process::Child) -> Result<Self, ()> {
            use std::os::fd::AsRawFd;
            unsafe extern "C" {
                fn fcntl(fd: i32, command: i32, ...) -> i32;
            }
            const F_GETFL: i32 = 3;
            const F_SETFL: i32 = 4;
            const O_NONBLOCK: i32 = 0x0000_0004;
            let mut bounded = Self {
                child,
                stdout: None,
                stderr: None,
                stdout_bytes: Vec::new(),
                stderr_bytes: Vec::new(),
                deadline: std::time::Instant::now() + CHILD_DEADLINE,
            };
            let stdout = bounded.child.stdout.take().ok_or(())?;
            let stderr = bounded.child.stderr.take().ok_or(())?;
            for fd in [stdout.as_raw_fd(), stderr.as_raw_fd()] {
                let flags = unsafe { fcntl(fd, F_GETFL) };
                if flags < 0 || unsafe { fcntl(fd, F_SETFL, flags | O_NONBLOCK) } < 0 {
                    return Err(());
                }
            }
            bounded.stdout = Some(stdout);
            bounded.stderr = Some(stderr);
            Ok(bounded)
        }

        fn pump_stream(
            stream: &mut impl std::io::Read,
            bytes: &mut Vec<u8>,
            deadline: std::time::Instant,
        ) -> Result<(), ()> {
            let mut chunk = [0_u8; 256];
            loop {
                if std::time::Instant::now() >= deadline {
                    return Err(());
                }
                match stream.read(&mut chunk) {
                    Ok(0) => return Ok(()),
                    Ok(count) => {
                        if bytes.len().checked_add(count).ok_or(())? > CHILD_OUTPUT_MAX {
                            return Err(());
                        }
                        bytes.extend_from_slice(&chunk[..count]);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {
                        if std::time::Instant::now() >= deadline {
                            return Err(());
                        }
                        continue;
                    }
                    Err(_) => return Err(()),
                }
            }
        }

        fn pump(&mut self) -> Result<(), ()> {
            self.pump_with_deadline(self.deadline)
        }

        fn pump_with_deadline(&mut self, deadline: std::time::Instant) -> Result<(), ()> {
            Self::pump_stream(
                self.stdout.as_mut().ok_or(())?,
                &mut self.stdout_bytes,
                deadline,
            )?;
            Self::pump_stream(
                self.stderr.as_mut().ok_or(())?,
                &mut self.stderr_bytes,
                deadline,
            )
        }

        fn wait_token(&mut self, token: &[u8]) -> Result<(), ()> {
            loop {
                self.pump()?;
                if std::time::Instant::now() >= self.deadline {
                    return Err(());
                }
                if self
                    .stdout_bytes
                    .windows(token.len())
                    .any(|window| window == token)
                {
                    return Ok(());
                }
                if self.child.try_wait().map_err(|_| ())?.is_some()
                    || std::time::Instant::now() >= self.deadline
                {
                    return Err(());
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        fn wait_exit(&mut self) -> Result<std::process::ExitStatus, ()> {
            loop {
                self.pump()?;
                if std::time::Instant::now() >= self.deadline {
                    return Err(());
                }
                if let Some(status) = self.child.try_wait().map_err(|_| ())? {
                    self.pump()?;
                    if std::time::Instant::now() >= self.deadline {
                        return Err(());
                    }
                    return Ok(status);
                }
                if std::time::Instant::now() >= self.deadline {
                    return Err(());
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        fn kill_reap(&mut self) -> Result<std::process::ExitStatus, ()> {
            let _ = self.child.kill();
            let reap_deadline = std::time::Instant::now() + CHILD_DEADLINE;
            while std::time::Instant::now() < reap_deadline {
                let _ = self.pump_with_deadline(reap_deadline);
                if let Some(status) = self.child.try_wait().map_err(|_| ())? {
                    return Ok(status);
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(())
        }
    }

    #[cfg(target_os = "macos")]
    impl Drop for BoundedChild {
        fn drop(&mut self) {
            let _ = self.kill_reap();
        }
    }

    #[test]
    fn exact_authority_dependency_validation_has_a_bounded_comparison_topology() {
        const DEPENDENCY_COUNT: u64 = 256;

        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let containing_entry_index = JournalEntryIndex::try_from(DEPENDENCY_COUNT + 1).unwrap();
        let mut expected = Vec::new();
        for index in 1..=DEPENDENCY_COUNT {
            let mut hash = [0_u8; ID_LENGTH];
            hash[..8].copy_from_slice(&index.to_be_bytes());
            let mut record_id = [0_u8; ID_LENGTH];
            record_id[..8].copy_from_slice(&index.to_le_bytes());
            expected.push(JournalReference::new(
                registry_id,
                JournalEntryIndex::try_from(index).unwrap(),
                JournalEntryHash::try_from(hash.as_slice()).unwrap(),
                EventTypeId::try_from(808).unwrap(),
                EventRecordId::try_from(record_id.as_slice()).unwrap(),
            ));
        }
        let authority_dependencies =
            AuthorityDependencyCollection::from_authoritative_ordered_elements(
                AuthorityDependencyContext::new(registry_id, containing_entry_index),
                expected.clone(),
            )
            .unwrap();
        let entry = RetainedJournalEntry::Common(CommonRetainedJournalEntry {
            registry_id,
            entry_index: containing_entry_index,
            previous_entry_hash: JournalEntryHash::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            event_type_id: EventTypeId::try_from(808).unwrap(),
            event_record_id: EventRecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
            storage_capability_class_id: RecordId::try_from([0x40; ID_LENGTH].as_slice()).unwrap(),
            environment_observation_id: RecordId::try_from([0x50; ID_LENGTH].as_slice()).unwrap(),
            lifecycle_object_kind: LifecycleObjectKind::FormalFindingClassification,
            lifecycle_object_id: [0x60; ID_LENGTH],
            freeze_attempt_intended_root_id: None,
            identity_dependencies: IdentityDependencyCollection {
                elements: Vec::new(),
            },
            authority_dependencies,
            authoritative_bytes: Vec::new(),
        });

        expected.reverse();
        crate::JOURNAL_REFERENCE_EQUALITY_COMPARISONS.with(|comparisons| comparisons.set(0));
        assert_eq!(
            validate_exact_authority_dependencies(&entry, &expected),
            Ok(())
        );
        let comparisons =
            crate::JOURNAL_REFERENCE_EQUALITY_COMPARISONS.with(|comparisons| comparisons.get());
        assert!(
            comparisons > 0,
            "the equality-comparison probe was bypassed"
        );

        assert!(
            comparisons <= expected.len() * 16,
            "exact dependency validation used {comparisons} reference comparisons for {} dependencies",
            expected.len()
        );
    }

    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    #[test]
    fn store_open_fails_without_mandatory_retained_generation_protection() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-open-protection-unavailable-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("registry")).unwrap();
        fs::create_dir(root.join("journal")).unwrap();
        fs::create_dir(root.join("records")).unwrap();
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let storage_capability_class_id = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let environment_observation_id = RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
        let genesis_record = GenesisRecord::new(GenesisRecordInput {
            registry_id,
            journal_format_version: 1,
            record_identity_profile_id: 1,
            storage_capability_class_id,
            environment_observation_id,
            created_by_tool_version: "store-open-protection-test".to_owned(),
        })
        .unwrap();
        let genesis_entry = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from(genesis_record.record_id().as_bytes().as_slice()).unwrap(),
            storage_capability_class_id,
            environment_observation_id,
        );
        fs::write(
            root.join("registry/genesis.cbor"),
            genesis_record.authoritative_cbor(),
        )
        .unwrap();
        fs::write(
            root.join("journal/00000000000000000000.cbor"),
            genesis_entry.authoritative_cbor(),
        )
        .unwrap();

        assert_eq!(
            AuthoritativeRegistryStore::open(&root).unwrap_err(),
            AuthoritativeRegistryStoreOpenError::RetainedGenerationProtectionUnavailable
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn store_open_retains_a_supported_genesis_generation() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-open-protection-unavailable-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("registry")).unwrap();
        fs::create_dir(root.join("journal")).unwrap();
        fs::create_dir(root.join("records")).unwrap();
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let storage_capability_class_id = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let environment_observation_id = RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
        let genesis_record = GenesisRecord::new(GenesisRecordInput {
            registry_id,
            journal_format_version: 1,
            record_identity_profile_id: 1,
            storage_capability_class_id,
            environment_observation_id,
            created_by_tool_version: "store-open-protection-test".to_owned(),
        })
        .unwrap();
        let genesis_entry = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from(genesis_record.record_id().as_bytes().as_slice()).unwrap(),
            storage_capability_class_id,
            environment_observation_id,
        );
        fs::write(
            root.join("registry/genesis.cbor"),
            genesis_record.authoritative_cbor(),
        )
        .unwrap();
        fs::write(
            root.join("journal/00000000000000000000.cbor"),
            genesis_entry.authoritative_cbor(),
        )
        .unwrap();

        let mut store = AuthoritativeRegistryStore::open(&root).unwrap();
        assert_eq!(store.root(), root.canonicalize().unwrap());
        let displaced = root.with_extension("displaced");
        assert_eq!(
            store.reload_authoritative_namespaces_with_hook(|| {
                fs::rename(&root, &displaced).unwrap();
                fs::create_dir_all(root.join("registry")).unwrap();
                fs::create_dir(root.join("journal")).unwrap();
                fs::create_dir(root.join("records")).unwrap();
                fs::write(
                    root.join("registry/genesis.cbor"),
                    genesis_record.authoritative_cbor(),
                )
                .unwrap();
                fs::write(
                    root.join("journal/00000000000000000000.cbor"),
                    genesis_entry.authoritative_cbor(),
                )
                .unwrap();
            }),
            Err(AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged
            ))
        );
        drop(store);
        fs::remove_dir_all(displaced).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_freeze_validation_rejects_a_transplanted_retained_root() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-root-transplant-{}-{sequence}",
            std::process::id()
        ));
        let displaced = root.with_extension("displaced");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&displaced);
        fs::create_dir_all(root.join("registry")).unwrap();
        fs::create_dir(root.join("journal")).unwrap();
        fs::create_dir(root.join("records")).unwrap();
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let storage_capability_class_id = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let environment_observation_id = RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
        let genesis_record = GenesisRecord::new(GenesisRecordInput {
            registry_id,
            journal_format_version: 1,
            record_identity_profile_id: 1,
            storage_capability_class_id,
            environment_observation_id,
            created_by_tool_version: "root-transplant-test".to_owned(),
        })
        .unwrap();
        let genesis_entry = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from(genesis_record.record_id().as_bytes().as_slice()).unwrap(),
            storage_capability_class_id,
            environment_observation_id,
        );
        fs::write(
            root.join("registry/genesis.cbor"),
            genesis_record.authoritative_cbor(),
        )
        .unwrap();
        fs::write(
            root.join("journal/00000000000000000000.cbor"),
            genesis_entry.authoritative_cbor(),
        )
        .unwrap();
        let store = AuthoritativeRegistryStore::open(&root).unwrap();
        let reference = store.retained_journal().current_head_reference();
        assert_eq!(
            store.validate_freeze_committed_authority(reference.clone()),
            Err(AuthoritativeFreezeCommittedBindingError::Structural(
                ResolvedFreezeCommittedBindingError::CommittedEventMismatch
            ))
        );
        fs::rename(&root, &displaced).unwrap();
        fs::create_dir(&root).unwrap();
        for name in ["registry", "journal", "records"] {
            fs::rename(displaced.join(name), root.join(name)).unwrap();
        }
        assert_eq!(
            store.validate_freeze_committed_authority(reference),
            Err(AuthoritativeFreezeCommittedBindingError::RetainedGenerationChanged)
        );
        drop(store);
        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir_all(&displaced).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_reload_rejects_a_fresh_candidate_from_a_transient_clone_root() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("evidence-registry-reload-clone-{sequence}"));
        let displaced = root.with_extension("old");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&displaced);
        fs::create_dir_all(root.join("registry")).unwrap();
        fs::create_dir(root.join("journal")).unwrap();
        fs::create_dir(root.join("records")).unwrap();
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let capability = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let environment = RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
        let genesis = GenesisRecord::new(GenesisRecordInput {
            registry_id,
            journal_format_version: 1,
            record_identity_profile_id: 1,
            storage_capability_class_id: capability,
            environment_observation_id: environment,
            created_by_tool_version: "reload-clone-test".to_owned(),
        })
        .unwrap();
        let entry = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from(genesis.record_id().as_bytes().as_slice()).unwrap(),
            capability,
            environment,
        );
        fs::write(
            root.join("registry/genesis.cbor"),
            genesis.authoritative_cbor(),
        )
        .unwrap();
        fs::write(
            root.join("journal/00000000000000000000.cbor"),
            entry.authoritative_cbor(),
        )
        .unwrap();
        let mut store = AuthoritativeRegistryStore::open(&root).unwrap();
        assert_eq!(store.reload_authoritative_namespaces(), Ok(()));
        assert_eq!(
            store.reload_authoritative_namespaces_with_candidate_loader(|path| {
                fs::rename(path, &displaced).unwrap();
                fs::create_dir_all(path.join("registry")).unwrap();
                fs::create_dir(path.join("journal")).unwrap();
                fs::create_dir(path.join("records")).unwrap();
                fs::write(
                    path.join("registry/genesis.cbor"),
                    genesis.authoritative_cbor(),
                )
                .unwrap();
                fs::write(
                    path.join("journal/00000000000000000000.cbor"),
                    entry.authoritative_cbor(),
                )
                .unwrap();
                let candidate = AuthoritativeRegistryStore::open(path);
                fs::remove_dir_all(path).unwrap();
                fs::rename(&displaced, path).unwrap();
                candidate
            }),
            Err(AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged
            ))
        );
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(any(windows, target_os = "linux", target_os = "macos"))]
    #[test]
    fn store_open_rejects_authority_bytes_changed_after_replay_before_positive_return() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-open-generation-race-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("registry")).unwrap();
        fs::create_dir(root.join("journal")).unwrap();
        fs::create_dir(root.join("records")).unwrap();
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let storage_capability_class_id = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let environment_observation_id = RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
        let genesis_record = GenesisRecord::new(GenesisRecordInput {
            registry_id,
            journal_format_version: 1,
            record_identity_profile_id: 1,
            storage_capability_class_id,
            environment_observation_id,
            created_by_tool_version: "store-open-generation-race-test".to_owned(),
        })
        .unwrap();
        let genesis_entry = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from(genesis_record.record_id().as_bytes().as_slice()).unwrap(),
            storage_capability_class_id,
            environment_observation_id,
        );
        let genesis_path = root.join("registry/genesis.cbor");
        fs::write(&genesis_path, genesis_record.authoritative_cbor()).unwrap();
        fs::write(
            root.join("journal/00000000000000000000.cbor"),
            genesis_entry.authoritative_cbor(),
        )
        .unwrap();

        let result = AuthoritativeRegistryStore::open_with_generation_hook(
            &root,
            AuthoritativeRegistryStoreOpenProfile::LegacyThreeNamespace,
            || {
                let mut changed = genesis_record.authoritative_cbor();
                changed[0] ^= 1;
                fs::write(&genesis_path, changed).unwrap();
            },
        );

        assert_eq!(
            result.unwrap_err(),
            AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_fifo_read_child_helper() {
        use std::io::Write;
        let Ok(path) = std::env::var("EVIDENCE_REGISTRY_FIFO_PATH") else {
            return;
        };
        assert!(read_regular_file(Path::new(&path), &mut NamespaceBudget::new()).is_err());
        println!("FIFO_REJECTED");
        std::io::stdout().flush().unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_store_open_fifo_root_child_helper() {
        use std::io::Write;
        let Ok(path) = std::env::var("EVIDENCE_REGISTRY_STORE_OPEN_FIFO_ROOT") else {
            return;
        };
        println!("STORE_ROOT_FIFO_CHILD_STARTED");
        std::io::stdout().flush().unwrap();
        assert_eq!(
            AuthoritativeRegistryStore::open(Path::new(&path)).unwrap_err(),
            AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid
        );
        println!("STORE_ROOT_FIFO_REJECTED");
        std::io::stdout().flush().unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_bounded_child_fault_helper() {
        use std::io::Write;
        let Ok(mode) = std::env::var("EVIDENCE_REGISTRY_BOUNDED_CHILD_MODE") else {
            return;
        };
        match mode.as_str() {
            "missing" => std::thread::sleep(std::time::Duration::from_secs(10)),
            "partial" => {
                print!("PARTIAL");
                std::io::stdout().flush().unwrap();
                std::thread::sleep(std::time::Duration::from_secs(10));
            }
            "stdout" => {
                std::io::stdout().write_all(&vec![b'x'; 8192]).unwrap();
                std::io::stdout().flush().unwrap();
                std::thread::sleep(std::time::Duration::from_secs(10));
            }
            "stderr" => {
                std::io::stderr().write_all(&vec![b'x'; 8192]).unwrap();
                std::io::stderr().flush().unwrap();
                std::thread::sleep(std::time::Duration::from_secs(10));
            }
            "early" => (),
            "ready" => {
                println!("READY");
                std::io::stdout().flush().unwrap();
                std::thread::sleep(std::time::Duration::from_secs(10));
            }
            "fifo" => {
                println!("FIFO_REJECTED");
                std::io::stdout().flush().unwrap();
                std::thread::sleep(std::time::Duration::from_secs(10));
            }
            _ => panic!("unknown bounded child mode"),
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_bounded_child_pump_rejects_expired_deadline_before_read() {
        struct InterruptedThenWouldBlock {
            reads: usize,
        }

        impl std::io::Read for InterruptedThenWouldBlock {
            fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
                self.reads += 1;
                Err(std::io::Error::from(if self.reads == 1 {
                    std::io::ErrorKind::Interrupted
                } else {
                    std::io::ErrorKind::WouldBlock
                }))
            }
        }

        let mut stream = InterruptedThenWouldBlock { reads: 0 };
        let mut bytes = Vec::new();
        let expired = std::time::Instant::now() - std::time::Duration::from_millis(1);
        assert!(BoundedChild::pump_stream(&mut stream, &mut bytes, expired).is_err());
        assert_eq!(stream.reads, 0);
        assert!(bytes.is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_bounded_child_fault_matrix() {
        use std::process::{Command, Stdio};
        let _spawn_guard = MACOS_PROCESS_SPAWN_TEST_MUTEX.lock().unwrap();
        for (mode, token, token_expected) in [
            ("missing", b"READY".as_slice(), false),
            ("partial", b"READY".as_slice(), false),
            ("stdout", b"READY".as_slice(), false),
            ("stderr", b"READY".as_slice(), false),
            ("early", b"READY".as_slice(), false),
            ("ready", b"READY".as_slice(), true),
            ("fifo", b"FIFO_REJECTED".as_slice(), true),
        ] {
            let mut child = BoundedChild::new(
                Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "authoritative_store::tests::macos_bounded_child_fault_helper",
                        "--nocapture",
                    ])
                    .env("EVIDENCE_REGISTRY_BOUNDED_CHILD_MODE", mode)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap(),
            )
            .unwrap();
            let token_result = child.wait_token(token);
            assert_eq!(token_result.is_ok(), token_expected, "{mode}");
            if mode == "early" {
                assert!(child.wait_exit().is_ok(), "{mode}");
            } else {
                assert!(child.wait_exit().is_err(), "{mode}");
                let _ = child.kill_reap().unwrap();
            }
            assert!(child.child.try_wait().unwrap().is_some(), "{mode}");
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_bounded_child_constructor_failure_reaps_owned_child() {
        use std::process::{Command, Stdio};

        let _spawn_guard = MACOS_PROCESS_SPAWN_TEST_MUTEX.lock().unwrap();
        for (label, stdout, stderr) in [
            ("stdout-null", Stdio::null(), Stdio::piped()),
            ("stderr-null", Stdio::piped(), Stdio::null()),
        ] {
            let child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "authoritative_store::tests::macos_bounded_child_fault_helper",
                    "--nocapture",
                ])
                .env("EVIDENCE_REGISTRY_BOUNDED_CHILD_MODE", "missing")
                .stdout(stdout)
                .stderr(stderr)
                .spawn()
                .unwrap();
            let pid = child.id();
            assert!(BoundedChild::new(child).is_err(), "{label}");

            let status = Command::new("ps")
                .args(["-p", &pid.to_string(), "-o", "stat="])
                .output()
                .unwrap();
            if !status.stdout.is_empty() {
                let _ = Command::new("kill").args(["-9", &pid.to_string()]).status();
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            assert!(
                status.stdout.is_empty(),
                "{label}: child {pid} was not reaped"
            );
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_store_open_namespace_fifo_child_helper() {
        use std::io::Write;
        let Ok(root) = std::env::var("EVIDENCE_REGISTRY_STORE_OPEN_FIFO_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        let expected = match std::env::var("EVIDENCE_REGISTRY_STORE_OPEN_EXPECTED").as_deref() {
            Ok("registry") => AuthoritativeRegistryStoreOpenError::RegistryNamespaceInvalid,
            Ok("journal") => AuthoritativeRegistryStoreOpenError::JournalNamespaceInvalid,
            Ok("records") => AuthoritativeRegistryStoreOpenError::RecordNamespaceInvalid,
            _ => panic!("missing namespace FIFO expected error"),
        };
        println!("STORE_NAMESPACE_FIFO_CHILD_STARTED");
        std::io::stdout().flush().unwrap();
        assert_eq!(
            AuthoritativeRegistryStore::open(root).unwrap_err(),
            expected
        );
        println!("STORE_NAMESPACE_FIFO_REJECTED");
        std::io::stdout().flush().unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_read_regular_file_rejects_fifo_without_blocking() {
        let _spawn_guard = MACOS_PROCESS_SPAWN_TEST_MUTEX.lock().unwrap();
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        use std::process::{Command, Stdio};

        unsafe extern "C" {
            fn mkfifo(path: *const i8, mode: u16) -> i32;
        }
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("evidence-registry-macos-fifo-{sequence}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let fifo = root.join("input.fifo");
        let fifo_c = CString::new(fifo.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { mkfifo(fifo_c.as_ptr(), 0o600) }, 0);
        let mut child = BoundedChild::new(
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "authoritative_store::tests::macos_fifo_read_child_helper",
                    "--nocapture",
                ])
                .env("EVIDENCE_REGISTRY_FIFO_PATH", &fifo)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        )
        .unwrap();
        let rejected = child.wait_token(b"FIFO_REJECTED").is_ok();
        let status = if rejected {
            child.wait_exit().ok()
        } else {
            child.kill_reap().ok()
        };
        assert!(rejected && status.is_some_and(|status| status.success()));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_store_open_rejects_fifo_root_without_blocking() {
        let _spawn_guard = MACOS_PROCESS_SPAWN_TEST_MUTEX.lock().unwrap();
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        use std::process::{Command, Stdio};

        unsafe extern "C" {
            fn mkfifo(path: *const i8, mode: u16) -> i32;
        }
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("evidence-registry-macos-store-fifo-{sequence}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let fifo = root.join("root.fifo");
        let fifo_c = CString::new(fifo.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { mkfifo(fifo_c.as_ptr(), 0o600) }, 0);
        let mut child = BoundedChild::new(
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "authoritative_store::tests::macos_store_open_fifo_root_child_helper",
                    "--nocapture",
                ])
                .env("EVIDENCE_REGISTRY_STORE_OPEN_FIFO_ROOT", &fifo)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        )
        .unwrap();
        let rejected = child.wait_token(b"STORE_ROOT_FIFO_REJECTED").is_ok();
        let status = if rejected {
            child.wait_exit().ok()
        } else {
            child.kill_reap().ok()
        };
        assert!(rejected && status.is_some_and(|status| status.success()));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_store_open_rejects_namespace_fifos_without_blocking() {
        let _spawn_guard = MACOS_PROCESS_SPAWN_TEST_MUTEX.lock().unwrap();
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        use std::process::{Command, Stdio};

        unsafe extern "C" {
            fn mkfifo(path: *const i8, mode: u16) -> i32;
        }
        for (case, expected) in [
            ("genesis", "registry"),
            ("registry", "registry"),
            ("journal", "journal"),
            ("records", "records"),
        ] {
            let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir()
                .join(format!("evidence-registry-macos-fifo-{case}-{sequence}"));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir(&root).unwrap();
            for child_name in ["registry", "journal", "records"] {
                if case == "genesis" || child_name != case {
                    fs::create_dir(root.join(child_name)).unwrap();
                }
            }
            let fifo = if case == "genesis" {
                root.join("registry/genesis.cbor")
            } else {
                root.join(case)
            };
            let fifo_c = CString::new(fifo.as_os_str().as_bytes()).unwrap();
            assert_eq!(unsafe { mkfifo(fifo_c.as_ptr(), 0o600) }, 0);
            let mut child = BoundedChild::new(
                Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "authoritative_store::tests::macos_store_open_namespace_fifo_child_helper",
                        "--nocapture",
                    ])
                    .env("EVIDENCE_REGISTRY_STORE_OPEN_FIFO_ROOT", &root)
                    .env("EVIDENCE_REGISTRY_STORE_OPEN_EXPECTED", expected)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap(),
            )
            .unwrap();
            let rejected = child.wait_token(b"STORE_NAMESPACE_FIFO_REJECTED").is_ok();
            let status = if rejected {
                child.wait_exit().ok()
            } else {
                child.kill_reap().ok()
            };
            assert!(rejected && status.is_some_and(|status| status.success()));
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_root_lock_child_helper() {
        use std::io::Write;
        let Ok(root) = std::env::var("EVIDENCE_REGISTRY_ROOT_LOCK_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        let hold = open_real_directory_hold(&root).unwrap();
        match std::env::var("EVIDENCE_REGISTRY_ROOT_LOCK_MODE").as_deref() {
            Ok("attempt") => {
                assert!(acquire_authoritative_publication_lock(&root, &hold).is_err());
                println!("ROOT_LOCK_BLOCKED");
            }
            Ok("hold") => {
                let lock = acquire_authoritative_publication_lock(&root, &hold).unwrap();
                println!("ROOT_LOCK_READY");
                std::io::stdout().flush().unwrap();
                let mut byte = [0_u8; 1];
                let _ = std::io::stdin().lock().read(&mut byte).unwrap();
                drop(lock);
            }
            Ok("exit") => {
                let lock = acquire_authoritative_publication_lock(&root, &hold).unwrap();
                println!("ROOT_LOCK_READY");
                std::io::stdout().flush().unwrap();
                drop(lock);
            }
            _ => panic!("missing root-lock child mode"),
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_root_lock_cross_process_release_lifecycle() {
        let _spawn_guard = MACOS_PROCESS_SPAWN_TEST_MUTEX.lock().unwrap();
        use std::process::{Command, Stdio};

        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-macos-root-lock-process-{sequence}"
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let root_hold = open_real_directory_hold(&root).unwrap();
        let spawn = |mode: &str, input: bool| {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args([
                    "--exact",
                    "authoritative_store::tests::macos_root_lock_child_helper",
                    "--nocapture",
                ])
                .env("EVIDENCE_REGISTRY_ROOT_LOCK_ROOT", &root)
                .env("EVIDENCE_REGISTRY_ROOT_LOCK_MODE", mode)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            if input {
                command.stdin(Stdio::piped());
            }
            BoundedChild::new(command.spawn().unwrap()).unwrap()
        };
        let owner = acquire_authoritative_publication_lock(&root, &root_hold).unwrap();
        let mut blocked = spawn("attempt", false);
        assert!(blocked.wait_token(b"ROOT_LOCK_BLOCKED").is_ok());
        assert!(blocked.wait_exit().unwrap().success());
        drop(owner);
        let mut normal = spawn("exit", false);
        assert!(normal.wait_token(b"ROOT_LOCK_READY").is_ok());
        assert!(normal.wait_exit().unwrap().success());
        let reacquired = acquire_authoritative_publication_lock(&root, &root_hold).unwrap();
        drop(reacquired);
        let mut abnormal = spawn("hold", true);
        assert!(abnormal.wait_token(b"ROOT_LOCK_READY").is_ok());
        assert!(abnormal.child.try_wait().unwrap().is_none());
        assert!(acquire_authoritative_publication_lock(&root, &root_hold).is_err());
        let status = abnormal.kill_reap().unwrap();
        assert!(!status.success());
        let after_kill = acquire_authoritative_publication_lock(&root, &root_hold).unwrap();
        drop(after_kill);
        drop(root_hold);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_authoritative_publication_lock_serializes_cooperative_writers() {
        let _spawn_guard = MACOS_PROCESS_SPAWN_TEST_MUTEX.lock().unwrap();
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-macos-publication-lock-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let root_hold = open_real_directory_hold(&root).unwrap();

        let first = acquire_authoritative_publication_lock(&root, &root_hold).unwrap();
        fs::remove_file(root.join(".evidence-registry-publication.lock")).ok();
        assert!(acquire_authoritative_publication_lock(&root, &root_hold).is_err());
        drop(first);
        let second = acquire_authoritative_publication_lock(&root, &root_hold).unwrap();
        drop(second);

        drop(root_hold);
        fs::remove_dir(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_downgraded_publication_witness_preserves_identity_and_bytes() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-macos-publication-handoff-{}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let hold = open_real_directory_hold(&root).unwrap();
        let final_name = Path::new("final.cbor");
        let publication =
            match publish_new_immutable_file(&root, &hold, final_name, b"exact handoff").unwrap() {
                ImmutablePublicationOutcome::Published(publication) => publication,
                _ => panic!("publication did not produce a retained witness"),
            };
        let mut retained = downgrade_publication_witness(publication.retained_witness).unwrap();
        revalidate_retained_file_witness(&mut retained).unwrap();
        drop(retained);

        let replacement_source = root.join("replacement-source.cbor");
        fs::write(&replacement_source, b"exact handoff").unwrap();
        let replacement_witness =
            read_regular_file(&root.join(final_name), &mut NamespaceBudget::new())
                .unwrap()
                .witness;
        fs::rename(&replacement_source, root.join(final_name)).unwrap();
        assert!(downgrade_publication_witness(replacement_witness).is_err());
        drop(hold);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_immutable_publication_is_no_replace() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-macos-no-replace-{}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let parent_hold = open_real_directory_hold(&root).unwrap();
        let final_name = Path::new("final.cbor");
        assert!(matches!(
            publish_new_immutable_file(&root, &parent_hold, final_name, b"first").unwrap(),
            ImmutablePublicationOutcome::Published(_)
        ));
        assert!(matches!(
            publish_new_immutable_file(&root, &parent_hold, final_name, b"second").unwrap(),
            ImmutablePublicationOutcome::Conflict
        ));
        use std::os::unix::fs::PermissionsExt;
        eprintln!(
            "final_mode={:o}",
            fs::metadata(root.join(final_name))
                .unwrap()
                .permissions()
                .mode()
        );
        assert_eq!(fs::read(root.join(final_name)).unwrap(), b"first");
        drop(parent_hold);
        fs::remove_file(root.join(final_name)).unwrap();
        fs::remove_dir(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_publication_after_visibility_fault_preserves_visible_bytes() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("evidence-registry-macos-visible-{sequence}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let hold = open_real_directory_hold(&root).unwrap();
        assert!(matches!(
            publish_new_immutable_file_with_hooks(
                &root,
                &hold,
                Path::new("final.cbor"),
                b"visible",
                || {},
                || Err(())
            )
            .unwrap(),
            ImmutablePublicationOutcome::VisibleReceiptUncertain
        ));
        assert_eq!(fs::read(root.join("final.cbor")).unwrap(), b"visible");
        drop(hold);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_publication_post_visibility_witness_drift_is_uncertain_and_retained() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("evidence-registry-macos-witness-drift-{sequence}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let hold = open_real_directory_hold(&root).unwrap();
        assert!(matches!(
            publish_new_immutable_file_with_hooks(
                &root,
                &hold,
                Path::new("final.cbor"),
                b"original",
                || {},
                || {
                    fs::write(root.join("final.cbor"), b"mutated!").unwrap();
                    Ok(())
                }
            )
            .unwrap(),
            ImmutablePublicationOutcome::VisibleReceiptUncertain
        ));
        assert_eq!(fs::read(root.join("final.cbor")).unwrap(), b"mutated!");
        drop(hold);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_publication_preserves_dangling_final_symlink() {
        use std::os::unix::fs::symlink;
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("evidence-registry-macos-dangling-{sequence}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        symlink("missing-competitor", root.join("final.cbor")).unwrap();
        let hold = open_real_directory_hold(&root).unwrap();
        assert!(matches!(
            publish_new_immutable_file(&root, &hold, Path::new("final.cbor"), b"candidate")
                .unwrap(),
            ImmutablePublicationOutcome::Conflict
        ));
        assert_eq!(
            fs::read_link(root.join("final.cbor")).unwrap(),
            Path::new("missing-competitor")
        );
        drop(hold);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_publication_rejects_moved_parent_without_deleting_decoy_temp() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("evidence-registry-macos-parent-move-{sequence}"));
        let displaced = root.with_extension("old");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&displaced);
        fs::create_dir(&root).unwrap();
        let hold = open_real_directory_hold(&root).unwrap();
        let decoy_temp = format!(
            ".evidence-registry-publish-{}-{}.tmp",
            std::process::id(),
            sequence + 1
        );
        assert!(publish_new_immutable_file_with_hook(
            &root,
            &hold,
            Path::new("final.cbor"),
            b"candidate",
            || {
                fs::rename(&root, &displaced).unwrap();
                fs::create_dir(&root).unwrap();
                fs::write(root.join(&decoy_temp), b"competitor").unwrap();
            }
        )
        .is_err());
        assert!(!displaced.join("final.cbor").exists());
        assert!(!root.join("final.cbor").exists());
        assert_eq!(fs::read(root.join(&decoy_temp)).unwrap(), b"competitor");
        drop(hold);
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(displaced).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_admitted_link_count_rejects_symlink_residue_and_accepts_exact_pair() {
        use std::os::unix::fs::symlink;
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("evidence-registry-macos-link-count-{sequence}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let final_path = root.join("final.cbor");
        let temp = root.join(".evidence-registry-publish-1-1.tmp");
        fs::write(&final_path, b"exact pair").unwrap();
        fs::hard_link(&final_path, root.join("external")).unwrap();
        symlink(&final_path, &temp).unwrap();
        let file = open_identity_handle_for_regular_file(&final_path).unwrap();
        assert!(!metadata_has_admitted_link_count(
            &final_path,
            &file.metadata().unwrap(),
            &file
        ));
        fs::remove_file(root.join("external")).unwrap();
        fs::remove_file(&temp).unwrap();
        fs::hard_link(&final_path, &temp).unwrap();
        assert!(metadata_has_admitted_link_count(
            &final_path,
            &file.metadata().unwrap(),
            &file
        ));
        assert_eq!(fs::read(&final_path).unwrap(), b"exact pair");
        assert_eq!(fs::read(&temp).unwrap(), b"exact pair");
        drop(file);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_identity_opener_rejects_a_symlink() {
        use std::os::unix::fs::symlink;

        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-macos-nofollow-{}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let regular = root.join("regular.cbor");
        let alias = root.join("alias.cbor");
        fs::write(&regular, b"exact bytes").unwrap();
        symlink(&regular, &alias).unwrap();
        assert!(open_identity_handle_for_regular_file(&regular).is_ok());
        assert!(open_identity_handle_for_regular_file(&alias).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_selected_embedded_source_rejects_a_symlink_leaf() {
        use std::os::unix::fs::symlink;

        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-macos-selected-source-nofollow-{}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let outside = root.join("outside");
        let alias = root.join("alias");
        fs::write(&outside, b"outside bytes").unwrap();
        symlink(&outside, &alias).unwrap();

        assert!(matches!(
            collect_selected_embedded_source(&root),
            Err(SelectedEmbeddedFreezePreparationError::SourceInvalid)
        ));

        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_read_regular_file_rejects_a_symlink() {
        use std::os::unix::fs::symlink;

        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-macos-read-nofollow-{}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let regular = root.join("regular.cbor");
        let alias = root.join("alias.cbor");
        fs::write(&regular, b"exact bytes").unwrap();
        symlink(&regular, &alias).unwrap();
        assert_eq!(
            read_regular_file(&regular, &mut NamespaceBudget::new())
                .unwrap()
                .bytes,
            b"exact bytes"
        );
        assert!(read_regular_file(&alias, &mut NamespaceBudget::new()).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_revalidation_rejects_recreated_root_with_original_children() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-linux-root-continuity-{sequence}-{}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("registry")).unwrap();
        fs::create_dir(root.join("journal")).unwrap();
        fs::create_dir(root.join("records")).unwrap();
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let storage_capability_class_id = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let environment_observation_id = RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
        let genesis = GenesisRecord::new(GenesisRecordInput {
            registry_id,
            journal_format_version: 1,
            record_identity_profile_id: 1,
            storage_capability_class_id,
            environment_observation_id,
            created_by_tool_version: "linux-root-continuity-test".to_owned(),
        })
        .unwrap();
        let entry = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from(genesis.record_id().as_bytes().as_slice()).unwrap(),
            storage_capability_class_id,
            environment_observation_id,
        );
        fs::write(
            root.join("registry/genesis.cbor"),
            genesis.authoritative_cbor(),
        )
        .unwrap();
        fs::write(
            root.join("journal/00000000000000000000.cbor"),
            entry.authoritative_cbor(),
        )
        .unwrap();
        let store = AuthoritativeRegistryStore::open(&root).unwrap();
        let displaced = root.with_extension("displaced");
        fs::rename(&root, &displaced).unwrap();
        fs::create_dir(&root).unwrap();
        for child in ["registry", "journal", "records"] {
            fs::rename(displaced.join(child), root.join(child)).unwrap();
        }
        assert!(store.revalidate_retained_generation().is_err());
        drop(store);
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(displaced).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_reload_rejects_transient_same_byte_genesis_inode_substitution() {
        let fixture = LinuxGenesisFixture::new("reload-transient-inode");
        let original_bytes = fs::read(&fixture.genesis).unwrap();
        let displaced = fixture
            .root
            .join("original-genesis-held-outside-registry.cbor");
        let mut store = AuthoritativeRegistryStore::open(&fixture.root).unwrap();
        let result = store.reload_authoritative_namespaces_with_hooks(
            || {
                fs::rename(&fixture.genesis, &displaced).unwrap();
                fs::write(&fixture.genesis, &original_bytes).unwrap();
            },
            || {
                fs::remove_file(&fixture.genesis).unwrap();
                fs::rename(&displaced, &fixture.genesis).unwrap();
            },
        );
        assert_eq!(
            result,
            Err(AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged,
            ))
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_reload_rejects_each_recreated_fixed_child_namespace() {
        for child in ["registry", "journal", "records"] {
            let fixture = LinuxGenesisFixture::new(child);
            let original = fixture.root.join(child);
            let displaced = fixture.root.join(format!("{child}-original"));
            let mut store = AuthoritativeRegistryStore::open(&fixture.root).unwrap();
            let result = store.reload_authoritative_namespaces_with_hook(|| {
                fs::rename(&original, &displaced).unwrap();
                fs::create_dir(&original).unwrap();
                if child == "registry" {
                    fs::copy(
                        displaced.join("genesis.cbor"),
                        original.join("genesis.cbor"),
                    )
                    .unwrap();
                } else if child == "journal" {
                    fs::copy(
                        displaced.join("00000000000000000000.cbor"),
                        original.join("00000000000000000000.cbor"),
                    )
                    .unwrap();
                }
            });
            assert_eq!(
                result,
                Err(AuthoritativeReviewAdmissionAcceptanceError::Store(
                    AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged,
                )),
                "{child} replacement must invalidate the retained namespace"
            );
            drop(store);
            fs::remove_dir_all(&original).unwrap();
            fs::rename(&displaced, &original).unwrap();
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_acceptance_rejects_retained_genesis_replacement_before_reload() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-linux-acceptance-generation-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("registry")).unwrap();
        fs::create_dir(root.join("journal")).unwrap();
        fs::create_dir(root.join("records")).unwrap();
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let storage_capability_class_id = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let environment_observation_id = RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
        let genesis_record = GenesisRecord::new(GenesisRecordInput {
            registry_id,
            journal_format_version: 1,
            record_identity_profile_id: 1,
            storage_capability_class_id,
            environment_observation_id,
            created_by_tool_version: "linux-acceptance-generation-test".to_owned(),
        })
        .unwrap();
        let genesis_entry = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from(genesis_record.record_id().as_bytes().as_slice()).unwrap(),
            storage_capability_class_id,
            environment_observation_id,
        );
        let genesis_path = root.join("registry/genesis.cbor");
        fs::write(&genesis_path, genesis_record.authoritative_cbor()).unwrap();
        fs::write(
            root.join("journal/00000000000000000000.cbor"),
            genesis_entry.authoritative_cbor(),
        )
        .unwrap();
        let mut store = AuthoritativeRegistryStore::open(&root).unwrap();

        let genesis_reference = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(0_u64).unwrap(),
            genesis_entry.entry_hash(),
            EventTypeId::try_from(1_u64).unwrap(),
            EventRecordId::try_from(genesis_record.record_id().as_bytes().as_slice()).unwrap(),
        );
        let accepted = store
            .accept_authoritative_review_admission(
                genesis_reference.clone(),
                &[],
                genesis_reference.clone(),
                &[],
            )
            .unwrap();
        drop(accepted);

        let replacement = GenesisRecord::new(GenesisRecordInput {
            registry_id,
            journal_format_version: 1,
            record_identity_profile_id: 1,
            storage_capability_class_id,
            environment_observation_id,
            created_by_tool_version: "linux-acceptance-generation-tesu".to_owned(),
        })
        .unwrap();
        assert_eq!(
            replacement.authoritative_cbor().len(),
            genesis_record.authoritative_cbor().len()
        );
        fs::write(&genesis_path, replacement.authoritative_cbor()).unwrap();

        assert!(matches!(
            store.accept_authoritative_review_admission(
                genesis_reference.clone(),
                &[],
                genesis_reference,
                &[],
            ),
            Err(AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged
            ))
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_acceptance_rejects_same_byte_genesis_inode_replacement_before_reload() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-linux-acceptance-inode-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("registry")).unwrap();
        fs::create_dir(root.join("journal")).unwrap();
        fs::create_dir(root.join("records")).unwrap();
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let storage_capability_class_id = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let environment_observation_id = RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
        let genesis_record = GenesisRecord::new(GenesisRecordInput {
            registry_id,
            journal_format_version: 1,
            record_identity_profile_id: 1,
            storage_capability_class_id,
            environment_observation_id,
            created_by_tool_version: "linux-acceptance-inode-test".to_owned(),
        })
        .unwrap();
        let genesis_entry = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from(genesis_record.record_id().as_bytes().as_slice()).unwrap(),
            storage_capability_class_id,
            environment_observation_id,
        );
        let genesis_path = root.join("registry/genesis.cbor");
        let genesis_bytes = genesis_record.authoritative_cbor();
        fs::write(&genesis_path, &genesis_bytes).unwrap();
        fs::write(
            root.join("journal/00000000000000000000.cbor"),
            genesis_entry.authoritative_cbor(),
        )
        .unwrap();
        let mut store = AuthoritativeRegistryStore::open(&root).unwrap();

        let replacement_path = root.join("registry/replacement.cbor");
        fs::write(&replacement_path, &genesis_bytes).unwrap();
        fs::rename(&replacement_path, &genesis_path).unwrap();

        let genesis_reference = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(0_u64).unwrap(),
            genesis_entry.entry_hash(),
            EventTypeId::try_from(1_u64).unwrap(),
            EventRecordId::try_from(genesis_record.record_id().as_bytes().as_slice()).unwrap(),
        );
        assert!(matches!(
            store.accept_authoritative_review_admission(
                genesis_reference.clone(),
                &[],
                genesis_reference,
                &[],
            ),
            Err(AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged
            ))
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn authoritative_publication_lock_serializes_terminal_writers() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-publication-lock-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let root_hold = open_real_directory_hold(&root).unwrap();

        let first = acquire_authoritative_publication_lock(&root, &root_hold).unwrap();
        assert!(acquire_authoritative_publication_lock(&root, &root_hold).is_err());
        drop(first);
        let second = acquire_authoritative_publication_lock(&root, &root_hold).unwrap();
        drop(second);

        fs::remove_file(root.join(".evidence-registry-publication.lock")).unwrap();
        drop(root_hold);
        fs::remove_dir(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_reload_rejects_same_registry_namespace_replacement_after_precheck() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-linux-reload-replacement-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("registry")).unwrap();
        fs::create_dir(root.join("journal")).unwrap();
        fs::create_dir(root.join("records")).unwrap();
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let storage_capability_class_id = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let environment_observation_id = RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
        let genesis_record = GenesisRecord::new(GenesisRecordInput {
            registry_id,
            journal_format_version: 1,
            record_identity_profile_id: 1,
            storage_capability_class_id,
            environment_observation_id,
            created_by_tool_version: "linux-reload-replacement-test".to_owned(),
        })
        .unwrap();
        let genesis_entry = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from(genesis_record.record_id().as_bytes().as_slice()).unwrap(),
            storage_capability_class_id,
            environment_observation_id,
        );
        let genesis_bytes = genesis_record.authoritative_cbor();
        let entry_bytes = genesis_entry.authoritative_cbor();
        fs::write(root.join("registry/genesis.cbor"), &genesis_bytes).unwrap();
        fs::write(root.join("journal/00000000000000000000.cbor"), &entry_bytes).unwrap();
        let mut store = AuthoritativeRegistryStore::open(&root).unwrap();
        let displaced = root.with_extension("displaced");
        assert!(matches!(
            store.reload_authoritative_namespaces_with_hook(|| {
                fs::rename(&root, &displaced).unwrap();
                fs::create_dir_all(root.join("registry")).unwrap();
                fs::create_dir(root.join("journal")).unwrap();
                fs::create_dir(root.join("records")).unwrap();
                fs::write(root.join("registry/genesis.cbor"), &genesis_bytes).unwrap();
                fs::write(root.join("journal/00000000000000000000.cbor"), &entry_bytes).unwrap();
            }),
            Err(AuthoritativeReviewAdmissionAcceptanceError::Store(
                AuthoritativeRegistryStoreOpenError::RetainedGenerationChanged
            ))
        ));
        drop(store);
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(displaced).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_publication_lock_prevents_lockfile_inode_split() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-linux-publication-lock-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let root_hold = open_real_directory_hold(&root).unwrap();
        let lock_path = root.join(".evidence-registry-publication.lock");
        fs::write(&lock_path, b"decoy-original").unwrap();

        let first = acquire_authoritative_publication_lock(&root, &root_hold).unwrap();
        assert!(acquire_authoritative_publication_lock(&root, &root_hold).is_err());
        fs::rename(&lock_path, root.join("displaced.lock")).unwrap();
        fs::write(&lock_path, b"replacement").unwrap();
        assert!(acquire_authoritative_publication_lock(&root, &root_hold).is_err());
        drop(first);
        let second = acquire_authoritative_publication_lock(&root, &root_hold).unwrap();
        drop(second);
        fs::remove_file(&lock_path).unwrap();
        fs::remove_file(root.join("displaced.lock")).unwrap();
        drop(root_hold);
        fs::remove_dir(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    struct OwnedProtocolChild {
        child: std::process::Child,
    }

    #[cfg(target_os = "linux")]
    struct OwnedForkChild {
        pid: i32,
        reaped: bool,
    }
    #[cfg(target_os = "linux")]
    struct OwnedFd(i32);
    #[cfg(target_os = "linux")]
    impl Drop for OwnedFd {
        fn drop(&mut self) {
            unsafe extern "C" {
                fn close(fd: i32) -> i32;
            }
            if self.0 >= 0 {
                let _ = unsafe { close(self.0) };
                self.0 = -1;
            }
        }
    }
    #[cfg(target_os = "linux")]
    impl OwnedForkChild {
        fn reap_by(&mut self, deadline: std::time::Instant) -> Result<i32, ()> {
            unsafe extern "C" {
                fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
            }
            let mut status = 0;
            loop {
                if unsafe { waitpid(self.pid, &mut status, 1) } == self.pid {
                    self.reaped = true;
                    return Ok(status);
                }
                if std::time::Instant::now() >= deadline {
                    return Err(());
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        fn kill_and_reap_by(&mut self, deadline: std::time::Instant) -> Result<i32, ()> {
            unsafe extern "C" {
                fn kill(pid: i32, signal: i32) -> i32;
            }
            if self.reaped {
                return Ok(0);
            }
            if unsafe { kill(self.pid, 9) } != 0
                && std::io::Error::last_os_error().raw_os_error() != Some(3)
            {
                return Err(());
            }
            self.reap_by(deadline)
        }
    }
    #[cfg(target_os = "linux")]
    impl Drop for OwnedForkChild {
        fn drop(&mut self) {
            if !self.reaped
                && self
                    .kill_and_reap_by(std::time::Instant::now() + std::time::Duration::from_secs(1))
                    .is_err()
            {
                if std::thread::panicking() {
                    eprintln!("owned fork child cleanup failed during unwind");
                } else {
                    panic!("owned fork child cleanup failed");
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    impl Drop for OwnedProtocolChild {
        fn drop(&mut self) {
            if self.child.try_wait().ok().flatten().is_none() {
                let result = self.kill_and_reap_by(
                    std::time::Instant::now() + std::time::Duration::from_secs(1),
                );
                if result.is_err() {
                    if std::thread::panicking() {
                        eprintln!("owned protocol child cleanup failed during unwind");
                    } else {
                        panic!("owned protocol child cleanup failed");
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    impl OwnedProtocolChild {
        fn kill_and_reap_by(
            &mut self,
            deadline: std::time::Instant,
        ) -> Result<std::process::ExitStatus, ()> {
            if let Some(status) = self.child.try_wait().map_err(|_| ())? {
                return Ok(status);
            }
            match self.child.kill() {
                Ok(()) => {}
                Err(error) if error.raw_os_error() == Some(3) => {}
                Err(_) => return Err(()),
            }
            self.reap_by(deadline)
        }

        fn reap_by(
            &mut self,
            deadline: std::time::Instant,
        ) -> Result<std::process::ExitStatus, ()> {
            loop {
                if let Some(status) = self.child.try_wait().map_err(|_| ())? {
                    return Ok(status);
                }
                if std::time::Instant::now() >= deadline {
                    return Err(());
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn protocol_line_by(
        output: &mut std::process::ChildStdout,
        deadline: std::time::Instant,
    ) -> Result<String, ()> {
        use std::io::Read;
        use std::os::fd::AsRawFd;
        unsafe extern "C" {
            fn poll(fds: *mut PollFd, nfds: usize, timeout: i32) -> i32;
            fn fcntl(fd: i32, command: i32, argument: i32) -> i32;
        }
        #[repr(C)]
        struct PollFd {
            fd: i32,
            events: i16,
            revents: i16,
        }
        const F_GETFL: i32 = 3;
        const F_SETFL: i32 = 4;
        const O_NONBLOCK: i32 = 0x800;
        let raw_fd = output.as_raw_fd();
        let flags = unsafe { fcntl(raw_fd, F_GETFL, 0) };
        if flags < 0 || unsafe { fcntl(raw_fd, F_SETFL, flags | O_NONBLOCK) } < 0 {
            return Err(());
        }
        let mut fd = PollFd {
            fd: raw_fd,
            events: 1,
            revents: 0,
        };
        let millis = deadline
            .saturating_duration_since(std::time::Instant::now())
            .as_millis()
            .min(i32::MAX as u128) as i32;
        if unsafe { poll(&mut fd, 1, millis) } <= 0 {
            return Err(());
        }
        let mut bytes = Vec::new();
        loop {
            let mut chunk = [0_u8; 16];
            match output.read(&mut chunk) {
                Ok(0) => return Err(()),
                Ok(count) => bytes.extend_from_slice(&chunk[..count]),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => return Err(()),
            }
            if bytes.len() > 64 {
                return Err(());
            }
            if bytes.last() == Some(&b'\n') {
                return std::str::from_utf8(&bytes)
                    .map(str::to_owned)
                    .map_err(|_| ());
            }
            let remaining = deadline
                .saturating_duration_since(std::time::Instant::now())
                .as_millis()
                .min(i32::MAX as u128) as i32;
            if remaining == 0 || unsafe { poll(&mut fd, 1, remaining) } <= 0 {
                return Err(());
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn protocol_command_by(
        input: &mut std::process::ChildStdin,
        command: u8,
        deadline: std::time::Instant,
    ) -> Result<(), ()> {
        use std::io::Write;
        use std::os::fd::AsRawFd;
        unsafe extern "C" {
            fn poll(fds: *mut PollFd, nfds: usize, timeout: i32) -> i32;
            fn fcntl(fd: i32, command: i32, argument: i32) -> i32;
        }
        #[repr(C)]
        struct PollFd {
            fd: i32,
            events: i16,
            revents: i16,
        }
        let raw_fd = input.as_raw_fd();
        let flags = unsafe { fcntl(raw_fd, 3, 0) };
        if flags < 0 || unsafe { fcntl(raw_fd, 4, flags | 0x800) } < 0 {
            return Err(());
        }
        loop {
            let millis = deadline
                .saturating_duration_since(std::time::Instant::now())
                .as_millis()
                .min(i32::MAX as u128) as i32;
            let mut fd = PollFd {
                fd: raw_fd,
                events: 4,
                revents: 0,
            };
            if millis == 0 || unsafe { poll(&mut fd, 1, millis) } <= 0 {
                return Err(());
            }
            match input.write(&[command]) {
                Ok(1) => return Ok(()),
                Ok(_) => return Err(()),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(_) => return Err(()),
            }
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_publication_lock_is_process_isolated_and_released_on_owner_exit() {
        use std::process::{Command, Stdio};

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-linux-publication-process-{sequence}-{}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let root_hold = open_real_directory_hold(&root).unwrap();
        let owner = acquire_authoritative_publication_lock(&root, &root_hold).unwrap();
        let script = r#"import fcntl, os, select, sys
f=os.open(sys.argv[1],os.O_RDONLY); print('READY',flush=True)
for _ in range(2):
 r,_,_=select.select([0],[],[],1)
 if not r: os._exit(90)
 if os.read(0,1)!=b'1': os._exit(91)
 try: fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB); print('ACQUIRED',flush=True)
 except BlockingIOError: print('BLOCKED',flush=True)
"#;
        let mut owned = OwnedProtocolChild {
            child: Command::new("python3")
                .arg("-c")
                .arg(script)
                .arg(&root)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
        };
        let mut input = owned.child.stdin.take().unwrap();
        let mut output = owned.child.stdout.take().unwrap();
        assert_eq!(protocol_line_by(&mut output, deadline).unwrap(), "READY\n");
        protocol_command_by(&mut input, b'1', deadline).unwrap();
        assert_eq!(
            protocol_line_by(&mut output, deadline).unwrap(),
            "BLOCKED\n"
        );
        drop(owner);
        protocol_command_by(&mut input, b'1', deadline).unwrap();
        assert_eq!(
            protocol_line_by(&mut output, deadline).unwrap(),
            "ACQUIRED\n"
        );
        drop(input);
        drop(output);
        assert!(owned.reap_by(deadline).unwrap().success());
        drop(acquire_authoritative_publication_lock(&root, &root_hold).unwrap());

        let abnormal = r#"import fcntl, os, select, sys
f=os.open(sys.argv[1],os.O_RDONLY); fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB); print('READY',flush=True)
r,_,_=select.select([0],[],[],1)
if not r: os._exit(90)
if os.read(0,1)!=b'X': os._exit(91)
os._exit(23)
"#;
        let mut abnormal_owner = OwnedProtocolChild {
            child: Command::new("python3")
                .arg("-c")
                .arg(abnormal)
                .arg(&root)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
        };
        let mut abnormal_input = abnormal_owner.child.stdin.take().unwrap();
        let mut abnormal_output = abnormal_owner.child.stdout.take().unwrap();
        assert_eq!(
            protocol_line_by(&mut abnormal_output, deadline).unwrap(),
            "READY\n"
        );
        assert!(acquire_authoritative_publication_lock(&root, &root_hold).is_err());
        protocol_command_by(&mut abnormal_input, b'X', deadline).unwrap();
        drop(abnormal_input);
        drop(abnormal_output);
        assert_eq!(abnormal_owner.reap_by(deadline).unwrap().code(), Some(23));
        drop(acquire_authoritative_publication_lock(&root, &root_hold).unwrap());
        drop(root_hold);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_supported_store_enforces_hardlink_alias_and_ordinary_writer_controls() {
        use std::io::{Seek, SeekFrom, Write};
        use std::os::unix::fs::{symlink, PermissionsExt};

        let positive = LinuxGenesisFixture::new("temp-hardlink-positive");
        fs::hard_link(
            &positive.genesis,
            positive
                .root
                .join("registry/.evidence-registry-publish-1-1.tmp"),
        )
        .unwrap();
        assert!(AuthoritativeRegistryStore::open(&positive.root).is_ok());

        let external = LinuxGenesisFixture::new("external-hardlink-negative");
        fs::hard_link(&external.genesis, external.root.join("external-alias.cbor")).unwrap();
        assert!(AuthoritativeRegistryStore::open(&external.root).is_err());

        let symlink_alias = LinuxGenesisFixture::new("external-plus-temp-symlink-negative");
        let external_alias = symlink_alias.root.join("external-alias.cbor");
        let temp_alias = symlink_alias
            .root
            .join("registry/.evidence-registry-publish-1-1.tmp");
        fs::hard_link(&symlink_alias.genesis, &external_alias).unwrap();
        symlink(&symlink_alias.genesis, &temp_alias).unwrap();
        assert!(AuthoritativeRegistryStore::open(&symlink_alias.root).is_err());
        assert!(external_alias.exists() && temp_alias.is_symlink());

        let permission = LinuxGenesisFixture::new("permission-denied");
        fs::set_permissions(&permission.genesis, fs::Permissions::from_mode(0o000)).unwrap();
        assert_eq!(
            fs::File::open(&permission.genesis).unwrap_err().kind(),
            std::io::ErrorKind::PermissionDenied
        );
        assert!(AuthoritativeRegistryStore::open(&permission.root).is_err());
        fs::set_permissions(&permission.genesis, fs::Permissions::from_mode(0o600)).unwrap();

        let writer = LinuxGenesisFixture::new("preexisting-writer");
        let mut fd = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&writer.genesis)
            .unwrap();
        let store = AuthoritativeRegistryStore::open(&writer.root).unwrap();
        fd.seek(SeekFrom::Start(0)).unwrap();
        fd.write_all(b"X").unwrap();
        fd.flush().unwrap();
        assert!(store.revalidate_retained_generation().is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_retained_reopen_rejects_fifo_replacement_without_blocking() {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        unsafe extern "C" {
            fn mkfifo(pathname: *const std::ffi::c_char, mode: u32) -> i32;
        }
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-linux-fifo-{sequence}-{}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let path = root.join("authority.cbor");
        let bytes = b"known authority bytes";
        fs::write(&path, bytes).unwrap();
        let source = RetainedFileWitness {
            path: path.clone(),
            file: OpenOptions::new().read(true).open(&path).unwrap(),
            expected_length: bytes.len(),
            expected_sha256: Sha256::digest(bytes).into(),
        };
        fs::remove_file(&path).unwrap();
        let raw = CString::new(path.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { mkfifo(raw.as_ptr(), 0o600) }, 0);
        assert!(open_retained_file_guard(&source).is_err());
        drop(source);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_reserved_temp_symlink_cannot_admit_external_hardlink() {
        use std::os::unix::fs::symlink;

        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-linux-link-admission-{sequence}-{}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let genesis = root.join("genesis.cbor");
        fs::write(&genesis, b"exact genesis bytes").unwrap();
        fs::hard_link(&genesis, root.join("external-alias.cbor")).unwrap();
        symlink(&genesis, root.join(".evidence-registry-publish-1-1.tmp")).unwrap();
        let file = OpenOptions::new().read(true).open(&genesis).unwrap();
        let metadata = file.metadata().unwrap();
        assert!(!metadata_has_admitted_link_count(
            &genesis, &metadata, &file
        ));
        drop(file);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "causal RED driver: inherited flock OFD retains lock after parent close"]
    fn linux_publication_lock_inherited_ofd_releases_with_owner_scope_red() {
        use std::os::fd::AsRawFd;
        unsafe extern "C" {
            fn fork() -> i32;
            fn pipe(fds: *mut i32) -> i32;
            fn read(fd: i32, b: *mut u8, n: usize) -> isize;
            fn write(fd: i32, b: *const u8, n: usize) -> isize;
            fn close(fd: i32) -> i32;
            fn _exit(status: i32) -> !;

            fn poll(fds: *mut PollFd, nfds: usize, timeout: i32) -> i32;
            fn fcntl(fd: i32, command: i32, argument: i32) -> i32;
        }
        #[repr(C)]
        struct PollFd {
            fd: i32,
            events: i16,
            revents: i16,
        }
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-linux-inherited-ofd-{sequence}-{}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let root_hold = open_real_directory_hold(&root).unwrap();
        let owner = acquire_authoritative_publication_lock(&root, &root_hold).unwrap();
        let owner_fd = owner.as_raw_fd();
        let flags = unsafe { fcntl(owner_fd, 1, 0) };
        let metadata = owner.file.metadata().unwrap();
        let mut ready = [0; 2];
        let mut release = [0; 2];
        assert_eq!(unsafe { pipe(ready.as_mut_ptr()) }, 0);
        assert_eq!(unsafe { pipe(release.as_mut_ptr()) }, 0);
        let pid = unsafe { fork() };
        assert!(pid >= 0);
        if pid == 0 {
            unsafe {
                close(ready[0]);
                close(release[1]);
            }
            let marker = *b"R";
            if unsafe { write(ready[1], marker.as_ptr(), 1) } != 1 {
                unsafe { _exit(92) }
            };
            let mut release_fd = PollFd {
                fd: release[0],
                events: 1,
                revents: 0,
            };
            if unsafe { poll(&mut release_fd, 1, 250) } <= 0 {
                unsafe { _exit(93) }
            }
            let mut command = [0];
            if unsafe { read(release[0], command.as_mut_ptr(), 1) } != 1 {
                unsafe { _exit(93) }
            };
            unsafe { _exit(0) }
        }
        let mut fork_child = OwnedForkChild { pid, reaped: false };
        let _ready_read = OwnedFd(ready[0]);
        let _release_write = OwnedFd(release[1]);
        unsafe {
            close(ready[1]);
            close(release[0]);
        }
        let mut fd = PollFd {
            fd: ready[0],
            events: 1,
            revents: 0,
        };
        assert!(unsafe { poll(&mut fd, 1, 250) } > 0);
        let mut marker = [0];
        assert_eq!(unsafe { read(ready[0], marker.as_mut_ptr(), 1) }, 1);
        assert_eq!(marker, *b"R");
        eprintln!(
            "inherited-ofd causal probe pid={} child={} fd={} fdflags={} metadata={:?}",
            std::process::id(),
            pid,
            owner_fd,
            flags,
            metadata
        );
        drop(owner);
        let contention = acquire_authoritative_publication_lock(&root, &root_hold);
        let released = *b"X";
        assert_eq!(unsafe { write(release[1], released.as_ptr(), 1) }, 1);
        let status = fork_child
            .reap_by(std::time::Instant::now() + std::time::Duration::from_millis(250))
            .unwrap_or_else(|_| {
                fork_child
                    .kill_and_reap_by(
                        std::time::Instant::now() + std::time::Duration::from_millis(250),
                    )
                    .unwrap()
            });
        assert_eq!(
            status, 0,
            "inherited child must exit normally, not by forced kill"
        );
        assert!(
            contention.is_ok(),
            "inherited child OFD retained the flock after parent owner drop"
        );
        drop(root_hold);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "bounded parent worker"]
    fn linux_inherited_ofd_parent_fault_worker_reaps_owned_child() {
        unsafe extern "C" {
            fn fork() -> i32;
            fn pipe(fds: *mut i32) -> i32;
            fn write(fd: i32, b: *const u8, n: usize) -> isize;
            fn poll(fds: *mut PollFd, nfds: usize, timeout: i32) -> i32;
            fn read(fd: i32, b: *mut u8, n: usize) -> isize;
            fn close(fd: i32) -> i32;
            fn _exit(status: i32) -> !;
            fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
        }
        #[repr(C)]
        struct PollFd {
            fd: i32,
            events: i16,
            revents: i16,
        }
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-fork-fault-{}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let root_hold = open_real_directory_hold(&root).unwrap();
        let pid = std::cell::Cell::new(-1);
        let ready_seen = std::cell::Cell::new(false);
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _owner = acquire_authoritative_publication_lock(&root, &root_hold).unwrap();
            let mut ready = [0; 2];
            assert_eq!(unsafe { pipe(ready.as_mut_ptr()) }, 0);
            let child = unsafe { fork() };
            if child == 0 {
                unsafe {
                    close(ready[0]);
                }
                let marker = *b"R";
                if unsafe { write(ready[1], marker.as_ptr(), 1) } != 1 {
                    unsafe { _exit(92) }
                };
                let mut hold = PollFd {
                    fd: -1,
                    events: 0,
                    revents: 0,
                };
                unsafe { poll(&mut hold, 1, 60_000) };
                unsafe { _exit(93) }
            }
            assert!(child > 0);
            pid.set(child);
            let _read = OwnedFd(ready[0]);
            let _write = OwnedFd(ready[1]);
            let _child = OwnedForkChild {
                pid: child,
                reaped: false,
            };
            let mut fd = PollFd {
                fd: ready[0],
                events: 1,
                revents: 0,
            };
            assert!(unsafe { poll(&mut fd, 1, 250) } > 0);
            let mut marker = [0];
            assert_eq!(unsafe { read(ready[0], marker.as_mut_ptr(), 1) }, 1);
            assert_eq!(marker, *b"R");
            ready_seen.set(true);
            panic!("deterministic parent fault after READY");
        }));
        assert!(ready_seen.get(), "worker never reached READY");
        assert_eq!(
            panic.unwrap_err().downcast_ref::<&str>(),
            Some(&"deterministic parent fault after READY")
        );
        let mut status = 0;
        assert_eq!(unsafe { waitpid(pid.get(), &mut status, 1) }, -1);
        assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(10));
        drop(acquire_authoritative_publication_lock(&root, &root_hold).unwrap());
        drop(root_hold);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "causal RED driver: post-lock error must unlock retained OFD duplicate"]
    fn linux_publication_lock_post_lock_error_releases_retained_ofd_red() {
        use std::cell::RefCell;
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-linux-postlock-ofd-{sequence}-{}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let root_hold = open_real_directory_hold(&root).unwrap();
        let duplicate = RefCell::new(None);
        assert!(acquire_authoritative_publication_lock_with_post_lock_hook(
            &root,
            &root_hold,
            |file| {
                *duplicate.borrow_mut() = Some(file.try_clone().map_err(|_| ())?);
                Err(())
            }
        )
        .is_err());
        let acquired = acquire_authoritative_publication_lock(&root, &root_hold).is_ok();
        drop(duplicate.into_inner());
        assert!(acquired, "post-lock error retained a duplicated OFD flock");
        drop(root_hold);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_publication_lock_stalled_ready_times_out_and_reaps_owned_child() {
        use std::process::{Command, Stdio};

        let started = std::time::Instant::now();
        let mut owned = OwnedProtocolChild {
            child: Command::new("python3")
                .arg("-c")
                .arg("import time; time.sleep(60)")
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
        };
        let mut output = owned.child.stdout.take().unwrap();
        assert!(
            protocol_line_by(&mut output, started + std::time::Duration::from_millis(50)).is_err()
        );
        drop(output);
        let status = owned
            .kill_and_reap_by(started + std::time::Duration::from_secs(1))
            .unwrap();
        assert!(!status.success());
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "bounded parent worker"]
    fn linux_open_root_fifo_without_writer_worker() {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        unsafe extern "C" {
            fn mkfifo(pathname: *const std::ffi::c_char, mode: u32) -> i32;
        }
        let fixture = LinuxGenesisFixture::new("root-fifo-red");
        fs::remove_dir_all(&fixture.root).unwrap();
        let raw = CString::new(fixture.root.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { mkfifo(raw.as_ptr(), 0o600) }, 0);
        match AuthoritativeRegistryStore::open(&fixture.root) {
            Err(error) => assert_eq!(
                error,
                AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid
            ),
            Ok(_) => panic!("root FIFO unexpectedly opened"),
        };
        fs::remove_file(&fixture.root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "bounded parent worker"]
    fn linux_open_genesis_fifo_without_writer_worker() {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        unsafe extern "C" {
            fn mkfifo(pathname: *const std::ffi::c_char, mode: u32) -> i32;
        }
        let fixture = LinuxGenesisFixture::new("genesis-fifo-red");
        fs::remove_file(&fixture.genesis).unwrap();
        let raw = CString::new(fixture.genesis.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { mkfifo(raw.as_ptr(), 0o600) }, 0);
        match AuthoritativeRegistryStore::open(&fixture.root) {
            Err(error) => assert_eq!(
                error,
                AuthoritativeRegistryStoreOpenError::RegistryNamespaceInvalid
            ),
            Ok(_) => panic!("genesis FIFO unexpectedly opened"),
        };
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "bounded parent worker"]
    fn linux_open_fixed_child_fifo_without_writer_worker() {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        unsafe extern "C" {
            fn mkfifo(pathname: *const std::ffi::c_char, mode: u32) -> i32;
        }
        for name in ["registry", "journal", "records"] {
            let fixture = LinuxGenesisFixture::new(name);
            let path = fixture.root.join(name);
            fs::remove_dir_all(&path).unwrap();
            let raw = CString::new(path.as_os_str().as_bytes()).unwrap();
            assert_eq!(unsafe { mkfifo(raw.as_ptr(), 0o600) }, 0);
            let expected = match name {
                "registry" => AuthoritativeRegistryStoreOpenError::RegistryNamespaceInvalid,
                "journal" => AuthoritativeRegistryStoreOpenError::JournalNamespaceInvalid,
                "records" => AuthoritativeRegistryStoreOpenError::RecordNamespaceInvalid,
                _ => unreachable!(),
            };
            match AuthoritativeRegistryStore::open(&fixture.root) {
                Err(error) => assert_eq!(error, expected, "{name}"),
                Ok(_) => panic!("{name} FIFO unexpectedly opened"),
            };
        }
    }

    #[cfg(target_os = "linux")]
    fn run_ignored_linux_worker(name: &str) -> Result<(), ()> {
        use std::fs::OpenOptions;
        use std::process::{Command, Stdio};
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let logs = std::env::current_dir()
            .map_err(|_| ())?
            .join("target/cross-platform/linux/successor/worker-logs");
        fs::create_dir_all(&logs).map_err(|_| ())?;
        let stem = format!(
            "{}-{sequence}-{}",
            name.replace(':', "_"),
            std::process::id()
        );
        let stdout_path = logs.join(format!("{stem}.stdout"));
        let stderr_path = logs.join(format!("{stem}.stderr"));
        let stdout = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&stdout_path)
            .map_err(|_| ())?;
        let stderr = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&stderr_path)
            .map_err(|_| ())?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
        let mut owned = OwnedProtocolChild {
            child: Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg(name)
                .arg("--ignored")
                .stdout(Stdio::from(stdout))
                .stderr(Stdio::from(stderr))
                .spawn()
                .map_err(|_| ())?,
        };
        match owned.reap_by(deadline) {
            Ok(status) if status.success() => {}
            Ok(_) => return Err(()),
            Err(()) => {
                owned.kill_and_reap_by(
                    std::time::Instant::now() + std::time::Duration::from_secs(1),
                )?;
                return Err(());
            }
        }
        for path in [&stdout_path, &stderr_path] {
            if fs::metadata(path).map_err(|_| ())?.len() > 16 * 1024 {
                return Err(());
            }
        }
        let stdout = fs::read(&stdout_path).map_err(|_| ())?;
        let text = std::str::from_utf8(&stdout).map_err(|_| ())?;
        if !text.contains("running 1 test") || !text.contains("test result: ok. 1 passed; 0 failed")
        {
            return Err(());
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_open_fifo_workers_are_bounded_and_reject_typed_inputs() {
        for name in [
            "authoritative_store::tests::linux_open_root_fifo_without_writer_worker",
            "authoritative_store::tests::linux_open_genesis_fifo_without_writer_worker",
            "authoritative_store::tests::linux_open_fixed_child_fifo_without_writer_worker",
        ] {
            run_ignored_linux_worker(name).unwrap();
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_inherited_ofd_worker_is_bounded_in_default_suite() {
        run_ignored_linux_worker("authoritative_store::tests::linux_publication_lock_inherited_ofd_releases_with_owner_scope_red").unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_inherited_ofd_parent_fault_worker_is_bounded_in_default_suite() {
        run_ignored_linux_worker(
            "authoritative_store::tests::linux_inherited_ofd_parent_fault_worker_reaps_owned_child",
        )
        .unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_post_lock_error_worker_is_bounded_in_default_suite() {
        run_ignored_linux_worker(
            "authoritative_store::tests::linux_publication_lock_post_lock_error_releases_retained_ofd_red",
        )
        .unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_ignored_worker_supervisor_rejects_zero_match_filter() {
        assert!(
            run_ignored_linux_worker("authoritative_store::tests::no_such_ignored_worker").is_err()
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_protocol_helper_rejects_faulty_child_responses_and_reaps_them() {
        use std::process::{Command, Stdio};

        for (name, script) in [
            ("eof", "import sys; sys.stdout.flush()"),
            ("oversized", "import sys; print('X'*65,flush=True)"),
            (
                "partial",
                "import sys,time; sys.stdout.write('PART'); sys.stdout.flush(); time.sleep(60)",
            ),
            ("wrongstage", "print('WRONG',flush=True)"),
            (
                "missingresponse",
                "print('READY',flush=True); import time; time.sleep(60)",
            ),
        ] {
            let started = std::time::Instant::now();
            let mut owned = OwnedProtocolChild {
                child: Command::new("python3")
                    .arg("-c")
                    .arg(script)
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap(),
            };
            let mut output = owned.child.stdout.take().unwrap();
            let deadline = started + std::time::Duration::from_millis(75);
            if name == "missingresponse" {
                assert_eq!(protocol_line_by(&mut output, deadline).unwrap(), "READY\n");
            }
            if name == "wrongstage" {
                assert_ne!(protocol_line_by(&mut output, deadline).unwrap(), "READY\n");
            } else {
                assert!(protocol_line_by(&mut output, deadline).is_err(), "{name}");
            }
            drop(output);
            assert!(
                owned
                    .kill_and_reap_by(started + std::time::Duration::from_secs(1))
                    .is_ok(),
                "{name}"
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_abnormal_lock_oracles_detect_omission_and_ignored_exit() {
        use std::process::{Command, Stdio};
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-linux-abnormal-oracle-{sequence}-{}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let root_hold = open_real_directory_hold(&root).unwrap();
        let script = r#"import fcntl, os, select, sys, time
f=os.open(sys.argv[1],os.O_RDONLY)
if sys.argv[2] != 'omit': fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
print('READY',flush=True)
r,_,_=select.select([0],[],[],1)
if not r: os._exit(90)
if os.read(0,1)!=b'X': os._exit(91)
if sys.argv[2] == 'ignore': time.sleep(60)
os._exit(23)
"#;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
        let mut omitted = OwnedProtocolChild {
            child: Command::new("python3")
                .arg("-c")
                .arg(script)
                .arg(&root)
                .arg("omit")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
        };
        let mut omitted_input = omitted.child.stdin.take().unwrap();
        let mut omitted_output = omitted.child.stdout.take().unwrap();
        assert_eq!(
            protocol_line_by(&mut omitted_output, deadline).unwrap(),
            "READY\n"
        );
        assert!(
            acquire_authoritative_publication_lock(&root, &root_hold).is_ok(),
            "omitted flock must fail live-contention oracle"
        );
        protocol_command_by(&mut omitted_input, b'X', deadline).unwrap();
        drop(omitted_input);
        drop(omitted_output);
        assert_eq!(omitted.reap_by(deadline).unwrap().code(), Some(23));
        let mut ignored = OwnedProtocolChild {
            child: Command::new("python3")
                .arg("-c")
                .arg(script)
                .arg(&root)
                .arg("ignore")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
        };
        let mut ignored_input = ignored.child.stdin.take().unwrap();
        let mut ignored_output = ignored.child.stdout.take().unwrap();
        let short = std::time::Instant::now() + std::time::Duration::from_millis(100);
        assert_eq!(
            protocol_line_by(&mut ignored_output, short).unwrap(),
            "READY\n"
        );
        assert!(acquire_authoritative_publication_lock(&root, &root_hold).is_err());
        protocol_command_by(&mut ignored_input, b'X', short).unwrap();
        drop(ignored_input);
        drop(ignored_output);
        assert!(ignored.reap_by(short).is_err());
        assert!(!ignored
            .kill_and_reap_by(std::time::Instant::now() + std::time::Duration::from_secs(1))
            .unwrap()
            .success());
        drop(acquire_authoritative_publication_lock(&root, &root_hold).unwrap());
        drop(root_hold);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_published_witness_downgrades_exactly_and_rejects_post_publish_writer() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-linux-downgrade-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let root_hold = open_real_directory_hold(&root).unwrap();

        let publication = match publish_new_immutable_file(
            &root,
            &root_hold,
            Path::new("exact.cbor"),
            b"exact bytes",
        )
        .unwrap()
        {
            ImmutablePublicationOutcome::Published(publication) => publication,
            _ => panic!("expected Linux publication"),
        };
        assert_eq!(
            publication.facts.parent_directory_flush,
            DurabilityActionState::Performed
        );
        let mut witness = downgrade_publication_witness(publication.retained_witness).unwrap();
        revalidate_retained_file_witness(&mut witness).unwrap();
        assert_eq!(
            witness.expected_sha256,
            <[u8; ID_LENGTH]>::from(Sha256::digest(b"exact bytes"))
        );
        assert!(matches!(
            publish_new_immutable_file(&root, &root_hold, Path::new("exact.cbor"), b"competitor")
                .unwrap(),
            ImmutablePublicationOutcome::Conflict
        ));

        let changed_publication = match publish_new_immutable_file(
            &root,
            &root_hold,
            Path::new("changed.cbor"),
            b"changed bytes",
        )
        .unwrap()
        {
            ImmutablePublicationOutcome::Published(publication) => publication,
            _ => panic!("expected Linux publication"),
        };
        let mut writer = OpenOptions::new()
            .write(true)
            .open(root.join("changed.cbor"))
            .unwrap();
        writer.write_all(b"X").unwrap();
        writer.sync_all().unwrap();
        drop(writer);
        assert!(downgrade_publication_witness(changed_publication.retained_witness).is_err());

        drop(witness);
        drop(root_hold);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_held_handle_promotion_is_single_name_and_no_replace() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-held-promotion-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let temp = root.join("candidate.tmp");
        let final_path = root.join("final.cbor");
        let mut file = create_owned_publication_temp(&temp).unwrap();
        file.write_all(b"authoritative bytes").unwrap();
        file.sync_all().unwrap();

        let parent_hold = open_publication_directory_hold(&root).unwrap();
        assert_eq!(
            sync_retained_directory(&root, &parent_hold),
            Ok(DurabilityActionState::Performed)
        );
        assert!(
            fs::rename(&root, root.with_extension("displaced")).is_err(),
            "the publication-parent handle must deny replacement through its durability boundary"
        );
        assert_eq!(
            promote_held_file_no_replace(&file, &parent_hold, &final_path),
            Ok(HeldFilePromotion::Published)
        );
        assert!(!temp.exists());
        assert_eq!(fs::read(&final_path).unwrap(), b"authoritative bytes");
        assert_eq!(windows_file_identity(&file).unwrap().link_count, 1);
        drop(file);

        let second_temp = root.join("competitor.tmp");
        let mut competitor = create_owned_publication_temp(&second_temp).unwrap();
        competitor.write_all(b"competitor bytes").unwrap();
        competitor.sync_all().unwrap();
        let competing_outcome =
            promote_held_file_no_replace(&competitor, &parent_hold, &final_path);
        assert_eq!(
            (
                competing_outcome,
                fs::read(&final_path).unwrap(),
                second_temp.exists()
            ),
            (
                Ok(HeldFilePromotion::Conflict),
                b"authoritative bytes".to_vec(),
                true
            )
        );
        assert_eq!(fs::read(&final_path).unwrap(), b"authoritative bytes");
        assert_eq!(fs::read(&second_temp).unwrap(), b"competitor bytes");

        remove_owned_publication_temp(competitor, &second_temp).unwrap();
        assert!(!second_temp.exists());
        fs::remove_file(final_path).unwrap();
        drop(parent_hold);
        fs::remove_dir(root).unwrap();
    }

    #[cfg(any(windows, target_os = "linux"))]
    #[test]
    fn post_visibility_failure_is_an_explicit_uncertain_publication() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-visible-uncertain-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let parent_hold = open_publication_directory_hold(&root).unwrap();

        let outcome = publish_new_immutable_file_with_hooks(
            &root,
            &parent_hold,
            Path::new("terminal.cbor"),
            b"terminal bytes",
            || {},
            || Err(()),
        );

        assert!(matches!(
            outcome,
            Ok(ImmutablePublicationOutcome::VisibleReceiptUncertain)
        ));
        assert_eq!(
            fs::read(root.join("terminal.cbor")).unwrap(),
            b"terminal bytes"
        );

        fs::remove_file(root.join("terminal.cbor")).unwrap();
        drop(parent_hold);
        fs::remove_dir(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_child_directory_open_is_root_handle_relative() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let parent = std::env::temp_dir().join(format!(
            "evidence-registry-relative-open-{}-{sequence}",
            std::process::id()
        ));
        let original_root = parent.join("root");
        let moved_root = parent.join("moved-root");
        fs::create_dir_all(original_root.join("registry")).unwrap();
        fs::write(original_root.join("registry/original"), b"original").unwrap();
        let root_hold = open_windows_identity_handle(&original_root, true).unwrap();
        fs::rename(&original_root, &moved_root).unwrap();
        fs::create_dir_all(original_root.join("registry")).unwrap();
        fs::write(original_root.join("registry/replacement"), b"replacement").unwrap();

        let child = open_child_directory_hold(&root_hold, &original_root, "registry").unwrap();
        let moved_child = open_real_directory_hold(&moved_root.join("registry")).unwrap();
        let replacement_child = open_real_directory_hold(&original_root.join("registry")).unwrap();
        assert!(handles_identify_same_object(&child, &moved_child));
        assert!(!handles_identify_same_object(&child, &replacement_child));

        let displaced_child = moved_root.join("registry-displaced");
        let rename = fs::rename(moved_root.join("registry"), &displaced_child);
        if rename.is_ok() {
            fs::rename(&displaced_child, moved_root.join("registry")).unwrap();
        }
        assert!(
            rename.is_err(),
            "the rooted child hold must deny namespace substitution while pathname reads occur"
        );

        drop(replacement_child);
        drop(moved_child);
        drop(child);
        drop(root_hold);
        fs::remove_dir_all(parent).unwrap();
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn linux_child_directory_open_is_root_handle_relative() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let parent = std::env::temp_dir().join(format!(
            "evidence-registry-relative-open-{}-{sequence}",
            std::process::id()
        ));
        let original_root = parent.join("root");
        let moved_root = parent.join("moved-root");
        fs::create_dir_all(original_root.join("registry")).unwrap();
        fs::write(original_root.join("registry/original"), b"original").unwrap();
        let root_hold = open_real_directory_hold(&original_root).unwrap();
        fs::rename(&original_root, &moved_root).unwrap();
        fs::create_dir_all(original_root.join("registry")).unwrap();
        fs::write(original_root.join("registry/replacement"), b"replacement").unwrap();

        let moved_child = open_real_directory_hold(&moved_root.join("registry")).unwrap();
        let replacement_child = open_real_directory_hold(&original_root.join("registry")).unwrap();
        let child = open_child_directory_hold(&root_hold, &original_root, "registry");
        #[cfg(target_os = "linux")]
        let child = child.unwrap();
        #[cfg(target_os = "macos")]
        let child = match child {
            Ok(child) => child,
            Err(()) => {
                drop(replacement_child);
                drop(moved_child);
                drop(root_hold);
                fs::remove_dir_all(parent).unwrap();
                return;
            }
        };
        {
            assert!(handles_identify_same_object(&child, &moved_child));
            assert!(!handles_identify_same_object(&child, &replacement_child));
            drop(child);
        }

        drop(replacement_child);
        drop(moved_child);
        drop(root_hold);
        fs::remove_dir_all(parent).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_publication_stays_bound_to_the_retained_parent_generation() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let outer = std::env::temp_dir().join(format!(
            "evidence-registry-rooted-publication-{}-{sequence}",
            std::process::id()
        ));
        let parent = outer.join("records");
        let displaced = outer.join("records-displaced");
        fs::create_dir_all(&parent).unwrap();
        let parent_hold = open_real_directory_hold(&parent).unwrap();

        assert!(matches!(
            publish_new_immutable_file_with_hook(
                &parent,
                &parent_hold,
                Path::new("final.cbor"),
                b"owned",
                || {
                    fs::rename(&parent, &displaced).unwrap();
                    fs::create_dir(&parent).unwrap();
                },
            ),
            Err(())
        ));
        assert!(!parent.join("final.cbor").exists());
        assert!(!displaced.join("final.cbor").exists());
        assert_eq!(fs::read_dir(&parent).unwrap().count(), 0);
        assert_eq!(fs::read_dir(&displaced).unwrap().count(), 0);

        drop(parent_hold);
        fs::remove_dir(parent).unwrap();
        fs::remove_dir(displaced).unwrap();
        fs::remove_dir(outer).unwrap();
    }

    #[cfg(not(windows))]
    #[test]
    fn retained_directory_sync_rejects_path_replacement() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let parent = std::env::temp_dir().join(format!(
            "evidence-registry-held-directory-sync-{}-{sequence}",
            std::process::id()
        ));
        let original = parent.join("namespace");
        let displaced = parent.join("namespace-displaced");
        fs::create_dir_all(&original).unwrap();
        let hold = open_real_directory_hold(&original).unwrap();
        fs::rename(&original, &displaced).unwrap();
        fs::create_dir(&original).unwrap();

        assert_eq!(sync_retained_directory(&original, &hold), Err(()));

        drop(hold);
        fs::remove_dir(original).unwrap();
        fs::remove_dir(displaced).unwrap();
        fs::remove_dir(parent).unwrap();
    }

    #[test]
    fn file_rename_information_layout_supports_32_and_64_bit_windows_abi() {
        assert_eq!(
            file_rename_information_layout(4),
            Some(FileRenameInformationLayout {
                root_directory_offset: 4,
                file_name_length_offset: 8,
                file_name_offset: 12,
                allocation_header_size: 16,
                alignment: 4,
            })
        );
        assert_eq!(
            file_rename_information_layout(8),
            Some(FileRenameInformationLayout {
                root_directory_offset: 8,
                file_name_length_offset: 16,
                file_name_offset: 20,
                allocation_header_size: 24,
                alignment: 8,
            })
        );
        assert_eq!(file_rename_information_layout(16), None);
    }

    #[test]
    fn every_frozen_event_rejects_a_frame_valid_but_schema_incomplete_record() {
        let mappings = [
            (1, 1),
            (100, 3),
            (101, 4),
            (102, 5),
            (103, 6),
            (104, 7),
            (200, 12),
            (300, 30),
            (301, 31),
            (302, 32),
            (303, 32),
            (400, 40),
            (500, 50),
            (501, 50),
            (600, 62),
            (700, 71),
            (701, 72),
            (702, 72),
            (703, 72),
            (800, 80),
            (801, 81),
            (802, 82),
            (803, 83),
            (804, 84),
            (805, 85),
            (806, 86),
            (807, 87),
            (808, 88),
        ];
        for (event_type, record_type) in mappings {
            let mut bytes = Vec::new();
            bytes.push(0x84);
            encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
            encode_uint(&mut bytes, record_type);
            encode_uint(&mut bytes, 1);
            bytes.push(0xa2);
            bytes.extend_from_slice(&[0x00, 0x01, 0x01]);
            encode_uint(&mut bytes, record_type);
            assert!(StrictRecordFrame::decode_authoritative(&bytes).is_ok());
            assert_eq!(
                validate_event_record_type_local_schema(event_type, &bytes),
                Err(RecordDecodeError),
                "event_type_id={event_type} must require its complete type-local schema"
            );
        }
    }

    #[test]
    fn freeze_commit_rejection_accepts_any_structural_attempted_transition_event_type() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let start = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(1).unwrap(),
            JournalEntryHash::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(100).unwrap(),
            EventRecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
        );
        let conflicting = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(2).unwrap(),
            JournalEntryHash::try_from([0x40; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(102).unwrap(),
            EventRecordId::try_from([0x50; ID_LENGTH].as_slice()).unwrap(),
        );
        let mut bytes = Vec::new();
        bytes.push(0x84);
        encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut bytes, 7);
        encode_uint(&mut bytes, 1);
        bytes.push(0xa7);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x07, 0x10]);
        encode_bstr_32(&mut bytes, &[0x60; ID_LENGTH]);
        bytes.push(0x11);
        bytes.extend_from_slice(&start.authoritative_cbor());
        bytes.push(0x12);
        bytes.extend_from_slice(&conflicting.authoritative_cbor());
        bytes.push(0x13);
        encode_uint(&mut bytes, 102);
        bytes.push(0x14);
        encode_text(&mut bytes, "ATTEMPT_ALREADY_TERMINAL");

        assert_eq!(validate_event_record_type_local_schema(104, &bytes), Ok(()));
    }

    #[test]
    fn closeout_rejects_failed_candidate_roles_outside_the_closeout_role_registry() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let reference = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(1).unwrap(),
            JournalEntryHash::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(101).unwrap(),
            EventRecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
        );
        let mut bytes = Vec::new();
        bytes.push(0x84);
        encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut bytes, 50);
        encode_uint(&mut bytes, 1);
        bytes.push(0xab);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 50, 0x10, 0x02, 0x11]);
        bytes.extend_from_slice(&reference.authoritative_cbor());
        bytes.push(0x12);
        encode_bstr_32(&mut bytes, &[0x40; ID_LENGTH]);
        bytes.extend_from_slice(&[0x13, 0x80, 0x14, 0x80, 0x16]);
        bytes.extend_from_slice(&reference.authoritative_cbor());
        bytes.extend_from_slice(&[0x18, 0x1c, 0x81, 0xa3, 0x00, 0x01, 0x01]);
        encode_bstr_32(&mut bytes, &[0x50; ID_LENGTH]);
        bytes.extend_from_slice(&[0x02, 0x14, 0x18, 0x1e, 0x81]);
        encode_text(&mut bytes, "REJECTED");
        bytes.extend_from_slice(&[0x18, 0x1f]);
        bytes.extend_from_slice(&reference.authoritative_cbor());

        assert_eq!(
            validate_event_record_type_local_schema(501, &bytes),
            Err(RecordDecodeError)
        );
    }

    #[test]
    fn closeout_rejects_a_resolved_and_failed_predecessor_closeout_combination() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let reference = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(1).unwrap(),
            JournalEntryHash::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(101).unwrap(),
            EventRecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
        );
        let mut bytes = Vec::new();
        bytes.push(0x84);
        encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut bytes, 50);
        encode_uint(&mut bytes, 1);
        bytes.push(0xac);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 50, 0x10, 0x02, 0x11]);
        bytes.extend_from_slice(&reference.authoritative_cbor());
        bytes.push(0x12);
        encode_bstr_32(&mut bytes, &[0x40; ID_LENGTH]);
        bytes.extend_from_slice(&[0x13, 0x80, 0x14, 0x80, 0x16]);
        bytes.extend_from_slice(&reference.authoritative_cbor());
        bytes.extend_from_slice(&[0x18, 0x18]);
        encode_bstr_32(&mut bytes, &[0x50; ID_LENGTH]);
        bytes.extend_from_slice(&[0x18, 0x1c, 0x81, 0xa3, 0x00, 0x08, 0x01]);
        encode_bstr_32(&mut bytes, &[0x60; ID_LENGTH]);
        bytes.extend_from_slice(&[0x02, 0x14, 0x18, 0x1e, 0x81]);
        encode_text(&mut bytes, "REJECTED");
        bytes.extend_from_slice(&[0x18, 0x1f]);
        bytes.extend_from_slice(&reference.authoritative_cbor());

        assert_eq!(
            validate_event_record_type_local_schema(501, &bytes),
            Err(RecordDecodeError)
        );
    }

    #[test]
    fn closeout_rejects_resolved_predecessor_with_failed_journal_candidate() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let freeze = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(1).unwrap(),
            JournalEntryHash::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(101).unwrap(),
            EventRecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
        );
        let predecessor = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(2).unwrap(),
            JournalEntryHash::try_from([0x21; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(500).unwrap(),
            EventRecordId::try_from([0x50; ID_LENGTH].as_slice()).unwrap(),
        );
        let mut bytes = Vec::new();
        bytes.push(0x84);
        encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut bytes, 50);
        encode_uint(&mut bytes, 1);
        bytes.push(0xac);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 50, 0x10, 0x02, 0x11]);
        bytes.extend_from_slice(&freeze.authoritative_cbor());
        bytes.push(0x12);
        encode_bstr_32(&mut bytes, &[0x40; ID_LENGTH]);
        bytes.extend_from_slice(&[0x13, 0x80, 0x14, 0x80, 0x16]);
        bytes.extend_from_slice(&freeze.authoritative_cbor());
        bytes.extend_from_slice(&[0x18, 0x18]);
        encode_bstr_32(&mut bytes, predecessor.event_record_id().as_bytes());
        bytes.extend_from_slice(&[0x18, 0x1d, 0x81, 0xa3, 0x00, 0x08, 0x01]);
        bytes.extend_from_slice(&predecessor.authoritative_cbor());
        bytes.extend_from_slice(&[0x02, 0x01, 0x18, 0x1e, 0x81]);
        encode_text(&mut bytes, "REJECTED");
        bytes.extend_from_slice(&[0x18, 0x1f]);
        bytes.extend_from_slice(&freeze.authoritative_cbor());

        assert_eq!(
            validate_event_record_type_local_schema(501, &bytes),
            Err(RecordDecodeError)
        );
    }

    #[test]
    fn closeout_authority_dependencies_treat_reused_formal_authority_as_one_set_member() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let shared_definition = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(1).unwrap(),
            JournalEntryHash::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(800).unwrap(),
            EventRecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
        );
        let entry = RetainedJournalEntry::Common(CommonRetainedJournalEntry {
            registry_id,
            entry_index: JournalEntryIndex::try_from(2).unwrap(),
            previous_entry_hash: shared_definition.entry_hash(),
            event_type_id: EventTypeId::try_from(500).unwrap(),
            event_record_id: EventRecordId::try_from([0x40; ID_LENGTH].as_slice()).unwrap(),
            storage_capability_class_id: RecordId::try_from([0x50; ID_LENGTH].as_slice()).unwrap(),
            environment_observation_id: RecordId::try_from([0x60; ID_LENGTH].as_slice()).unwrap(),
            lifecycle_object_kind: LifecycleObjectKind::Registry,
            lifecycle_object_id: *registry_id.as_bytes(),
            freeze_attempt_intended_root_id: None,
            identity_dependencies: IdentityDependencyCollection {
                elements: Vec::new(),
            },
            authority_dependencies: AuthorityDependencyCollection {
                elements: vec![shared_definition.clone()],
            },
            authoritative_bytes: Vec::new(),
        });

        assert_eq!(
            validate_exact_authority_dependencies(
                &entry,
                &[shared_definition.clone(), shared_definition]
            ),
            Ok(())
        );
    }

    #[test]
    fn closeout_replay_rejects_candidates_marked_both_resolved_and_failed() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let storage = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let environment = RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
        let genesis = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from([0x40; ID_LENGTH].as_slice()).unwrap(),
            storage,
            environment,
        );
        let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
        let operation_start = journal.current_head_reference();

        let mut policy_bytes = Vec::new();
        policy_bytes.push(0x84);
        encode_text(&mut policy_bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut policy_bytes, 40);
        encode_uint(&mut policy_bytes, 1);
        policy_bytes.push(0xa5);
        policy_bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 40, 0x10]);
        encode_bstr_32(&mut policy_bytes, &[0x41; ID_LENGTH]);
        policy_bytes.extend_from_slice(&[0x18, 0x1d]);
        policy_bytes.extend_from_slice(&operation_start.authoritative_cbor());
        policy_bytes.extend_from_slice(&[0x18, 0x1e, 0x81, 0x03]);
        let policy_id = StrictRecordFrame::decode_authoritative(&policy_bytes)
            .unwrap()
            .record_id();

        let manifest_id = RecordId::try_from([0x42; ID_LENGTH].as_slice()).unwrap();
        let mut receipt_bytes = Vec::new();
        receipt_bytes.push(0x84);
        encode_text(&mut receipt_bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut receipt_bytes, 4);
        encode_uint(&mut receipt_bytes, 1);
        receipt_bytes.push(0xb1);
        receipt_bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x04, 0x10]);
        encode_bstr_32(&mut receipt_bytes, &[0x43; ID_LENGTH]);
        receipt_bytes.push(0x11);
        encode_bstr_32(&mut receipt_bytes, &[0x44; ID_LENGTH]);
        receipt_bytes.push(0x12);
        receipt_bytes.extend_from_slice(&operation_start.authoritative_cbor());
        receipt_bytes.push(0x13);
        encode_bstr_32(&mut receipt_bytes, &[0x45; ID_LENGTH]);
        receipt_bytes.push(0x14);
        encode_bstr_32(&mut receipt_bytes, manifest_id.as_bytes());
        receipt_bytes.extend_from_slice(&[0x15, 0x01, 0x16]);
        encode_bstr_32(&mut receipt_bytes, &[0x46; ID_LENGTH]);
        receipt_bytes.extend_from_slice(&[0x17, 0x01, 0x18, 0x18]);
        encode_bstr_32(&mut receipt_bytes, &[0x47; ID_LENGTH]);
        receipt_bytes.extend_from_slice(&[0x18, 0x19]);
        encode_bstr_32(&mut receipt_bytes, policy_id.as_bytes());
        receipt_bytes.extend_from_slice(&[
            0x18, 0x1a, 0x01, 0x18, 0x1b, 0x01, 0x18, 0x1c, 0x01, 0x18, 0x1d, 0xf4, 0x18, 0x1f,
        ]);
        encode_text(&mut receipt_bytes, "closeout-overlap-test");
        let receipt = FreezeReceiptRecord::decode_authoritative(&receipt_bytes).unwrap();

        let append_authority = |journal: &mut RetainedJournal,
                                event_type: u16,
                                record_id: RecordId,
                                marker: u8| {
            let previous = journal.current_head_reference();
            journal
                .entries
                .push(RetainedJournalEntry::Common(CommonRetainedJournalEntry {
                    registry_id,
                    entry_index: JournalEntryIndex::try_from(previous.entry_index().value() + 1)
                        .unwrap(),
                    previous_entry_hash: previous.entry_hash(),
                    event_type_id: EventTypeId::try_from(u64::from(event_type)).unwrap(),
                    event_record_id: EventRecordId::try_from(record_id.as_bytes().as_slice())
                        .unwrap(),
                    storage_capability_class_id: storage,
                    environment_observation_id: environment,
                    lifecycle_object_kind: EventTypeId::try_from(u64::from(event_type))
                        .unwrap()
                        .lifecycle_object_kind(),
                    lifecycle_object_id: *record_id.as_bytes(),
                    freeze_attempt_intended_root_id: None,
                    identity_dependencies: IdentityDependencyCollection {
                        elements: Vec::new(),
                    },
                    authority_dependencies: AuthorityDependencyCollection {
                        elements: Vec::new(),
                    },
                    authoritative_bytes: vec![marker],
                }));
            journal.current_head_reference()
        };
        let freeze_reference = append_authority(&mut journal, 101, receipt.record_id(), 0x51);
        let review_record_id = RecordId::try_from([0x52; ID_LENGTH].as_slice()).unwrap();
        let review_reference = append_authority(&mut journal, 302, review_record_id, 0x52);
        let policy_reference = append_authority(&mut journal, 400, policy_id, 0x53);

        let mut closeout_bytes = Vec::new();
        closeout_bytes.push(0x84);
        encode_text(&mut closeout_bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut closeout_bytes, 50);
        encode_uint(&mut closeout_bytes, 1);
        closeout_bytes.push(0xab);
        closeout_bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 50, 0x10, 0x02, 0x11]);
        closeout_bytes.extend_from_slice(&freeze_reference.authoritative_cbor());
        closeout_bytes.push(0x12);
        encode_bstr_32(&mut closeout_bytes, manifest_id.as_bytes());
        closeout_bytes.extend_from_slice(&[0x13, 0x81]);
        closeout_bytes.extend_from_slice(&review_reference.authoritative_cbor());
        closeout_bytes.extend_from_slice(&[0x14, 0x80, 0x16]);
        closeout_bytes.extend_from_slice(&policy_reference.authoritative_cbor());
        closeout_bytes.extend_from_slice(&[0x18, 0x1d, 0x81, 0xa3, 0x00, 0x05, 0x01]);
        closeout_bytes.extend_from_slice(&review_reference.authoritative_cbor());
        closeout_bytes.extend_from_slice(&[0x02, 0x01, 0x18, 0x1e, 0x81]);
        encode_text(&mut closeout_bytes, "REVIEW_REJECTED");
        closeout_bytes.extend_from_slice(&[0x18, 0x1f]);
        closeout_bytes.extend_from_slice(&operation_start.authoritative_cbor());
        let closeout_id = StrictRecordFrame::decode_authoritative(&closeout_bytes)
            .unwrap()
            .record_id();
        let closeout_entry = RetainedJournalEntry::Common(CommonRetainedJournalEntry {
            registry_id,
            entry_index: JournalEntryIndex::try_from(
                journal.current_head_reference().entry_index().value() + 1,
            )
            .unwrap(),
            previous_entry_hash: journal.current_head_reference().entry_hash(),
            event_type_id: EventTypeId::try_from(501).unwrap(),
            event_record_id: EventRecordId::try_from(closeout_id.as_bytes().as_slice()).unwrap(),
            storage_capability_class_id: storage,
            environment_observation_id: environment,
            lifecycle_object_kind: EventTypeId::try_from(501).unwrap().lifecycle_object_kind(),
            lifecycle_object_id: *closeout_id.as_bytes(),
            freeze_attempt_intended_root_id: None,
            identity_dependencies: IdentityDependencyCollection {
                elements: vec![IdentityDependency::RecordId(manifest_id)],
            },
            authority_dependencies: AuthorityDependencyCollection {
                elements: vec![
                    freeze_reference.clone(),
                    review_reference.clone(),
                    policy_reference.clone(),
                ],
            },
            authoritative_bytes: Vec::new(),
        });
        let mut records = vec![
            (receipt.record_id(), receipt_bytes),
            (policy_id, policy_bytes),
        ];
        records.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));

        assert_eq!(
            validate_closeout_event_bindings(&journal, &closeout_entry, &closeout_bytes, &records,),
            Err(RecordDecodeError)
        );

        let mut manifest_overlap_bytes = Vec::new();
        manifest_overlap_bytes.push(0x84);
        encode_text(&mut manifest_overlap_bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut manifest_overlap_bytes, 50);
        encode_uint(&mut manifest_overlap_bytes, 1);
        manifest_overlap_bytes.push(0xab);
        manifest_overlap_bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 50, 0x10, 0x02, 0x11]);
        manifest_overlap_bytes.extend_from_slice(&freeze_reference.authoritative_cbor());
        manifest_overlap_bytes.push(0x12);
        encode_bstr_32(&mut manifest_overlap_bytes, manifest_id.as_bytes());
        manifest_overlap_bytes.extend_from_slice(&[0x13, 0x81]);
        manifest_overlap_bytes.extend_from_slice(&review_reference.authoritative_cbor());
        manifest_overlap_bytes.extend_from_slice(&[0x14, 0x80, 0x16]);
        manifest_overlap_bytes.extend_from_slice(&policy_reference.authoritative_cbor());
        manifest_overlap_bytes.extend_from_slice(&[0x18, 0x1c, 0x81, 0xa3, 0x00, 0x0e, 0x01]);
        encode_bstr_32(&mut manifest_overlap_bytes, manifest_id.as_bytes());
        manifest_overlap_bytes.extend_from_slice(&[0x02, 0x14, 0x18, 0x1e, 0x81]);
        encode_text(&mut manifest_overlap_bytes, "MANIFEST_REJECTED");
        manifest_overlap_bytes.extend_from_slice(&[0x18, 0x1f]);
        manifest_overlap_bytes.extend_from_slice(&operation_start.authoritative_cbor());
        let manifest_overlap_id = StrictRecordFrame::decode_authoritative(&manifest_overlap_bytes)
            .unwrap()
            .record_id();
        let manifest_overlap_entry = RetainedJournalEntry::Common(CommonRetainedJournalEntry {
            registry_id,
            entry_index: JournalEntryIndex::try_from(
                journal.current_head_reference().entry_index().value() + 1,
            )
            .unwrap(),
            previous_entry_hash: journal.current_head_reference().entry_hash(),
            event_type_id: EventTypeId::try_from(501).unwrap(),
            event_record_id: EventRecordId::try_from(manifest_overlap_id.as_bytes().as_slice())
                .unwrap(),
            storage_capability_class_id: storage,
            environment_observation_id: environment,
            lifecycle_object_kind: EventTypeId::try_from(501).unwrap().lifecycle_object_kind(),
            lifecycle_object_id: *manifest_overlap_id.as_bytes(),
            freeze_attempt_intended_root_id: None,
            identity_dependencies: IdentityDependencyCollection {
                elements: vec![IdentityDependency::RecordId(manifest_id)],
            },
            authority_dependencies: AuthorityDependencyCollection {
                elements: vec![
                    freeze_reference.clone(),
                    review_reference.clone(),
                    policy_reference.clone(),
                ],
            },
            authoritative_bytes: Vec::new(),
        });
        assert_eq!(
            validate_closeout_event_bindings(
                &journal,
                &manifest_overlap_entry,
                &manifest_overlap_bytes,
                &records,
            ),
            Err(RecordDecodeError)
        );

        let mut review_record_overlap_bytes = Vec::new();
        review_record_overlap_bytes.push(0x84);
        encode_text(
            &mut review_record_overlap_bytes,
            "EvidenceRegistry.Record.v1",
        );
        encode_uint(&mut review_record_overlap_bytes, 50);
        encode_uint(&mut review_record_overlap_bytes, 1);
        review_record_overlap_bytes.push(0xab);
        review_record_overlap_bytes
            .extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 50, 0x10, 0x02, 0x11]);
        review_record_overlap_bytes.extend_from_slice(&freeze_reference.authoritative_cbor());
        review_record_overlap_bytes.push(0x12);
        encode_bstr_32(&mut review_record_overlap_bytes, manifest_id.as_bytes());
        review_record_overlap_bytes.extend_from_slice(&[0x13, 0x81]);
        review_record_overlap_bytes.extend_from_slice(&review_reference.authoritative_cbor());
        review_record_overlap_bytes.extend_from_slice(&[0x14, 0x80, 0x16]);
        review_record_overlap_bytes.extend_from_slice(&policy_reference.authoritative_cbor());
        review_record_overlap_bytes.extend_from_slice(&[0x18, 0x1c, 0x81, 0xa3, 0x00, 0x05, 0x01]);
        encode_bstr_32(
            &mut review_record_overlap_bytes,
            review_record_id.as_bytes(),
        );
        review_record_overlap_bytes.extend_from_slice(&[0x02, 0x14, 0x18, 0x1e, 0x81]);
        encode_text(&mut review_record_overlap_bytes, "REVIEW_RECORD_REJECTED");
        review_record_overlap_bytes.extend_from_slice(&[0x18, 0x1f]);
        review_record_overlap_bytes.extend_from_slice(&operation_start.authoritative_cbor());
        let review_record_overlap_id =
            StrictRecordFrame::decode_authoritative(&review_record_overlap_bytes)
                .unwrap()
                .record_id();
        let review_record_overlap_entry =
            RetainedJournalEntry::Common(CommonRetainedJournalEntry {
                registry_id,
                entry_index: JournalEntryIndex::try_from(
                    journal.current_head_reference().entry_index().value() + 1,
                )
                .unwrap(),
                previous_entry_hash: journal.current_head_reference().entry_hash(),
                event_type_id: EventTypeId::try_from(501).unwrap(),
                event_record_id: EventRecordId::try_from(
                    review_record_overlap_id.as_bytes().as_slice(),
                )
                .unwrap(),
                storage_capability_class_id: storage,
                environment_observation_id: environment,
                lifecycle_object_kind: EventTypeId::try_from(501).unwrap().lifecycle_object_kind(),
                lifecycle_object_id: *review_record_overlap_id.as_bytes(),
                freeze_attempt_intended_root_id: None,
                identity_dependencies: IdentityDependencyCollection {
                    elements: vec![IdentityDependency::RecordId(manifest_id)],
                },
                authority_dependencies: AuthorityDependencyCollection {
                    elements: vec![freeze_reference, review_reference, policy_reference],
                },
                authoritative_bytes: Vec::new(),
            });
        assert_eq!(
            validate_closeout_event_bindings(
                &journal,
                &review_record_overlap_entry,
                &review_record_overlap_bytes,
                &records,
            ),
            Err(RecordDecodeError)
        );
    }

    #[test]
    fn policy_recorded_accepts_a_valid_zero_requirement_freeze_commit_policy() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let operation_start = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(1).unwrap(),
            JournalEntryHash::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(100).unwrap(),
            EventRecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
        );
        let mut bytes = Vec::new();
        bytes.push(0x84);
        encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut bytes, 40);
        encode_uint(&mut bytes, 1);
        bytes.push(0xa5);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 40, 0x10]);
        encode_bstr_32(&mut bytes, &[0x40; ID_LENGTH]);
        bytes.extend_from_slice(&[0x18, 0x1d]);
        bytes.extend_from_slice(&operation_start.authoritative_cbor());
        bytes.extend_from_slice(&[0x18, 0x1e, 0x81, 0x01]);

        assert_eq!(validate_event_record_type_local_schema(400, &bytes), Ok(()));
    }

    #[test]
    fn general_policy_accepts_zero_required_count_as_a_canonical_uint() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let operation_start = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(1).unwrap(),
            JournalEntryHash::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(100).unwrap(),
            EventRecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
        );
        let mut bytes = Vec::new();
        bytes.push(0x84);
        encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut bytes, 40);
        encode_uint(&mut bytes, 1);
        bytes.push(0xa6);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 40, 0x10]);
        encode_bstr_32(&mut bytes, &[0x40; ID_LENGTH]);
        bytes.extend_from_slice(&[0x11, 0x81, 0xa5, 0x00, 0x01, 0x01]);
        encode_bstr_32(&mut bytes, &[0x41; ID_LENGTH]);
        bytes.push(0x02);
        encode_bstr_32(&mut bytes, &[0x42; ID_LENGTH]);
        bytes.push(0x03);
        encode_bstr_32(&mut bytes, &[0x43; ID_LENGTH]);
        bytes.extend_from_slice(&[0x04, 0x00, 0x18, 0x1d]);
        bytes.extend_from_slice(&operation_start.authoritative_cbor());
        bytes.extend_from_slice(&[0x18, 0x1e, 0x81, 0x02]);

        assert_eq!(validate_event_record_type_local_schema(400, &bytes), Ok(()));
    }

    #[test]
    fn general_policy_rejects_duplicate_review_selectors_with_different_counts() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let operation_start = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(1).unwrap(),
            JournalEntryHash::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(100).unwrap(),
            EventRecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
        );
        let mut bytes = Vec::new();
        bytes.push(0x84);
        encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut bytes, 40);
        encode_uint(&mut bytes, 1);
        bytes.push(0xa6);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 40, 0x10]);
        encode_bstr_32(&mut bytes, &[0x40; ID_LENGTH]);
        bytes.extend_from_slice(&[0x11, 0x82]);
        for required_count in [0_u8, 1] {
            bytes.extend_from_slice(&[0xa5, 0x00, 0x01, 0x01]);
            encode_bstr_32(&mut bytes, &[0x41; ID_LENGTH]);
            bytes.push(0x02);
            encode_bstr_32(&mut bytes, &[0x42; ID_LENGTH]);
            bytes.push(0x03);
            encode_bstr_32(&mut bytes, &[0x43; ID_LENGTH]);
            bytes.extend_from_slice(&[0x04, required_count]);
        }
        bytes.extend_from_slice(&[0x18, 0x1d]);
        bytes.extend_from_slice(&operation_start.authoritative_cbor());
        bytes.extend_from_slice(&[0x18, 0x1e, 0x81, 0x02]);

        assert_eq!(
            validate_event_record_type_local_schema(400, &bytes),
            Err(RecordDecodeError)
        );
    }

    #[test]
    fn general_policy_accepts_zero_as_a_diversity_distinct_count() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let operation_start = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(1).unwrap(),
            JournalEntryHash::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(100).unwrap(),
            EventRecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
        );
        let mut bytes = Vec::new();
        bytes.push(0x84);
        encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut bytes, 40);
        encode_uint(&mut bytes, 1);
        bytes.push(0xa6);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 40, 0x10]);
        encode_bstr_32(&mut bytes, &[0x40; ID_LENGTH]);
        bytes.extend_from_slice(&[0x18, 0x1b, 0x81, 0xa2, 0x00, 0x01, 0x01, 0x00]);
        bytes.extend_from_slice(&[0x18, 0x1d]);
        bytes.extend_from_slice(&operation_start.authoritative_cbor());
        bytes.extend_from_slice(&[0x18, 0x1e, 0x81, 0x03]);

        assert_eq!(validate_event_record_type_local_schema(400, &bytes), Ok(()));
    }

    #[test]
    fn general_policy_intervening_event_ids_use_the_event_type_domain() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let operation_start = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(1).unwrap(),
            JournalEntryHash::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(100).unwrap(),
            EventRecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
        );
        let build = |event_type_id: u64| {
            let mut bytes = Vec::new();
            bytes.push(0x84);
            encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
            encode_uint(&mut bytes, 40);
            encode_uint(&mut bytes, 1);
            bytes.push(0xa6);
            bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 40, 0x10]);
            encode_bstr_32(&mut bytes, &[0x40; ID_LENGTH]);
            bytes.extend_from_slice(&[0x15, 0xa1, 0x00, 0x81]);
            encode_uint(&mut bytes, event_type_id);
            bytes.extend_from_slice(&[0x18, 0x1d]);
            bytes.extend_from_slice(&operation_start.authoritative_cbor());
            bytes.extend_from_slice(&[0x18, 0x1e, 0x81, 0x03]);
            bytes
        };

        assert_eq!(
            validate_event_record_type_local_schema(400, &build(1)),
            Ok(())
        );
        for invalid in [0, 65_536] {
            assert_eq!(
                validate_event_record_type_local_schema(400, &build(invalid)),
                Err(RecordDecodeError)
            );
        }
    }

    #[test]
    fn rejected_review_admission_accepts_an_exact_failed_request_candidate() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let operation_start = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(1).unwrap(),
            JournalEntryHash::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(100).unwrap(),
            EventRecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
        );
        let mut bytes = Vec::new();
        bytes.push(0x84);
        encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut bytes, 32);
        encode_uint(&mut bytes, 1);
        bytes.push(0xa6);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 32, 0x10, 0x02]);
        bytes.extend_from_slice(&[0x14, 0x81, 0xa3, 0x00, 0x01, 0x01]);
        encode_bstr_32(&mut bytes, &[0x40; ID_LENGTH]);
        bytes.extend_from_slice(&[0x02, 0x14, 0x16, 0x81]);
        encode_text(&mut bytes, "REQUEST_INVALID");
        bytes.push(0x17);
        bytes.extend_from_slice(&operation_start.authoritative_cbor());

        assert_eq!(validate_event_record_type_local_schema(303, &bytes), Ok(()));
    }

    #[test]
    fn rejected_admission_failed_journal_candidates_are_not_identity_dependencies() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let genesis = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
            RecordId::try_from([0x40; ID_LENGTH].as_slice()).unwrap(),
        );
        let journal = RetainedJournal::from_genesis(genesis).unwrap();
        let operation_start = journal.current_head_reference();
        let failed_candidate = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(99).unwrap(),
            JournalEntryHash::try_from([0x50; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(300).unwrap(),
            EventRecordId::try_from([0x60; ID_LENGTH].as_slice()).unwrap(),
        );
        let mut record_bytes = Vec::new();
        record_bytes.push(0x84);
        encode_text(&mut record_bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut record_bytes, 32);
        encode_uint(&mut record_bytes, 1);
        record_bytes.push(0xa6);
        record_bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 32, 0x10, 0x02]);
        record_bytes.extend_from_slice(&[0x15, 0x81, 0xa3, 0x00, 0x01, 0x01]);
        record_bytes.extend_from_slice(&failed_candidate.authoritative_cbor());
        record_bytes.extend_from_slice(&[0x02, 0x01, 0x16, 0x81]);
        encode_text(&mut record_bytes, "REQUEST_UNAVAILABLE");
        record_bytes.push(0x17);
        record_bytes.extend_from_slice(&operation_start.authoritative_cbor());
        let record_id = StrictRecordFrame::decode_authoritative(&record_bytes)
            .unwrap()
            .record_id();
        let entry = RetainedJournalEntry::Common(CommonRetainedJournalEntry {
            registry_id,
            entry_index: JournalEntryIndex::try_from(1).unwrap(),
            previous_entry_hash: operation_start.entry_hash(),
            event_type_id: EventTypeId::try_from(303).unwrap(),
            event_record_id: EventRecordId::try_from(record_id.as_bytes().as_slice()).unwrap(),
            storage_capability_class_id: RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
            environment_observation_id: RecordId::try_from([0x40; ID_LENGTH].as_slice()).unwrap(),
            lifecycle_object_kind: LifecycleObjectKind::ReviewAdmissionAttempt,
            lifecycle_object_id: *record_id.as_bytes(),
            freeze_attempt_intended_root_id: None,
            identity_dependencies: IdentityDependencyCollection {
                elements: Vec::new(),
            },
            authority_dependencies: AuthorityDependencyCollection {
                elements: Vec::new(),
            },
            authoritative_bytes: Vec::new(),
        });

        assert_eq!(
            validate_review_admission_event_bindings(&journal, &entry, &record_bytes, &[]),
            Ok(())
        );
    }

    #[test]
    fn storage_change_does_not_promote_should_identity_bindings_to_must() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let prior_storage = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let new_storage = RecordId::try_from([0x21; ID_LENGTH].as_slice()).unwrap();
        let prior_environment = RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
        let new_environment = RecordId::try_from([0x31; ID_LENGTH].as_slice()).unwrap();
        let genesis = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from([0x40; ID_LENGTH].as_slice()).unwrap(),
            prior_storage,
            prior_environment,
        );
        let journal = RetainedJournal::from_genesis(genesis).unwrap();
        let operation_start = journal.current_head_reference();
        let mut record_bytes = Vec::new();
        record_bytes.push(0x84);
        encode_text(&mut record_bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut record_bytes, 62);
        encode_uint(&mut record_bytes, 1);
        record_bytes.push(0xa9);
        record_bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 62, 0x10]);
        encode_bstr_32(&mut record_bytes, prior_storage.as_bytes());
        record_bytes.push(0x11);
        encode_bstr_32(&mut record_bytes, new_storage.as_bytes());
        record_bytes.push(0x12);
        encode_bstr_32(&mut record_bytes, prior_environment.as_bytes());
        record_bytes.push(0x13);
        encode_bstr_32(&mut record_bytes, new_environment.as_bytes());
        record_bytes.extend_from_slice(&[0x14, 0x80, 0x15, 0x80, 0x16]);
        record_bytes.extend_from_slice(&operation_start.authoritative_cbor());
        let record_id = StrictRecordFrame::decode_authoritative(&record_bytes)
            .unwrap()
            .record_id();
        let entry = RetainedJournalEntry::Common(CommonRetainedJournalEntry {
            registry_id,
            entry_index: JournalEntryIndex::try_from(1).unwrap(),
            previous_entry_hash: operation_start.entry_hash(),
            event_type_id: EventTypeId::try_from(600).unwrap(),
            event_record_id: EventRecordId::try_from(record_id.as_bytes().as_slice()).unwrap(),
            storage_capability_class_id: new_storage,
            environment_observation_id: new_environment,
            lifecycle_object_kind: LifecycleObjectKind::Registry,
            lifecycle_object_id: *registry_id.as_bytes(),
            freeze_attempt_intended_root_id: None,
            identity_dependencies: IdentityDependencyCollection {
                elements: Vec::new(),
            },
            authority_dependencies: AuthorityDependencyCollection {
                elements: Vec::new(),
            },
            authoritative_bytes: Vec::new(),
        });

        assert_eq!(
            validate_event_record_reference_bindings(&journal, &entry, &record_bytes, &[]),
            Ok(())
        );
    }

    fn capability_observation_provenance_record(
        storage: RecordId,
        environment: RecordId,
        cache_reused: bool,
        include_process_scope: bool,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(0x84);
        encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut bytes, 63);
        encode_uint(&mut bytes, 1);
        bytes.push(if include_process_scope { 0xa6 } else { 0xa5 });
        bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 63, 0x10]);
        encode_bstr_32(&mut bytes, storage.as_bytes());
        bytes.push(0x11);
        encode_bstr_32(&mut bytes, environment.as_bytes());
        bytes.extend_from_slice(&[0x12, if cache_reused { 0xf5 } else { 0xf4 }]);
        if include_process_scope {
            bytes.extend_from_slice(&[0x13, 0x41, 0x01]);
        }
        bytes
    }

    fn minimal_establishment_record(definition: &JournalReference) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(0x84);
        encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut bytes, 81);
        encode_uint(&mut bytes, 1);
        bytes.push(0xa3);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 81, 0x10]);
        bytes.extend_from_slice(&definition.authoritative_cbor());
        bytes
    }

    fn compatibility_record(source: &JournalReference, target: &JournalReference) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(0x84);
        encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut bytes, 84);
        encode_uint(&mut bytes, 1);
        bytes.push(0xa8);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 84, 0x10]);
        bytes.extend_from_slice(&source.authoritative_cbor());
        bytes.push(0x11);
        bytes.extend_from_slice(&target.authoritative_cbor());
        bytes.push(0x12);
        encode_bstr_32(&mut bytes, &[0x40; ID_LENGTH]);
        bytes.push(0x13);
        encode_bstr_32(&mut bytes, &[0x41; ID_LENGTH]);
        bytes.extend_from_slice(&[0x14, 0x80, 0x15]);
        bytes.extend_from_slice(&source.authoritative_cbor());
        bytes
    }

    fn bootstrap_declaration_record(
        scope: RecordId,
        operation_start: &JournalReference,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(0x84);
        encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut bytes, 86);
        encode_uint(&mut bytes, 1);
        bytes.push(0xac);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 86, 0x10]);
        encode_bstr_32(&mut bytes, scope.as_bytes());
        for key in 17_u8..=22 {
            bytes.extend_from_slice(&[key, 0x80]);
        }
        bytes.extend_from_slice(&[0x17, 0x80, 0x18, 0x18, 0x80, 0x18, 0x1a]);
        bytes.extend_from_slice(&operation_start.authoritative_cbor());
        bytes
    }

    #[test]
    fn closeout_formal_support_requires_exact_directional_compatibility() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let reference = |index: u64, event_type: u16, record_id: RecordId| {
            JournalReference::new(
                registry_id,
                JournalEntryIndex::try_from(index).unwrap(),
                JournalEntryHash::try_from([index as u8; ID_LENGTH].as_slice()).unwrap(),
                EventTypeId::try_from(u64::from(event_type)).unwrap(),
                EventRecordId::try_from(record_id.as_bytes().as_slice()).unwrap(),
            )
        };
        let source_id = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let target_id = RecordId::try_from([0x21; ID_LENGTH].as_slice()).unwrap();
        let establishment_id = RecordId::try_from([0x22; ID_LENGTH].as_slice()).unwrap();
        let compatibility_id = RecordId::try_from([0x23; ID_LENGTH].as_slice()).unwrap();
        let swapped_compatibility_id = RecordId::try_from([0x24; ID_LENGTH].as_slice()).unwrap();
        let source = reference(1, 800, source_id);
        let target = reference(2, 800, target_id);
        let establishment = reference(3, 801, establishment_id);
        let compatibility = reference(4, 804, compatibility_id);
        let swapped_compatibility = reference(5, 804, swapped_compatibility_id);
        let mut records = vec![
            (establishment_id, minimal_establishment_record(&source)),
            (compatibility_id, compatibility_record(&source, &target)),
            (
                swapped_compatibility_id,
                compatibility_record(&target, &source),
            ),
        ];
        records.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));

        assert_eq!(
            validate_closeout_formal_binding(
                &records,
                &target,
                &establishment,
                Some(&compatibility),
            ),
            Ok(())
        );
        assert_eq!(
            validate_closeout_formal_binding(
                &records,
                &target,
                &establishment,
                Some(&swapped_compatibility),
            ),
            Err(RecordDecodeError)
        );
        assert_eq!(
            validate_closeout_formal_binding(
                &records,
                &source,
                &establishment,
                Some(&compatibility),
            ),
            Err(RecordDecodeError),
            "same-Definition support forbids an unnecessary compatibility authority"
        );
    }

    #[test]
    fn closeout_bootstrap_support_requires_every_declaration_scope_to_match() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let required_scope = RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
        let wrong_scope = RecordId::try_from([0x31; ID_LENGTH].as_slice()).unwrap();
        let declaration_id = RecordId::try_from([0x32; ID_LENGTH].as_slice()).unwrap();
        let operation_start = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(0).unwrap(),
            JournalEntryHash::try_from([0x40; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(1).unwrap(),
            EventRecordId::try_from([0x41; ID_LENGTH].as_slice()).unwrap(),
        );
        let declaration = JournalReference::new(
            registry_id,
            JournalEntryIndex::try_from(1).unwrap(),
            JournalEntryHash::try_from([0x42; ID_LENGTH].as_slice()).unwrap(),
            EventTypeId::try_from(806).unwrap(),
            EventRecordId::try_from(declaration_id.as_bytes().as_slice()).unwrap(),
        );
        let matching = vec![(
            declaration_id,
            bootstrap_declaration_record(required_scope, &operation_start),
        )];
        let mismatched = vec![(
            declaration_id,
            bootstrap_declaration_record(wrong_scope, &operation_start),
        )];

        assert_eq!(
            decode_bootstrap_declaration_scope(&matching[0].1),
            Ok(required_scope)
        );

        assert_eq!(
            validate_closeout_bootstrap_scopes(
                &matching,
                std::slice::from_ref(&declaration),
                required_scope,
            ),
            Ok(())
        );
        assert_eq!(
            validate_closeout_bootstrap_scopes(&mismatched, &[declaration], required_scope),
            Err(RecordDecodeError)
        );
    }

    #[test]
    fn capability_observation_provenance_requires_the_complete_frozen_schema() {
        let storage = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let environment = RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
        let complete = capability_observation_provenance_record(storage, environment, true, true);
        let missing_process_scope =
            capability_observation_provenance_record(storage, environment, true, false);

        assert_eq!(
            decode_capability_observation_provenance(&complete),
            Ok((storage, environment, true))
        );
        assert_eq!(
            decode_capability_observation_provenance(&missing_process_scope),
            Err(RecordDecodeError)
        );
    }

    #[test]
    fn event_600_replay_binds_prior_capability_values_to_the_exact_predecessor() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let prior_storage = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let prior_environment = RecordId::try_from([0x21; ID_LENGTH].as_slice()).unwrap();
        let new_storage = RecordId::try_from([0x22; ID_LENGTH].as_slice()).unwrap();
        let new_environment = RecordId::try_from([0x23; ID_LENGTH].as_slice()).unwrap();
        let wrong_prior_storage = RecordId::try_from([0x24; ID_LENGTH].as_slice()).unwrap();
        let genesis_record = GenesisRecord::new(GenesisRecordInput {
            registry_id,
            journal_format_version: 1,
            record_identity_profile_id: 1,
            storage_capability_class_id: prior_storage,
            environment_observation_id: prior_environment,
            created_by_tool_version: "event-600-test".to_owned(),
        })
        .unwrap();
        let genesis = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from(genesis_record.record_id().as_bytes().as_slice()).unwrap(),
            prior_storage,
            prior_environment,
        );
        let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
        let operation_start = journal.current_head_reference();

        let mut record_bytes = Vec::new();
        record_bytes.push(0x84);
        encode_text(&mut record_bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut record_bytes, 62);
        encode_uint(&mut record_bytes, 1);
        record_bytes.push(0xa9);
        record_bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 62, 0x10]);
        encode_bstr_32(&mut record_bytes, wrong_prior_storage.as_bytes());
        record_bytes.push(0x11);
        encode_bstr_32(&mut record_bytes, new_storage.as_bytes());
        record_bytes.push(0x12);
        encode_bstr_32(&mut record_bytes, prior_environment.as_bytes());
        record_bytes.push(0x13);
        encode_bstr_32(&mut record_bytes, new_environment.as_bytes());
        record_bytes.extend_from_slice(&[0x14, 0x80, 0x15, 0x80, 0x16]);
        record_bytes.extend_from_slice(&operation_start.authoritative_cbor());
        let record_id = StrictRecordFrame::decode_authoritative(&record_bytes)
            .unwrap()
            .record_id();
        let entry = RetainedJournalEntry::Common(CommonRetainedJournalEntry {
            registry_id,
            entry_index: JournalEntryIndex::try_from(1).unwrap(),
            previous_entry_hash: operation_start.entry_hash(),
            event_type_id: EventTypeId::try_from(600).unwrap(),
            event_record_id: EventRecordId::try_from(record_id.as_bytes().as_slice()).unwrap(),
            storage_capability_class_id: new_storage,
            environment_observation_id: new_environment,
            lifecycle_object_kind: LifecycleObjectKind::Registry,
            lifecycle_object_id: *registry_id.as_bytes(),
            freeze_attempt_intended_root_id: None,
            identity_dependencies: IdentityDependencyCollection {
                elements: Vec::new(),
            },
            authority_dependencies: AuthorityDependencyCollection {
                elements: Vec::new(),
            },
            authoritative_bytes: Vec::new(),
        });

        assert_eq!(
            validate_event_record_reference_bindings(&journal, &entry, &record_bytes, &[]),
            Err(RecordDecodeError)
        );

        let mut valid_record_bytes = record_bytes;
        let wrong_prior = valid_record_bytes
            .windows(ID_LENGTH)
            .position(|window| window == wrong_prior_storage.as_bytes())
            .unwrap();
        valid_record_bytes[wrong_prior..wrong_prior + ID_LENGTH]
            .copy_from_slice(prior_storage.as_bytes());
        let valid_record_id = StrictRecordFrame::decode_authoritative(&valid_record_bytes)
            .unwrap()
            .record_id();
        let mut valid_entry = entry;
        let RetainedJournalEntry::Common(common) = &mut valid_entry else {
            unreachable!();
        };
        common.event_record_id =
            EventRecordId::try_from(valid_record_id.as_bytes().as_slice()).unwrap();
        journal.entries.push(valid_entry);
        let mut records = vec![
            (
                genesis_record.record_id(),
                genesis_record.authoritative_cbor(),
            ),
            (valid_record_id, valid_record_bytes),
        ];
        records.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
        assert_eq!(
            validate_authoritative_event_records(&journal, &records),
            Err(AuthoritativeRegistryStoreOpenError::EventSemanticAuthorityUnavailable)
        );
    }

    #[test]
    fn unsupported_contextual_event_semantics_fail_closed_before_positive_replay() {
        for event_type in [200, 300, 301, 302, 303, 500, 501, 600, 700] {
            assert_eq!(
                event_semantic_authority_is_unavailable(event_type, &[], &[]),
                Ok(true)
            );
        }
    }

    #[test]
    fn event_801_requires_authoritative_cache_provenance_and_matching_capability_epoch() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let storage = RecordId::try_from([0x23; ID_LENGTH].as_slice()).unwrap();
        let environment = RecordId::try_from([0x24; ID_LENGTH].as_slice()).unwrap();
        let genesis = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from([0x11; ID_LENGTH].as_slice()).unwrap(),
            storage,
            environment,
        );
        let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
        let genesis_reference = journal.current_head_reference();
        let definition_record_id = RecordId::try_from([0x12; ID_LENGTH].as_slice()).unwrap();
        journal
            .entries
            .push(RetainedJournalEntry::Common(CommonRetainedJournalEntry {
                registry_id,
                entry_index: JournalEntryIndex::try_from(1).unwrap(),
                previous_entry_hash: genesis_reference.entry_hash(),
                event_type_id: EventTypeId::try_from(800).unwrap(),
                event_record_id: EventRecordId::try_from(
                    definition_record_id.as_bytes().as_slice(),
                )
                .unwrap(),
                storage_capability_class_id: storage,
                environment_observation_id: environment,
                lifecycle_object_kind: EventTypeId::try_from(800).unwrap().lifecycle_object_kind(),
                lifecycle_object_id: *definition_record_id.as_bytes(),
                freeze_attempt_intended_root_id: None,
                identity_dependencies: IdentityDependencyCollection {
                    elements: Vec::new(),
                },
                authority_dependencies: AuthorityDependencyCollection {
                    elements: Vec::new(),
                },
                authoritative_bytes: b"retained definition".to_vec(),
            }));
        let definition = journal.current_head_reference();
        let changed_storage = RecordId::try_from([0x25; ID_LENGTH].as_slice()).unwrap();
        let changed_environment = RecordId::try_from([0x26; ID_LENGTH].as_slice()).unwrap();
        let append_epoch = |journal: &mut RetainedJournal,
                            storage,
                            environment,
                            marker: &'static [u8]| {
            let previous = journal.current_head_reference();
            let index = previous.entry_index().value() + 1;
            journal
                .entries
                .push(RetainedJournalEntry::Common(CommonRetainedJournalEntry {
                    registry_id,
                    entry_index: JournalEntryIndex::try_from(index).unwrap(),
                    previous_entry_hash: previous.entry_hash(),
                    event_type_id: EventTypeId::try_from(600).unwrap(),
                    event_record_id: EventRecordId::try_from([index as u8; ID_LENGTH].as_slice())
                        .unwrap(),
                    storage_capability_class_id: storage,
                    environment_observation_id: environment,
                    lifecycle_object_kind: LifecycleObjectKind::Registry,
                    lifecycle_object_id: *registry_id.as_bytes(),
                    freeze_attempt_intended_root_id: None,
                    identity_dependencies: IdentityDependencyCollection {
                        elements: Vec::new(),
                    },
                    authority_dependencies: AuthorityDependencyCollection {
                        elements: Vec::new(),
                    },
                    authoritative_bytes: marker.to_vec(),
                }));
            journal.current_head_reference()
        };
        append_epoch(
            &mut journal,
            changed_storage,
            changed_environment,
            b"changed capability epoch",
        );
        let current_epoch = append_epoch(
            &mut journal,
            storage,
            environment,
            b"restored capability epoch",
        );
        let scope = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let method = RecordId::try_from([0x21; ID_LENGTH].as_slice()).unwrap();
        let evidence = RecordId::try_from([0x22; ID_LENGTH].as_slice()).unwrap();

        let check = |provenance: Option<(bool, RecordId, RecordId, bool)>,
                     capability_epoch: &JournalReference| {
            let provenance_bytes = provenance.map(
                |(cache_reused, provenance_storage, provenance_environment, _)| {
                    capability_observation_provenance_record(
                        provenance_storage,
                        provenance_environment,
                        cache_reused,
                        true,
                    )
                },
            );
            let provenance_id = provenance_bytes.as_ref().map(|bytes| {
                StrictRecordFrame::decode_authoritative(bytes)
                    .unwrap()
                    .record_id()
            });
            let mut record_bytes = Vec::new();
            record_bytes.push(0x84);
            encode_text(&mut record_bytes, "EvidenceRegistry.Record.v1");
            encode_uint(&mut record_bytes, 81);
            encode_uint(&mut record_bytes, 1);
            record_bytes.push(if provenance_id.is_some() { 0xaf } else { 0xae });
            record_bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 81, 0x10]);
            record_bytes.extend_from_slice(&definition.authoritative_cbor());
            record_bytes.push(0x11);
            encode_bstr_32(&mut record_bytes, scope.as_bytes());
            record_bytes.push(0x12);
            encode_bstr_32(&mut record_bytes, method.as_bytes());
            record_bytes.extend_from_slice(&[0x13, 0x01, 0x14, 0x01, 0x15, 0x81]);
            encode_bstr_32(&mut record_bytes, evidence.as_bytes());
            record_bytes.push(0x16);
            encode_bstr_32(&mut record_bytes, storage.as_bytes());
            record_bytes.push(0x17);
            encode_bstr_32(&mut record_bytes, environment.as_bytes());
            record_bytes.extend_from_slice(&[0x18, 0x18]);
            record_bytes.extend_from_slice(&capability_epoch.authoritative_cbor());
            if let Some(provenance_id) = provenance_id {
                record_bytes.extend_from_slice(&[0x18, 0x19]);
                encode_bstr_32(&mut record_bytes, provenance_id.as_bytes());
            }
            record_bytes.extend_from_slice(&[0x18, 0x1a, 0x80, 0x18, 0x1c]);
            record_bytes.extend_from_slice(&genesis_reference.authoritative_cbor());
            record_bytes.extend_from_slice(&[0x18, 0x1d, 0x80]);
            let record_id = StrictRecordFrame::decode_authoritative(&record_bytes)
                .unwrap()
                .record_id();
            assert_eq!(
                validate_event_record_type_local_schema(801, &record_bytes),
                Ok(())
            );
            assert_eq!(
                event_semantic_authority_is_unavailable(801, &record_bytes, &[]),
                Ok(provenance_id.is_none())
            );
            let mut identity_ids = vec![scope, method, evidence, storage, environment];
            if let Some(provenance_id) = provenance_id {
                identity_ids.push(provenance_id);
            }
            let entry = RetainedJournalEntry::Common(CommonRetainedJournalEntry {
                registry_id,
                entry_index: JournalEntryIndex::try_from(
                    journal.current_head_reference().entry_index().value() + 1,
                )
                .unwrap(),
                previous_entry_hash: journal.current_head_reference().entry_hash(),
                event_type_id: EventTypeId::try_from(801).unwrap(),
                event_record_id: EventRecordId::try_from(record_id.as_bytes().as_slice()).unwrap(),
                storage_capability_class_id: storage,
                environment_observation_id: environment,
                lifecycle_object_kind: EventTypeId::try_from(801).unwrap().lifecycle_object_kind(),
                lifecycle_object_id: *record_id.as_bytes(),
                freeze_attempt_intended_root_id: None,
                identity_dependencies: IdentityDependencyCollection {
                    elements: identity_ids
                        .into_iter()
                        .map(IdentityDependency::RecordId)
                        .collect(),
                },
                authority_dependencies: AuthorityDependencyCollection {
                    elements: vec![definition.clone()],
                },
                authoritative_bytes: Vec::new(),
            });
            let exact_provenance_available = provenance
                .map(|(_, _, _, exact_provenance_available)| exact_provenance_available)
                .unwrap_or(false);
            let mut records = if exact_provenance_available {
                provenance_id
                    .zip(provenance_bytes)
                    .into_iter()
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            records.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
            validate_event_record_reference_bindings(&journal, &entry, &record_bytes, &records)
        };

        assert_eq!(check(None, &current_epoch), Ok(()));
        assert_eq!(
            check(
                Some((true, storage, environment, true)),
                &genesis_reference
            ),
            Err(RecordDecodeError),
            "GENESIS is stale after the first material capability change even when later facts return"
        );
        assert_eq!(
            check(Some((true, storage, environment, true)), &current_epoch),
            Ok(())
        );
        assert_eq!(
            check(Some((true, storage, environment, true)), &definition),
            Err(RecordDecodeError),
            "an arbitrary prior event is not the capability epoch"
        );
        assert_eq!(
            check(Some((false, storage, environment, true)), &current_epoch),
            Err(RecordDecodeError),
            "asserted cache reuse requires provenance that records cache_reused=true"
        );
        assert_eq!(
            check(
                Some((true, changed_storage, environment, true)),
                &current_epoch
            ),
            Err(RecordDecodeError),
            "cache provenance must bind the exact current storage capability"
        );
        assert_eq!(
            check(
                Some((true, storage, changed_environment, true)),
                &current_epoch
            ),
            Err(RecordDecodeError),
            "cache provenance must bind the exact current environment observation"
        );
        assert_eq!(
            check(Some((true, storage, environment, false)), &current_epoch),
            Err(RecordDecodeError),
            "key 25 requires the exact referenced provenance Record bytes"
        );
    }

    #[test]
    fn fresh_event_801_reaches_the_production_semantic_authority_boundary() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let storage = RecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap();
        let environment = RecordId::try_from([0x21; ID_LENGTH].as_slice()).unwrap();
        let genesis_record = GenesisRecord::new(GenesisRecordInput {
            registry_id,
            journal_format_version: 1,
            record_identity_profile_id: 1,
            storage_capability_class_id: storage,
            environment_observation_id: environment,
            created_by_tool_version: "fresh-event-801-production-test".to_owned(),
        })
        .unwrap();
        let genesis = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from(genesis_record.record_id().as_bytes().as_slice()).unwrap(),
            storage,
            environment,
        );
        let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
        let genesis_reference = journal.current_head_reference();
        let scope = RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap();
        let method = RecordId::try_from([0x31; ID_LENGTH].as_slice()).unwrap();
        let evidence = RecordId::try_from([0x32; ID_LENGTH].as_slice()).unwrap();

        let mut definition_bytes = Vec::new();
        definition_bytes.push(0x84);
        encode_text(&mut definition_bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut definition_bytes, 80);
        encode_uint(&mut definition_bytes, 1);
        definition_bytes.push(0xa8);
        definition_bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 80, 0x10]);
        encode_text(&mut definition_bytes, "test-definition");
        definition_bytes.extend_from_slice(&[0x11, 0x01, 0x12]);
        encode_text(&mut definition_bytes, "test-definition-v1");
        definition_bytes.push(0x13);
        encode_bstr_32(&mut definition_bytes, scope.as_bytes());
        definition_bytes.push(0x14);
        encode_bstr_32(&mut definition_bytes, method.as_bytes());
        definition_bytes.push(0x15);
        definition_bytes.extend_from_slice(&genesis_reference.authoritative_cbor());
        let definition_id = StrictRecordFrame::decode_authoritative(&definition_bytes)
            .unwrap()
            .record_id();
        let definition_entry = RetainedJournalEntry::Common(CommonRetainedJournalEntry {
            registry_id,
            entry_index: JournalEntryIndex::try_from(1).unwrap(),
            previous_entry_hash: genesis_reference.entry_hash(),
            event_type_id: EventTypeId::try_from(800).unwrap(),
            event_record_id: EventRecordId::try_from(definition_id.as_bytes().as_slice()).unwrap(),
            storage_capability_class_id: storage,
            environment_observation_id: environment,
            lifecycle_object_kind: EventTypeId::try_from(800).unwrap().lifecycle_object_kind(),
            lifecycle_object_id: *definition_id.as_bytes(),
            freeze_attempt_intended_root_id: None,
            identity_dependencies: IdentityDependencyCollection {
                elements: vec![
                    IdentityDependency::RecordId(scope),
                    IdentityDependency::RecordId(method),
                ],
            },
            authority_dependencies: AuthorityDependencyCollection {
                elements: Vec::new(),
            },
            authoritative_bytes: b"definition entry".to_vec(),
        });
        journal.entries.push(definition_entry);
        let definition_reference = journal.current_head_reference();

        let mut fresh_bytes = Vec::new();
        fresh_bytes.push(0x84);
        encode_text(&mut fresh_bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut fresh_bytes, 81);
        encode_uint(&mut fresh_bytes, 1);
        fresh_bytes.push(0xae);
        fresh_bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 81, 0x10]);
        fresh_bytes.extend_from_slice(&definition_reference.authoritative_cbor());
        fresh_bytes.push(0x11);
        encode_bstr_32(&mut fresh_bytes, scope.as_bytes());
        fresh_bytes.push(0x12);
        encode_bstr_32(&mut fresh_bytes, method.as_bytes());
        fresh_bytes.extend_from_slice(&[0x13, 0x01, 0x14, 0x01, 0x15, 0x81]);
        encode_bstr_32(&mut fresh_bytes, evidence.as_bytes());
        fresh_bytes.push(0x16);
        encode_bstr_32(&mut fresh_bytes, storage.as_bytes());
        fresh_bytes.push(0x17);
        encode_bstr_32(&mut fresh_bytes, environment.as_bytes());
        fresh_bytes.extend_from_slice(&[0x18, 0x18]);
        fresh_bytes.extend_from_slice(&genesis_reference.authoritative_cbor());
        fresh_bytes.extend_from_slice(&[0x18, 0x1a, 0x80, 0x18, 0x1c]);
        fresh_bytes.extend_from_slice(&genesis_reference.authoritative_cbor());
        fresh_bytes.extend_from_slice(&[0x18, 0x1d, 0x80]);
        let fresh_id = StrictRecordFrame::decode_authoritative(&fresh_bytes)
            .unwrap()
            .record_id();
        journal
            .entries
            .push(RetainedJournalEntry::Common(CommonRetainedJournalEntry {
                registry_id,
                entry_index: JournalEntryIndex::try_from(2).unwrap(),
                previous_entry_hash: definition_reference.entry_hash(),
                event_type_id: EventTypeId::try_from(801).unwrap(),
                event_record_id: EventRecordId::try_from(fresh_id.as_bytes().as_slice()).unwrap(),
                storage_capability_class_id: storage,
                environment_observation_id: environment,
                lifecycle_object_kind: EventTypeId::try_from(801).unwrap().lifecycle_object_kind(),
                lifecycle_object_id: *fresh_id.as_bytes(),
                freeze_attempt_intended_root_id: None,
                identity_dependencies: IdentityDependencyCollection {
                    elements: vec![
                        IdentityDependency::RecordId(storage),
                        IdentityDependency::RecordId(environment),
                        IdentityDependency::RecordId(scope),
                        IdentityDependency::RecordId(method),
                        IdentityDependency::RecordId(evidence),
                    ],
                },
                authority_dependencies: AuthorityDependencyCollection {
                    elements: vec![definition_reference],
                },
                authoritative_bytes: b"fresh establishment entry".to_vec(),
            }));
        let mut records = vec![
            (
                genesis_record.record_id(),
                genesis_record.authoritative_cbor(),
            ),
            (definition_id, definition_bytes),
            (fresh_id, fresh_bytes),
        ];
        records.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));

        assert_eq!(
            validate_authoritative_event_records(&journal, &records),
            Err(AuthoritativeRegistryStoreOpenError::EventSemanticAuthorityUnavailable)
        );
    }

    #[test]
    fn eviction_terminal_requires_an_exact_eviction_start_authority_type() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let genesis = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
            RecordId::try_from([0x40; ID_LENGTH].as_slice()).unwrap(),
        );
        let journal = RetainedJournal::from_genesis(genesis).unwrap();
        let wrong_start = journal.current_head_reference();
        let mut record_bytes = Vec::new();
        record_bytes.push(0x84);
        encode_text(&mut record_bytes, "EvidenceRegistry.Record.v1");
        encode_uint(&mut record_bytes, 72);
        encode_uint(&mut record_bytes, 1);
        record_bytes.push(0xa6);
        record_bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x18, 72, 0x10]);
        encode_bstr_32(&mut record_bytes, &[0x50; ID_LENGTH]);
        record_bytes.push(0x11);
        record_bytes.extend_from_slice(&wrong_start.authoritative_cbor());
        record_bytes.extend_from_slice(&[0x12, 0x01, 0x14, 0x81]);
        encode_text(&mut record_bytes, "COMMITTED");
        let record_id = StrictRecordFrame::decode_authoritative(&record_bytes)
            .unwrap()
            .record_id();
        let entry = RetainedJournalEntry::Common(CommonRetainedJournalEntry {
            registry_id,
            entry_index: JournalEntryIndex::try_from(1).unwrap(),
            previous_entry_hash: wrong_start.entry_hash(),
            event_type_id: EventTypeId::try_from(701).unwrap(),
            event_record_id: EventRecordId::try_from(record_id.as_bytes().as_slice()).unwrap(),
            storage_capability_class_id: RecordId::try_from([0x30; ID_LENGTH].as_slice()).unwrap(),
            environment_observation_id: RecordId::try_from([0x40; ID_LENGTH].as_slice()).unwrap(),
            lifecycle_object_kind: LifecycleObjectKind::ArtifactEvictionAttempt,
            lifecycle_object_id: [0x50; ID_LENGTH],
            freeze_attempt_intended_root_id: None,
            identity_dependencies: IdentityDependencyCollection {
                elements: Vec::new(),
            },
            authority_dependencies: AuthorityDependencyCollection {
                elements: vec![wrong_start],
            },
            authoritative_bytes: Vec::new(),
        });

        assert_eq!(
            validate_event_record_reference_bindings(&journal, &entry, &record_bytes, &[]),
            Err(RecordDecodeError)
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn freeze_entry_for_record_binding_test(
        registry_id: RegistryId,
        entry_index: u64,
        previous_entry_hash: JournalEntryHash,
        event_type: u16,
        event_record_id: RecordId,
        freeze_attempt_id: [u8; ID_LENGTH],
        authority: Option<&JournalReference>,
        second_event_key: u8,
        second_event_field: &[u8],
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
        bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
        bytes.push(0xae);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01]);
        encode_bstr_32(&mut bytes, registry_id.as_bytes());
        bytes.push(0x02);
        encode_uint(&mut bytes, entry_index);
        bytes.push(0x03);
        encode_bstr_32(&mut bytes, previous_entry_hash.as_bytes());
        bytes.push(0x04);
        encode_uint(&mut bytes, u64::from(event_type));
        bytes.push(0x05);
        encode_bstr_32(&mut bytes, event_record_id.as_bytes());
        bytes.push(0x06);
        bytes.push(0x80);
        bytes.push(0x07);
        if let Some(authority) = authority {
            bytes.push(0x81);
            bytes.extend_from_slice(&authority.authoritative_cbor());
        } else {
            bytes.push(0x80);
        }
        bytes.extend_from_slice(&[0x08, 0x02, 0x09]);
        encode_bstr_32(&mut bytes, &freeze_attempt_id);
        bytes.push(0x0a);
        encode_bstr_32(&mut bytes, &[0x40; ID_LENGTH]);
        bytes.push(0x0b);
        encode_bstr_32(&mut bytes, &[0x60; ID_LENGTH]);
        bytes.push(0x10);
        encode_bstr_32(&mut bytes, &freeze_attempt_id);
        bytes.push(second_event_key);
        bytes.extend_from_slice(second_event_field);
        bytes
    }

    fn generic_abort_binding_fixture(
        record_freeze_attempt_id: [u8; ID_LENGTH],
        record_start_matches_journal_authority: bool,
    ) -> (RetainedJournal, Vec<(RecordId, Vec<u8>)>) {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let genesis_record = GenesisRecord::new(GenesisRecordInput {
            registry_id,
            journal_format_version: 1,
            record_identity_profile_id: 1,
            storage_capability_class_id: RecordId::try_from([0x40; ID_LENGTH].as_slice()).unwrap(),
            environment_observation_id: RecordId::try_from([0x60; ID_LENGTH].as_slice()).unwrap(),
            created_by_tool_version: "binding-test".to_owned(),
        })
        .unwrap();
        let genesis = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from(genesis_record.record_id().as_bytes().as_slice()).unwrap(),
            RecordId::try_from([0x40; ID_LENGTH].as_slice()).unwrap(),
            RecordId::try_from([0x60; ID_LENGTH].as_slice()).unwrap(),
        );
        let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
        let freeze_attempt_id = FreezeAttemptId::try_from([0x70; ID_LENGTH].as_slice()).unwrap();
        let start_record = FreezeAttemptStartRecord::new(FreezeAttemptStartRecordInput {
            freeze_attempt_id,
            intended_root_id: derive_freeze_root(registry_id, freeze_attempt_id).intended_root_id(),
            subject_id: [0x80; ID_LENGTH],
            policy_record_id: RecordId::try_from([0x90; ID_LENGTH].as_slice()).unwrap(),
        });
        let genesis_reference = journal.current_head_reference();
        let start_entry = freeze_entry_for_record_binding_test(
            registry_id,
            1,
            journal.current_head_reference().entry_hash(),
            100,
            start_record.record_id(),
            *freeze_attempt_id.as_bytes(),
            None,
            0x11,
            &{
                let mut value = Vec::new();
                encode_bstr_32(&mut value, start_record.input().intended_root_id.as_bytes());
                value
            },
        );
        journal.append_strict_entry(&start_entry).unwrap();
        let start_reference = journal.current_head_reference();

        let mut abort_record = Vec::new();
        abort_record.push(0x84);
        encode_text(&mut abort_record, "EvidenceRegistry.Record.v1");
        encode_uint(&mut abort_record, 5);
        encode_uint(&mut abort_record, 1);
        abort_record.push(0xa6);
        abort_record.extend_from_slice(&[0x00, 0x01, 0x01, 0x05, 0x10]);
        encode_bstr_32(&mut abort_record, &record_freeze_attempt_id);
        abort_record.push(0x11);
        abort_record.extend_from_slice(
            if record_start_matches_journal_authority {
                &start_reference
            } else {
                &genesis_reference
            }
            .authoritative_cbor()
            .as_slice(),
        );
        abort_record.extend_from_slice(&[0x13, 0x80, 0x14]);
        abort_record.extend_from_slice(&journal.current_head_reference().authoritative_cbor());
        let abort_record_id = StrictRecordFrame::decode_authoritative(&abort_record)
            .unwrap()
            .record_id();
        let abort_entry = freeze_entry_for_record_binding_test(
            registry_id,
            2,
            start_reference.entry_hash(),
            102,
            abort_record_id,
            *freeze_attempt_id.as_bytes(),
            Some(&start_reference),
            0x12,
            &start_reference.authoritative_cbor(),
        );
        journal.append_strict_entry(&abort_entry).unwrap();
        let mut records = vec![
            (
                genesis_record.record_id(),
                genesis_record.authoritative_cbor(),
            ),
            (start_record.record_id(), start_record.authoritative_cbor()),
            (abort_record_id, abort_record),
        ];
        records.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
        (journal, records)
    }

    #[test]
    fn authoritative_replay_rejects_a_generic_record_for_a_different_lifecycle_object() {
        let (journal, records) = generic_abort_binding_fixture([0x71; ID_LENGTH], true);
        assert_eq!(
            validate_authoritative_event_records(&journal, &records),
            Err(AuthoritativeRegistryStoreOpenError::EventRecordDecode)
        );
    }

    #[test]
    fn authoritative_replay_requires_a_generic_records_named_authority_dependency() {
        let (journal, records) = generic_abort_binding_fixture([0x70; ID_LENGTH], false);
        assert_eq!(
            validate_authoritative_event_records(&journal, &records),
            Err(AuthoritativeRegistryStoreOpenError::EventRecordDecode)
        );
    }

    #[test]
    fn retained_replay_accepts_a_valid_freeze_commit_rejection_entry() {
        let (mut journal, _) = generic_abort_binding_fixture([0x70; ID_LENGTH], true);
        let registry_id = journal.registry_id;
        let start_entry = &journal.entries[1];
        let start_reference = JournalReference::new(
            registry_id,
            start_entry.entry_index(),
            start_entry.entry_hash(),
            start_entry.event_type_id(),
            start_entry.event_record_id(),
        );
        let conflicting_terminal = journal.current_head_reference();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
        bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
        bytes.push(0xaf);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01]);
        encode_bstr_32(&mut bytes, registry_id.as_bytes());
        bytes.push(0x02);
        encode_uint(&mut bytes, 3);
        bytes.push(0x03);
        encode_bstr_32(&mut bytes, conflicting_terminal.entry_hash().as_bytes());
        bytes.push(0x04);
        encode_uint(&mut bytes, 104);
        bytes.push(0x05);
        encode_bstr_32(&mut bytes, &[0xa0; ID_LENGTH]);
        bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x82]);
        bytes.extend_from_slice(&start_reference.authoritative_cbor());
        bytes.extend_from_slice(&conflicting_terminal.authoritative_cbor());
        bytes.extend_from_slice(&[0x08, 0x02, 0x09]);
        encode_bstr_32(&mut bytes, &[0x70; ID_LENGTH]);
        bytes.push(0x0a);
        encode_bstr_32(&mut bytes, &[0x40; ID_LENGTH]);
        bytes.push(0x0b);
        encode_bstr_32(&mut bytes, &[0x60; ID_LENGTH]);
        bytes.push(0x10);
        encode_bstr_32(&mut bytes, &[0x70; ID_LENGTH]);
        bytes.push(0x12);
        bytes.extend_from_slice(&start_reference.authoritative_cbor());
        bytes.push(0x13);
        bytes.extend_from_slice(&conflicting_terminal.authoritative_cbor());

        assert_eq!(journal.append_strict_entry(&bytes), Ok(()));
        assert_eq!(
            journal.current_head_reference().event_type_id().value(),
            104
        );
    }

    fn policy_recorded_entry_for_ancestor_test(
        registry_id: RegistryId,
        entry_index: u64,
        previous_entry_hash: JournalEntryHash,
        policy_record_id: RecordId,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
        bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
        bytes.push(0xac);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01]);
        encode_bstr_32(&mut bytes, registry_id.as_bytes());
        bytes.push(0x02);
        encode_uint(&mut bytes, entry_index);
        bytes.push(0x03);
        encode_bstr_32(&mut bytes, previous_entry_hash.as_bytes());
        bytes.push(0x04);
        encode_uint(&mut bytes, 400);
        bytes.push(0x05);
        encode_bstr_32(&mut bytes, policy_record_id.as_bytes());
        bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x80, 0x08]);
        encode_uint(&mut bytes, 7);
        bytes.push(0x09);
        encode_bstr_32(&mut bytes, policy_record_id.as_bytes());
        bytes.push(0x0a);
        encode_bstr_32(&mut bytes, &[0x40; ID_LENGTH]);
        bytes.push(0x0b);
        encode_bstr_32(&mut bytes, &[0x60; ID_LENGTH]);
        bytes
    }

    #[test]
    fn published_reference_authentication_accepts_an_exact_retained_ancestor() {
        let registry_id = RegistryId::try_from([0x10; ID_LENGTH].as_slice()).unwrap();
        let genesis = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from([0x20; ID_LENGTH].as_slice()).unwrap(),
            RecordId::try_from([0x40; ID_LENGTH].as_slice()).unwrap(),
            RecordId::try_from([0x60; ID_LENGTH].as_slice()).unwrap(),
        );
        let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
        let first = policy_recorded_entry_for_ancestor_test(
            registry_id,
            1,
            journal.current_head_reference().entry_hash(),
            RecordId::try_from([0x81; ID_LENGTH].as_slice()).unwrap(),
        );
        journal.append_strict_entry(&first).unwrap();
        let published = journal.current_head_reference();
        let second = policy_recorded_entry_for_ancestor_test(
            registry_id,
            2,
            published.entry_hash(),
            RecordId::try_from([0x82; ID_LENGTH].as_slice()).unwrap(),
        );
        journal.append_strict_entry(&second).unwrap();
        assert_ne!(journal.current_head_reference(), published);

        assert_eq!(
            authenticate_published_journal_reference(&journal, &published),
            Ok(())
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_retained_generation_guard_denies_path_and_alias_writes() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-generation-guard-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let path = root.join("retained.cbor");
        let alias = root.join("retained-alias.cbor");
        fs::write(&path, b"retained generation").unwrap();
        let source = read_regular_file(&path, &mut NamespaceBudget::new()).unwrap();
        let mut guard = open_retained_file_guard(&source.witness).unwrap();

        assert!(fs::write(&path, b"path mutation").is_err());
        revalidate_retained_file_witness(&mut guard).unwrap();

        drop(guard);
        fs::hard_link(&path, &alias).unwrap();
        assert!(open_retained_file_guard(&source.witness).is_err());
        drop(source);
        fs::remove_file(alias).unwrap();
        fs::remove_file(path).unwrap();
        fs::remove_dir(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_revalidation_preserves_the_retained_handle_cursor() {
        use std::io::{Seek, SeekFrom};

        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "evidence-registry-macos-positional-revalidate-{}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let path = root.join("retained.cbor");
        fs::write(&path, b"retained positional bytes").unwrap();
        let mut retained = read_regular_file(&path, &mut NamespaceBudget::new()).unwrap();
        retained.witness.file.seek(SeekFrom::Start(7)).unwrap();
        let mut shared = retained.witness.file.try_clone().unwrap();
        assert_eq!(shared.stream_position().unwrap(), 7);
        revalidate_retained_file_witness(&mut retained.witness).unwrap();
        assert_eq!(retained.witness.file.stream_position().unwrap(), 7);
        assert_eq!(shared.stream_position().unwrap(), 7);
        drop(shared);
        drop(retained);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn authoritative_namespace_budget_bounds_object_count_independently() {
        let mut budget = NamespaceBudget::new();
        for _ in 0..AUTHORITATIVE_STORE_MAX_OBJECTS {
            budget.reserve(0).unwrap();
        }
        assert_eq!(budget.reserve(0), Err(NamespaceReadError::ResourceLimit));
    }

    #[test]
    fn authoritative_namespace_budget_bounds_aggregate_bytes_independently() {
        let mut budget = NamespaceBudget::new();
        for _ in 0..(AUTHORITATIVE_STORE_MAX_NAMESPACE_BYTES / AUTHORITATIVE_STORE_MAX_OBJECT_BYTES)
        {
            budget
                .reserve(u64::try_from(AUTHORITATIVE_STORE_MAX_OBJECT_BYTES).unwrap())
                .unwrap();
        }
        assert_eq!(budget.reserve(1), Err(NamespaceReadError::ResourceLimit));
    }

    #[test]
    fn record_loader_counts_publication_residue_in_its_single_namespace_pass() {
        let sequence = NEXT_AUTHORITATIVE_PUBLICATION_TEMP.fetch_add(1, Ordering::Relaxed);
        let records = std::env::temp_dir().join(format!(
            "evidence-registry-record-budget-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&records).unwrap();
        fs::write(
            records.join(".evidence-registry-publish-1-1.tmp"),
            b"residue",
        )
        .unwrap();
        let mut budget = NamespaceBudget::new();
        for _ in 0..AUTHORITATIVE_STORE_MAX_OBJECTS {
            budget.reserve(0).unwrap();
        }

        assert!(matches!(
            load_record_namespace(&records, &mut budget),
            Err(AuthoritativeRegistryStoreOpenError::NamespaceResourceLimit)
        ));

        fs::remove_dir_all(records).unwrap();
    }

    #[test]
    fn frozen_event_record_type_map_is_complete() {
        let expected = [
            (1, 1),
            (100, 3),
            (101, 4),
            (102, 5),
            (103, 6),
            (104, 7),
            (200, 12),
            (300, 30),
            (301, 31),
            (302, 32),
            (303, 32),
            (400, 40),
            (500, 50),
            (501, 50),
            (600, 62),
            (700, 71),
            (701, 72),
            (702, 72),
            (703, 72),
            (800, 80),
            (801, 81),
            (802, 82),
            (803, 83),
            (804, 84),
            (805, 85),
            (806, 86),
            (807, 87),
            (808, 88),
        ];
        for (event_type, record_type) in expected {
            assert_eq!(
                expected_record_type_for_event(event_type),
                Some(record_type)
            );
        }
        assert_eq!(expected_record_type_for_event(0), None);
        assert_eq!(expected_record_type_for_event(809), None);
    }
}
