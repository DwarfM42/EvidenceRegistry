use evidence_registry::{JournalAnchor, JournalEntryHash, JournalEntryIndex, RegistryId};

const EXPECTED_JOURNAL_ANCHOR_HEX: &str = "83782145766964656e636552656769737472792e4a6f75726e616c416e63686f722e763101a4005820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f011818025820202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f0301";
const EXPECTED_JOURNAL_ANCHOR_ID_HEX: &str =
    "0a4468788d18e5f2aa9e95a8417a452e19b99dc94170a99268e041c10033391c";

fn lowercase_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Independent direct framing of Identity Format v0.3 §§93–96 for the fixed
/// input below; it does not call the production Journal Anchor encoder.
fn manually_construct_journal_anchor() -> Vec<u8> {
    let registry: [u8; 32] = core::array::from_fn(|index| index as u8);
    let head_hash: [u8; 32] = core::array::from_fn(|index| 0x20 + index as u8);
    let mut bytes = Vec::with_capacity(113);
    bytes.extend_from_slice(&[0x83, 0x78, 0x21]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalAnchor.v1");
    bytes.extend_from_slice(&[0x01, 0xa4, 0x00, 0x58, 0x20]);
    bytes.extend_from_slice(&registry);
    bytes.extend_from_slice(&[0x01, 0x18, 0x18, 0x02, 0x58, 0x20]);
    bytes.extend_from_slice(&head_hash);
    bytes.extend_from_slice(&[0x03, 0x01]);
    bytes
}

#[test]
fn journal_anchor_derivation_001_matches_fixed_bytes_and_independent_framing() {
    let registry_bytes: [u8; 32] = core::array::from_fn(|index| index as u8);
    let head_hash_bytes: [u8; 32] = core::array::from_fn(|index| 0x20 + index as u8);
    let registry = RegistryId::try_from(registry_bytes.as_slice()).unwrap();
    let head_index = JournalEntryIndex::try_from(24_u64).unwrap();
    let head_hash = JournalEntryHash::try_from(head_hash_bytes.as_slice()).unwrap();

    let anchor = JournalAnchor::new(registry, head_index, head_hash, 1).unwrap();
    let independently_constructed = manually_construct_journal_anchor();

    assert_eq!(
        lowercase_hex(&independently_constructed),
        EXPECTED_JOURNAL_ANCHOR_HEX
    );
    assert_eq!(anchor.authoritative_cbor(), independently_constructed);
    assert_eq!(
        lowercase_hex(anchor.anchor_id().as_bytes()),
        EXPECTED_JOURNAL_ANCHOR_ID_HEX
    );
}
