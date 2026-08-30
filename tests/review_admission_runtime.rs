use evidence_registry::{
    compare_retained_journal_anchor_history, derive_freeze_root,
    derive_review_admission_section_83_disposition, evaluate_retained_review_admission_policy_46,
    policy_evaluator_registry, resolve_retained_review_package_anchor_input,
    route_retained_review_admission_policy_context, route_retained_review_admission_section_82,
    route_review_admission_after_anchor_comparison,
    validate_review_admission_common_request_result_fields,
    validate_review_package_anchor_transport, validate_review_request_recorded_binding,
    validate_review_result_recorded_binding, AuthoritativeRegistryStore, EventRecordId,
    ExactRecordByteResolver, FreezeAttemptId, FreezeAttemptStartRecord,
    FreezeAttemptStartRecordInput, GenesisJournalEntry, GenesisRecord, GenesisRecordInput,
    JournalAnchor, JournalAnchorHistoryComparison, JournalEntryHash, JournalEntryIndex,
    JournalReference, RecordId, RegistryId, RetainedJournal, RetainedJournalError,
    RetainedReviewPackageAnchorInputError, ReviewAdmissionAcceptanceError,
    ReviewAdmissionCommonRequestResultError, ReviewAdmissionExactScopeProfileError,
    ReviewAdmissionPolicy46RouteOutcome, ReviewAdmissionPolicyContextPrerequisitesError,
    ReviewAdmissionPolicyContextRouteOutcome, ReviewAdmissionPolicyRecord,
    ReviewAdmissionPolicyRecordedBindingError, ReviewAdmissionRuntimeError,
    ReviewAdmissionRuntimeOutcome, ReviewAdmissionSection82AuthorityError,
    ReviewAdmissionSection82PrerequisiteFailure, ReviewAdmissionSection82RoutingOutcome,
    ReviewAdmissionSection83Disposition, ReviewPackageAnchorTransportError, ReviewRequestRecord,
    ReviewRequestRecordedBindingError, ReviewResultRecord, ReviewResultRecordedBindingError,
    StrictRecordFrame, REVIEW_ADMISSION_MAX_OPAQUE_INPUT_BYTES,
    REVIEW_ADMISSION_MAX_OUTSTANDING_ACCEPTANCES,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

fn id(first: u8) -> [u8; 32] {
    core::array::from_fn(|index| first.wrapping_add(index as u8))
}

fn genesis_record() -> GenesisRecord {
    GenesisRecord::new(GenesisRecordInput {
        registry_id: RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        journal_format_version: 1,
        record_identity_profile_id: 1,
        storage_capability_class_id: RecordId::try_from(id(0x40).as_slice()).unwrap(),
        environment_observation_id: RecordId::try_from(id(0x60).as_slice()).unwrap(),
        created_by_tool_version: "review-admission-runtime-test".to_owned(),
    })
    .unwrap()
}

fn genesis() -> GenesisJournalEntry {
    let record = genesis_record();
    GenesisJournalEntry::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        EventRecordId::try_from(record.record_id().as_bytes().as_slice()).unwrap(),
        RecordId::try_from(id(0x40).as_slice()).unwrap(),
        RecordId::try_from(id(0x60).as_slice()).unwrap(),
    )
}

fn append_bstr_32(output: &mut Vec<u8>, bytes: [u8; 32]) {
    output.extend_from_slice(&[0x58, 0x20]);
    output.extend_from_slice(&bytes);
}

fn append_journal_reference(output: &mut Vec<u8>, reference: &JournalReference) {
    output.push(0x85);
    append_bstr_32(output, *reference.registry_id().as_bytes());
    output.push(reference.entry_index().value() as u8);
    append_bstr_32(output, *reference.entry_hash().as_bytes());
    match reference.event_type_id().value() {
        value @ 0..=23 => output.push(value as u8),
        value @ 24..=0xff => output.extend_from_slice(&[0x18, value as u8]),
        value => output.extend_from_slice(&[0x19, (value >> 8) as u8, value as u8]),
    }
    append_bstr_32(output, *reference.event_record_id().as_bytes());
}

const MANIFEST_RECORD_HEX: &str = concat!(
    "84781a45766964656e636552656769737472792e5265636f72642e76310201a700010102105820",
    "e0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff110112011301148185",
    "0181416100015820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
);

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}

fn freeze_attempt_start_record() -> FreezeAttemptStartRecord {
    let registry_id = RegistryId::try_from(id(0x00).as_slice()).unwrap();
    let freeze_attempt_id = FreezeAttemptId::try_from(id(0xa0).as_slice()).unwrap();
    FreezeAttemptStartRecord::new(FreezeAttemptStartRecordInput {
        freeze_attempt_id,
        intended_root_id: derive_freeze_root(registry_id, freeze_attempt_id).intended_root_id(),
        subject_id: id(0xe0),
        policy_record_id: RecordId::try_from(id(0x40).as_slice()).unwrap(),
    })
}

fn freeze_start_entry(
    previous_entry_hash: JournalEntryHash,
    start_record: &FreezeAttemptStartRecord,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xae);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01]);
    append_bstr_32(&mut bytes, id(0x00));
    bytes.extend_from_slice(&[0x02, 0x01, 0x03]);
    append_bstr_32(&mut bytes, *previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x18, 0x64, 0x05]);
    append_bstr_32(&mut bytes, *start_record.record_id().as_bytes());
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x80, 0x08, 0x02, 0x09]);
    append_bstr_32(&mut bytes, id(0xa0));
    bytes.push(0x0a);
    append_bstr_32(&mut bytes, id(0x40));
    bytes.push(0x0b);
    append_bstr_32(&mut bytes, id(0x60));
    bytes.push(0x10);
    append_bstr_32(
        &mut bytes,
        *start_record.input().freeze_attempt_id.as_bytes(),
    );
    bytes.push(0x11);
    append_bstr_32(
        &mut bytes,
        *start_record.input().intended_root_id.as_bytes(),
    );
    bytes
}

fn freeze_receipt_bytes(start_reference: &JournalReference, manifest_id: RecordId) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x04, 0x01, 0xb1, 0x00, 0x01, 0x01, 0x04]);
    bytes.push(0x10);
    append_bstr_32(&mut bytes, id(0xa0));
    bytes.push(0x11);
    append_bstr_32(&mut bytes, id(0xb0));
    bytes.push(0x12);
    append_journal_reference(&mut bytes, start_reference);
    bytes.push(0x13);
    append_bstr_32(&mut bytes, id(0xe0));
    bytes.push(0x14);
    append_bstr_32(&mut bytes, *manifest_id.as_bytes());
    bytes.extend_from_slice(&[0x15, 0x01]);
    bytes.push(0x16);
    append_bstr_32(&mut bytes, id(0xd0));
    bytes.extend_from_slice(&[0x17, 0x01]);
    bytes.extend_from_slice(&[0x18, 0x18]);
    append_bstr_32(&mut bytes, id(0xf0));
    bytes.extend_from_slice(&[0x18, 0x19]);
    append_bstr_32(&mut bytes, id(0x40));
    bytes.extend_from_slice(&[
        0x18, 0x1a, 0x01, 0x18, 0x1b, 0x01, 0x18, 0x1c, 0x01, 0x18, 0x1d, 0xf5, 0x18, 0x1f, 0x64,
    ]);
    bytes.extend_from_slice(b"test");
    bytes
}

fn freeze_committed_entry(
    previous_entry_hash: JournalEntryHash,
    start_reference: &JournalReference,
    receipt_record_id: RecordId,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xae);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01]);
    append_bstr_32(&mut bytes, id(0x00));
    bytes.extend_from_slice(&[0x02, 0x02, 0x03]);
    append_bstr_32(&mut bytes, *previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x18, 0x65, 0x05]);
    append_bstr_32(&mut bytes, *receipt_record_id.as_bytes());
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x81]);
    append_journal_reference(&mut bytes, start_reference);
    bytes.extend_from_slice(&[0x08, 0x02, 0x09]);
    append_bstr_32(&mut bytes, id(0xa0));
    bytes.push(0x0a);
    append_bstr_32(&mut bytes, id(0x40));
    bytes.push(0x0b);
    append_bstr_32(&mut bytes, id(0x60));
    bytes.push(0x10);
    append_bstr_32(&mut bytes, id(0xa0));
    bytes.push(0x12);
    append_journal_reference(&mut bytes, start_reference);
    bytes
}

fn reference(index: u64, event_type_id: u16, record_first: u8) -> JournalReference {
    JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(index).unwrap(),
        JournalEntryHash::try_from(id(0x80 + index as u8).as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(u64::from(event_type_id)).unwrap(),
        EventRecordId::try_from(id(record_first).as_slice()).unwrap(),
    )
}

fn independently_construct_version_1_review_request() -> Vec<u8> {
    independently_construct_version_1_review_request_with_anchor_id(id(0x10))
}

fn independently_construct_version_1_review_request_with_anchor_id(
    review_package_anchor_id: [u8; 32],
) -> Vec<u8> {
    let freeze_authority = reference(1, 101, 0x20);
    let policy_authority = reference(2, 400, 0x40);
    let operation_start = reference(3, 300, 0x60);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x18, 30, 0x01, 0xac, 0x00, 0x01, 0x01, 0x18, 30, 0x10]);
    append_journal_reference(&mut bytes, &freeze_authority);
    bytes.push(0x11);
    append_bstr_32(&mut bytes, id(0x80));
    bytes.extend_from_slice(&[0x12, 0x07, 0x13]);
    append_bstr_32(&mut bytes, id(0xa0));
    bytes.push(0x14);
    append_journal_reference(&mut bytes, &policy_authority);
    bytes.push(0x15);
    append_bstr_32(&mut bytes, id(0xc0));
    bytes.push(0x16);
    append_bstr_32(&mut bytes, id(0xe0));
    bytes.push(0x17);
    append_bstr_32(&mut bytes, review_package_anchor_id);
    bytes.extend_from_slice(&[0x18, 0x18]);
    append_journal_reference(&mut bytes, &operation_start);
    bytes.extend_from_slice(&[0x18, 0x19, 0x01]);
    bytes
}

fn independently_construct_version_1_review_request_with_freeze_policy_scope_anchor_and_start(
    freeze_authority: &JournalReference,
    policy_authority: &JournalReference,
    scope_ref: RecordId,
    review_package_anchor_id: [u8; 32],
    operation_start: &JournalReference,
    manifest_id: RecordId,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x18, 30, 0x01, 0xac, 0x00, 0x01, 0x01, 0x18, 30, 0x10]);
    append_journal_reference(&mut bytes, freeze_authority);
    bytes.push(0x11);
    append_bstr_32(&mut bytes, *manifest_id.as_bytes());
    bytes.extend_from_slice(&[0x12, 0x07, 0x13]);
    append_bstr_32(&mut bytes, id(0xa0));
    bytes.push(0x14);
    append_journal_reference(&mut bytes, policy_authority);
    bytes.push(0x15);
    append_bstr_32(&mut bytes, *scope_ref.as_bytes());
    bytes.push(0x16);
    append_bstr_32(&mut bytes, id(0xe0));
    bytes.push(0x17);
    append_bstr_32(&mut bytes, review_package_anchor_id);
    bytes.extend_from_slice(&[0x18, 0x18]);
    append_journal_reference(&mut bytes, operation_start);
    bytes.extend_from_slice(&[0x18, 0x19, 0x01]);
    bytes
}

fn append_identity_dependencies(output: &mut Vec<u8>, dependencies: &[(u8, [u8; 32])]) {
    assert!(dependencies.len() < 24);
    output.push(0x80 + dependencies.len() as u8);
    for (kind, value) in dependencies {
        output.extend_from_slice(&[0x82, *kind]);
        append_bstr_32(output, *value);
    }
}

fn review_request_recorded_entry(
    previous_entry_hash: JournalEntryHash,
    review_request_record_id: RecordId,
    review_package_anchor_id: Option<[u8; 32]>,
) -> Vec<u8> {
    let identity_dependencies = review_package_anchor_id
        .map(|anchor_id| vec![(2, anchor_id)])
        .unwrap_or_default();
    review_request_recorded_entry_with_identity_dependencies(
        previous_entry_hash,
        review_request_record_id,
        &identity_dependencies,
    )
}

fn review_request_recorded_entry_with_identity_dependencies(
    previous_entry_hash: JournalEntryHash,
    review_request_record_id: RecordId,
    identity_dependencies: &[(u8, [u8; 32])],
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.extend_from_slice(&[0xac, 0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, 0x01, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x19, 0x01, 0x2c, 0x05]);
    append_bstr_32(&mut bytes, *review_request_record_id.as_bytes());
    bytes.push(0x06);
    append_identity_dependencies(&mut bytes, identity_dependencies);
    bytes.extend_from_slice(&[0x07, 0x80, 0x08, 0x04, 0x09]);
    append_bstr_32(&mut bytes, *review_request_record_id.as_bytes());
    bytes.extend_from_slice(&[0x0a]);
    append_bstr_32(&mut bytes, id(0x40));
    bytes.extend_from_slice(&[0x0b]);
    append_bstr_32(&mut bytes, id(0x60));
    bytes
}

fn review_request_recorded_entry_at(
    entry_index: u8,
    previous_entry_hash: JournalEntryHash,
    review_request_record_id: RecordId,
    review_package_anchor_id: [u8; 32],
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.extend_from_slice(&[0xac, 0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, entry_index, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x19, 0x01, 0x2c, 0x05]);
    append_bstr_32(&mut bytes, *review_request_record_id.as_bytes());
    bytes.extend_from_slice(&[0x06, 0x81, 0x82, 0x02]);
    append_bstr_32(&mut bytes, review_package_anchor_id);
    bytes.extend_from_slice(&[0x07, 0x80, 0x08, 0x04, 0x09]);
    append_bstr_32(&mut bytes, *review_request_record_id.as_bytes());
    bytes.extend_from_slice(&[0x0a]);
    append_bstr_32(&mut bytes, id(0x40));
    bytes.extend_from_slice(&[0x0b]);
    append_bstr_32(&mut bytes, id(0x60));
    bytes
}

fn review_request_recorded_entry_at_with_authorities(
    entry_index: u8,
    previous_entry_hash: JournalEntryHash,
    review_request_record_id: RecordId,
    review_package_anchor_id: [u8; 32],
    identity_dependencies: &[(u8, [u8; 32])],
    authorities: &[&JournalReference],
) -> Vec<u8> {
    assert!(!identity_dependencies.is_empty() && identity_dependencies.len() < 24);
    assert!(authorities.len() < 24);
    let mut bytes = review_request_recorded_entry_at(
        entry_index,
        previous_entry_hash,
        review_request_record_id,
        review_package_anchor_id,
    );
    let identity_start = bytes
        .windows(4)
        .position(|window| window == [0x06, 0x81, 0x82, 0x02])
        .expect("independent Request helper has the expected Anchor identity set")
        + 1;
    let authority_key = bytes
        .windows(4)
        .position(|window| window == [0x07, 0x80, 0x08, 0x04])
        .expect("independent Request helper has the expected empty authority set");
    let mut encoded_identities = Vec::new();
    append_identity_dependencies(&mut encoded_identities, identity_dependencies);
    bytes.splice(identity_start..authority_key, encoded_identities);
    if !authorities.is_empty() {
        let empty_authorities = bytes
            .windows(4)
            .position(|window| window == [0x07, 0x80, 0x08, 0x04])
            .expect("independent Request helper has the expected empty authority set");
        let mut encoded_authorities = vec![0x80 + authorities.len() as u8];
        for authority in authorities {
            append_journal_reference(&mut encoded_authorities, authority);
        }
        bytes.splice(
            empty_authorities + 1..empty_authorities + 2,
            encoded_authorities,
        );
    }
    bytes
}

fn independently_construct_version_1_review_result() -> Vec<u8> {
    independently_construct_version_1_review_result_with_transport(
        reference(1, 300, 0x20),
        id(0x10),
    )
}

fn independently_construct_version_1_review_result_with_transport(
    request_authority: JournalReference,
    review_package_anchor_id: [u8; 32],
) -> Vec<u8> {
    independently_construct_version_1_review_result_with_section_82_fields(
        request_authority,
        reference(2, 101, 0x40),
        review_package_anchor_id,
    )
}

fn independently_construct_version_1_review_result_with_section_82_fields(
    request_authority: JournalReference,
    freeze_authority: JournalReference,
    review_package_anchor_id: [u8; 32],
) -> Vec<u8> {
    let operation_start = reference(3, 300, 0x60);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x18, 31, 0x01, 0xaf, 0x00, 0x01, 0x01, 0x18, 31, 0x10]);
    append_journal_reference(&mut bytes, &request_authority);
    bytes.push(0x11);
    append_journal_reference(&mut bytes, &freeze_authority);
    bytes.push(0x12);
    append_bstr_32(&mut bytes, id(0x80));
    bytes.extend_from_slice(&[0x13, 0x07, 0x14]);
    append_bstr_32(&mut bytes, id(0xc0));
    bytes.push(0x15);
    append_bstr_32(&mut bytes, id(0xe0));
    bytes.extend_from_slice(&[0x16, 0x01, 0x17, 0x01, 0x18, 0x18, 0x81, 0x62]);
    bytes.extend_from_slice(b"OK");
    bytes.extend_from_slice(&[0x18, 0x19, 0x80, 0x18, 0x1b]);
    append_bstr_32(&mut bytes, review_package_anchor_id);
    bytes.extend_from_slice(&[0x18, 0x1c]);
    append_journal_reference(&mut bytes, &operation_start);
    bytes.extend_from_slice(&[0x18, 0x1d, 0x01]);
    bytes
}

fn independently_construct_version_1_review_result_with_freeze_scope_and_start(
    request_authority: &JournalReference,
    freeze_authority: &JournalReference,
    scope_ref: RecordId,
    review_package_anchor_id: [u8; 32],
    operation_start: &JournalReference,
    manifest_id: RecordId,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x18, 31, 0x01, 0xaf, 0x00, 0x01, 0x01, 0x18, 31, 0x10]);
    append_journal_reference(&mut bytes, request_authority);
    bytes.push(0x11);
    append_journal_reference(&mut bytes, freeze_authority);
    bytes.push(0x12);
    append_bstr_32(&mut bytes, *manifest_id.as_bytes());
    bytes.extend_from_slice(&[0x13, 0x07, 0x14]);
    append_bstr_32(&mut bytes, *scope_ref.as_bytes());
    bytes.push(0x15);
    append_bstr_32(&mut bytes, id(0xe0));
    bytes.extend_from_slice(&[0x16, 0x01, 0x17, 0x01, 0x18, 0x18, 0x81, 0x62]);
    bytes.extend_from_slice(b"OK");
    bytes.extend_from_slice(&[0x18, 0x19, 0x80, 0x18, 0x1b]);
    append_bstr_32(&mut bytes, review_package_anchor_id);
    bytes.extend_from_slice(&[0x18, 0x1c]);
    append_journal_reference(&mut bytes, operation_start);
    bytes.extend_from_slice(&[0x18, 0x1d, 0x01]);
    bytes
}

fn review_result_recorded_entry(
    entry_index: u8,
    previous_entry_hash: JournalEntryHash,
    review_result_record_id: RecordId,
    review_package_anchor_id: Option<[u8; 32]>,
    request_authority_dependency: Option<&JournalReference>,
) -> Vec<u8> {
    let identity_dependencies = review_package_anchor_id
        .map(|anchor_id| vec![(2, anchor_id)])
        .unwrap_or_default();
    review_result_recorded_entry_with_identity_dependencies(
        entry_index,
        previous_entry_hash,
        review_result_record_id,
        &identity_dependencies,
        request_authority_dependency,
    )
}

fn review_result_recorded_entry_with_identity_dependencies(
    entry_index: u8,
    previous_entry_hash: JournalEntryHash,
    review_result_record_id: RecordId,
    identity_dependencies: &[(u8, [u8; 32])],
    request_authority_dependency: Option<&JournalReference>,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.extend_from_slice(&[0xac, 0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, entry_index, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x19, 0x01, 0x2d, 0x05]);
    append_bstr_32(&mut bytes, *review_result_record_id.as_bytes());
    bytes.push(0x06);
    append_identity_dependencies(&mut bytes, identity_dependencies);
    bytes.push(0x07);
    if let Some(request_authority_dependency) = request_authority_dependency {
        bytes.push(0x81);
        append_journal_reference(&mut bytes, request_authority_dependency);
    } else {
        bytes.push(0x80);
    }
    bytes.extend_from_slice(&[0x08, 0x05, 0x09]);
    append_bstr_32(&mut bytes, *review_result_record_id.as_bytes());
    bytes.extend_from_slice(&[0x0a]);
    append_bstr_32(&mut bytes, id(0x40));
    bytes.extend_from_slice(&[0x0b]);
    append_bstr_32(&mut bytes, id(0x60));
    bytes
}

fn review_result_recorded_entry_with_authorities(
    entry_index: u8,
    previous_entry_hash: JournalEntryHash,
    review_result_record_id: RecordId,
    review_package_anchor_id: [u8; 32],
    authorities: &[&JournalReference],
) -> Vec<u8> {
    assert!(!authorities.is_empty() && authorities.len() < 24);
    let mut bytes = review_result_recorded_entry(
        entry_index,
        previous_entry_hash,
        review_result_record_id,
        Some(review_package_anchor_id),
        None,
    );
    let empty_authorities = bytes
        .windows(4)
        .position(|window| window == [0x07, 0x80, 0x08, 0x05])
        .expect("independent Result helper has the expected empty authority set");
    let mut encoded_authorities = vec![0x80 + authorities.len() as u8];
    for authority in authorities {
        append_journal_reference(&mut encoded_authorities, authority);
    }
    bytes.splice(
        empty_authorities + 1..empty_authorities + 2,
        encoded_authorities,
    );
    bytes
}

fn independently_construct_review_admission_policy_with_requirement_roles(
    operation_start: &JournalReference,
    gate_scope_ref: RecordId,
    review_requirement_scope_ref: RecordId,
    review_roles: &[u8],
) -> Vec<u8> {
    assert!(!review_roles.is_empty() && review_roles.len() < 24);
    assert!(review_roles.windows(2).all(|pair| pair[0] < pair[1]));
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x18, 40, 0x01, 0xa9, 0x00, 0x01, 0x01, 0x18, 40, 0x10]);
    append_bstr_32(&mut bytes, *gate_scope_ref.as_bytes());
    bytes.push(0x11);
    bytes.push(0x80 + review_roles.len() as u8);
    for review_role in review_roles {
        bytes.extend_from_slice(&[0xa5, 0x00, *review_role, 0x01]);
        append_bstr_32(&mut bytes, *review_requirement_scope_ref.as_bytes());
        bytes.push(0x02);
        append_bstr_32(&mut bytes, id(0xe0));
        bytes.push(0x03);
        append_bstr_32(&mut bytes, id(0xa0));
        bytes.extend_from_slice(&[0x04, 0x01]);
    }
    bytes.extend_from_slice(&[0x12, 0x81, 0x01, 0x13, 0x81, 0x01]);
    bytes.extend_from_slice(&[0x18, 0x18, 0xa1, 0x00, 0x82, 0x01, 0x02, 0x18, 0x1d]);
    append_journal_reference(&mut bytes, operation_start);
    bytes.extend_from_slice(&[0x18, 0x1e, 0x81, 0x02]);
    bytes
}

fn policy_recorded_entry_at(
    entry_index: u8,
    previous_entry_hash: JournalEntryHash,
    policy_record_id: RecordId,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.extend_from_slice(&[0xac, 0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, entry_index, 0x03, 0x58, 0x20]);
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

fn independently_construct_exact_review_admission_scope() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[
        0x14, 0x01, 0xa5, 0x00, 0x01, 0x01, 0x14, 0x10, 0x01, 0x11, 0x01, 0x12, 0x40,
    ]);
    bytes
}

fn independently_construct_labeled_exact_review_admission_scope(label: &str) -> Vec<u8> {
    assert!(label.len() < 24);
    let mut bytes = independently_construct_exact_review_admission_scope();
    let map_offset = bytes
        .windows(4)
        .position(|window| window == [0x14, 0x01, 0xa5, 0x00])
        .unwrap()
        + 2;
    bytes[map_offset] = 0xa6;
    bytes.extend_from_slice(&[0x13, 0x60 + label.len() as u8]);
    bytes.extend_from_slice(label.as_bytes());
    bytes
}

struct RecordBytes(Vec<(RecordId, Vec<u8>)>);

impl ExactRecordByteResolver for RecordBytes {
    fn resolve(&self, record_id: RecordId) -> Option<&[u8]> {
        self.0
            .iter()
            .find(|(candidate, _)| *candidate == record_id)
            .map(|(_, bytes)| bytes.as_slice())
    }
}

static NEXT_AUTHORITATIVE_REVIEW_STORE_FIXTURE: AtomicU64 = AtomicU64::new(1);

struct AuthoritativeReviewStoreFixtureDir {
    path: PathBuf,
}

impl AuthoritativeReviewStoreFixtureDir {
    fn from_fixture(fixture: &AuthoritativeReviewAdmissionFixture) -> Self {
        let sequence = NEXT_AUTHORITATIVE_REVIEW_STORE_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "evidence-registry-authoritative-review-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        fs::create_dir(path.join("registry")).unwrap();
        fs::create_dir(path.join("journal")).unwrap();
        fs::create_dir(path.join("records")).unwrap();
        fs::write(
            path.join("registry/genesis.cbor"),
            &fixture.genesis_record_bytes,
        )
        .unwrap();
        for (index, bytes) in fixture.journal_entry_bytes.iter().enumerate() {
            fs::write(
                path.join("journal").join(format!("{index:020}.cbor")),
                bytes,
            )
            .unwrap();
        }
        for (record_id, bytes) in &fixture.resolver.0 {
            fs::write(
                path.join("records").join(record_filename(*record_id)),
                bytes,
            )
            .unwrap();
        }
        Self { path }
    }
}

impl Drop for AuthoritativeReviewStoreFixtureDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn record_filename(record_id: RecordId) -> String {
    let mut name = String::with_capacity(69);
    for byte in record_id.as_bytes() {
        use std::fmt::Write as _;
        write!(&mut name, "{byte:02x}").unwrap();
    }
    name.push_str(".cbor");
    name
}

struct AuthoritativeReviewAdmissionFixture {
    journal: RetainedJournal,
    genesis_record_bytes: Vec<u8>,
    journal_entry_bytes: Vec<Vec<u8>>,
    request_reference: JournalReference,
    request_bytes: Vec<u8>,
    result_reference: JournalReference,
    result_bytes: Vec<u8>,
    resolver: RecordBytes,
}

#[derive(Default)]
struct ReviewAdmissionFixtureOverrides {
    policy_operation_start: Option<JournalReference>,
    policy_context_id: Option<u8>,
    gate_scope_bytes: Option<Vec<u8>>,
    common_scope_bytes: Option<Vec<u8>>,
    request_operation_start: Option<JournalReference>,
    result_operation_start: Option<JournalReference>,
}

struct FutureReviewAdmissionPair {
    request_entry: Vec<u8>,
    request_reference: JournalReference,
    request_bytes: Vec<u8>,
    result_entry: Vec<u8>,
    result_reference: JournalReference,
    result_bytes: Vec<u8>,
}

#[derive(Clone, Copy)]
enum AuthorityDependencyMode {
    Missing,
    Exact,
    Extra,
}

#[derive(Clone, Copy)]
enum IdentityDependencyMode {
    Missing,
    Exact,
    Extra,
}

fn authoritative_review_admission_fixture(
    evaluator_1015_scope_matches: bool,
) -> AuthoritativeReviewAdmissionFixture {
    authoritative_review_admission_fixture_with_roles(evaluator_1015_scope_matches, &[7])
}

fn authoritative_review_admission_fixture_with_roles(
    evaluator_1015_scope_matches: bool,
    review_roles: &[u8],
) -> AuthoritativeReviewAdmissionFixture {
    authoritative_review_admission_fixture_with_roles_and_authority_dependencies(
        evaluator_1015_scope_matches,
        review_roles,
        IdentityDependencyMode::Exact,
        AuthorityDependencyMode::Exact,
        AuthorityDependencyMode::Exact,
    )
}

fn authoritative_review_admission_fixture_with_roles_and_authority_dependencies(
    evaluator_1015_scope_matches: bool,
    review_roles: &[u8],
    request_identities: IdentityDependencyMode,
    request_authorities: AuthorityDependencyMode,
    result_authorities: AuthorityDependencyMode,
) -> AuthoritativeReviewAdmissionFixture {
    authoritative_review_admission_fixture_with_overrides(
        evaluator_1015_scope_matches,
        review_roles,
        request_identities,
        request_authorities,
        result_authorities,
        ReviewAdmissionFixtureOverrides::default(),
    )
}

fn authoritative_review_admission_fixture_with_overrides(
    evaluator_1015_scope_matches: bool,
    review_roles: &[u8],
    request_identities: IdentityDependencyMode,
    request_authorities: AuthorityDependencyMode,
    result_authorities: AuthorityDependencyMode,
    overrides: ReviewAdmissionFixtureOverrides,
) -> AuthoritativeReviewAdmissionFixture {
    let ReviewAdmissionFixtureOverrides {
        policy_operation_start,
        policy_context_id,
        gate_scope_bytes,
        common_scope_bytes,
        request_operation_start,
        result_operation_start,
    } = overrides;
    let genesis = genesis();
    let registry_id = RegistryId::try_from(id(0x00).as_slice()).unwrap();
    let genesis_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(0_u64).unwrap(),
        genesis.entry_hash(),
        evidence_registry::EventTypeId::try_from(1_u64).unwrap(),
        EventRecordId::try_from(genesis_record().record_id().as_bytes().as_slice()).unwrap(),
    );
    let start_record = freeze_attempt_start_record();
    let start_entry = freeze_start_entry(genesis.entry_hash(), &start_record);
    let start_entry_hash: [u8; 32] = Sha256::digest(&start_entry).into();
    let start_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(start_entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(100_u64).unwrap(),
        EventRecordId::try_from(start_record.record_id().as_bytes().as_slice()).unwrap(),
    );
    let manifest_bytes = hex_bytes(MANIFEST_RECORD_HEX);
    let manifest_id = RecordId::try_from(Sha256::digest(&manifest_bytes).as_slice()).unwrap();
    let receipt_bytes = freeze_receipt_bytes(&start_reference, manifest_id);
    let receipt_id = RecordId::try_from(Sha256::digest(&receipt_bytes).as_slice()).unwrap();
    let committed_entry = freeze_committed_entry(
        JournalEntryHash::try_from(start_entry_hash.as_slice()).unwrap(),
        &start_reference,
        receipt_id,
    );
    let committed_entry_hash: [u8; 32] = Sha256::digest(&committed_entry).into();
    let freeze_authority_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(2_u64).unwrap(),
        JournalEntryHash::try_from(committed_entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(101_u64).unwrap(),
        EventRecordId::try_from(receipt_id.as_bytes().as_slice()).unwrap(),
    );
    let gate_scope_bytes =
        gate_scope_bytes.unwrap_or_else(independently_construct_exact_review_admission_scope);
    let gate_scope_id = RecordId::try_from(Sha256::digest(&gate_scope_bytes).as_slice()).unwrap();
    let common_scope_bytes = common_scope_bytes.unwrap_or_else(|| {
        if evaluator_1015_scope_matches {
            gate_scope_bytes.clone()
        } else {
            independently_construct_labeled_exact_review_admission_scope("other")
        }
    });
    let common_scope_id =
        RecordId::try_from(Sha256::digest(&common_scope_bytes).as_slice()).unwrap();
    let mut policy_bytes = independently_construct_review_admission_policy_with_requirement_roles(
        policy_operation_start
            .as_ref()
            .unwrap_or(&genesis_reference),
        gate_scope_id,
        common_scope_id,
        review_roles,
    );
    if let Some(policy_context_id) = policy_context_id {
        *policy_bytes.last_mut().unwrap() = policy_context_id;
    }
    let policy_id = RecordId::try_from(Sha256::digest(&policy_bytes).as_slice()).unwrap();
    let policy_entry = policy_recorded_entry_at(
        3,
        JournalEntryHash::try_from(committed_entry_hash.as_slice()).unwrap(),
        policy_id,
    );
    let policy_entry_hash: [u8; 32] = Sha256::digest(&policy_entry).into();
    let policy_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(3_u64).unwrap(),
        JournalEntryHash::try_from(policy_entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(400_u64).unwrap(),
        EventRecordId::try_from(policy_id.as_bytes().as_slice()).unwrap(),
    );
    let package_anchor = JournalAnchor::new(
        registry_id,
        JournalEntryIndex::try_from(0_u64).unwrap(),
        genesis.entry_hash(),
        1,
    )
    .unwrap();
    let request_bytes =
        independently_construct_version_1_review_request_with_freeze_policy_scope_anchor_and_start(
            &freeze_authority_reference,
            &policy_reference,
            common_scope_id,
            *package_anchor.anchor_id().as_bytes(),
            request_operation_start
                .as_ref()
                .unwrap_or(&policy_reference),
            manifest_id,
        );
    let request = ReviewRequestRecord::decode_authoritative(&request_bytes).unwrap();
    let mut request_identity_dependencies = vec![
        (1, *request.manifest_id().as_bytes()),
        (1, *request.required_checks_ref().as_bytes()),
        (1, *request.review_scope_ref().as_bytes()),
        (1, *request.review_method_ref().as_bytes()),
        (2, *request.review_package_anchor_id().as_bytes()),
    ];
    match request_identities {
        IdentityDependencyMode::Missing => {
            request_identity_dependencies.remove(0);
        }
        IdentityDependencyMode::Exact => {}
        IdentityDependencyMode::Extra => {
            request_identity_dependencies.push((1, *start_record.record_id().as_bytes()))
        }
    }
    request_identity_dependencies.sort_unstable();
    let request_entry = match request_authorities {
        AuthorityDependencyMode::Missing => review_request_recorded_entry_at_with_authorities(
            4,
            JournalEntryHash::try_from(policy_entry_hash.as_slice()).unwrap(),
            request.record_id(),
            *package_anchor.anchor_id().as_bytes(),
            &request_identity_dependencies,
            &[],
        ),
        AuthorityDependencyMode::Exact => review_request_recorded_entry_at_with_authorities(
            4,
            JournalEntryHash::try_from(policy_entry_hash.as_slice()).unwrap(),
            request.record_id(),
            *package_anchor.anchor_id().as_bytes(),
            &request_identity_dependencies,
            &[&freeze_authority_reference, &policy_reference],
        ),
        AuthorityDependencyMode::Extra => review_request_recorded_entry_at_with_authorities(
            4,
            JournalEntryHash::try_from(policy_entry_hash.as_slice()).unwrap(),
            request.record_id(),
            *package_anchor.anchor_id().as_bytes(),
            &request_identity_dependencies,
            &[
                &genesis_reference,
                &freeze_authority_reference,
                &policy_reference,
            ],
        ),
    };
    let request_entry_hash: [u8; 32] = Sha256::digest(&request_entry).into();
    let request_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(4_u64).unwrap(),
        JournalEntryHash::try_from(request_entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(300_u64).unwrap(),
        EventRecordId::try_from(request.record_id().as_bytes().as_slice()).unwrap(),
    );
    let result_bytes = independently_construct_version_1_review_result_with_freeze_scope_and_start(
        &request_reference,
        &freeze_authority_reference,
        common_scope_id,
        *package_anchor.anchor_id().as_bytes(),
        result_operation_start
            .as_ref()
            .unwrap_or(&request_reference),
        manifest_id,
    );
    let result = ReviewResultRecord::decode_authoritative(&result_bytes).unwrap();
    let result_entry = match result_authorities {
        AuthorityDependencyMode::Missing => review_result_recorded_entry(
            5,
            JournalEntryHash::try_from(request_entry_hash.as_slice()).unwrap(),
            result.record_id(),
            Some(*package_anchor.anchor_id().as_bytes()),
            None,
        ),
        AuthorityDependencyMode::Exact => review_result_recorded_entry(
            5,
            JournalEntryHash::try_from(request_entry_hash.as_slice()).unwrap(),
            result.record_id(),
            Some(*package_anchor.anchor_id().as_bytes()),
            Some(&request_reference),
        ),
        AuthorityDependencyMode::Extra => review_result_recorded_entry_with_authorities(
            5,
            JournalEntryHash::try_from(request_entry_hash.as_slice()).unwrap(),
            result.record_id(),
            *package_anchor.anchor_id().as_bytes(),
            &[&freeze_authority_reference, &request_reference],
        ),
    };
    let result_entry_hash: [u8; 32] = Sha256::digest(&result_entry).into();
    let result_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(5_u64).unwrap(),
        JournalEntryHash::try_from(result_entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(301_u64).unwrap(),
        EventRecordId::try_from(result.record_id().as_bytes().as_slice()).unwrap(),
    );
    let genesis_entry_bytes = genesis.authoritative_cbor();
    let genesis_record_bytes = genesis_record().authoritative_cbor();
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&start_entry).unwrap();
    journal.append_strict_entry(&committed_entry).unwrap();
    journal.append_strict_entry(&policy_entry).unwrap();
    journal.append_strict_entry(&request_entry).unwrap();
    journal.append_strict_entry(&result_entry).unwrap();
    let mut records = vec![
        (genesis_record().record_id(), genesis_record_bytes.clone()),
        (start_record.record_id(), start_record.authoritative_cbor()),
        (receipt_id, receipt_bytes),
        (manifest_id, manifest_bytes),
        (gate_scope_id, gate_scope_bytes),
        (policy_id, policy_bytes),
        (request.record_id(), request_bytes.clone()),
        (result.record_id(), result_bytes.clone()),
    ];
    if common_scope_id != gate_scope_id {
        records.push((common_scope_id, common_scope_bytes));
    }
    AuthoritativeReviewAdmissionFixture {
        journal,
        genesis_record_bytes,
        journal_entry_bytes: vec![
            genesis_entry_bytes,
            start_entry,
            committed_entry,
            policy_entry,
            request_entry,
            result_entry,
        ],
        request_reference,
        request_bytes,
        result_reference,
        result_bytes,
        resolver: RecordBytes(records),
    }
}

fn future_review_admission_pair(
    fixture: &AuthoritativeReviewAdmissionFixture,
    operation_start: &JournalReference,
) -> FutureReviewAdmissionPair {
    let existing_request =
        ReviewRequestRecord::decode_authoritative(&fixture.request_bytes).unwrap();
    let request_bytes =
        independently_construct_version_1_review_request_with_freeze_policy_scope_anchor_and_start(
            existing_request.freeze_authority_ref(),
            existing_request.policy_authority_ref(),
            existing_request.review_scope_ref(),
            *existing_request.review_package_anchor_id().as_bytes(),
            operation_start,
            existing_request.manifest_id(),
        );
    let request = ReviewRequestRecord::decode_authoritative(&request_bytes).unwrap();
    let request_index = u8::try_from(operation_start.entry_index().value() + 1).unwrap();
    let mut request_identity_dependencies = vec![
        (1, *request.manifest_id().as_bytes()),
        (1, *request.required_checks_ref().as_bytes()),
        (1, *request.review_scope_ref().as_bytes()),
        (1, *request.review_method_ref().as_bytes()),
        (2, *request.review_package_anchor_id().as_bytes()),
    ];
    request_identity_dependencies.sort_unstable();
    let request_entry = review_request_recorded_entry_at_with_authorities(
        request_index,
        operation_start.entry_hash(),
        request.record_id(),
        *request.review_package_anchor_id().as_bytes(),
        &request_identity_dependencies,
        &[
            existing_request.freeze_authority_ref(),
            existing_request.policy_authority_ref(),
        ],
    );
    let request_entry_hash: [u8; 32] = Sha256::digest(&request_entry).into();
    let request_reference = JournalReference::new(
        operation_start.registry_id(),
        JournalEntryIndex::try_from(u64::from(request_index)).unwrap(),
        JournalEntryHash::try_from(request_entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(300_u64).unwrap(),
        EventRecordId::try_from(request.record_id().as_bytes().as_slice()).unwrap(),
    );
    let result_bytes = independently_construct_version_1_review_result_with_freeze_scope_and_start(
        &request_reference,
        existing_request.freeze_authority_ref(),
        existing_request.review_scope_ref(),
        *existing_request.review_package_anchor_id().as_bytes(),
        operation_start,
        existing_request.manifest_id(),
    );
    let result = ReviewResultRecord::decode_authoritative(&result_bytes).unwrap();
    let result_index = request_index.checked_add(1).unwrap();
    let result_entry = review_result_recorded_entry(
        result_index,
        JournalEntryHash::try_from(request_entry_hash.as_slice()).unwrap(),
        result.record_id(),
        Some(*result.review_package_anchor_id().as_bytes()),
        Some(&request_reference),
    );
    let result_entry_hash: [u8; 32] = Sha256::digest(&result_entry).into();
    let result_reference = JournalReference::new(
        operation_start.registry_id(),
        JournalEntryIndex::try_from(u64::from(result_index)).unwrap(),
        JournalEntryHash::try_from(result_entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(301_u64).unwrap(),
        EventRecordId::try_from(result.record_id().as_bytes().as_slice()).unwrap(),
    );
    FutureReviewAdmissionPair {
        request_entry,
        request_reference,
        request_bytes,
        result_entry,
        result_reference,
        result_bytes,
    }
}

fn unsupported_review_admission_scope_profile() -> Vec<u8> {
    let mut bytes = independently_construct_exact_review_admission_scope();
    let profile = bytes
        .windows(6)
        .position(|window| window == [0x10, 0x01, 0x11, 0x01, 0x12, 0x40])
        .unwrap();
    bytes[profile + 1] = 2;
    bytes
}

fn fixture_policy_gate_scope_ref(fixture: &AuthoritativeReviewAdmissionFixture) -> RecordId {
    let request = ReviewRequestRecord::decode_authoritative(&fixture.request_bytes).unwrap();
    let policy_id = RecordId::try_from(
        request
            .policy_authority_ref()
            .event_record_id()
            .as_bytes()
            .as_slice(),
    )
    .unwrap();
    ReviewAdmissionPolicyRecord::decode_authoritative(fixture.resolver.resolve(policy_id).unwrap())
        .unwrap()
        .gate_scope_ref()
}

fn authoritative_review_admission_fixture_with_test_overrides(
    overrides: ReviewAdmissionFixtureOverrides,
) -> AuthoritativeReviewAdmissionFixture {
    authoritative_review_admission_fixture_with_overrides(
        false,
        &[7],
        IdentityDependencyMode::Exact,
        AuthorityDependencyMode::Exact,
        AuthorityDependencyMode::Exact,
        overrides,
    )
}

fn assert_complete_review_admission_authority_error(
    mut fixture: AuthoritativeReviewAdmissionFixture,
    expected: ReviewAdmissionSection82AuthorityError,
) {
    let retained_head = fixture.journal.current_head_reference();
    let accepted = fixture
        .journal
        .accept_review_admission(
            fixture.request_reference.clone(),
            &fixture.request_bytes,
            fixture.result_reference.clone(),
            &fixture.result_bytes,
        )
        .unwrap();

    assert_eq!(
        fixture
            .journal
            .complete_review_admission(accepted, &fixture.resolver)
            .unwrap(),
        ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(expected),
            ),
        )
    );
    assert_eq!(fixture.journal.current_head_reference(), retained_head);
}

#[test]
fn retained_journal_resolves_a_version_1_anchor_only_from_its_retained_prefixes() {
    let genesis = genesis();
    let expected = JournalAnchor::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(0_u64).unwrap(),
        genesis.entry_hash(),
        1,
    )
    .unwrap();
    let journal = RetainedJournal::from_genesis(genesis).unwrap();

    let resolved = journal
        .resolve_journal_anchor_id(expected.anchor_id())
        .unwrap();

    assert_eq!(resolved.authoritative_cbor(), expected.authoritative_cbor());
}

#[test]
fn retained_journal_exposes_the_exact_current_head_for_accept_and_snapshot() {
    let genesis = genesis();
    let expected = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(0_u64).unwrap(),
        genesis.entry_hash(),
        evidence_registry::EventTypeId::try_from(1_u64).unwrap(),
        EventRecordId::try_from(genesis_record().record_id().as_bytes().as_slice()).unwrap(),
    );
    let journal = RetainedJournal::from_genesis(genesis).unwrap();

    assert_eq!(journal.current_head_reference(), expected);
}

#[test]
fn policy_registry_binds_the_frozen_evaluator_1015_identity() {
    assert!(policy_evaluator_registry().iter().any(|registration| {
        registration.id == 1015
            && registration.name == "POLICY_REVIEW_ADMISSION_GATE_SCOPE_EXACT_BINDING"
    }));
}

#[test]
fn retained_anchor_history_comparator_classifies_every_section_116_relation() {
    let genesis = genesis();
    let genesis_anchor = JournalAnchor::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(0_u64).unwrap(),
        genesis.entry_hash(),
        1,
    )
    .unwrap();
    let request_bytes = independently_construct_version_1_review_request();
    let request = ReviewRequestRecord::decode_authoritative(&request_bytes).unwrap();
    let entry = review_request_recorded_entry(
        genesis.entry_hash(),
        request.record_id(),
        Some(*request.review_package_anchor_id().as_bytes()),
    );
    let entry_hash: [u8; 32] = Sha256::digest(&entry).into();
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&entry).unwrap();
    let current_anchor = JournalAnchor::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(entry_hash.as_slice()).unwrap(),
        1,
    )
    .unwrap();
    let diverged_anchor = JournalAnchor::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(0_u64).unwrap(),
        JournalEntryHash::try_from(id(0xd0).as_slice()).unwrap(),
        1,
    )
    .unwrap();
    let behind_anchor = JournalAnchor::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(2_u64).unwrap(),
        JournalEntryHash::try_from(id(0xd0).as_slice()).unwrap(),
        1,
    )
    .unwrap();
    let foreign_anchor = JournalAnchor::new(
        RegistryId::try_from(id(0x01).as_slice()).unwrap(),
        JournalEntryIndex::try_from(0_u64).unwrap(),
        JournalEntryHash::try_from(entry_hash.as_slice()).unwrap(),
        1,
    )
    .unwrap();
    let invalid_anchor = JournalAnchor::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(0_u64).unwrap(),
        JournalEntryHash::try_from(entry_hash.as_slice()).unwrap(),
        2,
    )
    .unwrap();

    assert_eq!(
        compare_retained_journal_anchor_history(&journal, &current_anchor),
        JournalAnchorHistoryComparison::AnchorEqualsCurrentHead
    );
    assert_eq!(
        compare_retained_journal_anchor_history(&journal, &genesis_anchor),
        JournalAnchorHistoryComparison::AnchorIsValidAncestor
    );
    assert_eq!(
        compare_retained_journal_anchor_history(&journal, &diverged_anchor),
        JournalAnchorHistoryComparison::JournalDivergence
    );
    assert_eq!(
        compare_retained_journal_anchor_history(&journal, &behind_anchor),
        JournalAnchorHistoryComparison::JournalHistoryBehindAnchor
    );
    assert_eq!(
        compare_retained_journal_anchor_history(&journal, &foreign_anchor),
        JournalAnchorHistoryComparison::AnchorFromDifferentRegistry
    );
    assert_eq!(
        compare_retained_journal_anchor_history(&journal, &invalid_anchor),
        JournalAnchorHistoryComparison::AnchorInvalid
    );
}

#[test]
fn returned_anchor_router_is_preterminal_on_failure_and_neutral_on_handoff() {
    for comparison in [
        JournalAnchorHistoryComparison::JournalDivergence,
        JournalAnchorHistoryComparison::JournalHistoryBehindAnchor,
        JournalAnchorHistoryComparison::AnchorFromDifferentRegistry,
        JournalAnchorHistoryComparison::AnchorInvalid,
    ] {
        let handoff_calls = std::cell::Cell::new(0_u8);
        let outcome = route_review_admission_after_anchor_comparison(comparison, || {
            handoff_calls.set(handoff_calls.get() + 1)
        });
        assert_eq!(
            outcome,
            ReviewAdmissionSection82RoutingOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::ReturnedAnchorComparison(comparison)
            )
        );
        assert_eq!(handoff_calls.get(), 0);
    }

    for comparison in [
        JournalAnchorHistoryComparison::AnchorEqualsCurrentHead,
        JournalAnchorHistoryComparison::AnchorIsValidAncestor,
    ] {
        let handoff_calls = std::cell::Cell::new(0_u8);
        let outcome = route_review_admission_after_anchor_comparison(comparison, || {
            handoff_calls.set(handoff_calls.get() + 1)
        });
        assert_eq!(
            outcome,
            ReviewAdmissionSection82RoutingOutcome::PolicyRouteEligible
        );
        assert_eq!(handoff_calls.get(), 1);
    }
}

#[test]
fn version_1_review_request_strictly_decodes_its_anchor_transport_fields() {
    let bytes = independently_construct_version_1_review_request();
    assert!(
        StrictRecordFrame::decode_authoritative(&bytes).is_ok(),
        "{bytes:02x?}"
    );
    let record = ReviewRequestRecord::decode_authoritative(&bytes).unwrap();

    assert_eq!(record.review_package_anchor_binding_version(), 1);
    assert_eq!(record.review_package_anchor_id().as_bytes(), &id(0x10));
}

#[test]
fn version_1_review_result_strictly_decodes_the_exact_anchor_transport_fields() {
    let bytes = independently_construct_version_1_review_result();
    let record = ReviewResultRecord::decode_authoritative(&bytes).unwrap();

    assert_eq!(record.review_package_anchor_binding_version(), 1);
    assert_eq!(record.review_package_anchor_id().as_bytes(), &id(0x10));
}

#[test]
fn version_1_review_records_expose_exact_section_82_comparison_fields() {
    let request = ReviewRequestRecord::decode_authoritative(
        &independently_construct_version_1_review_request(),
    )
    .unwrap();
    let result = ReviewResultRecord::decode_authoritative(
        &independently_construct_version_1_review_result(),
    )
    .unwrap();

    assert_eq!(request.freeze_authority_ref().event_type_id().value(), 101);
    assert_eq!(request.manifest_id().as_bytes(), &id(0x80));
    assert_eq!(request.review_role_id(), 7);
    assert_eq!(request.required_checks_ref().as_bytes(), &id(0xa0));
    assert_eq!(request.policy_authority_ref().event_type_id().value(), 400);
    assert_eq!(request.review_scope_ref().as_bytes(), &id(0xc0));
    assert_eq!(request.review_method_ref().as_bytes(), &id(0xe0));
    assert_eq!(result.freeze_authority_ref().event_type_id().value(), 101);
    assert_eq!(result.manifest_id().as_bytes(), &id(0x80));
    assert_eq!(result.review_role_id(), 7);
    assert_eq!(result.review_scope_ref().as_bytes(), &id(0xc0));
    assert_eq!(result.review_method_ref().as_bytes(), &id(0xe0));
    assert_eq!(result.method_status(), 1);
    assert_eq!(result.finding_state(), 1);
}

#[test]
fn section_82_common_request_result_fields_require_exact_identity_equality() {
    let request_bytes = independently_construct_version_1_review_request();
    let request = ReviewRequestRecord::decode_authoritative(&request_bytes).unwrap();
    let result_bytes = independently_construct_version_1_review_result_with_section_82_fields(
        reference(1, 300, 0x20),
        request.freeze_authority_ref().clone(),
        id(0x10),
    );

    assert_eq!(
        validate_review_admission_common_request_result_fields(&request_bytes, &result_bytes)
            .unwrap()
            .as_bytes(),
        &id(0xc0),
    );

    let mut scope_mismatched_result = result_bytes;
    let scope_offset = scope_mismatched_result
        .windows(4)
        .position(|window| window == [0x14, 0x58, 0x20, 0xc0])
        .unwrap();
    scope_mismatched_result[scope_offset + 3] = 0xc1;
    assert_eq!(
        validate_review_admission_common_request_result_fields(
            &request_bytes,
            &scope_mismatched_result,
        ),
        Err(ReviewAdmissionCommonRequestResultError::ReviewScopeMismatch),
    );
}

#[test]
fn version_1_review_request_requires_its_exact_retained_recorded_event() {
    let genesis = genesis();
    let request_bytes = independently_construct_version_1_review_request();
    let request = ReviewRequestRecord::decode_authoritative(&request_bytes).unwrap();
    let entry = review_request_recorded_entry(
        genesis.entry_hash(),
        request.record_id(),
        Some(*request.review_package_anchor_id().as_bytes()),
    );
    let entry_hash: [u8; 32] = Sha256::digest(&entry).into();
    let recorded_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(300_u64).unwrap(),
        EventRecordId::try_from(request.record_id().as_bytes().as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&entry).unwrap();

    let binding =
        validate_review_request_recorded_binding(&journal, &recorded_reference, &request_bytes)
            .unwrap();

    assert_eq!(binding.record_id(), request.record_id());
}

#[test]
fn version_1_review_request_recorded_event_requires_its_matching_anchor_identity_commitment() {
    let genesis = genesis();
    let request_bytes = independently_construct_version_1_review_request();
    let request = ReviewRequestRecord::decode_authoritative(&request_bytes).unwrap();
    // This retained event deliberately has no identity dependencies. A v1
    // REVIEW_REQUEST_RECORDED event must not bind without its exact typed
    // JOURNAL_ANCHOR_ID commitment.
    let entry = review_request_recorded_entry(genesis.entry_hash(), request.record_id(), None);
    let entry_hash: [u8; 32] = Sha256::digest(&entry).into();
    let recorded_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(300_u64).unwrap(),
        EventRecordId::try_from(request.record_id().as_bytes().as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&entry).unwrap();

    assert_eq!(
        validate_review_request_recorded_binding(&journal, &recorded_reference, &request_bytes),
        Err(ReviewRequestRecordedBindingError::RequestEventAnchorIdentityDependencyMismatch),
        "a v1 REVIEW_REQUEST_RECORDED event without JOURNAL_ANCHOR_ID must fail closed",
    );
}

#[test]
fn version_1_review_request_recorded_event_rejects_substituted_or_wrong_kind_anchor_commitments() {
    let genesis = genesis();
    let request_bytes = independently_construct_version_1_review_request();
    let request = ReviewRequestRecord::decode_authoritative(&request_bytes).unwrap();

    for (identity_anchor, wrong_kind) in [
        (id(0x11), false),
        (*request.review_package_anchor_id().as_bytes(), true),
    ] {
        let mut entry = review_request_recorded_entry(
            genesis.entry_hash(),
            request.record_id(),
            Some(identity_anchor),
        );
        if wrong_kind {
            let kind_offset = entry
                .windows(5)
                .position(|window| window == [0x81, 0x82, 0x02, 0x58, 0x20])
                .unwrap()
                + 2;
            entry[kind_offset] = 0x01;
        }
        let entry_hash: [u8; 32] = Sha256::digest(&entry).into();
        let recorded_reference = JournalReference::new(
            RegistryId::try_from(id(0x00).as_slice()).unwrap(),
            JournalEntryIndex::try_from(1_u64).unwrap(),
            JournalEntryHash::try_from(entry_hash.as_slice()).unwrap(),
            evidence_registry::EventTypeId::try_from(300_u64).unwrap(),
            EventRecordId::try_from(request.record_id().as_bytes().as_slice()).unwrap(),
        );
        let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
        journal.append_strict_entry(&entry).unwrap();

        assert_eq!(
            validate_review_request_recorded_binding(&journal, &recorded_reference, &request_bytes),
            Err(ReviewRequestRecordedBindingError::RequestEventAnchorIdentityDependencyMismatch)
        );
    }
}

#[test]
fn version_1_review_result_requires_its_exact_retained_recorded_event() {
    let genesis = genesis();
    let result_bytes = independently_construct_version_1_review_result();
    let result = ReviewResultRecord::decode_authoritative(&result_bytes).unwrap();
    let entry = review_result_recorded_entry(
        1,
        genesis.entry_hash(),
        result.record_id(),
        Some(*result.review_package_anchor_id().as_bytes()),
        None,
    );
    let entry_hash: [u8; 32] = Sha256::digest(&entry).into();
    let recorded_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(301_u64).unwrap(),
        EventRecordId::try_from(result.record_id().as_bytes().as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&entry).unwrap();

    let binding =
        validate_review_result_recorded_binding(&journal, &recorded_reference, &result_bytes)
            .unwrap();

    assert_eq!(binding.record_id(), result.record_id());
}

#[test]
fn version_1_review_result_recorded_event_requires_its_matching_anchor_identity_commitment() {
    let genesis = genesis();
    let result_bytes = independently_construct_version_1_review_result();
    let result = ReviewResultRecord::decode_authoritative(&result_bytes).unwrap();
    let entry =
        review_result_recorded_entry(1, genesis.entry_hash(), result.record_id(), None, None);
    let entry_hash: [u8; 32] = Sha256::digest(&entry).into();
    let recorded_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(301_u64).unwrap(),
        EventRecordId::try_from(result.record_id().as_bytes().as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&entry).unwrap();

    assert_eq!(
        validate_review_result_recorded_binding(&journal, &recorded_reference, &result_bytes),
        Err(ReviewResultRecordedBindingError::ResultEventAnchorIdentityDependencyMismatch),
    );
}

#[test]
fn version_1_review_result_recorded_event_rejects_a_substituted_anchor_identity_commitment() {
    let genesis = genesis();
    let result_bytes = independently_construct_version_1_review_result();
    let result = ReviewResultRecord::decode_authoritative(&result_bytes).unwrap();
    let entry = review_result_recorded_entry(
        1,
        genesis.entry_hash(),
        result.record_id(),
        Some(id(0x11)),
        None,
    );
    let entry_hash: [u8; 32] = Sha256::digest(&entry).into();
    let recorded_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(301_u64).unwrap(),
        EventRecordId::try_from(result.record_id().as_bytes().as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&entry).unwrap();

    assert_eq!(
        validate_review_result_recorded_binding(&journal, &recorded_reference, &result_bytes),
        Err(ReviewResultRecordedBindingError::ResultEventAnchorIdentityDependencyMismatch),
    );
}

#[test]
fn version_1_review_request_recorded_event_rejects_wrong_kind_or_multiple_anchor_commitments() {
    let genesis = genesis();
    let request_bytes = independently_construct_version_1_review_request();
    let request = ReviewRequestRecord::decode_authoritative(&request_bytes).unwrap();
    let expected_anchor = *request.review_package_anchor_id().as_bytes();
    let assert_mismatch = |identity_dependencies: &[(u8, [u8; 32])]| {
        let entry = review_request_recorded_entry_with_identity_dependencies(
            genesis.entry_hash(),
            request.record_id(),
            identity_dependencies,
        );
        let entry_hash: [u8; 32] = Sha256::digest(&entry).into();
        let recorded_reference = JournalReference::new(
            RegistryId::try_from(id(0x00).as_slice()).unwrap(),
            JournalEntryIndex::try_from(1_u64).unwrap(),
            JournalEntryHash::try_from(entry_hash.as_slice()).unwrap(),
            evidence_registry::EventTypeId::try_from(300_u64).unwrap(),
            EventRecordId::try_from(request.record_id().as_bytes().as_slice()).unwrap(),
        );
        let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
        journal.append_strict_entry(&entry).unwrap();
        assert_eq!(
            validate_review_request_recorded_binding(&journal, &recorded_reference, &request_bytes),
            Err(ReviewRequestRecordedBindingError::RequestEventAnchorIdentityDependencyMismatch)
        );
    };

    assert_mismatch(&[(1, expected_anchor)]);
    assert_mismatch(&[(2, expected_anchor), (2, id(0x11))]);
}

#[test]
fn version_1_review_result_recorded_event_rejects_wrong_kind_or_multiple_anchor_commitments() {
    let genesis = genesis();
    let result_bytes = independently_construct_version_1_review_result();
    let result = ReviewResultRecord::decode_authoritative(&result_bytes).unwrap();
    let expected_anchor = *result.review_package_anchor_id().as_bytes();
    let assert_mismatch = |identity_dependencies: &[(u8, [u8; 32])]| {
        let entry = review_result_recorded_entry_with_identity_dependencies(
            1,
            genesis.entry_hash(),
            result.record_id(),
            identity_dependencies,
            None,
        );
        let entry_hash: [u8; 32] = Sha256::digest(&entry).into();
        let recorded_reference = JournalReference::new(
            RegistryId::try_from(id(0x00).as_slice()).unwrap(),
            JournalEntryIndex::try_from(1_u64).unwrap(),
            JournalEntryHash::try_from(entry_hash.as_slice()).unwrap(),
            evidence_registry::EventTypeId::try_from(301_u64).unwrap(),
            EventRecordId::try_from(result.record_id().as_bytes().as_slice()).unwrap(),
        );
        let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
        journal.append_strict_entry(&entry).unwrap();
        assert_eq!(
            validate_review_result_recorded_binding(&journal, &recorded_reference, &result_bytes),
            Err(ReviewResultRecordedBindingError::ResultEventAnchorIdentityDependencyMismatch)
        );
    };

    assert_mismatch(&[(1, expected_anchor)]);
    assert_mismatch(&[(2, expected_anchor), (2, id(0x11))]);
}

#[test]
fn version_1_review_package_anchor_transport_requires_exact_request_result_equality() {
    let request_bytes = independently_construct_version_1_review_request();
    let result_bytes = independently_construct_version_1_review_result();

    assert!(validate_review_package_anchor_transport(&request_bytes, &result_bytes).is_ok());

    let mut substituted_result = result_bytes;
    let anchor_offset = substituted_result
        .windows(5)
        .position(|window| window == [0x18, 0x1b, 0x58, 0x20, 0x10])
        .unwrap();
    substituted_result[anchor_offset + 4] = 0x11;
    assert_eq!(
        validate_review_package_anchor_transport(&request_bytes, &substituted_result),
        Err(ReviewPackageAnchorTransportError::AnchorCarrierMismatch)
    );
}

#[test]
fn retained_review_package_anchor_input_uses_exact_request_result_transport_and_history() {
    let genesis = genesis();
    let expected_anchor = JournalAnchor::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(0_u64).unwrap(),
        genesis.entry_hash(),
        1,
    )
    .unwrap();
    let request_bytes = independently_construct_version_1_review_request_with_anchor_id(
        *expected_anchor.anchor_id().as_bytes(),
    );
    let request = ReviewRequestRecord::decode_authoritative(&request_bytes).unwrap();
    let request_entry = review_request_recorded_entry(
        genesis.entry_hash(),
        request.record_id(),
        Some(*request.review_package_anchor_id().as_bytes()),
    );
    let request_entry_hash: [u8; 32] = Sha256::digest(&request_entry).into();
    let request_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(request_entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(300_u64).unwrap(),
        EventRecordId::try_from(request.record_id().as_bytes().as_slice()).unwrap(),
    );
    let result_bytes = independently_construct_version_1_review_result_with_transport(
        request_reference.clone(),
        *expected_anchor.anchor_id().as_bytes(),
    );
    let result = ReviewResultRecord::decode_authoritative(&result_bytes).unwrap();
    let result_entry = review_result_recorded_entry(
        2,
        JournalEntryHash::try_from(request_entry_hash.as_slice()).unwrap(),
        result.record_id(),
        Some(*result.review_package_anchor_id().as_bytes()),
        Some(&request_reference),
    );
    let result_entry_hash: [u8; 32] = Sha256::digest(&result_entry).into();
    let result_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(2_u64).unwrap(),
        JournalEntryHash::try_from(result_entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(301_u64).unwrap(),
        EventRecordId::try_from(result.record_id().as_bytes().as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&request_entry).unwrap();
    journal.append_strict_entry(&result_entry).unwrap();

    let resolved = resolve_retained_review_package_anchor_input(
        &journal,
        &request_reference,
        &request_bytes,
        &result_reference,
        &result_bytes,
    )
    .unwrap();

    assert_eq!(
        resolved.authoritative_cbor(),
        expected_anchor.authoritative_cbor()
    );
    assert_eq!(
        compare_retained_journal_anchor_history(&journal, &resolved),
        JournalAnchorHistoryComparison::AnchorIsValidAncestor
    );
}

#[test]
fn retained_review_package_input_failure_is_preterminal_and_does_not_invoke_policy_evaluation() {
    let journal = RetainedJournal::from_genesis(genesis()).unwrap();
    let policy_evaluation_calls = std::cell::Cell::new(0_u8);

    let outcome = route_retained_review_admission_section_82(
        &journal,
        &reference(1, 300, 0x20),
        &[0xff],
        &reference(2, 301, 0x40),
        &[0xff],
        || {
            policy_evaluation_calls.set(policy_evaluation_calls.get() + 1);
            unreachable!("a retained Review Package input failure must not evaluate Policy")
        },
    );

    assert_eq!(
        outcome,
        ReviewAdmissionSection82RoutingOutcome::PreTerminal(
            ReviewAdmissionSection82PrerequisiteFailure::RetainedAnchorInput(
                RetainedReviewPackageAnchorInputError::RequestBinding(
                    ReviewRequestRecordedBindingError::RequestDecode
                )
            )
        )
    );
    assert_eq!(policy_evaluation_calls.get(), 0);
}

#[test]
fn retained_request_anchor_identity_failure_is_preterminal_and_does_not_invoke_policy_evaluation() {
    let genesis = genesis();
    let request_bytes = independently_construct_version_1_review_request();
    let request = ReviewRequestRecord::decode_authoritative(&request_bytes).unwrap();
    let entry = review_request_recorded_entry(genesis.entry_hash(), request.record_id(), None);
    let entry_hash: [u8; 32] = Sha256::digest(&entry).into();
    let request_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(300_u64).unwrap(),
        EventRecordId::try_from(request.record_id().as_bytes().as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&entry).unwrap();
    let policy_evaluation_calls = std::cell::Cell::new(0_u8);

    let outcome = route_retained_review_admission_section_82(
        &journal,
        &request_reference,
        &request_bytes,
        &reference(2, 301, 0x40),
        &[0xff],
        || {
            policy_evaluation_calls.set(policy_evaluation_calls.get() + 1);
            unreachable!("a retained Request anchor-identity failure must not evaluate Policy")
        },
    );

    assert_eq!(
        outcome,
        ReviewAdmissionSection82RoutingOutcome::PreTerminal(
            ReviewAdmissionSection82PrerequisiteFailure::RetainedAnchorInput(
                RetainedReviewPackageAnchorInputError::RequestBinding(
                    ReviewRequestRecordedBindingError::RequestEventAnchorIdentityDependencyMismatch
                )
            )
        )
    );
    assert_eq!(policy_evaluation_calls.get(), 0);
}

#[test]
fn retained_anchor_route_does_not_promote_structural_policy_context_to_authority() {
    let fixture = authoritative_review_admission_fixture(true);
    let request = ReviewRequestRecord::decode_authoritative(&fixture.request_bytes).unwrap();
    let policy_id = RecordId::try_from(
        request
            .policy_authority_ref()
            .event_record_id()
            .as_bytes()
            .as_slice(),
    )
    .unwrap();
    let outcome = route_retained_review_admission_policy_context(
        &fixture.journal,
        &fixture.request_reference,
        &fixture.request_bytes,
        &fixture.result_reference,
        &fixture.result_bytes,
        &fixture.resolver,
    );
    let policy_46_outcome = evaluate_retained_review_admission_policy_46(
        &fixture.journal,
        &fixture.request_reference,
        &fixture.request_bytes,
        &fixture.result_reference,
        &fixture.result_bytes,
        &fixture.resolver,
    );
    let section_83_disposition = derive_review_admission_section_83_disposition(&policy_46_outcome);

    assert!(matches!(
        outcome,
        ReviewAdmissionPolicyContextRouteOutcome::PolicyContextReady(prerequisites)
            if prerequisites.policy_record_id() == policy_id
    ));
    assert_eq!(
        policy_46_outcome,
        ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
            ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                ReviewAdmissionSection82AuthorityError::FreezeAuthorityEvidenceUnavailable,
            ),
        )
    );
    assert_eq!(
        section_83_disposition,
        ReviewAdmissionSection83Disposition::PreTerminal,
    );
}

#[test]
fn missing_policy_gate_scope_is_reported_before_generic_authority_unavailability() {
    let mut fixture = authoritative_review_admission_fixture(true);
    fixture.resolver.0.retain(|(_, bytes)| {
        StrictRecordFrame::decode_authoritative(bytes)
            .map(|frame| frame.record_type_id().value() != 20)
            .unwrap_or(true)
    });
    let accepted = fixture
        .journal
        .accept_review_admission(
            fixture.request_reference.clone(),
            &fixture.request_bytes,
            fixture.result_reference.clone(),
            &fixture.result_bytes,
        )
        .unwrap();

    let outcome = fixture
        .journal
        .complete_review_admission(accepted, &fixture.resolver)
        .unwrap();

    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                    ReviewAdmissionSection82AuthorityError::PolicyContextPrerequisites(
                        ReviewAdmissionPolicyContextPrerequisitesError::GateScope(
                            ReviewAdmissionExactScopeProfileError::ScopePayloadUnavailable
                        )
                    )
                )
            )
        )
    ));
}

#[test]
fn policy_registration_chronology_failure_is_integrated_before_generic_authority_unavailability() {
    let fixture = authoritative_review_admission_fixture_with_test_overrides(
        ReviewAdmissionFixtureOverrides {
            policy_operation_start: Some(reference(9, 300, 0x91)),
            ..Default::default()
        },
    );

    assert_complete_review_admission_authority_error(
        fixture,
        ReviewAdmissionSection82AuthorityError::PolicyContextPrerequisites(
            ReviewAdmissionPolicyContextPrerequisitesError::PolicyBinding(
                ReviewAdmissionPolicyRecordedBindingError::OperationStartReference(
                    RetainedJournalError::MissingReference,
                ),
            ),
        ),
    );
}

#[test]
fn unsupported_policy_context_is_integrated_before_generic_authority_unavailability() {
    let fixture = authoritative_review_admission_fixture_with_test_overrides(
        ReviewAdmissionFixtureOverrides {
            policy_context_id: Some(1),
            ..Default::default()
        },
    );

    assert_complete_review_admission_authority_error(
        fixture,
        ReviewAdmissionSection82AuthorityError::PolicyContextPrerequisites(
            ReviewAdmissionPolicyContextPrerequisitesError::PolicyContextUnsupported,
        ),
    );
}

#[test]
fn every_gate_scope_failure_is_integrated_before_generic_authority_unavailability() {
    let mut unavailable = authoritative_review_admission_fixture_with_test_overrides(
        ReviewAdmissionFixtureOverrides::default(),
    );
    let unavailable_gate_scope = fixture_policy_gate_scope_ref(&unavailable);
    unavailable
        .resolver
        .0
        .retain(|(record_id, _)| *record_id != unavailable_gate_scope);
    assert_complete_review_admission_authority_error(
        unavailable,
        ReviewAdmissionSection82AuthorityError::PolicyContextPrerequisites(
            ReviewAdmissionPolicyContextPrerequisitesError::GateScope(
                ReviewAdmissionExactScopeProfileError::ScopePayloadUnavailable,
            ),
        ),
    );

    let malformed = authoritative_review_admission_fixture_with_test_overrides(
        ReviewAdmissionFixtureOverrides {
            gate_scope_bytes: Some(vec![0xff]),
            ..Default::default()
        },
    );
    assert_complete_review_admission_authority_error(
        malformed,
        ReviewAdmissionSection82AuthorityError::PolicyContextPrerequisites(
            ReviewAdmissionPolicyContextPrerequisitesError::GateScope(
                ReviewAdmissionExactScopeProfileError::ScopeDecode,
            ),
        ),
    );

    let mut mismatched = authoritative_review_admission_fixture_with_test_overrides(
        ReviewAdmissionFixtureOverrides::default(),
    );
    let mismatched_gate_scope = fixture_policy_gate_scope_ref(&mismatched);
    mismatched
        .resolver
        .0
        .iter_mut()
        .find(|(record_id, _)| *record_id == mismatched_gate_scope)
        .unwrap()
        .1 = independently_construct_labeled_exact_review_admission_scope("substituted");
    assert_complete_review_admission_authority_error(
        mismatched,
        ReviewAdmissionSection82AuthorityError::PolicyContextPrerequisites(
            ReviewAdmissionPolicyContextPrerequisitesError::GateScope(
                ReviewAdmissionExactScopeProfileError::ScopeIdentityMismatch,
            ),
        ),
    );

    let unsupported = authoritative_review_admission_fixture_with_test_overrides(
        ReviewAdmissionFixtureOverrides {
            gate_scope_bytes: Some(unsupported_review_admission_scope_profile()),
            ..Default::default()
        },
    );
    assert_complete_review_admission_authority_error(
        unsupported,
        ReviewAdmissionSection82AuthorityError::PolicyContextPrerequisites(
            ReviewAdmissionPolicyContextPrerequisitesError::GateScope(
                ReviewAdmissionExactScopeProfileError::UnsupportedProfile,
            ),
        ),
    );
}

#[test]
fn every_common_review_scope_failure_is_integrated_before_generic_authority_unavailability() {
    let mut unavailable = authoritative_review_admission_fixture_with_test_overrides(
        ReviewAdmissionFixtureOverrides::default(),
    );
    let unavailable_common_scope =
        ReviewRequestRecord::decode_authoritative(&unavailable.request_bytes)
            .unwrap()
            .review_scope_ref();
    unavailable
        .resolver
        .0
        .retain(|(record_id, _)| *record_id != unavailable_common_scope);
    assert_complete_review_admission_authority_error(
        unavailable,
        ReviewAdmissionSection82AuthorityError::PolicyContextPrerequisites(
            ReviewAdmissionPolicyContextPrerequisitesError::CommonReviewScope(
                ReviewAdmissionExactScopeProfileError::ScopePayloadUnavailable,
            ),
        ),
    );

    let malformed = authoritative_review_admission_fixture_with_test_overrides(
        ReviewAdmissionFixtureOverrides {
            common_scope_bytes: Some(vec![0xff]),
            ..Default::default()
        },
    );
    assert_complete_review_admission_authority_error(
        malformed,
        ReviewAdmissionSection82AuthorityError::PolicyContextPrerequisites(
            ReviewAdmissionPolicyContextPrerequisitesError::CommonReviewScope(
                ReviewAdmissionExactScopeProfileError::ScopeDecode,
            ),
        ),
    );

    let mut mismatched = authoritative_review_admission_fixture_with_test_overrides(
        ReviewAdmissionFixtureOverrides::default(),
    );
    let mismatched_common_scope =
        ReviewRequestRecord::decode_authoritative(&mismatched.request_bytes)
            .unwrap()
            .review_scope_ref();
    mismatched
        .resolver
        .0
        .iter_mut()
        .find(|(record_id, _)| *record_id == mismatched_common_scope)
        .unwrap()
        .1 = independently_construct_labeled_exact_review_admission_scope("substituted");
    assert_complete_review_admission_authority_error(
        mismatched,
        ReviewAdmissionSection82AuthorityError::PolicyContextPrerequisites(
            ReviewAdmissionPolicyContextPrerequisitesError::CommonReviewScope(
                ReviewAdmissionExactScopeProfileError::ScopeIdentityMismatch,
            ),
        ),
    );

    let unsupported = authoritative_review_admission_fixture_with_test_overrides(
        ReviewAdmissionFixtureOverrides {
            common_scope_bytes: Some(unsupported_review_admission_scope_profile()),
            ..Default::default()
        },
    );
    assert_complete_review_admission_authority_error(
        unsupported,
        ReviewAdmissionSection82AuthorityError::PolicyContextPrerequisites(
            ReviewAdmissionPolicyContextPrerequisitesError::CommonReviewScope(
                ReviewAdmissionExactScopeProfileError::UnsupportedProfile,
            ),
        ),
    );
}

#[test]
fn request_operation_start_resolution_precedes_policy_context_and_generic_authority_failures() {
    let fixture = authoritative_review_admission_fixture_with_test_overrides(
        ReviewAdmissionFixtureOverrides {
            request_operation_start: Some(reference(9, 300, 0x92)),
            ..Default::default()
        },
    );

    assert_complete_review_admission_authority_error(
        fixture,
        ReviewAdmissionSection82AuthorityError::RequestOperationStartReference(
            RetainedJournalError::MissingReference,
        ),
    );
}

#[test]
fn result_operation_start_resolution_precedes_policy_context_and_generic_authority_failures() {
    let fixture = authoritative_review_admission_fixture_with_test_overrides(
        ReviewAdmissionFixtureOverrides {
            result_operation_start: Some(reference(9, 300, 0x93)),
            ..Default::default()
        },
    );

    assert_complete_review_admission_authority_error(
        fixture,
        ReviewAdmissionSection82AuthorityError::ResultOperationStartReference(
            RetainedJournalError::MissingReference,
        ),
    );
}

#[test]
fn unresolved_request_reference_is_not_classified_by_caller_entry_index() {
    let mut journal = RetainedJournal::from_genesis(genesis()).unwrap();
    let operation_start = journal.current_head_reference();
    let accepted = journal
        .accept_review_admission(
            reference(1, 300, 0x20),
            &[0xff],
            reference(2, 301, 0x40),
            &[0xff],
        )
        .unwrap();

    assert_eq!(accepted.operation_start_journal_ref(), &operation_start);
    let outcome = journal
        .complete_review_admission(accepted, &RecordBytes(Vec::new()))
        .unwrap();

    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                    ReviewAdmissionSection82AuthorityError::RetainedAnchorInput(
                        RetainedReviewPackageAnchorInputError::RequestBinding(
                            ReviewRequestRecordedBindingError::RequestDecode
                        )
                    )
                )
            )
        )
    ));
    assert_eq!(journal.current_head_reference(), operation_start);
}

#[test]
fn unresolved_result_reference_does_not_mask_request_decode_by_caller_entry_index() {
    let mut journal = RetainedJournal::from_genesis(genesis()).unwrap();
    let operation_start = journal.current_head_reference();
    let accepted = journal
        .accept_review_admission(
            reference(0, 300, 0x20),
            &[0xff],
            reference(1, 301, 0x40),
            &[0xff],
        )
        .unwrap();

    let outcome = journal
        .complete_review_admission(accepted, &RecordBytes(Vec::new()))
        .unwrap();

    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                    ReviewAdmissionSection82AuthorityError::RetainedAnchorInput(
                        RetainedReviewPackageAnchorInputError::RequestBinding(
                            ReviewRequestRecordedBindingError::RequestDecode
                        )
                    )
                )
            )
        )
    ));
    assert_eq!(journal.current_head_reference(), operation_start);
}

#[test]
fn retained_request_recorded_after_acceptance_is_classified_only_after_structural_resolution() {
    let mut fixture = authoritative_review_admission_fixture(true);
    let operation_start = fixture.journal.current_head_reference();
    let pair = future_review_admission_pair(&fixture, &operation_start);
    let accepted = fixture
        .journal
        .accept_review_admission(
            pair.request_reference.clone(),
            &pair.request_bytes,
            pair.result_reference.clone(),
            &pair.result_bytes,
        )
        .unwrap();
    fixture
        .journal
        .append_strict_entry(&pair.request_entry)
        .unwrap();
    fixture
        .journal
        .append_strict_entry(&pair.result_entry)
        .unwrap();
    let retained_head = fixture.journal.current_head_reference();

    let outcome = fixture
        .journal
        .complete_review_admission(accepted, &fixture.resolver)
        .unwrap();

    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::RequestAuthorityAfterOperationStart
            )
        )
    ));
    assert_eq!(fixture.journal.current_head_reference(), retained_head);
}

#[test]
fn retained_result_recorded_after_acceptance_is_classified_only_after_structural_resolution() {
    let mut fixture = authoritative_review_admission_fixture(true);
    let pair_start = fixture.journal.current_head_reference();
    let pair = future_review_admission_pair(&fixture, &pair_start);
    fixture
        .journal
        .append_strict_entry(&pair.request_entry)
        .unwrap();
    let operation_start = fixture.journal.current_head_reference();
    assert_eq!(operation_start, pair.request_reference);
    let accepted = fixture
        .journal
        .accept_review_admission(
            pair.request_reference.clone(),
            &pair.request_bytes,
            pair.result_reference.clone(),
            &pair.result_bytes,
        )
        .unwrap();
    fixture
        .journal
        .append_strict_entry(&pair.result_entry)
        .unwrap();
    let retained_head = fixture.journal.current_head_reference();

    let outcome = fixture
        .journal
        .complete_review_admission(accepted, &fixture.resolver)
        .unwrap();

    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::ResultAuthorityAfterOperationStart
            )
        )
    ));
    assert_eq!(fixture.journal.current_head_reference(), retained_head);
}

#[test]
fn accept_and_snapshot_precedes_malformed_opaque_input_decode() {
    let mut journal = RetainedJournal::from_genesis(genesis()).unwrap();
    let operation_start = journal.current_head_reference();
    let accepted = journal
        .accept_review_admission(
            reference(0, 300, 0x20),
            &[0xff],
            reference(0, 301, 0x40),
            &[0xff],
        )
        .unwrap();

    assert_eq!(accepted.operation_start_journal_ref(), &operation_start);
    let outcome = journal
        .complete_review_admission(accepted, &RecordBytes(Vec::new()))
        .unwrap();

    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                    ReviewAdmissionSection82AuthorityError::RetainedAnchorInput(
                        RetainedReviewPackageAnchorInputError::RequestBinding(
                            ReviewRequestRecordedBindingError::RequestDecode
                        )
                    )
                )
            )
        )
    ));
    assert_eq!(journal.current_head_reference(), operation_start);
}

#[test]
fn acceptance_rejects_oversized_opaque_inputs_before_owned_copy() {
    let journal = RetainedJournal::from_genesis(genesis()).unwrap();
    let oversized = vec![0_u8; REVIEW_ADMISSION_MAX_OPAQUE_INPUT_BYTES + 1];

    assert!(matches!(
        journal.accept_review_admission(
            reference(1, 300, 0x20),
            &oversized,
            reference(2, 301, 0x40),
            &[],
        ),
        Err(ReviewAdmissionAcceptanceError::InputTooLarge)
    ));
    assert_eq!(journal.current_head_reference().entry_index().value(), 0);
}

#[test]
fn acceptance_bounds_outstanding_owned_opaque_inputs_per_live_journal() {
    let journal = RetainedJournal::from_genesis(genesis()).unwrap();
    let mut accepted = Vec::new();
    for _ in 0..REVIEW_ADMISSION_MAX_OUTSTANDING_ACCEPTANCES {
        accepted.push(
            journal
                .accept_review_admission(reference(1, 300, 0x20), &[], reference(2, 301, 0x40), &[])
                .unwrap(),
        );
    }

    assert!(matches!(
        journal
            .accept_review_admission(reference(1, 300, 0x20), &[], reference(2, 301, 0x40), &[],),
        Err(ReviewAdmissionAcceptanceError::TooManyOutstandingAcceptances)
    ));
    drop(accepted.pop());
    assert!(journal
        .accept_review_admission(reference(1, 300, 0x20), &[], reference(2, 301, 0x40), &[],)
        .is_ok());
}

#[test]
fn structurally_complete_inputs_without_authoritative_store_provenance_remain_preterminal() {
    let mut fixture = authoritative_review_admission_fixture(true);
    let opening_head = fixture.journal.current_head_reference();
    let accepted = fixture
        .journal
        .accept_review_admission(
            fixture.request_reference.clone(),
            &fixture.request_bytes,
            fixture.result_reference.clone(),
            &fixture.result_bytes,
        )
        .unwrap();

    let outcome = fixture
        .journal
        .complete_review_admission(accepted, &fixture.resolver)
        .unwrap();

    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                    ReviewAdmissionSection82AuthorityError::FreezeAuthorityEvidenceUnavailable
                )
            )
        )
    ));
    assert_eq!(fixture.journal.current_head_reference(), opening_head);
}

#[test]
fn authoritative_store_derives_complete_section_82_without_caller_record_authority() {
    let fixture = authoritative_review_admission_fixture(true);
    let store_dir = AuthoritativeReviewStoreFixtureDir::from_fixture(&fixture);
    let mut store = AuthoritativeRegistryStore::open(&store_dir.path).unwrap();
    let accepted = store
        .accept_authoritative_review_admission(
            fixture.request_reference.clone(),
            &fixture.request_bytes,
            fixture.result_reference.clone(),
            &fixture.result_bytes,
        )
        .unwrap();
    assert_eq!(
        accepted.operation_start_journal_ref(),
        &fixture.result_reference
    );

    let section_82 = store
        .complete_authoritative_review_admission_section_82(accepted)
        .unwrap();

    assert_eq!(
        section_82.operation_start_journal_ref(),
        &fixture.result_reference
    );
    assert_eq!(
        section_82.request_event_reference(),
        &fixture.request_reference
    );
    assert_eq!(
        section_82.result_event_reference(),
        &fixture.result_reference
    );
    assert_eq!(
        section_82.policy_authority_ref(),
        ReviewRequestRecord::decode_authoritative(&fixture.request_bytes)
            .unwrap()
            .policy_authority_ref()
    );
    assert_eq!(
        section_82.common_review_scope_ref(),
        ReviewRequestRecord::decode_authoritative(&fixture.request_bytes)
            .unwrap()
            .review_scope_ref()
    );
    assert_eq!(
        section_82.freeze_authority().committed_event_reference(),
        ReviewRequestRecord::decode_authoritative(&fixture.request_bytes)
            .unwrap()
            .freeze_authority_ref()
    );

    let mut caller_substitution = fixture.request_bytes.clone();
    *caller_substitution.last_mut().unwrap() ^= 1;
    let substituted = store
        .accept_authoritative_review_admission(
            fixture.request_reference.clone(),
            &caller_substitution,
            fixture.result_reference.clone(),
            &fixture.result_bytes,
        )
        .unwrap();
    assert_eq!(
        store.complete_authoritative_review_admission_section_82(substituted),
        Err(
            evidence_registry::AuthoritativeReviewAdmissionSection82Error::PresentedRequestMismatch
        )
    );
}

#[test]
fn evaluator_1001_is_not_invoked_without_authoritative_source_provenance() {
    let mut fixture = authoritative_review_admission_fixture_with_roles(true, &[6, 7]);
    let opening_head = fixture.journal.current_head_reference();
    let accepted = fixture
        .journal
        .accept_review_admission(
            fixture.request_reference.clone(),
            &fixture.request_bytes,
            fixture.result_reference.clone(),
            &fixture.result_bytes,
        )
        .unwrap();
    let outcome = fixture
        .journal
        .complete_review_admission(accepted, &fixture.resolver)
        .unwrap();
    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                    ReviewAdmissionSection82AuthorityError::FreezeAuthorityEvidenceUnavailable
                )
            )
        )
    ));
    assert_eq!(fixture.journal.current_head_reference(), opening_head);
}

#[test]
fn accepted_instance_retains_opaque_input_bytes_against_caller_mutation() {
    let mut fixture = authoritative_review_admission_fixture(true);
    let mut caller_request_bytes = fixture.request_bytes.clone();
    let mut caller_result_bytes = fixture.result_bytes.clone();
    let accepted = fixture
        .journal
        .accept_review_admission(
            fixture.request_reference.clone(),
            &caller_request_bytes,
            fixture.result_reference.clone(),
            &caller_result_bytes,
        )
        .unwrap();
    caller_request_bytes.fill(0xff);
    caller_result_bytes.fill(0xff);

    let outcome = fixture
        .journal
        .complete_review_admission(accepted, &fixture.resolver)
        .unwrap();

    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                    ReviewAdmissionSection82AuthorityError::FreezeAuthorityEvidenceUnavailable
                )
            )
        )
    ));
}

#[test]
fn intervening_appends_do_not_refresh_operation_start_before_preterminal_stop() {
    let mut fixture = authoritative_review_admission_fixture(true);
    let operation_start = fixture.journal.current_head_reference();
    let accepted = fixture
        .journal
        .accept_review_admission(
            fixture.request_reference.clone(),
            &fixture.request_bytes,
            fixture.result_reference.clone(),
            &fixture.result_bytes,
        )
        .unwrap();
    let intervening_entry = review_request_recorded_entry_at(
        6,
        operation_start.entry_hash(),
        RecordId::try_from(id(0x30).as_slice()).unwrap(),
        id(0x50),
    );
    fixture
        .journal
        .append_strict_entry(&intervening_entry)
        .unwrap();
    let first_intervening_head = fixture.journal.current_head_reference();
    let second_intervening_entry = review_request_recorded_entry_at(
        7,
        first_intervening_head.entry_hash(),
        RecordId::try_from(id(0x31).as_slice()).unwrap(),
        id(0x51),
    );
    fixture
        .journal
        .append_strict_entry(&second_intervening_entry)
        .unwrap();
    let intervening_head = fixture.journal.current_head_reference();

    let outcome = fixture
        .journal
        .complete_review_admission(accepted, &fixture.resolver)
        .unwrap();
    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                    ReviewAdmissionSection82AuthorityError::FreezeAuthorityEvidenceUnavailable
                )
            )
        )
    ));
    assert!(operation_start.entry_index().value() < first_intervening_head.entry_index().value());
    assert!(first_intervening_head.entry_index().value() < intervening_head.entry_index().value());
    assert_eq!(fixture.journal.current_head_reference(), intervening_head);
}

#[test]
fn evaluator_1015_failure_is_not_invoked_without_authoritative_source_provenance() {
    let mut fixture = authoritative_review_admission_fixture(false);
    let opening_head = fixture.journal.current_head_reference();
    let accepted = fixture
        .journal
        .accept_review_admission(
            fixture.request_reference.clone(),
            &fixture.request_bytes,
            fixture.result_reference.clone(),
            &fixture.result_bytes,
        )
        .unwrap();
    let outcome = fixture
        .journal
        .complete_review_admission(accepted, &fixture.resolver)
        .unwrap();
    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                    ReviewAdmissionSection82AuthorityError::FreezeAuthorityEvidenceUnavailable
                )
            )
        )
    ));
    assert_eq!(fixture.journal.current_head_reference(), opening_head);
}

#[test]
fn request_and_result_require_exact_identity_and_authority_dependency_sets() {
    for (identity_mode, request_mode, result_mode, expected_error) in [
        (
            IdentityDependencyMode::Missing,
            AuthorityDependencyMode::Exact,
            AuthorityDependencyMode::Exact,
            ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                ReviewAdmissionSection82AuthorityError::RequestIdentityDependencySetMismatch,
            ),
        ),
        (
            IdentityDependencyMode::Extra,
            AuthorityDependencyMode::Exact,
            AuthorityDependencyMode::Exact,
            ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                ReviewAdmissionSection82AuthorityError::RequestIdentityDependencySetMismatch,
            ),
        ),
        (
            IdentityDependencyMode::Exact,
            AuthorityDependencyMode::Missing,
            AuthorityDependencyMode::Exact,
            ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                ReviewAdmissionSection82AuthorityError::RequestAuthorityDependencySetMismatch,
            ),
        ),
        (
            IdentityDependencyMode::Exact,
            AuthorityDependencyMode::Extra,
            AuthorityDependencyMode::Exact,
            ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                ReviewAdmissionSection82AuthorityError::RequestAuthorityDependencySetMismatch,
            ),
        ),
        (
            IdentityDependencyMode::Exact,
            AuthorityDependencyMode::Exact,
            AuthorityDependencyMode::Missing,
            ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                ReviewAdmissionSection82AuthorityError::RetainedAnchorInput(
                    RetainedReviewPackageAnchorInputError::ResultEventRequestAuthorityDependencyMissing,
                ),
            ),
        ),
        (
            IdentityDependencyMode::Exact,
            AuthorityDependencyMode::Exact,
            AuthorityDependencyMode::Extra,
            ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                ReviewAdmissionSection82AuthorityError::ResultAuthorityDependencySetMismatch,
            ),
        ),
    ] {
        let mut fixture =
            authoritative_review_admission_fixture_with_roles_and_authority_dependencies(
                true,
                &[7],
                identity_mode,
                request_mode,
                result_mode,
            );
        let opening_head = fixture.journal.current_head_reference();
        let accepted = fixture
            .journal
            .accept_review_admission(
                fixture.request_reference.clone(),
                &fixture.request_bytes,
                fixture.result_reference.clone(),
                &fixture.result_bytes,
            )
            .unwrap();

        let outcome = fixture
            .journal
            .complete_review_admission(accepted, &fixture.resolver)
            .unwrap();
        let ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(actual_error),
        ) = outcome
        else {
            panic!("an inexact direct authority set must remain preterminal")
        };

        assert_eq!(actual_error, expected_error);
        assert_eq!(fixture.journal.current_head_reference(), opening_head);
    }
}

#[test]
fn missing_freeze_manifest_authority_is_preterminal_and_publishes_nothing() {
    let mut fixture = authoritative_review_admission_fixture(true);
    let request = ReviewRequestRecord::decode_authoritative(&fixture.request_bytes).unwrap();
    fixture
        .resolver
        .0
        .retain(|(record_id, _)| *record_id != request.manifest_id());
    let opening_head = fixture.journal.current_head_reference();
    let accepted = fixture
        .journal
        .accept_review_admission(
            fixture.request_reference.clone(),
            &fixture.request_bytes,
            fixture.result_reference.clone(),
            &fixture.result_bytes,
        )
        .unwrap();
    let outcome = fixture
        .journal
        .complete_review_admission(accepted, &fixture.resolver)
        .unwrap();

    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(_)
    ));
    assert_eq!(fixture.journal.current_head_reference(), opening_head);
}

#[test]
fn missing_freeze_receipt_authority_is_preterminal_and_publishes_nothing() {
    let mut fixture = authoritative_review_admission_fixture(true);
    let request = ReviewRequestRecord::decode_authoritative(&fixture.request_bytes).unwrap();
    let receipt_id = RecordId::try_from(
        request
            .freeze_authority_ref()
            .event_record_id()
            .as_bytes()
            .as_slice(),
    )
    .unwrap();
    fixture
        .resolver
        .0
        .retain(|(record_id, _)| *record_id != receipt_id);
    let opening_head = fixture.journal.current_head_reference();
    let accepted = fixture
        .journal
        .accept_review_admission(
            fixture.request_reference.clone(),
            &fixture.request_bytes,
            fixture.result_reference.clone(),
            &fixture.result_bytes,
        )
        .unwrap();
    let outcome = fixture
        .journal
        .complete_review_admission(accepted, &fixture.resolver)
        .unwrap();

    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(_)
    ));
    assert_eq!(fixture.journal.current_head_reference(), opening_head);
}

#[test]
fn missing_exact_policy_authority_is_preterminal_and_publishes_nothing() {
    let mut fixture = authoritative_review_admission_fixture(true);
    let request = ReviewRequestRecord::decode_authoritative(&fixture.request_bytes).unwrap();
    let policy_id = RecordId::try_from(
        request
            .policy_authority_ref()
            .event_record_id()
            .as_bytes()
            .as_slice(),
    )
    .unwrap();
    fixture
        .resolver
        .0
        .retain(|(record_id, _)| *record_id != policy_id);
    let opening_head = fixture.journal.current_head_reference();
    let accepted = fixture
        .journal
        .accept_review_admission(
            fixture.request_reference.clone(),
            &fixture.request_bytes,
            fixture.result_reference.clone(),
            &fixture.result_bytes,
        )
        .unwrap();
    let outcome = fixture
        .journal
        .complete_review_admission(accepted, &fixture.resolver)
        .unwrap();

    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                    ReviewAdmissionSection82AuthorityError::PolicyPayloadUnavailable
                )
            )
        )
    ));
    assert_eq!(fixture.journal.current_head_reference(), opening_head);
}

#[test]
fn append_ordered_before_acceptance_is_reflected_in_operation_start() {
    let mut fixture = authoritative_review_admission_fixture(true);
    let before_acceptance_entry = review_request_recorded_entry_at(
        6,
        fixture.journal.current_head_reference().entry_hash(),
        RecordId::try_from(id(0x30).as_slice()).unwrap(),
        id(0x50),
    );
    fixture
        .journal
        .append_strict_entry(&before_acceptance_entry)
        .unwrap();
    let expected_operation_start = fixture.journal.current_head_reference();
    let accepted = fixture
        .journal
        .accept_review_admission(
            fixture.request_reference.clone(),
            &fixture.request_bytes,
            fixture.result_reference.clone(),
            &fixture.result_bytes,
        )
        .unwrap();

    assert_eq!(
        accepted.operation_start_journal_ref(),
        &expected_operation_start
    );
    let outcome = fixture
        .journal
        .complete_review_admission(accepted, &fixture.resolver)
        .unwrap();
    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(
            ReviewAdmissionPolicy46RouteOutcome::PreTerminal(
                ReviewAdmissionSection82PrerequisiteFailure::AuthorityInput(
                    ReviewAdmissionSection82AuthorityError::FreezeAuthorityEvidenceUnavailable
                )
            )
        )
    ));
    assert_eq!(
        fixture.journal.current_head_reference(),
        expected_operation_start
    );
}

#[test]
fn re_presentation_after_preterminal_stop_captures_a_new_operation_start() {
    let mut journal = RetainedJournal::from_genesis(genesis()).unwrap();
    let first_acceptance = journal
        .accept_review_admission(
            reference(1, 300, 0x20),
            &[0xff],
            reference(2, 301, 0x40),
            &[0xff],
        )
        .unwrap();
    let first_operation_start = first_acceptance.operation_start_journal_ref().clone();
    let outcome = journal
        .complete_review_admission(first_acceptance, &RecordBytes(Vec::new()))
        .unwrap();
    assert!(matches!(
        outcome,
        ReviewAdmissionRuntimeOutcome::PreTerminal(_)
    ));

    let intervening_entry = review_request_recorded_entry_at(
        1,
        first_operation_start.entry_hash(),
        RecordId::try_from(id(0x30).as_slice()).unwrap(),
        id(0x50),
    );
    journal.append_strict_entry(&intervening_entry).unwrap();
    let second_acceptance = journal
        .accept_review_admission(
            reference(1, 300, 0x20),
            &[0xff],
            reference(2, 301, 0x40),
            &[0xff],
        )
        .unwrap();

    assert_ne!(
        second_acceptance.operation_start_journal_ref(),
        &first_operation_start
    );
    assert_eq!(
        second_acceptance.operation_start_journal_ref(),
        &journal.current_head_reference()
    );
}

#[test]
fn accepted_operation_start_cannot_cross_unrelated_retained_journals() {
    let source = RetainedJournal::from_genesis(GenesisJournalEntry::new(
        RegistryId::try_from(id(0x01).as_slice()).unwrap(),
        EventRecordId::try_from(id(0x21).as_slice()).unwrap(),
        RecordId::try_from(id(0x41).as_slice()).unwrap(),
        RecordId::try_from(id(0x61).as_slice()).unwrap(),
    ))
    .unwrap();
    let mut target = authoritative_review_admission_fixture(true);
    let target_opening_head = target.journal.current_head_reference();
    let accepted = source
        .accept_review_admission(
            target.request_reference.clone(),
            &target.request_bytes,
            target.result_reference.clone(),
            &target.result_bytes,
        )
        .unwrap();

    let error = target
        .journal
        .complete_review_admission(accepted, &target.resolver)
        .unwrap_err();

    assert_eq!(
        error,
        ReviewAdmissionRuntimeError::AcceptanceJournalMismatch
    );
    assert_eq!(target.journal.current_head_reference(), target_opening_head);
}

#[test]
fn accepted_operation_start_cannot_cross_reconstructed_identical_prefixes() {
    let source = authoritative_review_admission_fixture(true);
    let mut target = authoritative_review_admission_fixture(true);
    assert_eq!(
        source.journal.current_head_reference(),
        target.journal.current_head_reference(),
        "the rejection must be instance-bound rather than a head-value comparison"
    );
    let target_opening_head = target.journal.current_head_reference();
    let accepted = source
        .journal
        .accept_review_admission(
            target.request_reference.clone(),
            &target.request_bytes,
            target.result_reference.clone(),
            &target.result_bytes,
        )
        .unwrap();

    let error = target
        .journal
        .complete_review_admission(accepted, &target.resolver)
        .unwrap_err();

    assert_eq!(
        error,
        ReviewAdmissionRuntimeError::AcceptanceJournalMismatch
    );
    assert_eq!(target.journal.current_head_reference(), target_opening_head);
}
