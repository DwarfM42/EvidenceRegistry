// Shared public Store-owned setup; no positive history is written by hand.
struct Fixture {
    base: PathBuf,
    preserve: bool,
    store: AuthoritativeRegistryStore,
    request: RecordedSelectedReviewRequest,
    plan: OutputCapturePlan,
    // Selected Stores intentionally retain namespace and file witnesses. Keep
    // this test fixture process-serial so the test harness cannot exceed the
    // platform's ordinary descriptor envelope by constructing many stores at
    // once. This guard is test-only and is released after the Store.
    _serial: std::sync::MutexGuard<'static, ()>,
}
static FIXTURE_SERIAL: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
fn record_id(r: &JournalReference) -> RecordId {
    RecordId::try_from(r.event_record_id().as_bytes().as_slice()).unwrap()
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|b| format!("{b:02x}")).collect()
}
fn binding(r: &JournalReference) -> RequestBinding {
    RequestBinding {
        registry_id: hex(r.registry_id().as_bytes()),
        entry_index: r.entry_index().value(),
        entry_hash: hex(r.entry_hash().as_bytes()),
        event_type_id: r.event_type_id().value().into(),
        event_record_id: hex(r.event_record_id().as_bytes()),
    }
}
impl Fixture {
    fn new() -> Self {
        let serial = FIXTURE_SERIAL
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let base = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!(
                "binder-bridge-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&base).unwrap();
        let base = fs::canonicalize(base).unwrap();
        fs::create_dir(base.join("source")).unwrap();
        fs::create_dir(base.join("out")).unwrap();
        fs::write(
            base.join("source/target.txt"),
            b"real retained target; fake reviewer does not inspect semantics",
        )
        .unwrap();
        let mut store = AuthoritativeRegistryStore::initialize_selected_profile(
            base.join("store"),
            RegistryId::try_from([61; 32].as_slice()).unwrap(),
            "bridge-fake-test",
        )
        .unwrap();
        let scope = |p| {
            ScopeRecord::new(ScopeRecordInput {
                scope_profile_id: p,
                scope_profile_version: 1,
                scope_payload: vec![],
                scope_label: None,
            })
            .unwrap()
        };
        let freeze_scope = scope(2);
        let review_scope = scope(1);
        store.stage_scope_record(&freeze_scope).unwrap();
        store.stage_scope_record(&review_scope).unwrap();
        let method = MethodRecord::new(MethodRecordInput {
            method_profile_id: 1,
            method_profile_version: 1,
            method_payload: vec![],
            method_label: None,
        })
        .unwrap();
        let check = CheckRecord::new(CheckRecordInput {
            check_profile_id: 1,
            check_profile_version: 1,
            check_payload: vec![],
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
        let freeze_policy = store
            .register_selected_minimal_policy(freeze_scope.record_id())
            .unwrap();
        let review_policy = store
            .register_selected_review_policy(
                review_scope.record_id(),
                vec![ReviewAdmissionReviewRequirement::new(
                    1,
                    review_scope.record_id(),
                    method.record_id(),
                    checks.record_id(),
                    1,
                )
                .unwrap()],
                Some(vec![1]),
                Some(vec![1]),
                Some(vec![1, 2]),
            )
            .unwrap();
        let prepared = store
            .prepare_selected_embedded_freeze(SelectedEmbeddedFreezePreparationInput {
                source_root: base.join("source"),
                freeze_attempt_id: FreezeAttemptId::try_from([62; 32].as_slice()).unwrap(),
                policy_record_id: record_id(&freeze_policy),
            })
            .unwrap();
        let target = store
            .commit_prepared_selected_embedded_freeze(prepared)
            .unwrap();
        let request = store
            .record_selected_review_request(SelectedReviewRequestInput {
                freeze_authority: target,
                review_policy_record_id: record_id(&review_policy),
                review_role_id: 1,
            })
            .unwrap();
        let plan = OutputCapturePlan {
            freeze_attempt_id: FreezeAttemptId::try_from([63; 32].as_slice()).unwrap(),
            policy_record_id: record_id(&freeze_policy),
        };
        Self {
            base,
            preserve: false,
            store,
            request,
            plan,
            _serial: serial,
        }
    }
    fn input(&self, status: u64) {
        let submission = Submission {
            schema: 1,
            request: binding(self.request.request_event_reference()),
            method_status: status,
            finding_state: 1,
            reason_codes: vec![],
            findings: vec![],
            reviewer_metadata: Some("deterministic fake reviewer; not AI".into()),
            artifacts: vec!["review.txt".into()],
        };
        fs::write(
            self.base.join("out/fixture-input.json"),
            serde_json::to_vec(&submission).unwrap(),
        )
        .unwrap();
    }
    fn intent(&self, child: &str) -> DispatchIntent {
        DispatchIntent {
            approved_output_root: self.base.join("out").to_str().unwrap().into(),
            mandatory_files: vec![
                "binder-stderr.bin".into(),
                "binder-stdout.bin".into(),
                "review.txt".into(),
                "submission.json".into(),
            ],
            command: vec![
                std::env::current_exe().unwrap().to_str().unwrap().into(),
                "--ignored".into(),
                "--exact".into(),
                child.into(),
                "--nocapture".into(),
            ],
            binder_identity: "bridge-test-fake-native-reviewer".into(),
            limits: CommandLimits {
                runtime_ms: 5000,
                pipe_drain_ms: 200,
                ..CommandLimits::default()
            },
            predecessor: None,
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        // Keep raw Store/ledger/output evidence for failed qualification runs.
        if self.preserve
            || std::thread::panicking()
            || std::env::var_os("BINDER_BRIDGE_RETAIN_FIXTURES").is_some()
        {
            eprintln!("RETAINED_BRIDGE_FIXTURE={}", self.base.display());
        } else {
            let _ = fs::remove_dir_all(&self.base);
        }
    }
}
