use evidence_registry::*;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture {
    root: PathBuf,
    source: PathBuf,
}
impl Fixture {
    fn new() -> (Self, AuthoritativeRegistryStore) {
        let root = std::env::current_dir()
            .unwrap()
            .join("target")
            .join(format!(
                "store-provisioning-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        let source = root.with_extension("source");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("evidence.txt"), b"exact captured evidence").unwrap();
        let store = AuthoritativeRegistryStore::initialize_selected_profile(
            &root,
            RegistryId::try_from([0x76; 32].as_slice()).unwrap(),
            "provisioning-test",
        )
        .unwrap();
        (Self { root, source }, store)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
        let _ = fs::remove_dir_all(&self.source);
    }
}
fn scope(profile: u64) -> ScopeRecord {
    ScopeRecord::new(ScopeRecordInput {
        scope_profile_id: profile,
        scope_profile_version: 1,
        scope_payload: vec![],
        scope_label: None,
    })
    .unwrap()
}
// Only adversarial fixtures below write authority bytes directly. The positive
// provisioning path above/below remains public-only.
fn write_record(root: &std::path::Path, id: RecordId, bytes: &[u8]) {
    let name: String = id
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    fs::write(root.join("records").join(format!("{name}.cbor")), bytes).unwrap();
}
fn record_count(root: &std::path::Path) -> usize {
    fs::read_dir(root.join("records")).unwrap().count()
}
fn method() -> MethodRecord {
    MethodRecord::new(MethodRecordInput {
        method_profile_id: 1,
        method_profile_version: 1,
        method_payload: vec![],
        method_label: None,
    })
    .unwrap()
}
fn check() -> CheckRecord {
    CheckRecord::new(CheckRecordInput {
        check_profile_id: 1,
        check_profile_version: 1,
        check_payload: vec![],
        check_label: None,
    })
    .unwrap()
}
fn policy_entry(
    store: &AuthoritativeRegistryStore,
    policy: &ReviewAdmissionPolicyRecord,
) -> Vec<u8> {
    let head = store.retained_journal().current_head_reference();
    let context = store.retained_journal().resolve_reference(&head).unwrap();
    let mut ids = vec![policy.gate_scope_ref()];
    for req in policy.review_requirements() {
        ids.extend([
            req.review_scope_ref(),
            req.review_method_ref(),
            req.required_checks_ref(),
        ]);
    }
    ids.sort();
    ids.dedup();
    let dependencies = IdentityDependencyCollection::from_unordered_semantic_elements(
        ids.into_iter().map(IdentityDependency::record_id).collect(),
    )
    .unwrap();
    let mut bytes = vec![0x82, 0x78, 0x20];
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.extend_from_slice(&[0xac, 0, 1, 1]);
    let bstr = |bytes: &mut Vec<u8>, value: &[u8]| {
        bytes.extend_from_slice(&[0x58, 0x20]);
        bytes.extend_from_slice(value);
    };
    bstr(&mut bytes, head.registry_id().as_bytes());
    bytes.extend_from_slice(&[2, u8::try_from(head.entry_index().value() + 1).unwrap(), 3]);
    bstr(&mut bytes, head.entry_hash().as_bytes());
    bytes.extend_from_slice(&[4, 0x19, 1, 0x90, 5]);
    bstr(&mut bytes, policy.record_id().as_bytes());
    bytes.push(6);
    bytes.extend_from_slice(&dependencies.authoritative_cbor());
    bytes.extend_from_slice(&[7, 0x80, 8, 7, 9]);
    bstr(&mut bytes, policy.record_id().as_bytes());
    bytes.push(10);
    bstr(&mut bytes, context.storage_capability_class_id().as_bytes());
    bytes.push(11);
    bstr(&mut bytes, context.environment_observation_id().as_bytes());
    bytes
}

#[test]
fn retained_policy_replay_rejects_wrong_parameter_types_and_every_check_member() {
    for mutation in 0..6 {
        let (fixture, mut store) = Fixture::new();
        let gate = scope(1);
        let method = method();
        let check = check();
        store.stage_scope_record(&gate).unwrap();
        store.stage_method_record(&method).unwrap();
        store.stage_check_record(&check).unwrap();
        let member = if mutation == 4 {
            gate.record_id()
        } else if mutation == 5 {
            RecordId::try_from([0xfa; 32].as_slice()).unwrap()
        } else {
            check.record_id()
        };
        let mut refs = vec![check.record_id(), member];
        refs.sort();
        refs.dedup();
        let checks = CheckSetRecord::new(CheckSetRecordInput { check_refs: refs }).unwrap();
        write_record(
            &fixture.root,
            checks.record_id(),
            &checks.authoritative_cbor(),
        );
        let policy = ReviewAdmissionPolicyRecord::new(ReviewAdmissionPolicyRecordInput {
            gate_scope_ref: if mutation == 0 {
                method.record_id()
            } else {
                gate.record_id()
            },
            review_requirements: vec![ReviewAdmissionReviewRequirement::new(
                1,
                if mutation == 1 {
                    method.record_id()
                } else {
                    gate.record_id()
                },
                if mutation == 2 {
                    gate.record_id()
                } else {
                    method.record_id()
                },
                if mutation == 3 {
                    gate.record_id()
                } else {
                    checks.record_id()
                },
                1,
            )
            .unwrap()],
            required_method_statuses: None,
            allowed_finding_states: None,
            acceptable_anchor_relation_ids: None,
            supported_context_ids: vec![2, 5],
            operation_start_journal_ref: store.retained_journal().current_head_reference(),
        })
        .unwrap();
        let entry = policy_entry(&store, &policy);
        write_record(
            &fixture.root,
            policy.record_id(),
            &policy.authoritative_cbor(),
        );
        drop(store);
        fs::write(
            fixture.root.join("journal/00000000000000000001.cbor"),
            entry,
        )
        .unwrap();
        assert!(
            matches!(
                AuthoritativeRegistryStore::open_selected_profile(&fixture.root),
                Err(AuthoritativeRegistryStoreOpenError::EventRecordDecode)
            ),
            "mutation {mutation} passed retained typed validation"
        );
    }
}

fn record_id(reference: &JournalReference) -> RecordId {
    RecordId::try_from(reference.event_record_id().as_bytes().as_slice()).unwrap()
}

#[test]
fn policy_registration_does_not_self_evaluate_gate_scope_equality() {
    let (fixture, mut store) = Fixture::new();
    let gate = scope(1);
    let selector_scope = ScopeRecord::new(ScopeRecordInput {
        scope_profile_id: 1,
        scope_profile_version: 1,
        scope_payload: vec![],
        scope_label: Some("distinct review scope".into()),
    })
    .unwrap();
    let method = method();
    let check = check();
    store.stage_scope_record(&gate).unwrap();
    store.stage_scope_record(&selector_scope).unwrap();
    store.stage_method_record(&method).unwrap();
    store.stage_check_record(&check).unwrap();
    let checks = CheckSetRecord::new(CheckSetRecordInput {
        check_refs: vec![check.record_id()],
    })
    .unwrap();
    store.stage_check_set_record(&checks).unwrap();
    let policy_ref = store
        .register_selected_review_policy(
            gate.record_id(),
            vec![ReviewAdmissionReviewRequirement::new(
                1,
                selector_scope.record_id(),
                method.record_id(),
                checks.record_id(),
                1,
            )
            .unwrap()],
            None,
            None,
            None,
        )
        .unwrap();
    let policy = ReviewAdmissionPolicyRecord::decode_authoritative(
        store.resolve(record_id(&policy_ref)).unwrap(),
    )
    .unwrap();
    assert_ne!(
        policy.gate_scope_ref(),
        policy.review_requirements()[0].review_scope_ref()
    );
    assert_eq!(
        store
            .retained_journal()
            .references()
            .map(|r| r.event_type_id().value())
            .collect::<Vec<_>>(),
        [1, 400]
    );
    drop(store);
    assert_eq!(
        AuthoritativeRegistryStore::open_selected_profile(&fixture.root)
            .unwrap()
            .retained_journal()
            .current_head_reference(),
        policy_ref
    );
}

#[test]
fn policy_registration_reserves_record_and_journal_slot_before_staging() {
    let (fixture, mut store) = Fixture::new();
    let gate = scope(2);
    store.stage_scope_record(&gate).unwrap();
    drop(store);
    // Deliberate boundary fixture: records + journal + registry/genesis duplicate.
    let count = record_count(&fixture.root)
        + fs::read_dir(fixture.root.join("journal")).unwrap().count()
        + 1;
    for n in count..AUTHORITATIVE_STORE_MAX_OBJECTS - 1 {
        let record = MethodRecord::new(MethodRecordInput {
            method_profile_id: n as u64,
            method_profile_version: 1,
            method_payload: vec![],
            method_label: None,
        })
        .unwrap();
        write_record(
            &fixture.root,
            record.record_id(),
            &record.authoritative_cbor(),
        );
    }
    let mut store = AuthoritativeRegistryStore::open_selected_profile(&fixture.root).unwrap();
    let head = store.retained_journal().current_head_reference();
    let count = record_count(&fixture.root);
    assert!(store
        .register_selected_minimal_policy(gate.record_id())
        .is_err());
    assert_eq!(
        record_count(&fixture.root),
        count,
        "must reserve BOTH new objects before Policy Record staging"
    );
    assert!(!fixture
        .root
        .join("journal/00000000000000000001.cbor")
        .exists());
    drop(store);
    assert_eq!(
        AuthoritativeRegistryStore::open_selected_profile(&fixture.root)
            .unwrap()
            .retained_journal()
            .current_head_reference(),
        head
    );
}

#[test]
fn provisioning_refuses_invalid_parameters_before_staging_or_next_slot() {
    let (fixture, mut store) = Fixture::new();
    let gate = scope(1);
    let method = method();
    let check = check();
    store.stage_scope_record(&gate).unwrap();
    store.stage_method_record(&method).unwrap();
    store.stage_check_record(&check).unwrap();
    let good_checks = CheckSetRecord::new(CheckSetRecordInput {
        check_refs: vec![check.record_id()],
    })
    .unwrap();
    store.stage_check_set_record(&good_checks).unwrap();
    let head = store.retained_journal().current_head_reference();
    let count = record_count(&fixture.root);
    for member in [
        gate.record_id(),
        method.record_id(),
        RecordId::try_from([0xfe; 32].as_slice()).unwrap(),
    ] {
        let mut refs = vec![check.record_id(), member];
        refs.sort();
        let checks = CheckSetRecord::new(CheckSetRecordInput { check_refs: refs }).unwrap();
        let error: evidence_registry::StoreProvisioningError =
            store.stage_check_set_record(&checks).unwrap_err();
        assert!(matches!(
            error,
            evidence_registry::StoreProvisioningError::InvalidParameterReference
        ));
        assert_eq!(store.resolve(checks.record_id()), None);
        assert_eq!(record_count(&fixture.root), count);
    }
    for (scope_id, method_id, checks_id) in [
        (
            method.record_id(),
            method.record_id(),
            good_checks.record_id(),
        ),
        (gate.record_id(), gate.record_id(), good_checks.record_id()),
        (gate.record_id(), method.record_id(), gate.record_id()),
        (
            gate.record_id(),
            method.record_id(),
            RecordId::try_from([0xfd; 32].as_slice()).unwrap(),
        ),
    ] {
        let requirement =
            ReviewAdmissionReviewRequirement::new(1, scope_id, method_id, checks_id, 1).unwrap();
        assert!(store
            .register_selected_review_policy(gate.record_id(), vec![requirement], None, None, None)
            .is_err());
        assert_eq!(record_count(&fixture.root), count);
    }
    assert!(store
        .register_selected_minimal_policy(gate.record_id())
        .is_err());
    let requirement = ReviewAdmissionReviewRequirement::new(
        1,
        gate.record_id(),
        method.record_id(),
        good_checks.record_id(),
        1,
    )
    .unwrap();
    assert!(store
        .register_selected_review_policy(
            gate.record_id(),
            vec![requirement],
            Some(vec![]),
            None,
            None
        )
        .is_err());
    let oversize = MethodRecord::new(MethodRecordInput {
        method_profile_id: 1,
        method_profile_version: 1,
        method_payload: vec![0; AUTHORITATIVE_STORE_MAX_OBJECT_BYTES],
        method_label: None,
    })
    .unwrap();
    assert!(store.stage_method_record(&oversize).is_err());
    assert_eq!(record_count(&fixture.root), count);
    assert_eq!(store.retained_journal().current_head_reference(), head);
    assert!(!fixture
        .root
        .join("journal/00000000000000000001.cbor")
        .exists());
    // Exact-byte reuse is idempotent staging, not a new portable parameter event.
    store.stage_scope_record(&gate).unwrap();
    assert_eq!(record_count(&fixture.root), count);
    drop(store);
    let reopened = AuthoritativeRegistryStore::open_selected_profile(&fixture.root).unwrap();
    assert_eq!(reopened.retained_journal().current_head_reference(), head);
}

#[test]
fn public_provisioning_reaches_request_with_exact_readback_and_cold_reopen() {
    let (fixture, mut store) = Fixture::new();
    let genesis = store.retained_journal().current_head_reference();
    let freeze_scope = scope(2);
    let review_scope = scope(1);
    assert_eq!(
        store.stage_scope_record(&freeze_scope).unwrap(),
        freeze_scope.record_id()
    );
    store.stage_scope_record(&review_scope).unwrap();
    let method = MethodRecord::new(MethodRecordInput {
        method_profile_id: 7,
        method_profile_version: 1,
        method_payload: b"definition only".to_vec(),
        method_label: None,
    })
    .unwrap();
    let check = CheckRecord::new(CheckRecordInput {
        check_profile_id: 9,
        check_profile_version: 1,
        check_payload: b"claim check definition".to_vec(),
        check_label: None,
    })
    .unwrap();
    store.stage_method_record(&method).unwrap();
    store.stage_check_record(&check).unwrap();
    let checks = CheckSetRecord::new(CheckSetRecordInput {
        check_refs: vec![check.record_id()],
    })
    .unwrap();
    store.stage_check_set_record(&checks).unwrap();
    for (id, bytes) in [
        (freeze_scope.record_id(), freeze_scope.authoritative_cbor()),
        (review_scope.record_id(), review_scope.authoritative_cbor()),
        (method.record_id(), method.authoritative_cbor()),
        (check.record_id(), check.authoritative_cbor()),
        (checks.record_id(), checks.authoritative_cbor()),
    ] {
        assert_eq!(store.resolve(id), Some(bytes.as_slice()));
    }
    assert_eq!(
        store.retained_journal().current_head_reference(),
        genesis,
        "portable parameters have no invented event"
    );
    let pf = store
        .register_selected_minimal_policy(freeze_scope.record_id())
        .unwrap();
    let decoded_pf =
        MinimalPolicyRecord::decode_authoritative(store.resolve(record_id(&pf)).unwrap()).unwrap();
    assert_eq!(decoded_pf.supported_context_ids(), [1]);
    assert_eq!(decoded_pf.operation_start_journal_ref(), &genesis);
    let requirement = ReviewAdmissionReviewRequirement::new(
        1,
        review_scope.record_id(),
        method.record_id(),
        checks.record_id(),
        1,
    )
    .unwrap();
    let pr = store
        .register_selected_review_policy(
            review_scope.record_id(),
            vec![requirement],
            Some(vec![1]),
            Some(vec![1]),
            Some(vec![1, 2]),
        )
        .unwrap();
    assert_eq!(pr.event_type_id().value(), 400);
    let decoded_pr =
        ReviewAdmissionPolicyRecord::decode_authoritative(store.resolve(record_id(&pr)).unwrap())
            .unwrap();
    assert_eq!(decoded_pr.supported_context_ids(), [2, 5]);
    assert_eq!(decoded_pr.operation_start_journal_ref(), &pf);
    drop(store);
    let mut store = AuthoritativeRegistryStore::open_selected_profile(&fixture.root).unwrap();
    assert_eq!(store.retained_journal().current_head_reference(), pr);
    let prepared = store
        .prepare_selected_embedded_freeze(SelectedEmbeddedFreezePreparationInput {
            source_root: fixture.source.clone(),
            freeze_attempt_id: FreezeAttemptId::try_from([0x77; 32].as_slice()).unwrap(),
            policy_record_id: record_id(&pf),
        })
        .unwrap();
    let freeze = store
        .commit_prepared_selected_embedded_freeze(prepared)
        .unwrap();
    let request_start = store.retained_journal().current_head_reference();
    let before_request_records = record_count(&fixture.root);
    assert_eq!(
        store.record_selected_review_request(SelectedReviewRequestInput {
            freeze_authority: freeze.clone(),
            review_policy_record_id: record_id(&pr),
            review_role_id: 2,
        }),
        Err(SelectedReviewRequestError::SelectorUnavailable)
    );
    assert_eq!(record_count(&fixture.root), before_request_records);
    assert_eq!(
        store.retained_journal().current_head_reference(),
        request_start
    );
    assert!(!fixture
        .root
        .join("journal/00000000000000000005.cbor")
        .exists());
    let request = store
        .record_selected_review_request(SelectedReviewRequestInput {
            freeze_authority: freeze.clone(),
            review_policy_record_id: record_id(&pr),
            review_role_id: 1,
        })
        .unwrap();
    assert_eq!(
        request.request().operation_start_journal_ref(),
        &request_start
    );
    assert_ne!(
        request.request().operation_start_journal_ref(),
        freeze.start_event_reference()
    );
    assert_eq!(request.request().policy_authority_ref(), &pr);
    assert_eq!(request.request().review_method_ref(), method.record_id());
    assert_eq!(request.request().required_checks_ref(), checks.record_id());
    drop(store);
    let reopened = AuthoritativeRegistryStore::open_selected_profile(&fixture.root).unwrap();
    assert_eq!(
        reopened
            .validate_selected_review_request(request.request_event_reference().clone())
            .unwrap(),
        request
    );
    assert_eq!(
        reopened
            .retained_journal()
            .references()
            .map(|r| r.event_type_id().value())
            .collect::<Vec<_>>(),
        [1, 400, 400, 100, 101, 300]
    );
}
