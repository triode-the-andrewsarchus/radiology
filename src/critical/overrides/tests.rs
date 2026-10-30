use super::*;

fn setup() -> (tempfile::TempDir, PathBuf, OverrideStore) {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path().join("repository with spaces");
    std::fs::create_dir(&root).unwrap();
    let store = OverrideStore::new(owner.path().join("settings/critical"));
    (owner, root, store)
}

fn decisions() -> CriticalityOverrides {
    let mut overrides = CriticalityOverrides::default();
    overrides.set_symbol(
        "rust-analyzer cargo domain 1 f().",
        Some(Criticality::Critical),
    );
    overrides.set_package("rust-analyzer cargo core", Some(Criticality::Ordinary));
    overrides
}

#[test]
fn atomic_marks_round_trip_isolate_roots_and_delete_without_creating_idle_storage() {
    let (_owner, root, store) = setup();
    assert!(!store.directory.exists());
    assert!(store
        .load(&root, &AtomicBool::new(false))
        .unwrap()
        .is_empty());
    assert!(!store.directory.exists());
    let overrides = decisions();
    assert_eq!(overrides.format_version, 1);
    assert!(store
        .save(&root, &overrides, store.reserve_revision())
        .unwrap());
    let loaded = store.load(&root, &AtomicBool::new(false)).unwrap();
    assert_eq!(loaded.symbols, overrides.symbols);
    assert_eq!(loaded.packages, overrides.packages);
    let other = root.with_file_name("other-repository");
    std::fs::create_dir(&other).unwrap();
    assert!(store
        .load(&other, &AtomicBool::new(false))
        .unwrap()
        .is_empty());
    store
        .save(&root, &Default::default(), store.reserve_revision())
        .unwrap();
    assert!(store
        .load(&root, &AtomicBool::new(false))
        .unwrap()
        .is_empty());
    assert_eq!(std::fs::read_dir(&store.directory).unwrap().count(), 0);
    assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
}

#[test]
fn delayed_saves_cannot_restore_older_choices_after_newer_changes_or_deletion() {
    let (_owner, root, store) = setup();
    let older = store.reserve_revision();
    let newer = store.reserve_revision();
    let mut latest = decisions();
    latest.set_package("rust-analyzer cargo core", Some(Criticality::Critical));
    store.clone().save(&root, &latest, newer).unwrap();
    assert!(!store.save(&root, &decisions(), older).unwrap());
    assert_eq!(
        store.load(&root, &AtomicBool::new(false)).unwrap().packages,
        latest.packages
    );
    let deletion = store.reserve_revision();
    store.save(&root, &Default::default(), deletion).unwrap();
    assert!(!store.save(&root, &latest, newer).unwrap());
    assert!(store
        .load(&root, &AtomicBool::new(false))
        .unwrap()
        .is_empty());
}

#[test]
fn corrupt_foreign_and_unsupported_marks_are_rejected_and_locals_never_persist() {
    let (_owner, root, store) = setup();
    store
        .save(&root, &decisions(), store.reserve_revision())
        .unwrap();
    let (path, _) = store.location(&root).unwrap();
    let pristine = std::fs::read(&path).unwrap();
    for (field, value) in [
        ("format_version", serde_json::json!(999)),
        ("repository_root", serde_json::json!("/foreign")),
    ] {
        let mut invalid: serde_json::Value = serde_json::from_slice(&pristine).unwrap();
        invalid[field] = value;
        std::fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(store.load(&root, &AtomicBool::new(false)).is_err());
    }
    std::fs::write(&path, b"{truncated").unwrap();
    assert!(store.load(&root, &AtomicBool::new(false)).is_err());
    let mut invalid = decisions();
    invalid
        .symbols
        .insert("local 7".into(), Criticality::Critical);
    invalid
        .symbols
        .insert("unknown".into(), Criticality::Unknown);
    store
        .save(&root, &invalid, store.reserve_revision())
        .unwrap();
    let loaded = store.load(&root, &AtomicBool::new(false)).unwrap();
    assert!(!loaded.symbols.contains_key("local 7"));
    assert!(!loaded.symbols.contains_key("unknown"));
}

#[test]
fn pending_edits_replay_on_loaded_choices_without_losing_unrelated_marks() {
    let mut loaded = decisions();
    let edit = MarkEdit {
        target: MarkTarget::Symbol("rust-analyzer cargo domain 1 other().".into()),
        decision: Some(Criticality::Ordinary),
    };
    let reset = MarkEdit {
        target: MarkTarget::Package("rust-analyzer cargo core".into()),
        decision: Some(Criticality::Unknown),
    };
    assert!(edit.apply(&mut loaded));
    assert!(!edit.apply(&mut loaded));
    assert!(reset.apply(&mut loaded));
    assert!(loaded.packages.is_empty());
    assert_eq!(loaded.symbols.len(), 2);
    assert!(!MarkEdit {
        target: MarkTarget::Symbol("local 7".into()),
        decision: Some(Criticality::Critical)
    }
    .apply(&mut loaded));
}

#[test]
fn failed_newer_saves_preserve_disk_and_prevent_delayed_older_snapshots() {
    let (_owner, root, store) = setup();
    store
        .save(&root, &decisions(), store.reserve_revision())
        .unwrap();
    let (path, _) = store.location(&root).unwrap();
    let pristine = std::fs::read(&path).unwrap();
    let delayed = store.reserve_revision();
    let mut oversized = decisions();
    oversized.set_symbol(&"x".repeat(MAX_BYTES as usize), Some(Criticality::Ordinary));
    assert!(store
        .save(&root, &oversized, store.reserve_revision())
        .is_err());
    assert_eq!(std::fs::read(&path).unwrap(), pristine);
    assert!(!store.save(&root, &Default::default(), delayed).unwrap());
    assert_eq!(std::fs::read(&path).unwrap(), pristine);
    assert!(store.load(&root, &AtomicBool::new(true)).is_err());
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    assert!(store
        .save(&root, &Default::default(), store.reserve_revision())
        .is_err());
    assert!(path.is_dir());
}

#[test]
fn marks_reject_inside_repository_storage_symlinks_and_oversized_files() {
    let (owner, root, store) = setup();
    let inside = OverrideStore::new(root.join("settings"));
    assert!(inside.load(&root, &AtomicBool::new(false)).is_err());
    assert!(inside
        .save(&root, &decisions(), inside.reserve_revision())
        .is_err());
    assert!(!root.join("settings").exists());
    assert!(store.load(&root, &AtomicBool::new(true)).is_err());
    store
        .save(&root, &decisions(), store.reserve_revision())
        .unwrap();
    let (path, _) = store.location(&root).unwrap();
    #[cfg(unix)]
    {
        let alias = owner.path().join("root-alias");
        std::os::unix::fs::symlink(&root, &alias).unwrap();
        assert_eq!(
            store.load(&alias, &AtomicBool::new(false)).unwrap(),
            store.load(&root, &AtomicBool::new(false)).unwrap()
        );
        std::fs::remove_file(&path).unwrap();
        let target = owner.path().join("unrelated.json");
        std::fs::write(&target, b"{}").unwrap();
        std::os::unix::fs::symlink(&target, &path).unwrap();
        assert!(store.load(&root, &AtomicBool::new(false)).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"{}");
        std::fs::remove_file(&path).unwrap();
    }
    let sparse = std::fs::File::create(&path).unwrap();
    sparse.set_len(MAX_BYTES + 1).unwrap();
    assert!(store
        .load(&root, &AtomicBool::new(false))
        .err()
        .unwrap()
        .contains("4 MiB"));
}

#[test]
fn concurrent_store_clones_keep_the_latest_reserved_decision() {
    let (_owner, root, store) = setup();
    let barrier = std::sync::Barrier::new(2);
    let older = store.reserve_revision();
    let newer = store.reserve_revision();
    let mut latest = decisions();
    latest.set_package("rust-analyzer cargo core", Some(Criticality::Critical));
    std::thread::scope(|scope| {
        let root = &root;
        let store = &store;
        let barrier = &barrier;
        scope.spawn(move || {
            barrier.wait();
            store.clone().save(root, &decisions(), older).unwrap();
        });
        let latest = &latest;
        scope.spawn(move || {
            barrier.wait();
            store.clone().save(root, latest, newer).unwrap();
        });
    });
    assert_eq!(
        store.load(&root, &AtomicBool::new(false)).unwrap().packages,
        latest.packages
    );
}
