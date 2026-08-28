use evidence_registry::{
    AuthorityDependencyCollection, AuthorityDependencyContext, EventRecordId, EventTypeId,
    GenesisJournalEntry, IdentityDependencyCollection, JournalEntryHash, JournalEntryIndex,
    JournalReference, OrdinaryVerificationJournalEntry, OrdinaryVerificationJournalEntryInput,
    RecordId, RegistryId,
};

const VECTOR_ID: &str = "JRN-VERIFICATION-ORDINARY-001";
const EXPECTED_ORDINARY_VERIFICATION_CBOR_HEX: &str = "82782045766964656e636552656769737472792e4a6f75726e616c456e7472792e7631ac0001015820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f021818035820202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f0418c8055820404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f06800781855820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f085820606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f18655820808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f0803095820404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f0a5820a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf0b5820c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf";
const EXPECTED_ORDINARY_VERIFICATION_HASH_HEX: &str =
    "fc0b5f650cb67481b6fde6bf6b9909692da1ebafbc758ed3580b6e7a45913cde";

fn lowercase_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn id(range_start: u8) -> [u8; 32] {
    core::array::from_fn(|index| range_start + index as u8)
}

/// Direct literal framing of the frozen Journal Entry grammar. This routine does
/// not call any EvidenceRegistry production encoder.
fn independently_construct_ordinary_verification() -> Vec<u8> {
    let registry = id(0x00);
    let previous_hash = id(0x20);
    let event_record = id(0x40);
    let freeze_entry_hash = id(0x60);
    let freeze_event_record = id(0x80);
    let storage_capability = id(0xa0);
    let environment = id(0xc0);

    let mut bytes = Vec::with_capacity(331);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xac);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01, 0x58, 0x20]);
    bytes.extend_from_slice(&registry);
    bytes.extend_from_slice(&[0x02, 0x18, 0x18, 0x03, 0x58, 0x20]);
    bytes.extend_from_slice(&previous_hash);
    bytes.extend_from_slice(&[0x04, 0x18, 0xc8, 0x05, 0x58, 0x20]);
    bytes.extend_from_slice(&event_record);
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x81, 0x85, 0x58, 0x20]);
    bytes.extend_from_slice(&registry);
    bytes.extend_from_slice(&[0x08, 0x58, 0x20]);
    bytes.extend_from_slice(&freeze_entry_hash);
    bytes.extend_from_slice(&[0x18, 0x65, 0x58, 0x20]);
    bytes.extend_from_slice(&freeze_event_record);
    bytes.extend_from_slice(&[0x08, 0x03, 0x09, 0x58, 0x20]);
    bytes.extend_from_slice(&event_record);
    bytes.extend_from_slice(&[0x0a, 0x58, 0x20]);
    bytes.extend_from_slice(&storage_capability);
    bytes.extend_from_slice(&[0x0b, 0x58, 0x20]);
    bytes.extend_from_slice(&environment);
    bytes
}

fn ordinary_verification_inputs() -> (
    RegistryId,
    JournalEntryIndex,
    JournalEntryHash,
    EventRecordId,
    IdentityDependencyCollection,
    AuthorityDependencyCollection,
    RecordId,
    RecordId,
) {
    let registry = RegistryId::try_from(id(0x00).as_slice()).unwrap();
    let entry_index = JournalEntryIndex::try_from(24_u64).unwrap();
    let previous_hash = JournalEntryHash::try_from(id(0x20).as_slice()).unwrap();
    let event_record = EventRecordId::try_from(id(0x40).as_slice()).unwrap();
    let identity_dependencies =
        IdentityDependencyCollection::from_unordered_semantic_elements(vec![]).unwrap();
    let freeze_reference = JournalReference::new(
        registry,
        JournalEntryIndex::try_from(8_u64).unwrap(),
        JournalEntryHash::try_from(id(0x60).as_slice()).unwrap(),
        EventTypeId::try_from(101_u64).unwrap(),
        EventRecordId::try_from(id(0x80).as_slice()).unwrap(),
    );
    let authority_dependencies = AuthorityDependencyCollection::from_unordered_semantic_elements(
        AuthorityDependencyContext::new(registry, entry_index),
        vec![freeze_reference],
    )
    .unwrap();
    let storage_capability = RecordId::try_from(id(0xa0).as_slice()).unwrap();
    let environment = RecordId::try_from(id(0xc0).as_slice()).unwrap();

    (
        registry,
        entry_index,
        previous_hash,
        event_record,
        identity_dependencies,
        authority_dependencies,
        storage_capability,
        environment,
    )
}

#[test]
fn ordinary_verification_vector_matches_independent_bytes_hash_and_strict_round_trip() {
    let (
        registry,
        entry_index,
        previous_hash,
        event_record,
        identity_dependencies,
        authority_dependencies,
        storage_capability,
        environment,
    ) = ordinary_verification_inputs();
    let independently_constructed = independently_construct_ordinary_verification();

    assert_eq!(
        lowercase_hex(&independently_constructed),
        EXPECTED_ORDINARY_VERIFICATION_CBOR_HEX,
        "{VECTOR_ID} independent literal framing"
    );

    let entry = OrdinaryVerificationJournalEntry::new(OrdinaryVerificationJournalEntryInput {
        registry_id: registry,
        entry_index,
        previous_entry_hash: previous_hash,
        event_record_id: event_record,
        identity_dependencies,
        authority_dependencies,
        storage_capability_class_id: storage_capability,
        environment_observation_id: environment,
    })
    .unwrap();
    assert_eq!(
        entry.authoritative_cbor(),
        independently_constructed,
        "{VECTOR_ID}"
    );
    assert_eq!(
        lowercase_hex(entry.entry_hash().as_bytes()),
        EXPECTED_ORDINARY_VERIFICATION_HASH_HEX,
        "{VECTOR_ID} independent SHA-256 expectation"
    );

    let decoded =
        OrdinaryVerificationJournalEntry::decode_authoritative(&independently_constructed)
            .expect("exact canonical ordinary Verification bytes must strictly decode");
    assert_eq!(
        decoded.authoritative_cbor(),
        independently_constructed,
        "{VECTOR_ID}"
    );
}

#[test]
fn journal_entry_strict_decoders_reject_truncation_wrong_width_wrong_type_and_nonminimal_uint() {
    let bytes = independently_construct_ordinary_verification();

    assert!(
        OrdinaryVerificationJournalEntry::decode_authoritative(&bytes[..bytes.len() - 1]).is_err(),
        "JRN-NEG-ORDINARY-TRUNCATED-001"
    );

    let mut wrong_width = bytes.clone();
    let registry_header = wrong_width
        .windows(3)
        .position(|window| window == [0x01, 0x58, 0x20])
        .unwrap()
        + 1;
    wrong_width[registry_header + 1] = 0x1f;
    assert!(
        OrdinaryVerificationJournalEntry::decode_authoritative(&wrong_width).is_err(),
        "JRN-NEG-ORDINARY-WRONG-WIDTH-001"
    );

    let mut wrong_type = bytes.clone();
    let previous_hash_header = wrong_type
        .windows(3)
        .position(|window| window == [0x03, 0x58, 0x20])
        .unwrap()
        + 1;
    wrong_type[previous_hash_header] = 0xf6;
    assert!(
        OrdinaryVerificationJournalEntry::decode_authoritative(&wrong_type).is_err(),
        "JRN-NEG-ORDINARY-WRONG-TYPE-001"
    );

    let mut nonminimal_index = bytes.clone();
    let canonical_index = nonminimal_index
        .windows(3)
        .position(|window| window == [0x02, 0x18, 0x18])
        .unwrap();
    nonminimal_index.splice(canonical_index + 1..canonical_index + 3, [0x19, 0x00, 0x18]);
    assert!(
        OrdinaryVerificationJournalEntry::decode_authoritative(&nonminimal_index).is_err(),
        "JRN-NEG-ORDINARY-NONMINIMAL-UINT-001"
    );
}

#[test]
fn genesis_strict_decoder_round_trips_only_exact_canonical_bytes() {
    let registry = RegistryId::try_from(id(0x00).as_slice()).unwrap();
    let event_record = EventRecordId::try_from(id(0x20).as_slice()).unwrap();
    let storage_capability = RecordId::try_from(id(0x40).as_slice()).unwrap();
    let environment = RecordId::try_from(id(0x60).as_slice()).unwrap();
    let genesis = GenesisJournalEntry::new(registry, event_record, storage_capability, environment);
    let bytes = genesis.authoritative_cbor();

    let decoded = GenesisJournalEntry::decode_authoritative(&bytes)
        .expect("canonical GENESIS bytes must strictly decode");
    assert_eq!(
        decoded.authoritative_cbor(),
        bytes,
        "JRN-GENESIS-STRICT-001"
    );

    let mut with_reserved_key = bytes;
    let map_header = with_reserved_key
        .iter()
        .position(|byte| *byte == 0xac)
        .unwrap();
    with_reserved_key[map_header] = 0xad;
    with_reserved_key.push(0x0c);
    with_reserved_key.push(0x00);
    assert!(
        GenesisJournalEntry::decode_authoritative(&with_reserved_key).is_err(),
        "JRN-NEG-GENESIS-RESERVED-KEY-001"
    );
}

#[test]
fn ordinary_verification_rejects_a_collection_without_the_structural_freeze_event() {
    let (
        registry,
        entry_index,
        previous_hash,
        event_record,
        identity_dependencies,
        _,
        storage_capability,
        environment,
    ) = ordinary_verification_inputs();
    let no_authority_dependencies =
        AuthorityDependencyCollection::from_unordered_semantic_elements(
            AuthorityDependencyContext::new(registry, entry_index),
            vec![],
        )
        .unwrap();

    assert!(
        OrdinaryVerificationJournalEntry::new(OrdinaryVerificationJournalEntryInput {
            registry_id: registry,
            entry_index,
            previous_entry_hash: previous_hash,
            event_record_id: event_record,
            identity_dependencies,
            authority_dependencies: no_authority_dependencies,
            storage_capability_class_id: storage_capability,
            environment_observation_id: environment,
        })
        .is_err(),
        "JRN-NEG-ORDINARY-MISSING-FREEZE-DEPENDENCY-001"
    );
}

#[test]
fn ordinary_verification_fixture_retains_scope_and_independent_reconstruction_metadata() {
    let fixture = include_str!("../vectors/journal-ordinary-verification-v1.txt");
    for required_line in [
        "vector_id=JRN-VERIFICATION-ORDINARY-001",
        "canonical_cbor_hex=827820",
        "entry_hash_sha256=fc0b5f650cb67481b6fde6bf6b9909692da1ebafbc758ed3580b6e7a45913cde",
        "independent_reconstruction=",
        "non_authority_claim=",
        "JRN-NEG-ORDINARY-NONMINIMAL-UINT-001",
    ] {
        assert!(
            fixture.contains(required_line),
            "missing fixture field: {required_line}"
        );
    }
}

#[test]
fn ordinary_verification_strict_decoder_accepts_compact_canonical_authority_dependencies() {
    let registry = RegistryId::try_from(id(0x00).as_slice()).unwrap();
    let entry_index = JournalEntryIndex::try_from(87_u64).unwrap();
    let authority_dependencies = AuthorityDependencyCollection::from_unordered_semantic_elements(
        AuthorityDependencyContext::new(registry, entry_index),
        (1_u64..=86)
            .map(|reference_index| {
                JournalReference::new(
                    registry,
                    JournalEntryIndex::try_from(reference_index).unwrap(),
                    JournalEntryHash::try_from([reference_index as u8; 32].as_slice()).unwrap(),
                    EventTypeId::try_from(if reference_index == 86 { 101 } else { 1 }).unwrap(),
                    EventRecordId::try_from([reference_index as u8; 32].as_slice()).unwrap(),
                )
            })
            .collect(),
    )
    .expect("canonical compact references are structurally valid without resolution");
    let entry = OrdinaryVerificationJournalEntry::new(OrdinaryVerificationJournalEntryInput {
        registry_id: registry,
        entry_index,
        previous_entry_hash: JournalEntryHash::try_from(id(0x20).as_slice()).unwrap(),
        event_record_id: EventRecordId::try_from(id(0x40).as_slice()).unwrap(),
        identity_dependencies: IdentityDependencyCollection::from_unordered_semantic_elements(
            vec![],
        )
        .unwrap(),
        authority_dependencies,
        storage_capability_class_id: RecordId::try_from(id(0xa0).as_slice()).unwrap(),
        environment_observation_id: RecordId::try_from(id(0xc0).as_slice()).unwrap(),
    })
    .expect("a compact structural dependency array is a valid ordinary Verification entry");
    let bytes = entry.authoritative_cbor();
    let decoded = OrdinaryVerificationJournalEntry::decode_authoritative(&bytes)
        .expect("JRN-REGRESSION-COMPACT-AUTHORITY-DEPS-001");
    assert_eq!(decoded.authoritative_cbor(), bytes);
}
