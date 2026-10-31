//! Non-following advisory file leases shared by cooperating background writers.

use std::{
    fs::{File, OpenOptions},
    path::Path,
};

#[cfg(unix)]
pub(crate) fn open_lock(path: &Path, create: bool) -> Result<File, String> {
    use std::os::unix::fs::OpenOptionsExt;
    let file = OpenOptions::new()
        .create(create)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(path)
        .map_err(|error| error.to_string())?;
    if !file
        .metadata()
        .map_err(|error| error.to_string())?
        .is_file()
    {
        return Err("Lease must be a regular file".into());
    }
    Ok(file)
}

#[cfg(unix)]
pub(crate) fn try_lock(file: &File, shared: bool) -> Result<bool, String> {
    use std::os::fd::AsRawFd;
    let operation = if shared { libc::LOCK_SH } else { libc::LOCK_EX };
    // SAFETY: the borrowed file owns this live descriptor throughout the call.
    if unsafe { libc::flock(file.as_raw_fd(), operation | libc::LOCK_NB) } == 0 {
        return Ok(true);
    }
    let error = std::io::Error::last_os_error();
    if matches!(
        error.kind(),
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
    ) {
        Ok(false)
    } else {
        Err(error.to_string())
    }
}

#[cfg(not(unix))]
pub(crate) fn open_lock(_: &Path, _: bool) -> Result<File, String> {
    Err("File leases require a POSIX host".into())
}
#[cfg(not(unix))]
pub(crate) fn try_lock(_: &File, _: bool) -> Result<bool, String> {
    Err("File leases require a POSIX host".into())
}
