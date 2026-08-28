use evidence_registry::{
    derive_freeze_root, validate_freeze_committed_binding, EventRecordId, EventTypeId,
    FreezeAttemptId, FreezeAttemptStartRecord, FreezeAttemptStartRecordInput,
    FreezeCommittedBindingInput, FreezeCommittedBindingOutcome, FreezeReceiptRecord,
    JournalEntryHash, JournalEntryIndex, JournalReference, RecordId, RegistryId, RetainedJournal,
    StrictRecordFrame,
};

fn id(first: u8) -> [u8; 32] {
    core::array::from_fn(|index| first.wrapping_add(index as u8))
}

fn append_bstr_32(output: &mut Vec<u8>, value: &[u8; 32]) {
    output.extend_from_slice(&[0x58, 0x20]);
    output.extend_from_slice(value);
}

fn append_reference(output: &mut Vec<u8>, reference: &JournalReference) {
    output.push(0x85);
    append_bstr_32(output, reference.registry_id().as_bytes());
    output.push(reference.entry_index().value() as u8);
    append_bstr_32(output, reference.entry_hash().as_bytes());
    output.extend_from_slice(&[0x18, reference.event_type_id().value() as u8]);
    append_bstr_32(output, reference.event_record_id().as_bytes());
}

fn start_record(registry_id: RegistryId) -> FreezeAttemptStartRecord {
    let freeze_attempt_id = FreezeAttemptId::try_from(id(0xa0).as_slice()).unwrap();
    FreezeAttemptStartRecord::new(FreezeAttemptStartRecordInput {
        freeze_attempt_id,
        intended_root_id: derive_freeze_root(registry_id, freeze_attempt_id).intended_root_id(),
        subject_id: id(0xe0),
        policy_record_id: RecordId::try_from(id(0x40).as_slice()).unwrap(),
    })
}

fn freeze_start_bytes(
    previous_entry_hash: JournalEntryHash,
    start_record: &FreezeAttemptStartRecord,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(299);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xae);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01]);
    append_bstr_32(&mut bytes, &id(0x00));
    bytes.extend_from_slice(&[0x02, 0x01, 0x03]);
    append_bstr_32(&mut bytes, previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x18, 0x64, 0x05]);
    append_bstr_32(&mut bytes, start_record.record_id().as_bytes());
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x80, 0x08, 0x02, 0x09]);
    append_bstr_32(&mut bytes, &id(0xa0));
    bytes.push(0x0a);
    append_bstr_32(&mut bytes, &id(0x40));
    bytes.push(0x0b);
    append_bstr_32(&mut bytes, &id(0x60));
    bytes.push(0x10);
    append_bstr_32(
        &mut bytes,
        start_record.input().freeze_attempt_id.as_bytes(),
    );
    bytes.push(0x11);
    append_bstr_32(&mut bytes, start_record.input().intended_root_id.as_bytes());
    bytes
}

fn receipt_bytes(start_reference: &JournalReference) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(620);
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x04, 0x01, 0xb1, 0x00, 0x01, 0x01, 0x04]);
    bytes.push(0x10);
    append_bstr_32(&mut bytes, &id(0xa0));
    bytes.push(0x11);
    append_bstr_32(&mut bytes, &id(0xb0));
    bytes.push(0x12);
    append_reference(&mut bytes, start_reference);
    bytes.push(0x13);
    append_bstr_32(&mut bytes, &id(0xe0));
    bytes.push(0x14);
    append_bstr_32(&mut bytes, &id(0xc0));
    bytes.extend_from_slice(&[0x15, 0x01]);
    bytes.push(0x16);
    append_bstr_32(&mut bytes, &id(0xd0));
    bytes.extend_from_slice(&[0x17, 0x01]);
    bytes.extend_from_slice(&[0x18, 0x18]);
    append_bstr_32(&mut bytes, &id(0xf0));
    bytes.extend_from_slice(&[0x18, 0x19]);
    append_bstr_32(&mut bytes, &id(0x40));
    bytes.extend_from_slice(&[
        0x18, 0x1a, 0x01, 0x18, 0x1b, 0x01, 0x18, 0x1c, 0x01, 0x18, 0x1d, 0xf5, 0x18, 0x1f, 0x64,
    ]);
    bytes.extend_from_slice(b"test");
    bytes
}

fn freeze_committed_bytes(
    previous_entry_hash: JournalEntryHash,
    start_reference: &JournalReference,
    receipt_record_id: RecordId,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(439);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xae);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01]);
    append_bstr_32(&mut bytes, &id(0x00));
    bytes.extend_from_slice(&[0x02, 0x02, 0x03]);
    append_bstr_32(&mut bytes, previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x18, 0x65, 0x05]);
    append_bstr_32(&mut bytes, receipt_record_id.as_bytes());
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x81]);
    append_reference(&mut bytes, start_reference);
    bytes.extend_from_slice(&[0x08, 0x02, 0x09]);
    append_bstr_32(&mut bytes, &id(0xa0));
    bytes.push(0x0a);
    append_bstr_32(&mut bytes, &id(0x40));
    bytes.push(0x0b);
    append_bstr_32(&mut bytes, &id(0x60));
    bytes.push(0x10);
    append_bstr_32(&mut bytes, &id(0xa0));
    bytes.push(0x12);
    append_reference(&mut bytes, start_reference);
    bytes
}

#[test]
fn freeze_committed_binding_returns_explicit_authority_evidence_unavailable_after_exact_structural_binding(
) {
    let registry_id = RegistryId::try_from(id(0x00).as_slice()).unwrap();
    let start_record = start_record(registry_id);
    let genesis = evidence_registry::GenesisJournalEntry::new(
        registry_id,
        EventRecordId::try_from(id(0x20).as_slice()).unwrap(),
        RecordId::try_from(id(0x40).as_slice()).unwrap(),
        RecordId::try_from(id(0x60).as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash(), &start_record))
        .unwrap();
    let start_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(1_u64).unwrap(),
        journal.reconstruct_state().unwrap().journal_head_hash(),
        EventTypeId::try_from(100_u64).unwrap(),
        EventRecordId::try_from(start_record.record_id().as_bytes().as_slice()).unwrap(),
    );
    let receipt_bytes = receipt_bytes(&start_reference);
    assert!(StrictRecordFrame::decode_authoritative(&receipt_bytes).is_ok());
    let receipt = FreezeReceiptRecord::decode_authoritative(&receipt_bytes).unwrap();
    journal
        .append_strict_entry(&freeze_committed_bytes(
            start_reference.entry_hash(),
            &start_reference,
            receipt.record_id(),
        ))
        .unwrap();
    let committed_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(2_u64).unwrap(),
        journal.reconstruct_state().unwrap().journal_head_hash(),
        EventTypeId::try_from(101_u64).unwrap(),
        EventRecordId::try_from(receipt.record_id().as_bytes().as_slice()).unwrap(),
    );

    assert!(matches!(
        validate_freeze_committed_binding(FreezeCommittedBindingInput {
            retained_journal: &journal,
            committed_event_reference: committed_reference,
            freeze_attempt_start_record: &start_record,
            freeze_receipt_record: &receipt,
        }),
        Ok(FreezeCommittedBindingOutcome::AuthorityEvidenceUnavailable)
    ));
}

#[test]
fn freeze_committed_binding_rejects_a_terminal_event_bound_to_a_different_receipt_identity() {
    let registry_id = RegistryId::try_from(id(0x00).as_slice()).unwrap();
    let start_record = start_record(registry_id);
    let genesis = evidence_registry::GenesisJournalEntry::new(
        registry_id,
        EventRecordId::try_from(id(0x20).as_slice()).unwrap(),
        RecordId::try_from(id(0x40).as_slice()).unwrap(),
        RecordId::try_from(id(0x60).as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash(), &start_record))
        .unwrap();
    let start_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(1_u64).unwrap(),
        journal.reconstruct_state().unwrap().journal_head_hash(),
        EventTypeId::try_from(100_u64).unwrap(),
        EventRecordId::try_from(start_record.record_id().as_bytes().as_slice()).unwrap(),
    );
    let receipt =
        FreezeReceiptRecord::decode_authoritative(&receipt_bytes(&start_reference)).unwrap();
    let different_receipt_id = RecordId::try_from(id(0x81).as_slice()).unwrap();
    journal
        .append_strict_entry(&freeze_committed_bytes(
            start_reference.entry_hash(),
            &start_reference,
            different_receipt_id,
        ))
        .unwrap();
    let committed_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(2_u64).unwrap(),
        journal.reconstruct_state().unwrap().journal_head_hash(),
        EventTypeId::try_from(101_u64).unwrap(),
        EventRecordId::try_from(different_receipt_id.as_bytes().as_slice()).unwrap(),
    );

    assert_eq!(
        validate_freeze_committed_binding(FreezeCommittedBindingInput {
            retained_journal: &journal,
            committed_event_reference: committed_reference,
            freeze_attempt_start_record: &start_record,
            freeze_receipt_record: &receipt,
        }),
        Err(evidence_registry::FreezeCommittedBindingError::ReceiptRecordMismatch)
    );
}
