use ai_agent_evidence_binder::{parse_submission, RequestBinding, SubmissionError};

fn binding() -> RequestBinding {
    RequestBinding {
        registry_id: "11".repeat(32),
        entry_index: 4,
        entry_hash: "22".repeat(32),
        event_type_id: 300,
        event_record_id: "33".repeat(32),
    }
}
fn valid() -> Vec<u8> {
    format!(r#"{{"schema":1,"request":{{"registry_id":"{}","entry_index":4,"entry_hash":"{}","event_type_id":300,"event_record_id":"{}"}},"method_status":1,"finding_state":1,"reason_codes":[],"findings":[],"reviewer_metadata":"CLEAN is an Agent claim, not truth","artifacts":["review.md","submission.json"]}}"#, "11".repeat(32), "22".repeat(32), "33".repeat(32)).into_bytes()
}
fn changed(old: &str, new: &str) -> Vec<u8> {
    String::from_utf8(valid())
        .unwrap()
        .replace(old, new)
        .into_bytes()
}
#[test]
fn structured_clean_is_only_a_claim() {
    let parsed = parse_submission(&valid(), &binding()).unwrap();
    assert_eq!(parsed.method_status, 1);
    assert_eq!(
        parsed.reviewer_metadata.as_deref(),
        Some("CLEAN is an Agent claim, not truth")
    );
}
#[test]
fn unknown_duplicate_missing_and_trailing_json_rejected() {
    for input in [
        changed("\"schema\":1", "\"schema\":1,\"schema\":1"),
        changed("\"schema\":1", "\"schema\":1,\"role\":1"),
        changed("\"schema\":1,", ""),
        [valid(), b" true".to_vec()].concat(),
        changed("\"entry_index\":4", "\"entry_index\":4,\"entry_index\":4"),
    ] {
        assert_eq!(
            parse_submission(&input, &binding()),
            Err(SubmissionError::Grammar)
        );
    }
}
#[test]
fn every_request_field_is_exact_untrusted_redundancy() {
    for (old, new) in [
        ("\"entry_index\":4", "\"entry_index\":5"),
        ("\"event_type_id\":300", "\"event_type_id\":301"),
    ] {
        assert!(parse_submission(&changed(old, new), &binding()).is_err());
    }
    for key in ["11", "22", "33"] {
        assert!(parse_submission(&changed(&key.repeat(32), &"44".repeat(32)), &binding()).is_err());
    }
}
#[test]
fn status_and_set_grammar_never_normalizes() {
    for input in [
        changed("\"method_status\":1", "\"method_status\":0"),
        changed("\"finding_state\":1", "\"finding_state\":4"),
        changed("\"schema\":1", "\"schema\":2"),
        changed("\"reason_codes\":[]", "\"reason_codes\":[\"z\",\"a\"]"),
        changed("\"reason_codes\":[]", "\"reason_codes\":[\"a\",\"a\"]"),
    ] {
        assert!(parse_submission(&input, &binding()).is_err());
    }
}
#[test]
fn external_ambiguous_and_duplicate_artifacts_are_rejected() {
    for name in [
        "../secret",
        "/secret",
        "C:/secret",
        "a\\\\b",
        "a//b",
        "a/./b",
        "NUL",
        "file.",
        "aux.txt",
        "COM1.log",
    ] {
        assert!(
            parse_submission(&changed("review.md", name), &binding()).is_err(),
            "{name}"
        );
    }
    assert!(parse_submission(&changed("review.md", "submission.json"), &binding()).is_err());
    assert!(parse_submission(
        &changed("\"artifacts\":", "\"digest\":\"fake\",\"artifacts\":"),
        &binding()
    )
    .is_err());
}
#[test]
fn identity_spelling_and_width_are_not_repaired() {
    for value in ["AB".repeat(32), "0".repeat(63), "g".repeat(64)] {
        assert!(parse_submission(&changed(&"11".repeat(32), &value), &binding()).is_err());
    }
}
#[test]
fn input_limit_precedes_json_allocation() {
    assert_eq!(
        parse_submission(&vec![b' '; 262_145], &binding()),
        Err(SubmissionError::Limit)
    );
}
#[test]
fn findings_preserve_order_and_duplicates_not_capture_authority() {
    let input = changed(
        "\"findings\":[]",
        &format!(
            "\"findings\":[\"{}\",\"{}\",\"{}\"]",
            "66".repeat(32),
            "44".repeat(32),
            "66".repeat(32)
        ),
    );
    let parsed = parse_submission(&input, &binding()).unwrap();
    assert_eq!(
        parsed.findings,
        vec!["66".repeat(32), "44".repeat(32), "66".repeat(32)]
    );
}
