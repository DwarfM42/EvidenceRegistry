use evidence_registry::{
    EventRecordId, EventTypeId, JournalEntryHash, JournalEntryIndex, JournalReference, RecordId,
    RegistryId, ReviewAdmissionJournalEntry, ReviewAdmissionJournalEntryInput,
    ReviewAdmissionRecord, ReviewAdmissionRecordInput,
};

fn id(first: u8) -> [u8; 32] {
    core::array::from_fn(|index| first + index as u8)
}

fn reference(index: u64, event_type_id: u16, record_first: u8) -> JournalReference {
    JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(index).unwrap(),
        JournalEntryHash::try_from(id(0x80 + index as u8).as_slice()).unwrap(),
        EventTypeId::try_from(u64::from(event_type_id)).unwrap(),
        EventRecordId::try_from(id(record_first).as_slice()).unwrap(),
    )
}

#[test]
fn accepted_review_admission_entry_binds_the_exact_evaluated_authorities() {
    let admission = ReviewAdmissionRecord::new(ReviewAdmissionRecordInput {
        disposition_id: 1,
        review_request_ref: reference(1, 300, 0x20),
        review_result_ref: reference(2, 301, 0x40),
        policy_authority_ref: reference(3, 400, 0x60),
        reason_codes: vec!["SAT".to_owned()],
        operation_start_journal_ref: reference(0, 1, 0x10),
    })
    .unwrap();
    let entry = ReviewAdmissionJournalEntry::new(ReviewAdmissionJournalEntryInput {
        registry_id: RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        entry_index: JournalEntryIndex::try_from(4_u64).unwrap(),
        previous_entry_hash: JournalEntryHash::try_from(id(0x84).as_slice()).unwrap(),
        admission,
        storage_capability_class_id: RecordId::try_from(id(0xa0).as_slice()).unwrap(),
        environment_observation_id: RecordId::try_from(id(0xc0).as_slice()).unwrap(),
    })
    .unwrap();

    assert_eq!(entry.event_type_id().value(), 302);
    assert_eq!(
        entry.event_record_id().as_bytes(),
        entry.admission().record_id().as_bytes()
    );
    assert_eq!(
        entry.authority_dependencies(),
        [
            reference(1, 300, 0x20),
            reference(2, 301, 0x40),
            reference(3, 400, 0x60),
        ],
    );
}
