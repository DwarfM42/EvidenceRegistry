use evidence_registry::{
    EventRecordId, EventTypeId, JournalEntryHash, JournalEntryIndex, JournalReference, RegistryId,
};

const ID_JOURNAL_REFERENCE_DERIVATION_001: &str = "ID-JOURNAL-REFERENCE-DERIVATION-001";
const EXPECTED_JOURNAL_REFERENCE_CBOR_HEX: &str = "855820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f18645820202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f18645820404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f";

fn lowercase_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// This fixed constructor deliberately does not call the production encoder.
/// It independently frames the Identity Format v0.3 §51 five-element tuple.
fn manually_construct_journal_reference_bytes(
    registry_id: &[u8; 32],
    entry_hash: &[u8; 32],
    event_record_id: &[u8; 32],
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(107);
    bytes.push(0x85);
    bytes.extend_from_slice(&[0x58, 0x20]);
    bytes.extend_from_slice(registry_id);
    bytes.extend_from_slice(&[0x18, 0x64]);
    bytes.extend_from_slice(&[0x58, 0x20]);
    bytes.extend_from_slice(entry_hash);
    bytes.extend_from_slice(&[0x18, 0x64]);
    bytes.extend_from_slice(&[0x58, 0x20]);
    bytes.extend_from_slice(event_record_id);
    bytes
}

#[test]
fn id_journal_reference_derivation_001_matches_fixed_bytes_and_manual_reproduction() {
    let registry_bytes = core::array::from_fn(|index| index as u8);
    let entry_hash_bytes = core::array::from_fn(|index| 0x20 + index as u8);
    let event_record_bytes = core::array::from_fn(|index| 0x40 + index as u8);

    let journal_reference = JournalReference::new(
        RegistryId::try_from(registry_bytes.as_slice()).unwrap(),
        JournalEntryIndex::try_from(100u64).unwrap(),
        JournalEntryHash::try_from(entry_hash_bytes.as_slice()).unwrap(),
        EventTypeId::try_from(100u64).unwrap(),
        EventRecordId::try_from(event_record_bytes.as_slice()).unwrap(),
    );

    let independently_constructed = manually_construct_journal_reference_bytes(
        &registry_bytes,
        &entry_hash_bytes,
        &event_record_bytes,
    );

    assert_eq!(
        lowercase_hex(&independently_constructed),
        EXPECTED_JOURNAL_REFERENCE_CBOR_HEX,
        "{ID_JOURNAL_REFERENCE_DERIVATION_001}"
    );
    assert_eq!(
        journal_reference.authoritative_cbor(),
        independently_constructed.as_slice(),
        "{ID_JOURNAL_REFERENCE_DERIVATION_001}"
    );
}

#[test]
fn id_journal_reference_out_of_range_index_fail_001_is_rejected_before_encoding() {
    assert!(
        JournalEntryIndex::try_from(evidence_registry::ER_UINT_MAX + 1).is_err(),
        "ID-JOURNAL-REFERENCE-INDEX-OUT-OF-RANGE-FAIL-001"
    );
}

#[test]
fn id_journal_reference_unregistered_event_fail_001_is_rejected_before_encoding() {
    assert!(
        EventTypeId::try_from(2u64).is_err(),
        "ID-JOURNAL-REFERENCE-UNREGISTERED-EVENT-FAIL-001"
    );
}

#[test]
fn id_journal_reference_current_v0x_event_registry_001_accepts_exactly_the_28_registered_values() {
    let registered = [
        1u64, 100, 101, 102, 103, 104, 200, 300, 301, 302, 303, 400, 500, 501, 600, 700, 701, 702,
        703, 800, 801, 802, 803, 804, 805, 806, 807, 808,
    ];
    assert_eq!(registered.len(), 28);
    assert!(registered
        .into_iter()
        .all(|event_type| EventTypeId::try_from(event_type).is_ok()));
    assert!(EventTypeId::try_from(809u64).is_err());
}

#[test]
fn id_journal_reference_bstr_length_fail_001_is_rejected_before_encoding() {
    assert!(
        JournalEntryHash::try_from([0u8; 31].as_slice()).is_err()
            && EventRecordId::try_from([0u8; 33].as_slice()).is_err(),
        "ID-JOURNAL-REFERENCE-BSTR-LENGTH-FAIL-001"
    );
}
