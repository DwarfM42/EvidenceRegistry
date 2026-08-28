use evidence_registry::{
    derive_freeze_root, validate_state_only_legal_transition, EventRecordId, FreezeAttemptId,
    FreezeAttemptState, GenesisJournalEntry, JournalEntryHash, JournalEntryIndex, JournalReference,
    LifecycleObjectKind, LifecycleObjectState, OneShotRecordedState, RecordId, RegistryId,
    RegistryLifecycleState, RetainedJournal, RetainedJournalError,
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
fn retained_journal_marks_a_well_framed_unimplemented_eviction_event_as_unsupported() {
    let genesis = genesis();
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();

    assert_eq!(
        journal.append_strict_entry(&eviction_committed_bytes(genesis.entry_hash())),
        Err(RetainedJournalError::UnsupportedEntry)
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
