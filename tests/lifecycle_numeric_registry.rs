use evidence_registry::LifecycleObjectKind;

#[test]
fn lifecycle_object_kind_numeric_registry_is_exact_and_rejects_unassigned_values() {
    let cases = [
        (1, LifecycleObjectKind::Registry),
        (2, LifecycleObjectKind::FreezeAttempt),
        (3, LifecycleObjectKind::Verification),
        (4, LifecycleObjectKind::ReviewRequest),
        (5, LifecycleObjectKind::ReviewResult),
        (6, LifecycleObjectKind::ReviewAdmissionAttempt),
        (7, LifecycleObjectKind::Policy),
        (8, LifecycleObjectKind::CloseoutAttempt),
        (9, LifecycleObjectKind::ArtifactEvictionAttempt),
        (10, LifecycleObjectKind::AssumptionDefinition),
        (11, LifecycleObjectKind::AssumptionEstablishment),
        (12, LifecycleObjectKind::AssumptionInvalidation),
        (13, LifecycleObjectKind::FormalVerification),
        (14, LifecycleObjectKind::AssumptionVersionCompatibility),
        (
            15,
            LifecycleObjectKind::AssumptionVersionCompatibilityInvalidation,
        ),
        (16, LifecycleObjectKind::BootstrapTrustDeclaration),
        (17, LifecycleObjectKind::BootstrapTrustInvalidation),
        (18, LifecycleObjectKind::FormalFindingClassification),
    ];

    for (numeric_kind, expected_kind) in cases {
        assert_eq!(expected_kind.value(), numeric_kind, "kind {numeric_kind}");
        assert_eq!(
            LifecycleObjectKind::try_from(u64::from(numeric_kind)),
            Ok(expected_kind),
            "kind {numeric_kind}"
        );
    }

    for unassigned in [0, 19, u64::MAX] {
        let error = LifecycleObjectKind::try_from(unassigned).unwrap_err();
        assert_eq!(error.value(), unassigned, "unassigned kind {unassigned}");
    }
}
