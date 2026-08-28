use evidence_registry::{
    ArtifactEvictionState, CloseoutAttemptState, EventTypeId, FreezeAttemptState,
    LifecycleObjectState, LifecycleTransitionError, OneShotRecordedState, RegistryLifecycleState,
    ReviewAdmissionState,
};

#[test]
fn every_registered_event_accepts_only_its_exact_matching_kind_predecessors() {
    use ArtifactEvictionState::{
        Absent as EvictionAbsent, Blocked, Committed as EvictionCommitted, InterruptedRecovery,
        Open as EvictionOpen,
    };
    use CloseoutAttemptState::{
        Absent as CloseoutAbsent, Committed as CloseoutCommitted, Rejected as CloseoutRejected,
    };
    use FreezeAttemptState::{
        AbortedByOperatorAssertion, AbortedRecovery, Absent as FreezeAbsent,
        Committed as FreezeCommitted, Open as FreezeOpen,
    };
    use LifecycleObjectState::{
        ArtifactEviction, CloseoutAttempt, FreezeAttempt, OneShot, Registry, ReviewAdmission,
    };
    use OneShotRecordedState::{Absent as OneShotAbsent, Recorded};
    use RegistryLifecycleState::{Absent as RegistryAbsent, InitializedAuthoritative};
    use ReviewAdmissionState::{Absent as ReviewAbsent, Accepted, Rejected};

    let genesis = [(Registry(RegistryAbsent), Registry(InitializedAuthoritative))];
    let freeze_open = [(FreezeAttempt(FreezeAbsent), FreezeAttempt(FreezeOpen))];
    let freeze_commit = [(FreezeAttempt(FreezeOpen), FreezeAttempt(FreezeCommitted))];
    let freeze_abort_recovery = [(FreezeAttempt(FreezeOpen), FreezeAttempt(AbortedRecovery))];
    let freeze_abort_operator = [(
        FreezeAttempt(FreezeOpen),
        FreezeAttempt(AbortedByOperatorAssertion),
    )];
    let freeze_observe = [
        (
            FreezeAttempt(FreezeCommitted),
            FreezeAttempt(FreezeCommitted),
        ),
        (
            FreezeAttempt(AbortedRecovery),
            FreezeAttempt(AbortedRecovery),
        ),
        (
            FreezeAttempt(AbortedByOperatorAssertion),
            FreezeAttempt(AbortedByOperatorAssertion),
        ),
    ];
    let one_shot = [(OneShot(OneShotAbsent), OneShot(Recorded))];
    let admission_accept = [(ReviewAdmission(ReviewAbsent), ReviewAdmission(Accepted))];
    let admission_reject = [(ReviewAdmission(ReviewAbsent), ReviewAdmission(Rejected))];
    let closeout_commit = [(
        CloseoutAttempt(CloseoutAbsent),
        CloseoutAttempt(CloseoutCommitted),
    )];
    let closeout_reject = [(
        CloseoutAttempt(CloseoutAbsent),
        CloseoutAttempt(CloseoutRejected),
    )];
    let registry_observe = [(
        Registry(InitializedAuthoritative),
        Registry(InitializedAuthoritative),
    )];
    let eviction_open = [(
        ArtifactEviction(EvictionAbsent),
        ArtifactEviction(EvictionOpen),
    )];
    let eviction_commit = [(
        ArtifactEviction(EvictionOpen),
        ArtifactEviction(EvictionCommitted),
    )];
    let eviction_interrupt = [(
        ArtifactEviction(EvictionOpen),
        ArtifactEviction(InterruptedRecovery),
    )];
    let eviction_block = [(ArtifactEviction(EvictionOpen), ArtifactEviction(Blocked))];

    let cases = [
        (1, &genesis[..]),
        (100, &freeze_open[..]),
        (101, &freeze_commit[..]),
        (102, &freeze_abort_recovery[..]),
        (103, &freeze_abort_operator[..]),
        (104, &freeze_observe[..]),
        (200, &one_shot[..]),
        (300, &one_shot[..]),
        (301, &one_shot[..]),
        (302, &admission_accept[..]),
        (303, &admission_reject[..]),
        (400, &one_shot[..]),
        (500, &closeout_commit[..]),
        (501, &closeout_reject[..]),
        (600, &registry_observe[..]),
        (700, &eviction_open[..]),
        (701, &eviction_commit[..]),
        (702, &eviction_interrupt[..]),
        (703, &eviction_block[..]),
        (800, &one_shot[..]),
        (801, &one_shot[..]),
        (802, &one_shot[..]),
        (803, &one_shot[..]),
        (804, &one_shot[..]),
        (805, &one_shot[..]),
        (806, &one_shot[..]),
        (807, &one_shot[..]),
        (808, &one_shot[..]),
    ];

    for (event_id, legal_transitions) in cases {
        let event = EventTypeId::try_from(event_id).unwrap();
        let kind = event.lifecycle_object_kind();
        for before in matching_states(kind) {
            let expected = legal_transitions
                .iter()
                .find(|(legal_before, _)| *legal_before == before)
                .map(|(_, result)| *result);
            assert_eq!(
                event.has_legal_predecessor(before),
                expected.is_some(),
                "event {event_id}, predecessor {before:?}"
            );
            assert_eq!(
                event.resulting_state(before),
                expected.ok_or(LifecycleTransitionError::IllegalPredecessor),
                "event {event_id}, predecessor {before:?}"
            );
        }

        let mismatched = match kind {
            evidence_registry::LifecycleObjectKind::Registry => OneShot(OneShotAbsent),
            _ => Registry(RegistryAbsent),
        };
        assert_eq!(
            event.resulting_state(mismatched),
            Err(LifecycleTransitionError::KindStateMismatch),
            "event {event_id}, mismatched predecessor {mismatched:?}"
        );
    }
}

fn matching_states(kind: evidence_registry::LifecycleObjectKind) -> Vec<LifecycleObjectState> {
    match kind {
        evidence_registry::LifecycleObjectKind::Registry => vec![
            LifecycleObjectState::Registry(RegistryLifecycleState::Absent),
            LifecycleObjectState::Registry(RegistryLifecycleState::InitializedAuthoritative),
        ],
        evidence_registry::LifecycleObjectKind::FreezeAttempt => vec![
            LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Absent),
            LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Open),
            LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Committed),
            LifecycleObjectState::FreezeAttempt(FreezeAttemptState::AbortedRecovery),
            LifecycleObjectState::FreezeAttempt(FreezeAttemptState::AbortedByOperatorAssertion),
        ],
        evidence_registry::LifecycleObjectKind::ReviewAdmissionAttempt => vec![
            LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Absent),
            LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Accepted),
            LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Rejected),
        ],
        evidence_registry::LifecycleObjectKind::CloseoutAttempt => vec![
            LifecycleObjectState::CloseoutAttempt(CloseoutAttemptState::Absent),
            LifecycleObjectState::CloseoutAttempt(CloseoutAttemptState::Committed),
            LifecycleObjectState::CloseoutAttempt(CloseoutAttemptState::Rejected),
        ],
        evidence_registry::LifecycleObjectKind::ArtifactEvictionAttempt => vec![
            LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Absent),
            LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Open),
            LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Committed),
            LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::InterruptedRecovery),
            LifecycleObjectState::ArtifactEviction(ArtifactEvictionState::Blocked),
        ],
        _ => vec![
            LifecycleObjectState::OneShot(OneShotRecordedState::Absent),
            LifecycleObjectState::OneShot(OneShotRecordedState::Recorded),
        ],
    }
}
