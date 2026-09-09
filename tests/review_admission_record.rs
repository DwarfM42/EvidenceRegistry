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

fn independently_construct_accepted_review_admission(selected: bool) -> Vec<u8> {
    let request_ref = reference(1, 300, 0x20);
    let result_ref = reference(2, 301, 0x40);
    let policy_ref = reference(3, 400, 0x60);
    let operation_start = reference(0, 1, 0x10);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[
        0x18,
        32,
        0x01,
        if selected { 0xaa } else { 0xa8 },
        0x00,
        0x01,
        0x01,
        0x18,
        32,
        0x10,
        0x01,
    ]);
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
    if selected {
        bytes.extend_from_slice(&[0x18, 0x18, 0x58, 0x20]);
        bytes.extend_from_slice(&[
            0x65, 0x81, 0x3e, 0x35, 0x6e, 0xab, 0xa8, 0x9c, 0x63, 0x84, 0x75, 0xa4, 0x0c, 0x69,
            0xf7, 0x28, 0x12, 0x3f, 0x43, 0x14, 0x04, 0x0f, 0x07, 0x5a, 0x09, 0xc6, 0xbe, 0x34,
            0xdb, 0x14, 0xe1, 0x14,
        ]);
        bytes.extend_from_slice(&[0x18, 0x19, 0x58, 0x20]);
        bytes.extend_from_slice(&evidence_registry::TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256);
    }
    bytes
}

#[test]
fn accepted_review_admission_requires_exact_resolved_request_result_and_policy_references() {
    let bytes = independently_construct_accepted_review_admission(false);

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
    let mut bytes = independently_construct_accepted_review_admission(false);
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
        independently_construct_accepted_review_admission(false),
    );
    assert_eq!(
        ReviewAdmissionRecord::decode_authoritative(&admission.authoritative_cbor()).unwrap(),
        admission,
    );
}

#[test]
fn review_admission_terminal_selector_requires_the_exact_legacy_and_closure_literals() {
    let predecessor_bytes = independently_construct_accepted_review_admission(false);
    assert_eq!(
        ReviewAdmissionRecord::decode_authoritative(&predecessor_bytes)
            .unwrap()
            .terminal_authority_closure_sha256(),
        None,
        "an absent selector remains a predecessor Admission form"
    );

    let mut selected_bytes = independently_construct_accepted_review_admission(true);
    assert_eq!(
        ReviewAdmissionRecord::decode_authoritative(&selected_bytes)
            .unwrap()
            .terminal_authority_closure_sha256(),
        Some(&evidence_registry::TERMINAL_AUTHORITY_CLOSURE_CORE_SHA256),
    );

    let selected = ReviewAdmissionRecord::new_selected(ReviewAdmissionRecordInput {
        disposition_id: 1,
        review_request_ref: reference(1, 300, 0x20),
        review_result_ref: reference(2, 301, 0x40),
        policy_authority_ref: reference(3, 400, 0x60),
        reason_codes: vec!["SAT".to_owned()],
        operation_start_journal_ref: reference(0, 1, 0x10),
    })
    .unwrap();
    assert_eq!(selected.authoritative_cbor(), selected_bytes);

    let map_header = selected_bytes
        .windows(4)
        .position(|window| window == [0x18, 0x20, 0x01, 0xaa])
        .unwrap()
        + 3;
    let mut old_only = selected_bytes.clone();
    old_only.truncate(old_only.len() - 36);
    old_only[map_header] = 0xa9;
    assert_eq!(
        ReviewAdmissionRecord::decode_authoritative(&old_only),
        Err(evidence_registry::RecordDecodeError),
        "the old type-32 selector alone remains its distinct predecessor lane"
    );

    let mut new_only = selected_bytes.clone();
    let new_only_start = new_only.len() - 72;
    new_only.drain(new_only_start..new_only_start + 36);
    new_only[map_header] = 0xa9;
    assert_eq!(
        ReviewAdmissionRecord::decode_authoritative(&new_only),
        Err(evidence_registry::RecordDecodeError),
        "the closure selector without the exact old selector cannot select the terminal lane"
    );

    let mut wrong_legacy = selected_bytes.clone();
    let wrong_legacy_selector_byte = wrong_legacy.len() - 68;
    wrong_legacy[wrong_legacy_selector_byte] ^= 1;
    assert_eq!(
        ReviewAdmissionRecord::decode_authoritative(&wrong_legacy),
        Err(evidence_registry::RecordDecodeError),
        "an incorrect old selector cannot downgrade a selected terminal form"
    );

    *selected_bytes.last_mut().unwrap() ^= 1;
    assert_eq!(
        ReviewAdmissionRecord::decode_authoritative(&selected_bytes),
        Err(evidence_registry::RecordDecodeError),
        "a malformed selected marker must not silently return to the predecessor form"
    );
}
