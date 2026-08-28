use evidence_registry::{
    FreezeAttemptId, FreezeAttemptStartRecord, FreezeAttemptStartRecordInput, IntendedRootId,
    RecordId, RecordTypeId,
};

fn id(first: u8) -> [u8; 32] {
    core::array::from_fn(|index| first + index as u8)
}

#[test]
fn freeze_attempt_start_record_round_trips_its_exact_typed_fields() {
    let input = FreezeAttemptStartRecordInput {
        freeze_attempt_id: FreezeAttemptId::try_from(id(0xa0).as_slice()).unwrap(),
        intended_root_id: IntendedRootId::try_from(id(0xc0).as_slice()).unwrap(),
        subject_id: id(0xe0),
        policy_record_id: RecordId::try_from(id(0x40).as_slice()).unwrap(),
    };
    let record = FreezeAttemptStartRecord::new(input.clone());
    let bytes = record.authoritative_cbor();

    let decoded = FreezeAttemptStartRecord::decode_authoritative(&bytes).unwrap();

    assert_eq!(decoded.input(), &input);
    assert_eq!(decoded.record_type_id(), RecordTypeId::try_from(3).unwrap());
    assert_eq!(decoded.record_id(), record.record_id());
    assert_eq!(decoded.authoritative_cbor(), bytes);
}
