use evidence_registry::{
    validate_policy_recorded_structural_binding, EventRecordId, EventTypeId, GenesisJournalEntry,
    JournalEntryHash, JournalEntryIndex, JournalReference, PolicyRecordedStructuralBindingError,
    RecordId, RegistryId, RetainedJournal,
};
use sha2::{Digest, Sha256};

const GENESIS_HASH_HEX: &str = "ae1919549c80a0c3708cce0485af881a5223fbcf80c6b27a2bbe602418dabd51";
const POLICY_RECORD_ID_HEX: &str =
    "eae423d323fbfec851bb96ce4c274bdcb428b50793c72ea950e4775f1909d40f";
const POLICY_ENTRY_HASH_HEX: &str =
    "15efedb0363bf6de4902733300e9024b7711970e4b5007e3a5188e32d74ba8b5";
const POLICY_RECORD_HEX: &str = concat!(
    "84781a45766964656e636552656769737472792e5265636f72642e7631182801a50001011828105820",
    "808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f181d855820",
    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f005820",
    "ae1919549c80a0c3708cce0485af881a5223fbcf80c6b27a2bbe602418dabd51015820",
    "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f181e8101"
);
const POLICY_ENTRY_HEX: &str = concat!(
    "82782045766964656e636552656769737472792e4a6f75726e616c456e7472792e7631",
    "ac0001015820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
    "0201035820ae1919549c80a0c3708cce0485af881a5223fbcf80c6b27a2bbe602418dabd51",
    "04190190055820eae423d323fbfec851bb96ce4c274bdcb428b50793c72ea950e4775f1909d40f",
    "068007800807095820eae423d323fbfec851bb96ce4c274bdcb428b50793c72ea950e4775f1909d40f",
    "0a5820404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f",
    "0b5820606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f"
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

fn retained_policy_fixture() -> (RetainedJournal, JournalReference, Vec<u8>) {
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

    let policy_bytes = hex_bytes(POLICY_RECORD_HEX);
    assert_eq!(sha256(&policy_bytes), hex_id(POLICY_RECORD_ID_HEX));
    let policy_entry = hex_bytes(POLICY_ENTRY_HEX);
    assert_eq!(sha256(&policy_entry), hex_id(POLICY_ENTRY_HASH_HEX));

    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&policy_entry).unwrap();
    let policy_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(hex_id(POLICY_ENTRY_HASH_HEX).as_slice()).unwrap(),
        EventTypeId::try_from(400_u64).unwrap(),
        EventRecordId::try_from(hex_id(POLICY_RECORD_ID_HEX).as_slice()).unwrap(),
    );
    (journal, policy_reference, policy_bytes)
}

fn policy_entry_bytes(policy_record_id: [u8; 32]) -> Vec<u8> {
    let mut bytes = hex_bytes(
        "82782045766964656e636552656769737472792e4a6f75726e616c456e7472792e7631ac0001015820",
    );
    bytes.extend_from_slice(&hex_id(
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
    ));
    bytes.extend_from_slice(&[0x02, 0x01, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(&hex_id(GENESIS_HASH_HEX));
    bytes.extend_from_slice(&[0x04, 0x19, 0x01, 0x90, 0x05, 0x58, 0x20]);
    bytes.extend_from_slice(&policy_record_id);
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x80, 0x08, 0x07, 0x09, 0x58, 0x20]);
    bytes.extend_from_slice(&policy_record_id);
    bytes.extend_from_slice(&[0x0a, 0x58, 0x20]);
    bytes.extend_from_slice(&hex_id(
        "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f",
    ));
    bytes.extend_from_slice(&[0x0b, 0x58, 0x20]);
    bytes.extend_from_slice(&hex_id(
        "606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f",
    ));
    bytes
}

fn retained_policy_fixture_for(policy_bytes: Vec<u8>) -> (RetainedJournal, JournalReference) {
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
    let policy_record_id = sha256(&policy_bytes);
    let policy_entry = policy_entry_bytes(policy_record_id);
    let policy_entry_hash = sha256(&policy_entry);
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&policy_entry).unwrap();
    let policy_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(policy_entry_hash.as_slice()).unwrap(),
        EventTypeId::try_from(400_u64).unwrap(),
        EventRecordId::try_from(policy_record_id.as_slice()).unwrap(),
    );
    (journal, policy_reference)
}

#[test]
fn policy_recorded_structural_binding_retains_exact_policy_and_prior_operation_start_facts() {
    let (journal, policy_reference, policy_bytes) = retained_policy_fixture();

    let binding =
        validate_policy_recorded_structural_binding(&journal, &policy_reference, &policy_bytes)
            .unwrap();

    assert_eq!(
        binding.policy_record_id().as_bytes(),
        &hex_id(POLICY_RECORD_ID_HEX)
    );
    assert_eq!(binding.policy_event_reference(), &policy_reference);
    assert_eq!(binding.operation_start_reference().entry_index().value(), 0);
}

#[test]
fn policy_recorded_structural_binding_fails_closed_for_decode_event_and_identity_failures() {
    let (journal, policy_reference, policy_bytes) = retained_policy_fixture();

    assert_eq!(
        validate_policy_recorded_structural_binding(&journal, &policy_reference, &[0x80]),
        Err(PolicyRecordedStructuralBindingError::PolicyDecode)
    );

    let unknown_policy_reference = JournalReference::new(
        policy_reference.registry_id(),
        policy_reference.entry_index(),
        policy_reference.entry_hash(),
        policy_reference.event_type_id(),
        EventRecordId::try_from(
            hex_id("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").as_slice(),
        )
        .unwrap(),
    );
    assert!(matches!(
        validate_policy_recorded_structural_binding(
            &journal,
            &unknown_policy_reference,
            &policy_bytes
        ),
        Err(PolicyRecordedStructuralBindingError::PolicyEventReference(
            _
        ))
    ));

    let genesis_reference = JournalReference::new(
        policy_reference.registry_id(),
        JournalEntryIndex::try_from(0_u64).unwrap(),
        JournalEntryHash::try_from(hex_id(GENESIS_HASH_HEX).as_slice()).unwrap(),
        EventTypeId::try_from(1_u64).unwrap(),
        EventRecordId::try_from(
            hex_id("202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f").as_slice(),
        )
        .unwrap(),
    );
    assert_eq!(
        validate_policy_recorded_structural_binding(&journal, &genesis_reference, &policy_bytes),
        Err(PolicyRecordedStructuralBindingError::PolicyEventMismatch)
    );

    let mut wrong_policy_identity = policy_bytes;
    let scope_offset = wrong_policy_identity
        .windows(4)
        .position(|window| window == [0x10, 0x58, 0x20, 0x80])
        .unwrap();
    wrong_policy_identity[scope_offset + 3] = 0x81;
    assert_eq!(
        validate_policy_recorded_structural_binding(
            &journal,
            &policy_reference,
            &wrong_policy_identity,
        ),
        Err(PolicyRecordedStructuralBindingError::PolicyEventRecordIdentityMismatch)
    );
}

#[test]
fn policy_recorded_structural_binding_fails_closed_for_invalid_operation_start_reference() {
    let mut invalid_operation_start_policy = hex_bytes(POLICY_RECORD_HEX);
    let operation_start_hash_offset = invalid_operation_start_policy
        .windows(4)
        .position(|window| window == [0x58, 0x20, 0xae, 0x19])
        .unwrap();
    invalid_operation_start_policy[operation_start_hash_offset + 2] = 0xaf;
    let (journal, policy_reference) =
        retained_policy_fixture_for(invalid_operation_start_policy.clone());

    assert!(matches!(
        validate_policy_recorded_structural_binding(
            &journal,
            &policy_reference,
            &invalid_operation_start_policy,
        ),
        Err(PolicyRecordedStructuralBindingError::OperationStartReference(_))
    ));
}
