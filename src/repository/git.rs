//! Shared bounded Git reads for discovery and checkout refresh.

use crate::process::{self, OutputStream, ProcessCommand};
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

pub(crate) struct GitRead<'a> {
    pub root: &'a Path,
    pub executable: &'a Path,
    pub cancel: &'a AtomicBool,
    pub deadline: Instant,
}

impl GitRead<'_> {
    pub fn check(&self) -> Result<(), String> {
        if self.cancel.load(Ordering::Relaxed) {
            Err("Repository read cancelled".into())
        } else if Instant::now() >= self.deadline {
            Err("Repository read timed out".into())
        } else {
            Ok(())
        }
    }

    fn run(&self, arguments: &[&str]) -> Result<(bool, Vec<u8>), crate::process::RunFailure> {
        let command = ProcessCommand {
            executable: self.executable.into(),
            directory: self.root.into(),
            arguments: ["-c", "color.ui=false", "-c", "core.quotepath=false"]
                .into_iter()
                .chain(arguments.iter().copied())
                .map(Into::into)
                .collect(),
            environment: [("GIT_OPTIONAL_LOCKS".into(), "0".into())].into(),
            resources: Vec::new(),
        };
        let mut bytes = Vec::new();
        let result = process::run_observed(
            &command,
            self.cancel,
            self.deadline.saturating_duration_since(Instant::now()),
            |stream, chunk| {
                if stream == OutputStream::Stdout {
                    if bytes.len().saturating_add(chunk.len()) > 16 * 1024 * 1024 {
                        return Err(std::io::Error::other("Git output exceeds 16 MiB"));
                    }
                    bytes.extend_from_slice(chunk);
                }
                Ok(())
            },
        )?;
        Ok((result.succeeded(), bytes))
    }

    pub fn optional(&self, arguments: &[&str]) -> Result<Option<Vec<u8>>, String> {
        match self.run(arguments) {
            Ok((true, bytes)) => Ok(Some(bytes)),
            Ok((false, _)) => Ok(None),
            Err(error)
                if matches!(
                    error.kind,
                    crate::process::RunFailureKind::Spawn {
                        kind: std::io::ErrorKind::NotFound,
                        ..
                    }
                ) =>
            {
                Ok(None)
            }
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn required(&self, arguments: &[&str]) -> Result<Vec<u8>, String> {
        self.optional(arguments)?
            .ok_or_else(|| format!("Git {} failed", arguments[0]))
    }
}
