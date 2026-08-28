use evidence_registry::{
    GenesisRecord, GenesisRecordInput, RecordId, RecordTypeId, RegistryId, StrictRecordFrame,
};
use std::process::Command;

fn id(first: u8) -> [u8; 32] {
    core::array::from_fn(|index| first + index as u8)
}

fn genesis_record() -> GenesisRecord {
    GenesisRecord::new(GenesisRecordInput {
        registry_id: RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        journal_format_version: 1,
        record_identity_profile_id: 1,
        storage_capability_class_id: RecordId::try_from(id(0x40).as_slice()).unwrap(),
        environment_observation_id: RecordId::try_from(id(0x60).as_slice()).unwrap(),
        created_by_tool_version: "record-frame-test".to_owned(),
    })
    .unwrap()
}

fn frame_with_nested_array_value(nesting: usize) -> Vec<u8> {
    let mut bytes = vec![0x84, 0x78, 0x1a];
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x01, 0x01, 0xa3, 0x00, 0x01, 0x01, 0x01, 0x10]);
    bytes.extend(core::iter::repeat_n(0x81, nesting));
    bytes.push(0xf5);
    bytes
}

fn frame_with_nested_map_value(nesting: usize) -> Vec<u8> {
    let mut bytes = vec![0x84, 0x78, 0x1a];
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x01, 0x01, 0xa3, 0x00, 0x01, 0x01, 0x01, 0x10]);
    for _ in 0..nesting {
        bytes.extend_from_slice(&[0xa1, 0x00]);
    }
    bytes.push(0xf5);
    bytes
}

fn frame_with_alternating_nested_value(nesting: usize) -> Vec<u8> {
    let mut bytes = vec![0x84, 0x78, 0x1a];
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x01, 0x01, 0xa3, 0x00, 0x01, 0x01, 0x01, 0x10]);
    for level in 0..nesting {
        if level % 2 == 0 {
            bytes.push(0x81);
        } else {
            bytes.extend_from_slice(&[0xa1, 0x00]);
        }
    }
    bytes.push(0xf5);
    bytes
}

fn frame_with_leaf_map_at_depth(nesting: usize, duplicate_keys: bool) -> Vec<u8> {
    let mut bytes = vec![0x84, 0x78, 0x1a];
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x01, 0x01, 0xa3, 0x00, 0x01, 0x01, 0x01, 0x10]);
    bytes.extend(core::iter::repeat_n(0x81, nesting));
    bytes.extend_from_slice(&[0xa2, 0x00, 0xf5]);
    bytes.extend_from_slice(if duplicate_keys {
        &[0x00, 0xf5]
    } else {
        &[0x01, 0xf5]
    });
    bytes
}

#[test]
fn strict_record_frame_preserves_canonical_type_and_self_hash_identity() {
    let record = genesis_record();
    let bytes = record.authoritative_cbor();

    let frame = StrictRecordFrame::decode_authoritative(&bytes).unwrap();

    assert_eq!(frame.record_type_id(), RecordTypeId::try_from(1).unwrap());
    assert_eq!(frame.schema_version(), 1);
    assert_eq!(frame.record_id(), record.record_id());
    assert_eq!(frame.authoritative_cbor(), bytes.as_slice());
}

#[test]
fn strict_record_frame_bounds_opaque_body_value_nesting() {
    assert!(StrictRecordFrame::decode_authoritative(&frame_with_nested_array_value(64)).is_ok());
    assert!(StrictRecordFrame::decode_authoritative(&frame_with_nested_array_value(65)).is_err());
}

#[test]
fn strict_record_frame_bounds_opaque_nested_map_values() {
    assert!(StrictRecordFrame::decode_authoritative(&frame_with_nested_map_value(64)).is_ok());
    assert!(StrictRecordFrame::decode_authoritative(&frame_with_nested_map_value(65)).is_err());
}

#[test]
fn strict_record_frame_bounds_opaque_alternating_container_values() {
    assert!(
        StrictRecordFrame::decode_authoritative(&frame_with_alternating_nested_value(64)).is_ok()
    );
    assert!(
        StrictRecordFrame::decode_authoritative(&frame_with_alternating_nested_value(65)).is_err()
    );
    assert!(
        StrictRecordFrame::decode_authoritative(&frame_with_alternating_nested_value(1024))
            .is_err()
    );

    let mut truncated = frame_with_alternating_nested_value(64);
    truncated.pop();
    assert!(StrictRecordFrame::decode_authoritative(&truncated).is_err());
}

#[test]
fn strict_record_frame_preserves_opaque_map_key_canonicality_at_depth_limit() {
    assert!(
        StrictRecordFrame::decode_authoritative(&frame_with_leaf_map_at_depth(63, false)).is_ok()
    );
    assert!(
        StrictRecordFrame::decode_authoritative(&frame_with_leaf_map_at_depth(63, true)).is_err()
    );
}

#[test]
fn strict_record_frame_rejects_hostile_deep_value_in_child_process() {
    const CHILD_ENV: &str = "EVIDENCE_REGISTRY_RECORD_FRAME_DEPTH_CHILD";

    if std::env::var_os(CHILD_ENV).is_some() {
        assert!(
            StrictRecordFrame::decode_authoritative(&frame_with_nested_array_value(100_000))
                .is_err()
        );
        return;
    }

    let output = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("strict_record_frame_rejects_hostile_deep_value_in_child_process")
        .arg("--nocapture")
        .env(CHILD_ENV, "1")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "depth-regression child failed: {output:?}"
    );
}
