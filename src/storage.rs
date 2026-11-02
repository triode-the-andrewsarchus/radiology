//! Shared private-settings file replacement; domain stores validate ownership first.
use std::{
    io::{self, Write},
    path::Path,
};

/// Creates the parent and replaces one file only after its new contents are synced.
/// Errors leave an existing destination intact; temporary files clean themselves up.
pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Settings path has no parent")
    })?;
    std::fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}
