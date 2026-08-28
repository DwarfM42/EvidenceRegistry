use evidence_registry::{EventShape, EventTypeId, LifecycleObjectKind};

#[test]
fn lifecycle_event_classification_covers_distinct_normative_shapes_and_kinds() {
    let genesis = EventTypeId::try_from(1u64).unwrap();
    assert_eq!(
        genesis.lifecycle_object_kind(),
        LifecycleObjectKind::Registry
    );
    assert_eq!(genesis.event_shape(), EventShape::BootstrapCreate);

    let freeze_start = EventTypeId::try_from(100u64).unwrap();
    assert_eq!(
        freeze_start.lifecycle_object_kind(),
        LifecycleObjectKind::FreezeAttempt
    );
    assert_eq!(freeze_start.event_shape(), EventShape::CreateOpen);

    let freeze_commit = EventTypeId::try_from(101u64).unwrap();
    assert_eq!(freeze_commit.event_shape(), EventShape::TransitionTerminal);

    let freeze_rejected = EventTypeId::try_from(104u64).unwrap();
    assert_eq!(
        freeze_rejected.event_shape(),
        EventShape::ObserveNoStateChange
    );

    let verification = EventTypeId::try_from(200u64).unwrap();
    assert_eq!(
        verification.lifecycle_object_kind(),
        LifecycleObjectKind::Verification
    );
    assert_eq!(verification.event_shape(), EventShape::CreateTerminal);

    let storage_change = EventTypeId::try_from(600u64).unwrap();
    assert_eq!(
        storage_change.lifecycle_object_kind(),
        LifecycleObjectKind::Registry
    );
    assert_eq!(
        storage_change.event_shape(),
        EventShape::ObserveNoStateChange
    );

    let eviction_start = EventTypeId::try_from(700u64).unwrap();
    assert_eq!(
        eviction_start.lifecycle_object_kind(),
        LifecycleObjectKind::ArtifactEvictionAttempt
    );
    assert_eq!(eviction_start.event_shape(), EventShape::CreateOpen);

    let classification = EventTypeId::try_from(808u64).unwrap();
    assert_eq!(
        classification.lifecycle_object_kind(),
        LifecycleObjectKind::FormalFindingClassification
    );
    assert_eq!(classification.event_shape(), EventShape::CreateTerminal);
}

#[test]
fn lifecycle_event_classification_covers_every_registered_v0x_event() {
    let cases = [
        (
            1,
            LifecycleObjectKind::Registry,
            EventShape::BootstrapCreate,
        ),
        (
            100,
            LifecycleObjectKind::FreezeAttempt,
            EventShape::CreateOpen,
        ),
        (
            101,
            LifecycleObjectKind::FreezeAttempt,
            EventShape::TransitionTerminal,
        ),
        (
            102,
            LifecycleObjectKind::FreezeAttempt,
            EventShape::TransitionTerminal,
        ),
        (
            103,
            LifecycleObjectKind::FreezeAttempt,
            EventShape::TransitionTerminal,
        ),
        (
            104,
            LifecycleObjectKind::FreezeAttempt,
            EventShape::ObserveNoStateChange,
        ),
        (
            200,
            LifecycleObjectKind::Verification,
            EventShape::CreateTerminal,
        ),
        (
            300,
            LifecycleObjectKind::ReviewRequest,
            EventShape::CreateTerminal,
        ),
        (
            301,
            LifecycleObjectKind::ReviewResult,
            EventShape::CreateTerminal,
        ),
        (
            302,
            LifecycleObjectKind::ReviewAdmissionAttempt,
            EventShape::CreateTerminal,
        ),
        (
            303,
            LifecycleObjectKind::ReviewAdmissionAttempt,
            EventShape::CreateTerminal,
        ),
        (400, LifecycleObjectKind::Policy, EventShape::CreateTerminal),
        (
            500,
            LifecycleObjectKind::CloseoutAttempt,
            EventShape::CreateTerminal,
        ),
        (
            501,
            LifecycleObjectKind::CloseoutAttempt,
            EventShape::CreateTerminal,
        ),
        (
            600,
            LifecycleObjectKind::Registry,
            EventShape::ObserveNoStateChange,
        ),
        (
            700,
            LifecycleObjectKind::ArtifactEvictionAttempt,
            EventShape::CreateOpen,
        ),
        (
            701,
            LifecycleObjectKind::ArtifactEvictionAttempt,
            EventShape::TransitionTerminal,
        ),
        (
            702,
            LifecycleObjectKind::ArtifactEvictionAttempt,
            EventShape::TransitionTerminal,
        ),
        (
            703,
            LifecycleObjectKind::ArtifactEvictionAttempt,
            EventShape::TransitionTerminal,
        ),
        (
            800,
            LifecycleObjectKind::AssumptionDefinition,
            EventShape::CreateTerminal,
        ),
        (
            801,
            LifecycleObjectKind::AssumptionEstablishment,
            EventShape::CreateTerminal,
        ),
        (
            802,
            LifecycleObjectKind::AssumptionInvalidation,
            EventShape::CreateTerminal,
        ),
        (
            803,
            LifecycleObjectKind::FormalVerification,
            EventShape::CreateTerminal,
        ),
        (
            804,
            LifecycleObjectKind::AssumptionVersionCompatibility,
            EventShape::CreateTerminal,
        ),
        (
            805,
            LifecycleObjectKind::AssumptionVersionCompatibilityInvalidation,
            EventShape::CreateTerminal,
        ),
        (
            806,
            LifecycleObjectKind::BootstrapTrustDeclaration,
            EventShape::CreateTerminal,
        ),
        (
            807,
            LifecycleObjectKind::BootstrapTrustInvalidation,
            EventShape::CreateTerminal,
        ),
        (
            808,
            LifecycleObjectKind::FormalFindingClassification,
            EventShape::CreateTerminal,
        ),
    ];

    assert_eq!(cases.len(), 28);
    for (event_id, expected_kind, expected_shape) in cases {
        let event = EventTypeId::try_from(event_id).unwrap();
        assert_eq!(
            event.lifecycle_object_kind(),
            expected_kind,
            "event {event_id}"
        );
        assert_eq!(event.event_shape(), expected_shape, "event {event_id}");
    }
}
