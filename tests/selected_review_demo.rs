//! Build the actual native example once per test process, including in fresh targets.
//! These process tests retain their fresh roots under target/; they never clean unknown work.
#![cfg(any(windows, target_os = "linux", target_os = "macos"))]

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn controlled_noncompliant_claim_completes_rejected_policy_and_cold_replays() {
    let binary = example();
    let root = fresh_root("rejected");
    let run = Command::new(&binary)
        .args(["run", "rejected"])
        .arg(&root)
        .output()
        .unwrap();
    let text = stdout(&run);
    assert!(text.contains("policy=GateUnsatisfied disposition=ReviewAdmissionRejected"));
    assert!(text.contains("event_types=[1, 400, 400, 100, 101, 300, 301, 303]"));
    assert!(text.contains("disposition_id=2"));
    assert!(text.contains("retained_claim method_status=2 finding_state=1"));
    assert!(text.contains("cold_readback=VERIFIED separate_process=true"));
    let cold = Command::new(binary)
        .arg("inspect")
        .arg(root)
        .output()
        .unwrap();
    assert!(stdout(&cold).contains("disposition_id=2"));
}

#[test]
fn invalid_claim_is_preterminal_without_result_or_admission_and_cold_request_is_unresolved() {
    let binary = example();
    let root = fresh_root("invalid");
    let run = Command::new(&binary)
        .args(["run", "invalid"])
        .arg(&root)
        .output()
        .unwrap();
    let text = stdout(&run);
    assert!(text.contains("preterminal=INVALID error=ResultConstruction"));
    assert!(text.contains("event_types=[1, 400, 400, 100, 101, 300]"));
    assert!(text.contains("terminal_admission=NONE policy_completion=NOT_ESTABLISHED"));
    assert!(text.contains("result_publication=NONE journal_head_unchanged=true"));
    assert!(!text.contains("publication_receipt=LIVE"));
    assert!(!root
        .join("store/journal/00000000000000000006.cbor")
        .exists());
    let cold = Command::new(binary)
        .arg("inspect")
        .arg(root)
        .output()
        .unwrap();
    assert!(stdout(&cold).contains("terminal_admission=NONE"));
}

#[test]
fn generic_scope_is_unsupported_not_completed_policy_rejection() {
    let binary = example();
    let root = fresh_root("unsupported");
    let run = Command::new(&binary)
        .args(["run", "unsupported"])
        .arg(&root)
        .output()
        .unwrap();
    let text = stdout(&run);
    assert!(text.contains("preterminal=UNSUPPORTED error=PolicyScopeApplicabilityUnavailable"));
    assert!(text.contains("route=GENERIC_NOT_SELECTED"));
    assert!(text.contains("event_types=[1, 400, 400, 100, 101, 300, 301]"));
    assert!(text.contains("terminal_admission=NONE policy_completion=NOT_ESTABLISHED"));
    assert!(text.contains("journal_head_unchanged=true"));
    assert!(!text.contains("publication_receipt=LIVE"));
    assert!(!root
        .join("store/journal/00000000000000000007.cbor")
        .exists());
    let cold = Command::new(binary)
        .arg("inspect")
        .arg(root)
        .output()
        .unwrap();
    assert!(stdout(&cold).contains("terminal_admission=NONE"));
}

#[test]
fn fresh_root_refusal_preserves_existing_files_and_history_without_cleanup() {
    let binary = example();
    let root = fresh_root("no-overwrite");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("owner.txt"), b"unknown work must survive").unwrap();
    let refused = Command::new(&binary)
        .args(["run", "accepted"])
        .arg(&root)
        .output()
        .unwrap();
    assert_eq!(refused.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("fresh_root_required"));
    assert_eq!(
        std::fs::read(root.join("owner.txt")).unwrap(),
        b"unknown work must survive"
    );
    assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);

    let existing = fresh_root("existing-store");
    stdout(
        &Command::new(&binary)
            .args(["run", "accepted"])
            .arg(&existing)
            .output()
            .unwrap(),
    );
    let before = snapshot(&existing);
    let refused = Command::new(&binary)
        .args(["run", "rejected"])
        .arg(&existing)
        .output()
        .unwrap();
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(
        snapshot(&existing),
        before,
        "refusal must not retry, erase or append history"
    );

    let absent = fresh_root("bad-usage");
    let invalid = Command::new(&binary)
        .args(["run", "invented"])
        .arg(&absent)
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(2));
    assert!(!absent.exists());
}

fn snapshot(root: &std::path::Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn visit(root: &std::path::Path, here: &std::path::Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in std::fs::read_dir(here).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                visit(root, &entry.path(), files);
            } else {
                files.push((
                    entry.path().strip_prefix(root).unwrap().into(),
                    std::fs::read(entry.path()).unwrap(),
                ));
            }
        }
    }
    let mut files = vec![];
    visit(root, root, &mut files);
    files.sort();
    files
}

#[test]
fn cold_readback_uses_retained_bytes_and_detects_retained_payload_corruption() {
    let binary = example();
    let root = fresh_root("tamper");
    let run = Command::new(&binary)
        .args(["run", "accepted"])
        .arg(&root)
        .output()
        .unwrap();
    let text = stdout(&run);
    let payload = PathBuf::from(
        text.lines()
            .find_map(|line| line.strip_prefix("retained_payload="))
            .unwrap(),
    );
    assert!(payload.starts_with(std::fs::canonicalize(&root).unwrap()));
    std::fs::write(
        root.join("source/target.txt"),
        b"changed original, not retained evidence",
    )
    .unwrap();
    stdout(
        &Command::new(&binary)
            .arg("inspect")
            .arg(&root)
            .output()
            .unwrap(),
    );
    // Explicit negative fixture mutation, never a fabricated positive authority.
    std::fs::write(payload.join("target.txt"), b"corrupted retained payload").unwrap();
    let cold = Command::new(binary)
        .arg("inspect")
        .arg(&root)
        .output()
        .unwrap();
    assert_eq!(cold.status.code(), Some(1));
    assert!(!String::from_utf8_lossy(&cold.stdout).contains("disposition_id=1"));
    assert!(String::from_utf8_lossy(&cold.stderr).contains("demo_failed"));
    assert!(root.exists(), "negative evidence is retained");
}

#[test]
fn native_example_is_bound_to_the_running_test_output_and_profile() {
    let test = std::env::current_exe().unwrap();
    let profile_dir = test.parent().unwrap().parent().unwrap();
    let executable = example();
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
    assert!(!std::fs::read(executable).unwrap().is_empty());
}

fn example() -> PathBuf {
    static EXECUTABLE: OnceLock<PathBuf> = OnceLock::new();
    EXECUTABLE
        .get_or_init(|| build_native_example("evidence-registry", "selected_review_demo"))
        .clone()
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

fn fresh_root(label: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!(
            "selected-review-demo-test-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
}

fn stdout(output: &Output) -> String {
    assert!(
        output.status.success(),
        "exit {:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert!(
        output.stdout.len() <= 32 * 1024,
        "summary must stay bounded"
    );
    String::from_utf8(output.stdout.clone()).unwrap()
}

#[test]
fn actual_example_accepts_and_a_new_process_revalidates_exact_history_and_payload() {
    let binary = example();
    let root = fresh_root("accepted");
    let run = Command::new(&binary)
        .args(["run", "accepted"])
        .arg(&root)
        .output()
        .unwrap();
    let text = stdout(&run);
    assert!(text.contains("reviewer=CONTROLLED_FAKE_NOT_AI"));
    assert!(text.contains("policy=Satisfied disposition=ReviewAdmissionAccepted"));
    assert!(text.contains("publication_receipt=LIVE"));
    assert!(text.contains("cold_readback=VERIFIED separate_process=true"));
    assert!(text.contains("event_types=[1, 400, 400, 100, 101, 300, 301, 302]"));
    assert!(text.contains("payload_files=2"));
    let cold = Command::new(&binary)
        .arg("inspect")
        .arg(&root)
        .output()
        .unwrap();
    let cold_text = stdout(&cold);
    assert!(cold_text.contains("disposition_id=1"));
    assert!(cold_text.contains("live_publication_receipt=NOT_RECOVERED"));
    let run_head = text.lines().find(|line| line.starts_with("head ")).unwrap();
    assert!(
        cold_text.lines().any(|line| line == run_head),
        "cold head must equal the producer's exact reference"
    );
    assert!(root.join("source/nested/check.txt").is_file());
    assert!(root.join("store/registry/genesis.cbor").is_file());
}
