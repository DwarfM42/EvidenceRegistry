use evidence_registry::{
    EventRecordId, EventTypeId, JournalEntryHash, JournalEntryIndex, JournalReference, RecordId,
    RegistryId, ReviewAdmissionRecord, ReviewAdmissionRecordInput,
};

fn id(first: u8) -> [u8; 32] {
    core::array::from_fn(|index| first + index as u8)
}

fn append_bstr_32(output: &mut Vec<u8>, bytes: [u8; 32]) {
    output.extend_from_slice(&[0x58, 0x20]);
    output.extend_from_slice(&bytes);
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

fn independently_construct_accepted_review_admission() -> Vec<u8> {
    let request_ref = reference(1, 300, 0x20);
    let result_ref = reference(2, 301, 0x40);
    let policy_ref = reference(3, 400, 0x60);
    let operation_start = reference(0, 1, 0x10);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x18, 32, 0x01, 0xa8, 0x00, 0x01, 0x01, 0x18, 32, 0x10, 0x01]);
    bytes.push(0x11);
    append_journal_reference(&mut bytes, &request_ref);
    bytes.push(0x12);
    append_journal_reference(&mut bytes, &result_ref);
    bytes.push(0x13);
    append_journal_reference(&mut bytes, &policy_ref);
    bytes.extend_from_slice(&[0x16, 0x81, 0x63]);
    bytes.extend_from_slice(b"SAT");
    bytes.push(0x17);
    append_journal_reference(&mut bytes, &operation_start);
    bytes
}

#[test]
fn accepted_review_admission_requires_exact_resolved_request_result_and_policy_references() {
    let bytes = independently_construct_accepted_review_admission();

    let admission = ReviewAdmissionRecord::decode_authoritative(&bytes).unwrap();

    assert_eq!(admission.disposition_id(), 1);
    assert_eq!(
        admission.review_request_ref(),
        Some(&reference(1, 300, 0x20))
    );
    assert_eq!(
        admission.review_result_ref(),
        Some(&reference(2, 301, 0x40))
    );
    assert_eq!(
        admission.policy_authority_ref(),
        Some(&reference(3, 400, 0x60))
    );
    assert_eq!(admission.reason_codes(), ["SAT"]);
    assert_eq!(
        admission.operation_start_journal_ref(),
        &reference(0, 1, 0x10),
    );
    assert_eq!(
        admission.record_id(),
        RecordId::try_from(admission.record_id().as_bytes().as_slice()).unwrap(),
    );
}

#[test]
fn rejected_review_admission_preserves_successfully_resolved_authority_references() {
    let mut bytes = independently_construct_accepted_review_admission();
    let disposition_offset = bytes
        .windows(3)
        .position(|window| window == [0x10, 0x01, 0x11])
        .unwrap()
        + 1;
    bytes[disposition_offset] = 2;

    let admission = ReviewAdmissionRecord::decode_authoritative(&bytes).unwrap();

    assert_eq!(admission.disposition_id(), 2);
    assert_eq!(
        admission.review_request_ref(),
        Some(&reference(1, 300, 0x20))
    );
    assert_eq!(
        admission.review_result_ref(),
        Some(&reference(2, 301, 0x40))
    );
    assert_eq!(
        admission.policy_authority_ref(),
        Some(&reference(3, 400, 0x60))
    );
}

#[test]
fn accepted_review_admission_serializes_canonical_record_bytes_from_exact_typed_references() {
    let input = ReviewAdmissionRecordInput {
        disposition_id: 1,
        review_request_ref: reference(1, 300, 0x20),
        review_result_ref: reference(2, 301, 0x40),
        policy_authority_ref: reference(3, 400, 0x60),
        reason_codes: vec!["SAT".to_owned()],
        operation_start_journal_ref: reference(0, 1, 0x10),
    };

    let admission = ReviewAdmissionRecord::new(input).unwrap();

    assert_eq!(
        admission.authoritative_cbor(),
        independently_construct_accepted_review_admission(),
    );
    assert_eq!(
        ReviewAdmissionRecord::decode_authoritative(&admission.authoritative_cbor()).unwrap(),
        admission,
    );
}
