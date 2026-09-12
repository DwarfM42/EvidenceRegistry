//! Native commands exercising the public-only companion, not fabricated Records.
use evidence_registry::AuthoritativeRegistryStore;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::{
        atomic::{AtomicU64, Ordering},
        OnceLock,
    },
};
static NEXT: AtomicU64 = AtomicU64::new(0);

#[test]
fn native_example_is_bound_to_the_running_test_output_and_profile() {
    let test = std::env::current_exe().unwrap();
    let profile_dir = test.parent().unwrap().parent().unwrap();
    let executable = binary();
    assert!(
        executable.starts_with(profile_dir),
        "{} must belong to the running test output {}",
        executable.display(),
        profile_dir.display()
    );
    assert_eq!(
        executable.parent().unwrap().file_name().unwrap(),
        "examples"
    );
    assert_eq!(
        executable.parent().unwrap().parent().unwrap().file_name(),
        profile_dir.file_name()
    );
    assert!(!fs::read(executable).unwrap().is_empty());
}

fn binary() -> &'static Path {
    static EXE: OnceLock<PathBuf> = OnceLock::new();
    EXE.get_or_init(|| build_native_example("ai-agent-evidence-binder", "binder"))
        .as_path()
}
// Native command tests intentionally build for Cargo's host, not a cross runner.
// Keep a scoped nested target below the running test profile: --target-dir, a
// configured build.target, and a target-triple directory cannot redirect lookup
// to an unrelated/default executable. Only the workspace's dev/release profiles
// are covered; fail explicitly rather than guess a custom profile's semantics.
fn build_native_example(package: &str, name: &str) -> PathBuf {
    use sha2::{Digest, Sha256};

    let test = std::env::current_exe().unwrap();
    let deps = test.parent().unwrap();
    assert_eq!(deps.file_name().unwrap(), "deps");
    let profile_dir = deps.parent().unwrap();
    let profile = profile_dir.file_name().unwrap().to_str().unwrap();
    let cargo_profile = match profile {
        "debug" => "dev",
        "release" => "release",
        other => panic!("native example tests do not cover custom profile {other}"),
    };
    let version = Command::new(env!("CARGO"))
        .arg("-vV")
        .env("RUSTUP_AUTO_INSTALL", "0")
        .output()
        .unwrap();
    assert!(version.status.success(), "cannot determine Cargo host");
    let version_text = String::from_utf8(version.stdout.clone()).unwrap();
    let host = version_text
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .expect("Cargo verbose version must identify its native host");
    let logs = profile_dir.join(format!(
        "native-log-{}-{:x}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    // Never erase earlier build evidence.
    std::fs::create_dir(&logs).unwrap();
    // Keep the cache leaf short for native MSVC linker MAX_PATH limits. Cargo
    // always builds/checks freshness; this is never an existence-only lookup.
    let target = profile_dir.join("n");
    let mut command = Command::new(env!("CARGO"));
    command
        .args([
            "build",
            "--locked",
            "--offline",
            "-p",
            package,
            "--example",
            name,
            "--profile",
            cargo_profile,
            "--target",
            host,
            "--target-dir",
        ])
        .arg(&target)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .current_dir(env!("CARGO_MANIFEST_DIR"));
    std::fs::write(logs.join("cargo-version.txt"), &version.stdout).unwrap();
    std::fs::write(logs.join("build-command.txt"), format!("{command:?}\n")).unwrap();
    let output = command.output().unwrap();
    std::fs::write(logs.join("build.stdout"), &output.stdout).unwrap();
    std::fs::write(logs.join("build.stderr"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "actual example build failed (logs retained at {}): {}",
        logs.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let executable = target
        .join(host)
        .join(profile)
        .join("examples")
        .join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    let bytes = std::fs::read(&executable).unwrap_or_else(|error| {
        panic!("missing native artifact {}: {error}", executable.display())
    });
    assert!(!bytes.is_empty());
    let identity =
        format!(
        "native_example test={} profile={profile} host={host} executable={} bytes={} sha256={:x}\n",
        test.display(), executable.display(), bytes.len(), Sha256::digest(&bytes)
    );
    std::fs::write(logs.join("artifact.txt"), &identity).unwrap();
    eprint!("{identity}");
    executable
}

struct Fixture {
    base: PathBuf,
    workspace: PathBuf,
    source: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("target")
            .join(format!(
                "consumer-example-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(base.join("source")).unwrap();
        fs::write(
            base.join("source/target.txt"),
            b"demonstration review target\n",
        )
        .unwrap();
        Self {
            workspace: base.join("workspace"),
            source: base.join("source"),
            base,
        }
    }
    fn init(&self) -> Output {
        Command::new(binary())
            .args(["init", "--workspace"])
            .arg(&self.workspace)
            .arg("--source")
            .arg(&self.source)
            .args([
                "--registry-id",
                &"61".repeat(32),
                "--target-attempt",
                &"62".repeat(32),
                "--source-files",
                "8",
                "--source-bytes",
                "65536",
                "--approve-demo-policy",
            ])
            .output()
            .unwrap()
    }
}
fn json(output: &Output, exit: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(exit),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
#[test]
fn init_publishes_real_target_and_exact_request_without_overwrite() {
    let f = Fixture::new();
    let result = json(&f.init(), 0);
    assert_eq!(result["operation"], "init");
    assert_eq!(result["policy"], "fixed-demo-not-semantic-review");
    let locator: Value =
        serde_json::from_slice(&fs::read(f.workspace.join("binding.json")).unwrap()).unwrap();
    let store =
        AuthoritativeRegistryStore::open_selected_profile(f.workspace.join("store")).unwrap();
    let request = store
        .retained_journal()
        .references()
        .find(|r| r.entry_index().value() == locator["request"]["entry_index"].as_u64().unwrap())
        .unwrap();
    store.validate_selected_review_request(request).unwrap();
    assert_eq!(
        store
            .retained_journal()
            .references()
            .filter(|r| r.event_type_id().value() == 300)
            .count(),
        1
    );
    let bytes = fs::read(f.workspace.join("binding.json")).unwrap();
    json(&f.init(), 2);
    assert_eq!(fs::read(f.workspace.join("binding.json")).unwrap(), bytes);
    assert!(f.workspace.join("prompt.json").is_file());
    assert!(f.workspace.join("ledger").is_file());
}

impl Fixture {
    fn run(&self, attempt: &str, capture: &str, mode: &str, predecessor: Option<&str>) -> Output {
        let mut command = Command::new(binary());
        command.args(["run", "--workspace"]).arg(&self.workspace);
        command.args(["--attempt", attempt, "--capture-attempt", capture]);
        command
            .arg("--output")
            .arg(self.workspace.join("outputs").join(attempt));
        command.arg("--approve-input-root").arg(&self.source);
        command.args([
            "--tool-permissions",
            "fake-only-no-network",
            "--runtime-ms",
            "10000",
            "--pipe-drain-ms",
            "1000",
            "--stdout-bytes",
            "65536",
            "--stderr-bytes",
            "65536",
            "--output-bytes",
            "1048576",
            "--output-files",
            "16",
            "--open-descriptors",
            "64",
        ]);
        if let Some(previous) = predecessor {
            command.args(["--predecessor", previous]);
        }
        command.arg("--").arg(binary()).arg("fake-reviewer");
        command.arg("--workspace").arg(&self.workspace);
        command.args(["--mode", mode]);
        command.output().unwrap()
    }
}

#[test]
fn literal_fake_dispatch_retains_result_without_replacing_request() {
    let f = Fixture::new();
    let initialized = json(&f.init(), 0);
    let output = json(&f.run("first", &"63".repeat(32), "clean", None), 0);
    assert_eq!(output["operation"], "run");
    assert_eq!(output["disposition"], "accepted");
    assert_eq!(output["result_recorded"], true);
    assert_eq!(output["request"], initialized["request"]);
    assert_ne!(output["output_freeze"], initialized["target"]);
    assert!(f.workspace.join("outputs/first/review.txt").is_file());
    let ledger = ai_agent_evidence_binder::ledger::read_ledger(f.workspace.join("ledger")).unwrap();
    assert_eq!(ledger.attempts().len(), 1);
    assert_eq!(
        ledger.attempts()[0].request.entry_hash,
        initialized["request"]["entry_hash"]
    );
    assert!(f.base.is_dir()); // All evidence, including failed test runs, is retained.
}

#[test]
fn cold_inspection_before_dispatch_is_read_only_and_shows_unmatched_request() {
    let f = Fixture::new();
    json(&f.init(), 0);
    let before = fs::read(f.workspace.join("ledger")).unwrap();
    let output = Command::new(binary())
        .args(["inspect", "--workspace"])
        .arg(&f.workspace)
        .output()
        .unwrap();
    let report = json(&output, 0);
    assert_eq!(report["read_only"], true);
    assert_eq!(report["counts"]["attempts"], 0);
    assert_eq!(report["counts"]["unmatched_store_requests"], 1);
    assert_eq!(fs::read(f.workspace.join("ledger")).unwrap(), before);
}

#[test]
fn used_capture_id_is_not_recycled_after_process_failure() {
    let f = Fixture::new();
    json(&f.init(), 0);
    json(&f.run("failed", &"63".repeat(32), "nonzero", None), 11);
    let before = fs::read(f.workspace.join("ledger")).unwrap();
    json(
        &f.run("retry", &"63".repeat(32), "clean", Some("failed")),
        2,
    );
    assert_eq!(fs::read(f.workspace.join("ledger")).unwrap(), before);
    assert!(!f.workspace.join("outputs/retry").exists());
}

#[test]
fn malformed_submission_reports_retained_capture_without_claiming_a_result() {
    let f = Fixture::new();
    json(&f.init(), 0);
    let report = json(&f.run("bad", &"63".repeat(32), "malformed", None), 12);
    assert_eq!(report["launch"], "observed_spawn");
    assert_eq!(report["process_exit"], 0);
    assert_eq!(report["result_recorded"], false);
    assert_eq!(report["failure_stage"], "submission");
    assert_eq!(report["output_freeze"]["event_type_id"], 101);
}

fn inspect_workspace(f: &Fixture, exit: i32) -> Value {
    json(
        &Command::new(binary())
            .args(["inspect", "--workspace"])
            .arg(&f.workspace)
            .output()
            .unwrap(),
        exit,
    )
}

fn tree_bytes(root: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    let mut out = std::collections::BTreeMap::new();
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(tree_bytes(&path));
        } else {
            out.insert(path.clone(), fs::read(path).unwrap());
        }
    }
    out
}

#[test]
fn all_attempts_survive_failure_rejection_acceptance_and_cold_readback() {
    let f = Fixture::new();
    let initialized = json(&f.init(), 0);
    let binding = fs::read(f.workspace.join("binding.json")).unwrap();
    json(&f.run("a", &"63".repeat(32), "nonzero", None), 11);
    let before_retry = fs::read(f.workspace.join("ledger")).unwrap();
    json(&f.run("b", &"64".repeat(32), "malformed", None), 2);
    assert_eq!(fs::read(f.workspace.join("ledger")).unwrap(), before_retry);
    json(&f.run("b", &"64".repeat(32), "malformed", Some("a")), 12);
    let rejected = json(&f.run("c", &"65".repeat(32), "rejected", Some("b")), 10);
    assert_eq!(rejected["disposition"], "rejected");
    assert_eq!(rejected["process_exit"], 0);
    json(&f.run("d", &"66".repeat(32), "clean", Some("c")), 0);
    let before = tree_bytes(&f.workspace);
    let report = inspect_workspace(&f, 0);
    assert_eq!(tree_bytes(&f.workspace), before);
    assert_eq!(fs::read(f.workspace.join("binding.json")).unwrap(), binding);
    assert_eq!(report["counts"]["attempts"], 4);
    assert_eq!(report["counts"]["store_requests"], 1);
    assert_eq!(report["counts"]["store_results"], 2);
    assert_eq!(report["counts"]["store_admissions_accepted"], 1);
    assert_eq!(report["counts"]["store_admissions_rejected"], 1);
    assert_eq!(report["counts"]["observed_failure_attempts"], 2);
    for attempt in report["attempts"].as_array().unwrap() {
        assert_eq!(attempt["request"], initialized["request"]);
        assert_eq!(attempt["live_witness"], false);
        assert_eq!(attempt["historical_receipt_recovered"], false);
    }
}

#[test]
fn exact_locator_fields_are_not_replaced_with_latest_request() {
    for field in [
        "registry_id",
        "entry_index",
        "entry_hash",
        "event_type_id",
        "event_record_id",
    ] {
        let f = Fixture::new();
        json(&f.init(), 0);
        let path = f.workspace.join("binding.json");
        let mut binding: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        binding["request"][field] = if field == "entry_index" || field == "event_type_id" {
            serde_json::json!(0)
        } else {
            serde_json::json!("00".repeat(32))
        };
        fs::write(path, serde_json::to_vec(&binding).unwrap()).unwrap();
        let before = tree_bytes(&f.workspace);
        json(&f.run("bad", &"63".repeat(32), "clean", None), 2);
        assert_eq!(tree_bytes(&f.workspace), before);
        assert_eq!(inspect_workspace(&f, 0)["counts"]["attempts"], 0);
    }
}

#[test]
fn inspection_omits_literal_argv_and_keeps_corrupt_prefix_visible() {
    use std::io::Write;
    let f = Fixture::new();
    json(&f.init(), 0);
    json(
        &f.run(
            "private",
            &"63".repeat(32),
            "private-token-DO-NOT-ECHO",
            None,
        ),
        11,
    );
    let report = inspect_workspace(&f, 0);
    assert!(!serde_json::to_string(&report)
        .unwrap()
        .contains("private-token-DO-NOT-ECHO"));
    assert_eq!(report["counts"]["attempts"], 1);
    fs::OpenOptions::new()
        .append(true)
        .open(f.workspace.join("ledger"))
        .unwrap()
        .write_all(b"bad")
        .unwrap();
    let before = tree_bytes(&f.workspace);
    let corrupt = inspect_workspace(&f, 14);
    assert_eq!(corrupt["counts"]["attempts"], 1);
    assert_eq!(corrupt["ledger"]["status"], "CorruptPrefix");
    assert_eq!(tree_bytes(&f.workspace), before);
}

#[test]
fn prompt_binds_the_actual_store_request_anchor_parameters_and_policy() {
    use ai_agent_evidence_binder::bridge::request_binding;
    let f = Fixture::new();
    let initialized = json(&f.init(), 0);
    let prompt: Value =
        serde_json::from_slice(&fs::read(f.workspace.join("prompt.json")).unwrap()).unwrap();
    // Resolve all reference fields against the cold public Store, not a locator
    // or a synthetic positive Request/Anchor fixture.
    let store =
        AuthoritativeRegistryStore::open_selected_profile(f.workspace.join("store")).unwrap();
    let q = store
        .retained_journal()
        .references()
        .find(|reference| {
            serde_json::to_value(request_binding(reference)).unwrap() == initialized["request"]
        })
        .unwrap();
    let recorded = store.validate_selected_review_request(q).unwrap();
    let request = recorded.request();
    let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    assert_eq!(prompt["untrusted_prompt_material"], true);
    assert_eq!(initialized["untrusted_prompt_material"], true);
    assert_eq!(initialized["request_record"], prompt["request_record"]);
    assert_eq!(
        prompt["request"],
        serde_json::to_value(request_binding(recorded.request_event_reference())).unwrap()
    );
    assert_eq!(
        prompt["request_record"],
        serde_json::json!({
            "record_id": hex(request.record_id().as_bytes()),
            "review_package_anchor_id": hex(request.review_package_anchor_id().as_bytes()),
            "freeze_authority_ref": request_binding(request.freeze_authority_ref()),
            "manifest_id": hex(request.manifest_id().as_bytes()),
            "review_role_id": request.review_role_id(),
            "required_checks_ref": hex(request.required_checks_ref().as_bytes()),
            "policy_authority_ref": request_binding(request.policy_authority_ref()),
            "review_policy_record_id": hex(request.policy_authority_ref().event_record_id().as_bytes()),
            "review_scope_ref": hex(request.review_scope_ref().as_bytes()),
            "review_method_ref": hex(request.review_method_ref().as_bytes()),
            "operation_start_journal_ref": request_binding(request.operation_start_journal_ref()),
            "terminal_authority_closure_sha256": request.terminal_authority_closure_sha256().map(|bytes| hex(bytes)),
        })
    );
    assert_ne!(
        prompt["request_record"]["review_policy_record_id"],
        serde_json::from_slice::<Value>(&fs::read(f.workspace.join("binding.json")).unwrap())
            .unwrap()["capture_policy"]["event_record_id"]
    );
    assert!(prompt["submission_shape_not_a_result"]["method_status"].is_null());
    assert!(prompt["submission_shape_not_a_result"]["finding_state"].is_null());
}

#[test]
fn prompt_material_explains_status_grammar_without_prefilling_success() {
    let f = Fixture::new();
    json(&f.init(), 0);
    let prompt: Value =
        serde_json::from_slice(&fs::read(f.workspace.join("prompt.json")).unwrap()).unwrap();
    assert_eq!(
        prompt["method_status_registry"]["3"],
        "ENVIRONMENTALLY_DEGRADED"
    );
    assert_eq!(prompt["finding_state_registry"]["2"], "BLOCKING");
    assert!(prompt["submission_shape_not_a_result"]["method_status"].is_null());
    assert_eq!(prompt["scope_method_check"], "Fixed selected-profile DEMO definitions, not proof of review execution or semantic correctness.");
}
