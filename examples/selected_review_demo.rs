//! Core-only, controlled fake review; no Agent, Binder, network, or execution attestation.
//! Build: cargo build --locked --offline --example selected_review_demo
//! Run:   cargo run --locked --offline --example selected_review_demo -- run accepted <ABSENT_APPROVED_ROOT>
//! Read:  cargo run --locked --offline --example selected_review_demo -- inspect <EXISTING_DEMO_ROOT>
//! The root's parent must exist. No overwrite, cleanup, retry, or resume is performed.
//! `run rejected`, `run invalid`, and `run unsupported` each require their own
//! absent approved root; all four scenarios return 0 only after their control and
//! separate-process readback succeed. Exit 0 does not mean Admission accepted.
//! Exit 1 is unexpected failure/uncertainty; 2 is usage error. Preserve stdout,
//! stderr, exit, and the entire root, including incomplete work. The generic
//! unsupported control is NOT a claim that the selected lane is unsupported.
//! Fixed synthetic identities are for isolated examples, not production registries.

use evidence_registry::*;
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;

const FAKE: &str = "CONTROLLED_FAKE_NOT_AI";
const SOURCE: &[u8] = b"Synthetic review target. No genuine AI review was performed.\n";
const CHECK: &[u8] = b"The demo retains bytes; acceptance does not prove semantic truth.\n";
type DemoResult<T> = Result<T, String>;

fn checked<T, E: std::fmt::Debug>(result: Result<T, E>) -> DemoResult<T> {
    result.map_err(|error| format!("{error:?}"))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn record_id(reference: &JournalReference) -> RecordId {
    RecordId::try_from(reference.event_record_id().as_bytes().as_slice()).expect("32-byte identity")
}
fn reference(label: &str, value: &JournalReference) -> String {
    format!(
        "{label} registry={} index={} hash={} event={} record={}",
        hex(value.registry_id().as_bytes()),
        value.entry_index().value(),
        hex(value.entry_hash().as_bytes()),
        value.event_type_id().value(),
        hex(value.event_record_id().as_bytes())
    )
}
fn create_source(path: &Path, bytes: &[u8]) -> DemoResult<()> {
    let mut file = checked(OpenOptions::new().write(true).create_new(true).open(path))?;
    checked(file.write_all(bytes))?;
    checked(file.sync_all())
}
fn scope(profile: u64) -> DemoResult<ScopeRecord> {
    checked(ScopeRecord::new(ScopeRecordInput {
        scope_profile_id: profile,
        scope_profile_version: 1,
        scope_payload: vec![],
        scope_label: None,
    }))
}

fn run(root: &Path, scenario: &str) -> DemoResult<()> {
    // This single create is the no-replace boundary for the entire demo tree.
    // A failure later deliberately leaves all partial work for inspection.
    checked(fs::create_dir(root)).map_err(|error| format!("fresh_root_required: {error}"))?;
    let source = root.join("source");
    checked(fs::create_dir(&source))?;
    checked(fs::create_dir(source.join("nested")))?;
    create_source(&source.join("target.txt"), SOURCE)?;
    create_source(&source.join("nested/check.txt"), CHECK)?;
    println!("reviewer={FAKE} semantic_truth=NOT_ESTABLISHED");
    // Fixed synthetic identities are demo inputs, not authenticated reviewer identities.
    let registry = checked(RegistryId::try_from([0x91; 32].as_slice()))?;
    let mut store = checked(AuthoritativeRegistryStore::initialize_selected_profile(
        root.join("store"),
        registry,
        "selected-review-demo-controlled-fake",
    ))?;
    let freeze_scope = scope(2)?;
    let review_scope = scope(1)?;
    checked(store.stage_scope_record(&freeze_scope))?;
    checked(store.stage_scope_record(&review_scope))?;
    let method = checked(MethodRecord::new(MethodRecordInput {
        method_profile_id: 1,
        method_profile_version: 1,
        method_payload: b"Controlled fake semantic report; no real review execution".to_vec(),
        method_label: Some(FAKE.into()),
    }))?;
    let check = checked(CheckRecord::new(CheckRecordInput {
        check_profile_id: 1,
        check_profile_version: 1,
        check_payload: b"Synthetic claim only, not independently verified".to_vec(),
        check_label: Some(FAKE.into()),
    }))?;
    checked(store.stage_method_record(&method))?;
    checked(store.stage_check_record(&check))?;
    let checks = checked(CheckSetRecord::new(CheckSetRecordInput {
        check_refs: vec![check.record_id()],
    }))?;
    checked(store.stage_check_set_record(&checks))?;
    let freeze_policy = checked(store.register_selected_minimal_policy(freeze_scope.record_id()))?;
    let requirement = checked(ReviewAdmissionReviewRequirement::new(
        1,
        review_scope.record_id(),
        method.record_id(),
        checks.record_id(),
        1,
    ))?;
    let review_policy = checked(store.register_selected_review_policy(
        review_scope.record_id(),
        vec![requirement],
        Some(vec![1]),
        Some(vec![1]),
        Some(vec![1, 2]),
    ))?;
    println!("{}", reference("freeze_policy", &freeze_policy));
    println!("{}", reference("review_policy", &review_policy));
    let prepared = checked(store.prepare_selected_embedded_freeze_with_limits(
        SelectedEmbeddedFreezePreparationInput {
            source_root: source,
            freeze_attempt_id: checked(FreezeAttemptId::try_from([0x92; 32].as_slice()))?,
            policy_record_id: record_id(&freeze_policy),
        },
        SelectedEmbeddedCaptureLimits {
            max_files: 2,
            max_content_bytes: 4096,
        },
    ))?;
    println!(
        "retained_payload={}",
        prepared.payload_directory().display()
    );
    let freeze = checked(store.commit_prepared_selected_embedded_freeze(prepared))?;
    println!(
        "{}",
        reference("target_freeze", freeze.committed_event_reference())
    );
    println!(
        "manifest_id={} subject_id={}",
        hex(freeze.manifest_record_id().as_bytes()),
        hex(&freeze.subject_id())
    );
    let request = checked(
        store.record_selected_review_request(SelectedReviewRequestInput {
            freeze_authority: freeze,
            review_policy_record_id: record_id(&review_policy),
            review_role_id: 1,
        }),
    )?;
    println!(
        "{}",
        reference("request", request.request_event_reference())
    );
    println!(
        "package_anchor_id={}",
        hex(request.request().review_package_anchor_id().as_bytes())
    );
    // Only semantic claims are supplied. Core derives redundant bindings and all
    // positive Record/Journal bytes; this example never writes either namespace.
    let before_result = store.retained_journal().current_head_reference();
    let result = store.record_selected_review_result(SelectedReviewResultInput {
        request_event_reference: request.request_event_reference().clone(),
        method_status: match scenario {
            "rejected" => 2,
            "invalid" => 999,
            _ => 1,
        },
        finding_state: 1,
        reason_codes: vec!["CONTROLLED_FAKE_CLAIM".into()],
        findings: vec![],
        reviewer_metadata: Some(FAKE.into()),
    });
    if scenario == "invalid" {
        match result {
            Err(SelectedReviewResultError::ResultConstruction) => {
                if store.retained_journal().current_head_reference() != before_result {
                    return Err("invalid claim changed the Journal head".into());
                }
                println!("preterminal=INVALID error=ResultConstruction result_publication=NONE journal_head_unchanged=true");
                return cold_readback(root, store);
            }
            Err(error) => return Err(format!("unexpected invalid-control error: {error:?}")),
            Ok(_) => {
                return Err(
                    "invalid control unexpectedly published a Result; retain and inspect".into(),
                )
            }
        }
    }
    let result = checked(result)?;
    println!("{}", reference("result", result.result_event_reference()));
    if scenario == "unsupported" {
        // The generic API deliberately has no selected exact-Scope authority.
        // Opaque intake acceptance is NOT Policy satisfaction or Admission.
        let before = store.retained_journal().current_head_reference();
        let intake = checked(store.accept_authoritative_review_admission(
            request.request_event_reference().clone(),
            &request.request().authoritative_cbor(),
            result.result_event_reference().clone(),
            &result.result().authoritative_cbor(),
        ))?;
        let outcome = checked(store.complete_authoritative_review_admission(intake))?;
        match outcome {
            AuthoritativeReviewAdmissionRuntimeOutcome::PreTerminal(
                AuthoritativeReviewAdmissionSection82Error::PolicyScopeApplicabilityUnavailable,
            ) => {
                if store.retained_journal().current_head_reference() != before {
                    return Err("unsupported route changed Journal head".into());
                }
                println!("preterminal=UNSUPPORTED error=PolicyScopeApplicabilityUnavailable route=GENERIC_NOT_SELECTED journal_head_unchanged=true");
                return cold_readback(root, store);
            }
            _ => return Err(
                "unsupported control unexpectedly changed outcome; retain and inspect, never retry"
                    .into(),
            ),
        }
    }
    let outcome =
        checked(store.complete_selected_review_admission(result.result_event_reference().clone()))?;
    match outcome {
        AuthoritativeReviewAdmissionRuntimeOutcome::Published(publication) => {
            println!(
                "policy={:?} disposition={:?}",
                publication.policy_completion().result(),
                publication.disposition()
            );
            println!(
                "{}",
                reference("admission", publication.journal_reference())
            );
            println!(
                "publication_receipt=LIVE durability={:?}",
                publication.durability()
            );
        }
        AuthoritativeReviewAdmissionRuntimeOutcome::PreTerminal(error) => {
            return Err(format!(
                "PRETERMINAL: {error:?}; no terminal success claimed"
            ))
        }
        AuthoritativeReviewAdmissionRuntimeOutcome::PublishedReceiptUncertain(uncertain) => {
            eprintln!(
                "{}",
                reference("uncertain_admission", uncertain.journal_reference())
            );
            return Err("PUBLICATION_UNCERTAIN: retain root, inspect exact history; do not retry or claim a live receipt".into());
        }
    }
    cold_readback(root, store)
}

fn cold_readback(root: &Path, store: AuthoritativeRegistryStore) -> DemoResult<()> {
    let expected_head = reference("head", &store.retained_journal().current_head_reference());
    drop(store);
    // A new OS process must reconstruct Store authority and Manifest-matched
    // payload from disk. It cannot inherit in-memory witnesses or a live receipt.
    let output = checked(
        Command::new(checked(std::env::current_exe())?)
            .arg("inspect")
            .arg(root)
            .output(),
    )?;
    if output.stdout.len() > 32 * 1024 || output.stderr.len() > 4096 {
        return Err("cold summary exceeded demo bound".into());
    }
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!(
            "cold process failed: exit={:?} stderr={}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if !String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line == expected_head)
    {
        return Err("cold head differs from producer head".into());
    }
    println!("cold_readback=VERIFIED separate_process=true");
    Ok(())
}

fn inspect(root: &Path) -> DemoResult<()> {
    let store = checked(AuthoritativeRegistryStore::open_selected_profile(
        root.join("store"),
    ))?;
    let refs: Vec<_> = store.retained_journal().references().collect();
    // This is a small example inspector, not a general-purpose history UI.
    if refs.len() > 16 {
        return Err("demo history limit exceeded; use a general library consumer".into());
    }
    println!(
        "inspection_pid={} live_publication_receipt=NOT_RECOVERED",
        std::process::id()
    );
    println!(
        "{}",
        reference("head", &store.retained_journal().current_head_reference())
    );
    println!(
        "event_types={:?}",
        refs.iter()
            .map(|r| r.event_type_id().value())
            .collect::<Vec<_>>()
    );
    let mut disposition = None;
    for event in refs {
        println!("{}", reference("retained", &event));
        match event.event_type_id().value() {
            101 => {
                let payload = checked(store.read_selected_embedded_payload(event))?;
                if payload.artifact_bytes().len() > 2 {
                    return Err("demo payload count exceeded".into());
                }
                println!(
                    "payload_files={} manifest_id={}",
                    payload.artifact_bytes().len(),
                    hex(payload.manifest().record_id().as_bytes())
                );
                for (index, bytes) in payload.artifact_bytes().iter().enumerate() {
                    println!(
                        "payload_artifact={index} bytes={} sha256={}",
                        bytes.len(),
                        hex(&Sha256::digest(bytes))
                    );
                }
            }
            300 => {
                checked(store.validate_selected_review_request(event))?;
            }
            301 => {
                let result = checked(store.validate_selected_review_result(event))?;
                println!(
                    "retained_claim method_status={} finding_state={}",
                    result.result().method_status(),
                    result.result().finding_state()
                );
            }
            302 | 303 => {
                let bytes = store
                    .resolve(record_id(&event))
                    .ok_or("Admission Record missing")?;
                let admission = checked(ReviewAdmissionRecord::decode_authoritative(bytes))?;
                disposition = Some(admission.disposition_id());
                // Never dump unbounded or terminal-active untrusted reason text.
                println!(
                    "disposition_id={} reason_codes_count={}",
                    admission.disposition_id(),
                    admission.reason_codes().len()
                );
            }
            _ => {}
        }
    }
    if disposition.is_none() {
        println!("terminal_admission=NONE policy_completion=NOT_ESTABLISHED");
    }
    println!("semantic_truth=NOT_ESTABLISHED reviewer_authentication=NOT_ESTABLISHED");
    Ok(())
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let result = match args.as_slice() {
        [command, scenario, root]
            if command == "run"
                && matches!(
                    scenario.to_str(),
                    Some("accepted" | "rejected" | "invalid" | "unsupported")
                ) =>
        {
            run(
                Path::new(root),
                scenario.to_str().expect("validated scenario"),
            )
        }
        [command, root] if command == "inspect" => inspect(Path::new(root)),
        _ => {
            eprintln!("usage: selected_review_demo run <accepted|rejected|invalid|unsupported> <ABSENT_APPROVED_ROOT> | inspect <EXISTING_DEMO_ROOT>");
            std::process::exit(2);
        }
    };
    if let Err(error) = result {
        eprintln!(
            "demo_failed: {error}; existing/partial work is retained, never cleaned or retried"
        );
        std::process::exit(1);
    }
}
