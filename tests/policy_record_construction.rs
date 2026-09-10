use evidence_registry::{
    EventRecordId, EventTypeId, JournalEntryHash, JournalEntryIndex, JournalReference,
    MinimalPolicyRecord, MinimalPolicyRecordInput, RecordId, RegistryId, StrictRecordFrame,
};
use sha2::{Digest, Sha256};

fn id(byte: u8) -> RecordId {
    RecordId::try_from([byte; 32].as_slice()).unwrap()
}

fn start() -> JournalReference {
    JournalReference::new(
        RegistryId::try_from([0x22; 32].as_slice()).unwrap(),
        JournalEntryIndex::try_from(7).unwrap(),
        JournalEntryHash::try_from([0x33; 32].as_slice()).unwrap(),
        EventTypeId::try_from(400).unwrap(),
        EventRecordId::try_from([0x44; 32].as_slice()).unwrap(),
    )
}

// Wire fixtures are independent of every production CBOR encoder.
fn bstr(byte: u8) -> Vec<u8> {
    let mut bytes = vec![0x58, 0x20];
    bytes.extend_from_slice(&[byte; 32]);
    bytes
}

fn requirement_wire(role: u8, count: u8) -> Vec<u8> {
    let mut bytes = vec![0xa5, 0, role, 1];
    bytes.extend(bstr(0x51));
    bytes.push(2);
    bytes.extend(bstr(0x52));
    bytes.push(3);
    bytes.extend(bstr(0x53));
    bytes.extend_from_slice(&[4, count]);
    bytes
}

fn review_wire(
    requirements: &[Vec<u8>],
    optional_fields: u8,
    optional: &[u8],
    contexts: &[u8],
) -> Vec<u8> {
    let mut body = vec![16];
    body.extend(bstr(0x11));
    body.extend_from_slice(&[17, 0x80 + requirements.len() as u8]);
    for requirement in requirements {
        body.extend_from_slice(requirement);
    }
    body.extend_from_slice(optional);
    body.extend(tail(contexts));
    frame(6 + optional_fields, &body)
}

#[test]
fn review_constructor_emits_exact_selected_policy_bytes() {
    use evidence_registry::{
        ReviewAdmissionPolicyRecord, ReviewAdmissionPolicyRecordInput,
        ReviewAdmissionReviewRequirement,
    };
    for with_options in [false, true] {
        let requirement =
            ReviewAdmissionReviewRequirement::new(1, id(0x51), id(0x52), id(0x53), 0).unwrap();
        let record = ReviewAdmissionPolicyRecord::new(ReviewAdmissionPolicyRecordInput {
            gate_scope_ref: id(0x11),
            review_requirements: vec![requirement.clone()],
            required_method_statuses: with_options.then(|| vec![1, 3]),
            allowed_finding_states: with_options.then(|| vec![2]),
            acceptable_anchor_relation_ids: with_options.then(|| vec![1, 6]),
            supported_context_ids: vec![2, 5],
            operation_start_journal_ref: start(),
        })
        .unwrap();
        let options: &[u8] = if with_options {
            &[18, 0x82, 1, 3, 19, 0x81, 2, 0x18, 24, 0xa1, 0, 0x82, 1, 6]
        } else {
            &[]
        };
        let expected = review_wire(
            &[requirement_wire(1, 0)],
            if with_options { 3 } else { 0 },
            options,
            &[2, 5],
        );
        assert_eq!(record.authoritative_cbor(), expected);
        assert_eq!(
            record.record_id().as_bytes().as_slice(),
            Sha256::digest(&expected).as_slice()
        );
        assert_eq!(record.gate_scope_ref(), id(0x11));
        assert_eq!(record.review_requirements(), [requirement]);
        assert_eq!(record.supported_context_ids(), [2, 5]);
        assert_eq!(record.operation_start_journal_ref(), &start());
        assert_eq!(
            record.required_method_statuses(),
            if with_options { &[1, 3][..] } else { &[] }
        );
        assert_eq!(
            record.allowed_finding_states(),
            if with_options { &[2][..] } else { &[] }
        );
        assert_eq!(
            record.acceptable_anchor_relation_ids(),
            if with_options { &[1, 6][..] } else { &[] }
        );
        assert_eq!(
            ReviewAdmissionPolicyRecord::decode_authoritative(&expected).unwrap(),
            record
        );
    }
}

#[test]
fn review_serializer_preserves_schema_valid_fields_outside_constructor_subset() {
    use evidence_registry::ReviewAdmissionPolicyRecord;
    // The existing decoder accepts key22 for declared Closeout context3, but its
    // Review evaluator ignores that field. Serialization must not discard it.
    let bytes = review_wire(&[requirement_wire(1, 0)], 1, &[22, 7], &[2, 3, 5]);
    let record = ReviewAdmissionPolicyRecord::decode_authoritative(&bytes).unwrap();
    assert_eq!(record.authoritative_cbor(), bytes);
    assert_eq!(
        record.record_id().as_bytes().as_slice(),
        Sha256::digest(record.authoritative_cbor()).as_slice()
    );
}

fn review_input() -> evidence_registry::ReviewAdmissionPolicyRecordInput {
    use evidence_registry::{ReviewAdmissionPolicyRecordInput, ReviewAdmissionReviewRequirement};
    ReviewAdmissionPolicyRecordInput {
        gate_scope_ref: id(0x11),
        review_requirements: vec![ReviewAdmissionReviewRequirement::new(
            1,
            id(0x51),
            id(0x52),
            id(0x53),
            0,
        )
        .unwrap()],
        required_method_statuses: None,
        allowed_finding_states: None,
        acceptable_anchor_relation_ids: None,
        supported_context_ids: vec![2, 5],
        operation_start_journal_ref: start(),
    }
}

#[test]
fn minimal_contexts_reject_unknown_unsorted_duplicates_and_missing_requirements() {
    for contexts in [
        vec![],
        vec![0],
        vec![2],
        vec![4],
        vec![5],
        vec![6],
        vec![1, 1],
        vec![3, 1],
    ] {
        let input = MinimalPolicyRecordInput {
            gate_scope_ref: id(0x11),
            supported_context_ids: contexts.iter().map(|v| u64::from(*v)).collect(),
            operation_start_journal_ref: start(),
        };
        assert!(MinimalPolicyRecord::new(input).is_err(), "{contexts:?}");
        let mut body = vec![16];
        body.extend(bstr(0x11));
        body.extend(tail(&contexts));
        assert!(
            MinimalPolicyRecord::decode_authoritative(&frame(5, &body)).is_err(),
            "{contexts:?}"
        );
    }
    for contexts in [vec![1], vec![3], vec![1, 3]] {
        assert!(MinimalPolicyRecord::new(MinimalPolicyRecordInput {
            gate_scope_ref: id(0x11),
            supported_context_ids: contexts,
            operation_start_journal_ref: start(),
        })
        .is_ok());
    }
    for value in [evidence_registry::ER_UINT_MAX + 1, u64::MAX] {
        assert!(MinimalPolicyRecord::new(MinimalPolicyRecordInput {
            gate_scope_ref: id(0x11),
            supported_context_ids: vec![value],
            operation_start_journal_ref: start(),
        })
        .is_err());
    }
}

#[test]
fn review_selectors_require_nonempty_canonical_order_and_uniqueness_excluding_count() {
    use evidence_registry::{ReviewAdmissionPolicyRecord, ReviewAdmissionReviewRequirement};
    let a = review_input().review_requirements[0].clone();
    let count_variant =
        ReviewAdmissionReviewRequirement::new(1, id(0x51), id(0x52), id(0x53), 1).unwrap();
    let b = ReviewAdmissionReviewRequirement::new(2, id(0x51), id(0x52), id(0x53), 0).unwrap();
    for requirements in [
        vec![],
        vec![a.clone(), a.clone()],
        vec![a.clone(), count_variant],
        vec![b.clone(), a.clone()],
    ] {
        let mut input = review_input();
        input.review_requirements = requirements;
        assert!(ReviewAdmissionPolicyRecord::new(input).is_err());
    }
    let mut input = review_input();
    input.review_requirements = vec![a, b];
    let record = ReviewAdmissionPolicyRecord::new(input).unwrap();
    assert_eq!(
        record.authoritative_cbor(),
        review_wire(
            &[requirement_wire(1, 0), requirement_wire(2, 0)],
            0,
            &[],
            &[2, 5]
        )
    );
    for requirements in [
        vec![],
        vec![requirement_wire(1, 0), requirement_wire(1, 0)],
        vec![requirement_wire(1, 0), requirement_wire(1, 1)],
        vec![requirement_wire(2, 0), requirement_wire(1, 0)],
        vec![requirement_wire(0, 0)],
        vec![requirement_wire(6, 0)],
    ] {
        assert!(
            ReviewAdmissionPolicyRecord::decode_authoritative(&review_wire(
                &requirements,
                0,
                &[],
                &[2, 5]
            ))
            .is_err()
        );
    }
}

#[test]
fn review_optional_sets_reject_empty_unknown_duplicate_and_unsorted_values() {
    use evidence_registry::{ReviewAdmissionPolicyRecord, ER_UINT_MAX};
    for key in [18, 19, 24] {
        let unknown = if key == 24 { 7 } else { 4 };
        for values in [
            vec![],
            vec![0],
            vec![unknown],
            vec![1, 1],
            vec![2, 1],
            vec![ER_UINT_MAX + 1],
            vec![u64::MAX],
        ] {
            let mut input = review_input();
            match key {
                18 => input.required_method_statuses = Some(values.clone()),
                19 => input.allowed_finding_states = Some(values.clone()),
                _ => input.acceptable_anchor_relation_ids = Some(values.clone()),
            }
            assert!(
                ReviewAdmissionPolicyRecord::new(input).is_err(),
                "{key} {values:?}"
            );
            if values.iter().all(|v| *v < 24) {
                let mut option = if key == 24 {
                    vec![0x18, 24, 0xa1, 0]
                } else {
                    vec![key]
                };
                option.push(0x80 + values.len() as u8);
                option.extend(values.iter().map(|v| *v as u8));
                assert!(
                    ReviewAdmissionPolicyRecord::decode_authoritative(&review_wire(
                        &[requirement_wire(1, 0)],
                        1,
                        &option,
                        &[2, 5]
                    ))
                    .is_err(),
                    "wire {key} {values:?}"
                );
            }
        }
    }
}

#[test]
fn review_contexts_and_dead_fields_use_existing_schema_applicability() {
    use evidence_registry::ReviewAdmissionPolicyRecord;
    for contexts in [
        vec![],
        vec![0],
        vec![1],
        vec![4],
        vec![6],
        vec![2, 2],
        vec![5, 2],
    ] {
        let mut input = review_input();
        input.supported_context_ids = contexts.iter().map(|v| u64::from(*v)).collect();
        assert!(
            ReviewAdmissionPolicyRecord::new(input).is_err(),
            "{contexts:?}"
        );
        assert!(
            ReviewAdmissionPolicyRecord::decode_authoritative(&review_wire(
                &[requirement_wire(1, 0)],
                0,
                &[],
                &contexts
            ))
            .is_err(),
            "wire {contexts:?}"
        );
    }
    // This local codec does not impose the selected Store's exact [2,5] profile.
    for contexts in [vec![2], vec![3], vec![5], vec![1, 2], vec![2, 5]] {
        let mut input = review_input();
        input.supported_context_ids = contexts;
        assert!(ReviewAdmissionPolicyRecord::new(input).is_ok());
    }
    for key in [18, 19, 24] {
        let mut input = review_input();
        input.supported_context_ids = vec![5];
        match key {
            18 => input.required_method_statuses = Some(vec![1]),
            19 => input.allowed_finding_states = Some(vec![1]),
            _ => input.acceptable_anchor_relation_ids = Some(vec![1]),
        }
        assert!(ReviewAdmissionPolicyRecord::new(input).is_err());
        let option = if key == 24 {
            vec![0x18, 24, 0xa1, 0, 0x81, 1]
        } else {
            vec![key, 0x81, 1]
        };
        assert!(
            ReviewAdmissionPolicyRecord::decode_authoritative(&review_wire(
                &[requirement_wire(1, 0)],
                1,
                &option,
                &[5]
            ))
            .is_err()
        );
    }
    for value in [evidence_registry::ER_UINT_MAX + 1, u64::MAX] {
        let mut input = review_input();
        input.supported_context_ids = vec![value];
        assert!(ReviewAdmissionPolicyRecord::new(input).is_err());
    }
}

#[test]
fn independent_review_wire_rejects_dead_unknown_reordered_and_noncanonical_fields() {
    use evidence_registry::ReviewAdmissionPolicyRecord;
    for (fields, option) in [
        (1, vec![22, 7]),                      // dead Closeout field in [2,5]
        (1, vec![0x18, 31, 0]),                // unknown Policy key
        (2, vec![19, 0x81, 1, 18, 0x81, 1]),   // descending map keys
        (2, vec![18, 0x81, 1, 18, 0x81, 1]),   // duplicate map keys
        (1, vec![18, 0x81, 0x18, 1]),          // nonminimal integer
        (1, vec![18, 0x81, 0x20]),             // negative instead of UInt
        (1, vec![0x18, 24, 0xa1, 1, 0x81, 1]), // wrong anchor map key
    ] {
        assert!(
            ReviewAdmissionPolicyRecord::decode_authoritative(&review_wire(
                &[requirement_wire(1, 0)],
                fields,
                &option,
                &[2, 5]
            ))
            .is_err(),
            "{option:?}"
        );
    }
    let valid = review_wire(&[requirement_wire(1, 0)], 0, &[], &[2, 5]);
    for length in 0..valid.len() {
        assert!(
            ReviewAdmissionPolicyRecord::decode_authoritative(&valid[..length]).is_err(),
            "truncation {length}"
        );
    }
    let mut trailing = valid;
    trailing.push(0);
    assert!(ReviewAdmissionPolicyRecord::decode_authoritative(&trailing).is_err());
}

fn frame(fields: u8, body: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0x84, 0x78, 0x1a];
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x18, 40, 1, 0xa0 + fields, 0, 1, 1, 0x18, 40]);
    bytes.extend_from_slice(body);
    bytes
}

fn tail(contexts: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0x18, 29, 0x85];
    bytes.extend(bstr(0x22));
    bytes.push(7);
    bytes.extend(bstr(0x33));
    bytes.extend_from_slice(&[0x19, 0x01, 0x90]);
    bytes.extend(bstr(0x44));
    bytes.extend_from_slice(&[0x18, 30, 0x80 + contexts.len() as u8]);
    bytes.extend_from_slice(contexts);
    bytes
}

#[test]
fn minimal_constructor_emits_exact_independent_bytes() {
    let record = MinimalPolicyRecord::new(MinimalPolicyRecordInput {
        gate_scope_ref: id(0x11),
        supported_context_ids: vec![1],
        operation_start_journal_ref: start(),
    })
    .unwrap();
    let mut body = vec![16];
    body.extend(bstr(0x11));
    body.extend(tail(&[1]));
    let expected = frame(5, &body);
    assert_eq!(record.authoritative_cbor(), expected);
    assert_eq!(
        record.record_id(),
        StrictRecordFrame::decode_authoritative(&expected)
            .unwrap()
            .record_id()
    );
    assert_eq!(
        record.record_id().as_bytes().as_slice(),
        Sha256::digest(&expected).as_slice()
    );
    assert_eq!(record.gate_scope_ref(), id(0x11));
    assert_eq!(record.supported_context_ids(), [1]);
    assert_eq!(record.operation_start_journal_ref(), &start());
    assert_eq!(
        MinimalPolicyRecord::decode_authoritative(&expected).unwrap(),
        record
    );
}

#[test]
fn review_requirement_constructor_preserves_exact_selector_and_uint_count() {
    use evidence_registry::{ReviewAdmissionReviewRequirement, ER_UINT_MAX};
    for role in 1..=5 {
        for count in [0, 1, ER_UINT_MAX] {
            let requirement =
                ReviewAdmissionReviewRequirement::new(role, id(0x51), id(0x52), id(0x53), count)
                    .unwrap();
            assert_eq!(requirement.review_role_id(), role);
            assert_eq!(requirement.review_scope_ref(), id(0x51));
            assert_eq!(requirement.review_method_ref(), id(0x52));
            assert_eq!(requirement.required_checks_ref(), id(0x53));
            assert_eq!(requirement.required_count(), count);
        }
    }
    for (role, count) in [
        (0, 1),
        (6, 1),
        (u64::MAX, 1),
        (1, ER_UINT_MAX + 1),
        (1, u64::MAX),
    ] {
        assert!(
            ReviewAdmissionReviewRequirement::new(role, id(0x51), id(0x52), id(0x53), count,)
                .is_err()
        );
    }
}
