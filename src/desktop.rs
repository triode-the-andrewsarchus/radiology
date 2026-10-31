//! File-manager selection through a short-lived D-Bus client, independent of egui.

use crate::{process::ProcessCommand, source::SourceLocation};
use std::{path::Path, sync::atomic::AtomicBool, time::Duration};

pub(crate) fn prepare_reveal(root: &Path, relative: &str) -> Result<ProcessCommand, String> {
    if !SourceLocation::is_repository_path(relative) {
        return Err("Invalid repository-relative reveal path".into());
    }
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let candidate = root.join(relative);
    let parent = candidate
        .parent()
        .ok_or("Reveal path has no parent")?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if !parent.starts_with(&root) {
        return Err("Reveal path leaves the repository".into());
    }
    // Select the directory entry itself, including a final symlink, rather than
    // following it to some other file. Ancestor links must stay within the root.
    let path = parent.join(candidate.file_name().ok_or("Reveal path has no filename")?);
    std::fs::symlink_metadata(&path)
        .map_err(|error| format!("Cannot reveal {relative}: {error}"))?;
    let uri =
        url::Url::from_file_path(&path).map_err(|()| "Cannot encode reveal path as a file URI")?;
    // JSON's quoted ASCII URI array is also valid GVariant syntax. Literal argv
    // avoids shell expansion, even for quotes, newlines and command-like names.
    let uris = serde_json::to_string(&[uri.as_str()]).map_err(|error| error.to_string())?;
    Ok(ProcessCommand {
        executable: "gdbus".into(),
        arguments: [
            "call",
            "--session",
            "--dest",
            "org.freedesktop.FileManager1",
            "--object-path",
            "/org/freedesktop/FileManager1",
            "--method",
            "org.freedesktop.FileManager1.ShowItems",
            &uris,
            "",
        ]
        .into_iter()
        .map(Into::into)
        .collect(),
        directory: root,
        environment: Default::default(),
        resources: vec![],
    })
}

pub(crate) fn show_items(
    command: &ProcessCommand,
    cancel: &AtomicBool,
    timeout: Duration,
) -> Result<(), String> {
    let output = crate::process::run(command, cancel, timeout)
        .map_err(|error| format!("Could not reveal item in File Manager.\n{error}\nRequires gdbus and a desktop file manager supporting file selection."))?;
    if output.succeeded() {
        Ok(())
    } else {
        Err(format!(
            "Could not reveal item in File Manager.\nRequest client {}\n{}\n{}",
            output.status_description(),
            output.stdout.text,
            output.stderr.text
        ))
    }
}

#[cfg(all(test, unix))]
mod tests;
