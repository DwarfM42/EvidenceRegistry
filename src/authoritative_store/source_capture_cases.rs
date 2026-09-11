//! Regression controls using the actual Store preparation/commit routes.
use super::*;

#[test]
fn source_capture_empty_directory_added_between_passes_preserves_start() {
    let mut f = Fixture::new();
    let extra = f.source.join("empty");
    let (hook, seen) = once("snapshot", move |_| {
        fs::create_dir(&extra).unwrap();
    });
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceChanged)
    );
    assert!(seen.get());
    assert_eq!(head(&f), 1);
    drop(hook);
    cleanup(f);
}

#[cfg(windows)]
#[test]
fn source_capture_repeated_scans_release_handles() {
    const NAME: &str = "authoritative_store::source_capture_tests::cases::source_capture_repeated_scans_release_handles";
    if std::env::var_os("EVIDENCE_CAPTURE_HANDLE_COUNT_CHILD").is_none() {
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", NAME, "--nocapture", "--test-threads=1"])
            .env("EVIDENCE_CAPTURE_HANDLE_COUNT_CHILD", "1")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while child.try_wait().unwrap().is_none() {
            if std::time::Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("source handle-count probe exceeded test watchdog");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        print!("{}", String::from_utf8_lossy(&output.stdout));
        return;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> *mut core::ffi::c_void;
        fn GetProcessHandleCount(process: *mut core::ffi::c_void, count: *mut u32) -> i32;
    }
    fn count() -> u32 {
        let mut count = 0;
        assert_ne!(
            unsafe { GetProcessHandleCount(GetCurrentProcess(), &mut count) },
            0
        );
        count
    }
    let f = Fixture::new();
    let mut path = f.source.clone();
    for _ in 0..32 {
        path.push("d");
        fs::create_dir(&path).unwrap();
    }
    let hold = source_capture::bind(&f.source).unwrap();
    collect_selected_embedded_source_from_hold(&hold).unwrap();
    let before = count();
    for _ in 0..25 {
        collect_selected_embedded_source_from_hold(&hold).unwrap();
    }
    let after = count();
    println!("capture process handle count: before={before} after={after}");
    assert_eq!(after, before);
    drop(hold);
    cleanup(f);
}

#[cfg(windows)]
#[test]
fn source_capture_ambiguous_or_invalid_utf16_names_fail_before_start() {
    use std::os::windows::ffi::OsStringExt;
    for name in [
        std::ffi::OsString::from("trailing."),
        std::ffi::OsString::from("space "),
        std::ffi::OsString::from_wide(&[0xd800]),
    ] {
        let mut f = Fixture::new();
        let leaf = fs::canonicalize(&f.source).unwrap().join(name);
        fs::write(&leaf, b"ambiguous").unwrap();
        assert_eq!(
            f.prepare(),
            Err(SelectedEmbeddedFreezePreparationError::SourceInvalid)
        );
        assert_eq!(head(&f), 0);
        cleanup(f);
    }
}

#[cfg(windows)]
#[test]
fn source_capture_junction_after_enumeration_never_reads_target() {
    let mut f = Fixture::new();
    let outside = f.base.join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("a"), b"SENTINEL").unwrap();
    OUTSIDE.with(|s| *s.borrow_mut() = Some(fs::File::open(outside.join("a")).unwrap()));
    let child = f.source.join("z");
    fs::create_dir(&child).unwrap();
    let (hook, seen) = once("names", move |_| {
        fs::remove_dir(&child).unwrap();
        let output = std::process::Command::new("cmd.exe")
            .args(["/d", "/c", "mklink", "/J"])
            .arg(&child)
            .arg(&outside)
            .output()
            .unwrap();
        assert!(output.status.success(), "junction fixture: {:?}", output);
    });
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceInvalid)
    );
    assert!(seen.get());
    assert_eq!(head(&f), 0);
    drop(hook);
    cleanup(f);
}

#[test]
fn source_capture_postbind_ancestor_change_does_not_adopt_replacement() {
    let mut f = Fixture::new();
    let parent = f.base.join("parent");
    fs::create_dir(&parent).unwrap();
    let source = parent.join("source");
    fs::rename(&f.source, &source).unwrap();
    f.source = source.clone();
    let moved = f.base.join("moved-parent");
    let (hook, seen) = once("bound", move |_| {
        if fs::rename(&parent, &moved).is_ok() {
            fs::create_dir(&parent).unwrap();
            fs::create_dir(&source).unwrap();
            fs::write(source.join("a"), b"SENTINEL").unwrap();
        } else {
            #[cfg(not(windows))]
            panic!("POSIX ancestor rename unexpectedly unavailable");
        }
    });
    let prepared = f.prepare().unwrap();
    assert!(seen.get());
    assert_eq!(
        fs::read(prepared.payload_directory().join("a")).unwrap(),
        b"original"
    );
    drop(hook);
    cleanup(f);
}
fn cleanup(f: Fixture) {
    let base = f.base.clone();
    drop(f);
    fs::remove_dir_all(base).unwrap();
}
fn head(f: &Fixture) -> u64 {
    f.store
        .retained_journal()
        .current_head_reference()
        .entry_index()
        .value()
}
fn once(
    phase: &'static str,
    mut action: impl FnMut(&[Vec<u8>]) + 'static,
) -> (Reset, Rc<Cell<bool>>) {
    let fired = Rc::new(Cell::new(false));
    let seen = fired.clone();
    let reset = install(move |p, path| {
        if p == phase && !fired.get() {
            fired.set(true);
            action(path);
        }
    });
    (reset, seen)
}
#[cfg(windows)]
fn symlink_file(target: &Path, link: &Path) {
    std::os::windows::fs::symlink_file(target, link).unwrap();
}
#[cfg(unix)]
fn symlink_file(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).unwrap();
}
#[cfg(windows)]
fn symlink_dir(target: &Path, link: &Path) {
    std::os::windows::fs::symlink_dir(target, link).unwrap();
}
#[cfg(unix)]
fn symlink_dir(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).unwrap();
}

#[test]
fn source_capture_leaf_symlink_after_enumeration_rejects_without_read() {
    let mut f = Fixture::new();
    let outside = f.base.join("outside");
    fs::write(&outside, b"SENTINEL").unwrap();
    OUTSIDE.with(|s| *s.borrow_mut() = Some(fs::File::open(&outside).unwrap()));
    let leaf = f.source.join("a");
    let (hook, seen) = once("names", move |_| {
        fs::remove_file(&leaf).unwrap();
        symlink_file(&outside, &leaf);
    });
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceInvalid)
    );
    assert!(seen.get());
    assert_eq!(head(&f), 0);
    drop(hook);
    cleanup(f);
}
#[test]
fn source_capture_hardlink_after_enumeration_rejects_without_read() {
    let mut f = Fixture::new();
    let outside = f.base.join("outside");
    fs::write(&outside, b"SENTINEL").unwrap();
    OUTSIDE.with(|s| *s.borrow_mut() = Some(fs::File::open(&outside).unwrap()));
    let leaf = f.source.join("a");
    let (hook, seen) = once("names", move |_| {
        fs::remove_file(&leaf).unwrap();
        fs::hard_link(&outside, &leaf).unwrap();
    });
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceInvalid)
    );
    assert!(seen.get());
    assert_eq!(head(&f), 0);
    drop(hook);
    cleanup(f);
}
#[test]
fn source_capture_directory_link_after_enumeration_rejects_without_read() {
    let mut f = Fixture::new();
    let outside = f.base.join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("a"), b"SENTINEL").unwrap();
    OUTSIDE.with(|s| *s.borrow_mut() = Some(fs::File::open(outside.join("a")).unwrap()));
    let child = f.source.join("z");
    fs::create_dir(&child).unwrap();
    let moved = f.base.join("moved");
    let (hook, seen) = once("names", move |_| {
        fs::rename(&child, &moved).unwrap();
        symlink_dir(&outside, &child);
    });
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceInvalid)
    );
    assert!(seen.get());
    assert_eq!(head(&f), 0);
    drop(hook);
    cleanup(f);
}
#[test]
fn source_capture_root_link_rejects_before_start() {
    let mut f = Fixture::new();
    let moved = f.base.join("moved");
    fs::rename(&f.source, &moved).unwrap();
    symlink_dir(&moved, &f.source);
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceInvalid)
    );
    assert_eq!(head(&f), 0);
    cleanup(f);
}
#[cfg(windows)]
#[test]
fn source_capture_held_leaf_blocks_write_and_replace() {
    let mut f = Fixture::new();
    let leaf = f.source.join("a");
    let moved = f.base.join("moved");
    let (hook, seen) = once("file", move |_| {
        assert!(fs::rename(&leaf, &moved).is_err());
        assert!(fs::write(&leaf, b"SENTINEL").is_err());
    });
    let prepared = f.prepare().unwrap();
    assert!(seen.get());
    assert_eq!(
        fs::read(prepared.payload_directory().join("a")).unwrap(),
        b"original"
    );
    drop(hook);
    cleanup(f);
}
#[cfg(windows)]
#[test]
fn source_capture_held_leaf_blocks_post_read_truncate() {
    let mut f = Fixture::new();
    let leaf = f.source.join("a");
    let (hook, seen) = once("read", move |_| {
        assert!(fs::write(&leaf, b"").is_err());
    });
    f.prepare().unwrap();
    assert!(seen.get());
    drop(hook);
    cleanup(f);
}
#[test]
fn source_capture_same_byte_ordinary_replacement_before_open_is_new_capture() {
    let mut f = Fixture::new();
    let leaf = f.source.join("a");
    let moved = f.base.join("moved");
    let (hook, seen) = once("names", move |_| {
        fs::rename(&leaf, &moved).unwrap();
        fs::write(&leaf, b"original").unwrap();
    });
    f.prepare().unwrap();
    assert!(seen.get());
    drop(hook);
    cleanup(f);
}
#[test]
fn source_capture_membership_drift_before_start_fails() {
    let mut f = Fixture::new();
    let extra = f.source.join("extra");
    let (hook, seen) = once("names", move |_| {
        fs::write(&extra, b"extra").unwrap();
    });
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceChanged)
    );
    assert!(seen.get());
    assert_eq!(head(&f), 0);
    drop(hook);
    cleanup(f);
}
#[test]
fn source_capture_opened_length_limit_is_not_enumeration_hint() {
    let mut f = Fixture::new();
    let leaf = f.source.join("a");
    let (hook, seen) = once("names", move |_| {
        fs::File::create(&leaf)
            .unwrap()
            .set_len(AUTHORITATIVE_STORE_MAX_OBJECT_BYTES as u64 + 1)
            .unwrap();
    });
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)
    );
    assert!(seen.get());
    assert_eq!(head(&f), 0);
    drop(hook);
    cleanup(f);
}
#[test]
fn source_capture_final_source_drift_preserves_unresolved_start() {
    for bytes in [b"SENTINEL".as_slice(), b"", b"original and grown"] {
        let mut f = Fixture::new();
        let leaf = f.source.join("a");
        let (hook, seen) = once("snapshot", move |_| {
            fs::write(&leaf, bytes).unwrap();
        });
        assert_eq!(
            f.prepare(),
            Err(SelectedEmbeddedFreezePreparationError::SourceChanged)
        );
        assert!(seen.get());
        assert_eq!(head(&f), 1);
        let store_root = f.base.join("store");
        drop(hook);
        drop(f.store);
        let reopened = AuthoritativeRegistryStore::open_selected_profile(&store_root).unwrap();
        assert_eq!(
            reopened
                .retained_journal()
                .current_head_reference()
                .event_type_id()
                .value(),
            100
        );
        drop(reopened);
        fs::remove_dir_all(f.base).unwrap();
    }
}
#[test]
fn source_capture_final_membership_drift_preserves_start() {
    let mut f = Fixture::new();
    let extra = f.source.join("extra");
    let (hook, seen) = once("snapshot", move |_| {
        fs::write(&extra, b"extra").unwrap();
    });
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceChanged)
    );
    assert!(seen.get());
    assert_eq!(head(&f), 1);
    drop(hook);
    cleanup(f);
}
#[test]
fn source_capture_payload_corruption_at_rescan_preserves_start() {
    let mut f = Fixture::new();
    let leaf = f
        .base
        .join("store/roots")
        .join("88".repeat(32))
        .join("payload/a");
    let (hook, seen) = once("payload", move |_| {
        fs::write(&leaf, b"tampered").unwrap();
    });
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceChanged)
    );
    assert!(seen.get());
    assert_eq!(head(&f), 1);
    drop(hook);
    cleanup(f);
}
#[cfg(windows)]
#[test]
fn source_capture_payload_root_replace_is_blocked_during_rescan() {
    let mut f = Fixture::new();
    let payload = f
        .base
        .join("store/roots")
        .join("88".repeat(32))
        .join("payload");
    let moved = f.base.join("moved");
    let (hook, seen) = once("payload", move |_| {
        assert!(fs::rename(&payload, &moved).is_err());
    });
    f.prepare().unwrap();
    assert!(seen.get());
    drop(hook);
    cleanup(f);
}
#[test]
fn source_capture_cold_payload_corruption_invalidates_committed_freeze() {
    let mut f = Fixture::new();
    let prepared = f.prepare().unwrap();
    let leaf = prepared.payload_directory().join("a");
    let committed = f
        .store
        .commit_prepared_selected_embedded_freeze(prepared)
        .unwrap();
    let store_root = f.base.join("store");
    drop(f.store);
    fs::write(&leaf, b"tampered").unwrap();
    let reopened = AuthoritativeRegistryStore::open_selected_profile(&store_root).unwrap();
    assert_eq!(
        reopened.validate_freeze_committed_authority(committed.committed_event_reference().clone()),
        Err(AuthoritativeFreezeCommittedBindingError::SelectedPrerequisiteInvalid)
    );
    drop(reopened);
    fs::remove_dir_all(f.base).unwrap();
}
#[test]
fn source_capture_repeated_scans_have_fresh_cursors() {
    let f = Fixture::new();
    for n in 0..100 {
        fs::write(f.source.join(format!("f{n:03}")), [n as u8]).unwrap();
    }
    let hold = source_capture::bind(&f.source).unwrap();
    let first = collect_selected_embedded_source_from_hold(&hold).unwrap();
    assert_eq!(first.artifacts.len(), 101);
    for _ in 0..10 {
        assert_eq!(
            collect_selected_embedded_source_from_hold(&hold).unwrap(),
            first
        );
    }
    drop(hold);
    cleanup(f);
}
#[test]
fn source_capture_entry_budget_counts_empty_directories() {
    let mut f = Fixture::new();
    for n in 0..SELECTED_EMBEDDED_CAPTURE_MAX_FILES {
        fs::create_dir(f.source.join(format!("d{n:04}"))).unwrap();
    }
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)
    );
    assert_eq!(head(&f), 0);
    cleanup(f);
}
#[test]
fn source_capture_aggregate_content_budget_rejects() {
    let mut f = Fixture::new();
    for n in 0..65 {
        fs::File::create(f.source.join(format!("f{n:03}")))
            .unwrap()
            .set_len(AUTHORITATIVE_STORE_MAX_OBJECT_BYTES as u64)
            .unwrap();
    }
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceResourceLimit)
    );
    assert_eq!(head(&f), 0);
    cleanup(f);
}

#[cfg(unix)]
#[test]
fn source_capture_same_root_survives_ambient_root_replacement() {
    let mut f = Fixture::new();
    let source = f.source.clone();
    let moved = f.base.join("moved");
    let (hook, seen) = once("snapshot", move |_| {
        fs::rename(&source, &moved).unwrap();
        fs::create_dir(&source).unwrap();
        fs::write(source.join("a"), b"SENTINEL").unwrap();
    });
    let prepared = f.prepare().unwrap();
    assert!(seen.get());
    assert_eq!(
        fs::read(prepared.payload_directory().join("a")).unwrap(),
        b"original"
    );
    drop(hook);
    cleanup(f);
}
#[cfg(unix)]
#[test]
fn source_capture_opened_directory_detachment_fails_relation_check() {
    let mut f = Fixture::new();
    let child = f.source.join("z");
    fs::create_dir(&child).unwrap();
    fs::write(child.join("a"), b"original").unwrap();
    let moved = f.base.join("moved");
    let (hook, seen) = once("directory", move |_| {
        fs::rename(&child, &moved).unwrap();
        fs::create_dir(&child).unwrap();
        fs::write(child.join("a"), b"SENTINEL").unwrap();
    });
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceChanged)
    );
    assert!(seen.get());
    assert_eq!(head(&f), 0);
    drop(hook);
    cleanup(f);
}
#[cfg(unix)]
#[test]
fn source_capture_fifo_is_rejected_without_blocking_open() {
    use std::os::unix::ffi::OsStrExt;
    let mut f = Fixture::new();
    let fifo = std::ffi::CString::new(f.source.join("fifo").as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    assert_eq!(
        f.prepare(),
        Err(SelectedEmbeddedFreezePreparationError::SourceInvalid)
    );
    assert_eq!(head(&f), 0);
    cleanup(f);
}
