use evidence_registry::{AuthoritativeRegistryStore, RegistryId, StorageCapabilityClassRecord};
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

#[test]
fn selected_initializer_records_real_staging_probe_results_without_synthetic_success() {
    let root = std::env::current_dir()
        .unwrap()
        .join("target")
        .join(format!(
            "evidence-registry-selected-probe-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
    let registry_id = RegistryId::try_from([0x43; 32].as_slice()).unwrap();

    let store = AuthoritativeRegistryStore::initialize_selected_profile(
        &root,
        registry_id,
        "selected-probe-test",
    )
    .unwrap();
    let capability = fs::read_dir(root.join("records"))
        .unwrap()
        .find_map(|entry| {
            let bytes = fs::read(entry.ok()?.path()).ok()?;
            StorageCapabilityClassRecord::decode_authoritative(&bytes).ok()
        })
        .expect("selected initialization must retain a type-60 capability observation");

    assert_eq!(capability.input().exclusive_create_capability, 1);
    assert_eq!(capability.input().locking_capability, 3);
    assert_eq!(capability.input().file_flush_capability, 1);
    assert_eq!(capability.input().directory_flush_capability, 1);
    assert_eq!(capability.input().no_replace_publication_capability, 3);
    assert_eq!(capability.input().atomic_rename_capability, 3);
    assert_eq!(capability.input().placeholder_capability, 3);
    assert!(
        fs::read_dir(root.join("coordination/staging"))
            .unwrap()
            .next()
            .is_none(),
        "successful probes must not leave nonauthoritative staging residue"
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}
