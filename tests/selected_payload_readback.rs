use evidence_registry::*;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture {
    base: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let base = std::env::current_dir()
            .unwrap()
            .join("target")
            .join(format!(
                "payload-readback-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&base).unwrap();
        Self { base }
    }
    fn capture(&self) -> (AuthoritativeRegistryStore, JournalReference, PathBuf) {
        let source = self.base.join("source");
        fs::create_dir(&source).unwrap();
        fs::create_dir(source.join("nested")).unwrap();
        fs::write(source.join("nested/review.txt"), b"exact captured review\n").unwrap();
        fs::write(source.join("submission.json"), b"{\"schema\":1}").unwrap();
        let mut store = AuthoritativeRegistryStore::initialize_selected_profile(
            self.base.join("store"),
            RegistryId::try_from([71; 32].as_slice()).unwrap(),
            "readback",
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
        let prepared = store
            .prepare_selected_embedded_freeze(SelectedEmbeddedFreezePreparationInput {
                source_root: source,
                freeze_attempt_id: FreezeAttemptId::try_from([72; 32].as_slice()).unwrap(),
                policy_record_id: RecordId::try_from(
                    policy.event_record_id().as_bytes().as_slice(),
                )
                .unwrap(),
            })
            .unwrap();
        let payload = prepared.payload_directory().to_owned();
        let committed = store
            .commit_prepared_selected_embedded_freeze(prepared)
            .unwrap();
        (
            store,
            committed.committed_event_reference().clone(),
            payload,
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}
#[test]
fn public_readback_returns_manifest_matched_bytes_without_ambient_paths() {
    let f = Fixture::new();
    let (store, reference, _) = f.capture();
    let head = store.retained_journal().current_head_reference();
    let captured = store
        .read_selected_embedded_payload(reference.clone())
        .unwrap();
    assert_eq!(
        captured.freeze_authority().committed_event_reference(),
        &reference
    );
    assert_eq!(
        captured.manifest().record_id(),
        captured.freeze_authority().manifest_record_id()
    );
    assert_eq!(captured.artifact_bytes().len(), 2);
    assert_eq!(captured.artifact_bytes()[0], b"exact captured review\n");
    assert_eq!(captured.artifact_bytes()[1], b"{\"schema\":1}");
    assert_eq!(
        captured.manifest().input().artifacts[0].path_components,
        vec![b"nested".to_vec(), b"review.txt".to_vec()]
    );
    assert_eq!(store.retained_journal().current_head_reference(), head);
    drop(store);
    let cold = AuthoritativeRegistryStore::open_selected_profile(f.base.join("store")).unwrap();
    let reopened = cold.read_selected_embedded_payload(reference).unwrap();
    assert_eq!(reopened.artifact_bytes(), captured.artifact_bytes());
    assert_eq!(cold.retained_journal().current_head_reference(), head);
}

#[test]
fn readback_revalidates_live_payload_not_a_previously_returned_snapshot() {
    let f = Fixture::new();
    let (store, reference, payload) = f.capture();
    let captured = store
        .read_selected_embedded_payload(reference.clone())
        .unwrap();
    fs::write(payload.join("submission.json"), b"{\"schema\":2}").unwrap();
    assert!(store
        .read_selected_embedded_payload(reference.clone())
        .is_err());
    assert_eq!(captured.artifact_bytes()[1], b"{\"schema\":1}");
    drop(store);
    let cold = AuthoritativeRegistryStore::open_selected_profile(f.base.join("store")).unwrap();
    assert!(cold.read_selected_embedded_payload(reference).is_err());
}

#[test]
fn readback_rejects_wrong_event_and_additional_or_hardlinked_payload() {
    let f = Fixture::new();
    let (store, reference, payload) = f.capture();
    let genesis = store.retained_journal().references().next().unwrap();
    assert!(store.read_selected_embedded_payload(genesis).is_err());
    fs::write(payload.join("extra.txt"), b"not in Manifest").unwrap();
    assert!(store
        .read_selected_embedded_payload(reference.clone())
        .is_err());
    fs::remove_file(payload.join("extra.txt")).unwrap();
    fs::hard_link(payload.join("submission.json"), f.base.join("alias")).unwrap();
    assert!(store.read_selected_embedded_payload(reference).is_err());
}
