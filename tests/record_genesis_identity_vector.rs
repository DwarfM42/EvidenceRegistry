use evidence_registry::{GenesisRecord, GenesisRecordInput, RecordId, RegistryId};

const VECTOR_ID: &str = "REC-GENESIS-IDENTITY-001";
const EXPECTED_GENESIS_RECORD_CBOR_HEX: &str = "84781a45766964656e636552656769737472792e5265636f72642e76310101a800010101105820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f11011201135820404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f145820606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f15781865766964656e63652d72656769737472792d746573742f31";
const EXPECTED_GENESIS_RECORD_ID_HEX: &str =
    "e3d7161b0fa9ade3963b4b3771f586f3b3d045bc479e86b30cfef859e14c6d1c";

fn lowercase_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Direct literal framing of Record Schema v0.3 §51 and Identity Format v0.3
/// §§33–36. It does not invoke an EvidenceRegistry Record encoder.
fn independently_construct_genesis_record() -> Vec<u8> {
    let registry: [u8; 32] = core::array::from_fn(|index| index as u8);
    let storage_capability: [u8; 32] = core::array::from_fn(|index| 0x40 + index as u8);
    let environment: [u8; 32] = core::array::from_fn(|index| 0x60 + index as u8);
    let mut bytes = Vec::with_capacity(180);
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x01, 0x01, 0xa8, 0x00, 0x01, 0x01, 0x01, 0x10, 0x58, 0x20]);
    bytes.extend_from_slice(&registry);
    bytes.extend_from_slice(&[0x11, 0x01, 0x12, 0x01, 0x13, 0x58, 0x20]);
    bytes.extend_from_slice(&storage_capability);
    bytes.extend_from_slice(&[0x14, 0x58, 0x20]);
    bytes.extend_from_slice(&environment);
    bytes.extend_from_slice(&[0x15, 0x78, 0x18]);
    bytes.extend_from_slice(b"evidence-registry-test/1");
    bytes
}

#[test]
fn genesis_record_vector_matches_independent_bytes_hash_and_strict_round_trip() {
    let registry_bytes: [u8; 32] = core::array::from_fn(|index| index as u8);
    let storage_capability_bytes: [u8; 32] = core::array::from_fn(|index| 0x40 + index as u8);
    let environment_bytes: [u8; 32] = core::array::from_fn(|index| 0x60 + index as u8);
    let input = GenesisRecordInput {
        registry_id: RegistryId::try_from(registry_bytes.as_slice()).unwrap(),
        journal_format_version: 1,
        record_identity_profile_id: 1,
        storage_capability_class_id: RecordId::try_from(storage_capability_bytes.as_slice())
            .unwrap(),
        environment_observation_id: RecordId::try_from(environment_bytes.as_slice()).unwrap(),
        created_by_tool_version: "evidence-registry-test/1".to_owned(),
    };
    let independently_constructed = independently_construct_genesis_record();
    assert_eq!(
        lowercase_hex(&independently_constructed),
        EXPECTED_GENESIS_RECORD_CBOR_HEX,
        "{VECTOR_ID} independent literal framing"
    );

    let record = GenesisRecord::new(input).unwrap();
    assert_eq!(
        record.authoritative_cbor(),
        independently_constructed,
        "{VECTOR_ID}"
    );
    assert_eq!(
        lowercase_hex(record.record_id().as_bytes()),
        EXPECTED_GENESIS_RECORD_ID_HEX,
        "{VECTOR_ID} independent SHA-256 expectation"
    );

    let decoded = GenesisRecord::decode_authoritative(&independently_constructed)
        .expect("canonical GENESIS Record bytes must strictly decode");
    assert_eq!(
        decoded.authoritative_cbor(),
        independently_constructed,
        "{VECTOR_ID}"
    );
}

#[test]
fn genesis_record_strict_decoder_rejects_noncanonical_reserved_and_wrong_framing_bytes() {
    let bytes = independently_construct_genesis_record();
    assert!(
        GenesisRecord::decode_authoritative(&bytes[..bytes.len() - 1]).is_err(),
        "REC-NEG-GENESIS-TRUNCATED-001"
    );

    let mut nonminimal_type = bytes.clone();
    let type_index = nonminimal_type
        .windows(3)
        .position(|window| window == [0x76, 0x31, 0x01])
        .unwrap()
        + 2;
    nonminimal_type.splice(type_index..type_index + 1, [0x18, 0x01]);
    assert!(
        GenesisRecord::decode_authoritative(&nonminimal_type).is_err(),
        "REC-NEG-GENESIS-NONMINIMAL-001"
    );

    let mut reserved_key = bytes;
    let map_index = reserved_key.iter().position(|byte| *byte == 0xa8).unwrap();
    reserved_key[map_index] = 0xa9;
    reserved_key.push(0x02);
    reserved_key.push(0x00);
    assert!(
        GenesisRecord::decode_authoritative(&reserved_key).is_err(),
        "REC-NEG-GENESIS-RESERVED-KEY-001"
    );
}

#[test]
fn genesis_record_fixture_retains_identity_scope_and_independent_reconstruction_metadata() {
    let fixture = include_str!("../vectors/record-genesis-v1.txt");
    for required_line in [
        "vector_id=REC-GENESIS-IDENTITY-001",
        "canonical_cbor_hex=84781a",
        "record_id_sha256=e3d7161b0fa9ade3963b4b3771f586f3b3d045bc479e86b30cfef859e14c6d1c",
        "independent_reconstruction=",
        "non_authority_claim=",
        "REC-NEG-GENESIS-NONMINIMAL-001",
    ] {
        assert!(
            fixture.contains(required_line),
            "missing fixture field: {required_line}"
        );
    }
}
