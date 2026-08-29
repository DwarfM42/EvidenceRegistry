use evidence_registry::{
    validate_event_record_structural_binding, EventRecordId, EventRecordStructuralBindingError,
    EventTypeId, GenesisJournalEntry, JournalEntryHash, JournalEntryIndex, JournalReference,
    RecordId, RegistryId, RetainedJournal,
};
use sha2::{Digest, Sha256};

const GENESIS_HASH_HEX: &str = "ae1919549c80a0c3708cce0485af881a5223fbcf80c6b27a2bbe602418dabd51";
const START_RECORD_ID_HEX: &str =
    "949f6d7f389641c8c615904f8b1473083b8b029f9866873de4bec17d4e1a685c";
const START_HASH_HEX: &str = "0d678a8c08b32c670e27eb5e815f8077e9be62f5c8578d19ce72835bd88ba168";

const START_RECORD_HEX: &str = concat!(
    "84781a45766964656e636552656769737472792e5265636f72642e76310301a600010103105820",
    "a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf115820",
    "7fa3d1bcfc31bc7b43176aec28a6d3d944acf931d877c80c6b61f788abe57db7125820",
    "e0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff135820",
    "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f"
);
const START_ENTRY_HEX: &str = concat!(
    "82782045766964656e636552656769737472792e4a6f75726e616c456e7472792e7631ae0001015820",
    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f0201035820",
    "ae1919549c80a0c3708cce0485af881a5223fbcf80c6b27a2bbe602418dabd51041864055820",
    "949f6d7f389641c8c615904f8b1473083b8b029f9866873de4bec17d4e1a685c068007800802095820",
    "a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf0a5820",
    "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f0b5820",
    "606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f105820",
    "a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf115820",
    "7fa3d1bcfc31bc7b43176aec28a6d3d944acf931d877c80c6b61f788abe57db7"
);

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}

fn hex_id(hex: &str) -> [u8; 32] {
    hex_bytes(hex).try_into().unwrap()
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn retained_start_fixture() -> (RetainedJournal, JournalReference, Vec<u8>) {
    let registry_id = RegistryId::try_from(
        hex_id("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f").as_slice(),
    )
    .unwrap();
    let genesis = GenesisJournalEntry::new(
        registry_id,
        EventRecordId::try_from(
            hex_id("202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f").as_slice(),
        )
        .unwrap(),
        RecordId::try_from(
            hex_id("404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f").as_slice(),
        )
        .unwrap(),
        RecordId::try_from(
            hex_id("606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f").as_slice(),
        )
        .unwrap(),
    );
    assert_eq!(genesis.entry_hash().as_bytes(), &hex_id(GENESIS_HASH_HEX));

    let start_record = hex_bytes(START_RECORD_HEX);
    assert_eq!(sha256(&start_record), hex_id(START_RECORD_ID_HEX));
    let start_entry = hex_bytes(START_ENTRY_HEX);
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&start_entry).unwrap();
    assert_eq!(
        journal
            .reconstruct_state()
            .unwrap()
            .journal_head_hash()
            .as_bytes(),
        &hex_id(START_HASH_HEX)
    );
    let start_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(hex_id(START_HASH_HEX).as_slice()).unwrap(),
        EventTypeId::try_from(100_u64).unwrap(),
        EventRecordId::try_from(hex_id(START_RECORD_ID_HEX).as_slice()).unwrap(),
    );
    (journal, start_reference, start_record)
}

#[test]
fn event_record_structural_binding_requires_exact_retained_identity_and_type() {
    let (journal, start_reference, start_record) = retained_start_fixture();

    let binding =
        validate_event_record_structural_binding(&journal, &start_reference, &start_record)
            .unwrap();
    assert_eq!(binding.event_reference(), &start_reference);
    assert_eq!(binding.record_id().as_bytes(), &hex_id(START_RECORD_ID_HEX));
    assert_eq!(binding.record_type_id().value(), 3);
}

#[test]
fn event_record_structural_binding_rejects_malformed_wrong_identity_wrong_type_and_unknown_reference(
) {
    let (journal, start_reference, start_record) = retained_start_fixture();

    assert_eq!(
        validate_event_record_structural_binding(&journal, &start_reference, &[0x80]),
        Err(EventRecordStructuralBindingError::RecordDecode)
    );

    let mut wrong_identity = start_record.clone();
    let offset = wrong_identity
        .windows(4)
        .position(|window| window == [0x10, 0x58, 0x20, 0xa0])
        .unwrap();
    wrong_identity[offset + 3] = 0xa1;
    assert_eq!(
        validate_event_record_structural_binding(&journal, &start_reference, &wrong_identity),
        Err(EventRecordStructuralBindingError::RecordIdMismatch)
    );

    let mut wrong_type = vec![0x84, 0x78, 0x1a];
    wrong_type.extend_from_slice(b"EvidenceRegistry.Record.v1");
    wrong_type.extend_from_slice(&[0x04, 0x01, 0xa2, 0x00, 0x01, 0x01, 0x04]);
    assert_eq!(
        validate_event_record_structural_binding(&journal, &start_reference, &wrong_type),
        Err(EventRecordStructuralBindingError::RecordTypeMismatch)
    );

    let unknown_reference = JournalReference::new(
        start_reference.registry_id(),
        start_reference.entry_index(),
        start_reference.entry_hash(),
        start_reference.event_type_id(),
        EventRecordId::try_from(
            hex_id("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").as_slice(),
        )
        .unwrap(),
    );
    assert!(matches!(
        validate_event_record_structural_binding(&journal, &unknown_reference, &start_record),
        Err(EventRecordStructuralBindingError::RetainedReference(_))
    ));
}
