use evidence_registry::{
    ArtifactEvictionState, CloseoutAttemptState, FreezeAttemptState, LifecycleObjectKind,
    LifecycleObjectState, LifecycleStateValidationError, OneShotRecordedState,
    RegistryLifecycleState, ReviewAdmissionState,
};

#[test]
fn lifecycle_terminality_exhaustively_covers_every_kind_and_state_representation() {
    let kinds = [
        LifecycleObjectKind::Registry,
        LifecycleObjectKind::FreezeAttempt,
        LifecycleObjectKind::Verification,
        LifecycleObjectKind::ReviewRequest,
        LifecycleObjectKind::ReviewResult,
        LifecycleObjectKind::ReviewAdmissionAttempt,
        LifecycleObjectKind::Policy,
        LifecycleObjectKind::CloseoutAttempt,
        LifecycleObjectKind::ArtifactEvictionAttempt,
        LifecycleObjectKind::AssumptionDefinition,
        LifecycleObjectKind::AssumptionEstablishment,
        LifecycleObjectKind::AssumptionInvalidation,
        LifecycleObjectKind::FormalVerification,
        LifecycleObjectKind::AssumptionVersionCompatibility,
        LifecycleObjectKind::AssumptionVersionCompatibilityInvalidation,
        LifecycleObjectKind::BootstrapTrustDeclaration,
        LifecycleObjectKind::BootstrapTrustInvalidation,
        LifecycleObjectKind::FormalFindingClassification,
    ];
    let states = [
        LifecycleObjectState::Registry(RegistryLifecycleState::Absent),
        LifecycleObjectState::Registry(RegistryLifecycleState::InitializedAuthoritative),
        LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Absent),
        LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Open),
        LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Committed),
        LifecycleObjectState::FreezeAttempt(FreezeAttemptState::AbortedRecovery),
        LifecycleObjectState::FreezeAttempt(FreezeAttemptState::AbortedByOperatorAssertion),
        LifecycleObjectState::OneShot(OneShotRecordedState::Absent),
        LifecycleObjectState::OneShot(OneShotRecordedState::Recorded),
        LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Absent),
        LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Accepted),
        LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Rejected),
        LifecycleObjectState::CloseoutAttempt(CloseoutAttemptState::Absent),
        LifecycleObjectState::CloseoutAttempt(CloseoutAttemptState::Committed),
        LifecycleObjectState::CloseoutAttempt(CloseoutAttemptState::Rejected),
        LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Absent),
        LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Open),
        LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Committed),
        LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::InterruptedRecovery),
        LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Blocked),
    ];

    for kind in kinds {
        for state in states {
            let expected = expected_terminality(kind, state);
            assert_eq!(kind.is_terminal(state), expected, "{kind:?} / {state:?}");
        }
    }
}

fn expected_terminality(
    kind: LifecycleObjectKind,
    state: LifecycleObjectState,
) -> Result<bool, LifecycleStateValidationError> {
    match kind {
        LifecycleObjectKind::Registry => Err(LifecycleStateValidationError::NoTerminality),
        LifecycleObjectKind::FreezeAttempt => match state {
            LifecycleObjectState::FreezeAttempt(value) => Ok(matches!(
                value,
                FreezeAttemptState::Committed
                    | FreezeAttemptState::AbortedRecovery
                    | FreezeAttemptState::AbortedByOperatorAssertion
            )),
            _ => Err(LifecycleStateValidationError::KindStateMismatch),
        },
        LifecycleObjectKind::ReviewAdmissionAttempt => match state {
            LifecycleObjectState::ReviewAdmission(value) => Ok(matches!(
                value,
                ReviewAdmissionState::Accepted | ReviewAdmissionState::Rejected
            )),
            _ => Err(LifecycleStateValidationError::KindStateMismatch),
        },
        LifecycleObjectKind::CloseoutAttempt => match state {
            LifecycleObjectState::CloseoutAttempt(value) => Ok(matches!(
                value,
                CloseoutAttemptState::Committed | CloseoutAttemptState::Rejected
            )),
            _ => Err(LifecycleStateValidationError::KindStateMismatch),
        },
        LifecycleObjectKind::ArtifactEvictionAttempt => match state {
            LifecycleObjectState::ArtifactEviction(value) => Ok(matches!(
                value,
                ArtifactEvictionState::Committed
                    | ArtifactEvictionState::InterruptedRecovery
                    | ArtifactEvictionState::Blocked
            )),
            _ => Err(LifecycleStateValidationError::KindStateMismatch),
        },
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
        | LifecycleObjectKind::FormalFindingClassification => match state {
            LifecycleObjectState::OneShot(value) => Ok(value == OneShotRecordedState::Recorded),
            _ => Err(LifecycleStateValidationError::KindStateMismatch),
        },
    }
}
