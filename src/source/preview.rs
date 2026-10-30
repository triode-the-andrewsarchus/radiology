//! Bounded real-source excerpts with exact index and snapshot validation.

use super::{
    loader::{check_source, SourceRead, SOURCE_READ_TIMEOUT},
    ReviewRevision, SourceBuffer, SourceLocation,
};
use crate::{
    background::LatestRead,
    semantic::{ScipRange, SemanticIndex},
    syntax::{DocumentSymbol, SymbolExtractor},
};
use std::{
    ops::Range,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

/// One numbered canonical source line in a bounded preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewLine {
    /// One-based source line number.
    pub number: usize,
    /// Original source text, without newline bytes, bounded at UTF-8 boundaries.
    pub text: String,
    /// Indexed occurrence bytes relative to the displayed text, when visible.
    pub emphasis: Option<Range<usize>>,
    /// Whether the displayed line omits a prefix or suffix.
    pub clipped: bool,
}

/// A source excerpt, never a substitute for an indexed navigation target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePreview {
    /// Repository-relative source path.
    pub path: String,
    /// Revision of the exact source storage that supplied these lines.
    pub revision: Option<ReviewRevision>,
    /// SHA-256 identity used to invalidate excerpts when source changes.
    pub content_digest: String,
    /// Bounded numbered lines from the exact destination source.
    pub lines: Vec<PreviewLine>,
    /// Whether more source follows the excerpt's line limit.
    pub truncated: bool,
}

/// Current asynchronous excerpt presentation.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum PreviewPhase {
    /// There is no eligible source destination.
    #[default]
    None,
    /// A replaceable worker is preparing this excerpt.
    Loading,
    /// Source and coordinates were validated successfully.
    Loaded(SourcePreview),
    /// The destination could not be read or validated.
    Unavailable(String),
}

/// Captured storage, displayed bytes and index for exact-source semantic queries.
pub struct SourceQueryInputs {
    /// Checkout or pinned review storage selected before the query starts.
    pub source: SourceRead,
    /// Immutable displayed bytes, which take precedence over storage reads.
    pub buffer: Arc<SourceBuffer>,
    /// Immutable index whose coordinates and source provenance must be validated.
    pub index: Arc<SemanticIndex>,
}

impl SourceQueryInputs {
    pub(crate) fn read(
        &self,
        path: &str,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<Arc<SourceBuffer>, String> {
        check_source(cancel, deadline)?;
        if !SourceLocation::is_repository_path(path) {
            return Err("Invalid repository-relative source path".into());
        }
        if path == self.buffer.path {
            Ok(self.buffer.clone())
        } else {
            self.source.read_cancellable(
                &SourceLocation::with_revision(path, 0, self.buffer.revision),
                cancel,
                deadline,
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PreviewKind {
    Definition,
    Reference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreviewTarget {
    pub path: String,
    pub range: ScipRange,
    pub kind: PreviewKind,
}

/// Owns a replaceable source excerpt and rejects retired worker publications.
#[derive(Default)]
pub struct SourcePreviewLoader {
    inputs: Option<Arc<SourceQueryInputs>>,
    target: Option<PreviewTarget>,
    job: LatestRead<Result<SourcePreview, String>>,
    phase: PreviewPhase,
}

impl SourcePreviewLoader {
    /// Returns the current source excerpt or preparation status.
    #[must_use]
    pub fn phase(&self) -> &PreviewPhase {
        &self.phase
    }

    pub(crate) fn request(
        &mut self,
        inputs: Option<Arc<SourceQueryInputs>>,
        target: Option<PreviewTarget>,
    ) -> bool {
        let same_inputs = match (&self.inputs, &inputs) {
            (Some(old), Some(new)) => Arc::ptr_eq(old, new),
            (None, None) => true,
            _ => false,
        };
        if same_inputs && self.target == target {
            return false;
        }
        self.job.stop();
        self.inputs = inputs;
        self.target = target;
        self.phase = PreviewPhase::None;
        if let (Some(inputs), Some(target)) = (&self.inputs, &self.target) {
            let inputs = inputs.clone();
            let target = target.clone();
            self.phase = PreviewPhase::Loading;
            self.job
                .spawn(move |cancel| prepare(&inputs, &target, cancel));
        }
        true
    }

    pub(crate) fn is_running(&self) -> bool {
        self.job.is_running()
    }

    pub(crate) fn poll(&mut self) -> bool {
        let result = match self.job.poll() {
            Ok(None) => return false,
            Ok(Some(result)) => result,
            Err(_) => Err("Source preview worker stopped unexpectedly.".into()),
        };
        self.phase = match result {
            Ok(preview) => PreviewPhase::Loaded(preview),
            Err(error) => PreviewPhase::Unavailable(error),
        };
        true
    }
}

fn prepare(
    inputs: &SourceQueryInputs,
    target: &PreviewTarget,
    cancel: &AtomicBool,
) -> Result<SourcePreview, String> {
    check_cancel(cancel)?;
    if inputs.buffer.revision == Some(ReviewRevision::Before) {
        return Err("After semantic previews are unavailable in Before source.".into());
    }
    let deadline = Instant::now() + SOURCE_READ_TIMEOUT;
    let buffer = inputs.read(&target.path, cancel, deadline)?;
    check_cancel(cancel)?;
    let range = inputs
        .index
        .byte_range(&buffer, &target.range)
        .filter(|range| !range.is_empty())
        .ok_or("The index no longer matches this source. Reindex to refresh its preview.")?;
    excerpt(&buffer, target, &range, cancel, deadline)
}

fn definition_end_line(
    buffer: &SourceBuffer,
    byte: usize,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<Option<usize>, String> {
    let symbols =
        SymbolExtractor::extract_cancellable(buffer, || check_source(cancel, deadline).is_err());
    check_source(cancel, deadline)?;
    Ok(declaration_end(&symbols?, byte).map(|end| buffer.line_at_byte(end.saturating_sub(1))))
}

fn excerpt(
    buffer: &SourceBuffer,
    target: &PreviewTarget,
    range: &Range<usize>,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<SourcePreview, String> {
    let first = buffer.line_at_byte(range.start);
    let (start, end, limit, stop_at_blank) = match target.kind {
        PreviewKind::Reference => (
            first.saturating_sub(2).max(1),
            (first + 2).min(buffer.line_count()),
            5,
            false,
        ),
        PreviewKind::Definition => {
            let end = definition_end_line(buffer, range.start, cancel, deadline)?;
            (first, end.unwrap_or(buffer.line_count()), 12, end.is_none())
        }
    };
    let mut lines = Vec::new();
    let mut number = start;
    while number <= end && lines.len() < limit {
        check_source(cancel, deadline)?;
        if stop_at_blank && number > first && buffer.line_text(number).trim().is_empty() {
            break;
        }
        lines.push(preview_line(buffer, number, range));
        number += 1;
    }
    let truncated = lines.len() == limit
        && number <= end
        && (!stop_at_blank || !buffer.line_text(number).trim().is_empty());
    Ok(SourcePreview {
        path: target.path.clone(),
        revision: buffer.revision,
        content_digest: buffer.content_digest().to_owned(),
        lines,
        truncated,
    })
}

fn declaration_end(symbols: &[DocumentSymbol], byte: usize) -> Option<usize> {
    symbols
        .iter()
        .filter_map(|symbol| symbol.find_innermost(byte))
        .filter(|symbol| symbol.selection_range.contains(&byte))
        .min_by_key(|symbol| (symbol.selection_range.start != byte, symbol.range.len()))
        .map(|symbol| symbol.range.end)
}

pub(crate) fn preview_line(
    buffer: &SourceBuffer,
    number: usize,
    occurrence: &Range<usize>,
) -> PreviewLine {
    const LINE_BYTES: usize = 2048;
    let line = buffer.line_byte_range(number);
    let focus = if line.contains(&occurrence.start) {
        occurrence.start
    } else {
        line.start
    };
    let start = buffer.clamp_to_char_boundary(focus.saturating_sub(LINE_BYTES / 4).max(line.start));
    let end = buffer.clamp_to_char_boundary(start.saturating_add(LINE_BYTES).min(line.end));
    let overlap = occurrence.start.max(start)..occurrence.end.min(end);
    let emphasis =
        (overlap.start < overlap.end).then(|| overlap.start - start..overlap.end - start);

    PreviewLine {
        number,
        text: buffer.text[start..end].to_owned(),
        emphasis,
        clipped: start > line.start || end < line.end,
    }
}

fn check_cancel(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("Source preview cancelled.".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{ScipDocument, ScipIndex};

    fn inputs(buffer: SourceBuffer) -> Arc<SourceQueryInputs> {
        let mut index = SemanticIndex::from_scip(ScipIndex {
            documents: vec![ScipDocument {
                relative_path: buffer.path.clone(),
                position_encoding: 2,
                ..Default::default()
            }],
            ..Default::default()
        });
        index
            .source_digests
            .insert(buffer.path.clone(), buffer.content_digest().into());
        let buffer = Arc::new(buffer);
        Arc::new(SourceQueryInputs {
            source: SourceRead::Buffer(buffer.clone()),
            buffer,
            index: Arc::new(index),
        })
    }

    fn target(buffer: &SourceBuffer, name: &str, kind: PreviewKind) -> PreviewTarget {
        let byte = buffer.text.find(name).unwrap();
        let line = buffer.line_at_byte(byte);
        let column = buffer.text[buffer.line_starts[line - 1]..byte]
            .encode_utf16()
            .count() as i32;
        PreviewTarget {
            path: buffer.path.clone(),
            range: ScipRange::from_packed(&[
                line as i32 - 1,
                column,
                column + name.encode_utf16().count() as i32,
            ])
            .unwrap(),
            kind,
        }
    }

    fn settle(loader: &mut SourcePreviewLoader) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while loader.is_running() {
            loader.poll();
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    #[test]
    fn definitions_stop_at_nested_declaration_end_and_keep_internal_blank_lines() {
        let inputs = inputs(SourceBuffer::from_str("a.rs", "mod outer {\n mod inner {\n  fn target() {\n\n   work();\n  }\n  fn neighbor() {}\n }\n}\n"));
        let target = target(&inputs.buffer, "target", PreviewKind::Definition);
        let preview = prepare(&inputs, &target, &AtomicBool::new(false)).unwrap();
        assert_eq!(
            preview
                .lines
                .iter()
                .map(|line| line.number)
                .collect::<Vec<_>>(),
            [3, 4, 5, 6]
        );
        assert!(preview.lines[1].text.is_empty());
        assert!(!preview.truncated);
        assert!(preview
            .lines
            .iter()
            .all(|line| !line.text.contains("neighbor")));
    }

    #[test]
    fn definition_line_limits_and_unsupported_blank_line_fallback_are_explicit() {
        let source = format!(
            "fn target() {{\n{}}}\nfn neighbor() {{}}",
            " work();\n".repeat(20)
        );
        let parsed = inputs(SourceBuffer::from_str("a.rs", &source));
        let preview = prepare(
            &parsed,
            &target(&parsed.buffer, "target", PreviewKind::Definition),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(preview.lines.len(), 12);
        assert!(preview.truncated);
        let unknown = inputs(SourceBuffer::from_str(
            "a.txt",
            "target\ncontent\n\nneighbor",
        ));
        let preview = prepare(
            &unknown,
            &target(&unknown.buffer, "target", PreviewKind::Definition),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(preview.lines.len(), 2);
        assert!(!preview.truncated);
    }

    #[test]
    fn utf16_context_crops_long_lines_around_exact_utf8_match_without_crlf_bytes() {
        let text = format!(
            "before\r\n{} café(); {}\r\nafter\r\n",
            "😀".repeat(800),
            "x".repeat(3000)
        );
        let inputs = inputs(SourceBuffer::from_str("a.rs", &text));
        let position = target(&inputs.buffer, "café", PreviewKind::Reference);
        let preview = prepare(&inputs, &position, &AtomicBool::new(false)).unwrap();
        let line = &preview.lines[1];
        assert!(line.clipped);
        assert!(line.text.len() <= 2048);
        assert_eq!(&line.text[line.emphasis.clone().unwrap()], "café");
        assert!(preview.lines.iter().all(|line| !line.text.contains('\r')));
        let mut invalid = position;
        invalid.range.start_col = 1;
        invalid.range.end_col = 2;
        assert!(
            prepare(&inputs, &invalid, &AtomicBool::new(false)).is_err(),
            "UTF-16 surrogate halves must not map"
        );
    }

    #[test]
    fn stale_before_bad_ranges_and_cancellation_never_produce_source_previews() {
        let mut inputs = inputs(SourceBuffer::from_str("a.rs", "fn target() {}"));
        let position = target(&inputs.buffer, "target", PreviewKind::Definition);
        assert!(prepare(&inputs, &position, &AtomicBool::new(true)).is_err());
        let index = Arc::make_mut(&mut Arc::get_mut(&mut inputs).unwrap().index);
        index
            .source_digests
            .insert("a.rs".into(), "other bytes".into());
        assert!(prepare(&inputs, &position, &AtomicBool::new(false))
            .unwrap_err()
            .contains("no longer matches"));
        Arc::make_mut(&mut Arc::get_mut(&mut inputs).unwrap().buffer).revision =
            Some(ReviewRevision::Before);
        assert!(prepare(&inputs, &position, &AtomicBool::new(false))
            .unwrap_err()
            .contains("Before"));
    }

    #[test]
    fn replaced_requests_cannot_publish_and_same_target_does_not_restart() {
        let inputs = inputs(SourceBuffer::from_str("a.rs", "fn one() {}\nfn two() {}"));
        let first = target(&inputs.buffer, "one", PreviewKind::Definition);
        let second = target(&inputs.buffer, "two", PreviewKind::Definition);
        let mut loader = SourcePreviewLoader::default();
        assert!(loader.request(Some(inputs.clone()), Some(first)));
        assert!(loader.request(Some(inputs.clone()), Some(second.clone())));
        settle(&mut loader);
        let PreviewPhase::Loaded(preview) = loader.phase() else {
            panic!("{:?}", loader.phase());
        };
        assert!(preview.lines[0].text.contains("two"));
        assert!(!loader.request(Some(inputs), Some(second)));
        assert!(!loader.is_running());
        loader.request(None, None);
        assert_eq!(loader.phase(), &PreviewPhase::None);
    }

    #[test]
    fn pinned_pr_preview_reads_git_objects_and_primed_source_wins_over_checkout() {
        let (_temp, root, snapshot) = crate::pull_request::review::tests::fixture_with_sources(
            "fn old() {}\n",
            "fn target() {\n head_only();\n}\n",
        );
        let source = SourceRead::Review(Arc::new(snapshot));
        let buffer = source
            .read(&SourceLocation::with_revision(
                "a file.rs",
                0,
                Some(ReviewRevision::After),
            ))
            .unwrap();
        let position = target(&buffer, "target", PreviewKind::Definition);
        let mut config = inputs((*buffer).clone());
        let config_mut = Arc::get_mut(&mut config).unwrap();
        config_mut.source = source.clone();
        config_mut.buffer = source
            .read(&SourceLocation::with_revision(
                "added.rs",
                0,
                Some(ReviewRevision::After),
            ))
            .unwrap();
        std::fs::write(root.join("a file.rs"), "fn checkout_only() {}\n").unwrap();
        let preview = prepare(&config, &position, &AtomicBool::new(false)).unwrap();
        assert!(preview.lines[1].text.contains("head_only"));
        let mut same_file = inputs((*buffer).clone());
        Arc::get_mut(&mut same_file).unwrap().source = SourceRead::Checkout(root);
        assert_eq!(
            prepare(&same_file, &position, &AtomicBool::new(false)).unwrap(),
            preview
        );
    }
    #[test]
    fn displayed_query_bytes_obey_retirement_and_deadlines_without_an_io_fallback() {
        let inputs = inputs(SourceBuffer::from_str("shown.rs", "fn shown() {}\n"));
        let future = Instant::now() + SOURCE_READ_TIMEOUT;
        assert!(inputs
            .read("shown.rs", &AtomicBool::new(true), future)
            .unwrap_err()
            .contains("cancelled"));
        assert!(inputs
            .read("shown.rs", &AtomicBool::new(false), Instant::now())
            .unwrap_err()
            .contains("timed out"));
        let buffer = inputs
            .read("shown.rs", &AtomicBool::new(false), future)
            .unwrap();
        assert!(Arc::ptr_eq(&buffer, &inputs.buffer));
    }
}

#[cfg(all(test, target_os = "linux"))]
mod cancellation_tests;
