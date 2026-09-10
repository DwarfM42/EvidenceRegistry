//! Native capture attacks. Hooks and fixture writes exist only in unit-test builds.
#[path = "source_capture_cases.rs"]
mod cases;
use super::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
type Hook = Box<dyn FnMut(&str, &[Vec<u8>])>;
thread_local! { static HOOK: RefCell<Option<Hook>> = RefCell::new(None); }
thread_local! { static OUTSIDE: RefCell<Option<fs::File>> = const { RefCell::new(None) }; }
pub(super) fn observe_read(file: &fs::File) {
    OUTSIDE.with(|outside| {
        if let Some(outside) = outside.borrow().as_ref() {
            assert!(
                !handles_identify_same_object(file, outside),
                "outside sentinel reached actual payload read boundary"
            );
        }
    });
}
pub(super) fn fire(phase: &str, path: &[Vec<u8>]) {
    HOOK.with(|h| {
        if let Some(h) = h.borrow_mut().as_mut() {
            h(phase, path);
        }
    });
}
struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        HOOK.with(|h| *h.borrow_mut() = None);
        OUTSIDE.with(|h| *h.borrow_mut() = None);
    }
}
fn install(h: impl FnMut(&str, &[Vec<u8>]) + 'static) -> Reset {
    HOOK.with(|slot| *slot.borrow_mut() = Some(Box::new(h)));
    Reset
}
static NEXT: AtomicU64 = AtomicU64::new(0);

#[test]
fn declared_limits_stop_at_actual_read_boundary_including_rescan() {
    for (max_files, max_content_bytes, expected_reads) in [(1, 16, 1), (2, 7, 0)] {
        let f = Fixture::new();
        fs::write(f.source.join("b"), b"original").unwrap();
        let reads = Rc::new(Cell::new(0));
        let observed = reads.clone();
        let _hook = install(move |phase, _| {
            if phase == "file" {
                observed.set(observed.get() + 1);
            }
        });
        let hold = source_capture::bind(&f.source).unwrap();
        assert_eq!(
            collect_selected_embedded_source_from_hold_with_limits(
                &hold,
                SelectedEmbeddedCaptureLimits {
                    max_files,
                    max_content_bytes
                }
            ),
            Err(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)
        );
        assert_eq!(reads.get(), expected_reads);
        drop(hold);
        drop(_hook);
        let base = f.base.clone();
        drop(f);
        fs::remove_dir_all(base).unwrap();
    }
    let mut f = Fixture::new();
    let added = f.source.join("b");
    let _hook = install(move |phase, path| {
        if phase == "snapshot" {
            fs::write(&added, b"new").unwrap();
        }
        if phase == "file" {
            assert_ne!(path, [b"b".to_vec()], "over-limit rescan reached read");
        }
    });
    let result = f.store.prepare_selected_embedded_freeze_with_limits(
        SelectedEmbeddedFreezePreparationInput {
            source_root: f.source.clone(),
            freeze_attempt_id: FreezeAttemptId::try_from([0x88; 32].as_slice()).unwrap(),
            policy_record_id: f.policy,
        },
        SelectedEmbeddedCaptureLimits {
            max_files: 1,
            max_content_bytes: 8,
        },
    );
    assert_eq!(
        result,
        Err(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)
    );
    // START stays visible after a later-pass failure; no event 101 is manufactured.
    assert_eq!(
        f.store
            .retained_journal()
            .current_head_reference()
            .event_type_id()
            .value(),
        100
    );
    drop(_hook);
    let base = f.base.clone();
    drop(f);
    fs::remove_dir_all(base).unwrap();
}

#[cfg(windows)]
#[test]
fn source_capture_commit_retains_attempt_during_payload_validation() {
    let mut f = Fixture::new();
    let prepared = f.prepare().unwrap();
    let attempt = prepared.payload_directory().parent().unwrap().to_path_buf();
    let moved = f.base.join("moved-attempt");
    let seen = Rc::new(Cell::new(false));
    let observed = seen.clone();
    let _hook = install(move |phase, _| {
        if phase == "retained_attempt" {
            seen.set(true);
            assert!(
                fs::rename(&attempt, &moved).is_err(),
                "commit payload validation lost retained attempt identity"
            );
        }
    });
    f.store
        .commit_prepared_selected_embedded_freeze(prepared)
        .unwrap();
    assert!(observed.get());
    let base = f.base.clone();
    drop(f);
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn source_capture_depth_budget_rejects_before_start() {
    let mut f = Fixture::new();
    let mut path = f.source.clone();
    for _ in 0..64 {
        path.push("d");
        fs::create_dir(&path).unwrap();
    }
    assert!(collect_selected_embedded_source(&f.source).is_ok());
    path.push("d");
    fs::create_dir(&path).unwrap();
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)
    );
    assert_eq!(
        f.store
            .retained_journal()
            .current_head_reference()
            .entry_index()
            .value(),
        0
    );
    let base = f.base.clone();
    drop(f);
    fs::remove_dir_all(base).unwrap();
}

struct Fixture {
    base: PathBuf,
    source: PathBuf,
    store: AuthoritativeRegistryStore,
    policy: RecordId,
}
impl Fixture {
    fn new() -> Self {
        let base = std::env::current_dir()
            .unwrap()
            .join("target")
            .join(format!(
                "source-capture-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&base).unwrap();
        let source = base.join("source");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("a"), b"original").unwrap();
        let root = base.join("store");
        let store = AuthoritativeRegistryStore::initialize_selected_profile(
            &root,
            RegistryId::try_from(&[0x77; 32][..]).unwrap(),
            "capture-test",
        )
        .unwrap();
        // Historical fixture setup, not public prerequisite-producer acceptance evidence.
        let mut scope = vec![0x84, 0x78, 0x1a];
        scope.extend_from_slice(b"EvidenceRegistry.Record.v1");
        scope.extend_from_slice(&[0x14, 1, 0xa5, 0, 1, 1, 0x14, 0x10, 2, 0x11, 1, 0x12, 0x40]);
        let scope_id = RecordId::try_from(&Sha256::digest(&scope)[..]).unwrap();
        let mut bytes = vec![0x84, 0x78, 0x1a];
        bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
        bytes.extend_from_slice(&[0x18, 0x28, 1, 0xa5, 0, 1, 1, 0x18, 0x28, 0x10]);
        fn b32(out: &mut Vec<u8>, b: &[u8]) {
            out.extend_from_slice(&[0x58, 0x20]);
            out.extend_from_slice(b);
        }
        b32(&mut bytes, scope_id.as_bytes());
        bytes.extend_from_slice(&[0x18, 0x1d, 0x85]);
        let head = store.retained_journal().current_head_reference();
        b32(&mut bytes, head.registry_id().as_bytes());
        bytes.push(0);
        b32(&mut bytes, head.entry_hash().as_bytes());
        bytes.push(1);
        b32(&mut bytes, head.event_record_id().as_bytes());
        bytes.extend_from_slice(&[0x18, 0x1e, 0x81, 1]);
        let policy = RecordId::try_from(&Sha256::digest(&bytes)[..]).unwrap();
        for (id, bytes) in [(scope_id, scope), (policy, bytes)] {
            let name: String = id.as_bytes().iter().map(|b| format!("{b:02x}")).collect();
            fs::write(root.join("records").join(format!("{name}.cbor")), bytes).unwrap();
        }
        drop(store);
        let store = AuthoritativeRegistryStore::open_selected_profile(&root).unwrap();
        Self {
            base,
            source,
            store,
            policy,
        }
    }
    fn prepare(
        &mut self,
    ) -> Result<PreparedSelectedEmbeddedFreeze, SelectedEmbeddedFreezePreparationError> {
        self.store
            .prepare_selected_embedded_freeze(SelectedEmbeddedFreezePreparationInput {
                source_root: self.source.clone(),
                freeze_attempt_id: FreezeAttemptId::try_from(&[0x88; 32][..]).unwrap(),
                policy_record_id: self.policy,
            })
    }
}
#[cfg(windows)]
#[test]
fn source_capture_preparation_retains_root_between_passes() {
    let mut f = Fixture::new();
    let source = f.source.clone();
    let moved = f.base.join("moved");
    let seen = Rc::new(Cell::new(false));
    let observed = seen.clone();
    let _hook = install(move |phase, _| {
        if phase == "snapshot" {
            seen.set(true);
            assert!(
                fs::rename(&source, &moved).is_err(),
                "bound source root was replaceable between actual preparation scans"
            );
        }
    });
    let prepared = f.prepare().unwrap();
    assert!(observed.get());
    assert_eq!(
        fs::read(prepared.payload_directory().join("a")).unwrap(),
        b"original"
    );
    let base = f.base.clone();
    drop(f);
    fs::remove_dir_all(base).unwrap();
}

#[cfg(windows)]
#[test]
fn source_capture_intermediate_swap_never_reads_outside() {
    let mut f = Fixture::new();
    let directory = f.source.join("z");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("a"), b"original").unwrap();
    let outside = f.base.join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("a"), b"SENTINEL").unwrap();
    OUTSIDE.with(|slot| *slot.borrow_mut() = Some(fs::File::open(outside.join("a")).unwrap()));
    let moved = f.base.join("moved");
    let fired = Rc::new(Cell::new(false));
    let observed = fired.clone();
    let _hook = install(move |phase, path| {
        if !fired.get() && phase == "directory" && path == [b"z".to_vec()] {
            fired.set(true);
            if fs::rename(&directory, &moved).is_ok() {
                std::os::windows::fs::symlink_dir(&outside, &directory).unwrap();
            }
        }
    });
    if let Ok(prepared) = f.prepare() {
        assert_eq!(
            fs::read(prepared.payload_directory().join("z/a")).unwrap(),
            b"original"
        );
    }
    assert!(observed.get());
    drop(_hook);
    let base = f.base.clone();
    drop(f);
    fs::remove_dir_all(base).unwrap();
}
