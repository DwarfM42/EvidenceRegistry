//! Private unit-build checkpoints: real producers, abrupt exit without Drop.
//! None of this module or its control files exist in a normal library build.
use super::*;
use crate::{ledger::CommandLimits, RequestBinding, Submission};
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
type Hook = Box<dyn FnMut(&'static str)>;
thread_local! {
    static HOOK: RefCell<Option<Hook>> = RefCell::new(None);
}
pub(super) fn fire(point: &'static str) {
    HOOK.with(|h| {
        if let Some(h) = h.borrow_mut().as_mut() {
            h(point);
        }
    });
}
include!("../../tests/bridge/fixture.rs");

const OWNER: &str = "bridge::crash_tests::boundary_owner";
const REVIEWER: &str = "bridge::crash_tests::boundary_reviewer";
const INSPECTOR: &str = "bridge::crash_tests::boundary_inspector";

#[derive(serde::Serialize, serde::Deserialize)]
struct Case {
    base: PathBuf,
    point: String,
    number: u8,
    intent: DispatchIntent,
    policy: Vec<u8>,
}

fn inventory(base: &Path) -> BTreeMap<String, String> {
    fn walk(root: &Path, at: &Path, entries: &mut BTreeMap<String, String>) {
        for entry in fs::read_dir(at).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let kind = entry.file_type().unwrap();
            assert!(!kind.is_symlink());
            if kind.is_dir() {
                walk(root, &path, entries);
            } else {
                assert!(kind.is_file());
                let bytes = fs::read(&path).unwrap();
                entries.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                    format!("{}:{}", bytes.len(), hex(&Sha256::digest(bytes))),
                );
            }
        }
    }
    let mut entries = BTreeMap::new();
    walk(base, &base.join("store"), &mut entries);
    if base.join("ledger").exists() {
        let bytes = fs::read(base.join("ledger")).unwrap();
        entries.insert(
            "ledger".into(),
            format!("{}:{}", bytes.len(), hex(&Sha256::digest(bytes))),
        );
    }
    entries
}

// Time limits only bound broken children; stage selection never depends on time.
fn child(entry: &str, cwd: &Path, label: &str) -> std::process::ExitStatus {
    let stdout = fs::File::create(cwd.join(format!("{label}-stdout.bin"))).unwrap();
    let stderr = fs::File::create(cwd.join(format!("{label}-stderr.bin"))).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", entry, "--nocapture"])
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!(
                "bounded child did not finish: {entry}, evidence {}",
                cwd.display()
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn cold(base: &Path, label: &str) -> serde_json::Value {
    let before = inventory(base);
    fs::write(
        base.join("expected-byte-inventory.json"),
        serde_json::to_vec_pretty(&before).unwrap(),
    )
    .unwrap();
    assert!(child(INSPECTOR, base, label).success());
    assert_eq!(
        inventory(base),
        before,
        "cold inspection must not repair or resume"
    );
    let bytes = fs::read(base.join(format!("{label}-stdout.bin"))).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    serde_json::from_str(
        text.lines()
            .find_map(|l| l.strip_prefix("COLD_BOUNDARY="))
            .unwrap(),
    )
    .unwrap()
}

fn configure(f: &Fixture, point: &str, number: u8, predecessor: Option<String>) -> PathBuf {
    let dir = f.base.join(format!("case-{number:02}-{point}"));
    fs::create_dir(&dir).unwrap();
    fs::create_dir(dir.join("out")).unwrap();
    fs::copy(
        f.base.join("out/fixture-input.json"),
        dir.join("out/fixture-input.json"),
    )
    .unwrap();
    let mut intent = f.intent(REVIEWER);
    intent.approved_output_root = dir.join("out").to_str().unwrap().into();
    intent.predecessor = predecessor;
    let case = Case {
        base: f.base.clone(),
        point: point.into(),
        number,
        intent,
        policy: f.plan.policy_record_id.as_bytes().to_vec(),
    };
    fs::write(
        dir.join("case.json"),
        serde_json::to_vec_pretty(&case).unwrap(),
    )
    .unwrap();
    dir
}

#[test]
fn crash_boundaries_preserve_all_attempts_and_cold_candidates() {
    let mut f = Fixture::new();
    f.input(1);
    // Each stop follows a real process exit-zero/EOF and a durable Binder intent.
    // These are API-entry/return seams, not Core-internal mid-copy/flush seams.
    let cases = [
        ("preparation_enter", 0, 0, 0, 0, "OutputPreparation", false),
        ("preparation_visible", 1, 0, 0, 0, "OutputPreparation", true),
        ("capture_enter", 1, 0, 0, 0, "OutputCapture", false),
        ("capture_visible", 1, 1, 0, 0, "OutputCapture", true),
        ("capture_recorded", 1, 1, 0, 0, "OutputCapture", false),
        ("result_enter", 1, 1, 0, 0, "ReviewResult", false),
        ("result_visible", 1, 1, 1, 0, "ReviewResult", true),
        ("admission_enter", 1, 1, 1, 0, "Admission", false),
        ("admission_visible", 1, 1, 1, 1, "Admission", true),
    ];
    let mut predecessor = None;
    let dirs: Vec<_> = cases
        .iter()
        .enumerate()
        .map(|(i, (point, ..))| {
            let dir = configure(&f, point, i as u8 + 1, predecessor.clone());
            predecessor = Some(point.to_string());
            dir
        })
        .collect();
    let success_dir = configure(&f, "success", 20, predecessor);
    assert_eq!(
        f.store.retained_journal().current_head_reference(),
        *f.request.request_event_reference()
    );
    let base = f.base.clone();
    f.preserve = true;
    // Independent producer must not contend with a live parent Store reader:
    // reused Record flushing on Windows needs the actual write/no-share handoff.
    drop(f);
    let (mut starts, mut freezes, mut results, mut admissions) = (1, 1, 0, 0);
    let mut previous = None;
    let mut old_store = inventory(&base);
    let mut old_ledger = Vec::new();
    for (i, (point, s, z, r, a, operation, candidate)) in cases.iter().enumerate() {
        let dir = dirs[i].clone();
        assert_eq!(
            child(OWNER, &dir, "owner").code(),
            Some(86),
            "{point}: {}",
            dir.display()
        );
        assert_eq!(
            fs::read_to_string(dir.join("checkpoint.txt")).unwrap(),
            *point
        );
        starts += s;
        freezes += z;
        results += r;
        admissions += a;
        let after = inventory(&base);
        for (path, digest) in &old_store {
            if path != "ledger" {
                assert_eq!(after.get(path), Some(digest), "earlier Store bytes {path}");
            }
        }
        let ledger = fs::read(base.join("ledger")).unwrap();
        assert!(
            ledger.starts_with(&old_ledger),
            "earlier ledger prefix retained"
        );
        old_ledger = ledger;
        old_store = after;
        let inspection = cold(&base, &format!("cold-{i:02}"));
        let counts = &inspection["counts"];
        assert_eq!(counts["attempts"], i + 1);
        assert_eq!(counts["unresolved_attempts"], i + 1);
        assert_eq!(counts["store_freeze_starts"], starts);
        assert_eq!(counts["store_freezes"], freezes);
        assert_eq!(counts["store_results"], results);
        assert_eq!(counts["store_admissions_accepted"], admissions);
        assert_eq!(counts["observed_failure_attempts"], 0);
        let attempts = inspection["attempts"].as_array().unwrap();
        let current = &attempts[i];
        assert_eq!(current["attempt_id"], *point);
        assert_eq!(
            current["predecessor"],
            serde_json::to_value(&previous).unwrap()
        );
        let publication = current["publications"].as_array().unwrap().last().unwrap();
        assert_eq!(publication["operation"], *operation);
        if *point == "capture_recorded" {
            assert_eq!(publication["ledger_status"], "KnownReferences");
        } else {
            assert_eq!(publication["ledger_status"], "Unresolved");
            if *candidate {
                assert!(publication["candidate_count"].as_u64().unwrap() > 0);
            }
        }
        assert_eq!(publication["candidate_attribution_established"], false);
        previous = Some(point.to_string());
    }
    // Explicit fresh success must not hide or resolve any previous crash.
    assert!(child(OWNER, &success_dir, "owner").success());
    let report = cold(&base, "cold-final");
    assert_eq!(report["counts"]["attempts"], cases.len() + 1);
    assert_eq!(report["counts"]["unresolved_attempts"], cases.len());
    assert_eq!(
        report["counts"]["attempt_statuses"]["HistoricalReferencesValidated"],
        1
    );
    assert_eq!(
        report["counts"]["store_admissions_accepted"],
        admissions + 1
    );
    assert!(fs::read(base.join("ledger"))
        .unwrap()
        .starts_with(&old_ledger));
    for (path, digest) in old_store {
        if path != "ledger" {
            assert_eq!(inventory(&base).get(&path), Some(&digest));
        }
    }
    eprintln!("CRASH_BOUNDARY_ALL_ATTEMPTS={}", base.display());
}

#[test]
#[ignore = "invoked abruptly terminated public-route owner, not skipped coverage"]
fn boundary_owner() {
    let case: Case = serde_json::from_slice(&fs::read("case.json").unwrap()).unwrap();
    let mut store =
        AuthoritativeRegistryStore::open_selected_profile(case.base.join("store")).unwrap();
    let request = store
        .retained_journal()
        .references()
        .find(|r| r.event_type_id().value() == 300)
        .unwrap();
    let mut writer = LedgerWriter::open(case.base.join("ledger")).unwrap();
    let point = case.point.clone();
    let base = case.base.clone();
    let number = case.number;
    #[cfg(windows)]
    let journal_guard = std::rc::Rc::new(RefCell::new(None));
    #[cfg(windows)]
    let journal_guard_for_hook = std::rc::Rc::clone(&journal_guard);
    HOOK.with(|h| {
        *h.borrow_mut() = Some(Box::new(move |seen| {
            let corruption_at = match point.as_str() {
                "commit_corruption" => Some(("capture_enter", number, "submission.json")),
                "readback_corruption" => Some(("capture_recorded", number, "submission.json")),
                "section82_corruption" => Some(("result_visible", 62, "target.txt")),
                "admission_corruption" => Some(("admission_enter", 62, "target.txt")),
                _ => None,
            };
            if let Some((at, id, name)) = corruption_at {
                if seen == at {
                    let leaf = base
                        .join("store/roots")
                        .join(hex(&[id; 32]))
                        .join("payload")
                        .join(name);
                    // Retain the exact genuine pre-corruption bytes as adverse evidence.
                    fs::copy(&leaf, "original-payload.bin").unwrap();
                    fs::write(&leaf, b"adversarial negative corruption").unwrap();
                }
            }
            #[cfg(windows)]
            if matches!(
                point.as_str(),
                "result_record_only" | "result_journal_unavailable"
            ) && seen == "result_enter"
            {
                use std::os::windows::fs::OpenOptionsExt;
                *journal_guard_for_hook.borrow_mut() = Some(
                    fs::OpenOptions::new()
                        .read(true)
                        .share_mode(1)
                        .custom_flags(0x02000000)
                        .open(base.join("store/journal"))
                        .unwrap(),
                );
                assert!(journal_guard_for_hook.borrow().is_some());
            }
            if seen == point || (point == "result_record_only" && seen == "publication_error") {
                let mut marker = fs::File::create("checkpoint.txt").unwrap();
                marker.write_all(seen.as_bytes()).unwrap();
                marker.sync_all().unwrap();
                // No Rust unwinding, no Store/ledger/completion destructor or cleanup.
                std::process::exit(86);
            }
        }))
    });
    let outcome = run_to_admission(
        &mut store,
        &mut writer,
        request,
        &case.point,
        case.intent,
        OutputCapturePlan {
            freeze_attempt_id: FreezeAttemptId::try_from([case.number; 32].as_slice()).unwrap(),
            policy_record_id: RecordId::try_from(case.policy.as_slice()).unwrap(),
        },
        &AtomicBool::new(false),
    )
    .unwrap();
    // Successful control runs have no guard. Checkpoint runs terminate without
    // unwinding while it is retained; an ordinary return must release it here.
    #[cfg(windows)]
    drop(journal_guard);
    assert_eq!(
        case.point, "success",
        "requested checkpoint was never reached"
    );
    assert!(matches!(
        outcome.admission,
        AuthoritativeReviewAdmissionRuntimeOutcome::Published(_)
    ));
}

#[test]
#[ignore = "actual managed native reviewer subprocess entry, not AI"]
fn boundary_reviewer() {
    fs::write("submission.json", fs::read("fixture-input.json").unwrap()).unwrap();
    fs::write("review.txt", b"untrusted CLEAN fixture, no semantic review").unwrap();
    println!("directly observed pipe; contents are an untrusted reviewer claim");
    eprintln!("actual reviewer stderr");
}

#[test]
#[ignore = "native cold all-attempt inspector, invoked by parent tests"]
fn boundary_inspector() {
    let base = std::env::current_dir().unwrap();
    let expected: BTreeMap<String, String> =
        serde_json::from_slice(&fs::read("expected-byte-inventory.json").unwrap()).unwrap();
    assert_eq!(inventory(&base), expected);
    let report = crate::inspection::inspect("store", "ledger").unwrap();
    assert!(report.read_only);
    assert!(!report.ledger_join_authoritative);
    assert_eq!(report.counts.independent_reviews_established, 0);
    assert!(report
        .attempts
        .iter()
        .all(|a| !a.live_witness && !a.historical_receipt_recovered));
    assert_eq!(report.counts.attempts, report.attempts.len());
    println!("COLD_BOUNDARY={}", serde_json::to_string(&report).unwrap());
    assert_eq!(inventory(&base), expected);
}
