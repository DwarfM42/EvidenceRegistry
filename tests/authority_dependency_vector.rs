use evidence_registry::{
    AuthorityDependencyCollection, AuthorityDependencyContext, EventRecordId, EventTypeId,
    JournalEntryHash, JournalEntryIndex, JournalReference, RegistryId,
};

const ID_AUTHORITY_DEPENDENCY_DERIVATION_001: &str = "ID-AUTHORITY-DEPENDENCY-DERIVATION-001";
const EXPECTED_AUTHORITY_DEPENDENCY_COLLECTION_CBOR_HEX: &str = "82855820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f015820606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f015820808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f855820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f18645820202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f18645820404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f";

fn lowercase_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn journal_reference(
    index: u64,
    entry_hash: [u8; 32],
    event_type: u64,
    event_record: [u8; 32],
) -> JournalReference {
    let registry_bytes: [u8; 32] = core::array::from_fn(|value| value as u8);
    JournalReference::new(
        RegistryId::try_from(registry_bytes.as_slice()).unwrap(),
        JournalEntryIndex::try_from(index).unwrap(),
        JournalEntryHash::try_from(entry_hash.as_slice()).unwrap(),
        EventTypeId::try_from(event_type).unwrap(),
        EventRecordId::try_from(event_record.as_slice()).unwrap(),
    )
}

fn authority_context(containing_entry_index: u64) -> AuthorityDependencyContext {
    let registry_bytes: [u8; 32] = core::array::from_fn(|value| value as u8);
    AuthorityDependencyContext::new(
        RegistryId::try_from(registry_bytes.as_slice()).unwrap(),
        JournalEntryIndex::try_from(containing_entry_index).unwrap(),
    )
}

/// This directly frames the two fixed JournalReference values; it does not use
/// the production collection serializer or comparator.
fn manually_construct_authority_dependency_collection() -> Vec<u8> {
    let registry: [u8; 32] = core::array::from_fn(|index| index as u8);
    let mut bytes = Vec::with_capacity(213);
    bytes.push(0x82);
    bytes.push(0x85);
    bytes.extend_from_slice(&[0x58, 0x20]);
    bytes.extend_from_slice(&registry);
    bytes.extend_from_slice(&[0x01, 0x58, 0x20]);
    bytes.extend(0x60..=0x7f);
    bytes.extend_from_slice(&[0x01, 0x58, 0x20]);
    bytes.extend(0x80..=0x9f);
    bytes.push(0x85);
    bytes.extend_from_slice(&[0x58, 0x20]);
    bytes.extend_from_slice(&registry);
    bytes.extend_from_slice(&[0x18, 0x64, 0x58, 0x20]);
    bytes.extend(0x20..=0x3f);
    bytes.extend_from_slice(&[0x18, 0x64, 0x58, 0x20]);
    bytes.extend(0x40..=0x5f);
    bytes
}

#[test]
fn id_authority_dependency_derivation_001_matches_fixed_bytes_and_manual_reproduction() {
    let later = journal_reference(
        100,
        core::array::from_fn(|index| 0x20 + index as u8),
        100,
        core::array::from_fn(|index| 0x40 + index as u8),
    );
    let earlier = journal_reference(
        1,
        core::array::from_fn(|index| 0x60 + index as u8),
        1,
        core::array::from_fn(|index| 0x80 + index as u8),
    );
    let collection = AuthorityDependencyCollection::from_unordered_semantic_elements(
        authority_context(101),
        vec![later, earlier],
    )
    .unwrap();
    let independently_constructed = manually_construct_authority_dependency_collection();

    assert_eq!(
        lowercase_hex(&independently_constructed),
        EXPECTED_AUTHORITY_DEPENDENCY_COLLECTION_CBOR_HEX,
        "{ID_AUTHORITY_DEPENDENCY_DERIVATION_001}"
    );
    assert_eq!(
        collection.authoritative_cbor(),
        independently_constructed.as_slice(),
        "{ID_AUTHORITY_DEPENDENCY_DERIVATION_001}"
    );
}

#[test]
fn id_authority_dependency_same_index_conflict_fail_001_is_rejected() {
    let first = journal_reference(7, [0x11; 32], 100, [0x21; 32]);
    let conflicting = journal_reference(7, [0x12; 32], 101, [0x22; 32]);

    assert!(
        AuthorityDependencyCollection::from_unordered_semantic_elements(
            authority_context(8),
            vec![first, conflicting],
        )
        .is_err(),
        "ID-AUTHORITY-DEPENDENCY-SAME-INDEX-CONFLICT-FAIL-001"
    );
}

#[test]
fn id_authority_dependency_forward_and_cross_registry_fail_001_are_rejected_before_encoding() {
    let forward = journal_reference(10, [0x31; 32], 100, [0x41; 32]);
    assert!(
        AuthorityDependencyCollection::from_unordered_semantic_elements(
            authority_context(10),
            vec![forward],
        )
        .is_err(),
        "ID-AUTHORITY-DEPENDENCY-FORWARD-FAIL-001"
    );

    let other_registry = RegistryId::try_from([0xffu8; 32].as_slice()).unwrap();
    let cross_registry = JournalReference::new(
        other_registry,
        JournalEntryIndex::try_from(1u64).unwrap(),
        JournalEntryHash::try_from([0x51u8; 32].as_slice()).unwrap(),
        EventTypeId::try_from(1u64).unwrap(),
        EventRecordId::try_from([0x61u8; 32].as_slice()).unwrap(),
    );
    assert!(
        AuthorityDependencyCollection::from_unordered_semantic_elements(
            authority_context(10),
            vec![cross_registry],
        )
        .is_err(),
        "ID-AUTHORITY-DEPENDENCY-CROSS-REGISTRY-FAIL-001"
    );
}

#[test]
fn id_authority_dependency_order_changed_fail_001_is_rejected_without_normalization() {
    let later = journal_reference(4, [0x71; 32], 100, [0x81; 32]);
    let earlier = journal_reference(1, [0x72; 32], 1, [0x82; 32]);

    assert!(
        AuthorityDependencyCollection::from_authoritative_ordered_elements(
            authority_context(5),
            vec![later, earlier],
        )
        .is_err(),
        "ID-AUTHORITY-DEPENDENCY-ORDER-CHANGED-FAIL-001"
    );
}
