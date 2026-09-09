use evidence_registry::{
    resolve_minimal_policy_gate_scope, ExactRecordByteResolver, MinimalPolicyRecord,
    PolicyScopeStructuralBindingError, RecordId, RecordTypeId, ScopeRecord, StrictRecordFrame,
};

const SCOPE_RECORD_NO_LABEL_HEX: &str =
    "84781a45766964656e636552656769737472792e5265636f72642e76311401a500010114100711021244deadbeef";
const SCOPE_RECORD_NO_LABEL_ID_HEX: &str =
    "8932284ecef0ffb56bc442eb071a55ef947b3a0d55a31585ec00998d7b503784";
const SCOPE_RECORD_WITH_LABEL_HEX: &str =
    "84781a45766964656e636552656769737472792e5265636f72642e76311401a600010114100711021244deadbeef136773636f70652041";
const SCOPE_RECORD_WITH_LABEL_ID_HEX: &str =
    "f318ed15ac6057b0014a1e399b7b772859e7a7c841958ecc66b36a14f957a9c5";
const MINIMAL_POLICY_BOUND_TO_SCOPE_HEX: &str =
    "84781a45766964656e636552656769737472792e5265636f72642e7631182801a500010118281058208932284ecef0ffb56bc442eb071a55ef947b3a0d55a31585ec00998d7b503784181d855820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f015820202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f1901905820404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f181e8101";
const MINIMAL_POLICY_BOUND_TO_SCOPE_ID_HEX: &str =
    "1d9e66d0c3167c1b2f143d019af044284df6a028c9d42217574145210894a945";

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}

fn record_id(hex: &str) -> RecordId {
    RecordId::try_from(hex_bytes(hex).as_slice()).unwrap()
}

fn replace_unique(bytes: &mut [u8], needle: &[u8], replacement_at_offset: usize, replacement: u8) {
    let matches = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(index, candidate)| (candidate == needle).then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "fixture marker must occur exactly once");
    bytes[matches[0] + replacement_at_offset] = replacement;
}

struct FixtureRecordResolver {
    records: Vec<(RecordId, Vec<u8>)>,
}

impl ExactRecordByteResolver for FixtureRecordResolver {
    fn resolve(&self, record_id: RecordId) -> Option<&[u8]> {
        self.records
            .iter()
            .find_map(|(stored_id, bytes)| (*stored_id == record_id).then_some(bytes.as_slice()))
    }
}

#[test]
fn scope_record_decodes_fixed_declared_fields_and_exact_identity() {
    let no_label =
        ScopeRecord::decode_authoritative(&hex_bytes(SCOPE_RECORD_NO_LABEL_HEX)).unwrap();
    assert_eq!(
        no_label.record_id(),
        record_id(SCOPE_RECORD_NO_LABEL_ID_HEX)
    );
    assert_eq!(
        no_label.record_type_id(),
        RecordTypeId::try_from(20_u64).unwrap()
    );
    assert_eq!(no_label.input().scope_profile_id, 7);
    assert_eq!(no_label.input().scope_profile_version, 2);
    assert_eq!(no_label.input().scope_payload, hex_bytes("deadbeef"));
    assert_eq!(no_label.input().scope_label, None);

    let with_label =
        ScopeRecord::decode_authoritative(&hex_bytes(SCOPE_RECORD_WITH_LABEL_HEX)).unwrap();
    assert_eq!(
        with_label.record_id(),
        record_id(SCOPE_RECORD_WITH_LABEL_ID_HEX)
    );
    assert_eq!(with_label.input().scope_label.as_deref(), Some("scope A"));
}

#[test]
fn scope_record_rejects_non_scope_and_unknown_local_body_key_variants() {
    let canonical = hex_bytes(SCOPE_RECORD_NO_LABEL_HEX);

    let mut unknown_body_key = canonical.clone();
    replace_unique(&mut unknown_body_key, &[0x11, 0x02, 0x12], 2, 0x14);
    let mut non_scope_type = canonical;
    replace_unique(&mut non_scope_type, &[0x14, 0x01, 0xa5], 0, 0x15);
    replace_unique(&mut non_scope_type, &[0x01, 0x14, 0x10], 1, 0x15);

    assert!(StrictRecordFrame::decode_authoritative(&unknown_body_key).is_ok());
    assert!(StrictRecordFrame::decode_authoritative(&non_scope_type).is_ok());
    for invalid in [unknown_body_key, non_scope_type] {
        assert!(ScopeRecord::decode_authoritative(&invalid).is_err());
    }
}

#[test]
fn scope_record_rejects_structurally_valid_non_text_label_at_local_label_gate() {
    let mut non_text_label = hex_bytes(SCOPE_RECORD_NO_LABEL_HEX);
    replace_unique(&mut non_text_label, &[0x01, 0xa5, 0x00, 0x01], 1, 0xa6);
    non_text_label.extend_from_slice(&[0x13, 0xf5]);

    assert!(StrictRecordFrame::decode_authoritative(&non_text_label).is_ok());
    assert!(ScopeRecord::decode_authoritative(&non_text_label).is_err());
}

#[test]
fn minimal_policy_scope_binding_is_exact_and_structural_only() {
    let policy_bytes = hex_bytes(MINIMAL_POLICY_BOUND_TO_SCOPE_HEX);
    let policy = MinimalPolicyRecord::decode_authoritative(&policy_bytes).unwrap();
    assert_eq!(
        policy.record_id(),
        record_id(MINIMAL_POLICY_BOUND_TO_SCOPE_ID_HEX)
    );
    let scope_bytes = hex_bytes(SCOPE_RECORD_NO_LABEL_HEX);
    let resolver = FixtureRecordResolver {
        records: vec![(record_id(SCOPE_RECORD_NO_LABEL_ID_HEX), scope_bytes)],
    };

    let binding = resolve_minimal_policy_gate_scope(&policy, &resolver).unwrap();
    assert_eq!(binding.policy_record_id(), policy.record_id());
    assert_eq!(
        binding.gate_scope_ref(),
        record_id(SCOPE_RECORD_NO_LABEL_ID_HEX)
    );
    assert_eq!(
        binding.scope_record().record_id(),
        record_id(SCOPE_RECORD_NO_LABEL_ID_HEX)
    );
    assert_eq!(binding.scope_record().input().scope_profile_id, 7);
    assert_eq!(
        binding.scope_record().input().scope_payload,
        hex_bytes("deadbeef")
    );
}

#[test]
fn minimal_policy_scope_binding_fails_closed_for_unavailable_invalid_or_wrong_identity_bytes() {
    let policy =
        MinimalPolicyRecord::decode_authoritative(&hex_bytes(MINIMAL_POLICY_BOUND_TO_SCOPE_HEX))
            .unwrap();
    let target = record_id(SCOPE_RECORD_NO_LABEL_ID_HEX);

    let unavailable = FixtureRecordResolver { records: vec![] };
    assert_eq!(
        resolve_minimal_policy_gate_scope(&policy, &unavailable),
        Err(PolicyScopeStructuralBindingError::ScopePayloadUnavailable)
    );

    let non_scope = FixtureRecordResolver {
        records: vec![(target, hex_bytes(MINIMAL_POLICY_BOUND_TO_SCOPE_HEX))],
    };
    assert_eq!(
        resolve_minimal_policy_gate_scope(&policy, &non_scope),
        Err(PolicyScopeStructuralBindingError::ScopeDecode)
    );

    let different_valid_scope = FixtureRecordResolver {
        records: vec![(target, hex_bytes(SCOPE_RECORD_WITH_LABEL_HEX))],
    };
    assert_eq!(
        resolve_minimal_policy_gate_scope(&policy, &different_valid_scope),
        Err(PolicyScopeStructuralBindingError::ScopeIdentityMismatch)
    );
}
