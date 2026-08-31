use evidence_registry::{ManifestRecord, RecordId, RecordTypeId, StrictRecordFrame};

const MANIFEST_RECORD_HEX: &str = concat!(
    "84781a45766964656e636552656769737472792e5265636f72642e76310201a700010102105820",
    "e0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff110112011301148185",
    "0181416100015820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
);
const MANIFEST_RECORD_ID_HEX: &str =
    "b4f57d90cf94e711e7ca69bd8cc79120a0b792ff0bf3d10675f41ff72fb4fcb5";

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}

fn hex_id(hex: &str) -> [u8; 32] {
    hex_bytes(hex).try_into().unwrap()
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

fn replace_unique_range(bytes: &mut Vec<u8>, needle: &[u8], replacement: &[u8]) {
    let matches = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(index, candidate)| (candidate == needle).then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "fixture marker must occur exactly once");
    bytes.splice(
        matches[0]..matches[0] + needle.len(),
        replacement.iter().copied(),
    );
}

fn manifest_with_artifact_paths(paths: &[&[u8]]) -> Vec<u8> {
    assert!(!paths.is_empty() && paths.len() < 24);
    let canonical = hex_bytes(MANIFEST_RECORD_HEX);
    let fields = canonical
        .windows(8)
        .position(|window| window == [0x11, 0x01, 0x12, 0x01, 0x13, 0x01, 0x14, 0x81])
        .unwrap();
    let mut bytes = canonical[..fields].to_vec();
    bytes.extend_from_slice(&[0x11, paths.len() as u8, 0x12, 0x01, 0x13, 0x01, 0x14]);
    bytes.push(0x80 + paths.len() as u8);
    for path in paths {
        assert!(!path.is_empty() && path.len() < 24);
        bytes.extend_from_slice(&[0x85, 0x01, 0x81, 0x40 + path.len() as u8]);
        bytes.extend_from_slice(path);
        bytes.extend_from_slice(&[0x00, 0x01, 0x58, 0x20]);
        bytes.extend(0_u8..32);
    }
    bytes
}

#[test]
fn manifest_record_decodes_fixed_local_artifact_fields_and_exact_identity() {
    let canonical = hex_bytes(MANIFEST_RECORD_HEX);
    let decoded = ManifestRecord::decode_authoritative(&canonical).unwrap();

    assert_eq!(
        decoded.record_id(),
        RecordId::try_from(hex_id(MANIFEST_RECORD_ID_HEX).as_slice()).unwrap()
    );
    assert_eq!(
        decoded.record_type_id(),
        RecordTypeId::try_from(2_u64).unwrap()
    );
    assert_eq!(
        decoded.input().subject_id,
        hex_id("e0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff")
    );
    assert_eq!(decoded.input().artifact_count, 1);
    assert_eq!(decoded.input().path_identity_profile_id, 1);
    assert_eq!(decoded.input().digest_profile_id, 1);
    assert_eq!(decoded.input().artifacts.len(), 1);
    assert_eq!(decoded.input().artifacts[0].artifact_kind_id, 1);
    assert_eq!(
        decoded.input().artifacts[0].path_components,
        vec![b"a".to_vec()]
    );
    assert_eq!(decoded.input().artifacts[0].size_bytes, 0);
    assert_eq!(decoded.input().artifacts[0].digest_algorithm_id, 1);
    assert_eq!(
        decoded.input().artifacts[0].digest_bytes,
        hex_id("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
    );
}

#[test]
fn manifest_record_rejects_nonadjacent_duplicate_raw_path_component_sequences() {
    let duplicate = manifest_with_artifact_paths(&[b"same", b"middle", b"same"]);

    assert!(StrictRecordFrame::decode_authoritative(&duplicate).is_ok());
    assert!(ManifestRecord::decode_authoritative(&duplicate).is_err());
}

#[test]
fn manifest_record_strict_decoder_rejects_artifact_count_or_local_artifact_grammar_mismatch() {
    let canonical = hex_bytes(MANIFEST_RECORD_HEX);

    let mut artifact_count_mismatch = canonical.clone();
    replace_unique(&mut artifact_count_mismatch, &[0x11, 0x01, 0x12], 1, 0x02);

    let mut empty_path = canonical.clone();
    replace_unique(
        &mut empty_path,
        &[0x85, 0x01, 0x81, 0x41, 0x61, 0x00],
        2,
        0x80,
    );

    let mut unsupported_digest_algorithm = canonical.clone();
    replace_unique(
        &mut unsupported_digest_algorithm,
        &[0x00, 0x01, 0x58, 0x20],
        1,
        0x02,
    );

    let mut short_digest = canonical.clone();
    replace_unique(&mut short_digest, &[0x01, 0x58, 0x20], 2, 0x1f);
    short_digest.pop();

    assert!(StrictRecordFrame::decode_authoritative(&artifact_count_mismatch).is_ok());
    assert!(StrictRecordFrame::decode_authoritative(&unsupported_digest_algorithm).is_ok());
    for invalid in [
        artifact_count_mismatch,
        empty_path,
        unsupported_digest_algorithm,
    ] {
        assert!(ManifestRecord::decode_authoritative(&invalid).is_err());
    }
    assert!(ManifestRecord::decode_authoritative(&short_digest).is_err());
}

#[test]
fn manifest_record_strict_decoder_rejects_unassigned_artifact_kinds_and_invalid_path_components() {
    let canonical = hex_bytes(MANIFEST_RECORD_HEX);

    let mut invalid_artifact_kind_zero = canonical.clone();
    replace_unique(
        &mut invalid_artifact_kind_zero,
        &[0x85, 0x01, 0x81, 0x41, 0x61],
        1,
        0x00,
    );
    let mut reserved_artifact_kind = canonical.clone();
    replace_unique(
        &mut reserved_artifact_kind,
        &[0x85, 0x01, 0x81, 0x41, 0x61],
        1,
        0x03,
    );
    let mut unassigned_artifact_kind = canonical.clone();
    replace_unique(
        &mut unassigned_artifact_kind,
        &[0x85, 0x01, 0x81, 0x41, 0x61],
        1,
        0x04,
    );

    let path_member = &[0x81, 0x41, 0x61, 0x00, 0x01, 0x58, 0x20];
    let mut empty_component = canonical.clone();
    replace_unique_range(
        &mut empty_component,
        path_member,
        &[0x81, 0x40, 0x00, 0x01, 0x58, 0x20],
    );
    let mut dot_component = canonical.clone();
    replace_unique(&mut dot_component, path_member, 2, b'.');
    let mut dot_dot_component = canonical;
    replace_unique_range(
        &mut dot_dot_component,
        path_member,
        &[0x81, 0x42, b'.', b'.', 0x00, 0x01, 0x58, 0x20],
    );

    for invalid in [
        invalid_artifact_kind_zero,
        reserved_artifact_kind,
        unassigned_artifact_kind,
        empty_component,
        dot_component,
        dot_dot_component,
    ] {
        assert!(StrictRecordFrame::decode_authoritative(&invalid).is_ok());
        assert!(ManifestRecord::decode_authoritative(&invalid).is_err());
    }
}
