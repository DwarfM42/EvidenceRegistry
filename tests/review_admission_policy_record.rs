use evidence_registry::{
    PolicyEvaluationContext, RecordId, ReviewAdmissionPolicyRecord, StrictRecordFrame,
};

fn id(first: u8) -> [u8; 32] {
    core::array::from_fn(|index| first + index as u8)
}

fn append_bstr_32(output: &mut Vec<u8>, bytes: [u8; 32]) {
    output.extend_from_slice(&[0x58, 0x20]);
    output.extend_from_slice(&bytes);
}

fn append_journal_reference(output: &mut Vec<u8>) {
    output.extend_from_slice(&[0x85, 0x58, 0x20]);
    output.extend_from_slice(&id(0x00));
    output.push(0x01);
    append_bstr_32(output, id(0x20));
    output.extend_from_slice(&[0x19, 0x01, 0x90]);
    append_bstr_32(output, id(0x40));
}

fn independently_construct_review_admission_policy() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x18, 40, 0x01, 0xa9, 0x00, 0x01, 0x01, 0x18, 40, 0x10]);
    append_bstr_32(&mut bytes, id(0xc0));
    bytes.extend_from_slice(&[0x11, 0x81, 0xa5, 0x00, 0x07, 0x01]);
    append_bstr_32(&mut bytes, id(0xc0));
    bytes.push(0x02);
    append_bstr_32(&mut bytes, id(0xe0));
    bytes.push(0x03);
    append_bstr_32(&mut bytes, id(0xa0));
    bytes.extend_from_slice(&[0x04, 0x01, 0x12, 0x81, 0x01, 0x13, 0x81, 0x01]);
    bytes.extend_from_slice(&[0x18, 0x18, 0xa1, 0x00, 0x82, 0x01, 0x02, 0x18, 0x1d]);
    append_journal_reference(&mut bytes);
    bytes.extend_from_slice(&[0x18, 0x1e, 0x81, 0x02]);
    bytes
}

#[test]
fn review_admission_policy_decodes_all_applicable_frozen_requirements() {
    let bytes = independently_construct_review_admission_policy();
    assert!(StrictRecordFrame::decode_authoritative(&bytes).is_ok());

    let policy = ReviewAdmissionPolicyRecord::decode_authoritative(&bytes).unwrap();

    assert_eq!(
        policy.record_id(),
        RecordId::try_from(policy.record_id().as_bytes().as_slice()).unwrap()
    );
    assert_eq!(policy.gate_scope_ref().as_bytes(), &id(0xc0));
    assert_eq!(
        policy.supported_context(),
        PolicyEvaluationContext::ReviewAdmission
    );
    assert_eq!(policy.review_requirements().len(), 1);
    assert_eq!(policy.required_method_statuses(), &[1]);
    assert_eq!(policy.allowed_finding_states(), &[1]);
    assert_eq!(policy.acceptable_anchor_relation_ids(), &[1, 2]);
}
