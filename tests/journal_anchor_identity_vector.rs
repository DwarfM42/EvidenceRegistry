use evidence_registry::{
    JournalAnchor, JournalEntryHash, JournalEntryIndex, RegistryId, ER_UINT_MAX,
};

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

/// Independent direct framing using already-encoded canonical UInt bytes. This
/// deliberately has no numeric encoding branch and so checks the production
/// encoder's width selection and big-endian output against literal CBOR bytes.
fn manually_construct_journal_anchor_with_uints(
    encoded_head_index: &[u8],
    encoded_format_version: &[u8],
) -> Vec<u8> {
    let registry: [u8; 32] = core::array::from_fn(|index| index as u8);
    let head_hash: [u8; 32] = core::array::from_fn(|index| 0x20 + index as u8);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x83, 0x78, 0x21]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalAnchor.v1");
    bytes.extend_from_slice(&[0x01, 0xa4, 0x00, 0x58, 0x20]);
    bytes.extend_from_slice(&registry);
    bytes.push(0x01);
    bytes.extend_from_slice(encoded_head_index);
    bytes.extend_from_slice(&[0x02, 0x58, 0x20]);
    bytes.extend_from_slice(&head_hash);
    bytes.push(0x03);
    bytes.extend_from_slice(encoded_format_version);
    bytes
}

fn journal_anchor_inputs() -> (RegistryId, JournalEntryHash) {
    let registry_bytes: [u8; 32] = core::array::from_fn(|index| index as u8);
    let head_hash_bytes: [u8; 32] = core::array::from_fn(|index| 0x20 + index as u8);
    (
        RegistryId::try_from(registry_bytes.as_slice()).unwrap(),
        JournalEntryHash::try_from(head_hash_bytes.as_slice()).unwrap(),
    )
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

#[test]
fn journal_anchor_uint_fields_use_shortest_canonical_encoding_at_boundaries() {
    let cases: &[(u64, &[u8])] = &[
        (0, &[0x00]),
        (23, &[0x17]),
        (24, &[0x18, 0x18]),
        (255, &[0x18, 0xff]),
        (256, &[0x19, 0x01, 0x00]),
        (65_535, &[0x19, 0xff, 0xff]),
        (65_536, &[0x1a, 0x00, 0x01, 0x00, 0x00]),
        (u32::MAX as u64, &[0x1a, 0xff, 0xff, 0xff, 0xff]),
        (
            u32::MAX as u64 + 1,
            &[0x1b, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
        ),
        (
            ER_UINT_MAX,
            &[0x1b, 0x00, 0x1f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
        ),
    ];

    for &(value, encoded_uint) in cases {
        let (registry_id, head_hash) = journal_anchor_inputs();
        let anchor = JournalAnchor::new(
            registry_id,
            JournalEntryIndex::try_from(value).unwrap(),
            head_hash,
            0,
        )
        .unwrap();
        assert_eq!(
            anchor.authoritative_cbor(),
            manually_construct_journal_anchor_with_uints(encoded_uint, &[0x00]),
            "journal_head_index={value}"
        );

        let (registry_id, head_hash) = journal_anchor_inputs();
        let anchor = JournalAnchor::new(
            registry_id,
            JournalEntryIndex::try_from(0_u64).unwrap(),
            head_hash,
            value,
        )
        .unwrap();
        assert_eq!(
            anchor.authoritative_cbor(),
            manually_construct_journal_anchor_with_uints(&[0x00], encoded_uint),
            "journal_format_version={value}"
        );
    }
}

#[test]
fn journal_anchor_rejects_uint_values_above_er_uint_max() {
    let too_large = ER_UINT_MAX + 1;
    assert!(JournalEntryIndex::try_from(too_large).is_err());

    let (registry_id, head_hash) = journal_anchor_inputs();
    assert!(JournalAnchor::new(
        registry_id,
        JournalEntryIndex::try_from(0_u64).unwrap(),
        head_hash,
        too_large,
    )
    .is_err());
}
