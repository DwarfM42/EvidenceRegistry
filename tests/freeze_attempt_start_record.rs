use evidence_registry::{
    FreezeAttemptId, FreezeAttemptStartRecord, FreezeAttemptStartRecordInput, IntendedRootId,
    RecordId, RecordTypeId, StrictRecordFrame,
};

fn id(first: u8) -> [u8; 32] {
    core::array::from_fn(|index| first + index as u8)
}

fn freeze_attempt_start_canonical_vector() -> Vec<u8> {
    let mut bytes = Vec::from([
        0x84, 0x78, 0x1a, 0x45, 0x76, 0x69, 0x64, 0x65, 0x6e, 0x63, 0x65, 0x52, 0x65, 0x67, 0x69,
        0x73, 0x74, 0x72, 0x79, 0x2e, 0x52, 0x65, 0x63, 0x6f, 0x72, 0x64, 0x2e, 0x76, 0x31, 0x03,
        0x01, 0xa6, 0x00, 0x01, 0x01, 0x03, 0x10, 0x58, 0x20,
    ]);
    bytes.extend(id(0xa0));
    bytes.extend_from_slice(&[0x11, 0x58, 0x20]);
    bytes.extend(id(0xc0));
    bytes.extend_from_slice(&[0x12, 0x58, 0x20]);
    bytes.extend(id(0xe0));
    bytes.extend_from_slice(&[0x13, 0x58, 0x20]);
    bytes.extend(id(0x40));
    bytes
}

fn freeze_attempt_start_input() -> FreezeAttemptStartRecordInput {
    FreezeAttemptStartRecordInput {
        freeze_attempt_id: FreezeAttemptId::try_from(id(0xa0).as_slice()).unwrap(),
        intended_root_id: IntendedRootId::try_from(id(0xc0).as_slice()).unwrap(),
        subject_id: id(0xe0),
        policy_record_id: RecordId::try_from(id(0x40).as_slice()).unwrap(),
    }
}

#[test]
fn freeze_attempt_start_record_matches_the_independent_canonical_vector_and_hash() {
    let input = freeze_attempt_start_input();
    let expected_bytes = freeze_attempt_start_canonical_vector();
    let expected_record_id = RecordId::try_from(
        [
            0xb3, 0x55, 0x2c, 0xc0, 0x8f, 0x7f, 0x6d, 0x3d, 0xc3, 0x01, 0x1e, 0x22, 0x97, 0x16,
            0x68, 0xec, 0x7f, 0xec, 0x08, 0x6a, 0x55, 0xdf, 0xc7, 0x4d, 0x9a, 0x47, 0xf1, 0xd8,
            0xd3, 0xa3, 0xac, 0x3a,
        ]
        .as_slice(),
    )
    .unwrap();

    let record = FreezeAttemptStartRecord::new(input.clone());
    assert_eq!(record.authoritative_cbor(), expected_bytes);
    assert_eq!(record.record_id(), expected_record_id);

    let decoded = FreezeAttemptStartRecord::decode_authoritative(&expected_bytes).unwrap();
    assert_eq!(decoded.input(), &input);
    assert_eq!(decoded.record_type_id(), RecordTypeId::try_from(3).unwrap());
    assert_eq!(decoded.record_id(), expected_record_id);
    assert_eq!(decoded.authoritative_cbor(), expected_bytes);
}

#[test]
fn freeze_attempt_start_record_strict_decoder_rejects_noncanonical_or_invalid_vectors() {
    let canonical = freeze_attempt_start_canonical_vector();

    let mut missing_member = canonical.clone();
    missing_member[31] = 0xa5;
    missing_member.truncate(141);

    let mut unknown_member = canonical.clone();
    unknown_member[31] = 0xa7;
    unknown_member.extend_from_slice(&[0x14, 0x58, 0x20]);
    unknown_member.extend(id(0x00));

    let mut duplicate_member = canonical.clone();
    duplicate_member[36] = 0x11;

    let mut out_of_order_member = canonical.clone();
    let first_local_member = canonical[36..71].to_vec();
    let second_local_member = canonical[71..106].to_vec();
    out_of_order_member[36..71].copy_from_slice(&second_local_member);
    out_of_order_member[71..106].copy_from_slice(&first_local_member);

    let mut wrong_outer_type = canonical.clone();
    wrong_outer_type[29] = 0x04;

    let mut wrong_outer_schema_version = canonical.clone();
    wrong_outer_schema_version[30] = 0x02;

    let mut wrong_body_schema_version = canonical.clone();
    wrong_body_schema_version[33] = 0x02;

    let mut wrong_body_record_type = canonical.clone();
    wrong_body_record_type[35] = 0x04;

    let mut nonminimal_outer_record_type = canonical.clone();
    nonminimal_outer_record_type.splice(29..30, [0x18, 0x03]);

    let mut invalid_bstr_length = canonical.clone();
    invalid_bstr_length[38] = 0x1f;

    let mut invalid_bstr_type = canonical.clone();
    invalid_bstr_type[37] = 0x40;

    let mut truncated = canonical.clone();
    truncated.pop();

    let mut appended_byte = canonical;
    appended_byte.push(0x00);

    for invalid in [
        missing_member,
        unknown_member,
        duplicate_member,
        wrong_outer_type,
        wrong_outer_schema_version,
        wrong_body_schema_version,
        wrong_body_record_type,
        nonminimal_outer_record_type,
        invalid_bstr_length,
        invalid_bstr_type,
        truncated,
        appended_byte,
    ] {
        assert!(FreezeAttemptStartRecord::decode_authoritative(&invalid).is_err());
    }

    assert!(StrictRecordFrame::decode_authoritative(&out_of_order_member).is_err());
    assert!(FreezeAttemptStartRecord::decode_authoritative(&out_of_order_member).is_err());
}
