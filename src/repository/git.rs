//! Shared bounded Git reads for discovery, source loading and checkout refresh.

use crate::{
    process::{self, OutputStream, ProcessCommand},
    pull_request::diff::{DiffHunk, FileLineChanges, UnifiedDiffParser},
    source::SourceBuffer,
};
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
    pub fn head(&self) -> Result<Option<String>, String> {
        self.optional(&["rev-parse", "--verify", "HEAD"])?
            .map(|bytes| {
                String::from_utf8(bytes)
                    .map(|head| head.trim_end().to_owned())
                    .map_err(|error| error.to_string())
            })
            .transpose()
    }

    pub fn checkout_source_lines(
        &self,
        buffer: &SourceBuffer,
    ) -> Result<Option<FileLineChanges>, String> {
        if self
            .optional(&["rev-parse", "--is-inside-work-tree"])?
            .as_deref()
            != Some(b"true\n")
        {
            return Ok(None);
        }
        let head = self.head()?;
        let lines = self.source_lines(buffer, head.as_deref())?;
        if self.head()? != head {
            return Err("Repository HEAD changed during source loading; retry.".into());
        }
        self.check()?;
        Ok(lines)
    }

    pub fn source_lines(
        &self,
        buffer: &SourceBuffer,
        base: Option<&str>,
    ) -> Result<Option<FileLineChanges>, String> {
        self.check()?;
        if buffer.revision.is_some() {
            return Ok(None);
        }
        if !same_source(self.root, buffer) {
            return Ok(None);
        }
        let base = if let Some(base) = base {
            let object = format!("{base}:{}", buffer.path);
            self.optional(&["cat-file", "-e", &object])?.map(|_| base)
        } else {
            None
        };
        let lines = if let Some(base) = base {
            let patch = self.required(&[
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "-U0",
                base,
                "--",
                &buffer.path,
            ])?;
            UnifiedDiffParser::parse(&String::from_utf8_lossy(&patch))
        } else {
            FileLineChanges {
                hunks: vec![DiffHunk {
                    before_start: 0,
                    before_count: 0,
                    after_start: 1,
                    after_count: buffer.text.lines().count(),
                }],
                is_binary: false,
            }
        };
        self.check()?;
        Ok(same_source(self.root, buffer).then_some(lines))
    }
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
            environment: [
                ("GIT_OPTIONAL_LOCKS".into(), "0".into()),
                ("GIT_LITERAL_PATHSPECS".into(), "1".into()),
            ]
            .into(),
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

fn same_source(root: &Path, buffer: &SourceBuffer) -> bool {
    SourceBuffer::read(&root.join(&buffer.path), &buffer.path, None)
        .is_ok_and(|disk| disk.content_digest() == buffer.content_digest())
}

#[cfg(all(test, unix))]
mod tests;
