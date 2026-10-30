//! A cooperative private-store writer lease retained by its Git subprocesses.

use crate::pull_request::control::ReviewControl;
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub(super) struct StoreLease {
    cache: PathBuf,
    file: File,
}

impl StoreLease {
    pub fn acquire(cache: &Path, control: &ReviewControl<'_>) -> Result<Self, String> {
        control.check()?;
        std::fs::create_dir_all(cache).map_err(|error| error.to_string())?;
        let cache = std::fs::canonicalize(cache).map_err(|error| error.to_string())?;
        let file = open_lock(&cache.join("review-store.lock"), true)?;
        lock(&file, control)?;
        Ok(Self { cache, file })
    }

    pub fn try_acquire(cache: &Path, control: &ReviewControl<'_>) -> Result<Option<Self>, String> {
        control.check()?;
        let cache = std::fs::canonicalize(cache).map_err(|error| error.to_string())?;
        let file = open_lock(&cache.join("review-store.lock"), false)?;
        if try_lock(&file, false)? {
            Ok(Some(Self { cache, file }))
        } else {
            Ok(None)
        }
    }

    pub fn cache(&self) -> &Path {
        &self.cache
    }
    pub fn file(&self) -> &File {
        &self.file
    }
}

#[cfg(unix)]
pub(super) fn open_lock(path: &Path, create: bool) -> Result<File, String> {
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
        return Err("Private review lease must be a regular file".into());
    }
    Ok(file)
}

#[cfg(unix)]
fn lock(file: &File, control: &ReviewControl<'_>) -> Result<(), String> {
    loop {
        control.check()?;
        if try_lock(file, false)? {
            return control.check();
        }
        std::thread::sleep(
            Duration::from_millis(10)
                .min(control.deadline.saturating_duration_since(Instant::now())),
        );
    }
}

#[cfg(unix)]
pub(super) fn try_lock(file: &File, shared: bool) -> Result<bool, String> {
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
pub(super) fn open_lock(_: &Path, _: bool) -> Result<File, String> {
    Err("Review store leases require a POSIX host".into())
}
#[cfg(not(unix))]
fn lock(_: &File, _: &ReviewControl<'_>) -> Result<(), String> {
    Err("Review store leases require a POSIX host".into())
}

#[cfg(not(unix))]
pub(super) fn try_lock(_: &File, _: bool) -> Result<bool, String> {
    Err("Review store leases require a POSIX host".into())
}
