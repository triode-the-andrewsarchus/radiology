//! One cancellation owner and total deadline for metadata and snapshot preparation.

use crate::repository::git::GitRead;
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

pub(crate) const PREPARATION_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Copy)]
pub(crate) struct ReviewControl<'a> {
    pub cancel: &'a AtomicBool,
    pub deadline: Instant,
    pub git: &'a Path,
    pub gh: &'a Path,
    pub lease: Option<&'a std::fs::File>,
}

impl<'a> ReviewControl<'a> {
    pub fn new(cancel: &'a AtomicBool) -> Self {
        Self {
            cancel,
            deadline: Instant::now() + PREPARATION_TIMEOUT,
            git: Path::new("git"),
            gh: Path::new("gh"),
            lease: None,
        }
    }

    pub fn git<'r>(&'r self, root: &'r Path) -> GitRead<'r> {
        GitRead {
            root,
            executable: self.git,
            cancel: self.cancel,
            deadline: self.deadline,
            lease: self.lease,
        }
    }

    pub fn check(&self) -> Result<(), String> {
        if self.cancel.load(Ordering::Relaxed) {
            Err("Review preparation cancelled".into())
        } else if Instant::now() >= self.deadline {
            Err("Review preparation timed out".into())
        } else {
            Ok(())
        }
    }
}
