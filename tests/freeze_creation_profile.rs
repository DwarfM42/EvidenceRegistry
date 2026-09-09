use evidence_registry::{FreezeCreationProfileRecord, RecordDecodeError};

fn exact_profile_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[
        0x18, 0x40, 0x01, 0xa3, 0x00, 0x01, 0x01, 0x18, 0x40, 0x10, 0x01,
    ]);
    bytes
}

#[test]
fn freeze_creation_profile_has_only_the_exact_assigned_type_version_and_profile() {
    let expected = exact_profile_bytes();

    let profile = FreezeCreationProfileRecord::new().unwrap();
    assert_eq!(profile.authoritative_cbor(), expected);
    assert_eq!(
        FreezeCreationProfileRecord::decode_authoritative(&expected).unwrap(),
        profile
    );

    let mut wrong_profile = expected;
    *wrong_profile.last_mut().unwrap() = 2;
    assert_eq!(
        FreezeCreationProfileRecord::decode_authoritative(&wrong_profile),
        Err(RecordDecodeError)
    );
}
