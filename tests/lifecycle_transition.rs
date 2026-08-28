use evidence_registry::{
    ArtifactEvictionState, EventTypeId, FreezeAttemptState, LifecycleObjectState,
    LifecycleTransitionError, OneShotRecordedState, RegistryLifecycleState,
};

#[test]
fn lifecycle_state_transition_kernel_requires_the_exact_table_predecessor() {
    let genesis = EventTypeId::try_from(1u64).unwrap();
    let registry_absent = LifecycleObjectState::Registry(RegistryLifecycleState::Absent);
    let registry_initialized =
        LifecycleObjectState::Registry(RegistryLifecycleState::InitializedAuthoritative);
    assert!(genesis.has_legal_predecessor(registry_absent));
    assert_eq!(
        genesis.resulting_state(registry_absent),
        Ok(registry_initialized)
    );
    assert!(!genesis.has_legal_predecessor(registry_initialized));

    let freeze_start = EventTypeId::try_from(100u64).unwrap();
    let freeze_open = LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Open);
    assert_eq!(
        freeze_start.resulting_state(LifecycleObjectState::FreezeAttempt(
            FreezeAttemptState::Absent,
        )),
        Ok(freeze_open)
    );

    let freeze_commit = EventTypeId::try_from(101u64).unwrap();
    let freeze_committed = LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Committed);
    assert_eq!(
        freeze_commit.resulting_state(freeze_open),
        Ok(freeze_committed)
    );

    let freeze_rejected = EventTypeId::try_from(104u64).unwrap();
    assert_eq!(
        freeze_rejected.resulting_state(freeze_committed),
        Ok(freeze_committed)
    );
    assert_eq!(
        freeze_rejected.resulting_state(freeze_open),
        Err(LifecycleTransitionError::IllegalPredecessor)
    );

    let verification = EventTypeId::try_from(200u64).unwrap();
    let one_shot_absent = LifecycleObjectState::OneShot(OneShotRecordedState::Absent);
    let one_shot_recorded = LifecycleObjectState::OneShot(OneShotRecordedState::Recorded);
    assert_eq!(
        verification.resulting_state(one_shot_absent),
        Ok(one_shot_recorded)
    );
    assert_eq!(
        verification.resulting_state(one_shot_recorded),
        Err(LifecycleTransitionError::IllegalPredecessor)
    );

    let storage_change = EventTypeId::try_from(600u64).unwrap();
    assert_eq!(
        storage_change.resulting_state(registry_initialized),
        Ok(registry_initialized)
    );

    let eviction_started = EventTypeId::try_from(700u64).unwrap();
    let eviction_open = LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Open);
    assert_eq!(
        eviction_started.resulting_state(LifecycleObjectState::ArtifactEviction(
            ArtifactEvictionState::Absent,
        )),
        Ok(eviction_open)
    );
    let eviction_blocked = EventTypeId::try_from(703u64).unwrap();
    assert_eq!(
        eviction_blocked.resulting_state(eviction_open),
        Ok(LifecycleObjectState::ArtifactEviction(
            ArtifactEvictionState::Blocked,
        ))
    );

    assert_eq!(
        freeze_start.resulting_state(registry_absent),
        Err(LifecycleTransitionError::KindStateMismatch)
    );
}

#[test]
fn lifecycle_state_transition_kernel_covers_every_registered_v0x_event() {
    let registry_absent = LifecycleObjectState::Registry(RegistryLifecycleState::Absent);
    let registry_initialized =
        LifecycleObjectState::Registry(RegistryLifecycleState::InitializedAuthoritative);
    let freeze_absent = LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Absent);
    let freeze_open = LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Open);
    let freeze_committed = LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Committed);
    let one_shot_absent = LifecycleObjectState::OneShot(OneShotRecordedState::Absent);
    let one_shot_recorded = LifecycleObjectState::OneShot(OneShotRecordedState::Recorded);
    let admission_absent =
        LifecycleObjectState::ReviewAdmission(evidence_registry::ReviewAdmissionState::Absent);
    let closeout_absent =
        LifecycleObjectState::CloseoutAttempt(evidence_registry::CloseoutAttemptState::Absent);
    let eviction_absent = LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Absent);
    let eviction_open = LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Open);

    let cases = [
        (1, registry_absent, registry_initialized),
        (100, freeze_absent, freeze_open),
        (101, freeze_open, freeze_committed),
        (
            102,
            freeze_open,
            LifecycleObjectState::FreezeAttempt(FreezeAttemptState::AbortedRecovery),
        ),
        (
            103,
            freeze_open,
            LifecycleObjectState::FreezeAttempt(FreezeAttemptState::AbortedByOperatorAssertion),
        ),
        (104, freeze_committed, freeze_committed),
        (200, one_shot_absent, one_shot_recorded),
        (300, one_shot_absent, one_shot_recorded),
        (301, one_shot_absent, one_shot_recorded),
        (
            302,
            admission_absent,
            LifecycleObjectState::ReviewAdmission(
                evidence_registry::ReviewAdmissionState::Accepted,
            ),
        ),
        (
            303,
            admission_absent,
            LifecycleObjectState::ReviewAdmission(
                evidence_registry::ReviewAdmissionState::Rejected,
            ),
        ),
        (400, one_shot_absent, one_shot_recorded),
        (
            500,
            closeout_absent,
            LifecycleObjectState::CloseoutAttempt(
                evidence_registry::CloseoutAttemptState::Committed,
            ),
        ),
        (
            501,
            closeout_absent,
            LifecycleObjectState::CloseoutAttempt(
                evidence_registry::CloseoutAttemptState::Rejected,
            ),
        ),
        (600, registry_initialized, registry_initialized),
        (700, eviction_absent, eviction_open),
        (
            701,
            eviction_open,
            LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Committed),
        ),
        (
            702,
            eviction_open,
            LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::InterruptedRecovery),
        ),
        (
            703,
            eviction_open,
            LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Blocked),
        ),
        (800, one_shot_absent, one_shot_recorded),
        (801, one_shot_absent, one_shot_recorded),
        (802, one_shot_absent, one_shot_recorded),
        (803, one_shot_absent, one_shot_recorded),
        (804, one_shot_absent, one_shot_recorded),
        (805, one_shot_absent, one_shot_recorded),
        (806, one_shot_absent, one_shot_recorded),
        (807, one_shot_absent, one_shot_recorded),
        (808, one_shot_absent, one_shot_recorded),
    ];

    assert_eq!(cases.len(), 28);
    for (event_id, before, expected_after) in cases {
        assert_eq!(
            EventTypeId::try_from(event_id)
                .unwrap()
                .resulting_state(before),
            Ok(expected_after),
            "event {event_id}"
        );
    }
}
