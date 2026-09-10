//! EvidenceRegistry implementation surface.
//!
//! This module begins with the deterministic FreezeRoot identity defined by
//! Identity Format v0.3 §§46–47. It deliberately exposes only the fixed
//! FreezeRoot tuple, rather than a generic identity envelope.

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

mod authoritative_store;
pub use authoritative_store::{
    AcceptedAuthoritativeReviewAdmission, AuthoritativeFreezeCommittedBinding,
    AuthoritativeFreezeCommittedBindingError, AuthoritativePublicationDurability,
    AuthoritativeRegistryStore, AuthoritativeRegistryStoreOpenError,
    AuthoritativeReviewAdmissionAcceptanceError, AuthoritativeReviewAdmissionPublication,
    AuthoritativeReviewAdmissionPublicationError,
    AuthoritativeReviewAdmissionPublishedReceiptUncertain,
    AuthoritativeReviewAdmissionRuntimeError, AuthoritativeReviewAdmissionRuntimeOutcome,
    AuthoritativeReviewAdmissionSection82, AuthoritativeReviewAdmissionSection82Error,
    DurabilityActionState, AUTHORITATIVE_STORE_MAX_NAMESPACE_BYTES,
    AUTHORITATIVE_STORE_MAX_OBJECTS, AUTHORITATIVE_STORE_MAX_OBJECT_BYTES,
};

const FREEZE_ROOT_DOMAIN: &[u8] = b"EvidenceRegistry.FreezeRoot.v1";
const ID_LENGTH: usize = 32;
/// SHA-256 of the adopted raw Terminal Authority Closure Core v0.1 bytes.
///
/// This is a selector literal, not an inferred runtime epoch or producer attestation.
pub const TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256: [u8; ID_LENGTH] = [
    0xcb, 0x7e, 0xe9, 0xe9, 0xe5, 0xf9, 0xd0, 0x22, 0x9d, 0xba, 0x26, 0x01, 0x2e, 0xcb, 0x52, 0x31,
    0xc2, 0x67, 0x71, 0xa6, 0xd5, 0x4c, 0x3c, 0x95, 0x76, 0x2c, 0x10, 0x94, 0xec, 0x1f, 0xc1, 0x6d,
];

const TERMINAL_REVIEW_ADMISSION_LEGACY_SELECTOR_SHA256: [u8; ID_LENGTH] = [
    0x65, 0x81, 0x3e, 0x35, 0x6e, 0xab, 0xa8, 0x9c, 0x63, 0x84, 0x75, 0xa4, 0x0c, 0x69, 0xf7, 0x28,
    0x12, 0x3f, 0x43, 0x14, 0x04, 0x0f, 0x07, 0x5a, 0x09, 0xc6, 0xbe, 0x34, 0xdb, 0x14, 0xe1, 0x14,
];

#[cfg(test)]
std::thread_local! {
    pub(crate) static JOURNAL_REFERENCE_EQUALITY_COMPARISONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static LIFECYCLE_KIND_EQUALITY_COMPARISONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static FREEZE_ATTEMPT_ID_EQUALITY_COMPARISONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// A Registry identity represented in its authoritative 32-byte form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegistryId([u8; ID_LENGTH]);

/// A Freeze attempt identity represented in its authoritative 32-byte form.
#[derive(Clone, Copy, Debug)]
pub struct FreezeAttemptId([u8; ID_LENGTH]);

impl PartialEq for FreezeAttemptId {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(test)]
        FREEZE_ATTEMPT_ID_EQUALITY_COMPARISONS.with(|comparisons| {
            comparisons.set(comparisons.get().saturating_add(1));
        });
        self.0 == other.0
    }
}

impl Eq for FreezeAttemptId {}

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
#[derive(Clone, Copy, Debug)]
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

impl PartialEq for LifecycleObjectKind {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(test)]
        LIFECYCLE_KIND_EQUALITY_COMPARISONS.with(|comparisons| {
            comparisons.set(comparisons.get().saturating_add(1));
        });
        self.value() == other.value()
    }
}

impl Eq for LifecycleObjectKind {}

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
#[derive(Clone, Debug)]
pub struct JournalReference {
    registry_id: RegistryId,
    entry_index: JournalEntryIndex,
    entry_hash: JournalEntryHash,
    event_type_id: EventTypeId,
    event_record_id: EventRecordId,
}

impl PartialEq for JournalReference {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(test)]
        JOURNAL_REFERENCE_EQUALITY_COMPARISONS.with(|comparisons| {
            comparisons.set(comparisons.get().saturating_add(1));
        });
        self.registry_id == other.registry_id
            && self.entry_index == other.entry_index
            && self.entry_hash == other.entry_hash
            && self.event_type_id == other.event_type_id
            && self.event_record_id == other.event_record_id
    }
}

impl Eq for JournalReference {}

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
    identity_dependencies: IdentityDependencyCollection,
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

    fn identity_dependencies(&self) -> &[IdentityDependency] {
        match self {
            Self::Genesis(_) => &[],
            Self::Common(entry) => &entry.identity_dependencies.elements,
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

/// A failure to derive exactly one Journal Anchor from retained Journal prefixes.
///
/// This is a retained-history result only. A successful resolution does not by
/// itself establish Review Package authority, §82 completion, Policy
/// satisfaction, Admission, or terminal publication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetainedJournalAnchorResolutionError {
    /// No retained prefix derives the supplied typed Journal Anchor identity.
    Missing,
    /// More than one retained prefix derives the supplied typed Journal Anchor identity.
    Ambiguous,
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
/// not an authority or admission decision. A live Journal is intentionally not
/// cloneable: reconstructing the same retained bytes creates a distinct mutation
/// domain and cannot inherit an outstanding live-operation capability.
///
/// ```compile_fail
/// use evidence_registry::RetainedJournal;
/// fn fork_live_history(journal: RetainedJournal) {
///     let _divergent_history = journal.clone();
/// }
/// ```
#[derive(Debug)]
pub struct RetainedJournal {
    live_instance_identity: Arc<RetainedJournalInstanceIdentity>,
    registry_id: RegistryId,
    entries: Vec<RetainedJournalEntry>,
    current_states: BTreeMap<(u8, [u8; ID_LENGTH]), LifecycleObjectState>,
}

#[cfg(test)]
std::thread_local! {
    static RETAINED_REPLAY_ENTRY_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[derive(Debug)]
struct RetainedJournalInstanceIdentity {
    outstanding_review_admissions: AtomicUsize,
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
            live_instance_identity: Arc::new(RetainedJournalInstanceIdentity {
                outstanding_review_admissions: AtomicUsize::new(0),
            }),
            registry_id,
            entries: Vec::new(),
            current_states: BTreeMap::new(),
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

    /// Returns the exact current retained Journal head reference.
    ///
    /// A retained Journal is created only from GENESIS and never removes an
    /// Entry, so this is always the final retained prefix. Reading this value
    /// alone does not provide the serializable accept-and-snapshot operation
    /// required for REVIEW_ADMISSION operation-start provenance.
    pub fn current_head_reference(&self) -> JournalReference {
        let entry = self
            .entries
            .last()
            .expect("RetainedJournal is initialized from one GENESIS Entry");
        JournalReference::new(
            self.registry_id,
            entry.entry_index(),
            entry.entry_hash(),
            entry.event_type_id(),
            entry.event_record_id(),
        )
    }

    /// Resolves a version-1 Journal Anchor identity solely by reconstructing
    /// canonical Anchors from retained Journal prefixes.
    ///
    /// No caller-supplied Anchor body, cache, helper-provided head, or raw
    /// digest substitute participates in the candidate set. The result remains
    /// retained-history structural evidence; callers must establish every
    /// separate lifecycle and authority precondition before treating it as a
    /// Review Admission input.
    pub fn resolve_journal_anchor_id(
        &self,
        anchor_id: JournalAnchorId,
    ) -> Result<JournalAnchor, RetainedJournalAnchorResolutionError> {
        let mut resolved = None;
        for entry in &self.entries {
            let candidate =
                JournalAnchor::new(self.registry_id, entry.entry_index(), entry.entry_hash(), 1)
                    .expect("Journal format version 1 is within the EvidenceRegistry UInt range");
            if candidate.anchor_id() != anchor_id {
                continue;
            }
            if resolved.replace(candidate).is_some() {
                return Err(RetainedJournalAnchorResolutionError::Ambiguous);
            }
        }
        resolved.ok_or(RetainedJournalAnchorResolutionError::Missing)
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
            let common = self.validate_freeze_commit_rejected(input, event_type_id)?;
            return self.append_decoded_common(common, input);
        }
        if event_type_id.value() == 700 {
            let common = self.validate_eviction_started(input, event_type_id)?;
            return self.append_decoded_common(common, input);
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
            return self.append_decoded_common(common, input);
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
                    self.append_decoded_common(common, input)
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
                    self.append_decoded_common(common, input)
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
            identity_dependencies: common.identity_dependencies,
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
            identity_dependencies: common.identity_dependencies,
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
            identity_dependencies: common.identity_dependencies,
            authority_dependencies: common.authority_dependencies,
            authoritative_bytes: input.to_vec(),
        }))
    }

    fn validate_freeze_commit_rejected(
        &self,
        input: &[u8],
        expected_event_type_id: EventTypeId,
    ) -> Result<DecodedCommonJournalEntry, RetainedJournalError> {
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
        Ok(common)
    }

    fn validate_eviction_started(
        &self,
        input: &[u8],
        expected_event_type_id: EventTypeId,
    ) -> Result<DecodedCommonJournalEntry, RetainedJournalError> {
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
        Ok(common)
    }

    fn append_decoded_common(
        &mut self,
        common: DecodedCommonJournalEntry,
        input: &[u8],
    ) -> Result<(), RetainedJournalError> {
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
            identity_dependencies: common.identity_dependencies,
            authority_dependencies: common.authority_dependencies,
            authoritative_bytes: input.to_vec(),
        }))
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
        let before = self
            .current_states
            .get(&(
                common.lifecycle_object_kind.value(),
                common.lifecycle_object_id,
            ))
            .copied()
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
        let mut states: Vec<(LifecycleObjectKind, [u8; ID_LENGTH], LifecycleObjectState)> =
            Vec::new();
        let mut state_indices: BTreeMap<(u8, [u8; ID_LENGTH]), usize> = BTreeMap::new();
        let mut freeze_attempts = Vec::new();
        let mut freeze_attempt_indices: BTreeMap<[u8; ID_LENGTH], usize> = BTreeMap::new();
        let mut capability_epochs = Vec::with_capacity(self.entries.len());
        let mut authority_dependency_edges = Vec::new();
        for entry in &self.entries {
            #[cfg(test)]
            RETAINED_REPLAY_ENTRY_VISITS.with(|visits| visits.set(visits.get() + 1));
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
            let state_index = state_indices.get(&(kind.value(), object_id)).copied();
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
                state_indices.insert((kind.value(), object_id), states.len());
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
                        freeze_attempt_indices
                            .insert(*freeze_attempt_id.as_bytes(), freeze_attempts.len());
                        freeze_attempts.push(ReconstructedFreezeAttempt {
                            freeze_attempt_id,
                            intended_root_id,
                            state,
                        });
                    }
                    101..=103 => {
                        let attempt_index = freeze_attempt_indices
                            .get(freeze_attempt_id.as_bytes())
                            .copied()
                            .ok_or(RetainedJournalError::LifecycleTransition)?;
                        freeze_attempts[attempt_index].state = state;
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
        let lifecycle_key = (
            entry.lifecycle_object_kind().value(),
            entry.lifecycle_object_id(),
        );
        let before = self
            .current_states
            .get(&lifecycle_key)
            .copied()
            .unwrap_or_else(|| absent_state_for_kind(entry.lifecycle_object_kind()));
        let after = entry
            .event_type_id()
            .resulting_state(before)
            .map_err(|_| RetainedJournalError::LifecycleTransition)?;
        validate_state_only_legal_transition(entry.event_type_id(), before, after)
            .map_err(|_| RetainedJournalError::LifecycleTransition)?;
        self.entries.push(entry);
        self.current_states.insert(lifecycle_key, after);
        Ok(())
    }

    fn identity_dependencies_for_reference(
        &self,
        reference: &JournalReference,
    ) -> Result<&[IdentityDependency], RetainedJournalError> {
        self.resolve_reference(reference)?;
        let entry_index = usize::try_from(reference.entry_index().value())
            .map_err(|_| RetainedJournalError::MissingReference)?;
        self.entries
            .get(entry_index)
            .map(RetainedJournalEntry::identity_dependencies)
            .ok_or(RetainedJournalError::MissingReference)
    }

    fn authority_dependencies_for_reference(
        &self,
        reference: &JournalReference,
    ) -> Result<&[JournalReference], RetainedJournalError> {
        self.resolve_reference(reference)?;
        let entry_index = usize::try_from(reference.entry_index().value())
            .map_err(|_| RetainedJournalError::MissingReference)?;
        self.entries
            .get(entry_index)
            .map(RetainedJournalEntry::authority_dependencies)
            .ok_or(RetainedJournalError::MissingReference)
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

/// The exact bounded current-history relation of a canonical Journal Anchor.
///
/// This is a retained-history comparison only. It does not decide whether a
/// Review Request or Result is authoritative, whether §82 has completed, any
/// Policy outcome, Admission, or terminal Journal publication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JournalAnchorHistoryComparison {
    AnchorEqualsCurrentHead,
    AnchorIsValidAncestor,
    JournalDivergence,
    JournalHistoryBehindAnchor,
    AnchorFromDifferentRegistry,
    AnchorInvalid,
}

/// Compares a canonical Journal Anchor against the contiguous retained Journal.
///
/// The comparison implements Lifecycle v0.10.2 §116 for the in-memory retained
/// Journal: same-head identity, matching prior prefix, divergence at an
/// existing index, history behind the Anchor, and cross-Registry rejection.
pub fn compare_retained_journal_anchor_history(
    retained_journal: &RetainedJournal,
    anchor: &JournalAnchor,
) -> JournalAnchorHistoryComparison {
    if anchor.registry_id != retained_journal.registry_id {
        return JournalAnchorHistoryComparison::AnchorFromDifferentRegistry;
    }
    if anchor.journal_format_version != 1 {
        return JournalAnchorHistoryComparison::AnchorInvalid;
    }
    let Some(current_head) = retained_journal.entries.last() else {
        return JournalAnchorHistoryComparison::AnchorInvalid;
    };
    if anchor.journal_head_index > current_head.entry_index() {
        return JournalAnchorHistoryComparison::JournalHistoryBehindAnchor;
    }
    let anchored_entry = usize::try_from(anchor.journal_head_index.value())
        .ok()
        .and_then(|index| retained_journal.entries.get(index));
    let Some(anchored_entry) = anchored_entry else {
        return JournalAnchorHistoryComparison::AnchorInvalid;
    };
    if anchored_entry.entry_hash() != anchor.journal_head_hash {
        return JournalAnchorHistoryComparison::JournalDivergence;
    }
    if anchored_entry.entry_index() == current_head.entry_index() {
        JournalAnchorHistoryComparison::AnchorEqualsCurrentHead
    } else {
        JournalAnchorHistoryComparison::AnchorIsValidAncestor
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

fn encode_bstr(output: &mut Vec<u8>, bytes: &[u8]) {
    let length = u64::try_from(bytes.len()).expect("platform usize fits into u64");
    debug_assert!(length <= ER_UINT_MAX);
    match length {
        0..=23 => output.push(0x40 | length as u8),
        24..=0xff => output.extend_from_slice(&[0x58, length as u8]),
        0x100..=0xffff => output.extend_from_slice(&[0x59, (length >> 8) as u8, length as u8]),
        0x1_0000..=0xffff_ffff => output.extend_from_slice(&[
            0x5a,
            (length >> 24) as u8,
            (length >> 16) as u8,
            (length >> 8) as u8,
            length as u8,
        ]),
        _ => output.extend_from_slice(&[
            0x5b,
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

fn encode_journal_reference(output: &mut Vec<u8>, reference: &JournalReference) {
    output.push(0x85);
    encode_bstr_32(output, reference.registry_id().as_bytes());
    encode_uint(output, reference.entry_index().value());
    encode_bstr_32(output, reference.entry_hash().as_bytes());
    encode_uint(output, u64::from(reference.event_type_id().value()));
    encode_bstr_32(output, reference.event_record_id().as_bytes());
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

    fn bstr(&mut self) -> Result<Vec<u8>, JournalEntryDecodeError> {
        let length = usize::try_from(self.initial(2)?).map_err(|_| JournalEntryDecodeError)?;
        let end = self
            .offset
            .checked_add(length)
            .ok_or(JournalEntryDecodeError)?;
        let bytes = self
            .input
            .get(self.offset..end)
            .ok_or(JournalEntryDecodeError)?
            .to_vec();
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

/// One structurally framed MANIFEST Artifact Entry.
///
/// `path_components` retains exact component bytes but does not interpret their
/// selected path-identity profile, establish path ordering/deduplication, or
/// establish any filesystem fact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestArtifactEntry {
    pub artifact_kind_id: u64,
    pub path_components: Vec<Vec<u8>>,
    pub size_bytes: u64,
    pub digest_algorithm_id: u64,
    pub digest_bytes: [u8; ID_LENGTH],
}

/// Derives the selected regular-file Subject projection from an already
/// canonical Manifest artifact array. This is a pure projection only; callers
/// must separately establish source, retained-payload, and Record authority.
pub fn derive_selected_regular_file_subject(
    artifacts: &[ManifestArtifactEntry],
) -> Result<[u8; ID_LENGTH], RecordDecodeError> {
    let mut previous_path: Option<&[Vec<u8>]> = None;
    for artifact in artifacts {
        if artifact.artifact_kind_id != 1
            || artifact.digest_algorithm_id != 1
            || artifact.path_components.is_empty()
            || artifact.path_components.iter().any(|component| {
                component.is_empty() || matches!(component.as_slice(), b"." | b"..")
            })
            || previous_path.is_some_and(|previous| previous >= artifact.path_components.as_slice())
        {
            return Err(RecordDecodeError);
        }
        previous_path = Some(&artifact.path_components);
    }

    let mut projection = Vec::with_capacity(64 + artifacts.len() * 64);
    projection.push(0x85);
    encode_text(&mut projection, "EvidenceRegistry.Subject.RegularFiles.v1");
    encode_uint(&mut projection, 1);
    encode_uint(&mut projection, 1);
    encode_uint(&mut projection, artifacts.len() as u64);
    encode_array_length(&mut projection, artifacts.len());
    for artifact in artifacts {
        projection.push(0x85);
        encode_uint(&mut projection, artifact.artifact_kind_id);
        encode_array_length(&mut projection, artifact.path_components.len());
        for component in &artifact.path_components {
            encode_bstr(&mut projection, component);
        }
        encode_uint(&mut projection, artifact.size_bytes);
        encode_uint(&mut projection, artifact.digest_algorithm_id);
        encode_bstr_32(&mut projection, &artifact.digest_bytes);
    }
    Ok(Sha256::digest(projection)
        .as_slice()
        .try_into()
        .expect("SHA-256 always has exact Subject width"))
}

/// Typed, Record-local fields decoded from a MANIFEST Record.
///
/// The selected path and digest profile semantics remain external. These fields
/// retain only the strict Record-local grammar, including the frozen SHA-256
/// digest algorithm and width, and make no Manifest semantic, authority,
/// custody, durability, or filesystem claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestRecordInput {
    pub subject_id: [u8; ID_LENGTH],
    pub artifact_count: u64,
    pub path_identity_profile_id: u64,
    pub digest_profile_id: u64,
    pub artifacts: Vec<ManifestArtifactEntry>,
}

/// The exact-byte, Record-local MANIFEST schema from Record Schema v0.3 §§52–54.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestRecord {
    record_id: RecordId,
    input: ManifestRecordInput,
}

impl ManifestRecord {
    /// Constructs canonical MANIFEST Record bytes from already selected local
    /// fields. This establishes neither source, retained-payload, profile, nor
    /// authority facts.
    pub fn new(input: ManifestRecordInput) -> Result<Self, RecordDecodeError> {
        if input.artifact_count != input.artifacts.len() as u64 {
            return Err(RecordDecodeError);
        }
        let mut artifact_paths = BTreeSet::new();
        for artifact in &input.artifacts {
            if !matches!(artifact.artifact_kind_id, 1 | 2)
                || artifact.path_components.is_empty()
                || artifact.path_components.iter().any(|component| {
                    component.is_empty() || matches!(component.as_slice(), b"." | b"..")
                })
                || artifact.digest_algorithm_id != 1
                || !artifact_paths.insert(artifact.path_components.clone())
            {
                return Err(RecordDecodeError);
            }
        }
        let mut record = Self {
            record_id: RecordId::try_from([0_u8; ID_LENGTH].as_slice())
                .expect("RecordId has exact fixed width"),
            input,
        };
        record.record_id =
            RecordId::try_from(Sha256::digest(record.authoritative_cbor()).as_slice())
                .expect("SHA-256 has exact RecordId width");
        Ok(record)
    }

    /// Emits the exact canonical MANIFEST Record bytes for these local fields.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(128 + self.input.artifacts.len() * 80);
        bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
        bytes.extend_from_slice(RECORD_DOMAIN);
        bytes.extend_from_slice(&[0x02, 0x01, 0xa7, 0x00, 0x01, 0x01, 0x02, 0x10]);
        encode_bstr_32(&mut bytes, &self.input.subject_id);
        encode_uint(&mut bytes, 17);
        encode_uint(&mut bytes, self.input.artifact_count);
        encode_uint(&mut bytes, 18);
        encode_uint(&mut bytes, self.input.path_identity_profile_id);
        encode_uint(&mut bytes, 19);
        encode_uint(&mut bytes, self.input.digest_profile_id);
        encode_uint(&mut bytes, 20);
        encode_array_length(&mut bytes, self.input.artifacts.len());
        for artifact in &self.input.artifacts {
            bytes.push(0x85);
            encode_uint(&mut bytes, artifact.artifact_kind_id);
            encode_array_length(&mut bytes, artifact.path_components.len());
            for component in &artifact.path_components {
                encode_bstr(&mut bytes, component);
            }
            encode_uint(&mut bytes, artifact.size_bytes);
            encode_uint(&mut bytes, artifact.digest_algorithm_id);
            encode_bstr_32(&mut bytes, &artifact.digest_bytes);
        }
        bytes
    }

    /// Strictly decodes the frozen MANIFEST local Record grammar without normalization.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let frame = StrictRecordFrame::decode_authoritative(input)?;
        if frame.record_type_id() != RecordTypeId::try_from(2).expect("assigned Record Type") {
            return Err(RecordDecodeError);
        }
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 2
            || cursor.uint().map_err(|_| RecordDecodeError)? != 1
        {
            return Err(RecordDecodeError);
        }
        cursor.map_exact(7).map_err(|_| RecordDecodeError)?;
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 2 {
            return Err(RecordDecodeError);
        }
        cursor.key(16).map_err(|_| RecordDecodeError)?;
        let subject_id = cursor.bstr_32().map_err(|_| RecordDecodeError)?;
        cursor.key(17).map_err(|_| RecordDecodeError)?;
        let artifact_count = cursor.uint().map_err(|_| RecordDecodeError)?;
        cursor.key(18).map_err(|_| RecordDecodeError)?;
        let path_identity_profile_id = cursor.uint().map_err(|_| RecordDecodeError)?;
        cursor.key(19).map_err(|_| RecordDecodeError)?;
        let digest_profile_id = cursor.uint().map_err(|_| RecordDecodeError)?;
        cursor.key(20).map_err(|_| RecordDecodeError)?;
        let actual_artifact_count = cursor.array().map_err(|_| RecordDecodeError)?;
        if artifact_count != actual_artifact_count as u64 {
            return Err(RecordDecodeError);
        }
        let mut artifacts = Vec::new();
        let mut artifact_paths = BTreeSet::new();
        for _ in 0..actual_artifact_count {
            cursor.array_exact(5).map_err(|_| RecordDecodeError)?;
            let artifact_kind_id = cursor.uint().map_err(|_| RecordDecodeError)?;
            if !matches!(artifact_kind_id, 1 | 2) {
                return Err(RecordDecodeError);
            }
            let component_count = cursor.array().map_err(|_| RecordDecodeError)?;
            if component_count == 0 {
                return Err(RecordDecodeError);
            }
            let mut path_components = Vec::new();
            for _ in 0..component_count {
                let component = cursor.bstr().map_err(|_| RecordDecodeError)?;
                if component.is_empty() || matches!(component.as_slice(), b"." | b"..") {
                    return Err(RecordDecodeError);
                }
                path_components.push(component);
            }
            let size_bytes = cursor.uint().map_err(|_| RecordDecodeError)?;
            let digest_algorithm_id = cursor.uint().map_err(|_| RecordDecodeError)?;
            if digest_algorithm_id != 1 {
                return Err(RecordDecodeError);
            }
            let digest_bytes = cursor.bstr_32().map_err(|_| RecordDecodeError)?;
            if !artifact_paths.insert(path_components.clone()) {
                return Err(RecordDecodeError);
            }
            artifacts.push(ManifestArtifactEntry {
                artifact_kind_id,
                path_components,
                size_bytes,
                digest_algorithm_id,
                digest_bytes,
            });
        }
        if !cursor.finished() {
            return Err(RecordDecodeError);
        }
        let decoded = Self::new(ManifestRecordInput {
            subject_id,
            artifact_count,
            path_identity_profile_id,
            digest_profile_id,
            artifacts,
        })?;
        if decoded.record_id != frame.record_id() || decoded.authoritative_cbor() != input {
            return Err(RecordDecodeError);
        }
        Ok(decoded)
    }

    /// The exact immutable identity of the strictly decoded MANIFEST Record.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// The frozen Record Type ID for MANIFEST.
    pub fn record_type_id(&self) -> RecordTypeId {
        RecordTypeId::try_from(2).expect("MANIFEST is assigned in Record Schema v0.3")
    }

    /// Returns exact local MANIFEST fields without profile or authority inference.
    pub fn input(&self) -> &ManifestRecordInput {
        &self.input
    }
}

/// The exact local fields from the frozen POLICY shape with no optional
/// requirements. This is a declared-context witness only; it does not establish
/// Policy authority, satisfaction, admission, or a lifecycle outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MinimalPolicyRecord {
    record_id: RecordId,
    gate_scope_ref: RecordId,
    operation_start_journal_ref: JournalReference,
    supported_context_ids: Vec<u64>,
}

/// One caller-selected frozen Policy evaluation context.
///
/// This exact registry mapping is not inferred from POLICY fields. A context
/// declaration is only one prerequisite for later Policy evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyEvaluationContext {
    FreezeCommit,
    ReviewAdmission,
    CloseoutCreation,
    CloseoutPostcondition,
    ReviewRequestCreation,
}

impl PolicyEvaluationContext {
    fn id(self) -> u64 {
        match self {
            Self::FreezeCommit => 1,
            Self::ReviewAdmission => 2,
            Self::CloseoutCreation => 3,
            Self::CloseoutPostcondition => 4,
            Self::ReviewRequestCreation => 5,
        }
    }
}

/// The exact result of checking whether a minimal POLICY declares one caller-selected context.
///
/// `Declared` is not a Policy applicability, satisfaction, authority, admission, Scope,
/// lifecycle, or external-truth conclusion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MinimalPolicyContextDeclaration {
    Declared,
    ContextUnsupported,
}

impl MinimalPolicyRecord {
    /// Strictly decodes only the unambiguous, zero-optional-requirement POLICY
    /// grammar. POLICY records with optional requirement fields remain outside
    /// this bounded local decoder rather than receiving invented semantics.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let frame = StrictRecordFrame::decode_authoritative(input)?;
        if frame.record_type_id() != RecordTypeId::try_from(40).expect("assigned Record Type") {
            return Err(RecordDecodeError);
        }
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 40
            || cursor.uint().map_err(|_| RecordDecodeError)? != 1
        {
            return Err(RecordDecodeError);
        }
        cursor.map_exact(5).map_err(|_| RecordDecodeError)?;
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 40 {
            return Err(RecordDecodeError);
        }
        cursor.key(16).map_err(|_| RecordDecodeError)?;
        let gate_scope_ref =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(29).map_err(|_| RecordDecodeError)?;
        let operation_start_journal_ref =
            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?;
        cursor.key(30).map_err(|_| RecordDecodeError)?;
        let context_count = cursor.array().map_err(|_| RecordDecodeError)?;
        if context_count == 0 || context_count > cursor.remaining() {
            return Err(RecordDecodeError);
        }
        let mut supported_context_ids = Vec::with_capacity(context_count);
        let mut previous_context_id = None;
        for _ in 0..context_count {
            let context_id = cursor.uint().map_err(|_| RecordDecodeError)?;
            if !matches!(context_id, 1..=5)
                || previous_context_id.is_some_and(|previous| context_id <= previous)
            {
                return Err(RecordDecodeError);
            }
            previous_context_id = Some(context_id);
            supported_context_ids.push(context_id);
        }
        if supported_context_ids.contains(&2)
            || supported_context_ids.contains(&4)
            || supported_context_ids.contains(&5)
            || !cursor.finished()
        {
            return Err(RecordDecodeError);
        }
        Ok(Self {
            record_id: frame.record_id(),
            gate_scope_ref,
            operation_start_journal_ref,
            supported_context_ids,
        })
    }

    /// The exact immutable identity of the strictly decoded POLICY Record.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// The frozen Record Type ID for POLICY.
    pub fn record_type_id(&self) -> RecordTypeId {
        RecordTypeId::try_from(40).expect("POLICY is assigned in Record Schema v0.3")
    }

    /// The exact declared gate Scope Record identity, without resolving it.
    pub fn gate_scope_ref(&self) -> RecordId {
        self.gate_scope_ref
    }

    /// The exact locally framed operation-start Journal reference, without resolving it.
    pub fn operation_start_journal_ref(&self) -> &JournalReference {
        &self.operation_start_journal_ref
    }

    /// The exact ordered supported-context IDs declared by this minimal Policy.
    pub fn supported_context_ids(&self) -> &[u64] {
        &self.supported_context_ids
    }

    /// Whether this exact minimal Policy explicitly declares FREEZE_COMMIT support.
    pub fn declares_freeze_commit_support(&self) -> bool {
        self.supported_context_ids.contains(&1)
    }
}

/// Checks only the explicit caller-selected context membership of a minimal POLICY.
///
/// The caller supplies the operation's context. This deliberately does not infer a context from
/// POLICY fields, evaluate the declared gate Scope, establish Policy applicability or
/// satisfaction, validate authority, or make an admission decision.
pub fn check_minimal_policy_context_declaration(
    policy: &MinimalPolicyRecord,
    context: PolicyEvaluationContext,
) -> MinimalPolicyContextDeclaration {
    if policy.supported_context_ids().contains(&context.id()) {
        MinimalPolicyContextDeclaration::Declared
    } else {
        MinimalPolicyContextDeclaration::ContextUnsupported
    }
}

/// One exact Review selector requirement from a POLICY Record evaluated only
/// in the frozen REVIEW_ADMISSION context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewAdmissionReviewRequirement {
    review_role_id: u64,
    review_scope_ref: RecordId,
    review_method_ref: RecordId,
    required_checks_ref: RecordId,
    required_count: u64,
}

impl ReviewAdmissionReviewRequirement {
    /// The exact required Review role identity.
    pub fn review_role_id(&self) -> u64 {
        self.review_role_id
    }

    /// The exact required Review Scope identity.
    pub fn review_scope_ref(&self) -> RecordId {
        self.review_scope_ref
    }

    /// The exact required Review Method identity.
    pub fn review_method_ref(&self) -> RecordId {
        self.review_method_ref
    }

    /// The exact required CheckSet identity.
    pub fn required_checks_ref(&self) -> RecordId {
        self.required_checks_ref
    }

    /// The required Review count. This field is not applicable to REVIEW_ADMISSION.
    pub fn required_count(&self) -> u64 {
        self.required_count
    }
}

/// The exact local POLICY fields which are applicable to REVIEW_ADMISSION.
///
/// This decoder implements the frozen v0.3 Policy body grammar together with
/// the v0.4 profile-1 prerequisite shape. A successful decode establishes only
/// local Policy grammar and exact Record identity; it does not establish Policy
/// authority, §82 completion, evaluator completion, satisfaction, Admission,
/// or a terminal event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewAdmissionPolicyRecord {
    record_id: RecordId,
    gate_scope_ref: RecordId,
    review_requirements: Vec<ReviewAdmissionReviewRequirement>,
    required_method_statuses: Option<Vec<u64>>,
    allowed_finding_states: Option<Vec<u64>>,
    acceptable_anchor_relation_ids: Option<Vec<u64>>,
    operation_start_journal_ref: JournalReference,
    supported_context_ids: Vec<u64>,
}

fn decode_nonempty_sorted_uint_set(
    cursor: &mut CborCursor<'_>,
    allowed: impl Fn(u64) -> bool,
) -> Result<Vec<u64>, RecordDecodeError> {
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count == 0 || count > cursor.remaining() {
        return Err(RecordDecodeError);
    }
    let mut values = Vec::with_capacity(count);
    let mut previous = None;
    for _ in 0..count {
        let value = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !allowed(value) || previous.is_some_and(|prior| value <= prior) {
            return Err(RecordDecodeError);
        }
        previous = Some(value);
        values.push(value);
    }
    Ok(values)
}

fn decode_review_admission_review_requirements(
    cursor: &mut CborCursor<'_>,
) -> Result<Vec<ReviewAdmissionReviewRequirement>, RecordDecodeError> {
    let count = cursor.array().map_err(|_| RecordDecodeError)?;
    if count == 0 || count > cursor.remaining() / 5 {
        return Err(RecordDecodeError);
    }
    let mut requirements = Vec::with_capacity(count);
    let mut previous: Option<(u64, RecordId, RecordId, RecordId, u64)> = None;
    for _ in 0..count {
        cursor.map_exact(5).map_err(|_| RecordDecodeError)?;
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        let review_role_id = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !matches!(review_role_id, 1..=5) {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        let review_scope_ref =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(2).map_err(|_| RecordDecodeError)?;
        let review_method_ref =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(3).map_err(|_| RecordDecodeError)?;
        let required_checks_ref =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(4).map_err(|_| RecordDecodeError)?;
        let required_count = cursor.uint().map_err(|_| RecordDecodeError)?;
        let selector = (
            review_role_id,
            review_scope_ref,
            review_method_ref,
            required_checks_ref,
            required_count,
        );
        if previous.as_ref().is_some_and(|prior| {
            selector <= *prior
                || (selector.0, selector.1, selector.2, selector.3)
                    == (prior.0, prior.1, prior.2, prior.3)
        }) {
            return Err(RecordDecodeError);
        }
        previous = Some(selector);
        requirements.push(ReviewAdmissionReviewRequirement {
            review_role_id,
            review_scope_ref,
            review_method_ref,
            required_checks_ref,
            required_count,
        });
    }
    Ok(requirements)
}

impl ReviewAdmissionPolicyRecord {
    /// Strictly decodes the complete subset of frozen POLICY requirements that
    /// is mechanically applicable to the v0.4 profile-1 REVIEW_ADMISSION path.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let frame = StrictRecordFrame::decode_authoritative(input)?;
        if frame.record_type_id() != RecordTypeId::try_from(40).expect("assigned Record Type") {
            return Err(RecordDecodeError);
        }
        authoritative_store::validate_policy_record_schema(input)?;
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 40
            || cursor.uint().map_err(|_| RecordDecodeError)? != 1
        {
            return Err(RecordDecodeError);
        }
        let field_count = cursor.map().map_err(|_| RecordDecodeError)?;
        if !(6..=17).contains(&field_count) {
            return Err(RecordDecodeError);
        }
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 40 {
            return Err(RecordDecodeError);
        }
        cursor.key(16).map_err(|_| RecordDecodeError)?;
        let gate_scope_ref =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        let mut review_requirements = None;
        let mut required_method_statuses = None;
        let mut allowed_finding_states = None;
        let mut acceptable_anchor_relation_ids = None;
        let mut operation_start_journal_ref = None;
        let mut supported_context_ids = None;
        let mut previous_key = 16;
        for _ in 3..field_count {
            let key = cursor.uint().map_err(|_| RecordDecodeError)?;
            if key <= previous_key {
                return Err(RecordDecodeError);
            }
            previous_key = key;
            match key {
                17 if review_requirements.is_none() => {
                    review_requirements =
                        Some(decode_review_admission_review_requirements(&mut cursor)?);
                }
                18 if required_method_statuses.is_none() => {
                    required_method_statuses =
                        Some(decode_nonempty_sorted_uint_set(&mut cursor, |id| {
                            matches!(id, 1..=3)
                        })?);
                }
                19 if allowed_finding_states.is_none() => {
                    allowed_finding_states =
                        Some(decode_nonempty_sorted_uint_set(&mut cursor, |id| {
                            matches!(id, 1..=3)
                        })?);
                }
                24 if acceptable_anchor_relation_ids.is_none() => {
                    cursor.map_exact(1).map_err(|_| RecordDecodeError)?;
                    cursor.key(0).map_err(|_| RecordDecodeError)?;
                    acceptable_anchor_relation_ids =
                        Some(decode_nonempty_sorted_uint_set(&mut cursor, |id| {
                            matches!(id, 1..=6)
                        })?);
                }
                20..=23 | 25..=28 => {
                    cursor.skip_value().map_err(|_| RecordDecodeError)?;
                }
                29 if operation_start_journal_ref.is_none() => {
                    operation_start_journal_ref =
                        Some(decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?);
                }
                30 if supported_context_ids.is_none() => {
                    supported_context_ids =
                        Some(decode_nonempty_sorted_uint_set(&mut cursor, |id| {
                            matches!(id, 1..=5)
                        })?);
                }
                _ => return Err(RecordDecodeError),
            }
        }
        let review_requirements = review_requirements.ok_or(RecordDecodeError)?;
        let operation_start_journal_ref = operation_start_journal_ref.ok_or(RecordDecodeError)?;
        let supported_context_ids = supported_context_ids.ok_or(RecordDecodeError)?;
        if !cursor.finished() {
            return Err(RecordDecodeError);
        }
        Ok(Self {
            record_id: frame.record_id(),
            gate_scope_ref,
            review_requirements,
            required_method_statuses,
            allowed_finding_states,
            acceptable_anchor_relation_ids,
            operation_start_journal_ref,
            supported_context_ids,
        })
    }

    /// The exact self-hash identity of the strictly decoded POLICY bytes.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// The exact gate Scope identity declared by POLICY key 16.
    pub fn gate_scope_ref(&self) -> RecordId {
        self.gate_scope_ref
    }

    /// The caller-selected Policy evaluation context for this bounded evaluator path.
    pub fn supported_context(&self) -> PolicyEvaluationContext {
        PolicyEvaluationContext::ReviewAdmission
    }

    /// The exact ordered Policy context declarations from key 30.
    pub fn supported_context_ids(&self) -> &[u64] {
        &self.supported_context_ids
    }

    /// The exact nonempty Review selector set required for REVIEW_ADMISSION.
    pub fn review_requirements(&self) -> &[ReviewAdmissionReviewRequirement] {
        &self.review_requirements
    }

    /// Optional global Method-status constraint applicable to REVIEW_RESULT.
    pub fn required_method_statuses(&self) -> &[u64] {
        self.required_method_statuses.as_deref().unwrap_or(&[])
    }

    /// Optional global Finding-state constraint applicable to REVIEW_RESULT.
    pub fn allowed_finding_states(&self) -> &[u64] {
        self.allowed_finding_states.as_deref().unwrap_or(&[])
    }

    /// Optional acceptable Journal-Anchor relation set.
    pub fn acceptable_anchor_relation_ids(&self) -> &[u64] {
        self.acceptable_anchor_relation_ids
            .as_deref()
            .unwrap_or(&[])
    }

    /// The exact locally decoded Policy-registration chronology reference.
    pub fn operation_start_journal_ref(&self) -> &JournalReference {
        &self.operation_start_journal_ref
    }
}

/// The typed pre-loop context-declaration result for an authoritative
/// Review-Admission Policy Record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionPolicyContextDeclaration {
    Declared,
    PolicyContextUnsupported,
}

/// Requires an explicit REVIEW_ADMISSION declaration before §46 requirement
/// evaluation. Other explicitly declared contexts remain separate and do not
/// become hidden gates for this operation.
pub fn check_review_admission_policy_context(
    policy: &ReviewAdmissionPolicyRecord,
) -> ReviewAdmissionPolicyContextDeclaration {
    if policy
        .supported_context_ids()
        .contains(&PolicyEvaluationContext::ReviewAdmission.id())
    {
        ReviewAdmissionPolicyContextDeclaration::Declared
    } else {
        ReviewAdmissionPolicyContextDeclaration::PolicyContextUnsupported
    }
}

/// Exact retained-history facts binding a REVIEW_ADMISSION Policy to its
/// `POLICY_RECORDED` event and strictly-prior registration chronology.
///
/// This is only a structural retained-history binding. It does not establish
/// Policy authority, §82 prerequisites, §46 completion, Policy satisfaction,
/// Admission, an event 302/303, or publication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewAdmissionPolicyRecordedBinding {
    policy_record_id: RecordId,
    policy_event_reference: JournalReference,
    operation_start_reference: JournalReference,
}

impl ReviewAdmissionPolicyRecordedBinding {
    /// The exact self-hash identity of the bound Policy Record.
    pub fn policy_record_id(&self) -> RecordId {
        self.policy_record_id
    }

    /// The exact retained `POLICY_RECORDED` event reference.
    pub fn policy_event_reference(&self) -> &JournalReference {
        &self.policy_event_reference
    }

    /// The exact strictly-prior registration chronology reference.
    pub fn operation_start_reference(&self) -> &JournalReference {
        &self.operation_start_reference
    }
}

/// Fail-closed outcomes while binding a REVIEW_ADMISSION Policy registration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionPolicyRecordedBindingError {
    /// The supplied Policy bytes fail the bounded Review-Admission Policy grammar.
    PolicyDecode,
    /// The supplied Policy event reference is absent or differs from retained history.
    PolicyEventReference(RetainedJournalError),
    /// The resolved retained entry is not a `POLICY_RECORDED` Policy event.
    PolicyEventMismatch,
    /// The Policy bytes and retained event Record identity differ.
    PolicyEventRecordIdentityMismatch,
    /// The declared registration chronology reference is absent or differs from retained history.
    OperationStartReference(RetainedJournalError),
    /// The declared registration chronology reference is not strictly prior to Policy recording.
    OperationStartNotStrictlyPrior,
}

/// Structurally binds exact REVIEW_ADMISSION Policy bytes to one retained
/// `POLICY_RECORDED` entry.
pub fn validate_review_admission_policy_recorded_binding(
    retained_journal: &RetainedJournal,
    policy_event_reference: &JournalReference,
    policy_bytes: &[u8],
) -> Result<ReviewAdmissionPolicyRecordedBinding, ReviewAdmissionPolicyRecordedBindingError> {
    let policy = ReviewAdmissionPolicyRecord::decode_authoritative(policy_bytes)
        .map_err(|_| ReviewAdmissionPolicyRecordedBindingError::PolicyDecode)?;
    let policy_event = retained_journal
        .resolve_reference(policy_event_reference)
        .map_err(ReviewAdmissionPolicyRecordedBindingError::PolicyEventReference)?;
    if policy_event.event_type_id().value() != 400
        || policy_event.event_type_id().required_record_type_id()
            != RecordTypeId::try_from(40).expect("POLICY is assigned in Record Schema v0.3")
        || policy_event.lifecycle_object_kind() != LifecycleObjectKind::Policy
        || policy_event.lifecycle_object_id() != *policy_event.event_record_id().as_bytes()
    {
        return Err(ReviewAdmissionPolicyRecordedBindingError::PolicyEventMismatch);
    }
    if policy_event.event_record_id().as_bytes() != policy.record_id().as_bytes() {
        return Err(ReviewAdmissionPolicyRecordedBindingError::PolicyEventRecordIdentityMismatch);
    }
    let operation_start = retained_journal
        .resolve_reference(policy.operation_start_journal_ref())
        .map_err(ReviewAdmissionPolicyRecordedBindingError::OperationStartReference)?;
    if operation_start.entry_index().value() >= policy_event.entry_index().value() {
        return Err(ReviewAdmissionPolicyRecordedBindingError::OperationStartNotStrictlyPrior);
    }
    Ok(ReviewAdmissionPolicyRecordedBinding {
        policy_record_id: policy.record_id(),
        policy_event_reference: policy_event_reference.clone(),
        operation_start_reference: policy.operation_start_journal_ref().clone(),
    })
}

/// Typed, Record-local fields decoded from a SCOPE Record.
///
/// Profile IDs, versions, payload bytes, and labels are retained exactly as declared.
/// Their profile semantics, applicability, authority, and authorization remain external.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeRecordInput {
    pub scope_profile_id: u64,
    pub scope_profile_version: u64,
    pub scope_payload: Vec<u8>,
    pub scope_label: Option<String>,
}

/// The exact-byte, Record-local SCOPE schema from Record Schema v0.3 §47.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeRecord {
    record_id: RecordId,
    input: ScopeRecordInput,
}

impl ScopeRecord {
    /// Strictly decodes the complete local SCOPE grammar without interpreting its profile.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let frame = StrictRecordFrame::decode_authoritative(input)?;
        if frame.record_type_id() != RecordTypeId::try_from(20).expect("assigned Record Type") {
            return Err(RecordDecodeError);
        }
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 20
            || cursor.uint().map_err(|_| RecordDecodeError)? != 1
        {
            return Err(RecordDecodeError);
        }
        let field_count = cursor.map().map_err(|_| RecordDecodeError)?;
        if !matches!(field_count, 5 | 6) {
            return Err(RecordDecodeError);
        }
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 20 {
            return Err(RecordDecodeError);
        }
        cursor.key(16).map_err(|_| RecordDecodeError)?;
        let scope_profile_id = cursor.uint().map_err(|_| RecordDecodeError)?;
        cursor.key(17).map_err(|_| RecordDecodeError)?;
        let scope_profile_version = cursor.uint().map_err(|_| RecordDecodeError)?;
        cursor.key(18).map_err(|_| RecordDecodeError)?;
        let scope_payload = cursor.bstr().map_err(|_| RecordDecodeError)?;
        let scope_label = if field_count == 6 {
            cursor.key(19).map_err(|_| RecordDecodeError)?;
            Some(cursor.text().map_err(|_| RecordDecodeError)?)
        } else {
            None
        };
        if !cursor.finished() {
            return Err(RecordDecodeError);
        }
        Ok(Self {
            record_id: frame.record_id(),
            input: ScopeRecordInput {
                scope_profile_id,
                scope_profile_version,
                scope_payload,
                scope_label,
            },
        })
    }

    /// The exact immutable identity of the strictly decoded SCOPE Record.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// The frozen Record Type ID for SCOPE.
    pub fn record_type_id(&self) -> RecordTypeId {
        RecordTypeId::try_from(20).expect("SCOPE is assigned in Record Schema v0.3")
    }

    /// Returns exact local SCOPE fields without profile or authority inference.
    pub fn input(&self) -> &ScopeRecordInput {
        &self.input
    }
}

/// Fail-closed outcomes while resolving the v0.4 profile-1 exact Review
/// Admission Scope representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionExactScopeProfileError {
    ScopePayloadUnavailable,
    ScopeDecode,
    ScopeIdentityMismatch,
    UnsupportedProfile,
}

/// Resolves one exact typed SCOPE Record selected by the frozen v0.4
/// `REVIEW_ADMISSION_EXACT_REVIEW_SCOPE_BINDING` profile.
///
/// This validates profile `(1, 1)` and empty payload only. It does not infer
/// Scope coverage, authority, Policy satisfaction, Admission, or a terminal
/// lifecycle result.
pub fn resolve_review_admission_exact_scope_profile(
    scope_ref: RecordId,
    resolver: &impl ExactRecordByteResolver,
) -> Result<ScopeRecord, ReviewAdmissionExactScopeProfileError> {
    let scope_bytes = resolver
        .resolve(scope_ref)
        .ok_or(ReviewAdmissionExactScopeProfileError::ScopePayloadUnavailable)?;
    let scope = ScopeRecord::decode_authoritative(scope_bytes)
        .map_err(|_| ReviewAdmissionExactScopeProfileError::ScopeDecode)?;
    if scope.record_id() != scope_ref {
        return Err(ReviewAdmissionExactScopeProfileError::ScopeIdentityMismatch);
    }
    if scope.input().scope_profile_id != 1
        || scope.input().scope_profile_version != 1
        || !scope.input().scope_payload.is_empty()
    {
        return Err(ReviewAdmissionExactScopeProfileError::UnsupportedProfile);
    }
    Ok(scope)
}

/// Exact structural facts established before the v0.4 profile-specific
/// Review Admission gate-Scope evaluator may be considered.
///
/// These facts are deliberately pre-evaluator only. They do not establish
/// Policy authority, completion under §46, Policy satisfaction, Admission, a
/// terminal disposition, or Journal publication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReviewAdmissionPolicyContextPrerequisites {
    policy_record_id: RecordId,
    gate_scope_ref: RecordId,
    common_review_scope_ref: RecordId,
}

impl ReviewAdmissionPolicyContextPrerequisites {
    /// The exact strictly decoded Policy identity bound to retained history.
    pub fn policy_record_id(&self) -> RecordId {
        self.policy_record_id
    }

    /// The exact Policy-declared profile-1 gate Scope identity.
    pub fn gate_scope_ref(&self) -> RecordId {
        self.gate_scope_ref
    }

    /// The exact common Request/Result profile-1 Review Scope identity.
    pub fn common_review_scope_ref(&self) -> RecordId {
        self.common_review_scope_ref
    }
}

/// Fail-closed outcomes while establishing the v0.4 profile-specific
/// Review Admission Policy context prerequisites.
///
/// None of these variants is an evaluator result or a completed Policy result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionPolicyContextPrerequisitesError {
    /// Exact Policy bytes did not bind to the retained `POLICY_RECORDED` event.
    PolicyBinding(ReviewAdmissionPolicyRecordedBindingError),
    /// The exact authoritative Policy does not declare REVIEW_ADMISSION.
    PolicyContextUnsupported,
    /// The common Request/Result §82 identity prerequisite did not hold.
    CommonRequestResult(ReviewAdmissionCommonRequestResultError),
    /// The Policy-declared gate Scope was unavailable, malformed, mismatched, or unsupported.
    GateScope(ReviewAdmissionExactScopeProfileError),
    /// The common Request/Result Review Scope was unavailable, malformed, mismatched, or unsupported.
    CommonReviewScope(ReviewAdmissionExactScopeProfileError),
}

/// Establishes only the frozen v0.4 profile-specific structural prerequisites
/// for a later Review Admission Policy evaluator.
///
/// This function first binds the exact Review Admission Policy to retained
/// history, then requires the exact common Request/Result §82 identity facts,
/// and finally resolves both named Scope identities as typed profile-1 empty
/// payload SCOPE Records. It deliberately performs no evaluator dispatch, §46
/// composition, Admission disposition selection, event construction, or
/// Journal publication.
pub fn validate_review_admission_policy_context_prerequisites(
    retained_journal: &RetainedJournal,
    policy_event_reference: &JournalReference,
    policy_bytes: &[u8],
    request_bytes: &[u8],
    result_bytes: &[u8],
    resolver: &impl ExactRecordByteResolver,
) -> Result<ReviewAdmissionPolicyContextPrerequisites, ReviewAdmissionPolicyContextPrerequisitesError>
{
    let policy_binding = validate_review_admission_policy_recorded_binding(
        retained_journal,
        policy_event_reference,
        policy_bytes,
    )
    .map_err(ReviewAdmissionPolicyContextPrerequisitesError::PolicyBinding)?;
    let common_review_scope_ref =
        validate_review_admission_common_request_result_fields(request_bytes, result_bytes)
            .map_err(ReviewAdmissionPolicyContextPrerequisitesError::CommonRequestResult)?;
    let policy = ReviewAdmissionPolicyRecord::decode_authoritative(policy_bytes).map_err(|_| {
        ReviewAdmissionPolicyContextPrerequisitesError::PolicyBinding(
            ReviewAdmissionPolicyRecordedBindingError::PolicyDecode,
        )
    })?;
    if check_review_admission_policy_context(&policy)
        == ReviewAdmissionPolicyContextDeclaration::PolicyContextUnsupported
    {
        return Err(ReviewAdmissionPolicyContextPrerequisitesError::PolicyContextUnsupported);
    }
    let gate_scope_ref = policy.gate_scope_ref();
    resolve_review_admission_exact_scope_profile(gate_scope_ref, resolver)
        .map_err(ReviewAdmissionPolicyContextPrerequisitesError::GateScope)?;
    resolve_review_admission_exact_scope_profile(common_review_scope_ref, resolver)
        .map_err(ReviewAdmissionPolicyContextPrerequisitesError::CommonReviewScope)?;
    Ok(ReviewAdmissionPolicyContextPrerequisites {
        policy_record_id: policy_binding.policy_record_id(),
        gate_scope_ref,
        common_review_scope_ref,
    })
}

/// Immutable registry metadata for a frozen POLICY evaluator.
///
/// This metadata assigns no generic evaluation behavior to any field or Scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicyEvaluatorRegistration {
    pub id: u16,
    pub name: &'static str,
}

const POLICY_EVALUATOR_REGISTRY: [PolicyEvaluatorRegistration; 14] = [
    PolicyEvaluatorRegistration {
        id: 1001,
        name: "POLICY_REVIEW_REQUIREMENT_MATCH",
    },
    PolicyEvaluatorRegistration {
        id: 1002,
        name: "POLICY_REVIEW_REQUIRED_COUNT",
    },
    PolicyEvaluatorRegistration {
        id: 1003,
        name: "POLICY_REQUIRED_METHOD_STATUS",
    },
    PolicyEvaluatorRegistration {
        id: 1004,
        name: "POLICY_ALLOWED_FINDING_STATE",
    },
    PolicyEvaluatorRegistration {
        id: 1005,
        name: "POLICY_VERIFICATION_REQUIREMENT",
    },
    PolicyEvaluatorRegistration {
        id: 1006,
        name: "POLICY_INTERVENING_EVENT_CONSTRAINT",
    },
    PolicyEvaluatorRegistration {
        id: 1007,
        name: "POLICY_MAX_JOURNAL_DISTANCE",
    },
    PolicyEvaluatorRegistration {
        id: 1008,
        name: "POLICY_MINIMUM_DURABILITY",
    },
    PolicyEvaluatorRegistration {
        id: 1009,
        name: "POLICY_JOURNAL_ANCHOR_REQUIREMENT",
    },
    PolicyEvaluatorRegistration {
        id: 1010,
        name: "POLICY_BOOTSTRAP_SCOPE_REQUIREMENT",
    },
    PolicyEvaluatorRegistration {
        id: 1011,
        name: "POLICY_FORMAL_FINDING_CLASSIFICATION_REQUIREMENT",
    },
    PolicyEvaluatorRegistration {
        id: 1012,
        name: "POLICY_ESTABLISHMENT_DIVERSITY_REQUIREMENT",
    },
    PolicyEvaluatorRegistration {
        id: 1013,
        name: "POLICY_CLOSEOUT_POSTCONDITION",
    },
    PolicyEvaluatorRegistration {
        id: 1014,
        name: "POLICY_SUPPORTED_CONTEXT",
    },
];

/// Registry metadata for immutable POLICY evaluator assignments.
pub fn policy_evaluator_registry() -> &'static [PolicyEvaluatorRegistration] {
    &POLICY_EVALUATOR_REGISTRY
}

/// A fail-closed §82 authority, identity, or chronology failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionSection82AuthorityError {
    RetainedAnchorInput(RetainedReviewPackageAnchorInputError),
    ReturnedAnchorComparison(JournalAnchorHistoryComparison),
    RequestDecode,
    ResultDecode,
    CommonRequestResult(ReviewAdmissionCommonRequestResultError),
    RequestIdentityDependencies(RetainedJournalError),
    RequestIdentityDependencySetMismatch,
    RequestAuthorityDependencies(RetainedJournalError),
    RequestAuthorityDependencySetMismatch,
    ResultAuthorityDependencies(RetainedJournalError),
    ResultAuthorityDependencySetMismatch,
    PolicyAuthorityReference(RetainedJournalError),
    PolicyAuthorityEventMismatch,
    PolicyPayloadUnavailable,
    PolicyDecode,
    PolicyIdentityMismatch,
    /// The exact retained Policy chronology, context declaration, or named Scope inputs failed.
    PolicyContextPrerequisites(ReviewAdmissionPolicyContextPrerequisitesError),
    RequestOperationStartReference(RetainedJournalError),
    RequestOperationStartNotStrictlyPrior,
    ResultOperationStartReference(RetainedJournalError),
    ResultOperationStartNotStrictlyPrior,
    FreezeBinding(ResolvedFreezeCommittedBindingError),
    /// Exact retained topology and purported Record bytes do not establish
    /// authoritative Journal/store provenance or an authoritative Record namespace.
    FreezeAuthorityEvidenceUnavailable,
    FreezeReceiptPayloadUnavailable,
    FreezeReceiptDecode,
    FreezeReceiptIdentityMismatch,
    FreezeManifestMismatch,
}

/// Checks every exact retained §82 input available through the current
/// structural Journal and exact-byte resolver seams.
///
/// Equal Request/Result fields alone are insufficient. This path also checks the
/// exact Request and Result events, their chronology, the returned Anchor, and
/// the retained FREEZE_COMMITTED Receipt/START/MANIFEST topology. The current
/// inputs cannot prove authoritative Journal/store provenance or an authoritative
/// Record namespace. A successful return establishes only that every available
/// structural prerequisite was checked; callers must still stop preterminally at
/// `FreezeAuthorityEvidenceUnavailable` before Policy evaluation.
fn validate_review_admission_section_82_structural_inputs(
    retained_journal: &RetainedJournal,
    request_event_reference: &JournalReference,
    request_bytes: &[u8],
    result_event_reference: &JournalReference,
    result_bytes: &[u8],
    resolver: &impl ExactRecordByteResolver,
) -> Result<(), ReviewAdmissionSection82AuthorityError> {
    let anchor = resolve_retained_review_package_anchor_input(
        retained_journal,
        request_event_reference,
        request_bytes,
        result_event_reference,
        result_bytes,
    )
    .map_err(ReviewAdmissionSection82AuthorityError::RetainedAnchorInput)?;
    match compare_retained_journal_anchor_history(retained_journal, &anchor) {
        JournalAnchorHistoryComparison::AnchorEqualsCurrentHead
        | JournalAnchorHistoryComparison::AnchorIsValidAncestor => {}
        failure => {
            return Err(ReviewAdmissionSection82AuthorityError::ReturnedAnchorComparison(failure));
        }
    }
    let request = ReviewRequestRecord::decode_authoritative(request_bytes)
        .map_err(|_| ReviewAdmissionSection82AuthorityError::RequestDecode)?;
    let result = ReviewResultRecord::decode_authoritative(result_bytes)
        .map_err(|_| ReviewAdmissionSection82AuthorityError::ResultDecode)?;
    validate_review_admission_common_request_result_fields(request_bytes, result_bytes)
        .map_err(ReviewAdmissionSection82AuthorityError::CommonRequestResult)?;

    let request_identity_dependencies = retained_journal
        .identity_dependencies_for_reference(request_event_reference)
        .map_err(ReviewAdmissionSection82AuthorityError::RequestIdentityDependencies)?;
    let expected_request_identity_dependencies = [
        IdentityDependency::RecordId(request.manifest_id()),
        IdentityDependency::RecordId(request.required_checks_ref()),
        IdentityDependency::RecordId(request.review_scope_ref()),
        IdentityDependency::RecordId(request.review_method_ref()),
        IdentityDependency::JournalAnchorId(request.review_package_anchor_id()),
    ];
    if request_identity_dependencies.len() != expected_request_identity_dependencies.len()
        || expected_request_identity_dependencies
            .iter()
            .any(|dependency| !request_identity_dependencies.contains(dependency))
    {
        return Err(ReviewAdmissionSection82AuthorityError::RequestIdentityDependencySetMismatch);
    }
    let request_authority_dependencies = retained_journal
        .authority_dependencies_for_reference(request_event_reference)
        .map_err(ReviewAdmissionSection82AuthorityError::RequestAuthorityDependencies)?;
    if request_authority_dependencies.len() != 2
        || !request_authority_dependencies.contains(request.freeze_authority_ref())
        || !request_authority_dependencies.contains(request.policy_authority_ref())
    {
        return Err(ReviewAdmissionSection82AuthorityError::RequestAuthorityDependencySetMismatch);
    }
    let result_authority_dependencies = retained_journal
        .authority_dependencies_for_reference(result_event_reference)
        .map_err(ReviewAdmissionSection82AuthorityError::ResultAuthorityDependencies)?;
    if result_authority_dependencies != [request_event_reference.clone()] {
        return Err(ReviewAdmissionSection82AuthorityError::ResultAuthorityDependencySetMismatch);
    }

    let policy_authority = retained_journal
        .resolve_reference(request.policy_authority_ref())
        .map_err(ReviewAdmissionSection82AuthorityError::PolicyAuthorityReference)?;
    if policy_authority.event_type_id().value() != 400
        || policy_authority
            .event_type_id()
            .required_record_type_id()
            .value()
            != 40
        || policy_authority.lifecycle_object_kind() != LifecycleObjectKind::Policy
        || policy_authority.lifecycle_object_id() != *policy_authority.event_record_id().as_bytes()
    {
        return Err(ReviewAdmissionSection82AuthorityError::PolicyAuthorityEventMismatch);
    }
    let policy_record_id =
        RecordId::try_from(policy_authority.event_record_id().as_bytes().as_slice())
            .expect("EventRecordId has RecordId width");
    let policy_bytes = resolver
        .resolve(policy_record_id)
        .ok_or(ReviewAdmissionSection82AuthorityError::PolicyPayloadUnavailable)?;
    let policy = ReviewAdmissionPolicyRecord::decode_authoritative(policy_bytes)
        .map_err(|_| ReviewAdmissionSection82AuthorityError::PolicyDecode)?;
    if policy.record_id() != policy_record_id {
        return Err(ReviewAdmissionSection82AuthorityError::PolicyIdentityMismatch);
    }
    let request_operation_start = retained_journal
        .resolve_reference(request.operation_start_journal_ref())
        .map_err(ReviewAdmissionSection82AuthorityError::RequestOperationStartReference)?;
    if request_operation_start.entry_index().value()
        >= request_event_reference.entry_index().value()
    {
        return Err(ReviewAdmissionSection82AuthorityError::RequestOperationStartNotStrictlyPrior);
    }
    let result_operation_start = retained_journal
        .resolve_reference(result.operation_start_journal_ref())
        .map_err(ReviewAdmissionSection82AuthorityError::ResultOperationStartReference)?;
    if result_operation_start.entry_index().value() >= result_event_reference.entry_index().value()
    {
        return Err(ReviewAdmissionSection82AuthorityError::ResultOperationStartNotStrictlyPrior);
    }
    validate_review_admission_policy_context_prerequisites(
        retained_journal,
        request.policy_authority_ref(),
        policy_bytes,
        request_bytes,
        result_bytes,
        resolver,
    )
    .map_err(ReviewAdmissionSection82AuthorityError::PolicyContextPrerequisites)?;

    match validate_resolved_freeze_committed_binding(
        retained_journal,
        request.freeze_authority_ref().clone(),
        resolver,
    )
    .map_err(ReviewAdmissionSection82AuthorityError::FreezeBinding)?
    {
        ResolvedFreezeCommittedBindingOutcome::AuthorityEvidenceUnavailable => {}
    }
    let receipt_record_id = RecordId::try_from(
        request
            .freeze_authority_ref()
            .event_record_id()
            .as_bytes()
            .as_slice(),
    )
    .expect("EventRecordId has RecordId width");
    let receipt_bytes = resolver
        .resolve(receipt_record_id)
        .ok_or(ReviewAdmissionSection82AuthorityError::FreezeReceiptPayloadUnavailable)?;
    let receipt = FreezeReceiptRecord::decode_authoritative(receipt_bytes)
        .map_err(|_| ReviewAdmissionSection82AuthorityError::FreezeReceiptDecode)?;
    if receipt.record_id() != receipt_record_id {
        return Err(ReviewAdmissionSection82AuthorityError::FreezeReceiptIdentityMismatch);
    }
    if receipt.input().manifest_id != request.manifest_id() {
        return Err(ReviewAdmissionSection82AuthorityError::FreezeManifestMismatch);
    }

    Ok(())
}

/// A §82 returned-Anchor comparison that prevents Policy evaluation from starting.
///
/// This is deliberately outside unavailable generic gate-Scope evaluation and §46's
/// completed-result grammar.
/// It establishes no Review Request/Result authority, Admission result, terminal
/// event, Journal publication, or generic Policy semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionSection82PrerequisiteFailure {
    /// Full retained Request/Result/Freeze/Manifest authority input did not bind.
    AuthorityInput(ReviewAdmissionSection82AuthorityError),
    /// The presented Request authority was recorded after this operation's immutable `L(A)`.
    RequestAuthorityAfterOperationStart,
    /// The presented Result authority was recorded after this operation's immutable `L(A)`.
    ResultAuthorityAfterOperationStart,
    /// Strict Request/Result-to-retained-Anchor recovery did not establish a
    /// §82 returned-Anchor input. This is not an evaluator or Policy result.
    RetainedAnchorInput(RetainedReviewPackageAnchorInputError),
    ReturnedAnchorComparison(JournalAnchorHistoryComparison),
}

/// The routing boundary between successful §82 returned-Anchor comparison and
/// subsequent Policy evaluation.
///
/// `PreTerminal` is not an accepted or rejected Admission disposition and must
/// not produce event 302 or 303. `PolicyRouteEligible` is only a retained
/// structural handoff point; it neither evaluates Policy nor represents §46,
/// Admission, a terminal event, or Journal publication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionSection82RoutingOutcome {
    PreTerminal(ReviewAdmissionSection82PrerequisiteFailure),
    PolicyRouteEligible,
}

/// Routes a completed retained-Journal Anchor comparison before Policy work.
///
/// Lifecycle v0.10.2 §117 admits only an equal current head or valid ancestor
/// to the subsequent Policy path. All other §116 classes fail closed at this
/// §82 boundary without invoking generic gate-Scope evaluation, §46
/// composition, or terminal-event publication.
pub fn route_review_admission_after_anchor_comparison(
    comparison: JournalAnchorHistoryComparison,
    on_policy_route_eligible: impl FnOnce(),
) -> ReviewAdmissionSection82RoutingOutcome {
    match comparison {
        JournalAnchorHistoryComparison::AnchorEqualsCurrentHead
        | JournalAnchorHistoryComparison::AnchorIsValidAncestor => {
            on_policy_route_eligible();
            ReviewAdmissionSection82RoutingOutcome::PolicyRouteEligible
        }
        failure => ReviewAdmissionSection82RoutingOutcome::PreTerminal(
            ReviewAdmissionSection82PrerequisiteFailure::ReturnedAnchorComparison(failure),
        ),
    }
}

/// Routes exact retained Review Request/Result inputs through the pre-§82
/// retained-Anchor boundary before any Policy work.
///
/// A strict input/binding/recovery failure is a pre-terminal §82 prerequisite
/// failure. Only after retained-only Anchor recovery and an acceptable §116
/// comparison may a separately established Policy process be notified of an
/// eligible route. This function does not receive or return Policy completion,
/// Admission, terminal Record construction, or Journal publication.
///
/// ```compile_fail
/// use evidence_registry::compose_review_admission_policy_46;
/// ```
///
/// ```compile_fail
/// use evidence_registry::derive_review_admission_lifecycle_outcome;
/// ```
///
/// The generic retained-history route itself returns none of those evaluator,
/// §46, or terminal-direction surfaces.
///
/// ```compile_fail
/// use evidence_registry::PolicyCompositionResult;
/// ```
///
/// ```compile_fail
/// use evidence_registry::ReviewAdmissionGateScopePolicy;
/// ```
///
/// ```compile_fail
/// use evidence_registry::ReviewAdmissionLifecycleOutcome;
/// ```
///
/// ```compile_fail
/// use evidence_registry::PolicyFieldEvaluatorMapping;
/// ```
///
/// ```compile_fail
/// use evidence_registry::policy_evaluator_1015_field_mapping;
/// ```
///
/// ```compile_fail
/// use evidence_registry::ReviewAdmissionScopeProfileRegistration;
/// ```
///
/// ```compile_fail
/// use evidence_registry::review_admission_scope_profile_1_v1;
/// ```
///
/// The generic retained-history route intentionally exposes none of those
/// unavailable evaluator, §46, or terminal-direction surfaces.
pub fn route_retained_review_admission_section_82(
    retained_journal: &RetainedJournal,
    request_event_reference: &JournalReference,
    request_bytes: &[u8],
    result_event_reference: &JournalReference,
    result_bytes: &[u8],
    on_policy_route_eligible: impl FnOnce(),
) -> ReviewAdmissionSection82RoutingOutcome {
    let anchor = match resolve_retained_review_package_anchor_input(
        retained_journal,
        request_event_reference,
        request_bytes,
        result_event_reference,
        result_bytes,
    ) {
        Ok(anchor) => anchor,
        Err(error) => {
            return ReviewAdmissionSection82RoutingOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::RetainedAnchorInput(error),
            );
        }
    };
    route_review_admission_after_anchor_comparison(
        compare_retained_journal_anchor_history(retained_journal, &anchor),
        on_policy_route_eligible,
    )
}

/// The bounded post-Anchor route into structural Review Admission Policy
/// context prerequisites.
///
/// `PolicyContextReady` establishes only the retained Request-selected Policy
/// identity and its v0.4 structural context prerequisites. It is not §46
/// completion, Policy satisfaction, Admission, a terminal disposition, or
/// Journal publication. Every other variant is pre-terminal and must not be
/// relabeled as a completed Policy result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionPolicyContextRouteOutcome {
    PreTerminal(ReviewAdmissionSection82PrerequisiteFailure),
    PolicyContextPrecondition(ReviewAdmissionPolicyContextRouteError),
    PolicyContextReady(ReviewAdmissionPolicyContextPrerequisites),
}

/// Fail-closed outcomes after a retained Review Package Anchor has reached an
/// acceptable §116 relationship but before §46 evaluation can begin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionPolicyContextRouteError {
    RequestDecode,
    PolicyPayloadUnavailable,
    PolicyContextPrerequisites(ReviewAdmissionPolicyContextPrerequisitesError),
}

/// Resolves the sole Request-selected Review Admission Policy bytes after, and
/// only after, the retained-Journal Anchor prerequisite succeeds.
///
/// The caller supplies no Policy reference or Policy bytes: both are derived
/// from the strictly decoded Request's `policy_authority_ref`, retained Journal
/// binding, and exact Record-id keyed byte resolver. This function deliberately
/// stops before evaluator dispatch, §46 composition, Admission construction,
/// terminal events, or publication. These structural inputs do not prove that
/// the selected Policy bytes came from an authoritative Record namespace.
pub fn route_retained_review_admission_policy_context(
    retained_journal: &RetainedJournal,
    request_event_reference: &JournalReference,
    request_bytes: &[u8],
    result_event_reference: &JournalReference,
    result_bytes: &[u8],
    resolver: &impl ExactRecordByteResolver,
) -> ReviewAdmissionPolicyContextRouteOutcome {
    let anchor = match resolve_retained_review_package_anchor_input(
        retained_journal,
        request_event_reference,
        request_bytes,
        result_event_reference,
        result_bytes,
    ) {
        Ok(anchor) => anchor,
        Err(error) => {
            return ReviewAdmissionPolicyContextRouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::RetainedAnchorInput(error),
            );
        }
    };
    match compare_retained_journal_anchor_history(retained_journal, &anchor) {
        JournalAnchorHistoryComparison::AnchorEqualsCurrentHead
        | JournalAnchorHistoryComparison::AnchorIsValidAncestor => {}
        failure => {
            return ReviewAdmissionPolicyContextRouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::ReturnedAnchorComparison(failure),
            );
        }
    }
    let request = match ReviewRequestRecord::decode_authoritative(request_bytes) {
        Ok(request) => request,
        Err(_) => {
            return ReviewAdmissionPolicyContextRouteOutcome::PolicyContextPrecondition(
                ReviewAdmissionPolicyContextRouteError::RequestDecode,
            );
        }
    };
    let policy_event_reference = request.policy_authority_ref();
    let policy_record_id = match RecordId::try_from(
        policy_event_reference
            .event_record_id()
            .as_bytes()
            .as_slice(),
    ) {
        Ok(record_id) => record_id,
        Err(_) => unreachable!("EventRecordId has RecordId width"),
    };
    let policy_bytes = match resolver.resolve(policy_record_id) {
        Some(bytes) => bytes,
        None => {
            return ReviewAdmissionPolicyContextRouteOutcome::PolicyContextPrecondition(
                ReviewAdmissionPolicyContextRouteError::PolicyPayloadUnavailable,
            );
        }
    };
    match validate_review_admission_policy_context_prerequisites(
        retained_journal,
        policy_event_reference,
        policy_bytes,
        request_bytes,
        result_bytes,
        resolver,
    ) {
        Ok(prerequisites) => {
            ReviewAdmissionPolicyContextRouteOutcome::PolicyContextReady(prerequisites)
        }
        Err(error) => ReviewAdmissionPolicyContextRouteOutcome::PolicyContextPrecondition(
            ReviewAdmissionPolicyContextRouteError::PolicyContextPrerequisites(error),
        ),
    }
}

/// One retained individual result from an applicable REVIEW_ADMISSION Policy
/// evaluator. This is not a completed Policy result or Lifecycle disposition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionIndividualEvaluatorOutcome {
    Pass,
    Fail,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReviewAdmissionIndividualEvaluatorResult {
    evaluator_id: u16,
    outcome: ReviewAdmissionIndividualEvaluatorOutcome,
}

impl ReviewAdmissionIndividualEvaluatorResult {
    /// The exact frozen evaluator identity invoked for this applicable requirement.
    pub fn evaluator_id(&self) -> u16 {
        self.evaluator_id
    }

    /// The individual evaluator outcome retained for §46 composition.
    pub fn outcome(&self) -> ReviewAdmissionIndividualEvaluatorOutcome {
        self.outcome
    }
}

/// The three completed results defined by Record Schema §46.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionCompletedPolicyResult {
    Satisfied,
    GateUnsatisfied,
    GateIndeterminate,
}

/// A completed Review Admission §46 result together with every applicable
/// individual evaluator outcome. This type does not determine §83 disposition,
/// construct an Admission, or construct/publish a Journal event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewAdmissionPolicy46Completion {
    result: ReviewAdmissionCompletedPolicyResult,
    evaluator_results: Vec<ReviewAdmissionIndividualEvaluatorResult>,
}

impl ReviewAdmissionPolicy46Completion {
    /// The completed §46 Policy result.
    pub fn result(&self) -> ReviewAdmissionCompletedPolicyResult {
        self.result
    }

    /// Every individual outcome retained before this completed result was composed.
    pub fn evaluator_results(&self) -> &[ReviewAdmissionIndividualEvaluatorResult] {
        &self.evaluator_results
    }
}

/// The only outcomes of the retained Review Admission §46 route.
///
/// Pre-terminal and context-precondition outcomes are not completed Policy
/// results and cannot be transformed into §83 lifecycle direction here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionPolicy46RouteOutcome {
    PreTerminal(ReviewAdmissionSection82PrerequisiteFailure),
    PolicyContextPrecondition(ReviewAdmissionPolicyContextRouteError),
    Completed(ReviewAdmissionPolicy46Completion),
}

/// The §83 Review Admission disposition selected only from a completed §46
/// result. This is not an Admission Record, terminal event, Journal append, or
/// publication effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionSection83Disposition {
    PreTerminal,
    ReviewAdmissionAccepted,
    ReviewAdmissionRejected,
}

/// Applies Lifecycle v0.10.4 §83 to the completed-result boundary established
/// by §46. A non-completed result remains pre-terminal; §83 never fabricates a
/// completed Policy result. Event construction and publication remain a later,
/// separate boundary.
pub fn derive_review_admission_section_83_disposition(
    policy_46_outcome: &ReviewAdmissionPolicy46RouteOutcome,
) -> ReviewAdmissionSection83Disposition {
    match policy_46_outcome {
        ReviewAdmissionPolicy46RouteOutcome::PreTerminal(_)
        | ReviewAdmissionPolicy46RouteOutcome::PolicyContextPrecondition(_) => {
            ReviewAdmissionSection83Disposition::PreTerminal
        }
        ReviewAdmissionPolicy46RouteOutcome::Completed(completion) => match completion.result() {
            ReviewAdmissionCompletedPolicyResult::Satisfied => {
                ReviewAdmissionSection83Disposition::ReviewAdmissionAccepted
            }
            ReviewAdmissionCompletedPolicyResult::GateUnsatisfied
            | ReviewAdmissionCompletedPolicyResult::GateIndeterminate => {
                ReviewAdmissionSection83Disposition::ReviewAdmissionRejected
            }
        },
    }
}

/// Applies the retained §82 authority-input gate before any REVIEW_ADMISSION
/// Policy evaluator can run.
///
/// The current structural Journal and exact-byte resolver cannot establish
/// authoritative Journal/store provenance or an authoritative Record namespace.
/// Therefore this route always returns a preterminal §82 authority-input outcome;
/// it cannot construct evaluator results or a completed §46 result.
pub fn evaluate_retained_review_admission_policy_46(
    retained_journal: &RetainedJournal,
    request_event_reference: &JournalReference,
    request_bytes: &[u8],
    result_event_reference: &JournalReference,
    result_bytes: &[u8],
    resolver: &impl ExactRecordByteResolver,
) -> ReviewAdmissionPolicy46RouteOutcome {
    let authority_error = match validate_review_admission_section_82_structural_inputs(
        retained_journal,
        request_event_reference,
        request_bytes,
        result_event_reference,
        result_bytes,
        resolver,
    ) {
        Err(error) => error,
        Ok(()) => ReviewAdmissionSection82AuthorityError::FreezeAuthorityEvidenceUnavailable,
    };
    ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
        ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(authority_error),
    )
}

/// Exact structural facts obtained by resolving a minimal POLICY's declared gate Scope.
///
/// This result establishes only strict local decoding and exact self-hash identity binding.
/// It does not establish Policy or Scope authority, satisfaction, admission, lifecycle
/// legality, resolver persistence, custody, durability, filesystem truth, or external trust.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyScopeStructuralBinding {
    policy_record_id: RecordId,
    gate_scope_ref: RecordId,
    scope_record: ScopeRecord,
}

impl PolicyScopeStructuralBinding {
    /// The exact locally decoded minimal POLICY identity.
    pub fn policy_record_id(&self) -> RecordId {
        self.policy_record_id
    }

    /// The exact SCOPE identity declared at POLICY key 16.
    pub fn gate_scope_ref(&self) -> RecordId {
        self.gate_scope_ref
    }

    /// The exact local SCOPE fields supplied for the declared identity.
    pub fn scope_record(&self) -> &ScopeRecord {
        &self.scope_record
    }
}

/// A fail-closed outcome while resolving the exact bytes declared by a minimal POLICY.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyScopeStructuralBindingError {
    /// The resolver had no bytes for the declared SCOPE identity; this proves no nonexistence fact.
    ScopePayloadUnavailable,
    /// Supplied bytes did not satisfy the strict local SCOPE grammar.
    ScopeDecode,
    /// Strictly decoded SCOPE bytes did not self-hash to the exact POLICY-declared identity.
    ScopeIdentityMismatch,
}

/// Resolves and structurally binds the exact SCOPE bytes declared by a minimal POLICY.
///
/// The injected resolver supplies only purported bytes. A successful result is deliberately
/// limited to the strict Policy-to-Scope identity and local declared fields; it is not a Policy
/// evaluation or an authority, authorization, admission, or Scope-semantics result.
pub fn resolve_minimal_policy_gate_scope(
    policy: &MinimalPolicyRecord,
    resolver: &impl ExactRecordByteResolver,
) -> Result<PolicyScopeStructuralBinding, PolicyScopeStructuralBindingError> {
    let gate_scope_ref = policy.gate_scope_ref();
    let scope_bytes = resolver
        .resolve(gate_scope_ref)
        .ok_or(PolicyScopeStructuralBindingError::ScopePayloadUnavailable)?;
    let scope_record = ScopeRecord::decode_authoritative(scope_bytes)
        .map_err(|_| PolicyScopeStructuralBindingError::ScopeDecode)?;
    if scope_record.record_id() != gate_scope_ref {
        return Err(PolicyScopeStructuralBindingError::ScopeIdentityMismatch);
    }
    Ok(PolicyScopeStructuralBinding {
        policy_record_id: policy.record_id(),
        gate_scope_ref,
        scope_record,
    })
}

/// Exact retained-history facts that bind a minimal POLICY Record to its
/// `POLICY_RECORDED` event and declared operation-start chronology reference.
///
/// This establishes only strict local POLICY decoding, exact Record/event identity,
/// retained event classification, and Journal order. It does not establish that an
/// operation occurred, Policy authority or satisfaction, Scope semantics, admission,
/// freshness, environment stability, lifecycle authority, custody, durability, or trust.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyRecordedStructuralBinding {
    policy_record_id: RecordId,
    policy_event_reference: JournalReference,
    operation_start_reference: JournalReference,
}

impl PolicyRecordedStructuralBinding {
    /// The exact identity of the strictly decoded minimal POLICY Record.
    pub fn policy_record_id(&self) -> RecordId {
        self.policy_record_id
    }

    /// The exact retained `POLICY_RECORDED` reference bound to the POLICY bytes.
    pub fn policy_event_reference(&self) -> &JournalReference {
        &self.policy_event_reference
    }

    /// The exact strictly-prior retained operation-start chronology reference.
    pub fn operation_start_reference(&self) -> &JournalReference {
        &self.operation_start_reference
    }
}

/// A fail-closed outcome while structurally binding a minimal POLICY registration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyRecordedStructuralBindingError {
    /// Supplied bytes did not satisfy the strict local minimal POLICY grammar.
    PolicyDecode,
    /// The supplied retained Policy-event reference was absent or differed from retained bytes.
    PolicyEventReference(RetainedJournalError),
    /// The resolved event was not the Record-backed `POLICY_RECORDED` Policy event shape.
    PolicyEventMismatch,
    /// Strictly decoded POLICY bytes did not self-hash to the retained event Record identity.
    PolicyEventRecordIdentityMismatch,
    /// The declared operation-start reference was absent or differed from retained bytes.
    OperationStartReference(RetainedJournalError),
    /// The declared operation-start entry was not strictly earlier than Policy registration.
    OperationStartNotStrictlyPrior,
}

/// Structurally binds exact minimal POLICY bytes to one retained `POLICY_RECORDED` Entry.
///
/// The operation-start reference is checked only as retained same-Registry chronology. Its
/// successful resolution does not make it an authority dependency or establish freshness,
/// environmental stability, Policy applicability, or any lifecycle outcome.
pub fn validate_policy_recorded_structural_binding(
    retained_journal: &RetainedJournal,
    policy_event_reference: &JournalReference,
    policy_bytes: &[u8],
) -> Result<PolicyRecordedStructuralBinding, PolicyRecordedStructuralBindingError> {
    let policy = MinimalPolicyRecord::decode_authoritative(policy_bytes)
        .map_err(|_| PolicyRecordedStructuralBindingError::PolicyDecode)?;
    let policy_event = retained_journal
        .resolve_reference(policy_event_reference)
        .map_err(PolicyRecordedStructuralBindingError::PolicyEventReference)?;
    if policy_event.event_type_id().value() != 400
        || policy_event.event_type_id().required_record_type_id()
            != RecordTypeId::try_from(40).expect("POLICY is assigned in Record Schema v0.3")
        || policy_event.lifecycle_object_kind() != LifecycleObjectKind::Policy
        || policy_event.lifecycle_object_id() != *policy_event.event_record_id().as_bytes()
    {
        return Err(PolicyRecordedStructuralBindingError::PolicyEventMismatch);
    }
    if policy_event.event_record_id().as_bytes() != policy.record_id().as_bytes() {
        return Err(PolicyRecordedStructuralBindingError::PolicyEventRecordIdentityMismatch);
    }
    let operation_start = retained_journal
        .resolve_reference(policy.operation_start_journal_ref())
        .map_err(PolicyRecordedStructuralBindingError::OperationStartReference)?;
    if operation_start.entry_index().value() >= policy_event.entry_index().value() {
        return Err(PolicyRecordedStructuralBindingError::OperationStartNotStrictlyPrior);
    }
    Ok(PolicyRecordedStructuralBinding {
        policy_record_id: policy.record_id(),
        policy_event_reference: policy_event_reference.clone(),
        operation_start_reference: policy.operation_start_journal_ref().clone(),
    })
}

/// The exact version-1 Review Package Anchor transport fields and predecessor
/// Review Request fields decoded from a prospective REVIEW_REQUEST Record.
///
/// Strict decoding establishes only the local Record grammar and self-hash
/// identity. It does not establish Request authority, package authority,
/// retained-Journal Anchor resolution, §82 completion, Policy satisfaction, or
/// Admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewRequestRecord {
    record_id: RecordId,
    freeze_authority_ref: JournalReference,
    manifest_id: RecordId,
    review_role_id: u64,
    required_checks_ref: RecordId,
    policy_authority_ref: JournalReference,
    review_scope_ref: RecordId,
    review_method_ref: RecordId,
    review_package_anchor_id: JournalAnchorId,
    operation_start_journal_ref: JournalReference,
    terminal_authority_closure_sha256: Option<[u8; ID_LENGTH]>,
}

impl ReviewRequestRecord {
    /// Strictly decodes the frozen Record Schema v0.3 version-1
    /// REVIEW_REQUEST local grammar without normalizing bytes.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let frame = StrictRecordFrame::decode_authoritative(input)?;
        if frame.record_type_id() != RecordTypeId::try_from(30).expect("assigned Record Type") {
            return Err(RecordDecodeError);
        }
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 30
            || cursor.uint().map_err(|_| RecordDecodeError)? != 1
        {
            return Err(RecordDecodeError);
        }
        let field_count = cursor.map().map_err(|_| RecordDecodeError)?;
        if !matches!(field_count, 11 | 13) {
            return Err(RecordDecodeError);
        }
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 30 {
            return Err(RecordDecodeError);
        }
        cursor.key(16).map_err(|_| RecordDecodeError)?;
        let freeze_authority_ref =
            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?;
        cursor.key(17).map_err(|_| RecordDecodeError)?;
        let manifest_id =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(18).map_err(|_| RecordDecodeError)?;
        let review_role_id = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !matches!(review_role_id, 1..=5) {
            return Err(RecordDecodeError);
        }
        cursor.key(19).map_err(|_| RecordDecodeError)?;
        let required_checks_ref =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(20).map_err(|_| RecordDecodeError)?;
        let policy_authority_ref =
            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?;
        cursor.key(21).map_err(|_| RecordDecodeError)?;
        let review_scope_ref =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(22).map_err(|_| RecordDecodeError)?;
        let review_method_ref =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(23).map_err(|_| RecordDecodeError)?;
        let review_package_anchor_id =
            JournalAnchorId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(24).map_err(|_| RecordDecodeError)?;
        let operation_start_journal_ref =
            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?;
        let terminal_authority_closure_sha256 = if field_count == 13 {
            cursor.key(25).map_err(|_| RecordDecodeError)?;
            if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
                return Err(RecordDecodeError);
            }
            cursor.key(26).map_err(|_| RecordDecodeError)?;
            let marker = cursor.bstr_32().map_err(|_| RecordDecodeError)?;
            (marker == TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256)
                .then_some(marker)
                .ok_or(RecordDecodeError)
                .map(Some)?
        } else {
            None
        };
        if !cursor.finished() {
            return Err(RecordDecodeError);
        }
        Ok(Self {
            record_id: frame.record_id(),
            freeze_authority_ref,
            manifest_id,
            review_role_id,
            required_checks_ref,
            policy_authority_ref,
            review_scope_ref,
            review_method_ref,
            review_package_anchor_id,
            operation_start_journal_ref,
            terminal_authority_closure_sha256,
        })
    }

    /// The self-hash identity of these exact strictly decoded Record bytes.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// The exact version-1 Review Package Anchor carrier decoded in its named domain.
    pub fn review_package_anchor_id(&self) -> JournalAnchorId {
        self.review_package_anchor_id
    }

    /// The exact retained Freeze authority reference required for §82 comparison.
    pub fn freeze_authority_ref(&self) -> &JournalReference {
        &self.freeze_authority_ref
    }

    /// The exact Manifest identity required for §82 comparison.
    pub fn manifest_id(&self) -> RecordId {
        self.manifest_id
    }

    /// The exact Review role identifier required for §82 comparison.
    pub fn review_role_id(&self) -> u64 {
        self.review_role_id
    }

    /// The exact required CheckSet identity required for §82 comparison.
    pub fn required_checks_ref(&self) -> RecordId {
        self.required_checks_ref
    }

    /// The exact Policy authority reference named by this Review Request.
    pub fn policy_authority_ref(&self) -> &JournalReference {
        &self.policy_authority_ref
    }

    /// The exact Review Scope identity required for §82 comparison.
    pub fn review_scope_ref(&self) -> RecordId {
        self.review_scope_ref
    }

    /// The exact Review Method identity required for §82 comparison.
    pub fn review_method_ref(&self) -> RecordId {
        self.review_method_ref
    }

    /// The exact operation-start chronology reference carried by the Request.
    pub fn operation_start_journal_ref(&self) -> &JournalReference {
        &self.operation_start_journal_ref
    }

    /// Returns the exact selected Request marker when this Request selected that route.
    pub fn terminal_authority_closure_sha256(&self) -> Option<&[u8; ID_LENGTH]> {
        self.terminal_authority_closure_sha256.as_ref()
    }
}

/// The exact version-1 Review Package Anchor transport fields and predecessor
/// Review Result fields decoded from a prospective REVIEW_RESULT Record.
///
/// Strict decoding establishes only the local Record grammar and self-hash
/// identity. It does not establish Result authority, Request authority,
/// retained-Journal Anchor resolution, §82 completion, Policy satisfaction, or
/// Admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewResultRecord {
    record_id: RecordId,
    review_request_authority_ref: JournalReference,
    freeze_authority_ref: JournalReference,
    manifest_id: RecordId,
    review_role_id: u64,
    review_scope_ref: RecordId,
    review_method_ref: RecordId,
    method_status: u64,
    finding_state: u64,
    reason_codes: Vec<String>,
    findings: Vec<RecordId>,
    reviewer_metadata: Option<String>,
    review_package_anchor_id: JournalAnchorId,
    operation_start_journal_ref: JournalReference,
}

impl ReviewResultRecord {
    /// Strictly decodes the frozen Record Schema v0.3 version-1
    /// REVIEW_RESULT local grammar without normalizing bytes.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let frame = StrictRecordFrame::decode_authoritative(input)?;
        if frame.record_type_id() != RecordTypeId::try_from(31).expect("assigned Record Type") {
            return Err(RecordDecodeError);
        }
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 31
            || cursor.uint().map_err(|_| RecordDecodeError)? != 1
        {
            return Err(RecordDecodeError);
        }
        let field_count = cursor.map().map_err(|_| RecordDecodeError)?;
        if !matches!(field_count, 14 | 15) {
            return Err(RecordDecodeError);
        }
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 31 {
            return Err(RecordDecodeError);
        }
        cursor.key(16).map_err(|_| RecordDecodeError)?;
        let review_request_authority_ref =
            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?;
        cursor.key(17).map_err(|_| RecordDecodeError)?;
        let freeze_authority_ref =
            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?;
        cursor.key(18).map_err(|_| RecordDecodeError)?;
        let manifest_id =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(19).map_err(|_| RecordDecodeError)?;
        let review_role_id = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !matches!(review_role_id, 1..=5) {
            return Err(RecordDecodeError);
        }
        cursor.key(20).map_err(|_| RecordDecodeError)?;
        let review_scope_ref =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(21).map_err(|_| RecordDecodeError)?;
        let review_method_ref =
            RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(22).map_err(|_| RecordDecodeError)?;
        let method_status = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !matches!(method_status, 1..=3) {
            return Err(RecordDecodeError);
        }
        cursor.key(23).map_err(|_| RecordDecodeError)?;
        let finding_state = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !matches!(finding_state, 1..=3) {
            return Err(RecordDecodeError);
        }
        cursor.key(24).map_err(|_| RecordDecodeError)?;
        let reason_code_count = cursor.array().map_err(|_| RecordDecodeError)?;
        if reason_code_count > cursor.remaining() {
            return Err(RecordDecodeError);
        }
        let mut reason_codes = Vec::with_capacity(reason_code_count);
        let mut previous_reason_code: Option<String> = None;
        for _ in 0..reason_code_count {
            let reason_code = cursor.text().map_err(|_| RecordDecodeError)?;
            if previous_reason_code
                .as_ref()
                .is_some_and(|previous| previous.as_bytes() >= reason_code.as_bytes())
            {
                return Err(RecordDecodeError);
            }
            previous_reason_code = Some(reason_code.clone());
            reason_codes.push(reason_code);
        }
        cursor.key(25).map_err(|_| RecordDecodeError)?;
        let finding_count = cursor.array().map_err(|_| RecordDecodeError)?;
        if finding_count > cursor.remaining() / 34 {
            return Err(RecordDecodeError);
        }
        let mut findings = Vec::with_capacity(finding_count);
        for _ in 0..finding_count {
            findings.push(
                RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                    .map_err(|_| RecordDecodeError)?,
            );
        }
        let reviewer_metadata = if field_count == 15 {
            cursor.key(26).map_err(|_| RecordDecodeError)?;
            Some(cursor.text().map_err(|_| RecordDecodeError)?)
        } else {
            None
        };
        cursor.key(27).map_err(|_| RecordDecodeError)?;
        let review_package_anchor_id =
            JournalAnchorId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                .map_err(|_| RecordDecodeError)?;
        cursor.key(28).map_err(|_| RecordDecodeError)?;
        let operation_start_journal_ref =
            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?;
        if !cursor.finished() {
            return Err(RecordDecodeError);
        }
        Ok(Self {
            record_id: frame.record_id(),
            review_request_authority_ref,
            freeze_authority_ref,
            manifest_id,
            review_role_id,
            review_scope_ref,
            review_method_ref,
            method_status,
            finding_state,
            reason_codes,
            findings,
            reviewer_metadata,
            review_package_anchor_id,
            operation_start_journal_ref,
        })
    }

    /// The self-hash identity of these exact strictly decoded Record bytes.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// The exact version-1 Review Package Anchor carrier decoded in its named domain.
    pub fn review_package_anchor_id(&self) -> JournalAnchorId {
        self.review_package_anchor_id
    }

    /// The exact retained Freeze authority reference required for §82 comparison.
    pub fn freeze_authority_ref(&self) -> &JournalReference {
        &self.freeze_authority_ref
    }

    /// The exact Manifest identity required for §82 comparison.
    pub fn manifest_id(&self) -> RecordId {
        self.manifest_id
    }

    /// The exact Review role identifier required for §82 comparison.
    pub fn review_role_id(&self) -> u64 {
        self.review_role_id
    }

    /// The exact Review Scope identity required for §82 comparison.
    pub fn review_scope_ref(&self) -> RecordId {
        self.review_scope_ref
    }

    /// The exact Review Method identity required for §82 comparison.
    pub fn review_method_ref(&self) -> RecordId {
        self.review_method_ref
    }

    /// The exact retained Review Request authority selected by this Result.
    pub fn review_request_authority_ref(&self) -> &JournalReference {
        &self.review_request_authority_ref
    }

    /// The exact operation-start chronology reference carried by the Result.
    pub fn operation_start_journal_ref(&self) -> &JournalReference {
        &self.operation_start_journal_ref
    }

    /// The exact Method-status registry value carried by the Review Result.
    pub fn method_status(&self) -> u64 {
        self.method_status
    }

    /// The exact Finding-state registry value carried by the Review Result.
    pub fn finding_state(&self) -> u64 {
        self.finding_state
    }

    /// The exact canonical Result reason-code set retained for terminal derivation.
    pub fn reason_codes(&self) -> &[String] {
        &self.reason_codes
    }

    pub(crate) fn findings(&self) -> &[RecordId] {
        &self.findings
    }
}

/// The strictly decoded terminal `REVIEW_ADMISSION` Record local grammar.
///
/// This narrow first slice accepts only terminal Records whose exact Request,
/// Result, and Policy authority references all resolved. It establishes neither
/// the §82/§46 facts that select a disposition nor a Journal publication effect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewAdmissionRecordInput {
    pub disposition_id: u64,
    pub review_request_ref: JournalReference,
    pub review_result_ref: JournalReference,
    pub policy_authority_ref: JournalReference,
    pub reason_codes: Vec<String>,
    pub operation_start_journal_ref: JournalReference,
}

/// A rejected local terminal REVIEW_ADMISSION Record construction request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReviewAdmissionRecordError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewAdmissionRecord {
    record_id: RecordId,
    disposition_id: u64,
    review_request_ref: JournalReference,
    review_result_ref: JournalReference,
    policy_authority_ref: JournalReference,
    reason_codes: Vec<String>,
    operation_start_journal_ref: JournalReference,
    terminal_authority_closure_sha256: Option<[u8; ID_LENGTH]>,
}

impl ReviewAdmissionRecord {
    /// Constructs one terminal Review Admission Record from exact typed,
    /// already-resolved references. This local constructor does not establish
    /// that the selected disposition is lawful or publish an event.
    pub fn new(input: ReviewAdmissionRecordInput) -> Result<Self, ReviewAdmissionRecordError> {
        Self::new_with_terminal_authority_closure(input, None)
    }

    /// Constructs the selected terminal form bound to both assigned selectors.
    ///
    /// This only constructs the exact Record-local selected encoding; it does
    /// not establish a lawful disposition or publish a Journal event.
    pub fn new_selected(
        input: ReviewAdmissionRecordInput,
    ) -> Result<Self, ReviewAdmissionRecordError> {
        Self::new_with_terminal_authority_closure(
            input,
            Some(TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256),
        )
    }

    fn new_with_terminal_authority_closure(
        input: ReviewAdmissionRecordInput,
        terminal_authority_closure_sha256: Option<[u8; ID_LENGTH]>,
    ) -> Result<Self, ReviewAdmissionRecordError> {
        if !matches!(input.disposition_id, 1 | 2)
            || input.review_request_ref.event_type_id().value() != 300
            || input.review_result_ref.event_type_id().value() != 301
            || input.policy_authority_ref.event_type_id().value() != 400
            || input
                .reason_codes
                .windows(2)
                .any(|pair| pair[0].as_bytes() >= pair[1].as_bytes())
        {
            return Err(ReviewAdmissionRecordError);
        }
        let mut record = Self {
            record_id: RecordId::try_from([0_u8; ID_LENGTH].as_slice())
                .expect("RecordId has exact fixed width"),
            disposition_id: input.disposition_id,
            review_request_ref: input.review_request_ref,
            review_result_ref: input.review_result_ref,
            policy_authority_ref: input.policy_authority_ref,
            reason_codes: input.reason_codes,
            operation_start_journal_ref: input.operation_start_journal_ref,
            terminal_authority_closure_sha256,
        };
        record.record_id =
            RecordId::try_from(Sha256::digest(record.authoritative_cbor()).as_slice())
                .expect("SHA-256 has exact RecordId width");
        Ok(record)
    }

    /// Emits the exact canonical terminal REVIEW_ADMISSION Record bytes.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(512);
        bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
        bytes.extend_from_slice(RECORD_DOMAIN);
        encode_uint(&mut bytes, 32);
        bytes.push(1);
        bytes.push(if self.terminal_authority_closure_sha256.is_some() {
            0xaa
        } else {
            0xa8
        });
        bytes.extend_from_slice(&[0x00, 0x01, 0x01]);
        encode_uint(&mut bytes, 32);
        encode_uint(&mut bytes, 16);
        encode_uint(&mut bytes, self.disposition_id);
        encode_uint(&mut bytes, 17);
        bytes.extend_from_slice(&self.review_request_ref.authoritative_cbor());
        encode_uint(&mut bytes, 18);
        bytes.extend_from_slice(&self.review_result_ref.authoritative_cbor());
        encode_uint(&mut bytes, 19);
        bytes.extend_from_slice(&self.policy_authority_ref.authoritative_cbor());
        encode_uint(&mut bytes, 22);
        encode_array_length(&mut bytes, self.reason_codes.len());
        for reason_code in &self.reason_codes {
            encode_text(&mut bytes, reason_code);
        }
        encode_uint(&mut bytes, 23);
        bytes.extend_from_slice(&self.operation_start_journal_ref.authoritative_cbor());
        if let Some(selector) = self.terminal_authority_closure_sha256 {
            encode_uint(&mut bytes, 24);
            encode_bstr_32(
                &mut bytes,
                &TERMINAL_REVIEW_ADMISSION_LEGACY_SELECTOR_SHA256,
            );
            encode_uint(&mut bytes, 25);
            encode_bstr_32(&mut bytes, &selector);
        }
        bytes
    }

    /// Strictly decodes the exact local terminal REVIEW_ADMISSION Record shape
    /// when all three authority references were successfully resolved.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let frame = StrictRecordFrame::decode_authoritative(input)?;
        if frame.record_type_id() != RecordTypeId::try_from(32).expect("assigned Record Type") {
            return Err(RecordDecodeError);
        }
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 32
            || cursor.uint().map_err(|_| RecordDecodeError)? != 1
        {
            return Err(RecordDecodeError);
        }
        let body_field_count = cursor.map().map_err(|_| RecordDecodeError)?;
        if !matches!(body_field_count, 8 | 10) {
            return Err(RecordDecodeError);
        }
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 32 {
            return Err(RecordDecodeError);
        }
        cursor.key(16).map_err(|_| RecordDecodeError)?;
        let disposition_id = cursor.uint().map_err(|_| RecordDecodeError)?;
        if !matches!(disposition_id, 1 | 2) {
            return Err(RecordDecodeError);
        }
        cursor.key(17).map_err(|_| RecordDecodeError)?;
        let review_request_ref =
            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?;
        cursor.key(18).map_err(|_| RecordDecodeError)?;
        let review_result_ref =
            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?;
        cursor.key(19).map_err(|_| RecordDecodeError)?;
        let policy_authority_ref =
            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?;
        if review_request_ref.event_type_id().value() != 300
            || review_result_ref.event_type_id().value() != 301
            || policy_authority_ref.event_type_id().value() != 400
        {
            return Err(RecordDecodeError);
        }
        cursor.key(22).map_err(|_| RecordDecodeError)?;
        let reason_code_count = cursor.array().map_err(|_| RecordDecodeError)?;
        if reason_code_count > cursor.remaining() {
            return Err(RecordDecodeError);
        }
        let mut reason_codes = Vec::with_capacity(reason_code_count);
        let mut previous_reason_code: Option<String> = None;
        for _ in 0..reason_code_count {
            let reason_code = cursor.text().map_err(|_| RecordDecodeError)?;
            if previous_reason_code
                .as_ref()
                .is_some_and(|previous| previous.as_bytes() >= reason_code.as_bytes())
            {
                return Err(RecordDecodeError);
            }
            previous_reason_code = Some(reason_code.clone());
            reason_codes.push(reason_code);
        }
        cursor.key(23).map_err(|_| RecordDecodeError)?;
        let operation_start_journal_ref =
            decode_journal_reference(&mut cursor).map_err(|_| RecordDecodeError)?;
        let terminal_authority_closure_sha256 = if body_field_count == 10 {
            cursor.key(24).map_err(|_| RecordDecodeError)?;
            if cursor.bstr_32().map_err(|_| RecordDecodeError)?
                != TERMINAL_REVIEW_ADMISSION_LEGACY_SELECTOR_SHA256
            {
                return Err(RecordDecodeError);
            }
            cursor.key(25).map_err(|_| RecordDecodeError)?;
            let selector = cursor.bstr_32().map_err(|_| RecordDecodeError)?;
            if selector != TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256 {
                return Err(RecordDecodeError);
            }
            Some(selector)
        } else {
            None
        };
        if !cursor.finished() {
            return Err(RecordDecodeError);
        }
        Ok(Self {
            record_id: frame.record_id(),
            disposition_id,
            review_request_ref,
            review_result_ref,
            policy_authority_ref,
            reason_codes,
            operation_start_journal_ref,
            terminal_authority_closure_sha256,
        })
    }

    /// The exact self-hash identity of the strictly decoded Admission Record.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// The frozen Review Admission disposition registry identity.
    pub fn disposition_id(&self) -> u64 {
        self.disposition_id
    }

    /// The exact retained REVIEW_REQUEST_RECORDED candidate reference.
    pub fn review_request_ref(&self) -> Option<&JournalReference> {
        Some(&self.review_request_ref)
    }

    /// The exact retained REVIEW_RESULT_RECORDED candidate reference.
    pub fn review_result_ref(&self) -> Option<&JournalReference> {
        Some(&self.review_result_ref)
    }

    /// The exact retained POLICY_RECORDED authority reference.
    pub fn policy_authority_ref(&self) -> Option<&JournalReference> {
        Some(&self.policy_authority_ref)
    }

    /// The canonical sorted reason-code set carried by this record.
    pub fn reason_codes(&self) -> &[String] {
        &self.reason_codes
    }

    /// The exact Record-local operation-start chronology reference.
    pub fn operation_start_journal_ref(&self) -> &JournalReference {
        &self.operation_start_journal_ref
    }

    /// The exact selected-terminal Core digest when both selector fields are present.
    pub fn terminal_authority_closure_sha256(&self) -> Option<&[u8; ID_LENGTH]> {
        self.terminal_authority_closure_sha256.as_ref()
    }
}

/// Typed inputs for a terminal Review Admission Journal Entry.
///
/// The Admission Record fixes terminal direction and exact direct authorities;
/// this construction layer does not select a disposition or publish bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewAdmissionJournalEntryInput {
    pub registry_id: RegistryId,
    pub entry_index: JournalEntryIndex,
    pub previous_entry_hash: JournalEntryHash,
    pub admission: ReviewAdmissionRecord,
    pub storage_capability_class_id: RecordId,
    pub environment_observation_id: RecordId,
}

/// A rejected terminal Review Admission Journal Entry construction request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReviewAdmissionJournalEntryError;

/// A canonical terminal Review Admission Journal Entry, not a publication effect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewAdmissionJournalEntry {
    registry_id: RegistryId,
    entry_index: JournalEntryIndex,
    previous_entry_hash: JournalEntryHash,
    event_type_id: EventTypeId,
    event_record_id: EventRecordId,
    admission: ReviewAdmissionRecord,
    authority_dependencies: AuthorityDependencyCollection,
    storage_capability_class_id: RecordId,
    environment_observation_id: RecordId,
}

impl ReviewAdmissionJournalEntry {
    /// Constructs the terminal Entry whose event type and direct authority
    /// dependencies are determined by the exact terminal Admission Record.
    pub fn new(
        input: ReviewAdmissionJournalEntryInput,
    ) -> Result<Self, ReviewAdmissionJournalEntryError> {
        if input.entry_index.value() == 0 {
            return Err(ReviewAdmissionJournalEntryError);
        }
        let event_type_id = EventTypeId::try_from(match input.admission.disposition_id() {
            1 => 302,
            2 => 303,
            _ => return Err(ReviewAdmissionJournalEntryError),
        })
        .expect("terminal REVIEW_ADMISSION event IDs are registered");
        let event_record_id =
            EventRecordId::try_from(input.admission.record_id().as_bytes().as_slice())
                .expect("RecordId has EventRecordId width");
        let authority_dependencies =
            AuthorityDependencyCollection::from_unordered_semantic_elements(
                AuthorityDependencyContext::new(input.registry_id, input.entry_index),
                vec![
                    input
                        .admission
                        .review_request_ref()
                        .expect("this resolved Admission shape always retains Request")
                        .clone(),
                    input
                        .admission
                        .review_result_ref()
                        .expect("this resolved Admission shape always retains Result")
                        .clone(),
                    input
                        .admission
                        .policy_authority_ref()
                        .expect("this resolved Admission shape always retains Policy")
                        .clone(),
                ],
            )
            .map_err(|_| ReviewAdmissionJournalEntryError)?;
        Ok(Self {
            registry_id: input.registry_id,
            entry_index: input.entry_index,
            previous_entry_hash: input.previous_entry_hash,
            event_type_id,
            event_record_id,
            admission: input.admission,
            authority_dependencies,
            storage_capability_class_id: input.storage_capability_class_id,
            environment_observation_id: input.environment_observation_id,
        })
    }

    /// The terminal event selected only by the Admission disposition.
    pub fn event_type_id(&self) -> EventTypeId {
        self.event_type_id
    }

    /// The exact Admission Record identity carried as the event Record ID.
    pub fn event_record_id(&self) -> EventRecordId {
        self.event_record_id
    }

    /// The exact publication-time Journal slot selected by compare-and-append.
    pub fn entry_index(&self) -> JournalEntryIndex {
        self.entry_index
    }

    /// The exact publication-time predecessor hash, distinct from `Hstart(A)`.
    pub fn previous_entry_hash(&self) -> JournalEntryHash {
        self.previous_entry_hash
    }

    /// The storage-capability observation derived from the publication-time head.
    pub fn storage_capability_class_id(&self) -> RecordId {
        self.storage_capability_class_id
    }

    /// The environment observation derived from the publication-time head.
    pub fn environment_observation_id(&self) -> RecordId {
        self.environment_observation_id
    }

    /// The exact terminal Admission Record carried by this Entry.
    pub fn admission(&self) -> &ReviewAdmissionRecord {
        &self.admission
    }

    /// Every exact direct authority dependency in canonical Journal order.
    pub fn authority_dependencies(&self) -> &[JournalReference] {
        &self.authority_dependencies.elements
    }

    /// Emits exact canonical terminal Review Admission Journal Entry bytes.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(600);
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
        encode_uint(&mut bytes, u64::from(self.event_type_id.value()));
        bytes.push(0x05);
        encode_bstr_32(&mut bytes, self.event_record_id.as_bytes());
        bytes.extend_from_slice(&[0x06, 0x80, 0x07]);
        bytes.extend_from_slice(&self.authority_dependencies.authoritative_cbor());
        bytes.extend_from_slice(&[0x08, 0x06, 0x09]);
        encode_bstr_32(&mut bytes, self.event_record_id.as_bytes());
        bytes.push(0x0a);
        encode_bstr_32(&mut bytes, self.storage_capability_class_id.as_bytes());
        bytes.push(0x0b);
        encode_bstr_32(&mut bytes, self.environment_observation_id.as_bytes());
        bytes
    }
}

static NEXT_REVIEW_ADMISSION_ACCEPTANCE_TOKEN: AtomicU64 = AtomicU64::new(1);
/// Maximum combined opaque Request and Result bytes owned by one live acceptance.
pub const REVIEW_ADMISSION_MAX_OPAQUE_INPUT_BYTES: usize = 1_048_576;
/// Maximum unconsumed acceptance capabilities owned by one live Journal instance.
pub const REVIEW_ADMISSION_MAX_OUTSTANDING_ACCEPTANCES: usize = 64;

/// One process-local Review Admission command accepted with a live retained-head
/// snapshot intended for the Lifecycle v0.10.6 `L(A)` boundary.
///
/// This value is intentionally non-cloneable and has no public constructor. Its
/// private execution token is local process state, not a Record field, Journal
/// field, caller identity, or persisted continuation. Request and Result payloads
/// remain opaque bytes until this value is consumed by `complete_review_admission`.
/// The token and snapshot do not establish authoritative Journal/store provenance.
///
/// ```compile_fail
/// use evidence_registry::AcceptedReviewAdmission;
/// fn duplicate(value: AcceptedReviewAdmission) {
///     let _replay = value.clone();
/// }
/// ```
pub struct AcceptedReviewAdmission {
    acceptance_token: u64,
    origin_journal_instance_identity: Arc<RetainedJournalInstanceIdentity>,
    request_event_reference: JournalReference,
    request_bytes: Vec<u8>,
    result_event_reference: JournalReference,
    result_bytes: Vec<u8>,
    operation_start_journal_ref: JournalReference,
}

impl AcceptedReviewAdmission {
    /// The first and only live retained head captured for this acceptance instance.
    pub fn operation_start_journal_ref(&self) -> &JournalReference {
        &self.operation_start_journal_ref
    }
}

impl Drop for AcceptedReviewAdmission {
    fn drop(&mut self) {
        let previous = self
            .origin_journal_instance_identity
            .outstanding_review_admissions
            .fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0);
    }
}

/// The bounded failure to create a fresh live Review Admission acceptance instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionAcceptanceError {
    /// The combined opaque Request and Result payload exceeds the local bounded-input policy.
    InputTooLarge,
    /// This live Journal already owns the maximum number of unconsumed acceptances.
    TooManyOutstandingAcceptances,
    /// Owned opaque input storage could not be reserved without panicking this route.
    InputAllocationFailed,
    /// The process-local opaque token space was exhausted without reuse.
    AcceptanceTokenExhausted,
}

/// The bounded local runtime outcomes after consuming one live acceptance instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionRuntimeOutcome {
    /// §82 or a pre-§46 context obligation failed; no terminal bytes were appended.
    PreTerminal(ReviewAdmissionPolicy46RouteOutcome),
}

/// A local acceptance-capability failure before §82 input checking begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionRuntimeError {
    AcceptanceJournalMismatch,
}

impl RetainedJournal {
    /// Atomically accepts one opaque Review Admission command and snapshots the
    /// exact live retained head at one local linearization point.
    ///
    /// This method performs no Request, Result, Policy, or Scope decoding. Safe
    /// Rust aliasing requires exclusive access for any append, so observing the
    /// head through this shared borrow serializes every append wholly before or
    /// wholly after this action. No caller-provided operation-start reference is
    /// accepted by this API. This local operation does not itself establish the
    /// authoritative Journal/store provenance required by `L(A)`.
    ///
    /// ```compile_fail
    /// use evidence_registry::{JournalReference, RetainedJournal};
    /// fn inject_head(
    ///     journal: &RetainedJournal,
    ///     request: JournalReference,
    ///     result: JournalReference,
    ///     caller_head: JournalReference,
    /// ) {
    ///     let _ = journal.accept_review_admission(
    ///         request,
    ///         &[],
    ///         result,
    ///         &[],
    ///         caller_head,
    ///     );
    /// }
    /// ```
    pub fn accept_review_admission(
        &self,
        request_event_reference: JournalReference,
        request_bytes: &[u8],
        result_event_reference: JournalReference,
        result_bytes: &[u8],
    ) -> Result<AcceptedReviewAdmission, ReviewAdmissionAcceptanceError> {
        if request_bytes
            .len()
            .checked_add(result_bytes.len())
            .is_none_or(|length| length > REVIEW_ADMISSION_MAX_OPAQUE_INPUT_BYTES)
        {
            return Err(ReviewAdmissionAcceptanceError::InputTooLarge);
        }
        self.live_instance_identity
            .outstanding_review_admissions
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                (current < REVIEW_ADMISSION_MAX_OUTSTANDING_ACCEPTANCES).then_some(current + 1)
            })
            .map_err(|_| ReviewAdmissionAcceptanceError::TooManyOutstandingAcceptances)?;
        let acceptance_token = NEXT_REVIEW_ADMISSION_ACCEPTANCE_TOKEN
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| {
                self.live_instance_identity
                    .outstanding_review_admissions
                    .fetch_sub(1, Ordering::AcqRel);
                ReviewAdmissionAcceptanceError::AcceptanceTokenExhausted
            })?;
        let operation_start_journal_ref = self.current_head_reference();
        let mut owned_request_bytes = Vec::new();
        if owned_request_bytes
            .try_reserve_exact(request_bytes.len())
            .is_err()
        {
            self.live_instance_identity
                .outstanding_review_admissions
                .fetch_sub(1, Ordering::AcqRel);
            return Err(ReviewAdmissionAcceptanceError::InputAllocationFailed);
        }
        owned_request_bytes.extend_from_slice(request_bytes);
        let mut owned_result_bytes = Vec::new();
        if owned_result_bytes
            .try_reserve_exact(result_bytes.len())
            .is_err()
        {
            self.live_instance_identity
                .outstanding_review_admissions
                .fetch_sub(1, Ordering::AcqRel);
            return Err(ReviewAdmissionAcceptanceError::InputAllocationFailed);
        }
        owned_result_bytes.extend_from_slice(result_bytes);
        Ok(AcceptedReviewAdmission {
            acceptance_token,
            origin_journal_instance_identity: Arc::clone(&self.live_instance_identity),
            request_event_reference,
            request_bytes: owned_request_bytes,
            result_event_reference,
            result_bytes: owned_result_bytes,
            operation_start_journal_ref,
        })
    }

    /// Consumes one live local acceptance through the §82 authority-input gate.
    ///
    /// `Hstart(A)` is never refreshed. With the current structural Journal and
    /// exact-byte resolver, complete §82 authority remains unavailable, so this
    /// method returns a preterminal outcome before evaluators or terminal
    /// publication. This runtime method exposes no terminal construction path
    /// and accepts no caller-selected predecessor, storage-capability, or
    /// environment identity.
    ///
    /// ```compile_fail
    /// use evidence_registry::{
    ///     AcceptedReviewAdmission, ExactRecordByteResolver, RecordId, RetainedJournal,
    /// };
    /// fn inject_terminal_observations(
    ///     journal: &mut RetainedJournal,
    ///     accepted: AcceptedReviewAdmission,
    ///     resolver: &impl ExactRecordByteResolver,
    ///     storage: RecordId,
    ///     environment: RecordId,
    /// ) {
    ///     let _ = journal.complete_review_admission(
    ///         accepted,
    ///         resolver,
    ///         storage,
    ///         environment,
    ///     );
    /// }
    /// ```
    pub fn complete_review_admission(
        &mut self,
        accepted: AcceptedReviewAdmission,
        resolver: &impl ExactRecordByteResolver,
    ) -> Result<ReviewAdmissionRuntimeOutcome, ReviewAdmissionRuntimeError> {
        if !Arc::ptr_eq(
            &accepted.origin_journal_instance_identity,
            &self.live_instance_identity,
        ) {
            return Err(ReviewAdmissionRuntimeError::AcceptanceJournalMismatch);
        }
        let _consumed_acceptance_token = accepted.acceptance_token;
        if let Err(authority_error) = validate_review_admission_section_82_structural_inputs(
            self,
            &accepted.request_event_reference,
            &accepted.request_bytes,
            &accepted.result_event_reference,
            &accepted.result_bytes,
            resolver,
        ) {
            return Ok(ReviewAdmissionRuntimeOutcome::PreTerminal(
                ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                    ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(authority_error),
                ),
            ));
        }
        if accepted.request_event_reference.entry_index().value()
            > accepted.operation_start_journal_ref.entry_index().value()
        {
            return Ok(ReviewAdmissionRuntimeOutcome::PreTerminal(
                ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                    ReviewAdmissionSection82PrerequisiteFailure::RequestAuthorityAfterOperationStart,
                ),
            ));
        }
        if accepted.result_event_reference.entry_index().value()
            > accepted.operation_start_journal_ref.entry_index().value()
        {
            return Ok(ReviewAdmissionRuntimeOutcome::PreTerminal(
                ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                    ReviewAdmissionSection82PrerequisiteFailure::ResultAuthorityAfterOperationStart,
                ),
            ));
        }
        Ok(ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                    ReviewAdmissionSection82AuthorityError::FreezeAuthorityEvidenceUnavailable,
                ),
            ),
        ))
    }
}

/// Fail-closed §82 prerequisite failures while comparing the exact common
/// Request/Result fields required before Policy evaluation.
///
/// These are not Policy evaluator results and must remain pre-terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAdmissionCommonRequestResultError {
    /// The prospective Review Request fails strict local decoding.
    RequestDecode,
    /// The prospective Review Result fails strict local decoding.
    ResultDecode,
    /// Request and Result name different Freeze authority references.
    FreezeAuthorityMismatch,
    /// Request and Result name different Manifest identities.
    ManifestMismatch,
    /// Request and Result name different Review roles.
    ReviewRoleMismatch,
    /// Request and Result name different Review Scope identities.
    ReviewScopeMismatch,
    /// Request and Result name different Review Method identities.
    ReviewMethodMismatch,
}

/// Validates the exact common Request/Result identity equalities which are
/// explicit §82 prerequisites before Policy evaluation.
///
/// A successful result is only the common Review Scope identity. It does not
/// establish retained-event binding, Request authority, Policy authority,
/// satisfaction, Admission, a terminal disposition, or publication.
pub fn validate_review_admission_common_request_result_fields(
    request_bytes: &[u8],
    result_bytes: &[u8],
) -> Result<RecordId, ReviewAdmissionCommonRequestResultError> {
    let request = ReviewRequestRecord::decode_authoritative(request_bytes)
        .map_err(|_| ReviewAdmissionCommonRequestResultError::RequestDecode)?;
    let result = ReviewResultRecord::decode_authoritative(result_bytes)
        .map_err(|_| ReviewAdmissionCommonRequestResultError::ResultDecode)?;
    if request.freeze_authority_ref() != result.freeze_authority_ref() {
        return Err(ReviewAdmissionCommonRequestResultError::FreezeAuthorityMismatch);
    }
    if request.manifest_id() != result.manifest_id() {
        return Err(ReviewAdmissionCommonRequestResultError::ManifestMismatch);
    }
    if request.review_role_id() != result.review_role_id() {
        return Err(ReviewAdmissionCommonRequestResultError::ReviewRoleMismatch);
    }
    if request.review_scope_ref() != result.review_scope_ref() {
        return Err(ReviewAdmissionCommonRequestResultError::ReviewScopeMismatch);
    }
    if request.review_method_ref() != result.review_method_ref() {
        return Err(ReviewAdmissionCommonRequestResultError::ReviewMethodMismatch);
    }
    Ok(request.review_scope_ref())
}

/// A local failure while checking version-1 Review Package Anchor transport.
///
/// These outcomes do not classify an evaluator result or any Admission result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewPackageAnchorTransportError {
    /// The supplied Request bytes fail the strict prospective local schema.
    RequestDecode,
    /// The supplied Result bytes fail the strict prospective local schema.
    ResultDecode,
    /// The version-1 Result carrier differs from the version-1 Request carrier.
    AnchorCarrierMismatch,
}

/// Validates only the exact version-1 Request/Result Anchor-carrier transport relation.
///
/// This strict local check proves transport integrity only. It does not establish
/// either Record's authority, authoritative Request recovery, retained-Journal
/// Anchor resolution, canonical Anchor validation, §82 completion, Policy
/// satisfaction, evaluator completion, or Admission.
pub fn validate_review_package_anchor_transport(
    request_bytes: &[u8],
    result_bytes: &[u8],
) -> Result<(), ReviewPackageAnchorTransportError> {
    let request = ReviewRequestRecord::decode_authoritative(request_bytes)
        .map_err(|_| ReviewPackageAnchorTransportError::RequestDecode)?;
    let result = ReviewResultRecord::decode_authoritative(result_bytes)
        .map_err(|_| ReviewPackageAnchorTransportError::ResultDecode)?;
    if request.review_package_anchor_id() != result.review_package_anchor_id() {
        return Err(ReviewPackageAnchorTransportError::AnchorCarrierMismatch);
    }
    Ok(())
}

/// Exact retained-history facts binding a prospective version-1 REVIEW_REQUEST
/// Record to its required REVIEW_REQUEST_RECORDED Entry.
///
/// This is an identity and retained-event binding only. It does not establish
/// Review Package authority, authority of the referenced Request inputs,
/// successful Review Request creation, §82 completion, Policy satisfaction, or
/// Admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewRequestRecordedBinding {
    record_id: RecordId,
    event_reference: JournalReference,
}

impl ReviewRequestRecordedBinding {
    /// The exact self-hash identity of the strictly decoded REVIEW_REQUEST.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// The exact retained REVIEW_REQUEST_RECORDED Entry bound to that identity.
    pub fn event_reference(&self) -> &JournalReference {
        &self.event_reference
    }
}

/// A failure while binding exact REVIEW_REQUEST bytes to retained history.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewRequestRecordedBindingError {
    /// The supplied Request bytes fail the strict prospective local schema.
    RequestDecode,
    /// The supplied Request event reference is absent or differs from retained bytes.
    RequestEventReference(RetainedJournalError),
    /// The retained Entry is not the Record-backed REVIEW_REQUEST_RECORDED event shape.
    RequestEventMismatch,
    /// The strict Request self-hash differs from the retained Entry Record identity.
    RequestEventRecordIdentityMismatch,
    /// The retained event lacks exactly one matching `JOURNAL_ANCHOR_ID` commitment.
    RequestEventAnchorIdentityDependencyMismatch,
}

fn has_exact_journal_anchor_identity_dependency(
    dependencies: &[IdentityDependency],
    expected: JournalAnchorId,
) -> bool {
    let mut anchors = dependencies
        .iter()
        .filter_map(|dependency| match dependency {
            IdentityDependency::JournalAnchorId(anchor_id) => Some(*anchor_id),
            IdentityDependency::RecordId(_) => None,
        });
    matches!((anchors.next(), anchors.next()), (Some(actual), None) if actual == expected)
}

/// Validates only the exact prospective REVIEW_REQUEST Record-to-retained-event
/// binding required before later Review Package and Review Admission processing.
pub fn validate_review_request_recorded_binding(
    retained_journal: &RetainedJournal,
    request_event_reference: &JournalReference,
    request_bytes: &[u8],
) -> Result<ReviewRequestRecordedBinding, ReviewRequestRecordedBindingError> {
    let request = ReviewRequestRecord::decode_authoritative(request_bytes)
        .map_err(|_| ReviewRequestRecordedBindingError::RequestDecode)?;
    let request_event = retained_journal
        .resolve_reference(request_event_reference)
        .map_err(ReviewRequestRecordedBindingError::RequestEventReference)?;
    if request_event.event_type_id().value() != 300
        || request_event.event_type_id().required_record_type_id()
            != RecordTypeId::try_from(30).expect("REVIEW_REQUEST is assigned in Record Schema v0.3")
        || request_event.lifecycle_object_kind() != LifecycleObjectKind::ReviewRequest
        || request_event.lifecycle_object_id() != *request_event.event_record_id().as_bytes()
    {
        return Err(ReviewRequestRecordedBindingError::RequestEventMismatch);
    }
    if request_event.event_record_id().as_bytes() != request.record_id().as_bytes() {
        return Err(ReviewRequestRecordedBindingError::RequestEventRecordIdentityMismatch);
    }
    let identity_dependencies = retained_journal
        .identity_dependencies_for_reference(request_event_reference)
        .map_err(ReviewRequestRecordedBindingError::RequestEventReference)?;
    if !has_exact_journal_anchor_identity_dependency(
        identity_dependencies,
        request.review_package_anchor_id(),
    ) {
        return Err(
            ReviewRequestRecordedBindingError::RequestEventAnchorIdentityDependencyMismatch,
        );
    }
    Ok(ReviewRequestRecordedBinding {
        record_id: request.record_id(),
        event_reference: request_event_reference.clone(),
    })
}

/// Exact retained-history facts binding a prospective version-1 REVIEW_RESULT
/// Record to its required REVIEW_RESULT_RECORDED Entry.
///
/// This is an identity and retained-event binding only. It does not establish
/// Result authority, Request authority, Review Package authority, §82
/// completion, Policy satisfaction, or Admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewResultRecordedBinding {
    record_id: RecordId,
    event_reference: JournalReference,
}

impl ReviewResultRecordedBinding {
    /// The exact self-hash identity of the strictly decoded REVIEW_RESULT.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// The exact retained REVIEW_RESULT_RECORDED Entry bound to that identity.
    pub fn event_reference(&self) -> &JournalReference {
        &self.event_reference
    }
}

/// A failure while binding exact REVIEW_RESULT bytes to retained history.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewResultRecordedBindingError {
    /// The supplied Result bytes fail the strict prospective local schema.
    ResultDecode,
    /// The supplied Result event reference is absent or differs from retained bytes.
    ResultEventReference(RetainedJournalError),
    /// The retained Entry is not the Record-backed REVIEW_RESULT_RECORDED event shape.
    ResultEventMismatch,
    /// The strict Result self-hash differs from the retained Entry Record identity.
    ResultEventRecordIdentityMismatch,
    /// The retained event lacks exactly one matching `JOURNAL_ANCHOR_ID` commitment.
    ResultEventAnchorIdentityDependencyMismatch,
}

/// Validates only the exact prospective REVIEW_RESULT Record-to-retained-event
/// binding required before later Review Admission processing.
pub fn validate_review_result_recorded_binding(
    retained_journal: &RetainedJournal,
    result_event_reference: &JournalReference,
    result_bytes: &[u8],
) -> Result<ReviewResultRecordedBinding, ReviewResultRecordedBindingError> {
    let result = ReviewResultRecord::decode_authoritative(result_bytes)
        .map_err(|_| ReviewResultRecordedBindingError::ResultDecode)?;
    let result_event = retained_journal
        .resolve_reference(result_event_reference)
        .map_err(ReviewResultRecordedBindingError::ResultEventReference)?;
    if result_event.event_type_id().value() != 301
        || result_event.event_type_id().required_record_type_id()
            != RecordTypeId::try_from(31).expect("REVIEW_RESULT is assigned in Record Schema v0.3")
        || result_event.lifecycle_object_kind() != LifecycleObjectKind::ReviewResult
        || result_event.lifecycle_object_id() != *result_event.event_record_id().as_bytes()
    {
        return Err(ReviewResultRecordedBindingError::ResultEventMismatch);
    }
    if result_event.event_record_id().as_bytes() != result.record_id().as_bytes() {
        return Err(ReviewResultRecordedBindingError::ResultEventRecordIdentityMismatch);
    }
    let identity_dependencies = retained_journal
        .identity_dependencies_for_reference(result_event_reference)
        .map_err(ReviewResultRecordedBindingError::ResultEventReference)?;
    if !has_exact_journal_anchor_identity_dependency(
        identity_dependencies,
        result.review_package_anchor_id(),
    ) {
        return Err(ReviewResultRecordedBindingError::ResultEventAnchorIdentityDependencyMismatch);
    }
    Ok(ReviewResultRecordedBinding {
        record_id: result.record_id(),
        event_reference: result_event_reference.clone(),
    })
}

/// A fail-closed outcome while deriving the retained Review Package Anchor input.
///
/// None of these outcomes is a completed Policy, evaluator, lifecycle, or
/// Admission result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetainedReviewPackageAnchorInputError {
    /// The Request does not have the required exact retained recorded-event binding.
    RequestBinding(ReviewRequestRecordedBindingError),
    /// The Result does not have the required exact retained recorded-event binding.
    ResultBinding(ReviewResultRecordedBindingError),
    /// The Result's embedded Request authority reference differs from the exact Request event.
    ResultRequestAuthorityReferenceMismatch,
    /// The retained Result event omits its exact direct dependency on the Request event.
    ResultEventRequestAuthorityDependencyMissing,
    /// The strict version-1 Request/Result transport relation is not preserved.
    AnchorTransport(ReviewPackageAnchorTransportError),
    /// The retained-Journal-only Anchor candidate set is missing or ambiguous.
    AnchorResolution(RetainedJournalAnchorResolutionError),
    /// The canonical retained Anchor's rederived identity differs from the carried identity.
    AnchorIdentityMismatch,
}

/// Resolves the version-1 Review Package Anchor from exact retained Request and
/// Result event bindings plus the retained Journal alone.
///
/// This establishes only the bounded pre-§82 structural input path: exact
/// Record/event binding, exact Result-to-Request reference/dependency binding,
/// preserved version-1 carrier, and canonical retained-prefix resolution. It
/// does not establish Request or Result authority, a current-history outcome,
/// §82 completion, Policy satisfaction, evaluator completion, Admission, or
/// terminal publication.
pub fn resolve_retained_review_package_anchor_input(
    retained_journal: &RetainedJournal,
    request_event_reference: &JournalReference,
    request_bytes: &[u8],
    result_event_reference: &JournalReference,
    result_bytes: &[u8],
) -> Result<JournalAnchor, RetainedReviewPackageAnchorInputError> {
    let request_binding = validate_review_request_recorded_binding(
        retained_journal,
        request_event_reference,
        request_bytes,
    )
    .map_err(RetainedReviewPackageAnchorInputError::RequestBinding)?;
    let result_binding = validate_review_result_recorded_binding(
        retained_journal,
        result_event_reference,
        result_bytes,
    )
    .map_err(RetainedReviewPackageAnchorInputError::ResultBinding)?;
    let result = ReviewResultRecord::decode_authoritative(result_bytes).map_err(|_| {
        RetainedReviewPackageAnchorInputError::ResultBinding(
            ReviewResultRecordedBindingError::ResultDecode,
        )
    })?;
    if result.review_request_authority_ref != *request_binding.event_reference() {
        return Err(RetainedReviewPackageAnchorInputError::ResultRequestAuthorityReferenceMismatch);
    }
    let result_entry = usize::try_from(result_binding.event_reference().entry_index().value())
        .ok()
        .and_then(|index| retained_journal.entries.get(index))
        .ok_or(
            RetainedReviewPackageAnchorInputError::ResultEventRequestAuthorityDependencyMissing,
        )?;
    if !result_entry
        .authority_dependencies()
        .contains(request_binding.event_reference())
    {
        return Err(
            RetainedReviewPackageAnchorInputError::ResultEventRequestAuthorityDependencyMissing,
        );
    }
    validate_review_package_anchor_transport(request_bytes, result_bytes)
        .map_err(RetainedReviewPackageAnchorInputError::AnchorTransport)?;
    let request = ReviewRequestRecord::decode_authoritative(request_bytes).map_err(|_| {
        RetainedReviewPackageAnchorInputError::RequestBinding(
            ReviewRequestRecordedBindingError::RequestDecode,
        )
    })?;
    let resolved = retained_journal
        .resolve_journal_anchor_id(request.review_package_anchor_id())
        .map_err(RetainedReviewPackageAnchorInputError::AnchorResolution)?;
    if resolved.anchor_id() != request.review_package_anchor_id() {
        return Err(RetainedReviewPackageAnchorInputError::AnchorIdentityMismatch);
    }
    Ok(resolved)
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

/// Typed inputs for a FREEZE_ATTEMPT_STARTED Journal Entry.
///
/// This construction layer derives the event's lifecycle and Record bindings
/// from the exact typed START Record. It neither allocates an Attempt nor
/// publishes an event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeAttemptStartJournalEntryInput {
    pub registry_id: RegistryId,
    pub entry_index: JournalEntryIndex,
    pub previous_entry_hash: JournalEntryHash,
    pub start_record: FreezeAttemptStartRecord,
    pub storage_capability_class_id: RecordId,
    pub environment_observation_id: RecordId,
}

/// A rejected FREEZE_ATTEMPT_STARTED Journal Entry construction request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FreezeAttemptStartJournalEntryError;

/// A canonical FREEZE_ATTEMPT_STARTED Journal Entry, not a publication effect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeAttemptStartJournalEntry {
    registry_id: RegistryId,
    entry_index: JournalEntryIndex,
    previous_entry_hash: JournalEntryHash,
    event_record_id: EventRecordId,
    start_record: FreezeAttemptStartRecord,
    storage_capability_class_id: RecordId,
    environment_observation_id: RecordId,
}

impl FreezeAttemptStartJournalEntry {
    /// Constructs the event-100 Entry derived from one exact START Record.
    pub fn new(
        input: FreezeAttemptStartJournalEntryInput,
    ) -> Result<Self, FreezeAttemptStartJournalEntryError> {
        if input.entry_index.value() == 0 {
            return Err(FreezeAttemptStartJournalEntryError);
        }
        let event_record_id =
            EventRecordId::try_from(input.start_record.record_id().as_bytes().as_slice())
                .expect("RecordId has EventRecordId width");
        Ok(Self {
            registry_id: input.registry_id,
            entry_index: input.entry_index,
            previous_entry_hash: input.previous_entry_hash,
            event_record_id,
            start_record: input.start_record,
            storage_capability_class_id: input.storage_capability_class_id,
            environment_observation_id: input.environment_observation_id,
        })
    }

    /// The frozen event type for a Freeze Attempt start.
    pub fn event_type_id(&self) -> EventTypeId {
        EventTypeId::try_from(100).expect("FREEZE_ATTEMPT_STARTED is registered")
    }

    /// The exact typed START Record identity carried by this Entry.
    pub fn event_record_id(&self) -> EventRecordId {
        self.event_record_id
    }

    /// The exact publication-time Journal slot selected by compare-and-append.
    pub fn entry_index(&self) -> JournalEntryIndex {
        self.entry_index
    }

    /// The exact publication-time predecessor hash.
    pub fn previous_entry_hash(&self) -> JournalEntryHash {
        self.previous_entry_hash
    }

    /// The exact typed START Record whose fields bind this Entry.
    pub fn start_record(&self) -> &FreezeAttemptStartRecord {
        &self.start_record
    }

    /// The storage-capability observation derived from the publication-time head.
    pub fn storage_capability_class_id(&self) -> RecordId {
        self.storage_capability_class_id
    }

    /// The environment observation derived from the publication-time head.
    pub fn environment_observation_id(&self) -> RecordId {
        self.environment_observation_id
    }

    /// Emits exact canonical FREEZE_ATTEMPT_STARTED Journal Entry bytes.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(320);
        bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
        bytes.extend_from_slice(JOURNAL_ENTRY_DOMAIN);
        bytes.push(0xae);
        bytes.extend_from_slice(&[0x00, 0x01, 0x01]);
        encode_bstr_32(&mut bytes, self.registry_id.as_bytes());
        bytes.push(0x02);
        encode_uint(&mut bytes, self.entry_index.value());
        bytes.push(0x03);
        encode_bstr_32(&mut bytes, self.previous_entry_hash.as_bytes());
        bytes.push(0x04);
        encode_uint(&mut bytes, u64::from(self.event_type_id().value()));
        bytes.push(0x05);
        encode_bstr_32(&mut bytes, self.event_record_id.as_bytes());
        bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x80, 0x08, 0x02, 0x09]);
        encode_bstr_32(
            &mut bytes,
            self.start_record.input().freeze_attempt_id.as_bytes(),
        );
        bytes.push(0x0a);
        encode_bstr_32(&mut bytes, self.storage_capability_class_id.as_bytes());
        bytes.push(0x0b);
        encode_bstr_32(&mut bytes, self.environment_observation_id.as_bytes());
        bytes.push(0x10);
        encode_bstr_32(
            &mut bytes,
            self.start_record.input().freeze_attempt_id.as_bytes(),
        );
        bytes.push(0x11);
        encode_bstr_32(
            &mut bytes,
            self.start_record.input().intended_root_id.as_bytes(),
        );
        bytes
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
    pub terminal_authority_closure_sha256: Option<[u8; ID_LENGTH]>,
    pub created_by_tool_version: String,
}

/// The exact-byte, Record-local FREEZE_RECEIPT schema from Record Schema v0.3 §56.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeReceiptRecord {
    record_id: RecordId,
    input: FreezeReceiptRecordInput,
}

impl FreezeReceiptRecord {
    /// Constructs canonical FREEZE_RECEIPT Record bytes from locally valid fields.
    ///
    /// This establishes only local grammar and Record identity. It does not
    /// establish policy, authority, admission, durability, custody, or lifecycle
    /// truth.
    pub fn new(input: FreezeReceiptRecordInput) -> Result<Self, RecordDecodeError> {
        if !matches!(input.custody_mode_id, 1 | 2)
            || !matches!(input.file_content_flush_state, 1..=3)
            || !matches!(input.atomic_publish_no_replace_state, 1..=3)
            || !matches!(input.parent_directory_flush_state, 1..=3)
            || input
                .terminal_authority_closure_sha256
                .is_some_and(|marker| marker != TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256)
        {
            return Err(RecordDecodeError);
        }
        let mut record = Self {
            record_id: RecordId::try_from([0_u8; ID_LENGTH].as_slice())
                .expect("RecordId has exact fixed width"),
            input,
        };
        record.record_id =
            RecordId::try_from(Sha256::digest(record.authoritative_cbor()).as_slice())
                .expect("SHA-256 has exact RecordId width");
        Ok(record)
    }

    /// Emits the exact canonical FREEZE_RECEIPT Record bytes for these local fields.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        let field_count = 17
            + usize::from(self.input.requested_commit_durability_ref.is_some())
            + usize::from(self.input.terminal_authority_closure_sha256.is_some());
        let mut bytes = Vec::with_capacity(512);
        bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
        bytes.extend_from_slice(RECORD_DOMAIN);
        bytes.extend_from_slice(&[0x04, 0x01]);
        encode_map_length(&mut bytes, field_count);
        encode_uint(&mut bytes, 0);
        encode_uint(&mut bytes, 1);
        encode_uint(&mut bytes, 1);
        encode_uint(&mut bytes, 4);
        encode_uint(&mut bytes, 16);
        encode_bstr_32(&mut bytes, self.input.freeze_attempt_id.as_bytes());
        encode_uint(&mut bytes, 17);
        encode_bstr_32(&mut bytes, &self.input.freeze_id);
        encode_uint(&mut bytes, 18);
        encode_journal_reference(&mut bytes, &self.input.attempt_start_journal_ref);
        encode_uint(&mut bytes, 19);
        encode_bstr_32(&mut bytes, &self.input.subject_id);
        encode_uint(&mut bytes, 20);
        encode_bstr_32(&mut bytes, self.input.manifest_id.as_bytes());
        encode_uint(&mut bytes, 21);
        encode_uint(&mut bytes, self.input.custody_mode_id);
        encode_uint(&mut bytes, 22);
        encode_bstr_32(&mut bytes, self.input.creation_profile_ref.as_bytes());
        encode_uint(&mut bytes, 23);
        encode_uint(&mut bytes, self.input.path_identity_profile_id);
        encode_uint(&mut bytes, 24);
        encode_bstr_32(&mut bytes, self.input.filesystem_profile_ref.as_bytes());
        encode_uint(&mut bytes, 25);
        encode_bstr_32(&mut bytes, self.input.policy_record_id.as_bytes());
        encode_uint(&mut bytes, 26);
        encode_uint(&mut bytes, self.input.file_content_flush_state);
        encode_uint(&mut bytes, 27);
        encode_uint(&mut bytes, self.input.atomic_publish_no_replace_state);
        encode_uint(&mut bytes, 28);
        encode_uint(&mut bytes, self.input.parent_directory_flush_state);
        encode_uint(&mut bytes, 29);
        bytes.push(if self.input.platform_strongest_available {
            0xf5
        } else {
            0xf4
        });
        if let Some(durability) = self.input.requested_commit_durability_ref {
            encode_uint(&mut bytes, 30);
            encode_bstr_32(&mut bytes, durability.as_bytes());
        }
        encode_uint(&mut bytes, 31);
        encode_text(&mut bytes, &self.input.created_by_tool_version);
        if let Some(marker) = self.input.terminal_authority_closure_sha256 {
            encode_uint(&mut bytes, 32);
            encode_bstr_32(&mut bytes, &marker);
        }
        bytes
    }

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
        if !matches!(field_count, 17..=19) {
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
        let next_key = cursor.uint().map_err(|_| RecordDecodeError)?;
        let requested_commit_durability_ref = if next_key == 30 {
            let durability = Some(
                RecordId::try_from(cursor.bstr_32().map_err(|_| RecordDecodeError)?.as_slice())
                    .map_err(|_| RecordDecodeError)?,
            );
            cursor.key(31).map_err(|_| RecordDecodeError)?;
            durability
        } else if next_key == 31 {
            None
        } else {
            return Err(RecordDecodeError);
        };
        let created_by_tool_version = cursor.text().map_err(|_| RecordDecodeError)?;
        let terminal_authority_closure_sha256 = match field_count {
            17 => None,
            18 if requested_commit_durability_ref.is_some() => None,
            18 | 19 => {
                cursor.key(32).map_err(|_| RecordDecodeError)?;
                let marker = cursor.bstr_32().map_err(|_| RecordDecodeError)?;
                (marker == TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256)
                    .then_some(marker)
                    .ok_or(RecordDecodeError)
                    .map(Some)?
            }
            _ => return Err(RecordDecodeError),
        };
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
                terminal_authority_closure_sha256,
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

    /// Returns the exact selected Freeze marker when this Receipt selected that route.
    pub fn terminal_authority_closure_sha256(&self) -> Option<&[u8; ID_LENGTH]> {
        self.input.terminal_authority_closure_sha256.as_ref()
    }
}

/// The exact structural inputs for one bounded FREEZE_COMMITTED binding check.
///
/// This input deliberately contains no Manifest, Policy, storage, custody, or
/// external-authority evidence. Those absences prohibit a positive authority or
/// Admission outcome from this structural layer alone.
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
    /// FREEZE_COMMITTED does not authority-depend on exactly its retained START.
    CommittedAuthorityDependencySetMismatch,
    /// The Receipt's start reference does not resolve to the retained FREEZE_ATTEMPT_STARTED event.
    AttemptStartReferenceMismatch,
    /// The retained FREEZE_ATTEMPT_STARTED event does not bind the exact START Record identity.
    AttemptStartRecordMismatch,
    /// FREEZE_ATTEMPT_STARTED carries an authority dependency forbidden by its event shape.
    AttemptStartAuthorityDependencySetMismatch,
    /// The exact START Record root does not match the retained START Journal root.
    IntendedRootMismatch,
    /// The START, Receipt, and terminal event do not bind one Freeze Attempt identity.
    FreezeAttemptMismatch,
    /// A selected Receipt's Freeze ID is not the exact START Freeze Attempt ID copy.
    SelectedFreezeIdMismatch,
    /// The Receipt and START do not preserve the same exact subject identity.
    SubjectMismatch,
    /// The Receipt and START do not preserve the same exact Policy identity.
    PolicyMismatch,
    /// The Receipt's retained start entry is not strictly prior to the terminal entry.
    AttemptStartNotPrior,
}

/// The only non-error outcome of the bounded FREEZE_COMMITTED structural check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FreezeCommittedBindingOutcome {
    /// Structural bindings agree; the evidence required for authority is not in this input.
    AuthorityEvidenceUnavailable,
}

/// Validates only the exact retained Journal / START / Receipt bindings available
/// to this structural layer.
///
/// A successful return is not an authority, Admission, Policy, external-custody,
/// durability, or persistence verdict.
pub fn validate_freeze_committed_binding(
    input: FreezeCommittedBindingInput<'_>,
) -> Result<FreezeCommittedBindingOutcome, FreezeCommittedBindingError> {
    if input.freeze_attempt_start_record.input.freeze_attempt_id
        != input.freeze_receipt_record.input.freeze_attempt_id
    {
        return Err(FreezeCommittedBindingError::FreezeAttemptMismatch);
    }
    if input
        .freeze_receipt_record
        .terminal_authority_closure_sha256()
        .is_some()
        && input.freeze_receipt_record.input.freeze_id
            != *input
                .freeze_attempt_start_record
                .input
                .freeze_attempt_id
                .as_bytes()
    {
        return Err(FreezeCommittedBindingError::SelectedFreezeIdMismatch);
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
    let start = input
        .retained_journal
        .resolve_reference(start_reference)
        .map_err(FreezeCommittedBindingError::RetainedReference)?;
    if start.entry_index().value() >= committed.entry_index().value() {
        return Err(FreezeCommittedBindingError::AttemptStartNotPrior);
    }
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
    let committed_authority_dependencies = input
        .retained_journal
        .authority_dependencies_for_reference(&input.committed_event_reference)
        .map_err(FreezeCommittedBindingError::RetainedReference)?;
    if committed_authority_dependencies != [start_reference.clone()] {
        return Err(FreezeCommittedBindingError::CommittedAuthorityDependencySetMismatch);
    }
    if start.event_record_id().as_bytes()
        != input.freeze_attempt_start_record.record_id().as_bytes()
    {
        return Err(FreezeCommittedBindingError::AttemptStartRecordMismatch);
    }
    let start_authority_dependencies = input
        .retained_journal
        .authority_dependencies_for_reference(start_reference)
        .map_err(FreezeCommittedBindingError::RetainedReference)?;
    if !start_authority_dependencies.is_empty() {
        return Err(FreezeCommittedBindingError::AttemptStartAuthorityDependencySetMismatch);
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
    /// The exact MANIFEST Record bytes required by the Receipt were unavailable.
    ManifestPayloadUnavailable,
    /// Supplied MANIFEST bytes did not satisfy the strict MANIFEST local grammar.
    ManifestDecode,
    /// Strictly decoded MANIFEST bytes did not hash to the Receipt's exact Manifest ID.
    ManifestIdentityMismatch,
    /// The exact local MANIFEST subject did not equal the Receipt subject.
    ManifestSubjectMismatch,
    /// The exact local MANIFEST path-profile ID did not equal the Receipt path-profile ID.
    ManifestPathIdentityProfileMismatch,
    /// The Receipt's retained START reference was not the required START event.
    AttemptStartReferenceMismatch,
    /// The exact START Record bytes required by the retained START Entry were unavailable.
    AttemptStartPayloadUnavailable,
    /// Supplied START bytes did not satisfy the strict FREEZE_ATTEMPT_START local grammar.
    AttemptStartDecode,
    /// Strictly decoded START bytes did not hash to the retained START Entry's exact Record ID.
    AttemptStartIdentityMismatch,
    /// Available exact inputs failed the bounded structural binding check.
    Binding(FreezeCommittedBindingError),
}

/// The bounded result after resolving every exact START, Receipt,
/// FREEZE_COMMITTED, and Manifest input available through the retained-history
/// and exact-byte composition seams.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedFreezeCommittedBindingOutcome {
    /// Structural topology and exact local Record identity agree, but these
    /// inputs do not prove authoritative Journal/store provenance or an
    /// authoritative exact-ID Record namespace.
    AuthorityEvidenceUnavailable,
}

/// Resolves exact Record bytes required to check the retained §55
/// FREEZE_COMMITTED topology and local Record identity chain.
///
/// A non-error result remains non-authoritative because `RetainedJournal` is an
/// in-memory structural reconstruction and `ExactRecordByteResolver` returns
/// only purported bytes. It proves no Freeze authority, Admission, Policy
/// satisfaction, canonical-store provenance, external custody, persistence, or
/// stronger durability than the supplied Receipt fields record.
pub fn validate_resolved_freeze_committed_binding(
    retained_journal: &RetainedJournal,
    committed_event_reference: JournalReference,
    resolver: &impl ExactRecordByteResolver,
) -> Result<ResolvedFreezeCommittedBindingOutcome, ResolvedFreezeCommittedBindingError> {
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
    let manifest_record_id = receipt.input().manifest_id;
    let manifest_bytes = resolver
        .resolve(manifest_record_id)
        .ok_or(ResolvedFreezeCommittedBindingError::ManifestPayloadUnavailable)?;
    let manifest = ManifestRecord::decode_authoritative(manifest_bytes)
        .map_err(|_| ResolvedFreezeCommittedBindingError::ManifestDecode)?;
    if manifest.record_id() != manifest_record_id {
        return Err(ResolvedFreezeCommittedBindingError::ManifestIdentityMismatch);
    }
    if manifest.input().subject_id != receipt.input().subject_id {
        return Err(ResolvedFreezeCommittedBindingError::ManifestSubjectMismatch);
    }
    if manifest.input().path_identity_profile_id != receipt.input().path_identity_profile_id {
        return Err(ResolvedFreezeCommittedBindingError::ManifestPathIdentityProfileMismatch);
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

    match validate_freeze_committed_binding(FreezeCommittedBindingInput {
        retained_journal,
        committed_event_reference,
        freeze_attempt_start_record: &freeze_attempt_start_record,
        freeze_receipt_record: &receipt,
    })
    .map_err(ResolvedFreezeCommittedBindingError::Binding)?
    {
        FreezeCommittedBindingOutcome::AuthorityEvidenceUnavailable => {}
    }
    Ok(ResolvedFreezeCommittedBindingOutcome::AuthorityEvidenceUnavailable)
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

/// The one exact static parameter Record for the selected Freeze creation profile.
///
/// This Record is content addressed only; it is not an event or evidence that a
/// Freeze creation procedure completed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeCreationProfileRecord {
    record_id: RecordId,
}

impl FreezeCreationProfileRecord {
    /// Constructs the sole assigned type-64/profile-1 parameter Record.
    pub fn new() -> Result<Self, RecordDecodeError> {
        let bytes = Self::exact_cbor();
        Self::decode_authoritative(&bytes)
    }

    fn exact_cbor() -> Vec<u8> {
        let mut bytes = Vec::with_capacity(48);
        bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
        bytes.extend_from_slice(RECORD_DOMAIN);
        bytes.extend_from_slice(&[
            0x18, 0x40, 0x01, 0xa3, 0x00, 0x01, 0x01, 0x18, 0x40, 0x10, 0x01,
        ]);
        bytes
    }

    /// Emits the sole canonical type-64/profile-1 Record byte sequence.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        Self::exact_cbor()
    }

    /// The identity of the exact static parameter Record.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// Strictly decodes only the exact assigned type-64/profile-1 Record.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 64
            || cursor.uint().map_err(|_| RecordDecodeError)? != 1
        {
            return Err(RecordDecodeError);
        }
        cursor.map_exact(3).map_err(|_| RecordDecodeError)?;
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 64 {
            return Err(RecordDecodeError);
        }
        cursor.key(16).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 || !cursor.finished() {
            return Err(RecordDecodeError);
        }
        if input != Self::exact_cbor() {
            return Err(RecordDecodeError);
        }
        let record_id = RecordId::try_from(Sha256::digest(input).as_slice())
            .expect("SHA-256 has exact RecordId width");
        Ok(Self { record_id })
    }
}

/// Required local fields of a type-60 STORAGE_CAPABILITY_CLASS Record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageCapabilityClassRecordInput {
    pub filesystem_transport: String,
    pub sync_management: String,
    pub placeholder_capability: u64,
    pub exclusive_create_capability: u64,
    pub no_replace_publication_capability: u64,
    pub locking_capability: u64,
    pub atomic_rename_capability: u64,
    pub file_flush_capability: u64,
    pub directory_flush_capability: u64,
}

/// The exact-byte, local type-60 STORAGE_CAPABILITY_CLASS grammar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageCapabilityClassRecord {
    record_id: RecordId,
    input: StorageCapabilityClassRecordInput,
}

impl StorageCapabilityClassRecord {
    /// Constructs a complete local capability-class Record without asserting
    /// that its described probes were performed.
    pub fn new(input: StorageCapabilityClassRecordInput) -> Result<Self, RecordDecodeError> {
        if !is_canonical_text_length(input.filesystem_transport.len() as u64)
            || !is_canonical_text_length(input.sync_management.len() as u64)
            || ![
                input.placeholder_capability,
                input.exclusive_create_capability,
                input.no_replace_publication_capability,
                input.locking_capability,
                input.atomic_rename_capability,
                input.file_flush_capability,
                input.directory_flush_capability,
            ]
            .iter()
            .all(|value| matches!(value, 1..=3))
        {
            return Err(RecordDecodeError);
        }
        let mut record = Self {
            record_id: RecordId::try_from([0; ID_LENGTH].as_slice())
                .expect("RecordId has exact fixed width"),
            input,
        };
        record.record_id =
            RecordId::try_from(Sha256::digest(record.authoritative_cbor()).as_slice())
                .expect("SHA-256 has exact RecordId width");
        Ok(record)
    }

    /// Emits the exact deterministic-CBOR type-60 Record bytes.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(128);
        bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
        bytes.extend_from_slice(RECORD_DOMAIN);
        bytes.extend_from_slice(&[0x18, 0x3c, 0x01, 0xab, 0x00, 0x01, 0x01, 0x18, 0x3c]);
        bytes.push(0x10);
        encode_text(&mut bytes, &self.input.filesystem_transport);
        bytes.push(0x11);
        encode_text(&mut bytes, &self.input.sync_management);
        for (key, value) in [
            (18, self.input.placeholder_capability),
            (19, self.input.exclusive_create_capability),
            (20, self.input.no_replace_publication_capability),
            (21, self.input.locking_capability),
            (22, self.input.atomic_rename_capability),
            (23, self.input.file_flush_capability),
            (24, self.input.directory_flush_capability),
        ] {
            encode_uint(&mut bytes, key);
            encode_uint(&mut bytes, value);
        }
        bytes
    }

    /// Returns the complete local Record fields.
    pub fn input(&self) -> &StorageCapabilityClassRecordInput {
        &self.input
    }

    /// Returns SHA-256 of the exact Record bytes.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// Strictly decodes only the complete required type-60 field set.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 60
            || cursor.uint().map_err(|_| RecordDecodeError)? != 1
        {
            return Err(RecordDecodeError);
        }
        cursor.map_exact(11).map_err(|_| RecordDecodeError)?;
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 60 {
            return Err(RecordDecodeError);
        }
        cursor.key(16).map_err(|_| RecordDecodeError)?;
        let filesystem_transport = cursor.text().map_err(|_| RecordDecodeError)?;
        cursor.key(17).map_err(|_| RecordDecodeError)?;
        let sync_management = cursor.text().map_err(|_| RecordDecodeError)?;
        let mut capabilities = [0; 7];
        for (offset, value) in capabilities.iter_mut().enumerate() {
            cursor
                .key(18 + offset as u64)
                .map_err(|_| RecordDecodeError)?;
            *value = cursor.uint().map_err(|_| RecordDecodeError)?;
        }
        if !cursor.finished() {
            return Err(RecordDecodeError);
        }
        let decoded = Self::new(StorageCapabilityClassRecordInput {
            filesystem_transport,
            sync_management,
            placeholder_capability: capabilities[0],
            exclusive_create_capability: capabilities[1],
            no_replace_publication_capability: capabilities[2],
            locking_capability: capabilities[3],
            atomic_rename_capability: capabilities[4],
            file_flush_capability: capabilities[5],
            directory_flush_capability: capabilities[6],
        })?;
        (decoded.authoritative_cbor() == input)
            .then_some(decoded)
            .ok_or(RecordDecodeError)
    }
}

/// Required and optional local fields of a type-61 ENVIRONMENT_OBSERVATION Record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvironmentObservationRecordInput {
    pub os_name: String,
    pub os_version: Option<String>,
    pub filesystem_reported_name: Option<String>,
    pub driver_details: Option<String>,
    pub mount_identity: Option<Vec<u8>>,
    pub volume_identity: Option<Vec<u8>>,
    pub resolved_registry_storage_identity: Option<Vec<u8>>,
    pub probe_tool_version: String,
    pub observation_limitations: Vec<String>,
}

/// The exact-byte, local type-61 ENVIRONMENT_OBSERVATION grammar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvironmentObservationRecord {
    record_id: RecordId,
    input: EnvironmentObservationRecordInput,
}

impl EnvironmentObservationRecord {
    /// Constructs a local observation Record without claiming that probes,
    /// capabilities, or a current filesystem state were established.
    pub fn new(input: EnvironmentObservationRecordInput) -> Result<Self, RecordDecodeError> {
        let texts = [
            Some(&input.os_name),
            input.os_version.as_ref(),
            input.filesystem_reported_name.as_ref(),
            input.driver_details.as_ref(),
            Some(&input.probe_tool_version),
        ];
        if texts.into_iter().flatten().any(|text| {
            u64::try_from(text.len()).map_or(true, |length| !is_canonical_text_length(length))
        }) || input.observation_limitations.iter().any(|text| {
            u64::try_from(text.len()).map_or(true, |length| !is_canonical_text_length(length))
        }) || input
            .observation_limitations
            .windows(2)
            .any(|pair| pair[0].as_bytes() >= pair[1].as_bytes())
        {
            return Err(RecordDecodeError);
        }
        let mut record = Self {
            record_id: RecordId::try_from([0; ID_LENGTH].as_slice())
                .expect("RecordId has exact fixed width"),
            input,
        };
        record.record_id =
            RecordId::try_from(Sha256::digest(record.authoritative_cbor()).as_slice())
                .expect("SHA-256 has exact RecordId width");
        Ok(record)
    }

    /// Emits the exact deterministic-CBOR type-61 Record bytes.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        let optional_count = [
            self.input.os_version.is_some(),
            self.input.filesystem_reported_name.is_some(),
            self.input.driver_details.is_some(),
            self.input.mount_identity.is_some(),
            self.input.volume_identity.is_some(),
            self.input.resolved_registry_storage_identity.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        let mut bytes = Vec::with_capacity(192);
        bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
        bytes.extend_from_slice(RECORD_DOMAIN);
        bytes.extend_from_slice(&[
            0x18,
            0x3d,
            0x01,
            0xa0 | (5 + optional_count) as u8,
            0x00,
            0x01,
            0x01,
            0x18,
            0x3d,
        ]);
        bytes.push(0x10);
        encode_text(&mut bytes, &self.input.os_name);
        if let Some(value) = &self.input.os_version {
            bytes.push(0x11);
            encode_text(&mut bytes, value);
        }
        if let Some(value) = &self.input.filesystem_reported_name {
            bytes.push(0x12);
            encode_text(&mut bytes, value);
        }
        if let Some(value) = &self.input.driver_details {
            bytes.push(0x13);
            encode_text(&mut bytes, value);
        }
        for (key, value) in [
            (20, self.input.mount_identity.as_ref()),
            (21, self.input.volume_identity.as_ref()),
            (22, self.input.resolved_registry_storage_identity.as_ref()),
        ] {
            if let Some(value) = value {
                encode_uint(&mut bytes, key);
                encode_bstr(&mut bytes, value);
            }
        }
        bytes.extend_from_slice(&[0x17]);
        encode_text(&mut bytes, &self.input.probe_tool_version);
        bytes.extend_from_slice(&[0x18, 0x18]);
        encode_array_length(&mut bytes, self.input.observation_limitations.len());
        for limitation in &self.input.observation_limitations {
            encode_text(&mut bytes, limitation);
        }
        bytes
    }

    /// Returns the complete local observation fields.
    pub fn input(&self) -> &EnvironmentObservationRecordInput {
        &self.input
    }

    /// Returns SHA-256 of the exact Record bytes.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// Strictly decodes only assigned type-61 fields without normalization.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 61
            || cursor.uint().map_err(|_| RecordDecodeError)? != 1
        {
            return Err(RecordDecodeError);
        }
        if !(5..=11).contains(&cursor.map().map_err(|_| RecordDecodeError)?) {
            return Err(RecordDecodeError);
        }
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 61 {
            return Err(RecordDecodeError);
        }
        cursor.key(16).map_err(|_| RecordDecodeError)?;
        let os_name = cursor.text().map_err(|_| RecordDecodeError)?;
        let mut optional_text = |key| -> Result<Option<String>, RecordDecodeError> {
            if cursor.input.get(cursor.offset) == Some(&key) {
                cursor.key(u64::from(key)).map_err(|_| RecordDecodeError)?;
                cursor.text().map(Some).map_err(|_| RecordDecodeError)
            } else {
                Ok(None)
            }
        };
        let os_version = optional_text(17)?;
        let filesystem_reported_name = optional_text(18)?;
        let driver_details = optional_text(19)?;
        let mut optional_bstr = |key| -> Result<Option<Vec<u8>>, RecordDecodeError> {
            if cursor.input.get(cursor.offset) == Some(&key) {
                cursor.key(u64::from(key)).map_err(|_| RecordDecodeError)?;
                cursor.bstr().map(Some).map_err(|_| RecordDecodeError)
            } else {
                Ok(None)
            }
        };
        let mount_identity = optional_bstr(20)?;
        let volume_identity = optional_bstr(21)?;
        let resolved_registry_storage_identity = optional_bstr(22)?;
        cursor.key(23).map_err(|_| RecordDecodeError)?;
        let probe_tool_version = cursor.text().map_err(|_| RecordDecodeError)?;
        cursor.key(24).map_err(|_| RecordDecodeError)?;
        let limitation_count = cursor.array().map_err(|_| RecordDecodeError)?;
        if limitation_count > cursor.remaining() {
            return Err(RecordDecodeError);
        }
        let mut observation_limitations = Vec::with_capacity(limitation_count);
        for _ in 0..limitation_count {
            observation_limitations.push(cursor.text().map_err(|_| RecordDecodeError)?);
        }
        if !cursor.finished() {
            return Err(RecordDecodeError);
        }
        let decoded = Self::new(EnvironmentObservationRecordInput {
            os_name,
            os_version,
            filesystem_reported_name,
            driver_details,
            mount_identity,
            volume_identity,
            resolved_registry_storage_identity,
            probe_tool_version,
            observation_limitations,
        })?;
        (decoded.authoritative_cbor() == input)
            .then_some(decoded)
            .ok_or(RecordDecodeError)
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

fn encode_map_length(output: &mut Vec<u8>, length: usize) {
    let length = u64::try_from(length).expect("platform usize fits into u64");
    debug_assert!(length <= ER_UINT_MAX);
    match length {
        0..=23 => output.push(0xa0 | length as u8),
        24..=0xff => output.extend_from_slice(&[0xb8, length as u8]),
        0x100..=0xffff => output.extend_from_slice(&[0xb9, (length >> 8) as u8, length as u8]),
        0x1_0000..=0xffff_ffff => output.extend_from_slice(&[
            0xba,
            (length >> 24) as u8,
            (length >> 16) as u8,
            (length >> 8) as u8,
            length as u8,
        ]),
        _ => output.extend_from_slice(&[
            0xbb,
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
    fn retained_append_does_not_replay_the_complete_prefix() {
        const APPEND_COUNT: usize = 128;

        let registry_id = RegistryId::try_from([0x11; ID_LENGTH].as_slice()).unwrap();
        let storage_capability_class_id = RecordId::try_from([0x22; ID_LENGTH].as_slice()).unwrap();
        let environment_observation_id = RecordId::try_from([0x33; ID_LENGTH].as_slice()).unwrap();
        let genesis = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from([0x44; ID_LENGTH].as_slice()).unwrap(),
            storage_capability_class_id,
            environment_observation_id,
        );
        let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
        RETAINED_REPLAY_ENTRY_VISITS.with(|visits| visits.set(0));

        for offset in 0..APPEND_COUNT {
            let index = u64::try_from(offset + 1).unwrap();
            let mut object_id = [0_u8; ID_LENGTH];
            object_id[..8].copy_from_slice(&index.to_be_bytes());
            let previous_entry_hash = journal.current_head_reference().entry_hash();
            journal
                .append(RetainedJournalEntry::Common(CommonRetainedJournalEntry {
                    registry_id,
                    entry_index: JournalEntryIndex::try_from(index).unwrap(),
                    previous_entry_hash,
                    event_type_id: EventTypeId::try_from(300).unwrap(),
                    event_record_id: EventRecordId::try_from(object_id.as_slice()).unwrap(),
                    storage_capability_class_id,
                    environment_observation_id,
                    lifecycle_object_kind: LifecycleObjectKind::ReviewRequest,
                    lifecycle_object_id: object_id,
                    freeze_attempt_intended_root_id: None,
                    identity_dependencies: IdentityDependencyCollection {
                        elements: Vec::new(),
                    },
                    authority_dependencies: AuthorityDependencyCollection {
                        elements: Vec::new(),
                    },
                    authoritative_bytes: index.to_be_bytes().to_vec(),
                }))
                .unwrap();
        }

        let replay_entry_visits = RETAINED_REPLAY_ENTRY_VISITS.with(std::cell::Cell::get);
        assert!(
            replay_entry_visits <= APPEND_COUNT,
            "append replayed {replay_entry_visits} retained entries while adding {APPEND_COUNT} independent objects"
        );
    }

    #[test]
    fn retained_reconstruction_uses_indexed_state_and_freeze_attempt_lookups() {
        const ATTEMPT_COUNT: usize = 96;

        let registry_id = RegistryId::try_from([0x51; ID_LENGTH].as_slice()).unwrap();
        let storage_capability_class_id = RecordId::try_from([0x52; ID_LENGTH].as_slice()).unwrap();
        let environment_observation_id = RecordId::try_from([0x53; ID_LENGTH].as_slice()).unwrap();
        let genesis = GenesisJournalEntry::new(
            registry_id,
            EventRecordId::try_from([0x54; ID_LENGTH].as_slice()).unwrap(),
            storage_capability_class_id,
            environment_observation_id,
        );
        let mut journal = RetainedJournal::from_genesis(genesis).unwrap();

        for offset in 0..ATTEMPT_COUNT {
            let entry_index = u64::try_from(journal.entries.len()).unwrap();
            let mut freeze_attempt_bytes = [0_u8; ID_LENGTH];
            freeze_attempt_bytes[..8]
                .copy_from_slice(&u64::try_from(offset + 1).unwrap().to_be_bytes());
            let freeze_attempt_id =
                FreezeAttemptId::try_from(freeze_attempt_bytes.as_slice()).unwrap();
            let intended_root_id =
                derive_freeze_root(registry_id, freeze_attempt_id).intended_root_id();
            let previous_entry_hash = journal.current_head_reference().entry_hash();
            journal
                .append(RetainedJournalEntry::Common(CommonRetainedJournalEntry {
                    registry_id,
                    entry_index: JournalEntryIndex::try_from(entry_index).unwrap(),
                    previous_entry_hash,
                    event_type_id: EventTypeId::try_from(100).unwrap(),
                    event_record_id: EventRecordId::try_from(freeze_attempt_bytes.as_slice())
                        .unwrap(),
                    storage_capability_class_id,
                    environment_observation_id,
                    lifecycle_object_kind: LifecycleObjectKind::FreezeAttempt,
                    lifecycle_object_id: freeze_attempt_bytes,
                    freeze_attempt_intended_root_id: Some(intended_root_id),
                    identity_dependencies: IdentityDependencyCollection {
                        elements: Vec::new(),
                    },
                    authority_dependencies: AuthorityDependencyCollection {
                        elements: Vec::new(),
                    },
                    authoritative_bytes: entry_index.to_be_bytes().to_vec(),
                }))
                .unwrap();
        }

        for offset in 0..ATTEMPT_COUNT {
            let entry_index = u64::try_from(journal.entries.len()).unwrap();
            let mut freeze_attempt_bytes = [0_u8; ID_LENGTH];
            freeze_attempt_bytes[..8]
                .copy_from_slice(&u64::try_from(offset + 1).unwrap().to_be_bytes());
            let mut event_record_bytes = freeze_attempt_bytes;
            event_record_bytes[ID_LENGTH - 1] = 0xA5;
            let previous_entry_hash = journal.current_head_reference().entry_hash();
            journal
                .append(RetainedJournalEntry::Common(CommonRetainedJournalEntry {
                    registry_id,
                    entry_index: JournalEntryIndex::try_from(entry_index).unwrap(),
                    previous_entry_hash,
                    event_type_id: EventTypeId::try_from(102).unwrap(),
                    event_record_id: EventRecordId::try_from(event_record_bytes.as_slice())
                        .unwrap(),
                    storage_capability_class_id,
                    environment_observation_id,
                    lifecycle_object_kind: LifecycleObjectKind::FreezeAttempt,
                    lifecycle_object_id: freeze_attempt_bytes,
                    freeze_attempt_intended_root_id: None,
                    identity_dependencies: IdentityDependencyCollection {
                        elements: Vec::new(),
                    },
                    authority_dependencies: AuthorityDependencyCollection {
                        elements: Vec::new(),
                    },
                    authoritative_bytes: entry_index.to_be_bytes().to_vec(),
                }))
                .unwrap();
        }

        LIFECYCLE_KIND_EQUALITY_COMPARISONS.with(|comparisons| comparisons.set(0));
        FREEZE_ATTEMPT_ID_EQUALITY_COMPARISONS.with(|comparisons| comparisons.set(0));
        let reconstructed = journal.reconstruct_state().unwrap();
        let state_key_comparisons = LIFECYCLE_KIND_EQUALITY_COMPARISONS.with(std::cell::Cell::get);
        let freeze_attempt_comparisons =
            FREEZE_ATTEMPT_ID_EQUALITY_COMPARISONS.with(std::cell::Cell::get);

        assert_eq!(reconstructed.freeze_attempts().len(), ATTEMPT_COUNT);
        assert!(
            state_key_comparisons <= journal.entries.len() * 4,
            "state reconstruction used {state_key_comparisons} lifecycle-kind comparisons for {} entries",
            journal.entries.len()
        );
        assert!(
            freeze_attempt_comparisons <= ATTEMPT_COUNT * 4,
            "state reconstruction used {freeze_attempt_comparisons} FreezeAttemptId comparisons for {ATTEMPT_COUNT} attempts"
        );
    }

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
