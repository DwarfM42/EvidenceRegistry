use evidence_registry::{
    derive_freeze_root, validate_state_only_legal_transition, EventRecordId, FreezeAttemptId,
    FreezeAttemptState, GenesisJournalEntry, JournalEntryHash, JournalEntryIndex, JournalReference,
    LifecycleObjectKind, LifecycleObjectState, OneShotRecordedState, RecordId, RegistryId,
    RegistryLifecycleState, ResolvedJournalReference, RetainedJournal, RetainedJournalError,
};

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

const INDEPENDENT_GENESIS_ENTRY_HASH: [u8; 32] = [
    0xae, 0x19, 0x19, 0x54, 0x9c, 0x80, 0xa0, 0xc3, 0x70, 0x8c, 0xce, 0x04, 0x85, 0xaf, 0x88, 0x1a,
    0x52, 0x23, 0xfb, 0xcf, 0x80, 0xc6, 0xb2, 0x7a, 0x2b, 0xbe, 0x60, 0x24, 0x18, 0xda, 0xbd, 0x51,
];
const INDEPENDENT_REVIEW_REQUEST_ENTRY_HASH: [u8; 32] = [
    0xd2, 0x25, 0xef, 0xc3, 0xd9, 0x6d, 0xc4, 0xb2, 0x66, 0x7f, 0xa8, 0x8c, 0xd3, 0xa3, 0xf4, 0xaf,
    0xe2, 0x0e, 0x23, 0x70, 0x5e, 0x35, 0x15, 0x93, 0x55, 0x34, 0xb0, 0xac, 0x6b, 0x4b, 0x6a, 0xe7,
];

/// Direct fixed framing for the Common Review Request Entry. Its predecessor
/// and expected Entry hash are independent constants, rather than values read
/// back from RetainedJournal reconstruction.
fn independently_construct_review_request_bytes() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(260);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xac);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, 0x01, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(&INDEPENDENT_GENESIS_ENTRY_HASH);
    bytes.extend_from_slice(&[0x04, 0x19, 0x01, 0x2c, 0x05, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x80));
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x80, 0x08, 0x04, 0x09, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x80));
    bytes.extend_from_slice(&[0x0a, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x40));
    bytes.extend_from_slice(&[0x0b, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x60));
    bytes
}

fn independent_review_request_reference(entry_index: u64) -> JournalReference {
    JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(entry_index).unwrap(),
        JournalEntryHash::try_from(INDEPENDENT_REVIEW_REQUEST_ENTRY_HASH.as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(300_u64).unwrap(),
        EventRecordId::try_from(id(0x80).as_slice()).unwrap(),
    )
}

fn review_request_bytes(previous_entry_hash: JournalEntryHash) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(225);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xac);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, 0x01, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x19, 0x01, 0x2c, 0x05, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x80));
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x80, 0x08, 0x04, 0x09, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x80));
    bytes.extend_from_slice(&[0x0a, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x40));
    bytes.extend_from_slice(&[0x0b, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x60));
    bytes
}

fn freeze_start_bytes(previous_entry_hash: JournalEntryHash) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(299);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xae);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, 0x01, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x18, 0x64, 0x05, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x80));
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x80, 0x08, 0x02, 0x09, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xa0));
    bytes.extend_from_slice(&[0x0a, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x40));
    bytes.extend_from_slice(&[0x0b, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x60));
    bytes.extend_from_slice(&[0x10, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xa0));
    bytes.extend_from_slice(&[0x11, 0x58, 0x20]);
    bytes.extend_from_slice(
        derive_freeze_root(
            RegistryId::try_from(id(0x00).as_slice()).unwrap(),
            FreezeAttemptId::try_from(id(0xa0).as_slice()).unwrap(),
        )
        .intended_root_id()
        .as_bytes(),
    );
    bytes
}

fn eviction_committed_bytes(previous_entry_hash: JournalEntryHash) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(407);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xae);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, 0x01, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x19, 0x02, 0xbd, 0x05, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x80));
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x80, 0x08, 0x03, 0x09, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xa0));
    bytes.extend_from_slice(&[0x0a, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x40));
    bytes.extend_from_slice(&[0x0b, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x60));
    bytes.extend_from_slice(&[0x15, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xa0));
    bytes.extend_from_slice(&[0x18, 0x19, 0x85, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.push(0x00);
    bytes.extend_from_slice(&[0x58, 0x20]);
    bytes.extend_from_slice(&id(0x20));
    bytes.extend_from_slice(&[0x18, 0x64, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x80));
    bytes
}

fn eviction_committed_bytes_with_unresolved_start_reference(
    previous_entry_hash: JournalEntryHash,
) -> Vec<u8> {
    fn append_start_reference(output: &mut Vec<u8>, entry_hash: JournalEntryHash) {
        output.extend_from_slice(&[0x85, 0x58, 0x20]);
        output.extend_from_slice(&id(0x00));
        output.push(0x00);
        output.extend_from_slice(&[0x58, 0x20]);
        output.extend_from_slice(entry_hash.as_bytes());
        output.extend_from_slice(&[0x19, 0x02, 0xbc, 0x58, 0x20]);
        output.extend_from_slice(&id(0x80));
    }

    let mut bytes = Vec::with_capacity(520);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xae);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, 0x01, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x19, 0x02, 0xbd, 0x05, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x80));
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x81]);
    append_start_reference(&mut bytes, previous_entry_hash);
    bytes.extend_from_slice(&[0x08, 0x03, 0x09, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xa0));
    bytes.extend_from_slice(&[0x0a, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x40));
    bytes.extend_from_slice(&[0x0b, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x60));
    bytes.extend_from_slice(&[0x15, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xa0));
    bytes.extend_from_slice(&[0x18, 0x19]);
    append_start_reference(&mut bytes, previous_entry_hash);
    bytes
}

fn eviction_started_bytes_with_mismatched_attempt_id(
    previous_entry_hash: JournalEntryHash,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(512);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xb0);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, 0x01, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x19, 0x02, 0xbc, 0x05, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x84));
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x80, 0x08, 0x03, 0x09, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xa0));
    bytes.extend_from_slice(&[0x0a, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x40));
    bytes.extend_from_slice(&[0x0b, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x60));
    bytes.extend_from_slice(&[0x15, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xb0));
    bytes.extend_from_slice(&[0x16, 0x85, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.push(0x00);
    bytes.extend_from_slice(&[0x58, 0x20]);
    bytes.extend_from_slice(previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x20));
    bytes.extend_from_slice(&[0x17, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xc0));
    bytes.extend_from_slice(&[0x18, 0x18, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xd0));
    bytes
}

fn eviction_started_bytes(
    previous_entry_hash: JournalEntryHash,
    freeze_authority_entry_index: u8,
    freeze_authority_entry_hash: JournalEntryHash,
    freeze_authority_event_type_id: u8,
    freeze_authority_event_record_id: [u8; 32],
) -> Vec<u8> {
    fn append_reference(
        output: &mut Vec<u8>,
        entry_index: u8,
        entry_hash: JournalEntryHash,
        event_type_id: u8,
        event_record_id: [u8; 32],
    ) {
        output.extend_from_slice(&[0x85, 0x58, 0x20]);
        output.extend_from_slice(&id(0x00));
        output.push(entry_index);
        output.extend_from_slice(&[0x58, 0x20]);
        output.extend_from_slice(entry_hash.as_bytes());
        if event_type_id < 24 {
            output.push(event_type_id);
        } else {
            output.extend_from_slice(&[0x18, event_type_id]);
        }
        output.extend_from_slice(&[0x58, 0x20]);
        output.extend_from_slice(&event_record_id);
    }

    let mut bytes = Vec::with_capacity(560);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xb0);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, 0x03, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x19, 0x02, 0xbc, 0x05, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x84));
    bytes.extend_from_slice(&[0x06, 0x82, 0x82, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xc0));
    bytes.extend_from_slice(&[0x82, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xd0));
    bytes.extend_from_slice(&[0x07, 0x81]);
    append_reference(
        &mut bytes,
        freeze_authority_entry_index,
        freeze_authority_entry_hash,
        freeze_authority_event_type_id,
        freeze_authority_event_record_id,
    );
    bytes.extend_from_slice(&[0x08, 0x09, 0x09, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xb0));
    bytes.extend_from_slice(&[0x0a, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x40));
    bytes.extend_from_slice(&[0x0b, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x60));
    bytes.extend_from_slice(&[0x15, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xb0));
    bytes.push(0x16);
    append_reference(
        &mut bytes,
        freeze_authority_entry_index,
        freeze_authority_entry_hash,
        freeze_authority_event_type_id,
        freeze_authority_event_record_id,
    );
    bytes.extend_from_slice(&[0x17, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xc0));
    bytes.extend_from_slice(&[0x18, 0x18, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xd0));
    bytes
}

fn freeze_committed_bytes(
    previous_entry_hash: JournalEntryHash,
    freeze_start_hash: JournalEntryHash,
    include_freeze_start_authority_dependency: bool,
    event_type_id: u8,
) -> Vec<u8> {
    fn append_freeze_start_reference(output: &mut Vec<u8>, freeze_start_hash: JournalEntryHash) {
        output.extend_from_slice(&[0x85, 0x58, 0x20]);
        output.extend_from_slice(&id(0x00));
        output.push(0x01);
        output.extend_from_slice(&[0x58, 0x20]);
        output.extend_from_slice(freeze_start_hash.as_bytes());
        output.extend_from_slice(&[0x18, 0x64, 0x58, 0x20]);
        output.extend_from_slice(&id(0x80));
    }

    let mut bytes = Vec::with_capacity(439);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xae);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, 0x02, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x18, event_type_id, 0x05, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x81));
    bytes.extend_from_slice(&[0x06, 0x80, 0x07]);
    if include_freeze_start_authority_dependency {
        bytes.push(0x81);
        append_freeze_start_reference(&mut bytes, freeze_start_hash);
    } else {
        bytes.push(0x80);
    }
    bytes.extend_from_slice(&[0x08, 0x02, 0x09, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xa0));
    bytes.extend_from_slice(&[0x0a, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x40));
    bytes.extend_from_slice(&[0x0b, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x60));
    bytes.extend_from_slice(&[0x10, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xa0));
    bytes.extend_from_slice(&[0x12]);
    append_freeze_start_reference(&mut bytes, freeze_start_hash);
    bytes
}

fn freeze_commit_rejected_bytes(
    previous_entry_hash: JournalEntryHash,
    freeze_start_hash: JournalEntryHash,
    conflicting_terminal_hash: JournalEntryHash,
    lifecycle_object_kind: u8,
    include_required_authority_dependencies: bool,
    conflicting_terminal_event_type_id: u8,
) -> Vec<u8> {
    fn append_reference(
        output: &mut Vec<u8>,
        entry_index: u8,
        entry_hash: JournalEntryHash,
        event_type_id: u8,
        event_record_id: [u8; 32],
    ) {
        output.extend_from_slice(&[0x85, 0x58, 0x20]);
        output.extend_from_slice(&id(0x00));
        output.push(entry_index);
        output.extend_from_slice(&[0x58, 0x20]);
        output.extend_from_slice(entry_hash.as_bytes());
        output.extend_from_slice(&[0x18, event_type_id, 0x58, 0x20]);
        output.extend_from_slice(&event_record_id);
    }

    let mut bytes = Vec::with_capacity(544);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xaf);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, 0x03, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x18, 0x68, 0x05, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x82));
    bytes.extend_from_slice(&[0x06, 0x80, 0x07]);
    if include_required_authority_dependencies {
        bytes.push(0x82);
        append_reference(&mut bytes, 1, freeze_start_hash, 100, id(0x80));
        append_reference(
            &mut bytes,
            2,
            conflicting_terminal_hash,
            conflicting_terminal_event_type_id,
            id(0x81),
        );
    } else {
        bytes.push(0x80);
    }
    bytes.extend_from_slice(&[0x08, lifecycle_object_kind, 0x09, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xa0));
    bytes.extend_from_slice(&[0x0a, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x40));
    bytes.extend_from_slice(&[0x0b, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x60));
    bytes.extend_from_slice(&[0x10, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0xa0));
    bytes.extend_from_slice(&[0x12]);
    append_reference(&mut bytes, 1, freeze_start_hash, 100, id(0x80));
    bytes.extend_from_slice(&[0x13]);
    append_reference(
        &mut bytes,
        2,
        conflicting_terminal_hash,
        conflicting_terminal_event_type_id,
        id(0x81),
    );
    bytes
}

fn verification_recorded_bytes(
    previous_entry_hash: JournalEntryHash,
    lifecycle_object_kind: u8,
    lifecycle_object_id: [u8; 32],
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.extend_from_slice(&[0xac, 0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x00));
    bytes.extend_from_slice(&[0x02, 0x01, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x18, 0xc8, 0x05, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x82));
    bytes.extend_from_slice(&[
        0x06,
        0x80,
        0x07,
        0x80,
        0x08,
        lifecycle_object_kind,
        0x09,
        0x58,
        0x20,
    ]);
    bytes.extend_from_slice(&lifecycle_object_id);
    bytes.extend_from_slice(&[0x0a, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x40));
    bytes.extend_from_slice(&[0x0b, 0x58, 0x20]);
    bytes.extend_from_slice(&id(0x50));
    bytes
}

#[test]
fn retained_genesis_replays_registry_state_and_resolves_its_exact_reference() {
    let genesis = genesis();
    let journal =
        RetainedJournal::from_authoritative_genesis(&genesis.authoritative_cbor()).unwrap();

    let replay = journal.reconstruct_state().unwrap();
    assert_eq!(
        replay.registry_state(),
        RegistryLifecycleState::InitializedAuthoritative
    );
    assert_eq!(replay.entry_count(), 1);
    assert_eq!(replay.journal_head_index().value(), 0);
    assert_eq!(replay.journal_head_hash(), genesis.entry_hash());

    let exact_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(0_u64).unwrap(),
        genesis.entry_hash(),
        evidence_registry::EventTypeId::try_from(1_u64).unwrap(),
        EventRecordId::try_from(id(0x20).as_slice()).unwrap(),
    );
    let resolved: ResolvedJournalReference = journal.resolve_reference(&exact_reference).unwrap();
    assert_eq!(resolved.registry_id(), exact_reference.registry_id());
    assert_eq!(resolved.entry_index(), exact_reference.entry_index());
    assert_eq!(resolved.entry_hash(), exact_reference.entry_hash());
    assert_eq!(resolved.event_type_id(), exact_reference.event_type_id());
    assert_eq!(
        resolved.event_record_id(),
        exact_reference.event_record_id()
    );
    assert_eq!(resolved.previous_entry_hash(), None);
    assert_eq!(
        resolved.lifecycle_object_kind(),
        LifecycleObjectKind::Registry
    );
    assert_eq!(
        resolved.lifecycle_object_id(),
        *exact_reference.registry_id().as_bytes()
    );

    let wrong_hash = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(0_u64).unwrap(),
        JournalEntryHash::try_from(id(0x80).as_slice()).unwrap(),
        evidence_registry::EventTypeId::try_from(1_u64).unwrap(),
        EventRecordId::try_from(id(0x20).as_slice()).unwrap(),
    );
    assert_eq!(
        journal.resolve_reference(&wrong_hash),
        Err(RetainedJournalError::ReferenceMismatch)
    );
}

#[test]
fn retained_journal_resolves_a_common_entry_from_independently_bound_bytes() {
    let genesis = genesis();
    assert_eq!(
        genesis.entry_hash().as_bytes(),
        &INDEPENDENT_GENESIS_ENTRY_HASH
    );
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal
        .append_strict_entry(&independently_construct_review_request_bytes())
        .unwrap();

    let exact_reference = independent_review_request_reference(1);
    let resolved = journal.resolve_reference(&exact_reference).unwrap();
    assert_eq!(resolved.registry_id(), exact_reference.registry_id());
    assert_eq!(resolved.entry_index(), exact_reference.entry_index());
    assert_eq!(resolved.entry_hash(), exact_reference.entry_hash());
    assert_eq!(
        resolved.previous_entry_hash(),
        Some(JournalEntryHash::try_from(INDEPENDENT_GENESIS_ENTRY_HASH.as_slice()).unwrap())
    );
    assert_eq!(resolved.event_type_id(), exact_reference.event_type_id());
    assert_eq!(
        resolved.event_record_id(),
        exact_reference.event_record_id()
    );
    assert_eq!(
        resolved.lifecycle_object_kind(),
        LifecycleObjectKind::ReviewRequest
    );
    assert_eq!(resolved.lifecycle_object_id(), id(0x80));
    assert_eq!(
        resolved.storage_capability_class_id(),
        RecordId::try_from(id(0x40).as_slice()).unwrap()
    );
    assert_eq!(
        resolved.environment_observation_id(),
        RecordId::try_from(id(0x60).as_slice()).unwrap()
    );
}

#[test]
fn retained_journal_rejects_absent_or_nonidentical_common_references() {
    let genesis = genesis();
    assert_eq!(
        genesis.entry_hash().as_bytes(),
        &INDEPENDENT_GENESIS_ENTRY_HASH
    );
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal
        .append_strict_entry(&independently_construct_review_request_bytes())
        .unwrap();

    let exact_reference = independent_review_request_reference(1);
    let wrong_registry = JournalReference::new(
        RegistryId::try_from(id(0x01).as_slice()).unwrap(),
        exact_reference.entry_index(),
        exact_reference.entry_hash(),
        exact_reference.event_type_id(),
        exact_reference.event_record_id(),
    );
    assert_eq!(
        journal.resolve_reference(&wrong_registry),
        Err(RetainedJournalError::ReferenceMismatch)
    );

    let absent_index = independent_review_request_reference(2);
    assert_eq!(
        journal.resolve_reference(&absent_index),
        Err(RetainedJournalError::MissingReference)
    );

    let wrong_hash = JournalReference::new(
        exact_reference.registry_id(),
        exact_reference.entry_index(),
        JournalEntryHash::try_from(id(0x81).as_slice()).unwrap(),
        exact_reference.event_type_id(),
        exact_reference.event_record_id(),
    );
    assert_eq!(
        journal.resolve_reference(&wrong_hash),
        Err(RetainedJournalError::ReferenceMismatch)
    );

    let wrong_event_type = JournalReference::new(
        exact_reference.registry_id(),
        exact_reference.entry_index(),
        exact_reference.entry_hash(),
        evidence_registry::EventTypeId::try_from(1_u64).unwrap(),
        exact_reference.event_record_id(),
    );
    assert_eq!(
        journal.resolve_reference(&wrong_event_type),
        Err(RetainedJournalError::ReferenceMismatch)
    );

    let wrong_event_record = JournalReference::new(
        exact_reference.registry_id(),
        exact_reference.entry_index(),
        exact_reference.entry_hash(),
        exact_reference.event_type_id(),
        EventRecordId::try_from(id(0x81).as_slice()).unwrap(),
    );
    assert_eq!(
        journal.resolve_reference(&wrong_event_record),
        Err(RetainedJournalError::ReferenceMismatch)
    );
}

#[test]
fn retained_journal_rejects_a_second_genesis_instead_of_overwriting_history() {
    let mut journal = RetainedJournal::from_genesis(genesis()).unwrap();

    assert_eq!(
        journal.append_genesis(genesis()),
        Err(RetainedJournalError::UnexpectedEntryIndex)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 1);
}

#[test]
fn retained_journal_replays_a_strict_record_backed_one_shot_event() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&review_request_bytes(genesis.entry_hash()))
        .unwrap();

    let replay = journal.reconstruct_state().unwrap();
    assert_eq!(replay.entry_count(), 2);
    assert_eq!(
        replay.state_for(
            LifecycleObjectKind::ReviewRequest,
            EventRecordId::try_from(id(0x80).as_slice())
                .unwrap()
                .as_bytes(),
        ),
        Some(LifecycleObjectState::OneShot(
            OneShotRecordedState::Recorded
        ))
    );
}

#[test]
fn retained_journal_rejects_a_broken_hash_link_without_mutating_replay_history() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    let mut malformed_link = review_request_bytes(genesis.entry_hash());
    let previous_hash = malformed_link
        .windows(3)
        .position(|window| window == [0x03, 0x58, 0x20])
        .unwrap()
        + 3;
    malformed_link[previous_hash] ^= 0x01;

    assert_eq!(
        journal.append_strict_entry(&malformed_link),
        Err(RetainedJournalError::PreviousHashMismatch)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 1);
}

#[test]
fn retained_journal_replays_a_structurally_valid_freeze_start() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();

    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash()))
        .unwrap();

    let replay = journal.reconstruct_state().unwrap();
    assert_eq!(replay.entry_count(), 2);
    assert_eq!(
        replay.state_for(LifecycleObjectKind::FreezeAttempt, &id(0xa0)),
        Some(LifecycleObjectState::FreezeAttempt(
            FreezeAttemptState::Open
        ))
    );
}

#[test]
fn journal_only_reconstruction_retains_the_capability_epoch_of_every_entry() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash()))
        .unwrap();

    let replay = journal.reconstruct_state().unwrap();
    let epochs = replay.capability_epochs();
    assert_eq!(epochs.len(), 2);
    for (expected_index, epoch) in epochs.iter().enumerate() {
        assert_eq!(epoch.entry_index().value(), expected_index as u64);
        assert_eq!(
            epoch.storage_capability_class_id(),
            RecordId::try_from(id(0x40).as_slice()).unwrap()
        );
        assert_eq!(
            epoch.environment_observation_id(),
            RecordId::try_from(id(0x60).as_slice()).unwrap()
        );
    }
}

#[test]
fn retained_reference_resolution_preserves_common_capability_and_environment_context() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash()))
        .unwrap();
    let head = journal.reconstruct_state().unwrap();
    let freeze_start_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(1_u64).unwrap(),
        head.journal_head_hash(),
        evidence_registry::EventTypeId::try_from(100_u64).unwrap(),
        EventRecordId::try_from(id(0x80).as_slice()).unwrap(),
    );

    let resolved = journal.resolve_reference(&freeze_start_reference).unwrap();
    assert_eq!(
        resolved.storage_capability_class_id(),
        RecordId::try_from(id(0x40).as_slice()).unwrap()
    );
    assert_eq!(
        resolved.environment_observation_id(),
        RecordId::try_from(id(0x60).as_slice()).unwrap()
    );
}

#[test]
fn retained_journal_rejects_a_freeze_start_with_a_nonderived_intended_root() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    let mut malformed = freeze_start_bytes(genesis.entry_hash());
    *malformed.last_mut().unwrap() ^= 0x01;

    assert_eq!(
        journal.append_strict_entry(&malformed),
        Err(RetainedJournalError::DecodeError)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 1);
}

#[test]
fn retained_journal_replays_a_freeze_commit_bound_to_its_retained_start() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash()))
        .unwrap();
    let freeze_start_hash = journal.reconstruct_state().unwrap().journal_head_hash();

    journal
        .append_strict_entry(&freeze_committed_bytes(
            freeze_start_hash,
            freeze_start_hash,
            true,
            101,
        ))
        .unwrap();

    let replay = journal.reconstruct_state().unwrap();
    assert_eq!(replay.entry_count(), 3);
    assert_eq!(
        replay.state_for(LifecycleObjectKind::FreezeAttempt, &id(0xa0)),
        Some(LifecycleObjectState::FreezeAttempt(
            FreezeAttemptState::Committed
        ))
    );
}

#[test]
fn journal_only_reconstruction_retains_freeze_root_and_terminal_disposition() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash()))
        .unwrap();
    let freeze_start_hash = journal.reconstruct_state().unwrap().journal_head_hash();
    journal
        .append_strict_entry(&freeze_committed_bytes(
            freeze_start_hash,
            freeze_start_hash,
            true,
            101,
        ))
        .unwrap();

    let replay = journal.reconstruct_state().unwrap();
    let attempts = replay.freeze_attempts();
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].freeze_attempt_id().as_bytes(), &id(0xa0));
    assert_eq!(
        attempts[0].intended_root_id().as_bytes(),
        &[
            0x7f, 0xa3, 0xd1, 0xbc, 0xfc, 0x31, 0xbc, 0x7b, 0x43, 0x17, 0x6a, 0xec, 0x28, 0xa6,
            0xd3, 0xd9, 0x44, 0xac, 0xf9, 0x31, 0xd8, 0x77, 0xc8, 0x0c, 0x6b, 0x61, 0xf7, 0x88,
            0xab, 0xe5, 0x7d, 0xb7,
        ]
    );
    assert_eq!(attempts[0].state(), FreezeAttemptState::Committed);
}

#[test]
fn retained_journal_replays_a_freeze_recovery_abort_bound_to_its_retained_start() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash()))
        .unwrap();
    let freeze_start_hash = journal.reconstruct_state().unwrap().journal_head_hash();

    journal
        .append_strict_entry(&freeze_committed_bytes(
            freeze_start_hash,
            freeze_start_hash,
            true,
            102,
        ))
        .unwrap();

    assert_eq!(
        journal
            .reconstruct_state()
            .unwrap()
            .state_for(LifecycleObjectKind::FreezeAttempt, &id(0xa0)),
        Some(LifecycleObjectState::FreezeAttempt(
            FreezeAttemptState::AbortedRecovery
        ))
    );
}

#[test]
fn retained_journal_rejects_a_second_freeze_terminal_without_mutating_history() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash()))
        .unwrap();
    let freeze_start_hash = journal.reconstruct_state().unwrap().journal_head_hash();
    journal
        .append_strict_entry(&freeze_committed_bytes(
            freeze_start_hash,
            freeze_start_hash,
            true,
            101,
        ))
        .unwrap();
    let committed_hash = journal.reconstruct_state().unwrap().journal_head_hash();
    let mut second_terminal = freeze_committed_bytes(committed_hash, freeze_start_hash, true, 102);
    let entry_index = second_terminal
        .windows(5)
        .position(|window| window == [0x02, 0x02, 0x03, 0x58, 0x20])
        .unwrap();
    second_terminal[entry_index + 1] = 0x03;

    assert_eq!(
        journal.append_strict_entry(&second_terminal),
        Err(RetainedJournalError::LifecycleTransition)
    );
    let replay = journal.reconstruct_state().unwrap();
    assert_eq!(replay.entry_count(), 3);
    assert_eq!(
        replay.state_for(LifecycleObjectKind::FreezeAttempt, &id(0xa0)),
        Some(LifecycleObjectState::FreezeAttempt(
            FreezeAttemptState::Committed
        ))
    );
}

#[test]
fn retained_journal_replays_a_freeze_operator_abort_bound_to_its_retained_start() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash()))
        .unwrap();
    let freeze_start_hash = journal.reconstruct_state().unwrap().journal_head_hash();

    journal
        .append_strict_entry(&freeze_committed_bytes(
            freeze_start_hash,
            freeze_start_hash,
            true,
            103,
        ))
        .unwrap();

    assert_eq!(
        journal
            .reconstruct_state()
            .unwrap()
            .state_for(LifecycleObjectKind::FreezeAttempt, &id(0xa0)),
        Some(LifecycleObjectState::FreezeAttempt(
            FreezeAttemptState::AbortedByOperatorAssertion
        ))
    );
}

#[test]
fn retained_journal_rejects_a_freeze_commit_without_its_named_authority_dependency() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash()))
        .unwrap();
    let freeze_start_hash = journal.reconstruct_state().unwrap().journal_head_hash();

    assert_eq!(
        journal.append_strict_entry(&freeze_committed_bytes(
            freeze_start_hash,
            freeze_start_hash,
            false,
            101,
        )),
        Err(RetainedJournalError::DecodeError)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 2);
}

#[test]
fn retained_journal_rejects_a_freeze_commit_rejection_with_a_nonfreeze_kind() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash()))
        .unwrap();
    let freeze_start_hash = journal.reconstruct_state().unwrap().journal_head_hash();
    journal
        .append_strict_entry(&freeze_committed_bytes(
            freeze_start_hash,
            freeze_start_hash,
            true,
            101,
        ))
        .unwrap();
    let terminal_hash = journal.reconstruct_state().unwrap().journal_head_hash();

    assert_eq!(
        journal.append_strict_entry(&freeze_commit_rejected_bytes(
            terminal_hash,
            freeze_start_hash,
            terminal_hash,
            4,
            false,
            101,
        )),
        Err(RetainedJournalError::DecodeError)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 3);
}

#[test]
fn retained_journal_rejects_a_freeze_commit_rejection_without_its_required_references() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash()))
        .unwrap();
    let freeze_start_hash = journal.reconstruct_state().unwrap().journal_head_hash();
    journal
        .append_strict_entry(&freeze_committed_bytes(
            freeze_start_hash,
            freeze_start_hash,
            true,
            101,
        ))
        .unwrap();
    let terminal_hash = journal.reconstruct_state().unwrap().journal_head_hash();

    assert_eq!(
        journal.append_strict_entry(&freeze_commit_rejected_bytes(
            terminal_hash,
            freeze_start_hash,
            terminal_hash,
            2,
            false,
            101,
        )),
        Err(RetainedJournalError::DecodeError)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 3);
}

#[test]
fn retained_journal_rejects_a_freeze_commit_rejection_with_a_nonterminal_conflict_reference() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash()))
        .unwrap();
    let freeze_start_hash = journal.reconstruct_state().unwrap().journal_head_hash();
    journal
        .append_strict_entry(&freeze_committed_bytes(
            freeze_start_hash,
            freeze_start_hash,
            true,
            101,
        ))
        .unwrap();
    let terminal_hash = journal.reconstruct_state().unwrap().journal_head_hash();

    assert_eq!(
        journal.append_strict_entry(&freeze_commit_rejected_bytes(
            terminal_hash,
            freeze_start_hash,
            terminal_hash,
            2,
            true,
            100,
        )),
        Err(RetainedJournalError::DecodeError)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 3);
}

#[test]
fn retained_journal_rejects_a_freeze_commit_rejection_with_a_broken_chain_link() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash()))
        .unwrap();
    let freeze_start_hash = journal.reconstruct_state().unwrap().journal_head_hash();
    journal
        .append_strict_entry(&freeze_committed_bytes(
            freeze_start_hash,
            freeze_start_hash,
            true,
            101,
        ))
        .unwrap();
    let terminal_hash = journal.reconstruct_state().unwrap().journal_head_hash();
    let mut malformed = freeze_commit_rejected_bytes(
        terminal_hash,
        freeze_start_hash,
        terminal_hash,
        2,
        true,
        101,
    );
    let previous_hash = malformed
        .windows(3)
        .position(|window| window == [0x03, 0x58, 0x20])
        .unwrap()
        + 3;
    malformed[previous_hash] ^= 0x01;

    assert_eq!(
        journal.append_strict_entry(&malformed),
        Err(RetainedJournalError::PreviousHashMismatch)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 3);
}

#[test]
fn retained_journal_rejects_an_ordinary_verification_with_wrong_lifecycle_binding() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();

    assert_eq!(
        journal.append_strict_entry(&verification_recorded_bytes(
            genesis.entry_hash(),
            4,
            id(0x83),
        )),
        Err(RetainedJournalError::DecodeError)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 1);
}

#[test]
fn retained_journal_rejects_an_eviction_start_with_mismatched_attempt_identity() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();

    assert_eq!(
        journal.append_strict_entry(&eviction_started_bytes_with_mismatched_attempt_id(
            genesis.entry_hash(),
        )),
        Err(RetainedJournalError::DecodeError)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 1);
}

#[test]
fn retained_journal_classifies_eviction_freeze_authority_by_exact_event_type() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash()))
        .unwrap();
    let freeze_start_hash = journal.reconstruct_state().unwrap().journal_head_hash();
    journal
        .append_strict_entry(&freeze_committed_bytes(
            freeze_start_hash,
            freeze_start_hash,
            true,
            101,
        ))
        .unwrap();
    let freeze_committed_hash = journal.reconstruct_state().unwrap().journal_head_hash();

    assert_eq!(
        journal.append_strict_entry(&eviction_started_bytes(
            freeze_committed_hash,
            2,
            freeze_committed_hash,
            101,
            id(0x81),
        )),
        Err(RetainedJournalError::UnsupportedEntry)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 3);

    assert_eq!(
        journal.append_strict_entry(&eviction_started_bytes(
            freeze_committed_hash,
            0,
            genesis.entry_hash(),
            1,
            id(0x20),
        )),
        Err(RetainedJournalError::DecodeError)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 3);
}

#[test]
fn retained_journal_rejects_an_eviction_entry_with_wrong_required_reference_role() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();

    assert_eq!(
        journal.append_strict_entry(&eviction_committed_bytes(genesis.entry_hash())),
        Err(RetainedJournalError::DecodeError)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 1);
}

#[test]
fn retained_journal_rejects_an_eviction_terminal_with_an_unresolved_named_start() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();

    assert_eq!(
        journal.append_strict_entry(&eviction_committed_bytes_with_unresolved_start_reference(
            genesis.entry_hash(),
        )),
        Err(RetainedJournalError::DecodeError)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 1);
}

#[test]
fn retained_journal_rejects_a_malformed_event_specific_entry_without_mutating_history() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    let mut malformed = freeze_start_bytes(genesis.entry_hash());
    malformed.pop();

    assert_eq!(
        journal.append_strict_entry(&malformed),
        Err(RetainedJournalError::DecodeError)
    );
    assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 1);
}

#[test]
fn runtime_state_only_legal_transition_rejects_an_incorrect_result_without_authority_claim() {
    let genesis = evidence_registry::EventTypeId::try_from(1_u64).unwrap();
    let absent = LifecycleObjectState::Registry(RegistryLifecycleState::Absent);
    let initialized =
        LifecycleObjectState::Registry(RegistryLifecycleState::InitializedAuthoritative);

    assert!(validate_state_only_legal_transition(genesis, absent, initialized).is_ok());
    assert!(validate_state_only_legal_transition(genesis, absent, absent).is_err());
}
