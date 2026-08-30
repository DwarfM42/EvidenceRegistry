use evidence_registry::{
    check_review_admission_policy_context, evaluate_review_admission_gate_scope_1015,
    policy_evaluator_registry, resolve_review_admission_exact_scope_profile,
    validate_review_admission_policy_context_prerequisites,
    validate_review_admission_policy_recorded_binding, EventRecordId, EventTypeId,
    ExactRecordByteResolver, GenesisJournalEntry, JournalEntryHash, JournalEntryIndex,
    JournalReference, PolicyEvaluationContext, RecordId, RegistryId, RetainedJournal,
    ReviewAdmissionCommonRequestResultError, ReviewAdmissionGateScope1015Result,
    ReviewAdmissionPolicyContextDeclaration, ReviewAdmissionPolicyContextPrerequisitesError,
    ReviewAdmissionPolicyRecord, StrictRecordFrame,
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

fn independently_construct_exact_review_admission_scope() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[
        0x14, 0x01, 0xa5, 0x00, 0x01, 0x01, 0x14, 0x10, 0x01, 0x11, 0x01, 0x12, 0x40,
    ]);
    bytes
}

fn synthetic_reference(index: u64, event_type_id: u16, record_first: u8) -> JournalReference {
    JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(index).unwrap(),
        JournalEntryHash::try_from(id(0x80 + index as u8).as_slice()).unwrap(),
        EventTypeId::try_from(u64::from(event_type_id)).unwrap(),
        EventRecordId::try_from(id(record_first).as_slice()).unwrap(),
    )
}

fn independently_construct_review_request_with_scope(scope_ref: RecordId) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x18, 30, 0x01, 0xac, 0x00, 0x01, 0x01, 0x18, 30, 0x10]);
    append_journal_reference(&mut bytes, &synthetic_reference(0, 101, 0x20));
    bytes.push(0x11);
    append_bstr_32(&mut bytes, id(0x80));
    bytes.extend_from_slice(&[0x12, 0x07, 0x13]);
    append_bstr_32(&mut bytes, id(0xa0));
    bytes.push(0x14);
    append_journal_reference(&mut bytes, &synthetic_reference(1, 400, 0x40));
    bytes.push(0x15);
    append_bstr_32(&mut bytes, *scope_ref.as_bytes());
    bytes.push(0x16);
    append_bstr_32(&mut bytes, id(0xe0));
    bytes.push(0x17);
    append_bstr_32(&mut bytes, id(0x10));
    bytes.extend_from_slice(&[0x18, 0x18]);
    append_journal_reference(&mut bytes, &synthetic_reference(2, 300, 0x60));
    bytes.extend_from_slice(&[0x18, 0x19, 0x01]);
    bytes
}

fn independently_construct_review_result_with_scope(scope_ref: RecordId) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x18, 31, 0x01, 0xaf, 0x00, 0x01, 0x01, 0x18, 31, 0x10]);
    append_journal_reference(&mut bytes, &synthetic_reference(1, 300, 0x20));
    bytes.push(0x11);
    append_journal_reference(&mut bytes, &synthetic_reference(0, 101, 0x20));
    bytes.push(0x12);
    append_bstr_32(&mut bytes, id(0x80));
    bytes.extend_from_slice(&[0x13, 0x07, 0x14]);
    append_bstr_32(&mut bytes, *scope_ref.as_bytes());
    bytes.push(0x15);
    append_bstr_32(&mut bytes, id(0xe0));
    bytes.extend_from_slice(&[0x16, 0x01, 0x17, 0x01, 0x18, 0x18, 0x81, 0x62]);
    bytes.extend_from_slice(b"OK");
    bytes.extend_from_slice(&[0x18, 0x19, 0x80, 0x18, 0x1b]);
    append_bstr_32(&mut bytes, id(0x10));
    bytes.extend_from_slice(&[0x18, 0x1c]);
    append_journal_reference(&mut bytes, &synthetic_reference(2, 300, 0x60));
    bytes.extend_from_slice(&[0x18, 0x1d, 0x01]);
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
fn review_admission_policy_decodes_all_applicable_frozen_requirements() {
    let start_genesis = genesis();
    let bytes = independently_construct_review_admission_policy(
        &genesis_reference(&start_genesis),
        RecordId::try_from(id(0xc0).as_slice()).unwrap(),
    );
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
    let policy_bytes = independently_construct_review_admission_policy(
        &genesis_reference(&genesis),
        RecordId::try_from(id(0xc0).as_slice()).unwrap(),
    );
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

#[test]
fn review_admission_exact_scope_profile_requires_exact_typed_empty_profile_payload() {
    let scope_bytes = independently_construct_exact_review_admission_scope();
    let scope_id = RecordId::try_from(Sha256::digest(&scope_bytes).as_slice()).unwrap();
    let resolver = RecordBytes(vec![(scope_id, scope_bytes)]);

    let scope = resolve_review_admission_exact_scope_profile(scope_id, &resolver).unwrap();

    assert_eq!(scope.record_id(), scope_id);
    assert_eq!(scope.input().scope_profile_id, 1);
    assert_eq!(scope.input().scope_profile_version, 1);
    assert!(scope.input().scope_payload.is_empty());
}

#[test]
fn review_admission_policy_context_prerequisites_stop_at_common_request_result_failure() {
    let genesis = genesis();
    let policy_bytes = independently_construct_review_admission_policy(
        &genesis_reference(&genesis),
        RecordId::try_from(id(0xc0).as_slice()).unwrap(),
    );
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

    assert_eq!(
        validate_review_admission_policy_context_prerequisites(
            &journal,
            &policy_reference,
            &policy_bytes,
            &[0xff],
            &[0xff],
            &RecordBytes(vec![]),
        ),
        Err(
            ReviewAdmissionPolicyContextPrerequisitesError::CommonRequestResult(
                ReviewAdmissionCommonRequestResultError::RequestDecode
            )
        )
    );
}

#[test]
fn review_admission_policy_context_prerequisites_resolve_both_exact_typed_scopes_and_evaluator_1015_accepts(
) {
    let genesis = genesis();
    let scope_bytes = independently_construct_exact_review_admission_scope();
    let scope_id = RecordId::try_from(Sha256::digest(&scope_bytes).as_slice()).unwrap();
    let policy_bytes =
        independently_construct_review_admission_policy(&genesis_reference(&genesis), scope_id);
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
    let request_bytes = independently_construct_review_request_with_scope(scope_id);
    let result_bytes = independently_construct_review_result_with_scope(scope_id);
    let resolver = RecordBytes(vec![(scope_id, scope_bytes)]);

    let prerequisites = validate_review_admission_policy_context_prerequisites(
        &journal,
        &policy_reference,
        &policy_bytes,
        &request_bytes,
        &result_bytes,
        &resolver,
    )
    .unwrap();

    assert_eq!(prerequisites.policy_record_id(), policy_record_id);
    assert_eq!(prerequisites.gate_scope_ref(), scope_id);
    assert_eq!(prerequisites.common_review_scope_ref(), scope_id);
    assert_eq!(
        evaluate_review_admission_gate_scope_1015(prerequisites),
        ReviewAdmissionGateScope1015Result::Pass,
    );
}

#[test]
fn review_admission_policy_preserves_context_unsupported_before_requirement_evaluation() {
    let genesis = genesis();
    let mut policy_bytes = independently_construct_review_admission_policy(
        &genesis_reference(&genesis),
        RecordId::try_from(id(0xc0).as_slice()).unwrap(),
    );
    let context_offset = policy_bytes
        .windows(3)
        .position(|window| window == [0x18, 0x1e, 0x81])
        .unwrap()
        + 3;
    policy_bytes[context_offset] = 1;
    let policy = ReviewAdmissionPolicyRecord::decode_authoritative(&policy_bytes).unwrap();

    assert_eq!(
        check_review_admission_policy_context(&policy),
        ReviewAdmissionPolicyContextDeclaration::PolicyContextUnsupported,
    );
}

#[test]
fn review_admission_policy_requires_the_exact_single_review_admission_context_for_profile_1015() {
    let genesis = genesis();
    let mut policy_bytes = independently_construct_review_admission_policy(
        &genesis_reference(&genesis),
        RecordId::try_from(id(0xc0).as_slice()).unwrap(),
    );
    let context_offset = policy_bytes
        .windows(3)
        .position(|window| window == [0x18, 0x1e, 0x81])
        .unwrap()
        + 2;
    policy_bytes.splice(context_offset..context_offset + 2, [0x82, 0x01, 0x02]);
    let policy = ReviewAdmissionPolicyRecord::decode_authoritative(&policy_bytes).unwrap();

    assert_eq!(
        check_review_admission_policy_context(&policy),
        ReviewAdmissionPolicyContextDeclaration::PolicyContextUnsupported,
    );
}

#[test]
fn evaluator_1015_is_exactly_registered_and_returns_only_its_individual_scope_result() {
    let genesis = genesis();
    let common_scope_bytes = independently_construct_exact_review_admission_scope();
    let common_scope_id =
        RecordId::try_from(Sha256::digest(&common_scope_bytes).as_slice()).unwrap();
    let mut gate_scope_bytes = independently_construct_exact_review_admission_scope();
    let map_start = gate_scope_bytes
        .windows(3)
        .position(|window| window == [0x14, 0x01, 0xa5])
        .unwrap()
        + 2;
    gate_scope_bytes[map_start] = 0xa6;
    gate_scope_bytes.extend_from_slice(&[0x13, 0x61, b'x']);
    let gate_scope_id = RecordId::try_from(Sha256::digest(&gate_scope_bytes).as_slice()).unwrap();
    let policy_bytes = independently_construct_review_admission_policy(
        &genesis_reference(&genesis),
        gate_scope_id,
    );
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
    let request_bytes = independently_construct_review_request_with_scope(common_scope_id);
    let result_bytes = independently_construct_review_result_with_scope(common_scope_id);
    let resolver = RecordBytes(vec![
        (common_scope_id, common_scope_bytes),
        (gate_scope_id, gate_scope_bytes),
    ]);
    let prerequisites = validate_review_admission_policy_context_prerequisites(
        &journal,
        &policy_reference,
        &policy_bytes,
        &request_bytes,
        &result_bytes,
        &resolver,
    )
    .unwrap();

    assert!(policy_evaluator_registry().iter().any(|registration| {
        registration.id == 1015
            && registration.name == "POLICY_REVIEW_ADMISSION_GATE_SCOPE_EXACT_BINDING"
    }));
    assert_eq!(
        evaluate_review_admission_gate_scope_1015(prerequisites),
        ReviewAdmissionGateScope1015Result::Fail,
    );
}
