//! Thin public-only consumer; fixed DEMO Policy, not a generic authority CLI.
//! Local binding/prompt/approvals are untrusted locators, never capabilities.
//! Private workspace: argv, transcripts and captured files can contain secrets.
use ai_agent_evidence_binder::{bridge::request_binding, ledger::LedgerWriter, RequestBinding};
use evidence_registry::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};
type Result<T> = std::result::Result<T, String>;
#[path = "support/dispatch.rs"]
mod dispatch;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    schema: u32,
    policy: String,
    source_root: String,
    request: RequestBinding,
    target: RequestBinding,
    capture_policy: RequestBinding,
}
struct Args {
    flags: BTreeMap<String, String>,
    argv: Vec<String>,
}
impl Args {
    fn parse(values: Vec<String>) -> Result<Self> {
        let mut flags = BTreeMap::new();
        let mut iter = values.into_iter();
        let mut argv = vec![];
        while let Some(key) = iter.next() {
            if key == "--" {
                argv.extend(iter);
                break;
            }
            if !key.starts_with("--") {
                return Err("expected_named_flag".into());
            }
            let value = if key == "--approve-demo-policy" {
                "yes".into()
            } else {
                iter.next().ok_or("missing_flag_value")?
            };
            if flags.insert(key, value).is_some() {
                return Err("duplicate_flag".into());
            }
        }
        Ok(Self { flags, argv })
    }
    fn take(&mut self, key: &str) -> Result<String> {
        self.flags
            .remove(key)
            .ok_or_else(|| format!("missing_{key}"))
    }
    fn number(&mut self, key: &str) -> Result<u64> {
        let s = self.take(key)?;
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err("unsigned_decimal_required".into());
        }
        s.parse().map_err(|_| "numeric_range".into())
    }
    fn done(&self) -> Result<()> {
        if !self.flags.is_empty() || !self.argv.is_empty() {
            Err("unknown_or_inapplicable_arguments".into())
        } else {
            Ok(())
        }
    }
}
fn checked<T, E: std::fmt::Debug>(value: std::result::Result<T, E>) -> Result<T> {
    value.map_err(|e| format!("{e:?}"))
}
fn exact_id(s: &str) -> Result<Vec<u8>> {
    if s.len() != 64
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("id_must_be_64_lowercase_hex".into());
    }
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| {
            u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16)
                .map_err(|_| "invalid_hex".into())
        })
        .collect()
}
fn rid(r: &JournalReference) -> RecordId {
    RecordId::try_from(r.event_record_id().as_bytes().as_slice()).expect("Core Record ID width")
}
fn clean_path(p: &Path) -> Result<()> {
    if p.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("parent_path_component".into());
    }
    Ok(())
}
fn fresh_path(p: &Path) -> Result<PathBuf> {
    clean_path(p)?;
    if fs::symlink_metadata(p).is_ok() {
        return Err("fresh_path_required".into());
    }
    let parent = p
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    Ok(checked(fs::canonicalize(parent))?.join(p.file_name().ok_or("missing_leaf")?))
}
fn existing(p: &Path) -> Result<PathBuf> {
    clean_path(p)?;
    checked(fs::canonicalize(p))
}
fn key(p: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from(p.to_string_lossy().to_lowercase())
    }
    #[cfg(not(windows))]
    {
        p.to_path_buf()
    }
}
fn overlaps(a: &Path, b: &Path) -> bool {
    key(a).starts_with(key(b)) || key(b).starts_with(key(a))
}
fn write_new(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut f = checked(OpenOptions::new().write(true).create_new(true).open(path))?;
    checked(f.write_all(&checked(serde_json::to_vec_pretty(value))?))?;
    checked(f.sync_all())
}
fn read_binding(root: &Path) -> Result<Binding> {
    let f = checked(fs::File::open(root.join("binding.json")))?;
    let mut bytes = vec![];
    checked(f.take(65_537).read_to_end(&mut bytes))?;
    if bytes.len() > 65_536 {
        return Err("binding_limit".into());
    }
    let b: Binding = checked(serde_json::from_slice(&bytes))?;
    if b.schema != 1 || b.policy != "fixed-demo-not-semantic-review" {
        return Err("unsupported_binding".into());
    }
    Ok(b)
}
fn init(mut args: Args) -> Result<(i32, Value)> {
    let root = fresh_path(Path::new(&args.take("--workspace")?))?;
    let source = existing(Path::new(&args.take("--source")?))?;
    if overlaps(&root, &source) {
        return Err("source_workspace_overlap".into());
    }
    let registry = checked(RegistryId::try_from(
        exact_id(&args.take("--registry-id")?)?.as_slice(),
    ))?;
    let attempt = checked(FreezeAttemptId::try_from(
        exact_id(&args.take("--target-attempt")?)?.as_slice(),
    ))?;
    let files = args.number("--source-files")?;
    let bytes = args.number("--source-bytes")?;
    if files == 0 || files > 256 || bytes == 0 || bytes > 67_108_864 {
        return Err("source_limits_out_of_range".into());
    }
    args.take("--approve-demo-policy")?;
    args.done()?;
    checked(fs::create_dir(&root))?; // never overwrite or repair partial initialization
    let mut store = checked(AuthoritativeRegistryStore::initialize_selected_profile(
        root.join("store"),
        registry,
        "binder-fixed-demo",
    ))?;
    let scope = |profile| {
        ScopeRecord::new(ScopeRecordInput {
            scope_profile_id: profile,
            scope_profile_version: 1,
            scope_payload: vec![],
            scope_label: None,
        })
    };
    let freeze_scope = checked(scope(2))?;
    let review_scope = checked(scope(1))?;
    checked(store.stage_scope_record(&freeze_scope))?;
    checked(store.stage_scope_record(&review_scope))?;
    let method = checked(MethodRecord::new(MethodRecordInput {
        method_profile_id: 1,
        method_profile_version: 1,
        method_payload: vec![],
        method_label: None,
    }))?;
    let check = checked(CheckRecord::new(CheckRecordInput {
        check_profile_id: 1,
        check_profile_version: 1,
        check_payload: vec![],
        check_label: None,
    }))?;
    checked(store.stage_method_record(&method))?;
    checked(store.stage_check_record(&check))?;
    let checks = checked(CheckSetRecord::new(CheckSetRecordInput {
        check_refs: vec![check.record_id()],
    }))?;
    checked(store.stage_check_set_record(&checks))?;
    let fp = checked(store.register_selected_minimal_policy(freeze_scope.record_id()))?;
    let rp = checked(store.register_selected_review_policy(
        review_scope.record_id(),
        vec![checked(ReviewAdmissionReviewRequirement::new(
            1,
            review_scope.record_id(),
            method.record_id(),
            checks.record_id(),
            1,
        ))?],
        Some(vec![1]),
        Some(vec![1]),
        Some(vec![1, 2]),
    ))?;
    let prepared = checked(store.prepare_selected_embedded_freeze_with_limits(
        SelectedEmbeddedFreezePreparationInput {
            source_root: source.clone(),
            freeze_attempt_id: attempt,
            policy_record_id: rid(&fp),
        },
        SelectedEmbeddedCaptureLimits {
            max_files: files as usize,
            max_content_bytes: bytes as usize,
        },
    ))?;
    let target = checked(store.commit_prepared_selected_embedded_freeze(prepared))?;
    let request = checked(
        store.record_selected_review_request(SelectedReviewRequestInput {
            freeze_authority: target,
            review_policy_record_id: rid(&rp),
            review_role_id: 1,
        }),
    )?;
    let binding = Binding {
        schema: 1,
        policy: "fixed-demo-not-semantic-review".into(),
        source_root: source.to_str().ok_or("path_encoding")?.into(),
        request: request_binding(request.request_event_reference()),
        target: request_binding(request.freeze_authority().committed_event_reference()),
        capture_policy: request_binding(&fp),
    };
    checked(fs::create_dir(root.join("outputs")))?;
    checked(fs::create_dir(root.join("approvals")))?;
    drop(checked(LedgerWriter::open(root.join("ledger")))?);
    write_new(&root.join("binding.json"), &binding)?;
    // Copy identities only from the Store-returned Request, never from an Agent
    // or a re-read locator. This JSON is explanatory material, not authority.
    let review_request = request.request();
    let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let prompt = json!({
        "schema": 1, "untrusted_prompt_material": true,
        "request": request_binding(request.request_event_reference()),
        "target": request_binding(review_request.freeze_authority_ref()),
        "request_record": {
            "record_id": hex(review_request.record_id().as_bytes()),
            "review_package_anchor_id": hex(review_request.review_package_anchor_id().as_bytes()),
            "freeze_authority_ref": request_binding(review_request.freeze_authority_ref()),
            "manifest_id": hex(review_request.manifest_id().as_bytes()),
            "review_role_id": review_request.review_role_id(),
            "required_checks_ref": hex(review_request.required_checks_ref().as_bytes()),
            "policy_authority_ref": request_binding(review_request.policy_authority_ref()),
            "review_policy_record_id": hex(review_request.policy_authority_ref().event_record_id().as_bytes()),
            "review_scope_ref": hex(review_request.review_scope_ref().as_bytes()),
            "review_method_ref": hex(review_request.review_method_ref().as_bytes()),
            "operation_start_journal_ref": request_binding(review_request.operation_start_journal_ref()),
            "terminal_authority_closure_sha256": review_request.terminal_authority_closure_sha256().map(|bytes| hex(bytes)),
        },
        "source_root": binding.source_root, "policy": binding.policy,
        "scope_method_check": "Fixed selected-profile DEMO definitions, not proof of review execution or semantic correctness.",
        "method_status_registry": {"1":"VALID","2":"INVALID","3":"ENVIRONMENTALLY_DEGRADED"},
        "finding_state_registry": {"1":"NO_BLOCKING","2":"BLOCKING","3":"INDETERMINATE"},
        "grammar": "Submission schema 1; exact five-field request redundancy; integer statuses 1..3; reason_codes sorted unique strings; findings exact retained lowercase-hex Finding Record IDs, not prose; reviewer_metadata string or null; artifacts unique portable ASCII relative paths under cwd, no absolute paths, traversal, links or external selections. Unknown/duplicate JSON keys rejected. Bounded submission 262144 bytes. Descriptive findings belong in mandatory review.txt even when no retained Finding Record exists.",
        "instructions": "Perform a fresh review of the approved target; do not change Policy or Request, import prior results, or retry automatically. Write review.txt and strict submission.json in cwd. Preserve actual findings and limitations in review.txt; do not invent Finding Record IDs. Source paths are locators, not proof of equality to retained target. Internal tools/delegations are Agent reports, not Binder observations. This fixed demo Policy accepts only method_status 1 and finding_state 1. Choose truthful supported statuses, not the value that passes. See Binder submission grammar for status/reason constraints. No stdin; caller supplies prompt/tool permissions in literal argv. Output capture includes all regular files under cwd; submission.artifacts is a candidate list, not permission or completeness proof. Mandatory review.txt is captured even when omitted from that list. Never claim semantic truth from acceptance.",
        "submission_shape_not_a_result": {"schema":1,"request":binding.request,"method_status":null,"finding_state":null,"reason_codes":[],"findings":[],"reviewer_metadata":null,"artifacts":["review.txt"]}
    });
    write_new(&root.join("prompt.json"), &prompt)?;
    drop(store);
    let cold = checked(AuthoritativeRegistryStore::open_selected_profile(
        root.join("store"),
    ))?;
    let q = resolve(&cold, &binding.request)?;
    checked(cold.validate_selected_review_request(q))?;
    Ok((
        0,
        json!({"operation":"init","policy":binding.policy,"request":binding.request,"target":binding.target,"workspace":root,"binding_authoritative":false,"untrusted_prompt_material":true,"request_record":prompt["request_record"]}),
    ))
}
fn resolve(
    store: &AuthoritativeRegistryStore,
    binding: &RequestBinding,
) -> Result<JournalReference> {
    exact_id(&binding.registry_id)?;
    exact_id(&binding.entry_hash)?;
    exact_id(&binding.event_record_id)?;
    store
        .retained_journal()
        .references()
        .find(|r| request_binding(r) == *binding)
        .ok_or_else(|| "exact_reference_not_retained".into())
}
fn execute() -> Result<(i32, Value)> {
    let mut values = std::env::args().skip(1);
    let command = values
        .next()
        .ok_or("expected_init_run_inspect_or_fake-reviewer")?;
    let args = Args::parse(values.collect())?;
    match command.as_str() {
        "init" => init(args),
        "run" => dispatch::run(args),
        "fake-reviewer" => dispatch::fake(args),
        "inspect" => inspect(args),
        _ => Err("unknown_command".into()),
    }
}
fn inspect(mut args: Args) -> Result<(i32, Value)> {
    let root = existing(Path::new(&args.take("--workspace")?))?;
    args.done()?;
    // Deliberately does not read binding.json: even damaged locator material must
    // not hide the valid ledger prefix, failures or unmatched Store Requests.
    let report = checked(ai_agent_evidence_binder::inspection::inspect(
        root.join("store"),
        root.join("ledger"),
    ))?;
    let healthy = report.ledger.boundary.is_none()
        && report.store.status == "ValidatedCapturedView"
        && report.candidate_enumeration_complete;
    let mut value = checked(serde_json::to_value(report))?;
    if let Some(frames) = value["ledger"]["frames"].as_array_mut() {
        for frame in frames {
            if frame["event"]["kind"] == "Intent" {
                frame["event"]["payload"]["command"] = json!(["<private literal argv omitted>"]);
            }
        }
    }
    value["operation"] = json!("inspect");
    value["ledger"]["frames_redacted"] = json!(true);
    value["ledger"]["redaction_note"] = json!("Displayed frames omit argv; not original ledger bytes. Counts/digests concern the private original. Other Agent material still requires privacy review.");
    Ok((if healthy { 0 } else { 14 }, value))
}
fn main() {
    let (code, output) = match execute() {
        Ok(result) => result,
        Err(error) => (
            2,
            json!({"operation":"error","error":error,"detail":"No retry or repair performed."}),
        ),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&output).expect("JSON output")
    );
    std::process::exit(code);
}
