use evidence_registry::{
    compose_review_admission_policy_46, derive_review_admission_lifecycle_outcome,
    evaluate_review_admission_gate_scope_1015, policy_evaluator_1015_field_mapping,
    policy_evaluator_registry, Evaluator1015Result, ExactRecordByteResolver,
    PolicyCompositionResult, PolicyEvaluationContext, RecordId, ReviewAdmissionGateScopeEvaluation,
    ReviewAdmissionGateScopePolicy, ReviewAdmissionLifecycleOutcome, ReviewAdmissionPreCompletion,
    ReviewAdmissionPrerequisiteFailure, StrictRecordFrame,
};

const SCOPE_PROFILE_1_EMPTY_HEX: &str =
    "84781a45766964656e636552656769737472792e5265636f72642e76311401a500010114100111011240";
const SCOPE_PROFILE_1_EMPTY_ID_HEX: &str =
    "f48b8a58a2659513c269949f40480dc7872d1df9e03bba43fe9aec84c8242147";
const SCOPE_PROFILE_1_EMPTY_LABEL_HEX: &str =
    "84781a45766964656e636552656769737472792e5265636f72642e76311401a60001011410011101124013656c6162656c";
const SCOPE_PROFILE_1_EMPTY_LABEL_ID_HEX: &str =
    "ea44cf7657f811a92dfa107fd9f9506cab525384ddf7538d6395cee48f2cb7a4";

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}

fn record_id(hex: &str) -> RecordId {
    RecordId::try_from(hex_bytes(hex).as_slice()).unwrap()
}

fn record_id_from_bytes(bytes: &[u8]) -> RecordId {
    StrictRecordFrame::decode_authoritative(bytes)
        .unwrap()
        .record_id()
}

fn scope_bytes(
    profile_id: u8,
    profile_version: u8,
    payload: &[u8],
    label: Option<&str>,
) -> Vec<u8> {
    assert!(payload.len() < 24);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x14, 0x01, if label.is_some() { 0xa6 } else { 0xa5 }]);
    bytes.extend_from_slice(&[
        0x00,
        0x01,
        0x01,
        0x14,
        0x10,
        profile_id,
        0x11,
        profile_version,
    ]);
    bytes.extend_from_slice(&[0x12, 0x40 + payload.len() as u8]);
    bytes.extend_from_slice(payload);
    if let Some(label) = label {
        assert!(label.len() < 24);
        bytes.extend_from_slice(&[0x13, 0x60 + label.len() as u8]);
        bytes.extend_from_slice(label.as_bytes());
    }
    bytes
}

struct FixtureRecordResolver {
    records: Vec<(RecordId, Vec<u8>)>,
}

impl ExactRecordByteResolver for FixtureRecordResolver {
    fn resolve(&self, record_id: RecordId) -> Option<&[u8]> {
        self.records
            .iter()
            .find_map(|(stored_id, bytes)| (*stored_id == record_id).then_some(bytes.as_slice()))
    }
}

fn fixture_scope() -> (RecordId, Vec<u8>) {
    (
        record_id(SCOPE_PROFILE_1_EMPTY_ID_HEX),
        hex_bytes(SCOPE_PROFILE_1_EMPTY_HEX),
    )
}

fn policy(gate_scope_ref: RecordId) -> ReviewAdmissionGateScopePolicy {
    ReviewAdmissionGateScopePolicy::new(gate_scope_ref, vec![2])
}

#[test]
fn e1015_exact_match_passes_then_participates_in_normal_policy_composition() {
    let (scope_id, scope) = fixture_scope();
    let resolver = FixtureRecordResolver {
        records: vec![(scope_id, scope)],
    };
    let evaluated = evaluate_review_admission_gate_scope_1015(
        Some(&policy(scope_id)),
        scope_id,
        scope_id,
        &resolver,
    );

    assert_eq!(
        evaluated,
        ReviewAdmissionGateScopeEvaluation::Completed(Evaluator1015Result::Pass)
    );
    assert_eq!(
        compose_review_admission_policy_46(evaluated, &[]),
        PolicyCompositionResult::Satisfied
    );
}

#[test]
fn e1015_exact_mismatch_is_a_real_failure_and_reaches_event_303_only_after_composition() {
    let (scope_id, scope) = fixture_scope();
    let different_scope_bytes = scope_bytes(1, 1, b"", Some("different identity"));
    let different_scope_id = record_id_from_bytes(&different_scope_bytes);
    let resolver = FixtureRecordResolver {
        records: vec![
            (scope_id, scope),
            (different_scope_id, different_scope_bytes),
        ],
    };
    let evaluated = evaluate_review_admission_gate_scope_1015(
        Some(&policy(different_scope_id)),
        scope_id,
        scope_id,
        &resolver,
    );

    assert_eq!(
        evaluated,
        ReviewAdmissionGateScopeEvaluation::Completed(Evaluator1015Result::Fail)
    );
    let composed = compose_review_admission_policy_46(evaluated, &[]);
    assert_eq!(composed, PolicyCompositionResult::GateUnsatisfied);
    assert_eq!(
        derive_review_admission_lifecycle_outcome(composed),
        ReviewAdmissionLifecycleOutcome::RejectedEvent303
    );
}

#[test]
fn e1015_section_82_scope_mismatch_short_circuits_before_evaluator_or_policy_result() {
    let (scope_id, scope) = fixture_scope();
    let other_scope_bytes = scope_bytes(1, 1, b"", Some("other review scope"));
    let other_scope_id = record_id_from_bytes(&other_scope_bytes);
    let resolver = FixtureRecordResolver {
        records: vec![(scope_id, scope), (other_scope_id, other_scope_bytes)],
    };
    let evaluated = evaluate_review_admission_gate_scope_1015(
        Some(&policy(scope_id)),
        scope_id,
        other_scope_id,
        &resolver,
    );

    assert_eq!(
        evaluated,
        ReviewAdmissionGateScopeEvaluation::PrerequisiteFailure(
            ReviewAdmissionPrerequisiteFailure::ReviewScopeMismatch
        )
    );
    assert_eq!(
        compose_review_admission_policy_46(evaluated, &[]),
        PolicyCompositionResult::PreCompletion
    );
    assert_eq!(
        derive_review_admission_lifecycle_outcome(PolicyCompositionResult::PreCompletion),
        ReviewAdmissionLifecycleOutcome::NoEvent
    );
}

#[test]
fn e1015_profile_1_1_requires_empty_payload() {
    let non_empty_scope = scope_bytes(1, 1, b"x", None);
    let non_empty_scope_id = record_id_from_bytes(&non_empty_scope);
    let resolver = FixtureRecordResolver {
        records: vec![(non_empty_scope_id, non_empty_scope)],
    };
    let evaluated = evaluate_review_admission_gate_scope_1015(
        Some(&policy(non_empty_scope_id)),
        non_empty_scope_id,
        non_empty_scope_id,
        &resolver,
    );

    assert_eq!(
        evaluated,
        ReviewAdmissionGateScopeEvaluation::PreCompletion(
            ReviewAdmissionPreCompletion::ProfilePayloadNotEmpty
        )
    );
    assert_eq!(
        compose_review_admission_policy_46(evaluated, &[]),
        PolicyCompositionResult::PreCompletion
    );
}

#[test]
fn e1015_unsupported_profile_or_version_stays_pre_completion() {
    let unsupported_scope = scope_bytes(2, 1, b"", None);
    let unsupported_scope_id = record_id_from_bytes(&unsupported_scope);
    let unsupported_version_scope = scope_bytes(1, 2, b"", None);
    let unsupported_version_scope_id = record_id_from_bytes(&unsupported_version_scope);
    let resolver = FixtureRecordResolver {
        records: vec![
            (unsupported_scope_id, unsupported_scope),
            (unsupported_version_scope_id, unsupported_version_scope),
        ],
    };
    let evaluated = evaluate_review_admission_gate_scope_1015(
        Some(&policy(unsupported_scope_id)),
        unsupported_scope_id,
        unsupported_scope_id,
        &resolver,
    );

    assert_eq!(
        evaluated,
        ReviewAdmissionGateScopeEvaluation::PreCompletion(
            ReviewAdmissionPreCompletion::UnsupportedScopeProfileVersion {
                profile_id: 2,
                profile_version: 1,
            }
        )
    );
    assert_eq!(
        evaluate_review_admission_gate_scope_1015(
            Some(&policy(unsupported_version_scope_id)),
            unsupported_version_scope_id,
            unsupported_version_scope_id,
            &resolver,
        ),
        ReviewAdmissionGateScopeEvaluation::PreCompletion(
            ReviewAdmissionPreCompletion::UnsupportedScopeProfileVersion {
                profile_id: 1,
                profile_version: 2,
            }
        )
    );
}

#[test]
fn e1015_unavailable_or_invalid_authoritative_scope_identity_stays_pre_completion() {
    let (scope_id, _) = fixture_scope();
    let unavailable = FixtureRecordResolver { records: vec![] };
    let invalid_identity = FixtureRecordResolver {
        records: vec![(scope_id, scope_bytes(1, 1, b"", Some("wrong bytes for id")))],
    };

    assert_eq!(
        evaluate_review_admission_gate_scope_1015(
            Some(&policy(scope_id)),
            scope_id,
            scope_id,
            &unavailable,
        ),
        ReviewAdmissionGateScopeEvaluation::PreCompletion(
            ReviewAdmissionPreCompletion::AuthoritativeScopeUnavailable
        )
    );
    assert_eq!(
        evaluate_review_admission_gate_scope_1015(
            Some(&policy(scope_id)),
            scope_id,
            scope_id,
            &invalid_identity,
        ),
        ReviewAdmissionGateScopeEvaluation::PreCompletion(
            ReviewAdmissionPreCompletion::AuthoritativeScopeIdentityInvalid
        )
    );
}

#[test]
fn e1015_policy_context_unsupported_stays_outside_completed_section_46_results() {
    let (scope_id, scope) = fixture_scope();
    let resolver = FixtureRecordResolver {
        records: vec![(scope_id, scope)],
    };
    let unsupported_context = ReviewAdmissionGateScopePolicy::new(scope_id, vec![1]);
    let evaluated = evaluate_review_admission_gate_scope_1015(
        Some(&unsupported_context),
        scope_id,
        scope_id,
        &resolver,
    );

    assert_eq!(
        evaluated,
        ReviewAdmissionGateScopeEvaluation::PolicyContextUnsupported
    );
    assert_eq!(
        compose_review_admission_policy_46(evaluated, &[]),
        PolicyCompositionResult::PreCompletion
    );
}

#[test]
fn e1015_is_confined_to_review_admission_context_and_does_not_offer_generic_scope_applicability() {
    assert_eq!(
        policy_evaluator_1015_field_mapping().context,
        PolicyEvaluationContext::ReviewAdmission
    );
    assert_eq!(
        policy_evaluator_1015_field_mapping().field_name,
        "gate_scope_ref"
    );
}

#[test]
fn e1015_scope_label_is_non_semantic_for_exact_match() {
    let (unlabeled_id, unlabeled_scope) = fixture_scope();
    let labeled_id = record_id(SCOPE_PROFILE_1_EMPTY_LABEL_ID_HEX);
    let resolver = FixtureRecordResolver {
        records: vec![
            (unlabeled_id, unlabeled_scope),
            (labeled_id, hex_bytes(SCOPE_PROFILE_1_EMPTY_LABEL_HEX)),
        ],
    };

    for scope_id in [unlabeled_id, labeled_id] {
        assert_eq!(
            evaluate_review_admission_gate_scope_1015(
                Some(&policy(scope_id)),
                scope_id,
                scope_id,
                &resolver,
            ),
            ReviewAdmissionGateScopeEvaluation::Completed(Evaluator1015Result::Pass)
        );
    }
}

#[test]
fn e1015_uses_exact_record_id_bytes_without_normalization() {
    let (scope_id, scope) = fixture_scope();
    let bytes_that_differ = scope_bytes(1, 1, b"", Some("scope"));
    let differing_id = record_id_from_bytes(&bytes_that_differ);
    assert_ne!(scope_id.as_bytes(), differing_id.as_bytes());
    let resolver = FixtureRecordResolver {
        records: vec![(scope_id, scope), (differing_id, bytes_that_differ)],
    };

    assert_eq!(
        evaluate_review_admission_gate_scope_1015(
            Some(&policy(differing_id)),
            scope_id,
            scope_id,
            &resolver,
        ),
        ReviewAdmissionGateScopeEvaluation::Completed(Evaluator1015Result::Fail)
    );
}

#[test]
fn e1015_registry_is_unique_and_preserves_existing_evaluator_ids() {
    let registry = policy_evaluator_registry();
    assert_eq!(registry.iter().filter(|entry| entry.id == 1015).count(), 1);
    for id in 1001..=1014 {
        assert_eq!(registry.iter().filter(|entry| entry.id == id).count(), 1);
    }
    let e1015 = registry.iter().find(|entry| entry.id == 1015).unwrap();
    assert_eq!(e1015.name, "REVIEW_ADMISSION_EXACT_REVIEW_SCOPE_BINDING");
}

#[test]
fn e1015_pass_alone_never_manufactures_event_302_but_composed_satisfaction_uses_existing_positive_path(
) {
    let (scope_id, scope) = fixture_scope();
    let resolver = FixtureRecordResolver {
        records: vec![(scope_id, scope)],
    };
    let evaluated = evaluate_review_admission_gate_scope_1015(
        Some(&policy(scope_id)),
        scope_id,
        scope_id,
        &resolver,
    );
    assert_eq!(
        evaluated,
        ReviewAdmissionGateScopeEvaluation::Completed(Evaluator1015Result::Pass)
    );

    let composed = compose_review_admission_policy_46(evaluated, &[]);
    assert_eq!(composed, PolicyCompositionResult::Satisfied);
    assert_eq!(
        derive_review_admission_lifecycle_outcome(composed),
        ReviewAdmissionLifecycleOutcome::AcceptedEvent302
    );
}

#[test]
fn e1015_does_not_rewrite_or_reinterpret_historical_records() {
    let historical_scope_bytes = hex_bytes(SCOPE_PROFILE_1_EMPTY_HEX);
    let historical_scope_id = record_id(SCOPE_PROFILE_1_EMPTY_ID_HEX);
    let resolver = FixtureRecordResolver {
        records: vec![(historical_scope_id, historical_scope_bytes.clone())],
    };
    let _ = evaluate_review_admission_gate_scope_1015(
        Some(&policy(historical_scope_id)),
        historical_scope_id,
        historical_scope_id,
        &resolver,
    );
    assert_eq!(
        resolver.records[0].1, historical_scope_bytes,
        "evaluator input is read-only historical evidence"
    );
}
