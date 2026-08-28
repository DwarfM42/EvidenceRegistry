use evidence_registry::{
    validate_state_only_legal_transition, EventRecordId, GenesisJournalEntry, JournalEntryHash,
    JournalEntryIndex, JournalReference, LifecycleObjectKind, LifecycleObjectState,
    OneShotRecordedState, RecordId, RegistryId, RegistryLifecycleState, RetainedJournal,
    RetainedJournalError,
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
    assert!(journal.resolve_reference(&exact_reference).is_ok());

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
fn runtime_state_only_legal_transition_rejects_an_incorrect_result_without_authority_claim() {
    let genesis = evidence_registry::EventTypeId::try_from(1_u64).unwrap();
    let absent = LifecycleObjectState::Registry(RegistryLifecycleState::Absent);
    let initialized =
        LifecycleObjectState::Registry(RegistryLifecycleState::InitializedAuthoritative);

    assert!(validate_state_only_legal_transition(genesis, absent, initialized).is_ok());
    assert!(validate_state_only_legal_transition(genesis, absent, absent).is_err());
}
