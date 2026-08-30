use evidence_registry::{
    validate_review_admission_policy_recorded_binding, EventRecordId, EventTypeId,
    GenesisJournalEntry, JournalEntryHash, JournalEntryIndex, JournalReference,
    PolicyEvaluationContext, RecordId, RegistryId, RetainedJournal, ReviewAdmissionPolicyRecord,
    StrictRecordFrame,
};
use sha2::{Digest, Sha256};

fn id(first: u8) -> [u8; 32] {
    core::array::from_fn(|index| first + index as u8)
}

fn append_bstr_32(output: &mut Vec<u8>, bytes: [u8; 32]) {
    output.extend_from_slice(&[0x58, 0x20]);
    output.extend_from_slice(&bytes);
}

fn append_journal_reference(output: &mut Vec<u8>, reference: &JournalReference) {
    output.extend_from_slice(&[0x85, 0x58, 0x20]);
    output.extend_from_slice(reference.registry_id().as_bytes());
    output.push(reference.entry_index().value() as u8);
    append_bstr_32(output, *reference.entry_hash().as_bytes());
    match reference.event_type_id().value() {
        value @ 0..=23 => output.push(value as u8),
        value @ 24..=0xff => output.extend_from_slice(&[0x18, value as u8]),
        value => output.extend_from_slice(&[0x19, (value >> 8) as u8, value as u8]),
    }
    append_bstr_32(output, *reference.event_record_id().as_bytes());
}

fn independently_construct_review_admission_policy(operation_start: &JournalReference) -> Vec<u8> {
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
    append_journal_reference(&mut bytes, operation_start);
    bytes.extend_from_slice(&[0x18, 0x1e, 0x81, 0x02]);
    bytes
}

fn genesis() -> GenesisJournalEntry {
    GenesisJournalEntry::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        EventRecordId::try_from(id(0x20).as_slice()).unwrap(),
        RecordId::try_from(id(0x40).as_slice()).unwrap(),
        RecordId::try_from(id(0x60).as_slice()).unwrap(),
    )
}

fn genesis_reference(genesis: &GenesisJournalEntry) -> JournalReference {
    JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(0_u64).unwrap(),
        genesis.entry_hash(),
        EventTypeId::try_from(1_u64).unwrap(),
        EventRecordId::try_from(id(0x20).as_slice()).unwrap(),
    )
}

fn policy_recorded_entry(
    previous_entry_hash: JournalEntryHash,
    policy_record_id: RecordId,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.extend_from_slice(&[0xac, 0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, 0x01, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x19, 0x01, 0x90, 0x05]);
    append_bstr_32(&mut bytes, *policy_record_id.as_bytes());
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x80, 0x08, 0x07, 0x09]);
    append_bstr_32(&mut bytes, *policy_record_id.as_bytes());
    bytes.extend_from_slice(&[0x0a]);
    append_bstr_32(&mut bytes, id(0x40));
    bytes.extend_from_slice(&[0x0b]);
    append_bstr_32(&mut bytes, id(0x60));
    bytes
}

#[test]
fn review_admission_policy_decodes_all_applicable_frozen_requirements() {
    let start_genesis = genesis();
    let bytes = independently_construct_review_admission_policy(&genesis_reference(&start_genesis));
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

#[test]
fn review_admission_policy_requires_exact_retained_policy_recording_and_prior_operation_start() {
    let genesis = genesis();
    let policy_bytes =
        independently_construct_review_admission_policy(&genesis_reference(&genesis));
    let policy_record_id = RecordId::try_from(Sha256::digest(&policy_bytes).as_slice()).unwrap();
    let policy_entry = policy_recorded_entry(genesis.entry_hash(), policy_record_id);
    let policy_entry_hash: [u8; 32] = Sha256::digest(&policy_entry).into();
    let policy_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(policy_entry_hash.as_slice()).unwrap(),
        EventTypeId::try_from(400_u64).unwrap(),
        EventRecordId::try_from(policy_record_id.as_bytes().as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&policy_entry).unwrap();

    let binding = validate_review_admission_policy_recorded_binding(
        &journal,
        &policy_reference,
        &policy_bytes,
    )
    .unwrap();

    assert_eq!(binding.policy_record_id(), policy_record_id);
    assert_eq!(binding.operation_start_reference().entry_index().value(), 0);
}
