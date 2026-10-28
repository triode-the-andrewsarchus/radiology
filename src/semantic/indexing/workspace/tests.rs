use super::*;
use std::process::Command;

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let owner = tempfile::tempdir().unwrap();
    let source = owner.path().join("source");
    let cache = owner.path().join("cache");
    fs::create_dir_all(source.join("src")).unwrap();
    fs::write(source.join("src/main.rs"), b"fn main() {}\r\n").unwrap();
    fs::write(
        source.join("Cargo.toml"),
        b"[package]\nname = 'fixture'\nversion = '0.1.0'\n",
    )
    .unwrap();
    (owner, source, cache)
}

fn isolate(source: &Path, cache: &Path, cancel: &AtomicBool) -> IndexingWorkspace {
    let inventory = InputInventory::discover(source, cancel).unwrap();
    IndexingWorkspace::create(
        source,
        &inventory,
        cache,
        WorkspaceLimits::default(),
        cancel,
    )
    .unwrap()
}

#[test]
fn copy_preserves_bytes_fingerprints_and_drop_cleanup_without_changing_source() {
    let (_owner, source, cache) = fixture();
    let cancel = AtomicBool::new(false);
    let workspace = isolate(&source, &cache, &cancel);
    assert_eq!(
        fs::read(workspace.root().join("src/main.rs")).unwrap(),
        b"fn main() {}\r\n"
    );
    workspace.verify_inputs(&cancel).unwrap();
    workspace.verify_source(&cancel).unwrap();
    let fingerprint = workspace.fingerprint();
    let another = isolate(&source, &cache, &cancel);
    assert_eq!(fingerprint, another.fingerprint());
    fs::write(workspace.root().join("src/main.rs"), "rewritten").unwrap();
    assert!(workspace
        .verify_inputs(&cancel)
        .unwrap_err()
        .contains("rewritten"));
    workspace.verify_source(&cancel).unwrap();
    let path = workspace.root().to_owned();
    drop(workspace);
    assert!(!path.exists());
    assert_eq!(
        fs::read(source.join("src/main.rs")).unwrap(),
        b"fn main() {}\r\n"
    );
}

#[test]
fn git_inventory_streams_beyond_diagnostic_cap_and_keeps_tracked_build_inputs() {
    let (_owner, source, _) = fixture();
    git(&source, &["init", "-q"]);
    fs::create_dir(source.join("build")).unwrap();
    fs::create_dir(source.join(".cargo")).unwrap();
    fs::write(source.join("build/compile_commands.json"), "[]").unwrap();
    fs::write(source.join(".cargo/config.toml"), "[build]\njobs=1\n").unwrap();
    fs::write(source.join(".gitignore"), "build/\n").unwrap();
    git(
        &source,
        &[
            "add",
            "-f",
            "build/compile_commands.json",
            ".cargo/config.toml",
            "src/main.rs",
        ],
    );
    fs::write(source.join("build/ignored.cache"), "ignored").unwrap();
    fs::remove_file(source.join("src/main.rs")).unwrap();
    fs::create_dir(source.join("many")).unwrap();
    for n in 0..1100 {
        fs::write(
            source.join(format!("many/{n:04}-{}.rs", "long".repeat(20))),
            "",
        )
        .unwrap();
    }
    fs::write(source.join("many/Unicode café\nline.rs"), "").unwrap();
    let cancel = AtomicBool::new(false);
    let inputs = InputInventory::discover(&source, &cancel).unwrap();
    assert!(inputs.paths.contains(&"build/compile_commands.json".into()));
    assert!(inputs.paths.contains(&".cargo/config.toml".into()));
    assert!(inputs.paths.contains(&"many/Unicode café\nline.rs".into()));
    assert!(!inputs.paths.contains(&"build/ignored.cache".into()));
    assert!(!inputs.paths.contains(&"src/main.rs".into()));
    assert_eq!(
        inputs
            .paths
            .iter()
            .filter(|p| p.starts_with("many/"))
            .count(),
        1101
    );
    assert!(inputs.paths.iter().map(String::len).sum::<usize>() > 64 * 1024);
}

#[test]
fn invalid_paths_cache_location_limits_and_cancellation_leave_no_input_copy() {
    let (_owner, source, cache) = fixture();
    let cancel = AtomicBool::new(false);
    for path in [
        "../outside",
        "/absolute",
        ".git/config",
        "a/.git/config",
        "a//b",
        "a/./b",
    ] {
        assert!(InputInventory::from_paths([path.into()]).is_err());
    }
    let inputs = InputInventory::discover(&source, &cancel).unwrap();
    let internal = source.join("must-not-create/cache");
    assert!(IndexingWorkspace::create(
        &source,
        &inputs,
        &internal,
        WorkspaceLimits::default(),
        &cancel
    )
    .is_err());
    assert!(!source.join("must-not-create").exists());
    assert!(IndexingWorkspace::create(
        &source,
        &inputs,
        &cache,
        WorkspaceLimits {
            total_bytes: 4,
            file_bytes: 4
        },
        &cancel
    )
    .is_err());
    assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
    let cancelled = AtomicBool::new(true);
    assert!(IndexingWorkspace::create(
        &source,
        &inputs,
        &cache,
        WorkspaceLimits::default(),
        &cancelled
    )
    .is_err());
    assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
}

#[test]
fn snapshot_verification_matches_real_git_blob_ids_and_rejects_source_edits() {
    let (_owner, source, cache) = fixture();
    git(&source, &["init", "-q"]);
    git(&source, &["add", "Cargo.toml", "src/main.rs"]);
    let expected = BTreeMap::from([
        (
            "Cargo.toml".into(),
            git(&source, &["rev-parse", ":Cargo.toml"]),
        ),
        (
            "src/main.rs".into(),
            git(&source, &["rev-parse", ":src/main.rs"]),
        ),
    ]);
    let cancel = AtomicBool::new(false);
    let workspace = isolate(&source, &cache, &cancel);
    workspace.verify_snapshot(&expected).unwrap();
    let mut wrong = expected.clone();
    wrong.remove("Cargo.toml");
    assert!(workspace.verify_snapshot(&wrong).is_err());
    wrong.insert("Cargo.toml".into(), "wrong blob".into());
    assert!(workspace.verify_snapshot(&wrong).is_err());
    fs::write(source.join("src/main.rs"), "changed").unwrap();
    assert!(workspace.verify_source(&cancel).is_err());
    workspace.verify_inputs(&cancel).unwrap();
    let changed = isolate(&source, &cache, &cancel);
    assert_ne!(workspace.fingerprint(), changed.fingerprint());
}

#[cfg(unix)]
#[test]
fn internal_links_are_redirected_and_external_documentation_is_literal() {
    use std::os::unix::fs::symlink;
    let (owner, source, cache) = fixture();
    let outside = owner.path().join("outside.md");
    fs::write(&outside, "outside secret").unwrap();
    symlink("src/main.rs", source.join("relative.rs")).unwrap();
    symlink(source.join("src/main.rs"), source.join("absolute.rs")).unwrap();
    symlink("src", source.join("alias")).unwrap();
    symlink("missing/internal.rs", source.join("dangling.rs")).unwrap();
    symlink(&outside, source.join("instructions.md")).unwrap();
    let cancel = AtomicBool::new(false);
    let workspace = isolate(&source, &cache, &cancel);
    assert_eq!(
        fs::read(workspace.root().join("relative.rs")).unwrap(),
        b"fn main() {}\r\n"
    );
    assert_eq!(
        fs::read(workspace.root().join("alias/main.rs")).unwrap(),
        b"fn main() {}\r\n"
    );
    assert!(fs::read_link(workspace.root().join("absolute.rs"))
        .unwrap()
        .starts_with(workspace.root()));
    assert!(fs::read_link(workspace.root().join("dangling.rs"))
        .unwrap()
        .starts_with(workspace.root()));
    assert_eq!(
        fs::read_to_string(workspace.root().join("instructions.md")).unwrap(),
        outside.to_str().unwrap()
    );
    workspace.verify_inputs(&cancel).unwrap();
    workspace.verify_source(&cancel).unwrap();
    symlink(&outside, source.join("external.rs")).unwrap();
    let inputs = InputInventory::discover(&source, &cancel).unwrap();
    assert!(IndexingWorkspace::create(
        &source,
        &inputs,
        &cache,
        WorkspaceLimits::default(),
        &cancel
    )
    .unwrap_err()
    .contains("external build input"));
}

#[cfg(unix)]
#[test]
fn executable_modes_and_link_parent_replacements_are_part_of_input_identity() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let (_owner, source, cache) = fixture();
    fs::write(source.join("gradlew"), "#!/bin/sh\n").unwrap();
    fs::set_permissions(source.join("gradlew"), fs::Permissions::from_mode(0o755)).unwrap();
    let cancel = AtomicBool::new(false);
    let workspace = isolate(&source, &cache, &cancel);
    assert_eq!(
        file_mode(&fs::metadata(workspace.root().join("gradlew")).unwrap()),
        0o755
    );
    fs::set_permissions(
        workspace.root().join("gradlew"),
        fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    assert!(workspace.verify_inputs(&cancel).is_err());
    fs::rename(
        workspace.root().join("src"),
        workspace.root().join("old-src"),
    )
    .unwrap();
    symlink(source.join("src"), workspace.root().join("src")).unwrap();
    assert!(workspace.verify_inputs(&cancel).is_err());
    symlink(source.join("src"), source.join("linked-dir")).unwrap();
    let unsafe_inputs = InputInventory::from_paths(["linked-dir/main.rs".into()]).unwrap();
    assert!(IndexingWorkspace::create(
        &source,
        &unsafe_inputs,
        &cache,
        WorkspaceLimits::default(),
        &cancel
    )
    .unwrap_err()
    .contains("symbolic-link directory"));
}

#[test]
fn git_worktrees_are_supported_and_submodules_are_reported_without_fabrication() {
    let (owner, source, _) = fixture();
    git(&source, &["init", "-q"]);
    git(&source, &["add", "."]);
    git(
        &source,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
    );
    let commit = git(&source, &["rev-parse", "HEAD"]);
    fs::create_dir(source.join("submodule")).unwrap();
    git(
        &source,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{commit},submodule"),
        ],
    );
    let inputs = InputInventory::discover(&source, &AtomicBool::new(false)).unwrap();
    assert_eq!(inputs.omitted_directories, ["submodule"]);
    assert!(!inputs.paths.contains(&"submodule".into()));
    let detached = owner.path().join("detached");
    git(
        &source,
        &[
            "worktree",
            "add",
            "--detach",
            detached.to_str().unwrap(),
            &commit,
        ],
    );
    assert!(detached.join(".git").is_file());
    let inputs = InputInventory::discover(&detached, &AtomicBool::new(false)).unwrap();
    assert_eq!(inputs.paths, ["Cargo.toml", "src/main.rs"]);
}

#[test]
fn cancellation_during_materialization_removes_the_started_workspace() {
    use std::sync::Arc;
    let (_owner, source, cache) = fixture();
    File::create(source.join("000-large.bin"))
        .unwrap()
        .set_len(64 * 1024 * 1024)
        .unwrap();
    fs::create_dir(&cache).unwrap();
    let inventory = InputInventory::discover(&source, &AtomicBool::new(false)).unwrap();
    let cancelled = Arc::new(AtomicBool::new(false));
    let token = cancelled.clone();
    let source_copy = source.clone();
    let cache_copy = cache.clone();
    let worker = std::thread::spawn(move || {
        IndexingWorkspace::create(
            &source_copy,
            &inventory,
            &cache_copy,
            WorkspaceLimits::default(),
            &token,
        )
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while fs::read_dir(&cache).unwrap().next().is_none() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
    cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(worker.join().unwrap().unwrap_err().contains("cancelled"));
    assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
    assert_eq!(
        fs::metadata(source.join("000-large.bin")).unwrap().len(),
        64 * 1024 * 1024
    );
}

#[cfg(unix)]
#[test]
fn link_targets_match_git_blobs_and_workspace_parent_links_cannot_write_into_source() {
    use std::os::unix::fs::symlink;
    let (owner, source, cache) = fixture();
    symlink("src/main.rs", source.join("alias.rs")).unwrap();
    git(&source, &["init", "-q"]);
    git(&source, &["add", "."]);
    let cancel = AtomicBool::new(false);
    let inputs = InputInventory::discover(&source, &cancel).unwrap();
    let expected: BTreeMap<_, _> = inputs
        .paths
        .iter()
        .map(|path| {
            (
                path.clone(),
                git(&source, &["rev-parse", &format!(":{path}")]),
            )
        })
        .collect();
    let workspace = isolate(&source, &cache, &cancel);
    workspace.verify_snapshot(&expected).unwrap();
    let parent_link = owner.path().join("cache-link");
    symlink(&source, &parent_link).unwrap();
    assert!(IndexingWorkspace::create(
        &source,
        &inputs,
        &parent_link.join("new-directory"),
        WorkspaceLimits::default(),
        &cancel
    )
    .is_err());
    assert!(!source.join("new-directory").exists());
}

#[test]
fn read_only_fingerprints_match_workspace_inputs_and_detect_modes_links_and_budget_limits() {
    let (_owner, source, cache) = fixture();
    let cancel = AtomicBool::new(false);
    create_link(Path::new("src/main.rs"), &source.join("alias.rs")).unwrap();
    let inventory = InputInventory::discover(&source, &cancel).unwrap();
    let original = IndexingWorkspace::fingerprint_source(
        &source,
        &inventory,
        WorkspaceLimits::default(),
        &cancel,
    )
    .unwrap();
    assert!(!cache.exists());
    let workspace = isolate(&source, &cache, &cancel);
    assert_eq!(original, workspace.fingerprint());
    assert!(IndexingWorkspace::fingerprint_source(
        &source,
        &inventory,
        WorkspaceLimits {
            total_bytes: 1,
            file_bytes: 1
        },
        &cancel
    )
    .is_err());
    assert!(IndexingWorkspace::fingerprint_source(
        &source,
        &inventory,
        WorkspaceLimits::default(),
        &AtomicBool::new(true)
    )
    .is_err());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            source.join("src/main.rs"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        assert_ne!(
            original,
            IndexingWorkspace::fingerprint_source(
                &source,
                &inventory,
                WorkspaceLimits::default(),
                &cancel
            )
            .unwrap()
        );
    }
    fs::remove_file(source.join("alias.rs")).unwrap();
    create_link(Path::new("Cargo.toml"), &source.join("alias.rs")).unwrap();
    assert_ne!(
        original,
        IndexingWorkspace::fingerprint_source(
            &source,
            &inventory,
            WorkspaceLimits::default(),
            &cancel
        )
        .unwrap()
    );
}
