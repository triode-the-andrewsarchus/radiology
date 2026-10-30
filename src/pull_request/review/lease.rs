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
        let file = open_lock(&cache.join("review-store.lock"))?;
        lock(&file, control)?;
        Ok(Self { cache, file })
    }

    pub fn cache(&self) -> &Path {
        &self.cache
    }
    pub fn file(&self) -> &File {
        &self.file
    }
}

#[cfg(unix)]
fn open_lock(path: &Path) -> Result<File, String> {
    use std::os::unix::fs::OpenOptionsExt;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
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
    use std::os::fd::AsRawFd;
    loop {
        control.check()?;
        // SAFETY: the borrowed file owns this live descriptor for the call.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
            return control.check();
        }
        let error = std::io::Error::last_os_error();
        if !matches!(
            error.kind(),
            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
        ) {
            return Err(error.to_string());
        }
        std::thread::sleep(
            Duration::from_millis(10)
                .min(control.deadline.saturating_duration_since(Instant::now())),
        );
    }
}

#[cfg(not(unix))]
fn open_lock(_: &Path) -> Result<File, String> {
    Err("Review store leases require a POSIX host".into())
}
#[cfg(not(unix))]
fn lock(_: &File, _: &ReviewControl<'_>) -> Result<(), String> {
    Err("Review store leases require a POSIX host".into())
}
