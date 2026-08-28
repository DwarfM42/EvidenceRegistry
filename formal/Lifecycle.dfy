// PURE_FORMAL lifecycle vocabulary and deterministic classification kernel.
//
// Sources: CROSS-REFERENCE v0.3 §§9–18, 23, 64–68, 72–76, and 92–93.
// Scope: exact symbolic event/object/state classifications and terminality only.
// This module deliberately does not claim Journal-reference, authority-dependency,
// Record-payload, runtime, storage, or replay validation.
module LifecycleModel {
  datatype EventType =
      Genesis
    | FreezeAttemptStarted
    | FreezeCommitted
    | FreezeAbortedRecovery
    | FreezeAbortedByOperatorAssertion
    | FreezeCommitRejected
    | VerificationRecorded
    | ReviewRequestRecorded
    | ReviewResultRecorded
    | ReviewAdmissionAccepted
    | ReviewAdmissionRejected
    | PolicyRecorded
    | CloseoutCommitted
    | CloseoutRejected
    | StorageCapabilityChanged
    | ArtifactEvictionStarted
    | ArtifactEvictionCommitted
    | ArtifactEvictionInterruptedRecovery
    | ArtifactEvictionBlocked
    | AssumptionDefinitionRecorded
    | AssumptionEstablishmentRecorded
    | AssumptionInvalidationRecorded
    | FormalVerificationRecorded
    | AssumptionVersionCompatibilityRecorded
    | AssumptionVersionCompatibilityInvalidated
    | BootstrapTrustDeclarationRecorded
    | BootstrapTrustInvalidated
    | FormalFindingClassificationRecorded

  datatype LifecycleObjectKind =
      Registry
    | FreezeAttempt
    | Verification
    | ReviewRequest
    | ReviewResult
    | ReviewAdmissionAttempt
    | Policy
    | CloseoutAttempt
    | ArtifactEvictionAttempt
    | AssumptionDefinition
    | AssumptionEstablishment
    | AssumptionInvalidation
    | FormalVerification
    | AssumptionVersionCompatibility
    | AssumptionVersionCompatibilityInvalidation
    | BootstrapTrustDeclaration
    | BootstrapTrustInvalidation
    | FormalFindingClassification

  datatype EventShape =
      BootstrapCreate
    | CreateOpen
    | CreateTerminal
    | TransitionTerminal
    | ObserveNoStateChange

  datatype RegistryState = RegistryAbsent | InitializedAuthoritative

  datatype FreezeAttemptState =
      FreezeAbsent
    | FreezeOpen
    | FreezeCommittedState
    | FreezeAbortedRecoveryState
    | FreezeAbortedByOperatorAssertionState

  datatype OneShotRecordedState = OneShotAbsent | Recorded

  datatype ReviewAdmissionState = AdmissionAbsent | AdmissionAccepted | AdmissionRejected

  datatype CloseoutAttemptState = CloseoutAbsent | CloseoutCommittedState | CloseoutRejectedState

  datatype ArtifactEvictionState =
      EvictionAbsent
    | EvictionOpen
    | EvictionCommitted
    | EvictionInterruptedRecovery
    | EvictionBlocked

  datatype LifecycleObjectState =
      RegistryObjectState(registry_state: RegistryState)
    | FreezeAttemptObjectState(freeze_state: FreezeAttemptState)
    | OneShotObjectState(one_shot_state: OneShotRecordedState)
    | ReviewAdmissionObjectState(admission_state: ReviewAdmissionState)
    | CloseoutAttemptObjectState(closeout_state: CloseoutAttemptState)
    | ArtifactEvictionObjectState(eviction_state: ArtifactEvictionState)

  function ObjectKindOf(event: EventType): LifecycleObjectKind {
    match event
      case Genesis => Registry
      case FreezeAttemptStarted => FreezeAttempt
      case FreezeCommitted => FreezeAttempt
      case FreezeAbortedRecovery => FreezeAttempt
      case FreezeAbortedByOperatorAssertion => FreezeAttempt
      case FreezeCommitRejected => FreezeAttempt
      case VerificationRecorded => Verification
      case ReviewRequestRecorded => ReviewRequest
      case ReviewResultRecorded => ReviewResult
      case ReviewAdmissionAccepted => ReviewAdmissionAttempt
      case ReviewAdmissionRejected => ReviewAdmissionAttempt
      case PolicyRecorded => Policy
      case CloseoutCommitted => CloseoutAttempt
      case CloseoutRejected => CloseoutAttempt
      case StorageCapabilityChanged => Registry
      case ArtifactEvictionStarted => ArtifactEvictionAttempt
      case ArtifactEvictionCommitted => ArtifactEvictionAttempt
      case ArtifactEvictionInterruptedRecovery => ArtifactEvictionAttempt
      case ArtifactEvictionBlocked => ArtifactEvictionAttempt
      case AssumptionDefinitionRecorded => AssumptionDefinition
      case AssumptionEstablishmentRecorded => AssumptionEstablishment
      case AssumptionInvalidationRecorded => AssumptionInvalidation
      case FormalVerificationRecorded => FormalVerification
      case AssumptionVersionCompatibilityRecorded => AssumptionVersionCompatibility
      case AssumptionVersionCompatibilityInvalidated => AssumptionVersionCompatibilityInvalidation
      case BootstrapTrustDeclarationRecorded => BootstrapTrustDeclaration
      case BootstrapTrustInvalidated => BootstrapTrustInvalidation
      case FormalFindingClassificationRecorded => FormalFindingClassification
  }

  function EventShapeOf(event: EventType): EventShape {
    match event
      case Genesis => BootstrapCreate
      case FreezeAttemptStarted => CreateOpen
      case FreezeCommitted => TransitionTerminal
      case FreezeAbortedRecovery => TransitionTerminal
      case FreezeAbortedByOperatorAssertion => TransitionTerminal
      case FreezeCommitRejected => ObserveNoStateChange
      case VerificationRecorded => CreateTerminal
      case ReviewRequestRecorded => CreateTerminal
      case ReviewResultRecorded => CreateTerminal
      case ReviewAdmissionAccepted => CreateTerminal
      case ReviewAdmissionRejected => CreateTerminal
      case PolicyRecorded => CreateTerminal
      case CloseoutCommitted => CreateTerminal
      case CloseoutRejected => CreateTerminal
      case StorageCapabilityChanged => ObserveNoStateChange
      case ArtifactEvictionStarted => CreateOpen
      case ArtifactEvictionCommitted => TransitionTerminal
      case ArtifactEvictionInterruptedRecovery => TransitionTerminal
      case ArtifactEvictionBlocked => TransitionTerminal
      case AssumptionDefinitionRecorded => CreateTerminal
      case AssumptionEstablishmentRecorded => CreateTerminal
      case AssumptionInvalidationRecorded => CreateTerminal
      case FormalVerificationRecorded => CreateTerminal
      case AssumptionVersionCompatibilityRecorded => CreateTerminal
      case AssumptionVersionCompatibilityInvalidated => CreateTerminal
      case BootstrapTrustDeclarationRecorded => CreateTerminal
      case BootstrapTrustInvalidated => CreateTerminal
      case FormalFindingClassificationRecorded => CreateTerminal
  }

  function HasTerminality(kind: LifecycleObjectKind): bool {
    kind != Registry
  }

  // OneShotObjectState represents each listed one-shot kind. The kind argument
  // remains explicit at the API boundary so non-one-shot state/kind confusion is
  // rejected by the StateMatchesKind precondition below.
  predicate StateMatchesKind(kind: LifecycleObjectKind, state: LifecycleObjectState) {
    match state
      case RegistryObjectState(_) => kind == Registry
      case FreezeAttemptObjectState(_) => kind == FreezeAttempt
      case OneShotObjectState(_) =>
        kind == Verification || kind == ReviewRequest || kind == ReviewResult || kind == Policy ||
        kind == AssumptionDefinition || kind == AssumptionEstablishment || kind == AssumptionInvalidation ||
        kind == FormalVerification || kind == AssumptionVersionCompatibility ||
        kind == AssumptionVersionCompatibilityInvalidation || kind == BootstrapTrustDeclaration ||
        kind == BootstrapTrustInvalidation || kind == FormalFindingClassification
      case ReviewAdmissionObjectState(_) => kind == ReviewAdmissionAttempt
      case CloseoutAttemptObjectState(_) => kind == CloseoutAttempt
      case ArtifactEvictionObjectState(_) => kind == ArtifactEvictionAttempt
  }

  function IsTerminal(kind: LifecycleObjectKind, state: LifecycleObjectState): bool
    requires HasTerminality(kind)
    requires StateMatchesKind(kind, state)
  {
    match state
      case RegistryObjectState(_) => false
      case FreezeAttemptObjectState(value) =>
        value == FreezeCommittedState || value == FreezeAbortedRecoveryState ||
        value == FreezeAbortedByOperatorAssertionState
      case OneShotObjectState(value) => value == Recorded
      case ReviewAdmissionObjectState(value) => value == AdmissionAccepted || value == AdmissionRejected
      case CloseoutAttemptObjectState(value) => value == CloseoutCommittedState || value == CloseoutRejectedState
      case ArtifactEvictionObjectState(value) =>
        value == EvictionCommitted || value == EvictionInterruptedRecovery || value == EvictionBlocked
  }

  lemma RegistryHasNoTerminality()
    ensures HasTerminality(Registry) == false
  {
  }

  lemma EveryNonRegistryKindHasTerminality(kind: LifecycleObjectKind)
    requires kind != Registry
    ensures HasTerminality(kind)
  {
  }

  lemma FreezeTerminalityIsExact(state: FreezeAttemptState)
    ensures IsTerminal(FreezeAttempt, FreezeAttemptObjectState(state)) ==
      (state == FreezeCommittedState || state == FreezeAbortedRecoveryState ||
       state == FreezeAbortedByOperatorAssertionState)
  {
  }

  lemma OneShotRecordedIsTerminal()
    ensures IsTerminal(Verification, OneShotObjectState(Recorded))
    ensures !IsTerminal(Verification, OneShotObjectState(OneShotAbsent))
  {
  }

  lemma AdmissionTerminalityIsExact(state: ReviewAdmissionState)
    ensures IsTerminal(ReviewAdmissionAttempt, ReviewAdmissionObjectState(state)) ==
      (state == AdmissionAccepted || state == AdmissionRejected)
  {
  }

  lemma CloseoutTerminalityIsExact(state: CloseoutAttemptState)
    ensures IsTerminal(CloseoutAttempt, CloseoutAttemptObjectState(state)) ==
      (state == CloseoutCommittedState || state == CloseoutRejectedState)
  {
  }

  lemma EvictionTerminalityIsExact(state: ArtifactEvictionState)
    ensures IsTerminal(ArtifactEvictionAttempt, ArtifactEvictionObjectState(state)) ==
      (state == EvictionCommitted || state == EvictionInterruptedRecovery || state == EvictionBlocked)
  {
  }

  lemma RegistryEventsAreExactlyClassified()
    ensures ObjectKindOf(Genesis) == Registry
    ensures EventShapeOf(Genesis) == BootstrapCreate
    ensures ObjectKindOf(StorageCapabilityChanged) == Registry
    ensures EventShapeOf(StorageCapabilityChanged) == ObserveNoStateChange
  {
  }

  lemma FreezeEventsAreExactlyClassified()
    ensures ObjectKindOf(FreezeAttemptStarted) == FreezeAttempt
    ensures EventShapeOf(FreezeAttemptStarted) == CreateOpen
    ensures EventShapeOf(FreezeCommitted) == TransitionTerminal
    ensures EventShapeOf(FreezeAbortedRecovery) == TransitionTerminal
    ensures EventShapeOf(FreezeAbortedByOperatorAssertion) == TransitionTerminal
    ensures EventShapeOf(FreezeCommitRejected) == ObserveNoStateChange
  {
  }

  lemma EvictionEventsAreExactlyClassified()
    ensures ObjectKindOf(ArtifactEvictionStarted) == ArtifactEvictionAttempt
    ensures EventShapeOf(ArtifactEvictionStarted) == CreateOpen
    ensures EventShapeOf(ArtifactEvictionCommitted) == TransitionTerminal
    ensures EventShapeOf(ArtifactEvictionInterruptedRecovery) == TransitionTerminal
    ensures EventShapeOf(ArtifactEvictionBlocked) == TransitionTerminal
  {
  }

  // This predicate is deliberately limited to the lifecycle-state portion of
  // the normative table. Full event legality additionally needs Journal
  // reference, authority-dependency, payload, and contextual validation and is
  // therefore outside this PURE_FORMAL classification kernel.
  predicate LegalPredecessor(
      kind: LifecycleObjectKind,
      event: EventType,
      before: LifecycleObjectState
  ) {
    match event
      case Genesis => kind == Registry && before == RegistryObjectState(RegistryAbsent)
      case FreezeAttemptStarted => kind == FreezeAttempt && before == FreezeAttemptObjectState(FreezeAbsent)
      case FreezeCommitted => kind == FreezeAttempt && before == FreezeAttemptObjectState(FreezeOpen)
      case FreezeAbortedRecovery => kind == FreezeAttempt && before == FreezeAttemptObjectState(FreezeOpen)
      case FreezeAbortedByOperatorAssertion => kind == FreezeAttempt && before == FreezeAttemptObjectState(FreezeOpen)
      case FreezeCommitRejected =>
        kind == FreezeAttempt &&
        (before == FreezeAttemptObjectState(FreezeCommittedState) ||
         before == FreezeAttemptObjectState(FreezeAbortedRecoveryState) ||
         before == FreezeAttemptObjectState(FreezeAbortedByOperatorAssertionState))
      case VerificationRecorded => kind == Verification && before == OneShotObjectState(OneShotAbsent)
      case ReviewRequestRecorded => kind == ReviewRequest && before == OneShotObjectState(OneShotAbsent)
      case ReviewResultRecorded => kind == ReviewResult && before == OneShotObjectState(OneShotAbsent)
      case ReviewAdmissionAccepted => kind == ReviewAdmissionAttempt && before == ReviewAdmissionObjectState(AdmissionAbsent)
      case ReviewAdmissionRejected => kind == ReviewAdmissionAttempt && before == ReviewAdmissionObjectState(AdmissionAbsent)
      case PolicyRecorded => kind == Policy && before == OneShotObjectState(OneShotAbsent)
      case CloseoutCommitted => kind == CloseoutAttempt && before == CloseoutAttemptObjectState(CloseoutAbsent)
      case CloseoutRejected => kind == CloseoutAttempt && before == CloseoutAttemptObjectState(CloseoutAbsent)
      case StorageCapabilityChanged => kind == Registry && before == RegistryObjectState(InitializedAuthoritative)
      case ArtifactEvictionStarted => kind == ArtifactEvictionAttempt && before == ArtifactEvictionObjectState(EvictionAbsent)
      case ArtifactEvictionCommitted => kind == ArtifactEvictionAttempt && before == ArtifactEvictionObjectState(EvictionOpen)
      case ArtifactEvictionInterruptedRecovery => kind == ArtifactEvictionAttempt && before == ArtifactEvictionObjectState(EvictionOpen)
      case ArtifactEvictionBlocked => kind == ArtifactEvictionAttempt && before == ArtifactEvictionObjectState(EvictionOpen)
      case AssumptionDefinitionRecorded => kind == AssumptionDefinition && before == OneShotObjectState(OneShotAbsent)
      case AssumptionEstablishmentRecorded => kind == AssumptionEstablishment && before == OneShotObjectState(OneShotAbsent)
      case AssumptionInvalidationRecorded => kind == AssumptionInvalidation && before == OneShotObjectState(OneShotAbsent)
      case FormalVerificationRecorded => kind == FormalVerification && before == OneShotObjectState(OneShotAbsent)
      case AssumptionVersionCompatibilityRecorded => kind == AssumptionVersionCompatibility && before == OneShotObjectState(OneShotAbsent)
      case AssumptionVersionCompatibilityInvalidated => kind == AssumptionVersionCompatibilityInvalidation && before == OneShotObjectState(OneShotAbsent)
      case BootstrapTrustDeclarationRecorded => kind == BootstrapTrustDeclaration && before == OneShotObjectState(OneShotAbsent)
      case BootstrapTrustInvalidated => kind == BootstrapTrustInvalidation && before == OneShotObjectState(OneShotAbsent)
      case FormalFindingClassificationRecorded => kind == FormalFindingClassification && before == OneShotObjectState(OneShotAbsent)
  }

  function ResultingState(
      kind: LifecycleObjectKind,
      event: EventType,
      before: LifecycleObjectState
  ): LifecycleObjectState
    requires LegalPredecessor(kind, event, before)
  {
    match event
      case Genesis => RegistryObjectState(InitializedAuthoritative)
      case FreezeAttemptStarted => FreezeAttemptObjectState(FreezeOpen)
      case FreezeCommitted => FreezeAttemptObjectState(FreezeCommittedState)
      case FreezeAbortedRecovery => FreezeAttemptObjectState(FreezeAbortedRecoveryState)
      case FreezeAbortedByOperatorAssertion => FreezeAttemptObjectState(FreezeAbortedByOperatorAssertionState)
      case FreezeCommitRejected => before
      case VerificationRecorded => OneShotObjectState(Recorded)
      case ReviewRequestRecorded => OneShotObjectState(Recorded)
      case ReviewResultRecorded => OneShotObjectState(Recorded)
      case ReviewAdmissionAccepted => ReviewAdmissionObjectState(AdmissionAccepted)
      case ReviewAdmissionRejected => ReviewAdmissionObjectState(AdmissionRejected)
      case PolicyRecorded => OneShotObjectState(Recorded)
      case CloseoutCommitted => CloseoutAttemptObjectState(CloseoutCommittedState)
      case CloseoutRejected => CloseoutAttemptObjectState(CloseoutRejectedState)
      case StorageCapabilityChanged => before
      case ArtifactEvictionStarted => ArtifactEvictionObjectState(EvictionOpen)
      case ArtifactEvictionCommitted => ArtifactEvictionObjectState(EvictionCommitted)
      case ArtifactEvictionInterruptedRecovery => ArtifactEvictionObjectState(EvictionInterruptedRecovery)
      case ArtifactEvictionBlocked => ArtifactEvictionObjectState(EvictionBlocked)
      case AssumptionDefinitionRecorded => OneShotObjectState(Recorded)
      case AssumptionEstablishmentRecorded => OneShotObjectState(Recorded)
      case AssumptionInvalidationRecorded => OneShotObjectState(Recorded)
      case FormalVerificationRecorded => OneShotObjectState(Recorded)
      case AssumptionVersionCompatibilityRecorded => OneShotObjectState(Recorded)
      case AssumptionVersionCompatibilityInvalidated => OneShotObjectState(Recorded)
      case BootstrapTrustDeclarationRecorded => OneShotObjectState(Recorded)
      case BootstrapTrustInvalidated => OneShotObjectState(Recorded)
      case FormalFindingClassificationRecorded => OneShotObjectState(Recorded)
  }

  // State-only transition relation. It cannot prove full authoritative Journal
  // admission because the remaining requirements are intentionally excluded
  // from this model's input domain.
  predicate LegalLifecycleTransition(
      kind: LifecycleObjectKind,
      before: LifecycleObjectState,
      event: EventType,
      after: LifecycleObjectState
  ) {
    LegalPredecessor(kind, event, before) && after == ResultingState(kind, event, before)
  }

  lemma StateObservingEventsDoNotTransition(
      kind: LifecycleObjectKind,
      before: LifecycleObjectState,
      event: EventType,
      after: LifecycleObjectState
  )
    requires LegalLifecycleTransition(kind, before, event, after)
    requires EventShapeOf(event) == ObserveNoStateChange
    ensures after == before
  {
    match event
      case FreezeCommitRejected =>
      case StorageCapabilityChanged =>
  }

  lemma CreateOpenRequiresAbsence(
      kind: LifecycleObjectKind,
      before: LifecycleObjectState,
      event: EventType,
      after: LifecycleObjectState
  )
    requires LegalLifecycleTransition(kind, before, event, after)
    requires EventShapeOf(event) == CreateOpen
    ensures before == FreezeAttemptObjectState(FreezeAbsent) ||
      before == ArtifactEvictionObjectState(EvictionAbsent)
  {
    match event
      case FreezeAttemptStarted =>
      case ArtifactEvictionStarted =>
  }

  lemma TerminalTransitionRequiresOpen(
      kind: LifecycleObjectKind,
      before: LifecycleObjectState,
      event: EventType,
      after: LifecycleObjectState
  )
    requires LegalLifecycleTransition(kind, before, event, after)
    requires EventShapeOf(event) == TransitionTerminal
    ensures before == FreezeAttemptObjectState(FreezeOpen) ||
      before == ArtifactEvictionObjectState(EvictionOpen)
  {
    match event
      case FreezeCommitted =>
      case FreezeAbortedRecovery =>
      case FreezeAbortedByOperatorAssertion =>
      case ArtifactEvictionCommitted =>
      case ArtifactEvictionInterruptedRecovery =>
      case ArtifactEvictionBlocked =>
  }

  // RED-first sentinel for Cross-Reference v0.3 §23 row 100, now discharged
  // by the lifecycle-state kernel above.
  lemma FreezeStartRequiresAbsence()
    ensures LegalPredecessor(
      FreezeAttempt,
      FreezeAttemptStarted,
      FreezeAttemptObjectState(FreezeAbsent))
  {
  }
}
