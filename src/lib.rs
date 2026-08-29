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

/// A numeric lifecycle object kind outside the frozen v0.x registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifecycleObjectKindError {
    value: u64,
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
    /// The permanent numeric kind assigned by Identity Format v0.3 §29.
    pub fn value(self) -> u8 {
        match self {
            Self::Registry => 1,
            Self::FreezeAttempt => 2,
            Self::Verification => 3,
            Self::ReviewRequest => 4,
            Self::ReviewResult => 5,
            Self::ReviewAdmissionAttempt => 6,
            Self::Policy => 7,
            Self::CloseoutAttempt => 8,
            Self::ArtifactEvictionAttempt => 9,
            Self::AssumptionDefinition => 10,
            Self::AssumptionEstablishment => 11,
            Self::AssumptionInvalidation => 12,
            Self::FormalVerification => 13,
            Self::AssumptionVersionCompatibility => 14,
            Self::AssumptionVersionCompatibilityInvalidation => 15,
            Self::BootstrapTrustDeclaration => 16,
            Self::BootstrapTrustInvalidation => 17,
            Self::FormalFindingClassification => 18,
        }
    }

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

impl LifecycleObjectKindError {
    /// The rejected numeric lifecycle object kind.
    pub fn value(self) -> u64 {
        self.value
    }
}

impl TryFrom<u64> for LifecycleObjectKind {
    type Error = LifecycleObjectKindError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Registry),
            2 => Ok(Self::FreezeAttempt),
            3 => Ok(Self::Verification),
            4 => Ok(Self::ReviewRequest),
            5 => Ok(Self::ReviewResult),
            6 => Ok(Self::ReviewAdmissionAttempt),
            7 => Ok(Self::Policy),
            8 => Ok(Self::CloseoutAttempt),
            9 => Ok(Self::ArtifactEvictionAttempt),
            10 => Ok(Self::AssumptionDefinition),
            11 => Ok(Self::AssumptionEstablishment),
            12 => Ok(Self::AssumptionInvalidation),
            13 => Ok(Self::FormalVerification),
            14 => Ok(Self::AssumptionVersionCompatibility),
            15 => Ok(Self::AssumptionVersionCompatibilityInvalidation),
            16 => Ok(Self::BootstrapTrustDeclaration),
            17 => Ok(Self::BootstrapTrustInvalidation),
            18 => Ok(Self::FormalFindingClassification),
            _ => Err(LifecycleObjectKindError { value }),
        }
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

/// An assigned Record Type identifier from Record Schema v0.3 §4.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordTypeId(u16);

/// A Record Type identifier that is unassigned, retired, or outside the v0.3 range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordTypeIdError {
    value: u64,
}

impl RecordTypeIdError {
    /// The rejected Record Type identifier.
    pub fn value(self) -> u64 {
        self.value
    }
}

impl TryFrom<u64> for RecordTypeId {
    type Error = RecordTypeIdError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let assigned = matches!(
            value,
            1..=7
                | 10..=12
                | 20..=23
                | 30..=32
                | 40
                | 50
                | 60..=63
                | 70..=72
                | 80..=90
        );
        if !assigned {
            return Err(RecordTypeIdError { value });
        }
        Ok(Self(value as u16))
    }
}

impl RecordTypeId {
    /// The assigned numeric Record Type identifier.
    pub fn value(self) -> u16 {
        self.0
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

    /// Returns the exact event-Record type required by Record Schema v0.3 §5.
    ///
    /// This mapping checks only the event-to-Record-type binding. It does not
    /// parse a Record, establish Record identity, resolve retained context, or
    /// establish authority or admission.
    pub fn required_record_type_id(self) -> RecordTypeId {
        let value = match self.0 {
            1 => 1,
            100 => 3,
            101 => 4,
            102 => 5,
            103 => 6,
            104 => 7,
            200 => 12,
            300 => 30,
            301 => 31,
            302 | 303 => 32,
            400 => 40,
            500 | 501 => 50,
            600 => 62,
            700 => 71,
            701..=703 => 72,
            800 => 80,
            801 => 81,
            802 => 82,
            803 => 83,
            804 => 84,
            805 => 85,
            806 => 86,
            807 => 87,
            808 => 88,
            _ => unreachable!("EventTypeId only contains registered v0.x event identifiers"),
        };
        RecordTypeId::try_from(value).expect("every frozen event-record mapping is assigned")
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

/// Validates the state-continuity portion of a runtime lifecycle transition.
///
/// This deliberately checks only the frozen event/kind/predecessor/result
/// relation. A caller must separately validate retained-Journal references,
/// Record payloads, authority dependencies, and admission before treating an
/// event as authoritative.
pub fn validate_state_only_legal_transition(
    event: EventTypeId,
    before: LifecycleObjectState,
    after: LifecycleObjectState,
) -> Result<(), LifecycleTransitionError> {
    if event.resulting_state(before)? == after {
        Ok(())
    } else {
        Err(LifecycleTransitionError::IllegalPredecessor)
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

    /// The referenced registered event type. This is structural metadata only;
    /// resolving the referenced Entry and its authority requires retained context.
    pub fn event_type_id(&self) -> EventTypeId {
        self.event_type_id
    }

    /// The referenced event Record identity.
    pub fn event_record_id(&self) -> EventRecordId {
        self.event_record_id
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

const JOURNAL_ENTRY_DOMAIN: &[u8] = b"EvidenceRegistry.JournalEntry.v1";
const JOURNAL_ANCHOR_DOMAIN: &[u8] = b"EvidenceRegistry.JournalAnchor.v1";

/// A structurally complete GENESIS Journal Entry from Identity Format v0.3 §§66–73.
///
/// The constructor fixes the only GENESIS event/index/previous-hash/object-kind
/// values. It does not claim to validate the referenced GENESIS Record payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenesisJournalEntry {
    registry_id: RegistryId,
    event_record_id: EventRecordId,
    storage_capability_class_id: RecordId,
    environment_observation_id: RecordId,
}

impl GenesisJournalEntry {
    /// Creates the fixed structural GENESIS Entry body from its four typed IDs.
    pub fn new(
        registry_id: RegistryId,
        event_record_id: EventRecordId,
        storage_capability_class_id: RecordId,
        environment_observation_id: RecordId,
    ) -> Self {
        Self {
            registry_id,
            event_record_id,
            storage_capability_class_id,
            environment_observation_id,
        }
    }

    /// Emits the exact canonical CBOR framing and all mandatory common fields.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        debug_assert_eq!(JOURNAL_ENTRY_DOMAIN.len(), 32);

        let mut bytes = Vec::with_capacity(225);
        bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
        bytes.extend_from_slice(JOURNAL_ENTRY_DOMAIN);

        // A definite-length map with common keys 0 through 11 in canonical
        // ascending integer-key order. GENESIS has no event-specific keys.
        bytes.push(0xac);
        bytes.extend_from_slice(&[0x00, 0x01]); // schema_version = 1
        bytes.push(0x01);
        encode_bstr_32(&mut bytes, self.registry_id.as_bytes());
        bytes.extend_from_slice(&[0x02, 0x00, 0x03, 0xf6, 0x04, 0x01]);
        bytes.push(0x05);
        encode_bstr_32(&mut bytes, self.event_record_id.as_bytes());
        bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x80, 0x08, 0x01]);
        bytes.push(0x09);
        encode_bstr_32(&mut bytes, self.registry_id.as_bytes());
        bytes.push(0x0a);
        encode_bstr_32(&mut bytes, self.storage_capability_class_id.as_bytes());
        bytes.push(0x0b);
        encode_bstr_32(&mut bytes, self.environment_observation_id.as_bytes());
        bytes
    }

    /// Returns `SHA256(authoritative_cbor())` as the Entry hash.
    pub fn entry_hash(&self) -> JournalEntryHash {
        let digest: [u8; ID_LENGTH] = Sha256::digest(self.authoritative_cbor())
            .as_slice()
            .try_into()
            .expect("SHA-256 always returns exactly 32 bytes");
        JournalEntryHash(digest)
    }

    /// Strictly decodes the complete fixed GENESIS Journal Entry grammar.
    ///
    /// This verifies canonical CBOR and the GENESIS-only structural relations;
    /// it deliberately does not load or validate the referenced GENESIS Record.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, JournalEntryDecodeError> {
        let common = decode_common_journal_entry(input)?;
        if common.entry_index.value() != 0
            || common.previous_entry_hash.is_some()
            || common.event_type_id.value() != 1
            || common.lifecycle_object_kind != LifecycleObjectKind::Registry
            || common.lifecycle_object_id != *common.registry_id.as_bytes()
            || !common.identity_dependencies.elements.is_empty()
            || !common.authority_dependencies.elements.is_empty()
        {
            return Err(JournalEntryDecodeError);
        }

        let decoded = Self::new(
            common.registry_id,
            common.event_record_id,
            common.storage_capability_class_id,
            common.environment_observation_id,
        );
        if decoded.authoritative_cbor() != input {
            return Err(JournalEntryDecodeError);
        }
        Ok(decoded)
    }
}

/// A structurally framed, non-POST `VERIFICATION_RECORDED` Journal Entry.
///
/// This type verifies exact Entry bytes, IDs, dependency collection structure,
/// and the event's explicit prior Freeze-reference shape. It does not establish
/// that the referenced Entry exists, has a matching hash, is authoritative, or
/// that the unavailable Verification Record payload is semantically valid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrdinaryVerificationJournalEntry {
    registry_id: RegistryId,
    entry_index: JournalEntryIndex,
    previous_entry_hash: JournalEntryHash,
    event_record_id: EventRecordId,
    identity_dependencies: IdentityDependencyCollection,
    authority_dependencies: AuthorityDependencyCollection,
    storage_capability_class_id: RecordId,
    environment_observation_id: RecordId,
}

/// A rejected ordinary-Verification construction or strict-decode input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrdinaryVerificationJournalEntryError;

/// A rejected Journal Entry byte sequence under the implemented strict grammar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JournalEntryDecodeError;

/// Typed fields for constructing the ordinary (non-POST) Verification form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrdinaryVerificationJournalEntryInput {
    pub registry_id: RegistryId,
    pub entry_index: JournalEntryIndex,
    pub previous_entry_hash: JournalEntryHash,
    pub event_record_id: EventRecordId,
    pub identity_dependencies: IdentityDependencyCollection,
    pub authority_dependencies: AuthorityDependencyCollection,
    pub storage_capability_class_id: RecordId,
    pub environment_observation_id: RecordId,
}

impl OrdinaryVerificationJournalEntry {
    /// Constructs the ordinary (non-POST) Verification Entry structural form.
    pub fn new(
        input: OrdinaryVerificationJournalEntryInput,
    ) -> Result<Self, OrdinaryVerificationJournalEntryError> {
        if input.entry_index.value() == 0
            || input
                .authority_dependencies
                .validate_for_context(AuthorityDependencyContext::new(
                    input.registry_id,
                    input.entry_index,
                ))
                .is_err()
            || !input.authority_dependencies.contains_event_type(101)
        {
            return Err(OrdinaryVerificationJournalEntryError);
        }

        Ok(Self {
            registry_id: input.registry_id,
            entry_index: input.entry_index,
            previous_entry_hash: input.previous_entry_hash,
            event_record_id: input.event_record_id,
            identity_dependencies: input.identity_dependencies,
            authority_dependencies: input.authority_dependencies,
            storage_capability_class_id: input.storage_capability_class_id,
            environment_observation_id: input.environment_observation_id,
        })
    }

    /// Emits the exact canonical Journal Entry framing and ordinary Verification body.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        debug_assert_eq!(JOURNAL_ENTRY_DOMAIN.len(), 32);

        let mut bytes = Vec::with_capacity(331);
        bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
        bytes.extend_from_slice(JOURNAL_ENTRY_DOMAIN);
        bytes.push(0xac);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01]);
        encode_bstr_32(&mut bytes, self.registry_id.as_bytes());
        bytes.push(0x02);
        encode_uint(&mut bytes, self.entry_index.value());
        bytes.push(0x03);
        encode_bstr_32(&mut bytes, self.previous_entry_hash.as_bytes());
        bytes.push(0x04);
        encode_uint(&mut bytes, 200);
        bytes.push(0x05);
        encode_bstr_32(&mut bytes, self.event_record_id.as_bytes());
        bytes.push(0x06);
        bytes.extend_from_slice(&self.identity_dependencies.authoritative_cbor());
        bytes.push(0x07);
        bytes.extend_from_slice(&self.authority_dependencies.authoritative_cbor());
        bytes.extend_from_slice(&[0x08, 0x03, 0x09]);
        encode_bstr_32(&mut bytes, self.event_record_id.as_bytes());
        bytes.push(0x0a);
        encode_bstr_32(&mut bytes, self.storage_capability_class_id.as_bytes());
        bytes.push(0x0b);
        encode_bstr_32(&mut bytes, self.environment_observation_id.as_bytes());
        bytes
    }

    /// Returns `SHA256(authoritative_cbor())` as the Journal Entry hash.
    pub fn entry_hash(&self) -> JournalEntryHash {
        let digest: [u8; ID_LENGTH] = Sha256::digest(self.authoritative_cbor())
            .as_slice()
            .try_into()
            .expect("SHA-256 always returns exactly 32 bytes");
        JournalEntryHash(digest)
    }

    /// Strictly decodes an ordinary Verification Entry without normalizing bytes.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, JournalEntryDecodeError> {
        let common = decode_common_journal_entry(input)?;
        let previous_entry_hash = common.previous_entry_hash.ok_or(JournalEntryDecodeError)?;
        if common.entry_index.value() == 0
            || common.event_type_id.value() != 200
            || common.lifecycle_object_kind != LifecycleObjectKind::Verification
            || common.lifecycle_object_id != *common.event_record_id.as_bytes()
        {
            return Err(JournalEntryDecodeError);
        }

        let decoded = Self::new(OrdinaryVerificationJournalEntryInput {
            registry_id: common.registry_id,
            entry_index: common.entry_index,
            previous_entry_hash,
            event_record_id: common.event_record_id,
            identity_dependencies: common.identity_dependencies,
            authority_dependencies: common.authority_dependencies,
            storage_capability_class_id: common.storage_capability_class_id,
            environment_observation_id: common.environment_observation_id,
        })
        .map_err(|_| JournalEntryDecodeError)?;
        if decoded.authoritative_cbor() != input {
            return Err(JournalEntryDecodeError);
        }
        Ok(decoded)
    }
}

/// A retained, strictly decoded Journal subset currently supported by this
/// runtime. Unsupported event forms are intentionally not normalized into this
/// type: recovery must fail closed rather than skip an unknown entry.
#[derive(Clone, Debug, PartialEq, Eq)]
enum RetainedJournalEntry {
    Genesis(GenesisJournalEntry),
    Common(CommonRetainedJournalEntry),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CommonRetainedJournalEntry {
    registry_id: RegistryId,
    entry_index: JournalEntryIndex,
    previous_entry_hash: JournalEntryHash,
    event_type_id: EventTypeId,
    event_record_id: EventRecordId,
    storage_capability_class_id: RecordId,
    environment_observation_id: RecordId,
    lifecycle_object_kind: LifecycleObjectKind,
    lifecycle_object_id: [u8; ID_LENGTH],
    freeze_attempt_intended_root_id: Option<IntendedRootId>,
    authority_dependencies: AuthorityDependencyCollection,
    authoritative_bytes: Vec<u8>,
}

impl RetainedJournalEntry {
    fn registry_id(&self) -> RegistryId {
        match self {
            Self::Genesis(entry) => entry.registry_id,
            Self::Common(entry) => entry.registry_id,
        }
    }

    fn entry_index(&self) -> JournalEntryIndex {
        match self {
            Self::Genesis(_) => JournalEntryIndex(0),
            Self::Common(entry) => entry.entry_index,
        }
    }

    fn previous_entry_hash(&self) -> Option<JournalEntryHash> {
        match self {
            Self::Genesis(_) => None,
            Self::Common(entry) => Some(entry.previous_entry_hash),
        }
    }

    fn event_type_id(&self) -> EventTypeId {
        match self {
            Self::Genesis(_) => EventTypeId(1),
            Self::Common(entry) => entry.event_type_id,
        }
    }

    fn event_record_id(&self) -> EventRecordId {
        match self {
            Self::Genesis(entry) => entry.event_record_id,
            Self::Common(entry) => entry.event_record_id,
        }
    }

    fn storage_capability_class_id(&self) -> RecordId {
        match self {
            Self::Genesis(entry) => entry.storage_capability_class_id,
            Self::Common(entry) => entry.storage_capability_class_id,
        }
    }

    fn environment_observation_id(&self) -> RecordId {
        match self {
            Self::Genesis(entry) => entry.environment_observation_id,
            Self::Common(entry) => entry.environment_observation_id,
        }
    }

    fn entry_hash(&self) -> JournalEntryHash {
        match self {
            Self::Genesis(entry) => entry.entry_hash(),
            Self::Common(entry) => JournalEntryHash(
                Sha256::digest(&entry.authoritative_bytes)
                    .as_slice()
                    .try_into()
                    .expect("SHA-256 always returns exactly 32 bytes"),
            ),
        }
    }

    fn lifecycle_object_kind(&self) -> LifecycleObjectKind {
        match self {
            Self::Genesis(_) => LifecycleObjectKind::Registry,
            Self::Common(entry) => entry.lifecycle_object_kind,
        }
    }

    fn lifecycle_object_id(&self) -> [u8; ID_LENGTH] {
        match self {
            Self::Genesis(entry) => *entry.registry_id.as_bytes(),
            Self::Common(entry) => entry.lifecycle_object_id,
        }
    }

    fn freeze_attempt_intended_root_id(&self) -> Option<IntendedRootId> {
        match self {
            Self::Genesis(_) => None,
            Self::Common(entry) => entry.freeze_attempt_intended_root_id,
        }
    }

    fn authority_dependencies(&self) -> &[JournalReference] {
        match self {
            Self::Genesis(_) => &[],
            Self::Common(entry) => &entry.authority_dependencies.elements,
        }
    }
}

/// A fail-closed retained-Journal or state-reconstruction result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetainedJournalError {
    /// An Entry belongs to a Registry other than the Journal's GENESIS Registry.
    RegistryMismatch,
    /// The next Entry index is not the unique monotonically increasing slot.
    UnexpectedEntryIndex,
    /// An Entry does not bind the preceding exact Entry hash.
    PreviousHashMismatch,
    /// A JournalReference points at an Entry absent from retained history.
    MissingReference,
    /// A retained Entry does not match every field of the supplied reference.
    ReferenceMismatch,
    /// The supported retained subset contains an impossible lifecycle transition.
    LifecycleTransition,
    /// The bytes fail the currently implemented strict Journal Entry grammar.
    DecodeError,
    /// The event needs an event-specific strict decoder not implemented by this runtime.
    UnsupportedEntry,
}

/// Exact retained-history facts resolved from one JournalReference.
///
/// This is a resolution result, not an authority or admission verdict. It
/// exposes only retained Entry identity, chain position, and lifecycle facts
/// required by a caller that must perform further contextual validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedJournalReference {
    registry_id: RegistryId,
    entry_index: JournalEntryIndex,
    entry_hash: JournalEntryHash,
    previous_entry_hash: Option<JournalEntryHash>,
    event_type_id: EventTypeId,
    event_record_id: EventRecordId,
    storage_capability_class_id: RecordId,
    environment_observation_id: RecordId,
    lifecycle_object_kind: LifecycleObjectKind,
    lifecycle_object_id: [u8; ID_LENGTH],
    freeze_attempt_intended_root_id: Option<IntendedRootId>,
}

impl ResolvedJournalReference {
    /// The retained Registry identity of the resolved Entry.
    pub fn registry_id(&self) -> RegistryId {
        self.registry_id
    }

    /// The exact retained Journal index of the resolved Entry.
    pub fn entry_index(&self) -> JournalEntryIndex {
        self.entry_index
    }

    /// The recomputed hash of the exact retained Entry bytes.
    pub fn entry_hash(&self) -> JournalEntryHash {
        self.entry_hash
    }

    /// The predecessor hash retained by this Entry, or `None` for GENESIS.
    pub fn previous_entry_hash(&self) -> Option<JournalEntryHash> {
        self.previous_entry_hash
    }

    /// The retained registered event type.
    pub fn event_type_id(&self) -> EventTypeId {
        self.event_type_id
    }

    /// The event Record identity bound into the retained Entry.
    pub fn event_record_id(&self) -> EventRecordId {
        self.event_record_id
    }

    /// The exact retained storage-capability class identity for this Entry.
    pub fn storage_capability_class_id(&self) -> RecordId {
        self.storage_capability_class_id
    }

    /// The exact retained environment-observation identity for this Entry.
    pub fn environment_observation_id(&self) -> RecordId {
        self.environment_observation_id
    }

    /// The retained lifecycle-object kind.
    pub fn lifecycle_object_kind(&self) -> LifecycleObjectKind {
        self.lifecycle_object_kind
    }

    /// The retained lifecycle-object identity.
    pub fn lifecycle_object_id(&self) -> [u8; ID_LENGTH] {
        self.lifecycle_object_id
    }

    /// The exact intended-root identity retained by a FREEZE_ATTEMPT_STARTED Entry.
    ///
    /// `None` means the resolved Entry has no such Journal-native field; it does
    /// not establish any filesystem, Manifest, Policy, or authority fact.
    pub fn freeze_attempt_intended_root_id(&self) -> Option<IntendedRootId> {
        self.freeze_attempt_intended_root_id
    }
}

/// A contiguous, in-memory retained Journal for the currently implemented
/// strict Entry subset.
///
/// It verifies exact Entry hashes, indexes, predecessor links, and reference
/// field equality for entries it retains. This is structural/history validation,
/// not an authority or admission decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetainedJournal {
    registry_id: RegistryId,
    entries: Vec<RetainedJournalEntry>,
}

/// Lifecycle facts reconstructed from every retained Entry, starting at GENESIS.
///
/// The result says only that the retained, supported bytes replay through the
/// state-only transition kernel. It does not establish an authoritative Record
/// payload, authority dependency, policy, or admission result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReconstructedJournalState {
    registry_state: RegistryLifecycleState,
    entry_count: usize,
    journal_head_index: JournalEntryIndex,
    journal_head_hash: JournalEntryHash,
    states: Vec<(LifecycleObjectKind, [u8; ID_LENGTH], LifecycleObjectState)>,
    freeze_attempts: Vec<ReconstructedFreezeAttempt>,
    capability_epochs: Vec<ReconstructedCapabilityEpoch>,
    authority_dependency_edges: Vec<ReconstructedAuthorityDependency>,
}

/// One direct authority-dependency edge indexed by retained Journal bytes.
///
/// The edge preserves the exact dependent and prerequisite Journal References.
/// It does not itself establish that either event acquired authority, that its
/// Record payload is valid, or that an authority/admission gate succeeds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReconstructedAuthorityDependency {
    dependent_entry: JournalReference,
    dependency: JournalReference,
}

impl ReconstructedAuthorityDependency {
    /// The retained Entry that declares this direct dependency.
    pub fn dependent_entry(&self) -> &JournalReference {
        &self.dependent_entry
    }

    /// The exact prior Entry Reference named by the dependent Entry.
    pub fn dependency(&self) -> &JournalReference {
        &self.dependency
    }
}

/// The capability and environment identities indexed by one retained Journal Entry.
///
/// This is Journal-only historical context. It does not establish that either
/// referenced Record payload is available, valid, authoritative, or sufficient
/// for any Policy or admission decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReconstructedCapabilityEpoch {
    entry_index: JournalEntryIndex,
    storage_capability_class_id: RecordId,
    environment_observation_id: RecordId,
}

impl ReconstructedCapabilityEpoch {
    /// The exact Journal Entry position carrying this indexed context.
    pub fn entry_index(&self) -> JournalEntryIndex {
        self.entry_index
    }

    /// The exact storage-capability-class identity indexed by the Entry.
    pub fn storage_capability_class_id(&self) -> RecordId {
        self.storage_capability_class_id
    }

    /// The exact environment-observation identity indexed by the Entry.
    pub fn environment_observation_id(&self) -> RecordId {
        self.environment_observation_id
    }
}

/// Journal-only lifecycle facts for one retained Freeze Attempt.
///
/// These facts are reconstructed from supported Journal Entries only. They do
/// not establish Receipt validity, Freeze authority, Policy satisfaction,
/// custody, durability, or admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReconstructedFreezeAttempt {
    freeze_attempt_id: FreezeAttemptId,
    intended_root_id: IntendedRootId,
    state: FreezeAttemptState,
}

impl ReconstructedFreezeAttempt {
    /// The exact Attempt identity established by its retained START Entry.
    pub fn freeze_attempt_id(&self) -> FreezeAttemptId {
        self.freeze_attempt_id
    }

    /// The exact intended root identity indexed by the retained START Entry.
    pub fn intended_root_id(&self) -> IntendedRootId {
        self.intended_root_id
    }

    /// The Journal-only lifecycle disposition reconstructed for this Attempt.
    pub fn state(&self) -> FreezeAttemptState {
        self.state
    }
}

impl ReconstructedJournalState {
    /// The Registry lifecycle state reconstructed from the retained subset.
    pub fn registry_state(&self) -> RegistryLifecycleState {
        self.registry_state
    }

    /// The number of retained Entries included in this reconstruction.
    pub fn entry_count(&self) -> usize {
        self.entry_count
    }

    /// The final retained Entry index.
    pub fn journal_head_index(&self) -> JournalEntryIndex {
        self.journal_head_index
    }

    /// The recomputed hash of the final retained Entry.
    pub fn journal_head_hash(&self) -> JournalEntryHash {
        self.journal_head_hash
    }

    /// Returns the state of one exact kind-tagged lifecycle object, if retained.
    pub fn state_for(
        &self,
        kind: LifecycleObjectKind,
        object_id: &[u8; ID_LENGTH],
    ) -> Option<LifecycleObjectState> {
        self.states
            .iter()
            .find(|(stored_kind, stored_id, _)| *stored_kind == kind && stored_id == object_id)
            .map(|(_, _, state)| *state)
    }

    /// Every Freeze Attempt retained by the supported Journal replay, ordered
    /// by its START Entry's Journal position.
    pub fn freeze_attempts(&self) -> &[ReconstructedFreezeAttempt] {
        &self.freeze_attempts
    }

    /// The capability epoch indexed by every retained Entry, in Journal order.
    pub fn capability_epochs(&self) -> &[ReconstructedCapabilityEpoch] {
        &self.capability_epochs
    }

    /// Every direct authority-dependency edge indexed by retained Entries, in
    /// dependent Journal order and then canonical dependency order.
    pub fn authority_dependency_edges(&self) -> &[ReconstructedAuthorityDependency] {
        &self.authority_dependency_edges
    }
}

impl RetainedJournal {
    /// Starts a retained Journal from its exact structural GENESIS Entry.
    pub fn from_genesis(genesis: GenesisJournalEntry) -> Result<Self, RetainedJournalError> {
        let registry_id = genesis.registry_id;
        let mut journal = Self {
            registry_id,
            entries: Vec::new(),
        };
        journal.append_genesis(genesis)?;
        Ok(journal)
    }

    /// Strictly decodes exact GENESIS bytes before starting retained replay.
    pub fn from_authoritative_genesis(input: &[u8]) -> Result<Self, RetainedJournalError> {
        GenesisJournalEntry::decode_authoritative(input)
            .map_err(|_| RetainedJournalError::DecodeError)
            .and_then(Self::from_genesis)
    }

    /// Appends GENESIS only at the single initial Journal slot.
    pub fn append_genesis(
        &mut self,
        genesis: GenesisJournalEntry,
    ) -> Result<(), RetainedJournalError> {
        self.append(RetainedJournalEntry::Genesis(genesis))
    }

    /// Strictly decodes and appends a supported non-GENESIS Entry without
    /// normalizing its bytes. Event forms without a retained-runtime decoder
    /// remain unavailable until their exact decoder is present.
    ///
    /// This preserves structural/history evidence only; it is not authority or
    /// admission validation despite the Journal Entry wire-format name.
    pub fn append_strict_entry(&mut self, input: &[u8]) -> Result<(), RetainedJournalError> {
        let event_type_id = decode_journal_entry_event_type_id(input)
            .map_err(|_| RetainedJournalError::DecodeError)?;
        if event_type_id.value() == 1 {
            return GenesisJournalEntry::decode_authoritative(input)
                .map_err(|_| RetainedJournalError::DecodeError)
                .and_then(|entry| self.append(RetainedJournalEntry::Genesis(entry)));
        }
        if event_type_id.value() == 100 {
            return self.append_freeze_start(input, event_type_id);
        }
        if matches!(event_type_id.value(), 101..=103) {
            return self.append_freeze_terminal(input, event_type_id);
        }
        if event_type_id.value() == 104 {
            self.validate_freeze_commit_rejected(input, event_type_id)?;
            return Err(RetainedJournalError::UnsupportedEntry);
        }
        if event_type_id.value() == 700 {
            self.validate_eviction_started(input, event_type_id)?;
            return Err(RetainedJournalError::UnsupportedEntry);
        }
        if let Some(event_specific_keys) = event_specific_keys(event_type_id.value()) {
            let common = decode_journal_entry_with_event_specific_keys(input, event_specific_keys)
                .map_err(|_| RetainedJournalError::DecodeError)?;
            if common.event_type_id != event_type_id {
                return Err(RetainedJournalError::DecodeError);
            }
            if matches!(event_type_id.value(), 701..=703) {
                let [(21, DecodedEventSpecificField::Bytes(eviction_attempt_id)), (25, DecodedEventSpecificField::JournalReference(eviction_start_reference))] =
                    common.event_specific_fields.as_slice()
                else {
                    return Err(RetainedJournalError::DecodeError);
                };
                if common.lifecycle_object_kind != LifecycleObjectKind::ArtifactEvictionAttempt
                    || common.lifecycle_object_id != *eviction_attempt_id
                    || eviction_start_reference.event_type_id().value() != 700
                    || !common
                        .authority_dependencies
                        .elements
                        .iter()
                        .any(|reference| reference == eviction_start_reference)
                {
                    return Err(RetainedJournalError::DecodeError);
                }
                self.resolve_reference(eviction_start_reference)
                    .map_err(|_| RetainedJournalError::DecodeError)?;
                let eviction_start = self
                    .entries
                    .get(
                        usize::try_from(eviction_start_reference.entry_index().value())
                            .map_err(|_| RetainedJournalError::DecodeError)?,
                    )
                    .ok_or(RetainedJournalError::DecodeError)?;
                if eviction_start.lifecycle_object_kind()
                    != LifecycleObjectKind::ArtifactEvictionAttempt
                    || eviction_start.lifecycle_object_id() != *eviction_attempt_id
                {
                    return Err(RetainedJournalError::DecodeError);
                }
            }
            self.preflight_unsupported_common(&common)?;
            return Err(RetainedJournalError::UnsupportedEntry);
        }
        if event_type_id.value() == 200 {
            return match decode_common_journal_entry(input) {
                Ok(common) => {
                    if common.event_type_id != event_type_id
                        || common.lifecycle_object_kind != event_type_id.lifecycle_object_kind()
                        || common.lifecycle_object_id != *common.event_record_id.as_bytes()
                    {
                        return Err(RetainedJournalError::DecodeError);
                    }
                    self.preflight_unsupported_common(&common)?;
                    Err(RetainedJournalError::UnsupportedEntry)
                }
                Err(_) => {
                    let common = decode_journal_entry_with_event_specific_keys(input, &[20])
                        .map_err(|_| RetainedJournalError::DecodeError)?;
                    let [(20, DecodedEventSpecificField::JournalReference(closeout_reference))] =
                        common.event_specific_fields.as_slice()
                    else {
                        return Err(RetainedJournalError::DecodeError);
                    };
                    if common.event_type_id != event_type_id
                        || common.lifecycle_object_kind != event_type_id.lifecycle_object_kind()
                        || common.lifecycle_object_id != *common.event_record_id.as_bytes()
                        || closeout_reference.event_type_id().value() != 500
                        || !common
                            .authority_dependencies
                            .elements
                            .iter()
                            .any(|reference| reference == closeout_reference)
                    {
                        return Err(RetainedJournalError::DecodeError);
                    }
                    self.resolve_reference(closeout_reference)
                        .map_err(|_| RetainedJournalError::DecodeError)?;
                    self.preflight_unsupported_common(&common)?;
                    Err(RetainedJournalError::UnsupportedEntry)
                }
            };
        }
        let common =
            decode_common_journal_entry(input).map_err(|_| RetainedJournalError::DecodeError)?;
        if common.previous_entry_hash.is_none()
            || common.lifecycle_object_kind != common.event_type_id.lifecycle_object_kind()
        {
            return Err(RetainedJournalError::DecodeError);
        }
        let expected_object_id = match common.lifecycle_object_kind {
            LifecycleObjectKind::Registry => *common.registry_id.as_bytes(),
            LifecycleObjectKind::FreezeAttempt | LifecycleObjectKind::ArtifactEvictionAttempt => {
                return Err(RetainedJournalError::UnsupportedEntry)
            }
            _ => *common.event_record_id.as_bytes(),
        };
        if common.lifecycle_object_id != expected_object_id {
            return Err(RetainedJournalError::DecodeError);
        }

        self.append(RetainedJournalEntry::Common(CommonRetainedJournalEntry {
            registry_id: common.registry_id,
            entry_index: common.entry_index,
            previous_entry_hash: common
                .previous_entry_hash
                .ok_or(RetainedJournalError::DecodeError)?,
            event_type_id: common.event_type_id,
            event_record_id: common.event_record_id,
            storage_capability_class_id: common.storage_capability_class_id,
            environment_observation_id: common.environment_observation_id,
            lifecycle_object_kind: common.lifecycle_object_kind,
            lifecycle_object_id: common.lifecycle_object_id,
            freeze_attempt_intended_root_id: None,
            authority_dependencies: common.authority_dependencies,
            authoritative_bytes: input.to_vec(),
        }))
    }

    fn append_freeze_start(
        &mut self,
        input: &[u8],
        expected_event_type_id: EventTypeId,
    ) -> Result<(), RetainedJournalError> {
        let common = decode_journal_entry_with_event_specific_keys(input, &[16, 17])
            .map_err(|_| RetainedJournalError::DecodeError)?;
        if common.event_type_id != expected_event_type_id
            || common.lifecycle_object_kind != LifecycleObjectKind::FreezeAttempt
        {
            return Err(RetainedJournalError::DecodeError);
        }
        let [(16, DecodedEventSpecificField::Bytes(freeze_attempt_id)), (17, DecodedEventSpecificField::Bytes(intended_root_id))] =
            common.event_specific_fields.as_slice()
        else {
            return Err(RetainedJournalError::DecodeError);
        };
        let freeze_attempt_id = FreezeAttemptId::try_from(freeze_attempt_id.as_slice())
            .map_err(|_| RetainedJournalError::DecodeError)?;
        let intended_root_id = IntendedRootId::try_from(intended_root_id.as_slice())
            .map_err(|_| RetainedJournalError::DecodeError)?;
        if common.lifecycle_object_id != *freeze_attempt_id.as_bytes()
            || derive_freeze_root(common.registry_id, freeze_attempt_id)
                .intended_root_id()
                .as_bytes()
                != intended_root_id.as_bytes()
        {
            return Err(RetainedJournalError::DecodeError);
        }
        self.append(RetainedJournalEntry::Common(CommonRetainedJournalEntry {
            registry_id: common.registry_id,
            entry_index: common.entry_index,
            previous_entry_hash: common
                .previous_entry_hash
                .ok_or(RetainedJournalError::DecodeError)?,
            event_type_id: common.event_type_id,
            event_record_id: common.event_record_id,
            storage_capability_class_id: common.storage_capability_class_id,
            environment_observation_id: common.environment_observation_id,
            lifecycle_object_kind: common.lifecycle_object_kind,
            lifecycle_object_id: common.lifecycle_object_id,
            freeze_attempt_intended_root_id: Some(intended_root_id),
            authority_dependencies: common.authority_dependencies,
            authoritative_bytes: input.to_vec(),
        }))
    }

    fn append_freeze_terminal(
        &mut self,
        input: &[u8],
        expected_event_type_id: EventTypeId,
    ) -> Result<(), RetainedJournalError> {
        let common = decode_journal_entry_with_event_specific_keys(input, &[16, 18])
            .map_err(|_| RetainedJournalError::DecodeError)?;
        if common.event_type_id != expected_event_type_id
            || common.lifecycle_object_kind != LifecycleObjectKind::FreezeAttempt
        {
            return Err(RetainedJournalError::DecodeError);
        }
        let [(16, DecodedEventSpecificField::Bytes(freeze_attempt_id)), (18, DecodedEventSpecificField::JournalReference(freeze_start_reference))] =
            common.event_specific_fields.as_slice()
        else {
            return Err(RetainedJournalError::DecodeError);
        };
        if common.lifecycle_object_id != *freeze_attempt_id
            || !common
                .authority_dependencies
                .elements
                .iter()
                .any(|reference| reference == freeze_start_reference)
            || freeze_start_reference.event_type_id().value() != 100
        {
            return Err(RetainedJournalError::DecodeError);
        }
        self.resolve_reference(freeze_start_reference)?;
        let freeze_start = self
            .entries
            .get(
                usize::try_from(freeze_start_reference.entry_index().value())
                    .map_err(|_| RetainedJournalError::MissingReference)?,
            )
            .ok_or(RetainedJournalError::MissingReference)?;
        if freeze_start.lifecycle_object_kind() != LifecycleObjectKind::FreezeAttempt
            || freeze_start.lifecycle_object_id() != *freeze_attempt_id
        {
            return Err(RetainedJournalError::ReferenceMismatch);
        }
        self.append(RetainedJournalEntry::Common(CommonRetainedJournalEntry {
            registry_id: common.registry_id,
            entry_index: common.entry_index,
            previous_entry_hash: common
                .previous_entry_hash
                .ok_or(RetainedJournalError::DecodeError)?,
            event_type_id: common.event_type_id,
            event_record_id: common.event_record_id,
            storage_capability_class_id: common.storage_capability_class_id,
            environment_observation_id: common.environment_observation_id,
            lifecycle_object_kind: common.lifecycle_object_kind,
            lifecycle_object_id: common.lifecycle_object_id,
            freeze_attempt_intended_root_id: None,
            authority_dependencies: common.authority_dependencies,
            authoritative_bytes: input.to_vec(),
        }))
    }

    fn validate_freeze_commit_rejected(
        &self,
        input: &[u8],
        expected_event_type_id: EventTypeId,
    ) -> Result<(), RetainedJournalError> {
        let common = decode_journal_entry_with_event_specific_keys(input, &[16, 18, 19])
            .map_err(|_| RetainedJournalError::DecodeError)?;
        if common.event_type_id != expected_event_type_id
            || common.lifecycle_object_kind != LifecycleObjectKind::FreezeAttempt
        {
            return Err(RetainedJournalError::DecodeError);
        }
        let [(16, DecodedEventSpecificField::Bytes(freeze_attempt_id)), (18, DecodedEventSpecificField::JournalReference(freeze_start_reference)), (19, DecodedEventSpecificField::JournalReference(conflicting_terminal_reference))] =
            common.event_specific_fields.as_slice()
        else {
            return Err(RetainedJournalError::DecodeError);
        };
        if common.lifecycle_object_id != *freeze_attempt_id
            || !common
                .authority_dependencies
                .elements
                .iter()
                .any(|reference| reference == freeze_start_reference)
            || !common
                .authority_dependencies
                .elements
                .iter()
                .any(|reference| reference == conflicting_terminal_reference)
        {
            return Err(RetainedJournalError::DecodeError);
        }
        if freeze_start_reference.event_type_id().value() != 100
            || !matches!(
                conflicting_terminal_reference.event_type_id().value(),
                101..=103
            )
        {
            return Err(RetainedJournalError::DecodeError);
        }
        self.resolve_reference(freeze_start_reference)
            .map_err(|_| RetainedJournalError::DecodeError)?;
        self.resolve_reference(conflicting_terminal_reference)
            .map_err(|_| RetainedJournalError::DecodeError)?;
        let freeze_start = self
            .entries
            .get(
                usize::try_from(freeze_start_reference.entry_index().value())
                    .map_err(|_| RetainedJournalError::DecodeError)?,
            )
            .ok_or(RetainedJournalError::DecodeError)?;
        let conflicting_terminal = self
            .entries
            .get(
                usize::try_from(conflicting_terminal_reference.entry_index().value())
                    .map_err(|_| RetainedJournalError::DecodeError)?,
            )
            .ok_or(RetainedJournalError::DecodeError)?;
        if freeze_start.lifecycle_object_kind() != LifecycleObjectKind::FreezeAttempt
            || freeze_start.lifecycle_object_id() != *freeze_attempt_id
            || conflicting_terminal.lifecycle_object_kind() != LifecycleObjectKind::FreezeAttempt
            || conflicting_terminal.lifecycle_object_id() != *freeze_attempt_id
        {
            return Err(RetainedJournalError::DecodeError);
        }
        self.preflight_unsupported_common(&common)?;
        Ok(())
    }

    fn validate_eviction_started(
        &self,
        input: &[u8],
        expected_event_type_id: EventTypeId,
    ) -> Result<(), RetainedJournalError> {
        let common = decode_journal_entry_with_event_specific_keys(input, &[21, 22, 23, 24])
            .map_err(|_| RetainedJournalError::DecodeError)?;
        if common.event_type_id != expected_event_type_id
            || common.lifecycle_object_kind != LifecycleObjectKind::ArtifactEvictionAttempt
        {
            return Err(RetainedJournalError::DecodeError);
        }
        let [(21, DecodedEventSpecificField::Bytes(eviction_attempt_id)), (22, DecodedEventSpecificField::JournalReference(freeze_authority_reference)), (23, DecodedEventSpecificField::Bytes(pre_eviction_manifest_id)), (24, DecodedEventSpecificField::Bytes(eviction_scope_identity))] =
            common.event_specific_fields.as_slice()
        else {
            return Err(RetainedJournalError::DecodeError);
        };
        let pre_eviction_manifest_id = RecordId::try_from(pre_eviction_manifest_id.as_slice())
            .map_err(|_| RetainedJournalError::DecodeError)?;
        let eviction_scope_identity = RecordId::try_from(eviction_scope_identity.as_slice())
            .map_err(|_| RetainedJournalError::DecodeError)?;
        if common.lifecycle_object_id != *eviction_attempt_id
            || !common
                .authority_dependencies
                .elements
                .iter()
                .any(|reference| reference == freeze_authority_reference)
            || !common
                .identity_dependencies
                .elements
                .contains(&IdentityDependency::record_id(pre_eviction_manifest_id))
            || !common
                .identity_dependencies
                .elements
                .contains(&IdentityDependency::record_id(eviction_scope_identity))
        {
            return Err(RetainedJournalError::DecodeError);
        }
        self.resolve_reference(freeze_authority_reference)
            .map_err(|_| RetainedJournalError::DecodeError)?;
        let freeze_authority = self
            .entries
            .get(
                usize::try_from(freeze_authority_reference.entry_index().value())
                    .map_err(|_| RetainedJournalError::DecodeError)?,
            )
            .ok_or(RetainedJournalError::DecodeError)?;
        if freeze_authority.event_type_id().value() != 101
            || freeze_authority.lifecycle_object_kind() != LifecycleObjectKind::FreezeAttempt
        {
            return Err(RetainedJournalError::DecodeError);
        }
        self.preflight_unsupported_common(&common)?;
        Ok(())
    }

    /// Preflights the retained-history conditions shared by an event form this
    /// runtime recognizes structurally but deliberately does not retain.
    ///
    /// It does not decide authority, admission, or Record-payload semantics.
    fn preflight_unsupported_common(
        &self,
        common: &DecodedCommonJournalEntry,
    ) -> Result<(), RetainedJournalError> {
        if common.registry_id != self.registry_id {
            return Err(RetainedJournalError::RegistryMismatch);
        }
        let expected_index = u64::try_from(self.entries.len())
            .ok()
            .and_then(|value| JournalEntryIndex::try_from(value).ok())
            .ok_or(RetainedJournalError::UnexpectedEntryIndex)?;
        if common.entry_index != expected_index {
            return Err(RetainedJournalError::UnexpectedEntryIndex);
        }
        let expected_previous = self.entries.last().map(RetainedJournalEntry::entry_hash);
        if common.previous_entry_hash != expected_previous {
            return Err(RetainedJournalError::PreviousHashMismatch);
        }
        for reference in &common.authority_dependencies.elements {
            self.resolve_reference(reference)?;
        }
        let reconstructed = self
            .reconstruct_state()
            .map_err(|_| RetainedJournalError::LifecycleTransition)?;
        let before = reconstructed
            .state_for(common.lifecycle_object_kind, &common.lifecycle_object_id)
            .unwrap_or_else(|| absent_state_for_kind(common.lifecycle_object_kind));
        let after = common
            .event_type_id
            .resulting_state(before)
            .map_err(|_| RetainedJournalError::LifecycleTransition)?;
        validate_state_only_legal_transition(common.event_type_id, before, after)
            .map_err(|_| RetainedJournalError::LifecycleTransition)
    }

    /// Resolves every field of a JournalReference against retained Entry bytes.
    ///
    /// A successful result does not establish that the referenced event carried
    /// authority; it only establishes retained-history identity agreement and
    /// returns the exact facts available for later contextual validation.
    pub fn resolve_reference(
        &self,
        reference: &JournalReference,
    ) -> Result<ResolvedJournalReference, RetainedJournalError> {
        if reference.registry_id() != self.registry_id {
            return Err(RetainedJournalError::ReferenceMismatch);
        }
        let entry = self
            .entries
            .get(
                usize::try_from(reference.entry_index().value())
                    .map_err(|_| RetainedJournalError::MissingReference)?,
            )
            .ok_or(RetainedJournalError::MissingReference)?;
        if entry.entry_index() != reference.entry_index()
            || entry.entry_hash() != reference.entry_hash()
            || entry.event_type_id() != reference.event_type_id()
            || entry.event_record_id() != reference.event_record_id()
        {
            return Err(RetainedJournalError::ReferenceMismatch);
        }
        Ok(ResolvedJournalReference {
            registry_id: entry.registry_id(),
            entry_index: entry.entry_index(),
            entry_hash: entry.entry_hash(),
            previous_entry_hash: entry.previous_entry_hash(),
            event_type_id: entry.event_type_id(),
            event_record_id: entry.event_record_id(),
            storage_capability_class_id: entry.storage_capability_class_id(),
            environment_observation_id: entry.environment_observation_id(),
            lifecycle_object_kind: entry.lifecycle_object_kind(),
            lifecycle_object_id: entry.lifecycle_object_id(),
            freeze_attempt_intended_root_id: entry.freeze_attempt_intended_root_id(),
        })
    }

    /// Replays every supported retained Entry from GENESIS.
    pub fn reconstruct_state(&self) -> Result<ReconstructedJournalState, RetainedJournalError> {
        let mut registry_state = RegistryLifecycleState::Absent;
        let mut states = Vec::new();
        let mut freeze_attempts = Vec::new();
        let mut capability_epochs = Vec::with_capacity(self.entries.len());
        let mut authority_dependency_edges = Vec::new();
        for entry in &self.entries {
            capability_epochs.push(ReconstructedCapabilityEpoch {
                entry_index: entry.entry_index(),
                storage_capability_class_id: entry.storage_capability_class_id(),
                environment_observation_id: entry.environment_observation_id(),
            });
            let dependent_entry = JournalReference::new(
                entry.registry_id(),
                entry.entry_index(),
                entry.entry_hash(),
                entry.event_type_id(),
                entry.event_record_id(),
            );
            for dependency in entry.authority_dependencies() {
                authority_dependency_edges.push(ReconstructedAuthorityDependency {
                    dependent_entry: dependent_entry.clone(),
                    dependency: dependency.clone(),
                });
            }
            let kind = entry.lifecycle_object_kind();
            let object_id = entry.lifecycle_object_id();
            let state_index = states.iter().position(|(stored_kind, stored_id, _)| {
                *stored_kind == kind && *stored_id == object_id
            });
            let before = state_index
                .map(|index| states[index].2)
                .unwrap_or_else(|| absent_state_for_kind(kind));
            let after = entry
                .event_type_id()
                .resulting_state(before)
                .map_err(|_| RetainedJournalError::LifecycleTransition)?;
            validate_state_only_legal_transition(entry.event_type_id(), before, after)
                .map_err(|_| RetainedJournalError::LifecycleTransition)?;
            if let Some(index) = state_index {
                states[index].2 = after;
            } else {
                states.push((kind, object_id, after));
            }
            if let LifecycleObjectState::Registry(state) = after {
                registry_state = state;
            }
            if let LifecycleObjectState::FreezeAttempt(state) = after {
                let freeze_attempt_id = FreezeAttemptId::try_from(object_id.as_slice())
                    .map_err(|_| RetainedJournalError::DecodeError)?;
                match entry.event_type_id().value() {
                    100 => {
                        let intended_root_id = entry
                            .freeze_attempt_intended_root_id()
                            .ok_or(RetainedJournalError::DecodeError)?;
                        freeze_attempts.push(ReconstructedFreezeAttempt {
                            freeze_attempt_id,
                            intended_root_id,
                            state,
                        });
                    }
                    101..=103 => {
                        let attempt = freeze_attempts
                            .iter_mut()
                            .find(|attempt| attempt.freeze_attempt_id == freeze_attempt_id)
                            .ok_or(RetainedJournalError::LifecycleTransition)?;
                        attempt.state = state;
                    }
                    _ => {}
                }
            }
        }

        let head = self
            .entries
            .last()
            .ok_or(RetainedJournalError::UnexpectedEntryIndex)?;
        Ok(ReconstructedJournalState {
            registry_state,
            entry_count: self.entries.len(),
            journal_head_index: head.entry_index(),
            journal_head_hash: head.entry_hash(),
            states,
            freeze_attempts,
            capability_epochs,
            authority_dependency_edges,
        })
    }

    fn append(&mut self, entry: RetainedJournalEntry) -> Result<(), RetainedJournalError> {
        if entry.registry_id() != self.registry_id {
            return Err(RetainedJournalError::RegistryMismatch);
        }
        let expected_index = u64::try_from(self.entries.len())
            .ok()
            .and_then(|value| JournalEntryIndex::try_from(value).ok())
            .ok_or(RetainedJournalError::UnexpectedEntryIndex)?;
        if entry.entry_index() != expected_index {
            return Err(RetainedJournalError::UnexpectedEntryIndex);
        }
        let expected_previous = self.entries.last().map(RetainedJournalEntry::entry_hash);
        if entry.previous_entry_hash() != expected_previous {
            return Err(RetainedJournalError::PreviousHashMismatch);
        }
        for reference in entry.authority_dependencies() {
            self.resolve_reference(reference)?;
        }
        if !self.entries.is_empty() {
            let reconstructed = self
                .reconstruct_state()
                .map_err(|_| RetainedJournalError::LifecycleTransition)?;
            let before = reconstructed
                .state_for(entry.lifecycle_object_kind(), &entry.lifecycle_object_id())
                .unwrap_or_else(|| absent_state_for_kind(entry.lifecycle_object_kind()));
            let after = entry
                .event_type_id()
                .resulting_state(before)
                .map_err(|_| RetainedJournalError::LifecycleTransition)?;
            validate_state_only_legal_transition(entry.event_type_id(), before, after)
                .map_err(|_| RetainedJournalError::LifecycleTransition)?;
        }
        self.entries.push(entry);
        Ok(())
    }
}

fn absent_state_for_kind(kind: LifecycleObjectKind) -> LifecycleObjectState {
    match kind {
        LifecycleObjectKind::Registry => {
            LifecycleObjectState::Registry(RegistryLifecycleState::Absent)
        }
        LifecycleObjectKind::FreezeAttempt => {
            LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Absent)
        }
        LifecycleObjectKind::ReviewAdmissionAttempt => {
            LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Absent)
        }
        LifecycleObjectKind::CloseoutAttempt => {
            LifecycleObjectState::CloseoutAttempt(CloseoutAttemptState::Absent)
        }
        LifecycleObjectKind::ArtifactEvictionAttempt => {
            LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Absent)
        }
        _ => LifecycleObjectState::OneShot(OneShotRecordedState::Absent),
    }
}

/// A portable historical commitment to one Registry Journal head.
///
/// This type emits Identity Format v0.3 §§93–96 canonical Anchor bytes. It
/// does not assert that an external holder is trustworthy or that the Anchor
/// is itself an authoritative Registry Record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalAnchor {
    registry_id: RegistryId,
    journal_head_index: JournalEntryIndex,
    journal_head_hash: JournalEntryHash,
    journal_format_version: u64,
}

/// A Journal Anchor format version outside the general EvidenceRegistry UInt range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JournalAnchorFormatVersionError {
    value: u64,
}

impl JournalAnchor {
    /// Creates a Journal Anchor when its format version is a representable
    /// EvidenceRegistry nonnegative counter.
    pub fn new(
        registry_id: RegistryId,
        journal_head_index: JournalEntryIndex,
        journal_head_hash: JournalEntryHash,
        journal_format_version: u64,
    ) -> Result<Self, JournalAnchorFormatVersionError> {
        if journal_format_version > ER_UINT_MAX {
            return Err(JournalAnchorFormatVersionError {
                value: journal_format_version,
            });
        }

        Ok(Self {
            registry_id,
            journal_head_index,
            journal_head_hash,
            journal_format_version,
        })
    }

    /// Emits `deterministic_cbor([domain, 1, anchor_body])` with keys 0..3.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        debug_assert_eq!(JOURNAL_ANCHOR_DOMAIN.len(), 33);

        let mut bytes = Vec::with_capacity(113);
        bytes.extend_from_slice(&[0x83, 0x78, 0x21]);
        bytes.extend_from_slice(JOURNAL_ANCHOR_DOMAIN);
        bytes.extend_from_slice(&[0x01, 0xa4, 0x00]);
        encode_bstr_32(&mut bytes, self.registry_id.as_bytes());
        bytes.push(0x01);
        encode_uint(&mut bytes, self.journal_head_index.value());
        bytes.push(0x02);
        encode_bstr_32(&mut bytes, self.journal_head_hash.as_bytes());
        bytes.push(0x03);
        encode_uint(&mut bytes, self.journal_format_version);
        bytes
    }

    /// Returns `SHA256(authoritative_cbor())` as the Anchor ID.
    pub fn anchor_id(&self) -> JournalAnchorId {
        let digest: [u8; ID_LENGTH] = Sha256::digest(self.authoritative_cbor())
            .as_slice()
            .try_into()
            .expect("SHA-256 always returns exactly 32 bytes");
        JournalAnchorId(digest)
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

fn is_canonical_text_length(length: u64) -> bool {
    length <= ER_UINT_MAX
}

fn encode_text(output: &mut Vec<u8>, value: &str) {
    let length = u64::try_from(value.len()).expect("platform usize fits into u64");
    debug_assert!(is_canonical_text_length(length));
    match length {
        0..=23 => output.push(0x60 | length as u8),
        24..=0xff => output.extend_from_slice(&[0x78, length as u8]),
        0x100..=0xffff => output.extend_from_slice(&[0x79, (length >> 8) as u8, length as u8]),
        0x1_0000..=0xffff_ffff => output.extend_from_slice(&[
            0x7a,
            (length >> 24) as u8,
            (length >> 16) as u8,
            (length >> 8) as u8,
            length as u8,
        ]),
        _ => output.extend_from_slice(&[
            0x7b,
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
    output.extend_from_slice(value.as_bytes());
}

struct DecodedCommonJournalEntry {
    registry_id: RegistryId,
    entry_index: JournalEntryIndex,
    previous_entry_hash: Option<JournalEntryHash>,
    event_type_id: EventTypeId,
    event_record_id: EventRecordId,
    identity_dependencies: IdentityDependencyCollection,
    authority_dependencies: AuthorityDependencyCollection,
    lifecycle_object_kind: LifecycleObjectKind,
    lifecycle_object_id: [u8; ID_LENGTH],
    storage_capability_class_id: RecordId,
    environment_observation_id: RecordId,
    event_specific_fields: Vec<(u64, DecodedEventSpecificField)>,
}

enum DecodedEventSpecificField {
    Bytes([u8; ID_LENGTH]),
    JournalReference(JournalReference),
}

fn decode_common_journal_entry(
    input: &[u8],
) -> Result<DecodedCommonJournalEntry, JournalEntryDecodeError> {
    decode_journal_entry_with_event_specific_keys(input, &[])
}

fn decode_journal_entry_event_type_id(
    input: &[u8],
) -> Result<EventTypeId, JournalEntryDecodeError> {
    let mut cursor = CborCursor::new(input);
    cursor.array_exact(2)?;
    cursor.text_exact(JOURNAL_ENTRY_DOMAIN)?;
    cursor.map()?;
    cursor.key(0)?;
    if cursor.uint()? != 1 {
        return Err(JournalEntryDecodeError);
    }
    cursor.key(1)?;
    cursor.bstr_32()?;
    cursor.key(2)?;
    JournalEntryIndex::try_from(cursor.uint()?).map_err(|_| JournalEntryDecodeError)?;
    cursor.key(3)?;
    cursor.null_or_bstr_32()?;
    cursor.key(4)?;
    EventTypeId::try_from(cursor.uint()?).map_err(|_| JournalEntryDecodeError)
}

fn decode_journal_entry_with_event_specific_keys(
    input: &[u8],
    event_specific_keys: &[u64],
) -> Result<DecodedCommonJournalEntry, JournalEntryDecodeError> {
    let mut cursor = CborCursor::new(input);
    cursor.array_exact(2)?;
    cursor.text_exact(JOURNAL_ENTRY_DOMAIN)?;
    cursor.map_exact(12 + event_specific_keys.len())?;
    cursor.key(0)?;
    if cursor.uint()? != 1 {
        return Err(JournalEntryDecodeError);
    }
    cursor.key(1)?;
    let registry_id =
        RegistryId::try_from(cursor.bstr_32()?.as_slice()).map_err(|_| JournalEntryDecodeError)?;
    cursor.key(2)?;
    let entry_index =
        JournalEntryIndex::try_from(cursor.uint()?).map_err(|_| JournalEntryDecodeError)?;
    cursor.key(3)?;
    let previous_entry_hash = cursor
        .null_or_bstr_32()?
        .map(|bytes| {
            JournalEntryHash::try_from(bytes.as_slice()).map_err(|_| JournalEntryDecodeError)
        })
        .transpose()?;
    cursor.key(4)?;
    let event_type_id =
        EventTypeId::try_from(cursor.uint()?).map_err(|_| JournalEntryDecodeError)?;
    cursor.key(5)?;
    let event_record_id = EventRecordId::try_from(cursor.bstr_32()?.as_slice())
        .map_err(|_| JournalEntryDecodeError)?;
    cursor.key(6)?;
    let identity_dependencies = decode_identity_dependencies(&mut cursor)?;
    cursor.key(7)?;
    let authority_dependencies =
        decode_authority_dependencies(&mut cursor, registry_id, entry_index)?;
    cursor.key(8)?;
    let lifecycle_object_kind =
        LifecycleObjectKind::try_from(cursor.uint()?).map_err(|_| JournalEntryDecodeError)?;
    cursor.key(9)?;
    let lifecycle_object_id = cursor.bstr_32()?;
    cursor.key(10)?;
    let storage_capability_class_id =
        RecordId::try_from(cursor.bstr_32()?.as_slice()).map_err(|_| JournalEntryDecodeError)?;
    cursor.key(11)?;
    let environment_observation_id =
        RecordId::try_from(cursor.bstr_32()?.as_slice()).map_err(|_| JournalEntryDecodeError)?;
    let mut event_specific_fields = Vec::with_capacity(event_specific_keys.len());
    for key in event_specific_keys {
        cursor.key(*key)?;
        event_specific_fields.push((*key, decode_event_specific_value(&mut cursor, *key)?));
    }
    if !cursor.finished() {
        return Err(JournalEntryDecodeError);
    }
    Ok(DecodedCommonJournalEntry {
        registry_id,
        entry_index,
        previous_entry_hash,
        event_type_id,
        event_record_id,
        identity_dependencies,
        authority_dependencies,
        lifecycle_object_kind,
        lifecycle_object_id,
        storage_capability_class_id,
        environment_observation_id,
        event_specific_fields,
    })
}

fn event_specific_keys(event_type_id: u16) -> Option<&'static [u64]> {
    match event_type_id {
        100 => Some(&[16, 17]),
        101..=103 => Some(&[16, 18]),
        104 => Some(&[16, 18, 19]),
        700 => Some(&[21, 22, 23, 24]),
        701..=703 => Some(&[21, 25]),
        _ => None,
    }
}

fn decode_event_specific_value(
    cursor: &mut CborCursor<'_>,
    key: u64,
) -> Result<DecodedEventSpecificField, JournalEntryDecodeError> {
    match key {
        16 | 17 | 21 | 23 | 24 => cursor.bstr_32().map(DecodedEventSpecificField::Bytes),
        18 | 19 | 20 | 22 | 25 => {
            decode_journal_reference(cursor).map(DecodedEventSpecificField::JournalReference)
        }
        _ => Err(JournalEntryDecodeError),
    }
}

fn decode_identity_dependencies(
    cursor: &mut CborCursor<'_>,
) -> Result<IdentityDependencyCollection, JournalEntryDecodeError> {
    let length = cursor.array()?;
    if length > cursor.remaining() / 36 {
        return Err(JournalEntryDecodeError);
    }
    let mut elements = Vec::with_capacity(length);
    for _ in 0..length {
        cursor.array_exact(2)?;
        let kind = IdentityDependencyKind::try_from(cursor.uint()?)
            .map_err(|_| JournalEntryDecodeError)?;
        let bytes = cursor.bstr_32()?;
        elements.push(match kind {
            IdentityDependencyKind::RecordId => IdentityDependency::record_id(
                RecordId::try_from(bytes.as_slice()).map_err(|_| JournalEntryDecodeError)?,
            ),
            IdentityDependencyKind::JournalAnchorId => IdentityDependency::journal_anchor_id(
                JournalAnchorId::try_from(bytes.as_slice()).map_err(|_| JournalEntryDecodeError)?,
            ),
        });
    }
    IdentityDependencyCollection::from_authoritative_ordered_elements(elements)
        .map_err(|_| JournalEntryDecodeError)
}

fn decode_authority_dependencies(
    cursor: &mut CborCursor<'_>,
    registry_id: RegistryId,
    entry_index: JournalEntryIndex,
) -> Result<AuthorityDependencyCollection, JournalEntryDecodeError> {
    let length = cursor.array()?;
    // A canonical JournalReference is at least 105 bytes: array(5), three
    // bstr(32) fields, and one-byte encodings for index and event type.
    // This only bounds allocation; the parser still validates every element.
    if length > cursor.remaining() / 105 {
        return Err(JournalEntryDecodeError);
    }
    let mut elements = Vec::with_capacity(length);
    for _ in 0..length {
        elements.push(decode_journal_reference(cursor)?);
    }
    AuthorityDependencyCollection::from_authoritative_ordered_elements(
        AuthorityDependencyContext::new(registry_id, entry_index),
        elements,
    )
    .map_err(|_| JournalEntryDecodeError)
}

fn decode_journal_reference(
    cursor: &mut CborCursor<'_>,
) -> Result<JournalReference, JournalEntryDecodeError> {
    cursor.array_exact(5)?;
    let reference_registry =
        RegistryId::try_from(cursor.bstr_32()?.as_slice()).map_err(|_| JournalEntryDecodeError)?;
    let reference_index =
        JournalEntryIndex::try_from(cursor.uint()?).map_err(|_| JournalEntryDecodeError)?;
    let reference_hash = JournalEntryHash::try_from(cursor.bstr_32()?.as_slice())
        .map_err(|_| JournalEntryDecodeError)?;
    let reference_event =
        EventTypeId::try_from(cursor.uint()?).map_err(|_| JournalEntryDecodeError)?;
    let reference_record = EventRecordId::try_from(cursor.bstr_32()?.as_slice())
        .map_err(|_| JournalEntryDecodeError)?;
    Ok(JournalReference::new(
        reference_registry,
        reference_index,
        reference_hash,
        reference_event,
        reference_record,
    ))
}

struct CborCursor<'a> {
    input: &'a [u8],
    offset: usize,
}

impl<'a> CborCursor<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.input.len().saturating_sub(self.offset)
    }

    fn finished(&self) -> bool {
        self.offset == self.input.len()
    }

    fn byte(&mut self) -> Result<u8, JournalEntryDecodeError> {
        let result = *self.input.get(self.offset).ok_or(JournalEntryDecodeError)?;
        self.offset += 1;
        Ok(result)
    }

    fn argument(&mut self, additional: u8) -> Result<u64, JournalEntryDecodeError> {
        let (width, minimum) = match additional {
            value @ 0..=23 => return Ok(u64::from(value)),
            24 => (1, 24),
            25 => (2, 0x100),
            26 => (4, 0x1_0000),
            27 => (8, 0x1_0000_0000),
            _ => return Err(JournalEntryDecodeError),
        };
        let mut value = 0_u64;
        for _ in 0..width {
            value = (value << 8) | u64::from(self.byte()?);
        }
        if value < minimum || value > ER_UINT_MAX {
            return Err(JournalEntryDecodeError);
        }
        Ok(value)
    }

    fn initial(&mut self, major: u8) -> Result<u64, JournalEntryDecodeError> {
        let initial = self.byte()?;
        if initial >> 5 != major {
            return Err(JournalEntryDecodeError);
        }
        self.argument(initial & 0x1f)
    }

    fn uint(&mut self) -> Result<u64, JournalEntryDecodeError> {
        self.initial(0)
    }

    fn array(&mut self) -> Result<usize, JournalEntryDecodeError> {
        usize::try_from(self.initial(4)?).map_err(|_| JournalEntryDecodeError)
    }

    fn array_exact(&mut self, expected: usize) -> Result<(), JournalEntryDecodeError> {
        if self.array()? != expected {
            return Err(JournalEntryDecodeError);
        }
        Ok(())
    }

    fn map_exact(&mut self, expected: usize) -> Result<(), JournalEntryDecodeError> {
        let actual = self.map()?;
        if actual != expected {
            return Err(JournalEntryDecodeError);
        }
        Ok(())
    }

    fn map(&mut self) -> Result<usize, JournalEntryDecodeError> {
        usize::try_from(self.initial(5)?).map_err(|_| JournalEntryDecodeError)
    }

    fn text_exact(&mut self, expected: &[u8]) -> Result<(), JournalEntryDecodeError> {
        let length = usize::try_from(self.initial(3)?).map_err(|_| JournalEntryDecodeError)?;
        let end = self
            .offset
            .checked_add(length)
            .ok_or(JournalEntryDecodeError)?;
        if self.input.get(self.offset..end) != Some(expected) {
            return Err(JournalEntryDecodeError);
        }
        self.offset = end;
        Ok(())
    }

    fn text(&mut self) -> Result<String, JournalEntryDecodeError> {
        let length = usize::try_from(self.initial(3)?).map_err(|_| JournalEntryDecodeError)?;
        let end = self
            .offset
            .checked_add(length)
            .ok_or(JournalEntryDecodeError)?;
        let text = std::str::from_utf8(
            self.input
                .get(self.offset..end)
                .ok_or(JournalEntryDecodeError)?,
        )
        .map_err(|_| JournalEntryDecodeError)?
        .to_owned();
        self.offset = end;
        Ok(text)
    }

    fn bstr_32(&mut self) -> Result<[u8; ID_LENGTH], JournalEntryDecodeError> {
        if self.initial(2)? != ID_LENGTH as u64 {
            return Err(JournalEntryDecodeError);
        }
        let end = self
            .offset
            .checked_add(ID_LENGTH)
            .ok_or(JournalEntryDecodeError)?;
        let bytes = self
            .input
            .get(self.offset..end)
            .ok_or(JournalEntryDecodeError)?
            .try_into()
            .map_err(|_| JournalEntryDecodeError)?;
        self.offset = end;
        Ok(bytes)
    }

    fn null_or_bstr_32(&mut self) -> Result<Option<[u8; ID_LENGTH]>, JournalEntryDecodeError> {
        if self.input.get(self.offset) == Some(&0xf6) {
            self.offset += 1;
            Ok(None)
        } else {
            self.bstr_32().map(Some)
        }
    }

    fn bool(&mut self) -> Result<bool, JournalEntryDecodeError> {
        match self.byte()? {
            0xf4 => Ok(false),
            0xf5 => Ok(true),
            _ => Err(JournalEntryDecodeError),
        }
    }

    fn key(&mut self, expected: u64) -> Result<(), JournalEntryDecodeError> {
        if self.uint()? != expected {
            return Err(JournalEntryDecodeError);
        }
        Ok(())
    }

    /// Maximum container nesting accepted while structurally skipping an opaque CBOR value.
    ///
    /// The root opaque value is at depth zero, and each array/map child increments
    /// depth by one. This bounds the recursive parser independently of input size.
    const MAX_SKIP_VALUE_CONTAINER_NESTING: usize = 64;

    fn skip_value(&mut self) -> Result<(), JournalEntryDecodeError> {
        self.skip_value_at_depth(0)
    }

    fn skip_value_at_depth(&mut self, depth: usize) -> Result<(), JournalEntryDecodeError> {
        let initial = self.byte()?;
        let major = initial >> 5;
        let argument = self.argument(initial & 0x1f)?;
        match major {
            0 => Ok(()),
            2 | 3 => {
                let length = usize::try_from(argument).map_err(|_| JournalEntryDecodeError)?;
                let end = self
                    .offset
                    .checked_add(length)
                    .ok_or(JournalEntryDecodeError)?;
                let bytes = self
                    .input
                    .get(self.offset..end)
                    .ok_or(JournalEntryDecodeError)?;
                if major == 3 {
                    std::str::from_utf8(bytes).map_err(|_| JournalEntryDecodeError)?;
                }
                self.offset = end;
                Ok(())
            }
            4 => {
                if depth >= Self::MAX_SKIP_VALUE_CONTAINER_NESTING {
                    return Err(JournalEntryDecodeError);
                }
                for _ in 0..argument {
                    self.skip_value_at_depth(depth + 1)?;
                }
                Ok(())
            }
            5 => {
                if depth >= Self::MAX_SKIP_VALUE_CONTAINER_NESTING {
                    return Err(JournalEntryDecodeError);
                }
                let mut previous_key: Option<(usize, usize)> = None;
                for _ in 0..argument {
                    let key_start = self.offset;
                    self.skip_value_at_depth(depth + 1)?;
                    let key_end = self.offset;
                    if let Some((previous_start, previous_end)) = previous_key {
                        let previous = &self.input[previous_start..previous_end];
                        let current = &self.input[key_start..key_end];
                        if (previous.len(), previous) >= (current.len(), current) {
                            return Err(JournalEntryDecodeError);
                        }
                    }
                    previous_key = Some((key_start, key_end));
                    self.skip_value_at_depth(depth + 1)?;
                }
                Ok(())
            }
            7 if matches!(argument, 20 | 21) => Ok(()),
            _ => Err(JournalEntryDecodeError),
        }
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

const RECORD_DOMAIN: &[u8] = b"EvidenceRegistry.Record.v1";

/// Strictly validated Record framing and self-hash identity.
///
/// This validates the canonical v0.3 Record envelope, the duplicated type and
/// schema values, and the exact-byte Record identity. It deliberately does not
/// validate a type-local body schema, resolve a Record, or establish authority
/// or admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrictRecordFrame {
    record_type_id: RecordTypeId,
    schema_version: u64,
    record_id: RecordId,
    authoritative_bytes: Vec<u8>,
}

impl StrictRecordFrame {
    /// Strictly decodes canonical Record framing without normalizing its bytes.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        let record_type_id = RecordTypeId::try_from(cursor.uint().map_err(|_| RecordDecodeError)?)
            .map_err(|_| RecordDecodeError)?;
        let schema_version = cursor.uint().map_err(|_| RecordDecodeError)?;
        if schema_version != 1 {
            return Err(RecordDecodeError);
        }

        let field_count = cursor.map().map_err(|_| RecordDecodeError)?;
        if field_count < 2 {
            return Err(RecordDecodeError);
        }
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != schema_version {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if RecordTypeId::try_from(cursor.uint().map_err(|_| RecordDecodeError)?)
            .map_err(|_| RecordDecodeError)?
            != record_type_id
        {
            return Err(RecordDecodeError);
        }
        let mut previous_key = 1;
        for _ in 2..field_count {
            let key = cursor.uint().map_err(|_| RecordDecodeError)?;
            if key <= previous_key {
                return Err(RecordDecodeError);
            }
            previous_key = key;
            cursor.skip_value().map_err(|_| RecordDecodeError)?;
        }
        if !cursor.finished() {
            return Err(RecordDecodeError);
        }
        let digest: [u8; ID_LENGTH] = Sha256::digest(input)
            .as_slice()
            .try_into()
            .expect("SHA-256 always returns exactly 32 bytes");
        Ok(Self {
            record_type_id,
            schema_version,
            record_id: RecordId(digest),
            authoritative_bytes: input.to_vec(),
        })
    }

    /// The exact type declared by the validated Record frame.
    pub fn record_type_id(&self) -> RecordTypeId {
        self.record_type_id
    }

    /// The exact schema version declared by the validated Record frame.
    pub fn schema_version(&self) -> u64 {
        self.schema_version
    }

    /// The immutable SHA-256 identity of the validated exact Record bytes.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// The original canonical Record bytes; no normalization is performed.
    pub fn authoritative_cbor(&self) -> &[u8] {
        &self.authoritative_bytes
    }
}

/// The exact structural facts that bind one strict Record frame to one retained
/// Journal event. This is not type-local Record validation, authority, admission,
/// Policy satisfaction, custody, durability, or lifecycle truth.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventRecordStructuralBinding {
    event_reference: JournalReference,
    record_id: RecordId,
    record_type_id: RecordTypeId,
}

impl EventRecordStructuralBinding {
    /// The exact retained Journal reference bound by this structural check.
    pub fn event_reference(&self) -> &JournalReference {
        &self.event_reference
    }

    /// The exact self-hash identity of the supplied strict Record bytes.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// The exact Record type declared by the supplied strict Record frame.
    pub fn record_type_id(&self) -> RecordTypeId {
        self.record_type_id
    }
}

/// A fail-closed outcome of structurally binding exact Record bytes to a retained event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventRecordStructuralBindingError {
    /// The referenced event is absent, cross-Registry, or differs in any retained identity field.
    RetainedReference(RetainedJournalError),
    /// The supplied Record bytes fail strict canonical framing or exact self-hash derivation.
    RecordDecode,
    /// The strict Record type is not the frozen type required for the retained event.
    RecordTypeMismatch,
    /// The strict Record self-hash is not the retained event's exact Record identity.
    RecordIdMismatch,
}

/// Validates the exact structural Record-to-event binding available from retained history.
///
/// This composes retained five-field reference resolution, strict Record framing,
/// the frozen event-to-required-Record-type mapping, and exact Record identity.
/// It deliberately does not decode a type-local Record body or establish any
/// semantic, authority, admission, Policy, custody, durability, or lifecycle result.
pub fn validate_event_record_structural_binding(
    retained_journal: &RetainedJournal,
    event_reference: &JournalReference,
    record_bytes: &[u8],
) -> Result<EventRecordStructuralBinding, EventRecordStructuralBindingError> {
    let resolved = retained_journal
        .resolve_reference(event_reference)
        .map_err(EventRecordStructuralBindingError::RetainedReference)?;
    let frame = StrictRecordFrame::decode_authoritative(record_bytes)
        .map_err(|_| EventRecordStructuralBindingError::RecordDecode)?;
    if frame.record_type_id() != resolved.event_type_id().required_record_type_id() {
        return Err(EventRecordStructuralBindingError::RecordTypeMismatch);
    }
    if frame.record_id().as_bytes() != resolved.event_record_id().as_bytes() {
        return Err(EventRecordStructuralBindingError::RecordIdMismatch);
    }
    Ok(EventRecordStructuralBinding {
        event_reference: event_reference.clone(),
        record_id: frame.record_id(),
        record_type_id: frame.record_type_id(),
    })
}

/// Typed, Record-local fields for the frozen FREEZE_ATTEMPT_START Record schema.
///
/// The Registry-relative root derivation and any later Journal authority binding
/// require external context, so they are intentionally not inferred here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeAttemptStartRecordInput {
    pub freeze_attempt_id: FreezeAttemptId,
    pub intended_root_id: IntendedRootId,
    pub subject_id: [u8; ID_LENGTH],
    pub policy_record_id: RecordId,
}

/// The exact-byte, Record-local FREEZE_ATTEMPT_START schema from Record Schema v0.3 §55.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeAttemptStartRecord {
    input: FreezeAttemptStartRecordInput,
}

impl FreezeAttemptStartRecord {
    /// Constructs the locally well-typed Record fields without an authority claim.
    pub fn new(input: FreezeAttemptStartRecordInput) -> Self {
        Self { input }
    }

    /// Returns the exact Record-local fields without inferring external context.
    pub fn input(&self) -> &FreezeAttemptStartRecordInput {
        &self.input
    }

    /// The frozen Record Type ID for FREEZE_ATTEMPT_START.
    pub fn record_type_id(&self) -> RecordTypeId {
        RecordTypeId::try_from(3).expect("FREEZE_ATTEMPT_START is assigned in Record Schema v0.3")
    }

    /// Emits the exact canonical Record framing and all required local fields.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        debug_assert_eq!(RECORD_DOMAIN.len(), 26);
        let mut bytes = Vec::with_capacity(180);
        bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
        bytes.extend_from_slice(RECORD_DOMAIN);
        bytes.extend_from_slice(&[0x03, 0x01, 0xa6, 0x00, 0x01, 0x01, 0x03, 0x10]);
        encode_bstr_32(&mut bytes, self.input.freeze_attempt_id.as_bytes());
        bytes.push(0x11);
        encode_bstr_32(&mut bytes, self.input.intended_root_id.as_bytes());
        bytes.push(0x12);
        encode_bstr_32(&mut bytes, &self.input.subject_id);
        bytes.push(0x13);
        encode_bstr_32(&mut bytes, self.input.policy_record_id.as_bytes());
        bytes
    }

    /// Returns SHA-256 of the exact authoritative Record bytes.
    pub fn record_id(&self) -> RecordId {
        let digest: [u8; ID_LENGTH] = Sha256::digest(self.authoritative_cbor())
            .as_slice()
            .try_into()
            .expect("SHA-256 always returns exactly 32 bytes");
        RecordId(digest)
    }

    /// Strictly decodes this exact v0.3 Record schema without normalization.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let frame = StrictRecordFrame::decode_authoritative(input)?;
        if frame.record_type_id() != RecordTypeId::try_from(3).expect("assigned Record Type") {
            return Err(RecordDecodeError);
        }
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 3
            || cursor.uint().map_err(|_| RecordDecodeError)? != 1
        {
            return Err(RecordDecodeError);
        }
        cursor.map_exact(6).map_err(|_| RecordDecodeError)?;
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 3 {
            return Err(RecordDecodeError);
        }
        cursor.key(16).map_err(|_| RecordDecodeError)?;
        let freeze_attempt_id =
            FreezeAttemptId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(17).map_err(|_| RecordDecodeError)?;
        let intended_root_id =
            IntendedRootId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(18).map_err(|_| RecordDecodeError)?;
        let subject_id = cursor.bstr_32().map_err(|_| RecordDecodeError)?;
        cursor.key(19).map_err(|_| RecordDecodeError)?;
        let policy_record_id =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        if !cursor.finished() {
            return Err(RecordDecodeError);
        }
        let decoded = Self::new(FreezeAttemptStartRecordInput {
            freeze_attempt_id,
            intended_root_id,
            subject_id,
            policy_record_id,
        });
        if decoded.authoritative_cbor() != input {
            return Err(RecordDecodeError);
        }
        Ok(decoded)
    }
}

/// Typed, Record-local fields decoded from a FREEZE_RECEIPT Record.
///
/// These fields establish only the strict local Record grammar and exact Record
/// identity. They do not establish policy, authority, admission, durability,
/// custody, or lifecycle truth.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeReceiptRecordInput {
    pub freeze_attempt_id: FreezeAttemptId,
    pub freeze_id: [u8; ID_LENGTH],
    pub attempt_start_journal_ref: JournalReference,
    pub subject_id: [u8; ID_LENGTH],
    pub manifest_id: RecordId,
    pub custody_mode_id: u64,
    pub creation_profile_ref: RecordId,
    pub path_identity_profile_id: u64,
    pub filesystem_profile_ref: RecordId,
    pub policy_record_id: RecordId,
    pub file_content_flush_state: u64,
    pub atomic_publish_no_replace_state: u64,
    pub parent_directory_flush_state: u64,
    pub platform_strongest_available: bool,
    pub requested_commit_durability_ref: Option<RecordId>,
    pub created_by_tool_version: String,
}

/// The exact-byte, Record-local FREEZE_RECEIPT schema from Record Schema v0.3 §56.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeReceiptRecord {
    record_id: RecordId,
    input: FreezeReceiptRecordInput,
}

impl FreezeReceiptRecord {
    /// Strictly decodes the frozen FREEZE_RECEIPT local Record grammar without normalization.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let frame = StrictRecordFrame::decode_authoritative(input)?;
        if frame.record_type_id() != RecordTypeId::try_from(4).expect("assigned Record Type") {
            return Err(RecordDecodeError);
        }
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 4
            || cursor.uint().map_err(|_| RecordDecodeError)? != 1
        {
            return Err(RecordDecodeError);
        }
        let field_count = cursor.map().map_err(|_| RecordDecodeError)?;
        if !matches!(field_count, 17 | 18) {
            return Err(RecordDecodeError);
        }
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 4 {
            return Err(RecordDecodeError);
        }
        cursor.key(16).map_err(|_| RecordDecodeError)?;
        let freeze_attempt_id =
            FreezeAttemptId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(17).map_err(|_| RecordDecodeError)?;
        let freeze_id = cursor.bstr_32().map_err(|_| RecordDecodeError)?;
        cursor.key(18).map_err(|_| RecordDecodeError)?;
        let attempt_start_journal_ref =
            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?;
        cursor.key(19).map_err(|_| RecordDecodeError)?;
        let subject_id = cursor.bstr_32().map_err(|_| RecordDecodeError)?;
        cursor.key(20).map_err(|_| RecordDecodeError)?;
        let manifest_id =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(21).map_err(|_| RecordDecodeError)?;
        let custody_mode_id = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !matches!(custody_mode_id, 1 | 2) {
            return Err(RecordDecodeError);
        }
        cursor.key(22).map_err(|_| RecordDecodeError)?;
        let creation_profile_ref =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(23).map_err(|_| RecordDecodeError)?;
        let path_identity_profile_id = cursor.uint().map_err(|_| RecordDecodeError)?;
        cursor.key(24).map_err(|_| RecordDecodeError)?;
        let filesystem_profile_ref =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(25).map_err(|_| RecordDecodeError)?;
        let policy_record_id =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(26).map_err(|_| RecordDecodeError)?;
        let file_content_flush_state = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !matches!(file_content_flush_state, 1..=3) {
            return Err(RecordDecodeError);
        }
        cursor.key(27).map_err(|_| RecordDecodeError)?;
        let atomic_publish_no_replace_state = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !matches!(atomic_publish_no_replace_state, 1..=3) {
            return Err(RecordDecodeError);
        }
        cursor.key(28).map_err(|_| RecordDecodeError)?;
        let parent_directory_flush_state = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !matches!(parent_directory_flush_state, 1..=3) {
            return Err(RecordDecodeError);
        }
        cursor.key(29).map_err(|_| RecordDecodeError)?;
        let platform_strongest_available = cursor.bool().map_err(|_| RecordDecodeError)?;
        let requested_commit_durability_ref = if field_count == 18 {
            cursor.key(30).map_err(|_| RecordDecodeError)?;
            Some(
                RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                    .map_err(|_| RecordDecodeError)?,
            )
        } else {
            None
        };
        cursor.key(31).map_err(|_| RecordDecodeError)?;
        let created_by_tool_version = cursor.text().map_err(|_| RecordDecodeError)?;
        if !cursor.finished() {
            return Err(RecordDecodeError);
        }
        Ok(Self {
            record_id: frame.record_id(),
            input: FreezeReceiptRecordInput {
                freeze_attempt_id,
                freeze_id,
                attempt_start_journal_ref,
                subject_id,
                manifest_id,
                custody_mode_id,
                creation_profile_ref,
                path_identity_profile_id,
                filesystem_profile_ref,
                policy_record_id,
                file_content_flush_state,
                atomic_publish_no_replace_state,
                parent_directory_flush_state,
                platform_strongest_available,
                requested_commit_durability_ref,
                created_by_tool_version,
            },
        })
    }

    /// The exact immutable identity of the strictly decoded FREEZE_RECEIPT Record.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// Returns the exact typed Receipt fields without inferring external context.
    pub fn input(&self) -> &FreezeReceiptRecordInput {
        &self.input
    }
}

/// The exact structural inputs for one bounded FREEZE_COMMITTED binding check.
///
/// This input deliberately contains no Manifest, Policy, storage, custody, or
/// external-authority evidence. Those absences prohibit a positive authority or
/// admission outcome.
#[derive(Clone, Debug)]
pub struct FreezeCommittedBindingInput<'a> {
    pub retained_journal: &'a RetainedJournal,
    pub committed_event_reference: JournalReference,
    pub freeze_attempt_start_record: &'a FreezeAttemptStartRecord,
    pub freeze_receipt_record: &'a FreezeReceiptRecord,
}

/// A failure to establish the bounded FREEZE_COMMITTED structural bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FreezeCommittedBindingError {
    /// The supplied Journal reference does not resolve in retained history.
    RetainedReference(RetainedJournalError),
    /// The resolved terminal entry is not FREEZE_COMMITTED for this attempt.
    CommittedEventMismatch,
    /// The terminal Journal event does not bind the exact Receipt Record identity.
    ReceiptRecordMismatch,
    /// The Receipt's start reference does not resolve to the retained FREEZE_ATTEMPT_STARTED event.
    AttemptStartReferenceMismatch,
    /// The retained FREEZE_ATTEMPT_STARTED event does not bind the exact START Record identity.
    AttemptStartRecordMismatch,
    /// The exact START Record root does not match the retained START Journal root.
    IntendedRootMismatch,
    /// The START, Receipt, and terminal event do not bind one Freeze Attempt identity.
    FreezeAttemptMismatch,
    /// The Receipt and START do not preserve the same exact subject identity.
    SubjectMismatch,
    /// The Receipt and START do not preserve the same exact Policy identity.
    PolicyMismatch,
    /// The Receipt's retained start entry is not strictly prior to the terminal entry.
    AttemptStartNotPrior,
}

/// The only non-error outcome of the bounded FREEZE_COMMITTED binding check.
///
/// This is explicitly non-success for authority and admission: all available
/// structural bindings agree, but required authority evidence is unavailable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FreezeCommittedBindingOutcome {
    /// Structural bindings agree; no authority or admission verdict is available.
    AuthorityEvidenceUnavailable,
}

/// Validates only the exact retained Journal / START / Receipt bindings available
/// to this runtime for FREEZE_COMMITTED.
///
/// A successful return never means authority, admission, durability, custody,
/// policy satisfaction, or lifecycle truth has been established.
pub fn validate_freeze_committed_binding(
    input: FreezeCommittedBindingInput<'_>,
) -> Result<FreezeCommittedBindingOutcome, FreezeCommittedBindingError> {
    if input.freeze_attempt_start_record.input.freeze_attempt_id
        != input.freeze_receipt_record.input.freeze_attempt_id
    {
        return Err(FreezeCommittedBindingError::FreezeAttemptMismatch);
    }
    let committed = input
        .retained_journal
        .resolve_reference(&input.committed_event_reference)
        .map_err(FreezeCommittedBindingError::RetainedReference)?;
    if committed.event_type_id().value() != 101
        || committed.event_type_id().required_record_type_id().value() != 4
        || committed.lifecycle_object_kind() != LifecycleObjectKind::FreezeAttempt
        || committed.lifecycle_object_id()
            != *input
                .freeze_receipt_record
                .input
                .freeze_attempt_id
                .as_bytes()
    {
        return Err(FreezeCommittedBindingError::CommittedEventMismatch);
    }
    if committed.event_record_id().as_bytes() != input.freeze_receipt_record.record_id.as_bytes() {
        return Err(FreezeCommittedBindingError::ReceiptRecordMismatch);
    }
    let start_reference = &input.freeze_receipt_record.input.attempt_start_journal_ref;
    if start_reference.entry_index().value() >= committed.entry_index().value() {
        return Err(FreezeCommittedBindingError::AttemptStartNotPrior);
    }
    let start = input
        .retained_journal
        .resolve_reference(start_reference)
        .map_err(FreezeCommittedBindingError::RetainedReference)?;
    if start.event_type_id().value() != 100
        || start.event_type_id().required_record_type_id().value() != 3
        || start.lifecycle_object_kind() != LifecycleObjectKind::FreezeAttempt
        || start.lifecycle_object_id()
            != *input
                .freeze_attempt_start_record
                .input
                .freeze_attempt_id
                .as_bytes()
    {
        return Err(FreezeCommittedBindingError::AttemptStartReferenceMismatch);
    }
    if start.event_record_id().as_bytes()
        != input.freeze_attempt_start_record.record_id().as_bytes()
    {
        return Err(FreezeCommittedBindingError::AttemptStartRecordMismatch);
    }
    if start.freeze_attempt_intended_root_id()
        != Some(input.freeze_attempt_start_record.input.intended_root_id)
    {
        return Err(FreezeCommittedBindingError::IntendedRootMismatch);
    }
    if input.freeze_attempt_start_record.input.subject_id
        != input.freeze_receipt_record.input.subject_id
    {
        return Err(FreezeCommittedBindingError::SubjectMismatch);
    }
    if input.freeze_attempt_start_record.input.policy_record_id
        != input.freeze_receipt_record.input.policy_record_id
    {
        return Err(FreezeCommittedBindingError::PolicyMismatch);
    }
    Ok(FreezeCommittedBindingOutcome::AuthorityEvidenceUnavailable)
}

/// Supplies exact authoritative Record bytes for a requested immutable Record identity.
///
/// This is an injected composition seam, not a persistent Record store or namespace
/// contract. `None` means only that this supplier has no bytes for the requested
/// identity; it does not establish deletion, nonexistence, or any authority fact.
pub trait ExactRecordByteResolver {
    /// Returns bytes purported to have exactly the requested Record identity.
    fn resolve(&self, record_id: RecordId) -> Option<&[u8]>;
}

/// A failure while obtaining and composing exact START and Receipt Record bytes.
///
/// Each unavailable, invalid, or wrong-identity payload outcome remains distinct
/// from the retained-Journal structural binding result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedFreezeCommittedBindingError {
    /// Retained Journal context did not provide the required exact reference facts.
    RetainedReference(RetainedJournalError),
    /// The resolved terminal Entry is not the required FREEZE_COMMITTED event.
    CommittedEventMismatch,
    /// The exact Receipt Record bytes required by the terminal Entry were unavailable.
    ReceiptPayloadUnavailable,
    /// Supplied Receipt bytes did not satisfy the strict FREEZE_RECEIPT local grammar.
    ReceiptDecode,
    /// Strictly decoded Receipt bytes did not hash to the terminal Entry's exact Record ID.
    ReceiptIdentityMismatch,
    /// The Receipt's retained START reference was not the required START event.
    AttemptStartReferenceMismatch,
    /// The exact START Record bytes required by the retained START Entry were unavailable.
    AttemptStartPayloadUnavailable,
    /// Supplied START bytes did not satisfy the strict FREEZE_ATTEMPT_START local grammar.
    AttemptStartDecode,
    /// Strictly decoded START bytes did not hash to the retained START Entry's exact Record ID.
    AttemptStartIdentityMismatch,
    /// Available exact inputs failed the existing bounded structural binding check.
    Binding(FreezeCommittedBindingError),
}

/// Resolves exact Record bytes only as required to compose one bounded FREEZE_COMMITTED check.
///
/// A non-error result remains `AuthorityEvidenceUnavailable`: this adapter proves no
/// authority, admission, Policy satisfaction, semantic-MANIFEST validity, custody,
/// durability, lifecycle truth, persistence, or external trust.
pub fn validate_resolved_freeze_committed_binding(
    retained_journal: &RetainedJournal,
    committed_event_reference: JournalReference,
    resolver: &impl ExactRecordByteResolver,
) -> Result<FreezeCommittedBindingOutcome, ResolvedFreezeCommittedBindingError> {
    let committed = retained_journal
        .resolve_reference(&committed_event_reference)
        .map_err(ResolvedFreezeCommittedBindingError::RetainedReference)?;
    if committed.event_type_id().value() != 101
        || committed.event_type_id().required_record_type_id().value() != 4
        || committed.lifecycle_object_kind() != LifecycleObjectKind::FreezeAttempt
    {
        return Err(ResolvedFreezeCommittedBindingError::CommittedEventMismatch);
    }
    let receipt_record_id = RecordId::try_from(committed.event_record_id().as_bytes().as_slice())
        .expect("EventRecordId has the fixed RecordId width");
    let receipt_bytes = resolver
        .resolve(receipt_record_id)
        .ok_or(ResolvedFreezeCommittedBindingError::ReceiptPayloadUnavailable)?;
    let receipt = FreezeReceiptRecord::decode_authoritative(receipt_bytes)
        .map_err(|_| ResolvedFreezeCommittedBindingError::ReceiptDecode)?;
    if receipt.record_id() != receipt_record_id {
        return Err(ResolvedFreezeCommittedBindingError::ReceiptIdentityMismatch);
    }

    let start_reference = &receipt.input().attempt_start_journal_ref;
    let start = retained_journal
        .resolve_reference(start_reference)
        .map_err(ResolvedFreezeCommittedBindingError::RetainedReference)?;
    if start.event_type_id().value() != 100
        || start.event_type_id().required_record_type_id().value() != 3
        || start.lifecycle_object_kind() != LifecycleObjectKind::FreezeAttempt
    {
        return Err(ResolvedFreezeCommittedBindingError::AttemptStartReferenceMismatch);
    }
    let start_record_id = RecordId::try_from(start.event_record_id().as_bytes().as_slice())
        .expect("EventRecordId has the fixed RecordId width");
    let start_bytes = resolver
        .resolve(start_record_id)
        .ok_or(ResolvedFreezeCommittedBindingError::AttemptStartPayloadUnavailable)?;
    let freeze_attempt_start_record =
        FreezeAttemptStartRecord::decode_authoritative(start_bytes)
            .map_err(|_| ResolvedFreezeCommittedBindingError::AttemptStartDecode)?;
    if freeze_attempt_start_record.record_id() != start_record_id {
        return Err(ResolvedFreezeCommittedBindingError::AttemptStartIdentityMismatch);
    }

    validate_freeze_committed_binding(FreezeCommittedBindingInput {
        retained_journal,
        committed_event_reference,
        freeze_attempt_start_record: &freeze_attempt_start_record,
        freeze_receipt_record: &receipt,
    })
    .map_err(ResolvedFreezeCommittedBindingError::Binding)
}

/// Typed, Record-local fields for the frozen GENESIS Record schema.
///
/// These fields determine immutable Record identity only. They do not establish
/// that a Journal Entry later authorizes this Record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenesisRecordInput {
    pub registry_id: RegistryId,
    pub journal_format_version: u64,
    pub record_identity_profile_id: u64,
    pub storage_capability_class_id: RecordId,
    pub environment_observation_id: RecordId,
    pub created_by_tool_version: String,
}

/// A GENESIS Record field outside its exact structural domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenesisRecordFormatError;

/// A rejected GENESIS Record byte sequence under its strict v0.3 grammar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordDecodeError;

/// The exact-byte, structural GENESIS Record from Record Schema v0.3 §51.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenesisRecord {
    input: GenesisRecordInput,
}

impl GenesisRecord {
    /// Constructs a Record-local GENESIS statement without claiming authority.
    pub fn new(input: GenesisRecordInput) -> Result<Self, GenesisRecordFormatError> {
        if input.journal_format_version > ER_UINT_MAX
            || input.record_identity_profile_id > ER_UINT_MAX
            || u64::try_from(input.created_by_tool_version.len())
                .map_or(true, |length| !is_canonical_text_length(length))
        {
            return Err(GenesisRecordFormatError);
        }
        Ok(Self { input })
    }

    /// Emits the exact canonical Record framing and all required GENESIS fields.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        debug_assert_eq!(RECORD_DOMAIN.len(), 26);
        let mut bytes = Vec::with_capacity(180);
        bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
        bytes.extend_from_slice(RECORD_DOMAIN);
        bytes.extend_from_slice(&[0x01, 0x01, 0xa8, 0x00, 0x01, 0x01, 0x01, 0x10]);
        encode_bstr_32(&mut bytes, self.input.registry_id.as_bytes());
        bytes.push(0x11);
        encode_uint(&mut bytes, self.input.journal_format_version);
        bytes.push(0x12);
        encode_uint(&mut bytes, self.input.record_identity_profile_id);
        bytes.push(0x13);
        encode_bstr_32(
            &mut bytes,
            self.input.storage_capability_class_id.as_bytes(),
        );
        bytes.push(0x14);
        encode_bstr_32(&mut bytes, self.input.environment_observation_id.as_bytes());
        bytes.push(0x15);
        encode_text(&mut bytes, &self.input.created_by_tool_version);
        bytes
    }

    /// Returns `SHA256(authoritative_cbor())` as the immutable Record identity.
    pub fn record_id(&self) -> RecordId {
        let digest: [u8; ID_LENGTH] = Sha256::digest(self.authoritative_cbor())
            .as_slice()
            .try_into()
            .expect("SHA-256 always returns exactly 32 bytes");
        RecordId(digest)
    }

    /// Strictly decodes GENESIS Record bytes and rejects normalization.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1
            || cursor.uint().map_err(|_| RecordDecodeError)? != 1
        {
            return Err(RecordDecodeError);
        }
        cursor.map_exact(8).map_err(|_| RecordDecodeError)?;
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(16).map_err(|_| RecordDecodeError)?;
        let registry_id =
            RegistryId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(17).map_err(|_| RecordDecodeError)?;
        let journal_format_version = cursor.uint().map_err(|_| RecordDecodeError)?;
        cursor.key(18).map_err(|_| RecordDecodeError)?;
        let record_identity_profile_id = cursor.uint().map_err(|_| RecordDecodeError)?;
        cursor.key(19).map_err(|_| RecordDecodeError)?;
        let storage_capability_class_id =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(20).map_err(|_| RecordDecodeError)?;
        let environment_observation_id =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(21).map_err(|_| RecordDecodeError)?;
        let created_by_tool_version = cursor.text().map_err(|_| RecordDecodeError)?;
        if !cursor.finished() {
            return Err(RecordDecodeError);
        }
        let decoded = Self::new(GenesisRecordInput {
            registry_id,
            journal_format_version,
            record_identity_profile_id,
            storage_capability_class_id,
            environment_observation_id,
            created_by_tool_version,
        })
        .map_err(|_| RecordDecodeError)?;
        if decoded.authoritative_cbor() != input {
            return Err(RecordDecodeError);
        }
        Ok(decoded)
    }
}

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

    fn validate_for_context(
        &self,
        context: AuthorityDependencyContext,
    ) -> Result<(), AuthorityDependencyCollectionError> {
        validate_authority_dependency_context(context, &self.elements)?;
        validate_authority_dependency_indices(&self.elements)?;
        for pair in self.elements.windows(2) {
            if (pair[0].entry_index(), pair[0].entry_hash())
                > (pair[1].entry_index(), pair[1].entry_hash())
            {
                return Err(AuthorityDependencyCollectionError::NonCanonicalOrder);
            }
        }
        Ok(())
    }

    fn contains_event_type(&self, event_type_id: u16) -> bool {
        self.elements
            .iter()
            .any(|reference| reference.event_type_id().value() == event_type_id)
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
    fn canonical_text_lengths_end_at_er_uint_max() {
        assert!(is_canonical_text_length(ER_UINT_MAX));
        assert!(!is_canonical_text_length(ER_UINT_MAX + 1));
    }

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
