use evidence_registry::{MinimalPolicyRecord, RecordId, RecordTypeId, StrictRecordFrame};

const MINIMAL_POLICY_RECORD_HEX: &str = concat!(
    "84781a45766964656e636552656769737472792e5265636f72642e7631182801a50001011828105820",
    "a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf181d855820",
    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f015820",
    "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f18645820",
    "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f181e8101"
);
const MINIMAL_POLICY_RECORD_ID_HEX: &str =
    "826fd805e6d7a7d3633f87177afbcc5fc9bf1163757b67c9399ebe71ec159c21";

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}

fn hex_id(hex: &str) -> [u8; 32] {
    hex_bytes(hex).try_into().unwrap()
}

#[test]
fn minimal_policy_record_decodes_exact_identity_and_declares_freeze_commit_only() {
    let canonical = hex_bytes(MINIMAL_POLICY_RECORD_HEX);
    let decoded = MinimalPolicyRecord::decode_authoritative(&canonical).unwrap();

    assert_eq!(
        decoded.record_id(),
        RecordId::try_from(hex_id(MINIMAL_POLICY_RECORD_ID_HEX).as_slice()).unwrap()
    );
    assert_eq!(
        decoded.record_type_id(),
        RecordTypeId::try_from(40_u64).unwrap()
    );
    assert_eq!(
        decoded.gate_scope_ref().as_bytes(),
        &hex_id("a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf")
    );
    assert_eq!(
        decoded.operation_start_journal_ref().entry_index().value(),
        1
    );
    assert_eq!(decoded.supported_context_ids(), &[1]);
    assert!(decoded.declares_freeze_commit_support());
}

#[test]
fn minimal_policy_record_rejects_optional_requirements_and_does_not_infer_context_support() {
    let canonical = hex_bytes(MINIMAL_POLICY_RECORD_HEX);

    let mut optional_requirement = canonical.clone();
    let map_offset = optional_requirement
        .windows(4)
        .position(|candidate| candidate == [0x01, 0xa5, 0x00, 0x01])
        .unwrap()
        + 1;
    optional_requirement[map_offset] = 0xa6;
    let operation_ref_key_offset = optional_requirement
        .windows(2)
        .position(|candidate| candidate == [0x18, 0x1d])
        .unwrap();
    optional_requirement.splice(
        operation_ref_key_offset..operation_ref_key_offset,
        [0x11, 0x80],
    );

    let mut non_freeze_context = canonical.clone();
    *non_freeze_context.last_mut().unwrap() = 3;
    let mut review_context_without_requirements = canonical.clone();
    *review_context_without_requirements.last_mut().unwrap() = 2;
    let mut closeout_postcondition_without_requirements = canonical.clone();
    *closeout_postcondition_without_requirements
        .last_mut()
        .unwrap() = 4;

    assert!(StrictRecordFrame::decode_authoritative(&optional_requirement).is_ok());
    assert!(MinimalPolicyRecord::decode_authoritative(&optional_requirement).is_err());
    assert!(StrictRecordFrame::decode_authoritative(&review_context_without_requirements).is_ok());
    assert!(
        MinimalPolicyRecord::decode_authoritative(&review_context_without_requirements).is_err()
    );
    assert!(
        StrictRecordFrame::decode_authoritative(&closeout_postcondition_without_requirements)
            .is_ok()
    );
    assert!(MinimalPolicyRecord::decode_authoritative(
        &closeout_postcondition_without_requirements
    )
    .is_err());

    let decoded = MinimalPolicyRecord::decode_authoritative(&non_freeze_context).unwrap();
    assert_eq!(decoded.supported_context_ids(), &[3]);
    assert!(!decoded.declares_freeze_commit_support());
}
