//! EvidenceRegistry implementation surface.
//!
//! This module begins with the deterministic FreezeRoot identity defined by
//! Identity Format v0.3 §§46–47. It deliberately exposes only the fixed
//! FreezeRoot tuple, rather than a generic identity envelope.

use sha2::{Digest, Sha256};

const FREEZE_ROOT_DOMAIN: &[u8] = b"EvidenceRegistry.FreezeRoot.v1";
const ID_LENGTH: usize = 32;

/// A Registry identity represented in its authoritative 32-byte form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegistryId([u8; ID_LENGTH]);

/// A Freeze attempt identity represented in its authoritative 32-byte form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FreezeAttemptId([u8; ID_LENGTH]);

/// A derived FreezeRoot identity represented in its authoritative 32-byte form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntendedRootId([u8; ID_LENGTH]);

/// A rejected identity input whose byte-string length was not authoritative.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdLengthError {
    actual_length: usize,
}

impl IdLengthError {
    /// The received byte length.
    pub fn actual_length(self) -> usize {
        self.actual_length
    }
}

macro_rules! define_32_byte_identity {
    ($identity:ident) => {
        impl TryFrom<&[u8]> for $identity {
            type Error = IdLengthError;

            fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
                if value.len() != ID_LENGTH {
                    return Err(IdLengthError {
                        actual_length: value.len(),
                    });
                }

                let mut bytes = [0u8; ID_LENGTH];
                bytes.copy_from_slice(value);
                Ok(Self(bytes))
            }
        }

        impl $identity {
            /// Returns the exact authoritative bytes.
            pub fn as_bytes(&self) -> &[u8; ID_LENGTH] {
                &self.0
            }
        }
    };
}

define_32_byte_identity!(RegistryId);
define_32_byte_identity!(FreezeAttemptId);
define_32_byte_identity!(IntendedRootId);

/// The deterministic, canonical FreezeRoot identity tuple and its SHA-256 ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeRootDerivation {
    identity_bytes: Vec<u8>,
    intended_root_id: IntendedRootId,
}

impl FreezeRootDerivation {
    /// Returns the canonical CBOR tuple from which the root ID was derived.
    pub fn identity_bytes(&self) -> &[u8] {
        &self.identity_bytes
    }

    /// Returns `SHA256(identity_bytes)` as an authoritative 32-byte value.
    pub fn intended_root_id(&self) -> IntendedRootId {
        self.intended_root_id
    }
}

/// Derives the Identity Format v0.3 §46 FreezeRoot tuple and its intended root ID.
///
/// The bytes are exactly the definite-length CBOR array:
/// `["EvidenceRegistry.FreezeRoot.v1", registry_id, freeze_attempt_id]`.
pub fn derive_freeze_root(
    registry_id: RegistryId,
    freeze_attempt_id: FreezeAttemptId,
) -> FreezeRootDerivation {
    debug_assert_eq!(FREEZE_ROOT_DOMAIN.len(), 30);

    let mut identity_bytes = Vec::with_capacity(101);
    identity_bytes.extend_from_slice(&[0x83, 0x78, 0x1e]);
    identity_bytes.extend_from_slice(FREEZE_ROOT_DOMAIN);
    identity_bytes.extend_from_slice(&[0x58, 0x20]);
    identity_bytes.extend_from_slice(registry_id.as_bytes());
    identity_bytes.extend_from_slice(&[0x58, 0x20]);
    identity_bytes.extend_from_slice(freeze_attempt_id.as_bytes());

    let digest: [u8; ID_LENGTH] = Sha256::digest(&identity_bytes)
        .as_slice()
        .try_into()
        .expect("SHA-256 always returns exactly 32 bytes");

    FreezeRootDerivation {
        identity_bytes,
        intended_root_id: IntendedRootId(digest),
    }
}

/// The maximum exact unsigned integer permitted by Identity Format v0.3 §11.
pub const ER_UINT_MAX: u64 = 9_007_199_254_740_991;

/// A SHA-256 Journal Entry hash in authoritative 32-byte form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JournalEntryHash([u8; ID_LENGTH]);

/// A content-addressed event Record identity in authoritative 32-byte form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventRecordId([u8; ID_LENGTH]);

define_32_byte_identity!(JournalEntryHash);
define_32_byte_identity!(EventRecordId);

/// A validated Journal Entry index in the frozen exact-integer range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JournalEntryIndex(u64);

/// An error returned for an index outside `0..=ER_UINT_MAX`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JournalEntryIndexRangeError {
    value: u64,
}

impl JournalEntryIndexRangeError {
    /// The out-of-range value.
    pub fn value(self) -> u64 {
        self.value
    }
}

impl TryFrom<u64> for JournalEntryIndex {
    type Error = JournalEntryIndexRangeError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > ER_UINT_MAX {
            return Err(JournalEntryIndexRangeError { value });
        }
        Ok(Self(value))
    }
}

impl JournalEntryIndex {
    /// The validated unsigned index.
    pub fn value(self) -> u64 {
        self.0
    }
}

/// A v0.x lifecycle object kind from Identity Format v0.3 §29.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleObjectKind {
    /// Numeric kind 1.
    Registry,
    /// Numeric kind 2.
    FreezeAttempt,
    /// Numeric kind 3.
    Verification,
    /// Numeric kind 4.
    ReviewRequest,
    /// Numeric kind 5.
    ReviewResult,
    /// Numeric kind 6.
    ReviewAdmissionAttempt,
    /// Numeric kind 7.
    Policy,
    /// Numeric kind 8.
    CloseoutAttempt,
    /// Numeric kind 9.
    ArtifactEvictionAttempt,
    /// Numeric kind 10.
    AssumptionDefinition,
    /// Numeric kind 11.
    AssumptionEstablishment,
    /// Numeric kind 12.
    AssumptionInvalidation,
    /// Numeric kind 13.
    FormalVerification,
    /// Numeric kind 14.
    AssumptionVersionCompatibility,
    /// Numeric kind 15.
    AssumptionVersionCompatibilityInvalidation,
    /// Numeric kind 16.
    BootstrapTrustDeclaration,
    /// Numeric kind 17.
    BootstrapTrustInvalidation,
    /// Numeric kind 18.
    FormalFindingClassification,
}

/// The lifecycle state-transition shape assigned to a registered event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventShape {
    /// Creates the Registry from its absent state.
    BootstrapCreate,
    /// Creates a nonterminal attempt from its absent state.
    CreateOpen,
    /// Creates a terminal one-shot object from its absent state.
    CreateTerminal,
    /// Moves an open attempt into a terminal state.
    TransitionTerminal,
    /// Observes an existing state without changing it.
    ObserveNoStateChange,
}

/// The only authoritative Registry lifecycle states from Cross-Reference v0.3 §9.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistryLifecycleState {
    /// No authoritative GENESIS has occurred for this Registry history.
    Absent,
    /// GENESIS established the Registry's authoritative lifecycle state.
    InitializedAuthoritative,
}

/// The authoritative lifecycle states for a Freeze Attempt from §11.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FreezeAttemptState {
    /// The attempt does not exist in authoritative lifecycle history.
    Absent,
    /// The attempt was started and has not reached a terminal state.
    Open,
    /// The attempt completed successfully.
    Committed,
    /// Recovery aborted the attempt.
    AbortedRecovery,
    /// An operator assertion aborted the attempt.
    AbortedByOperatorAssertion,
}

/// The lifecycle states shared by record-backed one-shot kinds from §12.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OneShotRecordedState {
    /// The exact lifecycle object has no authoritative event yet.
    Absent,
    /// The exact lifecycle object has been recorded and is terminal.
    Recorded,
}

/// The authoritative lifecycle states for a Review Admission Attempt from §13.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionState {
    /// The attempt has no authoritative admission event.
    Absent,
    /// The admission was accepted.
    Accepted,
    /// The admission was rejected.
    Rejected,
}

/// The authoritative lifecycle states for a Closeout Attempt from §14.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseoutAttemptState {
    /// The attempt has no authoritative closeout event.
    Absent,
    /// The closeout was committed.
    Committed,
    /// The closeout was rejected.
    Rejected,
}

/// The authoritative lifecycle states for an Artifact Eviction Attempt from §15.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactEvictionState {
    /// The attempt does not exist in authoritative lifecycle history.
    Absent,
    /// The attempt was started and has not reached a terminal state.
    Open,
    /// The eviction was committed.
    Committed,
    /// Recovery interrupted the eviction.
    InterruptedRecovery,
    /// Verification blocked the eviction before deliberate destructive removal.
    Blocked,
}

/// A kind-tagged lifecycle state. The tag is checked before terminality evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleObjectState {
    /// A Registry state.
    Registry(RegistryLifecycleState),
    /// A Freeze Attempt state.
    FreezeAttempt(FreezeAttemptState),
    /// A state for one of the record-backed one-shot kinds.
    OneShot(OneShotRecordedState),
    /// A Review Admission Attempt state.
    ReviewAdmission(ReviewAdmissionState),
    /// A Closeout Attempt state.
    CloseoutAttempt(CloseoutAttemptState),
    /// An Artifact Eviction Attempt state.
    ArtifactEviction(ArtifactEvictionState),
}

/// A rejected terminality query outside its frozen applicability domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleStateValidationError {
    /// Registry has no terminality semantics.
    NoTerminality,
    /// The state representation does not belong to the queried lifecycle kind.
    KindStateMismatch,
}

impl LifecycleObjectKind {
    /// Whether the frozen lifecycle state model defines terminality for this kind.
    pub fn has_terminality(self) -> bool {
        self != Self::Registry
    }

    /// Evaluates terminality only for a state representation valid for this kind.
    pub fn is_terminal(
        self,
        state: LifecycleObjectState,
    ) -> Result<bool, LifecycleStateValidationError> {
        if !self.has_terminality() {
            return Err(LifecycleStateValidationError::NoTerminality);
        }
        if !state_matches_kind(self, state) {
            return Err(LifecycleStateValidationError::KindStateMismatch);
        }

        Ok(match state {
            LifecycleObjectState::Registry(_) => false,
            LifecycleObjectState::FreezeAttempt(state) => matches!(
                state,
                FreezeAttemptState::Committed
                    | FreezeAttemptState::AbortedRecovery
                    | FreezeAttemptState::AbortedByOperatorAssertion
            ),
            LifecycleObjectState::OneShot(state) => state == OneShotRecordedState::Recorded,
            LifecycleObjectState::ReviewAdmission(state) => matches!(
                state,
                ReviewAdmissionState::Accepted | ReviewAdmissionState::Rejected
            ),
            LifecycleObjectState::CloseoutAttempt(state) => matches!(
                state,
                CloseoutAttemptState::Committed | CloseoutAttemptState::Rejected
            ),
            LifecycleObjectState::ArtifactEviction(state) => matches!(
                state,
                ArtifactEvictionState::Committed
                    | ArtifactEvictionState::InterruptedRecovery
                    | ArtifactEvictionState::Blocked
            ),
        })
    }
}

fn state_matches_kind(kind: LifecycleObjectKind, state: LifecycleObjectState) -> bool {
    match state {
        LifecycleObjectState::Registry(_) => kind == LifecycleObjectKind::Registry,
        LifecycleObjectState::FreezeAttempt(_) => kind == LifecycleObjectKind::FreezeAttempt,
        LifecycleObjectState::OneShot(_) => matches!(
            kind,
            LifecycleObjectKind::Verification
                | LifecycleObjectKind::ReviewRequest
                | LifecycleObjectKind::ReviewResult
                | LifecycleObjectKind::Policy
                | LifecycleObjectKind::AssumptionDefinition
                | LifecycleObjectKind::AssumptionEstablishment
                | LifecycleObjectKind::AssumptionInvalidation
                | LifecycleObjectKind::FormalVerification
                | LifecycleObjectKind::AssumptionVersionCompatibility
                | LifecycleObjectKind::AssumptionVersionCompatibilityInvalidation
                | LifecycleObjectKind::BootstrapTrustDeclaration
                | LifecycleObjectKind::BootstrapTrustInvalidation
                | LifecycleObjectKind::FormalFindingClassification
        ),
        LifecycleObjectState::ReviewAdmission(_) => {
            kind == LifecycleObjectKind::ReviewAdmissionAttempt
        }
        LifecycleObjectState::CloseoutAttempt(_) => kind == LifecycleObjectKind::CloseoutAttempt,
        LifecycleObjectState::ArtifactEviction(_) => {
            kind == LifecycleObjectKind::ArtifactEvictionAttempt
        }
    }
}

/// A current v0.x registered Journal event type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventTypeId(u16);

/// A semantic identifier that is outside the frozen v0.x event registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventTypeIdError {
    value: u64,
}

/// A rejected lifecycle-state transition in the state-only transition kernel.
///
/// This kernel intentionally excludes Journal-reference, authority-dependency,
/// Record-payload, and replay validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleTransitionError {
    /// The supplied state representation does not match the event's object kind.
    KindStateMismatch,
    /// The supplied matching state is not the table's legal predecessor.
    IllegalPredecessor,
}

impl EventTypeIdError {
    /// The rejected event identifier.
    pub fn value(self) -> u64 {
        self.value
    }
}

impl TryFrom<u64> for EventTypeId {
    type Error = EventTypeIdError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let registered = matches!(
            value,
            1 | 100..=104 | 200 | 300..=303 | 400 | 500 | 501 | 600 | 700..=703 | 800..=808
        );
        if !registered {
            return Err(EventTypeIdError { value });
        }
        Ok(Self(value as u16))
    }
}

impl EventTypeId {
    /// The registered numeric event type identifier.
    pub fn value(self) -> u16 {
        self.0
    }

    /// Returns the exact lifecycle object kind assigned by Cross-Reference v0.3 §23.
    pub fn lifecycle_object_kind(self) -> LifecycleObjectKind {
        match self.0 {
            1 | 600 => LifecycleObjectKind::Registry,
            100..=104 => LifecycleObjectKind::FreezeAttempt,
            200 => LifecycleObjectKind::Verification,
            300 => LifecycleObjectKind::ReviewRequest,
            301 => LifecycleObjectKind::ReviewResult,
            302 | 303 => LifecycleObjectKind::ReviewAdmissionAttempt,
            400 => LifecycleObjectKind::Policy,
            500 | 501 => LifecycleObjectKind::CloseoutAttempt,
            700..=703 => LifecycleObjectKind::ArtifactEvictionAttempt,
            800 => LifecycleObjectKind::AssumptionDefinition,
            801 => LifecycleObjectKind::AssumptionEstablishment,
            802 => LifecycleObjectKind::AssumptionInvalidation,
            803 => LifecycleObjectKind::FormalVerification,
            804 => LifecycleObjectKind::AssumptionVersionCompatibility,
            805 => LifecycleObjectKind::AssumptionVersionCompatibilityInvalidation,
            806 => LifecycleObjectKind::BootstrapTrustDeclaration,
            807 => LifecycleObjectKind::BootstrapTrustInvalidation,
            808 => LifecycleObjectKind::FormalFindingClassification,
            _ => unreachable!("EventTypeId only contains registered v0.x event identifiers"),
        }
    }

    /// Returns the exact state-transition shape assigned by Cross-Reference v0.3 §66.
    pub fn event_shape(self) -> EventShape {
        match self.0 {
            1 => EventShape::BootstrapCreate,
            100 | 700 => EventShape::CreateOpen,
            101..=103 | 701..=703 => EventShape::TransitionTerminal,
            104 | 600 => EventShape::ObserveNoStateChange,
            200 | 300..=303 | 400 | 500 | 501 | 800..=808 => EventShape::CreateTerminal,
            _ => unreachable!("EventTypeId only contains registered v0.x event identifiers"),
        }
    }

    /// Returns whether `before` is this event's exact state-only table predecessor.
    pub fn has_legal_predecessor(self, before: LifecycleObjectState) -> bool {
        self.resulting_state(before).is_ok()
    }

    /// Applies this event's state-only transition from the normative table.
    ///
    /// A successful result establishes only lifecycle-state continuity. It does
    /// not establish full Journal admission or authority semantics.
    pub fn resulting_state(
        self,
        before: LifecycleObjectState,
    ) -> Result<LifecycleObjectState, LifecycleTransitionError> {
        if !state_matches_kind(self.lifecycle_object_kind(), before) {
            return Err(LifecycleTransitionError::KindStateMismatch);
        }

        let result = match self.0 {
            1 => match before {
                LifecycleObjectState::Registry(RegistryLifecycleState::Absent) => {
                    LifecycleObjectState::Registry(RegistryLifecycleState::InitializedAuthoritative)
                }
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            100 => match before {
                LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Absent) => {
                    LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Open)
                }
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            101 => match before {
                LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Open) => {
                    LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Committed)
                }
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            102 => match before {
                LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Open) => {
                    LifecycleObjectState::FreezeAttempt(FreezeAttemptState::AbortedRecovery)
                }
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            103 => match before {
                LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Open) => {
                    LifecycleObjectState::FreezeAttempt(
                        FreezeAttemptState::AbortedByOperatorAssertion,
                    )
                }
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            104 => match before {
                LifecycleObjectState::FreezeAttempt(
                    FreezeAttemptState::Committed
                    | FreezeAttemptState::AbortedRecovery
                    | FreezeAttemptState::AbortedByOperatorAssertion,
                ) => before,
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            200 | 300 | 301 | 400 | 800..=808 => match before {
                LifecycleObjectState::OneShot(OneShotRecordedState::Absent) => {
                    LifecycleObjectState::OneShot(OneShotRecordedState::Recorded)
                }
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            302 => match before {
                LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Absent) => {
                    LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Accepted)
                }
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            303 => match before {
                LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Absent) => {
                    LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Rejected)
                }
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            500 => match before {
                LifecycleObjectState::CloseoutAttempt(CloseoutAttemptState::Absent) => {
                    LifecycleObjectState::CloseoutAttempt(CloseoutAttemptState::Committed)
                }
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            501 => match before {
                LifecycleObjectState::CloseoutAttempt(CloseoutAttemptState::Absent) => {
                    LifecycleObjectState::CloseoutAttempt(CloseoutAttemptState::Rejected)
                }
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            600 => match before {
                LifecycleObjectState::Registry(
                    RegistryLifecycleState::InitializedAuthoritative,
                ) => before,
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            700 => match before {
                LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Absent) => {
                    LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Open)
                }
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            701 => match before {
                LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Open) => {
                    LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Committed)
                }
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            702 => match before {
                LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Open) => {
                    LifecycleObjectState::ArtifactEviction(
                        ArtifactEvictionState::InterruptedRecovery,
                    )
                }
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            703 => match before {
                LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Open) => {
                    LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Blocked)
                }
                _ => return Err(LifecycleTransitionError::IllegalPredecessor),
            },
            _ => unreachable!("EventTypeId only contains registered v0.x event identifiers"),
        };
        Ok(result)
    }
}

/// The structural JournalReference from Identity Format v0.3 §51.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalReference {
    registry_id: RegistryId,
    entry_index: JournalEntryIndex,
    entry_hash: JournalEntryHash,
    event_type_id: EventTypeId,
    event_record_id: EventRecordId,
}

impl JournalReference {
    /// Constructs a reference whose field widths, index range, and event type are already validated.
    pub fn new(
        registry_id: RegistryId,
        entry_index: JournalEntryIndex,
        entry_hash: JournalEntryHash,
        event_type_id: EventTypeId,
        event_record_id: EventRecordId,
    ) -> Self {
        Self {
            registry_id,
            entry_index,
            entry_hash,
            event_type_id,
            event_record_id,
        }
    }

    /// The Registry identity that must equal a containing Journal Entry's Registry in Profile L.
    pub fn registry_id(&self) -> RegistryId {
        self.registry_id
    }

    /// The referenced entry index, used by dependency ordering and strictly-prior validation.
    pub fn entry_index(&self) -> JournalEntryIndex {
        self.entry_index
    }

    /// The referenced entry hash, used only as the secondary canonical ordering key.
    pub fn entry_hash(&self) -> JournalEntryHash {
        self.entry_hash
    }

    /// Emits the exact deterministic CBOR array required by §51.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(107);
        bytes.push(0x85);
        encode_bstr_32(&mut bytes, self.registry_id.as_bytes());
        encode_uint(&mut bytes, self.entry_index.value());
        encode_bstr_32(&mut bytes, self.entry_hash.as_bytes());
        encode_uint(&mut bytes, u64::from(self.event_type_id.value()));
        encode_bstr_32(&mut bytes, self.event_record_id.as_bytes());
        bytes
    }
}

fn encode_bstr_32(output: &mut Vec<u8>, bytes: &[u8; ID_LENGTH]) {
    output.extend_from_slice(&[0x58, 0x20]);
    output.extend_from_slice(bytes);
}

fn encode_uint(output: &mut Vec<u8>, value: u64) {
    debug_assert!(value <= ER_UINT_MAX);
    match value {
        0..=23 => output.push(value as u8),
        24..=0xff => output.extend_from_slice(&[0x18, value as u8]),
        0x100..=0xffff => output.extend_from_slice(&[0x19, (value >> 8) as u8, value as u8]),
        0x1_0000..=0xffff_ffff => output.extend_from_slice(&[
            0x1a,
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ]),
        _ => output.extend_from_slice(&[
            0x1b,
            (value >> 56) as u8,
            (value >> 48) as u8,
            (value >> 40) as u8,
            (value >> 32) as u8,
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ]),
    }
}

/// A content-addressed EvidenceRegistry Record identity in authoritative 32-byte form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordId([u8; ID_LENGTH]);

/// A portable Journal Anchor identity in authoritative 32-byte form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JournalAnchorId([u8; ID_LENGTH]);

define_32_byte_identity!(RecordId);
define_32_byte_identity!(JournalAnchorId);

/// An active identity-dependency kind from Identity Format v0.3 §32.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdentityDependencyKind {
    /// `RECORD_ID`, numeric kind 1.
    RecordId,
    /// `JOURNAL_ANCHOR_ID`, numeric kind 2.
    JournalAnchorId,
}

/// An unassigned or reserved identity-dependency kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdentityDependencyKindError {
    value: u64,
}

impl TryFrom<u64> for IdentityDependencyKind {
    type Error = IdentityDependencyKindError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::RecordId),
            2 => Ok(Self::JournalAnchorId),
            // 3 is RESERVED_RAW_SHA256_DIGEST and is inactive in authoritative v0.x input.
            _ => Err(IdentityDependencyKindError { value }),
        }
    }
}

impl IdentityDependencyKind {
    fn numeric_id(self) -> u8 {
        match self {
            Self::RecordId => 1,
            Self::JournalAnchorId => 2,
        }
    }
}

/// A typed identity dependency from Identity Format v0.3 §§59–60.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityDependency {
    /// A typed Record identity, kind ID 1.
    RecordId(RecordId),
    /// A typed portable Journal Anchor identity, kind ID 2.
    JournalAnchorId(JournalAnchorId),
}

impl IdentityDependency {
    /// Constructs a `RECORD_ID` dependency.
    pub fn record_id(record_id: RecordId) -> Self {
        Self::RecordId(record_id)
    }

    /// Constructs a `JOURNAL_ANCHOR_ID` dependency.
    pub fn journal_anchor_id(anchor_id: JournalAnchorId) -> Self {
        Self::JournalAnchorId(anchor_id)
    }

    fn kind_id(self) -> u8 {
        match self {
            Self::RecordId(_) => IdentityDependencyKind::RecordId.numeric_id(),
            Self::JournalAnchorId(_) => IdentityDependencyKind::JournalAnchorId.numeric_id(),
        }
    }

    fn identity_bytes(self) -> [u8; ID_LENGTH] {
        match self {
            Self::RecordId(value) => *value.as_bytes(),
            Self::JournalAnchorId(value) => *value.as_bytes(),
        }
    }

    fn canonical_key(self) -> (u8, [u8; ID_LENGTH]) {
        (self.kind_id(), self.identity_bytes())
    }

    fn append_authoritative_cbor(self, output: &mut Vec<u8>) {
        output.push(0x82);
        encode_uint(output, u64::from(self.kind_id()));
        encode_bstr_32(output, &self.identity_bytes());
    }
}

/// A duplicate semantic element in an identity-dependency set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DuplicateIdentityDependencyError;

/// A rejected authoritative collection whose semantic elements were not in strict canonical order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityDependencyCollectionValidationError {
    /// Identical kind and bytes appeared more than once.
    Duplicate,
    /// A distinct element precedes a smaller canonical key.
    NonCanonicalOrder,
}

/// A canonical set of typed identity dependencies from Identity Format v0.3 §§61–63.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityDependencyCollection {
    elements: Vec<IdentityDependency>,
}

impl IdentityDependencyCollection {
    /// Validates duplicate exclusion and canonicalizes a semantic set for authoritative production.
    pub fn from_unordered_semantic_elements(
        mut elements: Vec<IdentityDependency>,
    ) -> Result<Self, DuplicateIdentityDependencyError> {
        validate_unordered_identity_dependency_uniqueness(&elements)?;
        elements.sort_by_key(|element| element.canonical_key());
        Ok(Self { elements })
    }

    /// Validates an externally supplied authoritative order without sorting or normalizing it.
    pub fn from_authoritative_ordered_elements(
        elements: Vec<IdentityDependency>,
    ) -> Result<Self, IdentityDependencyCollectionValidationError> {
        for pair in elements.windows(2) {
            let left = pair[0].canonical_key();
            let right = pair[1].canonical_key();
            if left == right {
                return Err(IdentityDependencyCollectionValidationError::Duplicate);
            }
            if left > right {
                return Err(IdentityDependencyCollectionValidationError::NonCanonicalOrder);
            }
        }
        Ok(Self { elements })
    }

    /// Emits the always-present, definite-length canonical dependency array from §63.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + self.elements.len() * 36);
        encode_array_length(&mut bytes, self.elements.len());
        for element in &self.elements {
            element.append_authoritative_cbor(&mut bytes);
        }
        bytes
    }
}

fn encode_array_length(output: &mut Vec<u8>, length: usize) {
    let length = u64::try_from(length).expect("platform usize fits into u64");
    debug_assert!(length <= ER_UINT_MAX);
    match length {
        0..=23 => output.push(0x80 | length as u8),
        24..=0xff => output.extend_from_slice(&[0x98, length as u8]),
        0x100..=0xffff => output.extend_from_slice(&[0x99, (length >> 8) as u8, length as u8]),
        0x1_0000..=0xffff_ffff => output.extend_from_slice(&[
            0x9a,
            (length >> 24) as u8,
            (length >> 16) as u8,
            (length >> 8) as u8,
            length as u8,
        ]),
        _ => output.extend_from_slice(&[
            0x9b,
            (length >> 56) as u8,
            (length >> 48) as u8,
            (length >> 40) as u8,
            (length >> 32) as u8,
            (length >> 24) as u8,
            (length >> 16) as u8,
            (length >> 8) as u8,
            length as u8,
        ]),
    }
}

/// Context from the containing Journal Entry required to make an authority dependency canonical.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthorityDependencyContext {
    registry_id: RegistryId,
    containing_entry_index: JournalEntryIndex,
}

impl AuthorityDependencyContext {
    /// Creates the same-Registry, strictly-prior context specified by §§53.1 and 54.
    pub fn new(registry_id: RegistryId, containing_entry_index: JournalEntryIndex) -> Self {
        Self {
            registry_id,
            containing_entry_index,
        }
    }
}

/// A rejected authority-dependency set with duplicate or conflicting entry indexes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthorityDependencyCollectionError {
    /// A Reference belongs to a Registry other than the containing Journal Entry's Registry.
    CrossRegistryJournalReference,
    /// A Reference is not strictly prior to the containing Journal Entry.
    ForwardJournalReference,
    /// The same entry index and hash appeared more than once.
    Duplicate,
    /// One entry index was paired with two different hashes.
    ConflictingSameIndex,
    /// The externally supplied collection was not in strict canonical order.
    NonCanonicalOrder,
}

/// A canonical set of Journal references from Identity Format v0.3 §§54–58.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorityDependencyCollection {
    elements: Vec<JournalReference>,
}

impl AuthorityDependencyCollection {
    /// Validates same-index exclusion and canonicalizes a semantic set for authoritative production.
    pub fn from_unordered_semantic_elements(
        context: AuthorityDependencyContext,
        mut elements: Vec<JournalReference>,
    ) -> Result<Self, AuthorityDependencyCollectionError> {
        validate_authority_dependency_context(context, &elements)?;
        validate_unordered_authority_dependency_indices(&elements)?;
        elements.sort_by_key(|reference| (reference.entry_index(), reference.entry_hash()));
        Ok(Self { elements })
    }

    /// Validates an externally supplied authoritative order without sorting or normalizing it.
    pub fn from_authoritative_ordered_elements(
        context: AuthorityDependencyContext,
        elements: Vec<JournalReference>,
    ) -> Result<Self, AuthorityDependencyCollectionError> {
        validate_authority_dependency_context(context, &elements)?;
        validate_authority_dependency_indices(&elements)?;
        for pair in elements.windows(2) {
            if (pair[0].entry_index(), pair[0].entry_hash())
                > (pair[1].entry_index(), pair[1].entry_hash())
            {
                return Err(AuthorityDependencyCollectionError::NonCanonicalOrder);
            }
        }
        Ok(Self { elements })
    }

    /// Emits the always-present, definite-length canonical dependency array from §58.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + self.elements.len() * 107);
        encode_array_length(&mut bytes, self.elements.len());
        for reference in &self.elements {
            bytes.extend_from_slice(&reference.authoritative_cbor());
        }
        bytes
    }
}

fn validate_authority_dependency_context(
    context: AuthorityDependencyContext,
    elements: &[JournalReference],
) -> Result<(), AuthorityDependencyCollectionError> {
    for reference in elements {
        if reference.registry_id() != context.registry_id {
            return Err(AuthorityDependencyCollectionError::CrossRegistryJournalReference);
        }
        if reference.entry_index() >= context.containing_entry_index {
            return Err(AuthorityDependencyCollectionError::ForwardJournalReference);
        }
    }
    Ok(())
}

fn validate_unordered_identity_dependency_uniqueness(
    elements: &[IdentityDependency],
) -> Result<(), DuplicateIdentityDependencyError> {
    for (index, element) in elements.iter().enumerate() {
        if elements[index + 1..]
            .iter()
            .any(|other| element.canonical_key() == other.canonical_key())
        {
            return Err(DuplicateIdentityDependencyError);
        }
    }
    Ok(())
}

fn validate_unordered_authority_dependency_indices(
    elements: &[JournalReference],
) -> Result<(), AuthorityDependencyCollectionError> {
    for (index, reference) in elements.iter().enumerate() {
        for other in &elements[index + 1..] {
            if reference.entry_index() == other.entry_index() {
                return if reference.entry_hash() == other.entry_hash() {
                    Err(AuthorityDependencyCollectionError::Duplicate)
                } else {
                    Err(AuthorityDependencyCollectionError::ConflictingSameIndex)
                };
            }
        }
    }
    Ok(())
}

fn validate_authority_dependency_indices(
    elements: &[JournalReference],
) -> Result<(), AuthorityDependencyCollectionError> {
    for pair in elements.windows(2) {
        if pair[0].entry_index() == pair[1].entry_index() {
            return if pair[0].entry_hash() == pair[1].entry_hash() {
                Err(AuthorityDependencyCollectionError::Duplicate)
            } else {
                Err(AuthorityDependencyCollectionError::ConflictingSameIndex)
            };
        }
    }
    Ok(())
}

#[cfg(test)]
mod construction_order_tests {
    use super::*;

    #[test]
    fn unordered_identity_uniqueness_rejects_nonadjacent_duplicate_before_sorting() {
        let record_a = RecordId::try_from([0x11; ID_LENGTH].as_slice()).unwrap();
        let record_b = RecordId::try_from([0x22; ID_LENGTH].as_slice()).unwrap();
        let elements = [
            IdentityDependency::record_id(record_a),
            IdentityDependency::record_id(record_b),
            IdentityDependency::record_id(record_a),
        ];

        assert_eq!(
            validate_unordered_identity_dependency_uniqueness(&elements),
            Err(DuplicateIdentityDependencyError)
        );
    }

    #[test]
    fn unordered_authority_index_validation_rejects_nonadjacent_conflict_before_sorting() {
        let registry = RegistryId::try_from([0x33; ID_LENGTH].as_slice()).unwrap();
        let reference = |index, hash| {
            JournalReference::new(
                registry,
                JournalEntryIndex::try_from(index).unwrap(),
                JournalEntryHash::try_from([hash; ID_LENGTH].as_slice()).unwrap(),
                EventTypeId::try_from(1).unwrap(),
                EventRecordId::try_from([0x44; ID_LENGTH].as_slice()).unwrap(),
            )
        };
        let elements = [reference(1, 0x51), reference(2, 0x52), reference(1, 0x53)];

        assert_eq!(
            validate_unordered_authority_dependency_indices(&elements),
            Err(AuthorityDependencyCollectionError::ConflictingSameIndex)
        );
    }
}
