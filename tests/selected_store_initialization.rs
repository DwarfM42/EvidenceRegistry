use evidence_registry::{AuthoritativeRegistryStore, RegistryId};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

#[test]
fn selected_initializer_creates_an_absent_root_then_reopens_the_exact_profile() {
    let root = std::env::current_dir()
        .unwrap()
        .join("target")
        .join(format!(
            "evidence-registry-selected-init-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
    assert!(!root.exists());
    let registry_id = RegistryId::try_from([0x42; 32].as_slice()).unwrap();

    let store = AuthoritativeRegistryStore::initialize_selected_profile(
        &root,
        registry_id,
        "selected-initializer-test",
    )
    .unwrap();

    assert_eq!(
        store
            .retained_journal()
            .current_head_reference()
            .registry_id(),
        registry_id
    );
    assert!(AuthoritativeRegistryStore::open_selected_profile(&root).is_ok());
    assert!(root.join("registry/genesis.cbor").is_file());
    assert!(root.join("coordination/staging").is_dir());
    drop(store);
    fs::remove_dir_all(root).unwrap();
}
