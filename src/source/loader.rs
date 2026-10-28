//! Owned off-thread source reads, structural analysis, and syntax highlighting.

use super::{ColumnEncoding, SourceBuffer, SourceLocation};
use crate::{
    bookmarks::bookmark::SourceBookmark,
    pull_request::diff::{FileLineChanges, UnifiedDiffParser},
    pull_request::review::ReviewSnapshot,
    syntax::{
        highlighter::{HighlightedSpan, SyntaxHighlighter},
        DocumentSyntax,
    },
};
use std::{
    path::PathBuf,
    sync::{atomic::Ordering, Arc},
};

/// Immutable source storage selected before a request leaves the reader thread.
#[derive(Clone)]
pub enum SourceRead {
    /// Read from an attached checkout without changing it.
    Checkout(PathBuf),
    /// Read exact pinned Git objects independent of checkout changes.
    Review(Arc<ReviewSnapshot>),
    /// Recolor an already loaded immutable source without re-reading its storage.
    Buffer(Arc<SourceBuffer>),
}

impl SourceRead {
    /// Reads canonical repository-relative source from the selected storage.
    ///
    /// # Errors
    /// Returns path, Git, file-size, UTF-8 or I/O validation failures.
    pub fn read(&self, location: &SourceLocation) -> Result<Arc<SourceBuffer>, String> {
        if !SourceLocation::is_repository_path(&location.path) {
            return Err("Invalid repository-relative source path".into());
        }
        match self {
            Self::Checkout(root) => {
                SourceBuffer::read(&root.join(&location.path), &location.path, None)
                    .map(Arc::new)
                    .map_err(|error| error.to_string())
            }
            Self::Review(snapshot) => snapshot
                .read(
                    &location.path,
                    location.revision.unwrap_or(super::ReviewRevision::After),
                )
                .map(Arc::new),
            Self::Buffer(buffer)
                if buffer.path == location.path && buffer.revision == location.revision =>
            {
                Ok(buffer.clone())
            }
            Self::Buffer(_) => Err("Source snapshot does not match the requested document".into()),
        }
    }

    fn changes(&self, location: &SourceLocation) -> Option<FileLineChanges> {
        match self {
            Self::Review(snapshot) => snapshot.diffs.get(&location.path).cloned(),
            Self::Checkout(root) => {
                let output = std::process::Command::new("git")
                    .args(["diff", "--no-ext-diff", "-U0", "HEAD", "--", &location.path])
                    .current_dir(root)
                    .output()
                    .ok()?;
                output
                    .status
                    .success()
                    .then(|| UnifiedDiffParser::parse(&String::from_utf8_lossy(&output.stdout)))
            }
            Self::Buffer(_) => None,
        }
    }
}

/// Destination resolved only after the exact source bytes have been read.
#[derive(Clone)]
pub enum SourceTarget {
    /// Original UTF-8 byte offset, clamped to a valid scalar boundary.
    Byte(usize),
    /// One-based source line and zero-based Unicode scalar column.
    LineColumn { line: usize, column: usize },
    /// Strict index coordinates in their declared encoding.
    Encoded {
        line: usize,
        column: usize,
        encoding: ColumnEncoding,
        /// Exact source identity when supplied by a generated index.
        content_digest: Option<String>,
    },
    /// Persisted anchor relocation against the newly read exact source.
    Bookmark(SourceBookmark),
}

impl SourceTarget {
    /// Whether these bytes satisfy the destination's optional provenance.
    pub(crate) fn accepts_source_identity(&self, buffer: &SourceBuffer) -> bool {
        match self {
            Self::Encoded {
                content_digest: Some(expected),
                ..
            } => expected == buffer.content_digest(),
            _ => true,
        }
    }

    pub(crate) fn resolve(&self, buffer: &SourceBuffer) -> Result<SourceLocation, String> {
        if !self.accepts_source_identity(buffer) {
            return Err(
                "Indexed source bytes changed; regenerate the index before navigating".into(),
            );
        }
        let byte = match self {
            Self::Byte(byte) => buffer.clamp_to_char_boundary((*byte).min(buffer.byte_len())),
            Self::LineColumn { line, column } => buffer.byte_offset(*line, *column),
            Self::Encoded {
                line,
                column,
                encoding,
                ..
            } => buffer
                .byte_offset_encoded(*line, *column, *encoding)
                .ok_or("Indexed source position is not valid in this snapshot")?,
            Self::Bookmark(bookmark) => return bookmark.resolve(buffer).map_err(str::to_string),
        };
        Ok(buffer.location_at_byte(byte))
    }
}

/// A complete immutable source request, independent of UI focus/history state.
#[derive(Clone)]
pub struct SourceLoadRequest {
    /// Storage snapshot captured by the host.
    pub source: SourceRead,
    /// Validated canonical path and review revision.
    pub location: SourceLocation,
    /// Destination to resolve against those exact bytes.
    pub target: SourceTarget,
}

/// Source and derived facts published atomically by one completed request.
pub struct PreparedSource {
    /// Source shared with local find, rendering and later analysis jobs.
    pub buffer: Arc<SourceBuffer>,
    /// Structural facts from one parse; absent when recoloring an existing buffer.
    pub syntax: Option<DocumentSyntax>,
    /// File-specific diff captured by the same storage request.
    pub changes: Option<FileLineChanges>,
    /// Syntax colors for the captured appearance.
    pub highlights: Vec<Vec<HighlightedSpan>>,
    /// Exact resolved destination, including revision and relocated bookmarks.
    pub destination: SourceLocation,
    /// Appearance used for highlights, so the host can reject stale colors.
    pub dark_mode: bool,
}

/// Latest-request ownership prevents retired jobs from publishing into another view.
#[derive(Default)]
pub struct SourceLoader {
    job: crate::background::LatestRead<Result<PreparedSource, String>>,
}

impl SourceLoader {
    /// Starts a bounded-worker request and retires the previous publication channel.
    pub fn start(
        &mut self,
        request: SourceLoadRequest,
        highlighter: Arc<SyntaxHighlighter>,
        dark_mode: bool,
    ) {
        self.job.spawn(move |cancel| {
            prepare(request, &highlighter, dark_mode, || {
                cancel.load(Ordering::Relaxed)
            })
        });
    }

    /// Retires queued work/publication while preserving the host's current source.
    pub fn stop(&mut self) {
        self.job.stop();
    }

    /// Returns whether the host should continue polling for a current completion.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.job.is_running()
    }

    /// Takes one completion; channel failures are explicit recoverable errors.
    pub fn poll(&mut self) -> Option<Result<PreparedSource, String>> {
        match self.job.poll() {
            Ok(result) => result,
            Err(_) => Some(Err("Source worker stopped unexpectedly".into())),
        }
    }
}

fn prepare(
    request: SourceLoadRequest,
    highlighter: &SyntaxHighlighter,
    dark_mode: bool,
    cancelled: impl Fn() -> bool,
) -> Result<PreparedSource, String> {
    let check = || {
        if cancelled() {
            Err("Source request cancelled".to_string())
        } else {
            Ok(())
        }
    };
    check()?;
    let buffer = request.source.read(&request.location)?;
    check()?;
    let destination = request.target.resolve(&buffer)?;
    let syntax = (!matches!(request.source, SourceRead::Buffer(_)))
        .then(|| DocumentSyntax::analyze(&buffer));
    check()?;
    let highlights = highlighter
        .highlight_document_cancellable(
            highlighter.find_syntax_for_path(&buffer.path),
            &buffer.text,
            dark_mode,
            &cancelled,
        )
        .ok_or("Source request cancelled")?;
    check()?;
    let changes = request.source.changes(&request.location);
    check()?;
    Ok(PreparedSource {
        buffer,
        syntax,
        changes,
        highlights,
        destination,
        dark_mode,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn finish(loader: &mut SourceLoader) -> Result<PreparedSource, String> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(result) = loader.poll() {
                return result;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn replacements_publish_only_current_source_and_report_read_failures() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("old.rs"), "fn old() {}\n".repeat(1000)).unwrap();
        std::fs::write(dir.path().join("new.rs"), "fn café() {\r\n 1;\r\n}\r\n").unwrap();
        let request = |path: &str, target| SourceLoadRequest {
            source: SourceRead::Checkout(dir.path().to_path_buf()),
            location: SourceLocation::new(path, 0),
            target,
        };
        let highlighter = Arc::new(SyntaxHighlighter::new());
        let mut loader = SourceLoader::default();
        loader.start(
            request("old.rs", SourceTarget::Byte(0)),
            highlighter.clone(),
            true,
        );
        loader.start(
            request(
                "new.rs",
                SourceTarget::Encoded {
                    line: 1,
                    column: 7,
                    encoding: ColumnEncoding::Utf16,
                    content_digest: None,
                },
            ),
            highlighter.clone(),
            true,
        );
        let prepared = finish(&mut loader).unwrap();
        assert_eq!(prepared.buffer.path, "new.rs");
        assert_eq!(prepared.destination.byte_offset, "fn café".len());
        assert_eq!(prepared.syntax.as_ref().unwrap().symbols[0].name, "café");
        assert!(!prepared.highlights.is_empty());
        assert!(!loader.is_running());
        assert!(loader.poll().is_none());
        std::fs::remove_file(dir.path().join("new.rs")).unwrap();
        loader.start(
            SourceLoadRequest {
                source: SourceRead::Buffer(prepared.buffer.clone()),
                location: prepared.buffer.location_at_byte(0),
                target: SourceTarget::Byte(0),
            },
            highlighter.clone(),
            false,
        );
        let recolored = finish(&mut loader).unwrap();
        assert!(Arc::ptr_eq(&prepared.buffer, &recolored.buffer));
        assert!(recolored.syntax.is_none());
        assert!(!recolored.dark_mode);
        loader.start(
            request("missing.rs", SourceTarget::Byte(0)),
            highlighter,
            true,
        );
        assert!(finish(&mut loader).is_err());
    }
}
