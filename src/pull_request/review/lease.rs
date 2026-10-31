//! A cooperative private-store writer lease retained by its Git subprocesses.

use crate::pull_request::control::ReviewControl;
use std::{
    fs::File,
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

#[cfg(not(unix))]
fn lock(_: &File, _: &ReviewControl<'_>) -> Result<(), String> {
    Err("Review store leases require a POSIX host".into())
}

pub(super) use crate::process::lease::{open_lock, try_lock};
