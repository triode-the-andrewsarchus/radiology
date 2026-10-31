use super::*;
use std::{
    fs,
    os::unix::{
        ffi::OsStrExt,
        fs::{symlink, PermissionsExt},
    },
};

#[test]
fn reveal_uses_literal_file_uris_for_files_directories_and_unusual_names() {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path().canonicalize().unwrap();
    let name = "space ' quote\"; $(touch sentinel) #é🦀\\\n.rs";
    fs::write(root.join(name), "unchanged source").unwrap();
    fs::create_dir(root.join("folder")).unwrap();
    for relative in [name, "folder"] {
        let command = prepare_reveal(&root, relative).unwrap();
        assert_eq!(command.executable, Path::new("gdbus"));
        assert_eq!(
            command.arguments[7],
            "org.freedesktop.FileManager1.ShowItems"
        );
        let uris: Vec<String> =
            serde_json::from_str(command.arguments[8].to_str().unwrap()).unwrap();
        assert_eq!(uris.len(), 1);
        assert_eq!(
            url::Url::parse(&uris[0]).unwrap().to_file_path().unwrap(),
            root.join(relative)
        );
        assert_eq!(command.arguments[9], "");
        assert_eq!(command.directory, root);
    }
    let non_utf8 = std::ffi::OsStr::from_bytes(b"target-\xff");
    fs::write(root.join(non_utf8), "bytes").unwrap();
    symlink(non_utf8, root.join("selected-link")).unwrap();
    let command = prepare_reveal(&root, "selected-link").unwrap();
    assert!(command.arguments[8]
        .to_str()
        .unwrap()
        .contains("selected-link"));
    assert!(!root.join("sentinel").exists());
    assert_eq!(
        fs::read_to_string(root.join(name)).unwrap(),
        "unchanged source"
    );
}

#[test]
fn reveal_rejects_traversal_missing_entries_and_escaping_ancestors() {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path().join("root");
    fs::create_dir(&root).unwrap();
    fs::write(owner.path().join("outside.rs"), "outside").unwrap();
    symlink(owner.path(), root.join("outside")).unwrap();
    for relative in [
        "",
        "../outside.rs",
        "/outside.rs",
        "missing",
        "outside/outside.rs",
    ] {
        assert!(prepare_reveal(&root, relative).is_err(), "{relative}");
    }
    fs::create_dir(root.join("inside")).unwrap();
    fs::write(root.join("inside/file.rs"), "inside").unwrap();
    symlink("inside", root.join("alias")).unwrap();
    assert!(prepare_reveal(&root, "alias/file.rs").is_ok());
}

#[test]
fn reveal_reports_client_failure_and_bounds_waits_without_owning_the_desktop_app() {
    let owner = tempfile::tempdir().unwrap();
    fs::write(owner.path().join("file.rs"), "source").unwrap();
    let mut command = prepare_reveal(owner.path(), "file.rs").unwrap();
    let client = crate::repository::test_support::fake_git(owner.path(), "printf '()\\n'");
    command.executable = client.clone();
    assert!(show_items(&command, &AtomicBool::new(false), Duration::from_secs(1)).is_ok());
    fs::write(
        &client,
        "#!/bin/sh\nprintf 'No file manager registered' >&2\nexit 4\n",
    )
    .unwrap();
    fs::set_permissions(&client, fs::Permissions::from_mode(0o755)).unwrap();
    let error = show_items(&command, &AtomicBool::new(false), Duration::from_secs(1)).unwrap_err();
    assert!(error.contains("status 4") && error.contains("No file manager registered"));
    fs::write(&client, "#!/bin/sh\nexec /bin/sleep 30\n").unwrap();
    assert!(
        show_items(&command, &AtomicBool::new(false), Duration::from_millis(50))
            .unwrap_err()
            .contains("timed out")
    );
    assert!(
        show_items(&command, &AtomicBool::new(true), Duration::from_secs(1))
            .unwrap_err()
            .contains("cancelled")
    );
    command.executable = owner.path().join("missing-gdbus");
    assert!(
        show_items(&command, &AtomicBool::new(false), Duration::from_secs(1))
            .unwrap_err()
            .contains("Requires gdbus")
    );
}
