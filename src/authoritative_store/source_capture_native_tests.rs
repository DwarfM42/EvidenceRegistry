use super::*;

#[cfg(windows)]
#[test]
fn source_capture_native_enumeration_error_is_not_empty_success() {
    let file = File::open(std::env::current_exe().unwrap()).unwrap();
    assert_eq!(native_names(&file, &mut budget(), 1), Err(INVALID));
}
fn budget() -> SelectedEmbeddedTraversalBudget {
    SelectedEmbeddedTraversalBudget {
        entries: 0,
        path_bytes: 0,
        content_bytes: 0,
        retained_path_bytes: 0,
    }
}
#[test]
fn source_capture_native_component_grammar_is_literal() {
    for bad in [
        "",
        ".",
        "..",
        "a/b",
        "a\\b",
        "C:x",
        "a\0b",
        "a.",
        "a ",
        "a\nb",
        "\\\\?\\C:\\",
    ] {
        assert_eq!(component(bad), Err(INVALID), "{bad:?}");
    }
    for good in ["a", "日本語", "é", "e\u{301}"] {
        assert_eq!(component(good), Ok(()));
    }
}
#[test]
fn source_capture_native_entry_budget_charges_before_rejection() {
    let mut budget = budget();
    let mut names = Vec::new();
    assert_eq!(
        append_name(&mut names, "bad/name".into(), &mut budget, 1),
        Err(INVALID)
    );
    assert_eq!(budget.entries, 1);
    budget.entries = SELECTED_EMBEDDED_CAPTURE_MAX_FILES;
    assert_eq!(
        append_name(&mut names, "a".into(), &mut budget, 1),
        Err(LIMIT)
    );
    assert!(names.is_empty());
}
#[cfg(windows)]
fn record(name: &[u16]) -> Vec<u8> {
    let mut bytes = vec![0; 104 + name.len() * 2];
    bytes[60..64].copy_from_slice(&((name.len() * 2) as u32).to_le_bytes());
    for (to, ch) in bytes[104..].chunks_exact_mut(2).zip(name) {
        to.copy_from_slice(&ch.to_le_bytes());
    }
    bytes
}
#[cfg(windows)]
#[test]
fn source_capture_windows_parser_accepts_exact_utf16() {
    let bytes = record(&"日本語".encode_utf16().collect::<Vec<_>>());
    let mut names = Vec::new();
    parse_windows_names(&bytes, &mut names, &mut budget(), 1).unwrap();
    assert_eq!(names, ["日本語"]);
}
#[cfg(windows)]
#[test]
fn source_capture_windows_parser_rejects_malformed_chains_and_utf16() {
    let good = record(&[b'a' as u16]);
    let mut cases = vec![vec![], vec![0; 103], record(&[0xd800])];
    for next in [1u32, 104, 112, u32::MAX] {
        let mut bytes = good.clone();
        bytes[0..4].copy_from_slice(&next.to_le_bytes());
        cases.push(bytes);
    }
    for len in [0u32, 1, 4, u32::MAX] {
        let mut bytes = good.clone();
        bytes[60..64].copy_from_slice(&len.to_le_bytes());
        cases.push(bytes);
    }
    let mut bytes = good.clone();
    bytes[68] = 25;
    cases.push(bytes);
    for bytes in cases {
        assert_eq!(
            parse_windows_names(&bytes, &mut Vec::new(), &mut budget(), 1),
            Err(INVALID)
        );
    }
}
