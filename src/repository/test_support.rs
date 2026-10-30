use std::path::{Path, PathBuf};

pub(crate) fn git(root: &Path, args: &[&str]) {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub(crate) fn repository(root: &Path) {
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "test@example.invalid"]);
    git(root, &["config", "user.name", "Test"]);
}

#[cfg(unix)]
pub(crate) fn fake_git(root: &Path, script: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = root.join("fake-git");
    std::fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path
}
