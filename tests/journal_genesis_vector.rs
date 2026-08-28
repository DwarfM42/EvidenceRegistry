use evidence_registry::{EventRecordId, GenesisJournalEntry, RecordId, RegistryId};

const EXPECTED_GENESIS_JOURNAL_ENTRY_HEX: &str = "82782045766964656e636552656769737472792e4a6f75726e616c456e7472792e7631ac0001015820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f020003f60401055820202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f068007800801095820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f0a5820404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f0b5820606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f";
const EXPECTED_GENESIS_ENTRY_HASH_HEX: &str =
    "ae1919549c80a0c3708cce0485af881a5223fbcf80c6b27a2bbe602418dabd51";

fn lowercase_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Independent direct framing of Identity Format v0.3 §§66–73 for the fixed
/// GENESIS input below; it does not call the production Journal Entry encoder.
fn manually_construct_genesis_journal_entry() -> Vec<u8> {
    let registry: [u8; 32] = core::array::from_fn(|index| index as u8);
    let event_record: [u8; 32] = core::array::from_fn(|index| 0x20 + index as u8);
    let storage_capability: [u8; 32] = core::array::from_fn(|index| 0x40 + index as u8);
    let environment: [u8; 32] = core::array::from_fn(|index| 0x60 + index as u8);
    let mut bytes = Vec::with_capacity(225);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xac);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&registry);
    bytes.extend_from_slice(&[0x02, 0x00, 0x03, 0xf6, 0x04, 0x01, 0x05, 0x58, 0x20]);
    bytes.extend_from_slice(&event_record);
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x80, 0x08, 0x01, 0x09, 0x58, 0x20]);
    bytes.extend_from_slice(&registry);
    bytes.extend_from_slice(&[0x0a, 0x58, 0x20]);
    bytes.extend_from_slice(&storage_capability);
    bytes.extend_from_slice(&[0x0b, 0x58, 0x20]);
    bytes.extend_from_slice(&environment);
    bytes
}

#[test]
fn journal_genesis_derivation_001_matches_fixed_bytes_and_independent_framing() {
    let registry_bytes: [u8; 32] = core::array::from_fn(|index| index as u8);
    let event_record_bytes: [u8; 32] = core::array::from_fn(|index| 0x20 + index as u8);
    let storage_capability_bytes: [u8; 32] = core::array::from_fn(|index| 0x40 + index as u8);
    let environment_bytes: [u8; 32] = core::array::from_fn(|index| 0x60 + index as u8);
    let registry = RegistryId::try_from(registry_bytes.as_slice()).unwrap();
    let event_record = EventRecordId::try_from(event_record_bytes.as_slice()).unwrap();
    let storage_capability = RecordId::try_from(storage_capability_bytes.as_slice()).unwrap();
    let environment = RecordId::try_from(environment_bytes.as_slice()).unwrap();

    let entry = GenesisJournalEntry::new(registry, event_record, storage_capability, environment);
    let independently_constructed = manually_construct_genesis_journal_entry();

    assert_eq!(
        lowercase_hex(&independently_constructed),
        EXPECTED_GENESIS_JOURNAL_ENTRY_HEX
    );
    assert_eq!(entry.authoritative_cbor(), independently_constructed);
    assert_eq!(
        lowercase_hex(entry.entry_hash().as_bytes()),
        EXPECTED_GENESIS_ENTRY_HASH_HEX
    );
}
