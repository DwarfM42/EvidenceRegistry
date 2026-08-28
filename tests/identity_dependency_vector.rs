use evidence_registry::{
    IdentityDependency, IdentityDependencyCollection, IdentityDependencyKind, JournalAnchorId,
    RecordId,
};

const ID_IDENTITY_DEPENDENCY_DERIVATION_001: &str = "ID-IDENTITY-DEPENDENCY-DERIVATION-001";
const EXPECTED_IDENTITY_DEPENDENCY_COLLECTION_CBOR_HEX: &str = "8282015820808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f82025820a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf";

fn lowercase_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// This fixed constructor deliberately does not call the production collection encoder.
/// It independently frames the Identity Format v0.3 §§60–63 canonical collection.
fn manually_construct_identity_dependency_collection(
    record_id: &[u8; 32],
    anchor_id: &[u8; 32],
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(73);
    bytes.push(0x82);
    bytes.extend_from_slice(&[0x82, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(record_id);
    bytes.extend_from_slice(&[0x82, 0x02, 0x58, 0x20]);
    bytes.extend_from_slice(anchor_id);
    bytes
}

#[test]
fn id_identity_dependency_derivation_001_matches_fixed_bytes_and_manual_reproduction() {
    let record_bytes = core::array::from_fn(|index| 0x80 + index as u8);
    let anchor_bytes = core::array::from_fn(|index| 0xa0 + index as u8);
    let record = RecordId::try_from(record_bytes.as_slice()).unwrap();
    let anchor = JournalAnchorId::try_from(anchor_bytes.as_slice()).unwrap();

    let collection = IdentityDependencyCollection::from_unordered_semantic_elements(vec![
        IdentityDependency::journal_anchor_id(anchor),
        IdentityDependency::record_id(record),
    ])
    .unwrap();
    let independently_constructed =
        manually_construct_identity_dependency_collection(&record_bytes, &anchor_bytes);

    assert_eq!(
        lowercase_hex(&independently_constructed),
        EXPECTED_IDENTITY_DEPENDENCY_COLLECTION_CBOR_HEX,
        "{ID_IDENTITY_DEPENDENCY_DERIVATION_001}"
    );
    assert_eq!(
        collection.authoritative_cbor(),
        independently_constructed.as_slice(),
        "{ID_IDENTITY_DEPENDENCY_DERIVATION_001}"
    );
}

#[test]
fn id_identity_dependency_order_changed_fail_001_is_rejected_without_normalization() {
    let record = RecordId::try_from([0x11u8; 32].as_slice()).unwrap();
    let anchor = JournalAnchorId::try_from([0x22u8; 32].as_slice()).unwrap();

    let result = IdentityDependencyCollection::from_authoritative_ordered_elements(vec![
        IdentityDependency::journal_anchor_id(anchor),
        IdentityDependency::record_id(record),
    ]);

    assert!(
        result.is_err(),
        "ID-IDENTITY-DEPENDENCY-ORDER-CHANGED-FAIL-001"
    );
}

#[test]
fn id_identity_dependency_duplicate_fail_001_is_rejected() {
    let record = RecordId::try_from([0x33u8; 32].as_slice()).unwrap();

    assert!(
        IdentityDependencyCollection::from_unordered_semantic_elements(vec![
            IdentityDependency::record_id(record),
            IdentityDependency::record_id(record),
        ])
        .is_err(),
        "ID-IDENTITY-DEPENDENCY-DUPLICATE-FAIL-001"
    );
}

#[test]
fn id_identity_dependency_reserved_raw_sha256_kind_fail_001_is_rejected() {
    assert!(IdentityDependencyKind::try_from(1u64).is_ok());
    assert!(IdentityDependencyKind::try_from(2u64).is_ok());
    assert!(IdentityDependencyKind::try_from(3u64).is_err());
    assert!(IdentityDependencyKind::try_from(0u64).is_err());
    assert!(IdentityDependencyKind::try_from(4u64).is_err());
}
