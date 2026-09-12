use evidence_registry::*;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> (
        Self,
        AuthoritativeRegistryStore,
        SelectedEmbeddedFreezePreparationInput,
    ) {
        let base = std::env::current_dir()
            .unwrap()
            .join("target")
            .join(format!(
                "capture-limits-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&base).unwrap();
        let source = base.join("source");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("a.txt"), b"abcd").unwrap();
        fs::write(source.join("b.txt"), b"efgh").unwrap();
        let mut store = AuthoritativeRegistryStore::initialize_selected_profile(
            base.join("store"),
            RegistryId::try_from([81; 32].as_slice()).unwrap(),
            "capture-limits",
        )
        .unwrap();
        let scope = ScopeRecord::new(ScopeRecordInput {
            scope_profile_id: 2,
            scope_profile_version: 1,
            scope_payload: vec![],
            scope_label: None,
        })
        .unwrap();
        store.stage_scope_record(&scope).unwrap();
        let policy = store
            .register_selected_minimal_policy(scope.record_id())
            .unwrap();
        let input = SelectedEmbeddedFreezePreparationInput {
            source_root: source,
            freeze_attempt_id: FreezeAttemptId::try_from([82; 32].as_slice()).unwrap(),
            policy_record_id: RecordId::try_from(policy.event_record_id().as_bytes().as_slice())
                .unwrap(),
        };
        (Self(base), store, input)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn declared_limits_reject_before_start_and_accept_exact_boundary() {
    let (_fixture, mut store, input) = Fixture::new();
    let head = store.retained_journal().current_head_reference();
    for limits in [
        SelectedEmbeddedCaptureLimits {
            max_files: 1,
            max_content_bytes: 8,
        },
        SelectedEmbeddedCaptureLimits {
            max_files: 2,
            max_content_bytes: 7,
        },
        SelectedEmbeddedCaptureLimits {
            max_files: 0,
            max_content_bytes: 8,
        },
        SelectedEmbeddedCaptureLimits {
            max_files: 2,
            max_content_bytes: 0,
        },
    ] {
        assert_eq!(
            store
                .prepare_selected_embedded_freeze_with_limits(input.clone(), limits)
                .unwrap_err(),
            SelectedEmbeddedFreezePreparationError::SourceResourceLimit
        );
        assert_eq!(store.retained_journal().current_head_reference(), head);
        assert!(!store
            .root()
            .join("journal")
            .join(format!("{:020}.cbor", head.entry_index().value() + 1))
            .exists());
    }
    let prepared = store
        .prepare_selected_embedded_freeze_with_limits(
            input,
            SelectedEmbeddedCaptureLimits {
                max_files: 2,
                max_content_bytes: 8,
            },
        )
        .unwrap();
    let committed = store
        .commit_prepared_selected_embedded_freeze(prepared)
        .unwrap();
    let payload = store
        .read_selected_embedded_payload(committed.committed_event_reference().clone())
        .unwrap();
    assert_eq!(
        payload.artifact_bytes(),
        &[b"abcd".to_vec(), b"efgh".to_vec()]
    );
}
