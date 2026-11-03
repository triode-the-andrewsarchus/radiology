//! Shared private-settings file replacement; domain stores validate ownership first.
use std::{
    io::{self, Write},
    path::Path,
};

/// Creates the parent and replaces one file only after its new contents are synced.
/// Errors leave an existing destination intact; temporary files clean themselves up.
pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    stage(path, bytes)?
        .persist(path)
        .map_err(|error| error.error)?;
    Ok(())
}

/// Installs defaults without replacing a file created concurrently by another caller.
pub(crate) fn create_if_missing(path: &Path, bytes: &[u8]) -> io::Result<()> {
    match stage(path, bytes)?.persist_noclobber(path) {
        Ok(_) => Ok(()),
        Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error.error),
    }
}

fn stage(path: &Path, bytes: &[u8]) -> io::Result<tempfile::NamedTempFile> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Settings path has no parent")
    })?;
    std::fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    Ok(temporary)
}
