use evidence_registry::{
    derive_selected_regular_file_subject, ManifestArtifactEntry, RecordDecodeError,
};

#[test]
fn selected_subject_projection_matches_the_frozen_empty_and_single_file_vectors() {
    assert_eq!(
        derive_selected_regular_file_subject(&[]).unwrap(),
        [
            0xa2, 0x16, 0x2a, 0x87, 0xb0, 0x4f, 0xf2, 0xdb, 0x6b, 0xd0, 0x05, 0x61, 0x01, 0x95,
            0x75, 0xbf, 0xa9, 0xa7, 0xaa, 0x88, 0xc4, 0xa7, 0x87, 0xfa, 0x53, 0x7b, 0xab, 0xd6,
            0xc0, 0x07, 0x32, 0xb4,
        ]
    );

    let artifact = ManifestArtifactEntry {
        artifact_kind_id: 1,
        path_components: vec![b"a".to_vec()],
        size_bytes: 3,
        digest_algorithm_id: 1,
        digest_bytes: [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ],
    };
    assert_eq!(
        derive_selected_regular_file_subject(&[artifact]).unwrap(),
        [
            0x2a, 0x04, 0x58, 0x2a, 0x51, 0xc7, 0xb8, 0x92, 0xad, 0xd0, 0xa8, 0x91, 0xe2, 0x09,
            0xa3, 0x0b, 0xc1, 0x71, 0xdd, 0xb9, 0x83, 0xdc, 0x40, 0x76, 0xba, 0x52, 0xa1, 0x9e,
            0xcb, 0x14, 0x5a, 0xdb,
        ]
    );
}

#[test]
fn selected_subject_projection_rejects_nonregular_unsorted_or_duplicate_paths() {
    let regular = |component: &[u8]| ManifestArtifactEntry {
        artifact_kind_id: 1,
        path_components: vec![component.to_vec()],
        size_bytes: 0,
        digest_algorithm_id: 1,
        digest_bytes: [0; 32],
    };
    assert_eq!(
        derive_selected_regular_file_subject(&[regular(b"b"), regular(b"a")]),
        Err(RecordDecodeError)
    );
    assert_eq!(
        derive_selected_regular_file_subject(&[regular(b"a"), regular(b"a")]),
        Err(RecordDecodeError)
    );
    let nonregular = ManifestArtifactEntry {
        artifact_kind_id: 2,
        ..regular(b"a")
    };
    assert_eq!(
        derive_selected_regular_file_subject(&[nonregular]),
        Err(RecordDecodeError)
    );
}
