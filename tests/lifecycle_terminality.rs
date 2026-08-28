use evidence_registry::{
    ArtifactEvictionState, CloseoutAttemptState, FreezeAttemptState, LifecycleObjectKind,
    LifecycleObjectState, LifecycleStateValidationError, OneShotRecordedState,
    RegistryLifecycleState, ReviewAdmissionState,
};

#[test]
fn lifecycle_terminality_is_exact_and_rejects_invalid_kind_state_calls() {
    assert!(!LifecycleObjectKind::Registry.has_terminality());
    assert!(LifecycleObjectKind::FreezeAttempt.has_terminality());
    assert!(LifecycleObjectKind::Verification.has_terminality());
    assert!(LifecycleObjectKind::ReviewAdmissionAttempt.has_terminality());
    assert!(LifecycleObjectKind::CloseoutAttempt.has_terminality());
    assert!(LifecycleObjectKind::ArtifactEvictionAttempt.has_terminality());

    assert_eq!(
        LifecycleObjectKind::FreezeAttempt.is_terminal(LifecycleObjectState::FreezeAttempt(
            FreezeAttemptState::Committed,
        )),
        Ok(true)
    );
    assert_eq!(
        LifecycleObjectKind::FreezeAttempt.is_terminal(LifecycleObjectState::FreezeAttempt(
            FreezeAttemptState::Open,
        )),
        Ok(false)
    );
    assert_eq!(
        LifecycleObjectKind::Verification.is_terminal(LifecycleObjectState::OneShot(
            OneShotRecordedState::Recorded,
        )),
        Ok(true)
    );
    assert_eq!(
        LifecycleObjectKind::ReviewAdmissionAttempt.is_terminal(
            LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Rejected),
        ),
        Ok(true)
    );
    assert_eq!(
        LifecycleObjectKind::CloseoutAttempt.is_terminal(LifecycleObjectState::CloseoutAttempt(
            CloseoutAttemptState::Rejected,
        )),
        Ok(true)
    );
    assert_eq!(
        LifecycleObjectKind::ArtifactEvictionAttempt.is_terminal(
            LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Blocked),
        ),
        Ok(true)
    );

    assert_eq!(
        LifecycleObjectKind::Registry.is_terminal(LifecycleObjectState::Registry(
            RegistryLifecycleState::InitializedAuthoritative,
        )),
        Err(LifecycleStateValidationError::NoTerminality)
    );
    assert_eq!(
        LifecycleObjectKind::FreezeAttempt.is_terminal(LifecycleObjectState::Registry(
            RegistryLifecycleState::InitializedAuthoritative,
        )),
        Err(LifecycleStateValidationError::KindStateMismatch)
    );
}
