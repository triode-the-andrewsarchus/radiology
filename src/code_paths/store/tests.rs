use super::*;

fn setup() -> (tempfile::TempDir, PathBuf, CodePathStore) {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    let store = CodePathStore::new(owner.path().join("settings/code-paths"));
    (owner, root, store)
}

#[test]
fn canonical_repository_storage_preserves_ids_order_duplicates_and_other_checkouts() {
    let (_owner, root, store) = setup();
    let first = CodePath::parse("Boot\na.c:f:start\na.c:f:repeat").unwrap();
    let second = CodePath::parse("Boot\nb.c:g:stop").unwrap();
    assert!(store.load(&root).unwrap().is_empty());
    assert!(!store.directory.exists());
    store.save(&root, &[first.clone(), second.clone()]).unwrap();
    assert_eq!(
        store.load(&root.join(".")).unwrap(),
        [first.clone(), second.clone()]
    );
    assert!(std::fs::read_dir(&root).unwrap().next().is_none());
    let other = root.with_file_name("other-repo");
    std::fs::create_dir(&other).unwrap();
    assert!(store.load(&other).unwrap().is_empty());
    store.save(&other, std::slice::from_ref(&second)).unwrap();
    store.save(&root, std::slice::from_ref(&first)).unwrap();
    assert_eq!(store.load(&root).unwrap(), [first]);
    assert_eq!(store.load(&other).unwrap(), [second]);
    store.save(&root, &[]).unwrap();
    assert!(store.load(&root).unwrap().is_empty());
}

#[test]
fn corrupt_foreign_and_unsupported_archives_cannot_be_overwritten_by_import_or_delete() {
    let (_owner, root, store) = setup();
    let trace = CodePath::parse("Boot\na.c:f:start").unwrap();
    store.save(&root, std::slice::from_ref(&trace)).unwrap();
    let (path, _) = store.location(&root).unwrap();
    let valid = std::fs::read(&path).unwrap();
    let mut foreign: serde_json::Value = serde_json::from_slice(&valid).unwrap();
    foreign["repository_root"] = "foreign".into();
    let mut unsupported: serde_json::Value = serde_json::from_slice(&valid).unwrap();
    unsupported["version"] = 2.into();
    for bytes in [
        b"bad json".to_vec(),
        serde_json::to_vec(&foreign).unwrap(),
        serde_json::to_vec(&unsupported).unwrap(),
    ] {
        std::fs::write(&path, &bytes).unwrap();
        assert!(store.load(&root).is_err());
        assert!(store.save(&root, std::slice::from_ref(&trace)).is_err());
        assert!(store.save(&root, &[]).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    std::fs::write(&path, &valid).unwrap();
    let mut invalid = trace.clone();
    invalid.steps[0].path = "../outside".into();
    assert!(store.save(&root, &[invalid]).is_err());
    assert!(store.save(&root, &[trace.clone(), trace]).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), valid);
}

#[test]
fn load_filters_invalid_records_and_duplicate_trace_ids_without_rewriting_storage() {
    let (_owner, root, store) = setup();
    let trace = CodePath::parse("T\na.c:f:one\na.c:f:two").unwrap();
    store.save(&root, std::slice::from_ref(&trace)).unwrap();
    let (path, root) = store.location(&root).unwrap();
    let mut invalid = trace.clone();
    invalid.id = uuid::Uuid::new_v4();
    invalid.steps[1].id = invalid.steps[0].id;
    let bytes = serde_json::to_vec(&Archive {
        version: 1,
        repository_root: root.clone(),
        paths: vec![trace.clone(), invalid, trace.clone()],
    })
    .unwrap();
    std::fs::write(&path, &bytes).unwrap();
    assert_eq!(store.load(&root).unwrap(), [trace]);
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

#[cfg(unix)]
#[test]
fn canonical_aliases_share_scope_but_source_storage_and_linked_archives_are_rejected() {
    let (owner, root, store) = setup();
    let alias = owner.path().join("alias");
    std::os::unix::fs::symlink(&root, &alias).unwrap();
    let trace = CodePath::parse("T\na.c:f:x").unwrap();
    store.save(&alias, std::slice::from_ref(&trace)).unwrap();
    assert_eq!(
        store.load(&root).unwrap().as_slice(),
        std::slice::from_ref(&trace)
    );
    let inside = CodePathStore::new(alias.join("settings"));
    assert!(inside.load(&root).is_err());
    assert!(inside.save(&root, std::slice::from_ref(&trace)).is_err());
    assert!(!root.join("settings").exists());
    let (path, _) = store.location(&root).unwrap();
    let outside = owner.path().join("keep.json");
    std::fs::rename(&path, &outside).unwrap();
    std::os::unix::fs::symlink(&outside, &path).unwrap();
    let bytes = std::fs::read(&outside).unwrap();
    assert!(store.load(&root).is_err());
    assert!(store.save(&root, &[trace]).is_err());
    assert_eq!(std::fs::read(&outside).unwrap(), bytes);
}
