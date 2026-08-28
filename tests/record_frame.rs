use evidence_registry::{
    GenesisRecord, GenesisRecordInput, RecordId, RecordTypeId, RegistryId, StrictRecordFrame,
};

fn id(first: u8) -> [u8; 32] {
    core::array::from_fn(|index| first + index as u8)
}

fn genesis_record() -> GenesisRecord {
    GenesisRecord::new(GenesisRecordInput {
        registry_id: RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        journal_format_version: 1,
        record_identity_profile_id: 1,
        storage_capability_class_id: RecordId::try_from(id(0x40).as_slice()).unwrap(),
        environment_observation_id: RecordId::try_from(id(0x60).as_slice()).unwrap(),
        created_by_tool_version: "record-frame-test".to_owned(),
    })
    .unwrap()
}

#[test]
fn strict_record_frame_preserves_canonical_type_and_self_hash_identity() {
    let record = genesis_record();
    let bytes = record.authoritative_cbor();

    let frame = StrictRecordFrame::decode_authoritative(&bytes).unwrap();

    assert_eq!(frame.record_type_id(), RecordTypeId::try_from(1).unwrap());
    assert_eq!(frame.schema_version(), 1);
    assert_eq!(frame.record_id(), record.record_id());
    assert_eq!(frame.authoritative_cbor(), bytes.as_slice());
}
