use evidence_registry::{derive_freeze_root, FreezeAttemptId, RegistryId};

const DOMAIN: &str = "EvidenceRegistry.FreezeRoot.v1";
const EXPECTED_CBOR_HEX: &str = "83781e45766964656e636552656769737472792e467265657a65526f6f742e76315820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f5820807ffefdfcfbfaf9f8f7f6f5f4f3f2f1f0efeeedecebeae9e8e7e6e5e4e3e2e1";
const EXPECTED_ROOT_SHA256_HEX: &str =
    "a26196a0b88e12fc5c5967c28ed2d26b63814a59640cf43bf27412688e3a89c2";

const ID_FREEZEROOT_DERIVATION_001: &str = "ID-FREEZEROOT-DERIVATION-001";
const ID_FREEZEROOT_DOMAIN_CHANGED_FAIL_001: &str = "ID-FREEZEROOT-DOMAIN-CHANGED-FAIL-001";
const ID_FREEZEROOT_TUPLE_SWAPPED_FAIL_001: &str = "ID-FREEZEROOT-TUPLE-SWAPPED-FAIL-001";
const ID_FREEZEROOT_REGISTRY_BYTE_CHANGED_FAIL_001: &str =
    "ID-FREEZEROOT-REGISTRY-BYTE-CHANGED-FAIL-001";
const ID_FREEZEROOT_ATTEMPT_BYTE_CHANGED_FAIL_001: &str =
    "ID-FREEZEROOT-ATTEMPT-BYTE-CHANGED-FAIL-001";
const ID_FREEZEROOT_REGISTRY_LEN_31_FAIL_001: &str = "ID-FREEZEROOT-REGISTRY-LEN-31-FAIL-001";
const ID_FREEZEROOT_REGISTRY_LEN_33_FAIL_001: &str = "ID-FREEZEROOT-REGISTRY-LEN-33-FAIL-001";
const ID_FREEZEROOT_ATTEMPT_LEN_31_FAIL_001: &str = "ID-FREEZEROOT-ATTEMPT-LEN-31-FAIL-001";
const ID_FREEZEROOT_ATTEMPT_LEN_33_FAIL_001: &str = "ID-FREEZEROOT-ATTEMPT-LEN-33-FAIL-001";

fn registry_fixture() -> [u8; 32] {
    core::array::from_fn(|index| index as u8)
}

fn attempt_fixture() -> [u8; 32] {
    [
        0x80, 0x7f, 0xfe, 0xfd, 0xfc, 0xfb, 0xfa, 0xf9, 0xf8, 0xf7, 0xf6, 0xf5, 0xf4, 0xf3, 0xf2,
        0xf1, 0xf0, 0xef, 0xee, 0xed, 0xec, 0xeb, 0xea, 0xe9, 0xe8, 0xe7, 0xe6, 0xe5, 0xe4, 0xe3,
        0xe2, 0xe1,
    ]
}

/// This manual constructor is deliberately not a call into the production encoder.
/// It covers the exact, fixed three-item bstr tuple required by Identity Format §46.
fn manually_construct_freeze_root_bytes(
    domain: &str,
    registry_id: &[u8; 32],
    attempt_id: &[u8; 32],
) -> Vec<u8> {
    assert_eq!(
        domain.len(),
        30,
        "this fixed vector requires the 30-byte domain framing"
    );
    let mut bytes = Vec::with_capacity(101);
    bytes.extend_from_slice(&[0x83, 0x78, 0x1e]);
    bytes.extend_from_slice(domain.as_bytes());
    bytes.extend_from_slice(&[0x58, 0x20]);
    bytes.extend_from_slice(registry_id);
    bytes.extend_from_slice(&[0x58, 0x20]);
    bytes.extend_from_slice(attempt_id);
    bytes
}

fn lowercase_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn id_freezeroot_derivation_001_matches_fixed_bytes_and_independent_sha256() {
    let registry_bytes = registry_fixture();
    let attempt_bytes = attempt_fixture();
    let registry = RegistryId::try_from(registry_bytes.as_slice()).expect("fixture has 32 bytes");
    let attempt =
        FreezeAttemptId::try_from(attempt_bytes.as_slice()).expect("fixture has 32 bytes");

    let derived = derive_freeze_root(registry, attempt);
    let manually_constructed =
        manually_construct_freeze_root_bytes(DOMAIN, &registry_bytes, &attempt_bytes);

    assert_eq!(
        lowercase_hex(&manually_constructed),
        EXPECTED_CBOR_HEX,
        "{ID_FREEZEROOT_DERIVATION_001}"
    );
    assert_eq!(
        derived.identity_bytes(),
        manually_constructed.as_slice(),
        "{ID_FREEZEROOT_DERIVATION_001}"
    );
    assert_eq!(
        lowercase_hex(derived.intended_root_id().as_bytes()),
        EXPECTED_ROOT_SHA256_HEX,
        "{ID_FREEZEROOT_DERIVATION_001}"
    );
}

#[test]
fn freezeroot_is_deterministic_for_identical_inputs() {
    let registry = RegistryId::try_from(registry_fixture().as_slice()).unwrap();
    let attempt = FreezeAttemptId::try_from(attempt_fixture().as_slice()).unwrap();

    assert_eq!(
        derive_freeze_root(registry, attempt),
        derive_freeze_root(registry, attempt)
    );
}

#[test]
fn id_freezeroot_registry_byte_changed_fail_001_changes_identity_and_root_id() {
    let registry_bytes = registry_fixture();
    let mut changed_registry_bytes = registry_bytes;
    changed_registry_bytes[0] ^= 0x01;
    let attempt = FreezeAttemptId::try_from(attempt_fixture().as_slice()).unwrap();

    assert_ne!(
        derive_freeze_root(
            RegistryId::try_from(registry_bytes.as_slice()).unwrap(),
            attempt
        ),
        derive_freeze_root(
            RegistryId::try_from(changed_registry_bytes.as_slice()).unwrap(),
            attempt
        ),
        "{ID_FREEZEROOT_REGISTRY_BYTE_CHANGED_FAIL_001}"
    );
}

#[test]
fn id_freezeroot_attempt_byte_changed_fail_001_changes_identity_and_root_id() {
    let attempt_bytes = attempt_fixture();
    let mut changed_attempt_bytes = attempt_bytes;
    changed_attempt_bytes[31] ^= 0x01;
    let registry = RegistryId::try_from(registry_fixture().as_slice()).unwrap();

    assert_ne!(
        derive_freeze_root(
            registry,
            FreezeAttemptId::try_from(attempt_bytes.as_slice()).unwrap()
        ),
        derive_freeze_root(
            registry,
            FreezeAttemptId::try_from(changed_attempt_bytes.as_slice()).unwrap()
        ),
        "{ID_FREEZEROOT_ATTEMPT_BYTE_CHANGED_FAIL_001}"
    );
}

#[test]
fn id_freezeroot_tuple_swapped_fail_001_does_not_match_specified_tuple() {
    let registry_bytes = registry_fixture();
    let attempt_bytes = attempt_fixture();

    assert_ne!(
        manually_construct_freeze_root_bytes(DOMAIN, &registry_bytes, &attempt_bytes),
        manually_construct_freeze_root_bytes(DOMAIN, &attempt_bytes, &registry_bytes),
        "{ID_FREEZEROOT_TUPLE_SWAPPED_FAIL_001}"
    );
}

#[test]
fn id_freezeroot_domain_changed_fail_001_does_not_match_frozen_domain_framing() {
    let registry_bytes = registry_fixture();
    let attempt_bytes = attempt_fixture();

    assert_ne!(
        manually_construct_freeze_root_bytes(DOMAIN, &registry_bytes, &attempt_bytes),
        manually_construct_freeze_root_bytes(
            "EvidenceRegistry.FreezeRoot.v2",
            &registry_bytes,
            &attempt_bytes
        ),
        "{ID_FREEZEROOT_DOMAIN_CHANGED_FAIL_001}"
    );
}

#[test]
fn id_freezeroot_registry_len_31_fail_001_rejects_before_derivation() {
    assert!(
        RegistryId::try_from([0u8; 31].as_slice()).is_err(),
        "{ID_FREEZEROOT_REGISTRY_LEN_31_FAIL_001}"
    );
}

#[test]
fn id_freezeroot_registry_len_33_fail_001_rejects_before_derivation() {
    assert!(
        RegistryId::try_from([0u8; 33].as_slice()).is_err(),
        "{ID_FREEZEROOT_REGISTRY_LEN_33_FAIL_001}"
    );
}

#[test]
fn id_freezeroot_attempt_len_31_fail_001_rejects_before_derivation() {
    assert!(
        FreezeAttemptId::try_from([0u8; 31].as_slice()).is_err(),
        "{ID_FREEZEROOT_ATTEMPT_LEN_31_FAIL_001}"
    );
}

#[test]
fn id_freezeroot_attempt_len_33_fail_001_rejects_before_derivation() {
    assert!(
        FreezeAttemptId::try_from([0u8; 33].as_slice()).is_err(),
        "{ID_FREEZEROOT_ATTEMPT_LEN_33_FAIL_001}"
    );
}
