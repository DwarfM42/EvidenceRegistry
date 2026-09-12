use evidence_registry::{ScopeRecord, ScopeRecordInput, StrictRecordFrame, ER_UINT_MAX};
use sha2::{Digest, Sha256};

#[test]
fn check_set_requires_nonempty_strict_order_without_resolving_members() {
    use evidence_registry::{CheckSetRecord, CheckSetRecordInput, RecordId};
    let a = RecordId::try_from([0; 32].as_slice()).unwrap();
    let b = RecordId::try_from([255; 32].as_slice()).unwrap();
    for refs in [vec![], vec![a, a], vec![b, a]] {
        assert!(CheckSetRecord::new(CheckSetRecordInput { check_refs: refs }).is_err());
    }
    for refs in [vec![a], vec![a, b]] {
        let input = CheckSetRecordInput { check_refs: refs };
        let record = CheckSetRecord::new(input.clone()).unwrap();
        assert_eq!(record.input(), &input);
        assert_eq!(record.check_refs(), input.check_refs);
        assert_eq!(record.record_type_id().value(), 23);
        let bytes = record.authoritative_cbor();
        assert_eq!(
            record.record_id().as_bytes().as_slice(),
            Sha256::digest(&bytes).as_slice()
        );
        assert_eq!(
            CheckSetRecord::decode_authoritative(&bytes).unwrap(),
            record
        );
        // These intentionally unresolved IDs prove that this is only a local codec.
    }
    for refs in [vec![], vec![a, a], vec![b, a]] {
        let mut body = vec![0x10, 0x80 + refs.len() as u8];
        for id in refs {
            body.extend_from_slice(&[0x58, 0x20]);
            body.extend_from_slice(id.as_bytes());
        }
        assert!(CheckSetRecord::decode_authoritative(&frame(23, 3, &body)).is_err());
    }
}

// Independent tiny wire fixtures: no production encoder used for malformed bytes.
#[test]
fn profile_codecs_reject_noncanonical_or_mistyped_local_fields() {
    use evidence_registry::{CheckRecord, MethodRecord};
    for kind in [20, 21, 22] {
        let accepts = |bytes: &[u8]| match kind {
            20 => ScopeRecord::decode_authoritative(bytes).is_ok(),
            21 => MethodRecord::decode_authoritative(bytes).is_ok(),
            _ => CheckRecord::decode_authoritative(bytes).is_ok(),
        };
        let valid = frame(kind, 5, &[16, 0, 17, 1, 18, 0x40]);
        assert!(accepts(&valid));
        for body in [
            vec![16, 0x40, 17, 1, 18, 0x40],    // profile not UInt
            vec![16, 0, 17, 0xf4, 18, 0x40],    // version not UInt
            vec![16, 0, 17, 1, 18, 0x60],       // payload not bstr
            vec![16, 0x18, 0, 17, 1, 18, 0x40], // overlong UInt
            vec![16, 0, 16, 1, 18, 0x40],       // duplicate key
            vec![17, 1, 16, 0, 18, 0x40],       // unordered keys
            vec![16, 0, 17, 1, 20, 0x40],       // unknown field replaces payload
            vec![16, 0, 17, 1, 18, 0x5f, 0xff], // indefinite bstr
            vec![16, 0, 17, 1, 18, 0x58, 0],    // overlong bstr length
            vec![16, 0, 17, 1, 18, 0x5a, 0xff, 0xff, 0xff, 0xff], // truncated huge bstr
        ] {
            assert!(
                !accepts(&frame(kind, 5, &body)),
                "kind={kind}, body={body:?}"
            );
        }
        for label in [vec![0xf6], vec![0x40], vec![0x61, 0xff], vec![0x7f, 0xff]] {
            let mut body = vec![16, 0, 17, 1, 18, 0x40, 19];
            body.extend_from_slice(&label);
            assert!(!accepts(&frame(kind, 6, &body)));
        }
        for field in [16, 17] {
            let mut body = vec![16, 0, 17, 1, 18, 0x40];
            let position = if field == 16 { 1 } else { 3 };
            body.splice(position..position + 1, [0x1b, 0, 0x20, 0, 0, 0, 0, 0, 0]);
            assert!(!accepts(&frame(kind, 5, &body))); // ER_UINT_MAX + 1
        }
        assert!(!accepts(&frame(kind, 4, &[16, 0, 17, 1])));
        assert!(!accepts(&frame(kind, 6, &[16, 0, 17, 1, 18, 0x40, 20, 0])));
        for end in 0..valid.len() {
            assert!(!accepts(&valid[..end]));
        }
        let mut trailing = valid.clone();
        trailing.push(0);
        assert!(!accepts(&trailing));
        let mut schema = valid.clone();
        schema[30] = 2;
        assert!(!accepts(&schema));
        let mut duplicated_type = valid.clone();
        duplicated_type[35] = 23;
        assert!(!accepts(&duplicated_type));
        let mut domain = valid.clone();
        domain[3] = b'x';
        assert!(!accepts(&domain));
        let mut indefinite_map = valid.clone();
        indefinite_map[31] = 0xbf;
        indefinite_map.push(0xff);
        assert!(!accepts(&indefinite_map));
    }
}

#[test]
fn check_set_decoder_rejects_wrong_shape_width_and_framing() {
    use evidence_registry::CheckSetRecord;
    let mut member = vec![0x58, 0x20];
    member.extend_from_slice(&[0; 32]);
    let mut body = vec![16, 0x81];
    body.extend_from_slice(&member);
    let valid = frame(23, 3, &body);
    assert!(CheckSetRecord::decode_authoritative(&valid).is_ok());
    for malformed in [
        frame(23, 2, &[]),
        frame(23, 3, &[16, 0x40]),
        frame(23, 3, &[16, 0x81, 0]),
        frame(23, 3, &[16, 0x9a, 0xff, 0xff, 0xff, 0xff]),
        frame(23, 3, &[16, 0x9f, 0xff]),
        frame(23, 4, &[16, 0x80, 17, 0]),
        frame(20, 3, &body),
        frame(21, 3, &body),
        frame(22, 3, &body),
    ] {
        assert!(CheckSetRecord::decode_authoritative(&malformed).is_err());
    }
    for width in [0, 31, 33] {
        let mut body = vec![16, 0x81, 0x58, width];
        body.extend(std::iter::repeat_n(0, usize::from(width)));
        assert!(CheckSetRecord::decode_authoritative(&frame(23, 3, &body)).is_err());
    }
    for end in 0..valid.len() {
        assert!(CheckSetRecord::decode_authoritative(&valid[..end]).is_err());
    }
    let mut trailing = valid;
    trailing.push(0);
    assert!(CheckSetRecord::decode_authoritative(&trailing).is_err());
}

#[test]
fn parameter_labels_and_types_are_identity_bearing() {
    use evidence_registry::{CheckRecord, MethodRecord};
    let mut all_ids = std::collections::BTreeSet::new();
    for kind in [20, 21, 22] {
        for label in [None, Some(""), Some("x"), Some("é"), Some("e\u{301}")] {
            let mut body = vec![16, 0, 17, 1, 18, 0x40];
            if let Some(label) = label {
                body.extend_from_slice(&[19, 0x60 + label.len() as u8]);
                body.extend_from_slice(label.as_bytes());
            }
            let bytes = frame(kind, if label.is_some() { 6 } else { 5 }, &body);
            let (id, serialized) = match kind {
                20 => {
                    let r = ScopeRecord::decode_authoritative(&bytes).unwrap();
                    (r.record_id(), r.authoritative_cbor())
                }
                21 => {
                    let r = MethodRecord::decode_authoritative(&bytes).unwrap();
                    (r.record_id(), r.authoritative_cbor())
                }
                _ => {
                    let r = CheckRecord::decode_authoritative(&bytes).unwrap();
                    (r.record_id(), r.authoritative_cbor())
                }
            };
            assert_eq!(serialized, bytes);
            assert!(all_ids.insert(id));
        }
    }
}

// Independent tiny wire fixtures: no production encoder used for malformed bytes.
fn frame(kind: u8, fields: u8, body: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0x84, 0x78, 0x1a];
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[kind, 1, 0xa0 + fields, 0, 1, 1, kind]);
    bytes.extend_from_slice(body);
    bytes
}

#[test]
fn check_is_a_typed_opaque_definition_not_a_method() {
    use evidence_registry::{CheckRecord, CheckRecordInput, MethodRecord};
    for label in [None, Some(String::new()), Some("check\0検査".to_owned())] {
        let input = CheckRecordInput {
            check_profile_id: 0,
            check_profile_version: ER_UINT_MAX,
            check_payload: vec![0xff, 0, 0x80],
            check_label: label,
        };
        let record = CheckRecord::new(input.clone()).unwrap();
        let bytes = record.authoritative_cbor();
        assert_eq!(record.input(), &input);
        assert_eq!(record.record_type_id().value(), 22);
        assert_eq!(
            record.record_id().as_bytes().as_slice(),
            Sha256::digest(&bytes).as_slice()
        );
        assert_eq!(CheckRecord::decode_authoritative(&bytes).unwrap(), record);
        assert!(MethodRecord::decode_authoritative(&bytes).is_err());
        for (profile, version) in [(ER_UINT_MAX + 1, 1), (1, u64::MAX)] {
            let mut invalid = input.clone();
            invalid.check_profile_id = profile;
            invalid.check_profile_version = version;
            assert!(CheckRecord::new(invalid).is_err());
        }
    }
}

#[test]
fn method_is_a_typed_opaque_definition_not_a_scope() {
    use evidence_registry::{MethodRecord, MethodRecordInput};
    let input = MethodRecordInput {
        method_profile_id: ER_UINT_MAX,
        method_profile_version: 0,
        method_payload: vec![0xff, 0, 0x80],
        method_label: Some("method\0定義".to_owned()),
    };
    let record = MethodRecord::new(input.clone()).unwrap();
    let bytes = record.authoritative_cbor();
    assert_eq!(record.input(), &input);
    assert_eq!(record.record_type_id().value(), 21);
    assert_eq!(
        record.record_id().as_bytes().as_slice(),
        Sha256::digest(&bytes).as_slice()
    );
    assert_eq!(MethodRecord::decode_authoritative(&bytes).unwrap(), record);
    assert!(ScopeRecord::decode_authoritative(&bytes).is_err());
    for (profile, version) in [(ER_UINT_MAX + 1, 1), (1, u64::MAX)] {
        let mut invalid = input.clone();
        invalid.method_profile_id = profile;
        invalid.method_profile_version = version;
        assert!(MethodRecord::new(invalid).is_err());
    }
}

#[test]
fn scope_constructor_preserves_exact_definition_and_identity() {
    for label in [None, Some(String::new()), Some("範囲\0e\u{301}".to_owned())] {
        let input = ScopeRecordInput {
            scope_profile_id: ER_UINT_MAX,
            scope_profile_version: 0,
            scope_payload: vec![0, 255, 128],
            scope_label: label,
        };
        let record = ScopeRecord::new(input.clone()).unwrap();
        let bytes = record.authoritative_cbor();
        assert_eq!(record.input(), &input);
        assert_eq!(record.record_type_id().value(), 20);
        assert_eq!(
            record.record_id().as_bytes().as_slice(),
            Sha256::digest(&bytes).as_slice()
        );
        assert_eq!(
            StrictRecordFrame::decode_authoritative(&bytes)
                .unwrap()
                .record_id(),
            record.record_id()
        );
        assert_eq!(ScopeRecord::decode_authoritative(&bytes).unwrap(), record);
    }
    for (profile, version) in [(u64::MAX, 1), (1, ER_UINT_MAX + 1)] {
        assert!(ScopeRecord::new(ScopeRecordInput {
            scope_profile_id: profile,
            scope_profile_version: version,
            scope_payload: vec![],
            scope_label: None,
        })
        .is_err());
    }
}
