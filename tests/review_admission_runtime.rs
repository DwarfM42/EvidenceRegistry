use evidence_registry::{
    compare_retained_journal_anchor_history, policy_evaluator_registry,
    resolve_retained_review_package_anchor_input, route_retained_review_admission_policy_context,
    route_retained_review_admission_section_82, route_review_admission_after_anchor_comparison,
    validate_review_admission_common_request_result_fields,
    validate_review_package_anchor_transport, validate_review_request_recorded_binding,
    validate_review_result_recorded_binding, EventRecordId, ExactRecordByteResolver,
    GenesisJournalEntry, JournalAnchor, JournalAnchorHistoryComparison, JournalEntryHash,
    JournalEntryIndex, JournalReference, RecordId, RegistryId, RetainedJournal,
    RetainedReviewPackageAnchorInputError, ReviewAdmissionCommonRequestResultError,
    ReviewAdmissionPolicyContextRouteOutcome, ReviewAdmissionSection82PrerequisiteFailure,
    ReviewAdmissionSection82RoutingOutcome, ReviewPackageAnchorTransportError, ReviewRequestRecord,
    ReviewRequestRecordedBindingError, ReviewResultRecord, ReviewResultRecordedBindingError,
    StrictRecordFrame,
};
use sha2::{Digest, Sha256};

fn id(first: u8) -> [u8; 32] {
    core::array::from_fn(|index| first + index as u8)
}

fn genesis() -> GenesisJournalEntry {
    GenesisJournalEntry::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        EventRecordId::try_from(id(0x20).as_slice()).unwrap(),
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

fn independently_construct_version_1_review_request_with_policy_scope_and_anchor(
    policy_authority: &JournalReference,
    scope_ref: RecordId,
    review_package_anchor_id: [u8; 32],
) -> Vec<u8> {
    let freeze_authority = reference(1, 101, 0x20);
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
    append_journal_reference(&mut bytes, policy_authority);
    bytes.push(0x15);
    append_bstr_32(&mut bytes, *scope_ref.as_bytes());
    bytes.push(0x16);
    append_bstr_32(&mut bytes, id(0xe0));
    bytes.push(0x17);
    append_bstr_32(&mut bytes, review_package_anchor_id);
    bytes.extend_from_slice(&[0x18, 0x18]);
    append_journal_reference(&mut bytes, &operation_start);
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

fn independently_construct_version_1_review_result_with_scope(
    request_authority: &JournalReference,
    scope_ref: RecordId,
    review_package_anchor_id: [u8; 32],
) -> Vec<u8> {
    let freeze_authority = reference(1, 101, 0x20);
    let operation_start = reference(3, 300, 0x60);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x18, 31, 0x01, 0xaf, 0x00, 0x01, 0x01, 0x18, 31, 0x10]);
    append_journal_reference(&mut bytes, request_authority);
    bytes.push(0x11);
    append_journal_reference(&mut bytes, &freeze_authority);
    bytes.push(0x12);
    append_bstr_32(&mut bytes, id(0x80));
    bytes.extend_from_slice(&[0x13, 0x07, 0x14]);
    append_bstr_32(&mut bytes, *scope_ref.as_bytes());
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

fn independently_construct_review_admission_policy(
    operation_start: &JournalReference,
    scope_ref: RecordId,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x18, 40, 0x01, 0xa9, 0x00, 0x01, 0x01, 0x18, 40, 0x10]);
    append_bstr_32(&mut bytes, *scope_ref.as_bytes());
    bytes.extend_from_slice(&[0x11, 0x81, 0xa5, 0x00, 0x07, 0x01]);
    append_bstr_32(&mut bytes, *scope_ref.as_bytes());
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

fn independently_construct_exact_review_admission_scope() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[
        0x14, 0x01, 0xa5, 0x00, 0x01, 0x01, 0x14, 0x10, 0x01, 0x11, 0x01, 0x12, 0x40,
    ]);
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
fn retained_anchor_routing_exposes_no_evaluator_1015_registration() {
    assert!(
        policy_evaluator_registry()
            .iter()
            .all(|registration| registration.id != 1015),
        "the retained-anchor routing boundary must not expose evaluator 1015 metadata",
    );
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
fn retained_anchor_route_derives_authoritative_policy_from_the_retained_request() {
    let genesis = genesis();
    let scope_bytes = independently_construct_exact_review_admission_scope();
    let scope_id = RecordId::try_from(Sha256::digest(&scope_bytes).as_slice()).unwrap();
    let genesis_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(0_u64).unwrap(),
        genesis.entry_hash(),
        evidence_registry::EventTypeId::try_from(1_u64).unwrap(),
        EventRecordId::try_from(id(0x20).as_slice()).unwrap(),
    );
    let policy_bytes =
        independently_construct_review_admission_policy(&genesis_reference, scope_id);
    let policy_id = RecordId::try_from(Sha256::digest(&policy_bytes).as_slice()).unwrap();
    let policy_entry = policy_recorded_entry(genesis.entry_hash(), policy_id);
    let policy_entry_hash: [u8; 32] = Sha256::digest(&policy_entry).into();
    let policy_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(policy_entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(400_u64).unwrap(),
        EventRecordId::try_from(policy_id.as_bytes().as_slice()).unwrap(),
    );
    let expected_anchor = JournalAnchor::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(0_u64).unwrap(),
        genesis.entry_hash(),
        1,
    )
    .unwrap();
    let request_bytes =
        independently_construct_version_1_review_request_with_policy_scope_and_anchor(
            &policy_reference,
            scope_id,
            *expected_anchor.anchor_id().as_bytes(),
        );
    let request = ReviewRequestRecord::decode_authoritative(&request_bytes).unwrap();
    let request_entry = review_request_recorded_entry_at(
        2,
        JournalEntryHash::try_from(policy_entry_hash.as_slice()).unwrap(),
        request.record_id(),
        *expected_anchor.anchor_id().as_bytes(),
    );
    let request_entry_hash: [u8; 32] = Sha256::digest(&request_entry).into();
    let request_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(2_u64).unwrap(),
        JournalEntryHash::try_from(request_entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(300_u64).unwrap(),
        EventRecordId::try_from(request.record_id().as_bytes().as_slice()).unwrap(),
    );
    let result_bytes = independently_construct_version_1_review_result_with_scope(
        &request_reference,
        scope_id,
        *expected_anchor.anchor_id().as_bytes(),
    );
    let result = ReviewResultRecord::decode_authoritative(&result_bytes).unwrap();
    let result_entry = review_result_recorded_entry(
        3,
        JournalEntryHash::try_from(request_entry_hash.as_slice()).unwrap(),
        result.record_id(),
        Some(*expected_anchor.anchor_id().as_bytes()),
        Some(&request_reference),
    );
    let result_entry_hash: [u8; 32] = Sha256::digest(&result_entry).into();
    let result_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(3_u64).unwrap(),
        JournalEntryHash::try_from(result_entry_hash.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(301_u64).unwrap(),
        EventRecordId::try_from(result.record_id().as_bytes().as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&policy_entry).unwrap();
    journal.append_strict_entry(&request_entry).unwrap();
    journal.append_strict_entry(&result_entry).unwrap();
    let resolver = RecordBytes(vec![(scope_id, scope_bytes), (policy_id, policy_bytes)]);

    let outcome = route_retained_review_admission_policy_context(
        &journal,
        &request_reference,
        &request_bytes,
        &result_reference,
        &result_bytes,
        &resolver,
    );

    assert!(matches!(
        outcome,
        ReviewAdmissionPolicyContextRouteOutcome::PolicyContextReady(prerequisites)
            if prerequisites.policy_record_id() == policy_id
    ));
}
