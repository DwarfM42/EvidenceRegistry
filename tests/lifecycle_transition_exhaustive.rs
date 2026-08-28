use evidence_registry::{
    ArtifactEvictionState, CloseoutAttemptState, EventShape, EventTypeId, FreezeAttemptState,
    LifecycleObjectKind, LifecycleObjectState, LifecycleTransitionError, OneShotRecordedState,
    RegistryLifecycleState, ReviewAdmissionState,
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
        (
            1,
            LifecycleObjectKind::Registry,
            EventShape::BootstrapCreate,
            &genesis[..],
        ),
        (
            100,
            LifecycleObjectKind::FreezeAttempt,
            EventShape::CreateOpen,
            &freeze_open[..],
        ),
        (
            101,
            LifecycleObjectKind::FreezeAttempt,
            EventShape::TransitionTerminal,
            &freeze_commit[..],
        ),
        (
            102,
            LifecycleObjectKind::FreezeAttempt,
            EventShape::TransitionTerminal,
            &freeze_abort_recovery[..],
        ),
        (
            103,
            LifecycleObjectKind::FreezeAttempt,
            EventShape::TransitionTerminal,
            &freeze_abort_operator[..],
        ),
        (
            104,
            LifecycleObjectKind::FreezeAttempt,
            EventShape::ObserveNoStateChange,
            &freeze_observe[..],
        ),
        (
            200,
            LifecycleObjectKind::Verification,
            EventShape::CreateTerminal,
            &one_shot[..],
        ),
        (
            300,
            LifecycleObjectKind::ReviewRequest,
            EventShape::CreateTerminal,
            &one_shot[..],
        ),
        (
            301,
            LifecycleObjectKind::ReviewResult,
            EventShape::CreateTerminal,
            &one_shot[..],
        ),
        (
            302,
            LifecycleObjectKind::ReviewAdmissionAttempt,
            EventShape::CreateTerminal,
            &admission_accept[..],
        ),
        (
            303,
            LifecycleObjectKind::ReviewAdmissionAttempt,
            EventShape::CreateTerminal,
            &admission_reject[..],
        ),
        (
            400,
            LifecycleObjectKind::Policy,
            EventShape::CreateTerminal,
            &one_shot[..],
        ),
        (
            500,
            LifecycleObjectKind::CloseoutAttempt,
            EventShape::CreateTerminal,
            &closeout_commit[..],
        ),
        (
            501,
            LifecycleObjectKind::CloseoutAttempt,
            EventShape::CreateTerminal,
            &closeout_reject[..],
        ),
        (
            600,
            LifecycleObjectKind::Registry,
            EventShape::ObserveNoStateChange,
            &registry_observe[..],
        ),
        (
            700,
            LifecycleObjectKind::ArtifactEvictionAttempt,
            EventShape::CreateOpen,
            &eviction_open[..],
        ),
        (
            701,
            LifecycleObjectKind::ArtifactEvictionAttempt,
            EventShape::TransitionTerminal,
            &eviction_commit[..],
        ),
        (
            702,
            LifecycleObjectKind::ArtifactEvictionAttempt,
            EventShape::TransitionTerminal,
            &eviction_interrupt[..],
        ),
        (
            703,
            LifecycleObjectKind::ArtifactEvictionAttempt,
            EventShape::TransitionTerminal,
            &eviction_block[..],
        ),
        (
            800,
            LifecycleObjectKind::AssumptionDefinition,
            EventShape::CreateTerminal,
            &one_shot[..],
        ),
        (
            801,
            LifecycleObjectKind::AssumptionEstablishment,
            EventShape::CreateTerminal,
            &one_shot[..],
        ),
        (
            802,
            LifecycleObjectKind::AssumptionInvalidation,
            EventShape::CreateTerminal,
            &one_shot[..],
        ),
        (
            803,
            LifecycleObjectKind::FormalVerification,
            EventShape::CreateTerminal,
            &one_shot[..],
        ),
        (
            804,
            LifecycleObjectKind::AssumptionVersionCompatibility,
            EventShape::CreateTerminal,
            &one_shot[..],
        ),
        (
            805,
            LifecycleObjectKind::AssumptionVersionCompatibilityInvalidation,
            EventShape::CreateTerminal,
            &one_shot[..],
        ),
        (
            806,
            LifecycleObjectKind::BootstrapTrustDeclaration,
            EventShape::CreateTerminal,
            &one_shot[..],
        ),
        (
            807,
            LifecycleObjectKind::BootstrapTrustInvalidation,
            EventShape::CreateTerminal,
            &one_shot[..],
        ),
        (
            808,
            LifecycleObjectKind::FormalFindingClassification,
            EventShape::CreateTerminal,
            &one_shot[..],
        ),
    ];

    for (event_id, expected_kind, expected_shape, legal_transitions) in cases {
        let event = EventTypeId::try_from(event_id).unwrap();
        assert_eq!(
            event.value(),
            event_id as u16,
            "event {event_id} numeric round trip"
        );
        assert_eq!(
            event.lifecycle_object_kind(),
            expected_kind,
            "event {event_id} object kind"
        );
        assert_eq!(
            event.event_shape(),
            expected_shape,
            "event {event_id} shape"
        );
        for before in matching_states(expected_kind) {
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

        let mismatched = match expected_kind {
            LifecycleObjectKind::Registry => OneShot(OneShotAbsent),
            _ => Registry(RegistryAbsent),
        };
        assert_eq!(
            event.resulting_state(mismatched),
            Err(LifecycleTransitionError::KindStateMismatch),
            "event {event_id}, mismatched predecessor {mismatched:?}"
        );
    }
}

#[test]
fn event_type_id_rejects_every_unregistered_value_through_the_registry_boundary() {
    let registered = [
        1_u64, 100, 101, 102, 103, 104, 200, 300, 301, 302, 303, 400, 500, 501, 600, 700, 701, 702,
        703, 800, 801, 802, 803, 804, 805, 806, 807, 808,
    ];

    for value in 0_u64..=809 {
        if registered.contains(&value) {
            continue;
        }
        let error = EventTypeId::try_from(value).unwrap_err();
        assert_eq!(error.value(), value, "unregistered event {value}");
    }

    for value in [810, u64::MAX] {
        let error = EventTypeId::try_from(value).unwrap_err();
        assert_eq!(error.value(), value, "unregistered event {value}");
    }
}

fn matching_states(kind: LifecycleObjectKind) -> Vec<LifecycleObjectState> {
    match kind {
        LifecycleObjectKind::Registry => vec![
            LifecycleObjectState::Registry(RegistryLifecycleState::Absent),
            LifecycleObjectState::Registry(RegistryLifecycleState::InitializedAuthoritative),
        ],
        LifecycleObjectKind::FreezeAttempt => vec![
            LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Absent),
            LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Open),
            LifecycleObjectState::FreezeAttempt(FreezeAttemptState::Committed),
            LifecycleObjectState::FreezeAttempt(FreezeAttemptState::AbortedRecovery),
            LifecycleObjectState::FreezeAttempt(FreezeAttemptState::AbortedByOperatorAssertion),
        ],
        LifecycleObjectKind::ReviewAdmissionAttempt => vec![
            LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Absent),
            LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Accepted),
            LifecycleObjectState::ReviewAdmission(ReviewAdmissionState::Rejected),
        ],
        LifecycleObjectKind::CloseoutAttempt => vec![
            LifecycleObjectState::CloseoutAttempt(CloseoutAttemptState::Absent),
            LifecycleObjectState::CloseoutAttempt(CloseoutAttemptState::Committed),
            LifecycleObjectState::CloseoutAttempt(CloseoutAttemptState::Rejected),
        ],
        LifecycleObjectKind::ArtifactEvictionAttempt => vec![
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
