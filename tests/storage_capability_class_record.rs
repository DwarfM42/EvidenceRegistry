use evidence_registry::{
    RecordDecodeError, StorageCapabilityClassRecord, StorageCapabilityClassRecordInput,
};

#[test]
fn storage_capability_class_round_trips_the_exact_required_type_60_fields() {
    let input = StorageCapabilityClassRecordInput {
        filesystem_transport: "ntfs".to_owned(),
        sync_management: "win32".to_owned(),
        placeholder_capability: 1,
        exclusive_create_capability: 1,
        no_replace_publication_capability: 1,
        locking_capability: 1,
        atomic_rename_capability: 2,
        file_flush_capability: 1,
        directory_flush_capability: 2,
    };
    let record = StorageCapabilityClassRecord::new(input.clone()).unwrap();
    assert_eq!(
        StorageCapabilityClassRecord::decode_authoritative(&record.authoritative_cbor()).unwrap(),
        record
    );
    assert_eq!(record.input(), &input);
}

#[test]
fn storage_capability_class_rejects_an_unassigned_capability_value() {
    assert_eq!(
        StorageCapabilityClassRecord::new(StorageCapabilityClassRecordInput {
            filesystem_transport: "ntfs".to_owned(),
            sync_management: "win32".to_owned(),
            placeholder_capability: 4,
            exclusive_create_capability: 1,
            no_replace_publication_capability: 1,
            locking_capability: 1,
            atomic_rename_capability: 1,
            file_flush_capability: 1,
            directory_flush_capability: 1,
        }),
        Err(RecordDecodeError)
    );
}
