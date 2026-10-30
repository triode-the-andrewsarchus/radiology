//! Owned off-thread source reads, structural analysis, and syntax highlighting.

use super::{ColumnEncoding, SourceBuffer, SourceLocation};
use crate::{
    bookmarks::bookmark::SourceBookmark,
    pull_request::diff::FileLineChanges,
    pull_request::review::ReviewSnapshot,
    syntax::{
        highlighter::{HighlightedSpan, SyntaxHighlighter},
        DocumentSyntax,
    },
};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
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
    /// Returns path, Git, file-size, UTF-8 or I/O validation failures, including
    /// a read exceeding its 30-second deadline.
    pub fn read(&self, location: &SourceLocation) -> Result<Arc<SourceBuffer>, String> {
        self.read_cancellable(
            location,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(30),
        )
    }

    pub(crate) fn read_cancellable(
        &self,
        location: &SourceLocation,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<Arc<SourceBuffer>, String> {
        check_source(cancel, deadline)?;
        if !SourceLocation::is_repository_path(&location.path) {
            return Err("Invalid repository-relative source path".into());
        }
        let buffer = match self {
            Self::Checkout(root) => {
                SourceBuffer::read(&root.join(&location.path), &location.path, None)
                    .map(Arc::new)
                    .map_err(|error| error.to_string())
            }
            Self::Review(snapshot) => snapshot
                .read_cancellable(
                    &location.path,
                    location.revision.unwrap_or(super::ReviewRevision::After),
                    cancel,
                    deadline,
                )
                .map(Arc::new),
            Self::Buffer(buffer)
                if buffer.path == location.path && buffer.revision == location.revision =>
            {
                Ok(buffer.clone())
            }
            Self::Buffer(_) => Err("Source snapshot does not match the requested document".into()),
        }?;
        check_source(cancel, deadline)?;
        Ok(buffer)
    }

    fn changes(
        &self,
        buffer: &SourceBuffer,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<Option<FileLineChanges>, String> {
        match self {
            Self::Review(snapshot) => Ok(snapshot.diffs.get(&buffer.path).cloned()),
            Self::Checkout(root) => crate::repository::git::GitRead {
                root,
                executable: std::path::Path::new("git"),
                cancel,
                deadline,
            }
            .checkout_source_lines(buffer),
            Self::Buffer(_) => Ok(None),
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
    /// Retained exact-source native captures for recoloring without another parse.
    pub native_highlights: Option<Arc<crate::syntax::native_highlights::NativeHighlights>>,
}

/// Source and derived facts published atomically by one completed request.
pub struct PreparedSource {
    /// Source shared with local find, rendering and later analysis jobs.
    pub buffer: Arc<SourceBuffer>,
    /// Structural facts from one parse; absent when recoloring an existing buffer.
    pub syntax: Option<DocumentSyntax>,
    /// Initial body/review fold choices prepared from these exact source facts.
    pub initial_folds: Option<std::collections::HashSet<std::ops::Range<usize>>>,
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
        self.job
            .spawn(move |cancel| prepare(request, &highlighter, dark_mode, cancel));
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

fn check_source(cancel: &AtomicBool, deadline: Instant) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("Source request cancelled".into())
    } else if Instant::now() >= deadline {
        Err("Source request timed out".into())
    } else {
        Ok(())
    }
}

fn prepare(
    request: SourceLoadRequest,
    highlighter: &SyntaxHighlighter,
    dark_mode: bool,
    cancel: &AtomicBool,
) -> Result<PreparedSource, String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    let check = || check_source(cancel, deadline);
    let cancelled = || check().is_err();
    check()?;
    let buffer = request
        .source
        .read_cancellable(&request.location, cancel, deadline)?;
    check()?;
    let destination = request.target.resolve(&buffer)?;
    let syntax = (!matches!(request.source, SourceRead::Buffer(_)))
        .then(|| DocumentSyntax::analyze(&buffer));
    check()?;
    let native = syntax
        .as_ref()
        .map(|syntax| syntax.native_highlights.as_ref())
        .or(request.native_highlights.as_deref())
        .filter(|native| native.matches(&buffer));
    let highlights = if let Some(native) = native {
        native.recolor(&buffer, dark_mode, cancelled)
    } else {
        highlighter.highlight_document_cancellable(
            highlighter.find_syntax_for_path(&buffer.path),
            &buffer.text,
            dark_mode,
            cancelled,
        )
    };
    check()?;
    let highlights = highlights.ok_or("Source request cancelled")?;
    let changes = request.source.changes(&buffer, cancel, deadline)?;
    check()?;
    let initial_folds = syntax
        .as_ref()
        .filter(|syntax| syntax.folds_supported)
        .map(|syntax| {
            let mut folds = crate::syntax::FoldStore::new();
            folds.apply_initial_defaults(&buffer, &syntax.folds, changes.as_ref());
            folds.folded_ranges
        });
    check()?;
    Ok(PreparedSource {
        buffer,
        syntax,
        initial_folds,
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

    fn request(root: &std::path::Path, path: &str, target: SourceTarget) -> SourceLoadRequest {
        SourceLoadRequest {
            source: SourceRead::Checkout(root.to_path_buf()),
            location: SourceLocation::new(path, 0),
            target,
            native_highlights: None,
        }
    }

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
    fn native_theme_recolor_reuses_exact_captures_and_rejects_foreign_snapshots() {
        let buffer = Arc::new(SourceBuffer::from_str(
            "missing.kt",
            "fun run() { val text = \"café 😀\" }\r\n",
        ));
        let facts = DocumentSyntax::analyze(&buffer).native_highlights;
        assert!(facts.matches(&buffer));
        let highlighter = Arc::new(SyntaxHighlighter::new());
        let request = |native_highlights| SourceLoadRequest {
            source: SourceRead::Buffer(buffer.clone()),
            location: buffer.location_at_byte(0),
            target: SourceTarget::Byte(0),
            native_highlights,
        };
        let mut loader = SourceLoader::default();
        loader.start(request(Some(facts.clone())), highlighter.clone(), true);
        let dark = finish(&mut loader).unwrap();
        assert!(dark.syntax.is_none());
        assert!(Arc::ptr_eq(&dark.buffer, &buffer));
        loader.start(request(Some(facts.clone())), highlighter.clone(), false);
        let light = finish(&mut loader).unwrap();
        assert!(light.syntax.is_none());
        assert_ne!(light.highlights, dark.highlights);
        assert!(facts.matches(&buffer));
        let foreign = DocumentSyntax::analyze(&SourceBuffer::from_str("other.kt", &buffer.text))
            .native_highlights;
        loader.start(request(Some(foreign)), highlighter, true);
        let fallback = finish(&mut loader).unwrap();
        assert!(fallback.syntax.is_none());
        assert_ne!(
            fallback.highlights, dark.highlights,
            "foreign captures must not recolor this buffer"
        );
    }

    #[test]
    fn replacements_publish_only_current_source_and_report_read_failures() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("old.rs"), "fn old() {}\n".repeat(1000)).unwrap();
        std::fs::write(dir.path().join("new.rs"), "fn café() {\r\n 1;\r\n}\r\n").unwrap();
        let highlighter = Arc::new(SyntaxHighlighter::new());
        let mut loader = SourceLoader::default();
        loader.start(
            request(dir.path(), "old.rs", SourceTarget::Byte(0)),
            highlighter.clone(),
            true,
        );
        loader.start(
            request(
                dir.path(),
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
        assert_eq!(
            prepared.initial_folds,
            Some(std::collections::HashSet::from([prepared
                .syntax
                .as_ref()
                .unwrap()
                .folds
                .iter()
                .find(|fold| fold.kind == crate::syntax::FoldKind::Callable)
                .unwrap()
                .byte_range
                .clone()]))
        );
        assert!(!prepared.highlights.is_empty());
        assert!(!loader.is_running());
        assert!(loader.poll().is_none());
        std::fs::remove_file(dir.path().join("new.rs")).unwrap();
        loader.start(
            SourceLoadRequest {
                source: SourceRead::Buffer(prepared.buffer.clone()),
                location: prepared.buffer.location_at_byte(0),
                target: SourceTarget::Byte(0),
                native_highlights: None,
            },
            highlighter.clone(),
            false,
        );
        let recolored = finish(&mut loader).unwrap();
        assert!(Arc::ptr_eq(&prepared.buffer, &recolored.buffer));
        assert!(recolored.syntax.is_none());
        assert!(!recolored.dark_mode);
        loader.start(
            request(dir.path(), "missing.rs", SourceTarget::Byte(0)),
            highlighter,
            true,
        );
        assert!(finish(&mut loader).is_err());
    }
    #[test]
    fn checkout_hunks_use_literal_paths_and_cover_untracked_source_on_first_open() {
        use crate::repository::test_support::{git, repository};
        let dir = tempfile::tempdir().unwrap();
        repository(dir.path());
        let path = "literal[ab] café.rs";
        std::fs::write(dir.path().join(path), "old\n").unwrap();
        std::fs::write(dir.path().join("literala café.rs"), "old\n").unwrap();
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "-qm", "initial"]);
        std::fs::write(dir.path().join(path), "new\nsecond\n").unwrap();
        std::fs::write(dir.path().join("literala café.rs"), "other\n".repeat(9)).unwrap();
        let highlighter = Arc::new(SyntaxHighlighter::new());
        let mut loader = SourceLoader::default();
        loader.start(
            request(dir.path(), path, SourceTarget::Byte(0)),
            highlighter.clone(),
            true,
        );
        let prepared = finish(&mut loader).unwrap();
        assert_eq!(prepared.changes.as_ref().unwrap().total_additions(), 2);
        assert_eq!(prepared.changes.as_ref().unwrap().total_deletions(), 1);
        let untracked = "new[1].rs";
        std::fs::write(dir.path().join(untracked), "fn new_file() {}\n").unwrap();
        loader.start(
            request(dir.path(), untracked, SourceTarget::Byte(0)),
            highlighter,
            true,
        );
        let prepared = finish(&mut loader).unwrap();
        assert_eq!(prepared.changes.as_ref().unwrap().total_additions(), 1);
        assert_eq!(prepared.changes.as_ref().unwrap().total_deletions(), 0);
    }

    #[test]
    fn cancellable_reads_reject_retired_or_expired_requests_before_publication() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("source.rs"), "fn sample() {}\n").unwrap();
        let storage = SourceRead::Checkout(dir.path().into());
        let location = SourceLocation::new("source.rs", 0);
        let error = storage
            .read_cancellable(
                &location,
                &AtomicBool::new(true),
                Instant::now() + Duration::from_secs(1),
            )
            .unwrap_err();
        assert!(error.contains("cancelled"));
        let error = storage
            .read_cancellable(&location, &AtomicBool::new(false), Instant::now())
            .unwrap_err();
        assert!(error.contains("timed out"));
    }
}
