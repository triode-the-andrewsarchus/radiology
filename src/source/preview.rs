//! Bounded real-source excerpts with exact index and snapshot validation.

use super::{
    loader::{check_source, SOURCE_READ_TIMEOUT},
    ReviewRevision, SourceBuffer,
};
use crate::{background::LatestRead, semantic::ScipRange, syntax::DocumentSymbol};
use std::{
    ops::Range,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Instant,
};

mod structure;

/// One numbered canonical source line in a bounded preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewLine {
    /// One-based source line number.
    pub number: usize,
    /// Bounded source-line display, with omission markers at cropped edges.
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

// Preserve the public input path while keeping query ownership independent of excerpts.
pub use super::queries::SourceQueryInputs;

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
    cache: super::queries::QueryCache<PreviewTarget, SourcePreview, 64>,
    structure: Arc<Mutex<structure::Cache>>,
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
        self.request_prepared(inputs, target, None)
    }

    /// Uses exact accepted source/outline ownership for same-file definitions.
    pub(crate) fn request_prepared(
        &mut self,
        inputs: Option<Arc<SourceQueryInputs>>,
        target: Option<PreviewTarget>,
        prepared: Option<(&SourceBuffer, &[DocumentSymbol])>,
    ) -> bool {
        let symbols = inputs
            .as_ref()
            .zip(target.as_ref())
            .and_then(|(inputs, target)| {
                prepared
                    .filter(|(owner, _)| {
                        target.kind == PreviewKind::Definition
                            && target.path == inputs.buffer.path
                            && super::buffer::SourceIdentity::new(owner).matches(&inputs.buffer)
                    })
                    .map(|(_, symbols)| symbols)
            });
        if self.cache.prepare(inputs.as_ref()) {
            self.structure = Arc::default();
        }
        let same_inputs = match (&self.inputs, &inputs) {
            (Some(old), Some(new)) => Arc::ptr_eq(old, new),
            (None, None) => true,
            _ => false,
        };
        if same_inputs
            && self.target == target
            && (symbols.is_none() || matches!(self.phase, PreviewPhase::Loaded(_)))
        {
            return false;
        }
        self.job.stop();
        self.inputs = inputs;
        self.target = target;
        self.phase = PreviewPhase::None;
        if let (Some(inputs), Some(target)) = (&self.inputs, &self.target) {
            if let Some(preview) = self.cache.get(target) {
                self.phase = PreviewPhase::Loaded(preview.clone());
                return true;
            }
            if let Some(symbols) = symbols {
                let result = prepare_with_structure(
                    inputs,
                    target,
                    &AtomicBool::new(false),
                    &self.structure,
                    Some(symbols),
                );
                self.publish(result);
            } else {
                let inputs = inputs.clone();
                let target = target.clone();
                self.phase = PreviewPhase::Loading;
                let structure = self.structure.clone();
                self.job.spawn(move |cancel| {
                    prepare_with_structure(&inputs, &target, cancel, &structure, None)
                });
            }
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
        self.publish(result);
        true
    }

    fn publish(&mut self, result: Result<SourcePreview, String>) {
        self.phase = match result {
            Ok(preview) => {
                self.cache
                    .observe_source(&preview.path, preview.revision, &preview.content_digest);
                if let Some(target) = &self.target {
                    let bytes = std::mem::size_of::<SourcePreview>()
                        + target.path.capacity()
                        + preview.path.capacity()
                        + preview.content_digest.capacity()
                        + preview.lines.capacity() * std::mem::size_of::<PreviewLine>()
                        + preview
                            .lines
                            .iter()
                            .map(|line| line.text.capacity())
                            .sum::<usize>();
                    self.cache.insert(target.clone(), preview.clone(), bytes);
                }
                PreviewPhase::Loaded(preview)
            }
            Err(error) => PreviewPhase::Unavailable(error),
        };
    }
}

#[cfg(test)]
fn prepare(
    inputs: &SourceQueryInputs,
    target: &PreviewTarget,
    cancel: &AtomicBool,
) -> Result<SourcePreview, String> {
    let inputs = Arc::new(SourceQueryInputs {
        source: inputs.source.clone(),
        buffer: inputs.buffer.clone(),
        index: inputs.index.clone(),
    });
    prepare_with_structure(&inputs, target, cancel, &Mutex::default(), None)
}

fn prepare_with_structure(
    inputs: &Arc<SourceQueryInputs>,
    target: &PreviewTarget,
    cancel: &AtomicBool,
    structure: &Mutex<structure::Cache>,
    prepared: Option<&[DocumentSymbol]>,
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
    let end = if target.kind == PreviewKind::Definition {
        if let Some(symbols) = prepared {
            structure::declaration_end(symbols, range.start)
                .map(|end| buffer.line_at_byte(end.saturating_sub(1)))
        } else {
            structure::end_line(structure, inputs, &buffer, range.start, cancel, deadline)?
        }
    } else {
        None
    };
    excerpt(&buffer, target, &range, end, cancel, deadline)
}

fn excerpt(
    buffer: &SourceBuffer,
    target: &PreviewTarget,
    range: &Range<usize>,
    definition_end: Option<usize>,
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
        PreviewKind::Definition => (
            first,
            definition_end.unwrap_or(buffer.line_count()),
            12,
            definition_end.is_none(),
        ),
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

pub(crate) fn preview_line(
    buffer: &SourceBuffer,
    number: usize,
    occurrence: &Range<usize>,
) -> PreviewLine {
    const LINE_BYTES: usize = 400;
    let line = buffer.line_byte_range(number);
    let focus = if line.contains(&occurrence.start) {
        occurrence.start
    } else {
        line.start
    };
    let (start, end) = if line.len() <= LINE_BYTES {
        (line.start, line.end)
    } else {
        let start =
            buffer.clamp_to_char_boundary(focus.saturating_sub(LINE_BYTES / 4).max(line.start));
        // Reserve both markers inside the same display budget. Canonical source
        // coordinates remain in the query result, outside this display window.
        let end = buffer.clamp_to_char_boundary((start + LINE_BYTES - 2 * "…".len()).min(line.end));
        (start, end)
    };
    let leading = if start > line.start { "…" } else { "" };
    let trailing = if end < line.end { "…" } else { "" };
    let overlap = occurrence.start.max(start)..occurrence.end.min(end);
    let emphasis = (overlap.start < overlap.end)
        .then(|| overlap.start - start + leading.len()..overlap.end - start + leading.len());
    PreviewLine {
        number,
        text: format!("{leading}{}{trailing}", &buffer.text[start..end]),
        emphasis,
        clipped: !leading.is_empty() || !trailing.is_empty(),
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
    use crate::semantic::SemanticIndex;
    use crate::semantic::{ScipDocument, ScipIndex};
    use crate::source::{loader::SourceRead, SourceLocation};

    pub(super) fn inputs(buffer: SourceBuffer) -> Arc<SourceQueryInputs> {
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

    pub(super) fn target(buffer: &SourceBuffer, name: &str, kind: PreviewKind) -> PreviewTarget {
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

    pub(super) fn settle(loader: &mut SourcePreviewLoader) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while loader.is_running() {
            loader.poll();
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    pub(super) fn assert_preview_lines<'a>(
        loader: &'a SourcePreviewLoader,
        expected: &[usize],
    ) -> &'a SourcePreview {
        let PreviewPhase::Loaded(preview) = loader.phase() else {
            panic!("{:?}", loader.phase());
        };
        assert_eq!(
            preview
                .lines
                .iter()
                .map(|line| line.number)
                .collect::<Vec<_>>(),
            expected
        );
        preview
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
    fn adjacent_cross_file_definition_previews_reuse_one_exact_parse() {
        let owner = tempfile::tempdir().unwrap();
        let text = "pub fn restore_base(image: Vec<u8>) -> PreparedBase {\n    restore(image)\n}\npub fn restore_app(image: Vec<u8>) -> PreparedApp {\n    restore(image)\n}\nfn restore(image: Vec<u8>) -> Restored {\n    let café = \"🚀\";\n    Restored\n}\n";
        std::fs::write(owner.path().join("app_image.rs"), text).unwrap();
        let mut config = inputs(SourceBuffer::from_str("app_image.rs", text));
        let captured = Arc::get_mut(&mut config).unwrap();
        captured.source = SourceRead::Checkout(owner.path().into());
        captured.buffer = Arc::new(SourceBuffer::from_str("host.rs", "// displayed host\n"));
        let notes = SourceBuffer::from_str("notes.txt", "one\ntwo\n\nthree\n");
        std::fs::write(owner.path().join("notes.txt"), &notes.text).unwrap();
        let index = Arc::make_mut(&mut captured.index);
        index.documents.insert(
            notes.path.clone(),
            ScipDocument {
                relative_path: notes.path.clone(),
                position_encoding: 2,
                ..Default::default()
            },
        );
        index
            .source_digests
            .insert(notes.path.clone(), notes.content_digest().into());
        let mut loader = SourcePreviewLoader::default();
        for (path, range, expected, last, parses) in [
            ("app_image.rs", [3, 7, 18], vec![4, 5, 6], "}", 1),
            ("app_image.rs", [6, 3, 10], vec![7, 8, 9, 10], "}", 1),
            ("notes.txt", [0, 0, 3], vec![1, 2], "two", 2),
        ] {
            loader.request(
                Some(config.clone()),
                Some(PreviewTarget {
                    path: path.into(),
                    range: ScipRange::from_packed(&range).unwrap(),
                    kind: PreviewKind::Definition,
                }),
            );
            settle(&mut loader);
            let preview = assert_preview_lines(&loader, &expected);
            assert!(!preview.truncated);
            assert_eq!(preview.lines.last().unwrap().text, last);
            assert_eq!(
                loader.structure.lock().unwrap().parses,
                parses,
                "different symbols in one exact destination must share its parsed structure"
            );
            if path == "notes.txt" {
                assert_eq!(preview.lines[0].text, "one");
            }
        }
    }

    #[test]
    fn structure_reuse_is_bounded_and_observed_byte_changes_reparse_the_destination() {
        let config = inputs(SourceBuffer::from_str("host.rs", "// captured host\n"));
        let cache = Mutex::new(structure::Cache::default());
        let buffers = (0..5)
            .map(|i| SourceBuffer::from_str(format!("file{i}.rs"), "fn work() {}\n"))
            .collect::<Vec<_>>();
        for buffer in &buffers {
            assert_eq!(
                structure::end_line(
                    &cache,
                    &config,
                    buffer,
                    3,
                    &AtomicBool::new(false),
                    Instant::now() + SOURCE_READ_TIMEOUT
                )
                .unwrap(),
                Some(1)
            );
        }
        assert_eq!(cache.lock().unwrap().parses, 5);
        structure::end_line(
            &cache,
            &config,
            &buffers[4],
            3,
            &AtomicBool::new(false),
            Instant::now() + SOURCE_READ_TIMEOUT,
        )
        .unwrap();
        assert_eq!(cache.lock().unwrap().parses, 5);
        structure::end_line(
            &cache,
            &config,
            &buffers[0],
            3,
            &AtomicBool::new(false),
            Instant::now() + SOURCE_READ_TIMEOUT,
        )
        .unwrap();
        assert_eq!(
            cache.lock().unwrap().parses,
            6,
            "a fifth destination evicts the oldest of four parsed files"
        );
        let edited = SourceBuffer::from_str("file0.rs", "fn work() {\n    other();\n}\n");
        for _ in 0..2 {
            assert_eq!(
                structure::end_line(
                    &cache,
                    &config,
                    &edited,
                    3,
                    &AtomicBool::new(false),
                    Instant::now() + SOURCE_READ_TIMEOUT
                )
                .unwrap(),
                Some(3)
            );
            assert_eq!(cache.lock().unwrap().parses, 7);
        }
    }

    #[test]
    fn structure_cache_waits_honor_cancellation_and_deadlines_without_acquiring_the_lock() {
        let config = inputs(SourceBuffer::from_str("a.rs", "fn work() {}\n"));
        let cache = Mutex::new(structure::Cache::default());
        let held = cache.lock().unwrap();
        assert!(structure::end_line(
            &cache,
            &config,
            &config.buffer,
            3,
            &AtomicBool::new(true),
            Instant::now() + SOURCE_READ_TIMEOUT
        )
        .is_err());
        assert!(structure::end_line(
            &cache,
            &config,
            &config.buffer,
            3,
            &AtomicBool::new(false),
            Instant::now() + std::time::Duration::from_millis(5)
        )
        .is_err());
        assert_eq!(held.parses, 0);
        drop(held);
        assert_eq!(
            structure::end_line(
                &cache,
                &config,
                &config.buffer,
                3,
                &AtomicBool::new(false),
                Instant::now() + SOURCE_READ_TIMEOUT
            )
            .unwrap(),
            Some(1)
        );
        assert_eq!(cache.lock().unwrap().parses, 1);
        let mut loader = SourcePreviewLoader::default();
        loader.request(Some(config.clone()), None);
        let shared = loader.structure.clone();
        let held = shared.lock().unwrap();
        loader.request(
            Some(config.clone()),
            Some(target(&config.buffer, "work", PreviewKind::Definition)),
        );
        assert!(loader.is_running());
        assert!(Arc::ptr_eq(&shared, &loader.structure));
        loader.request(Some(config), None);
        assert!(!loader.is_running());
        assert_eq!(loader.phase(), &PreviewPhase::None);
        assert_eq!(held.parses, 0);
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
        assert!(line.text.len() <= 400);
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
    fn swift_unicode_window_preserves_match_navigation_and_explicit_fallback_preview() {
        use crate::semantic::{
            index::SemanticSymbolID,
            reference_results::{query, ReferenceLimits},
        };
        let text = format!(
            "fn a() {{}}\nlet v = {} target(1); // tail\nfn b() {{}}\n",
            "é🚀x".repeat(30_000)
        );
        let mut config = inputs(SourceBuffer::from_str("gen.rs", &text));
        let position = target(&config.buffer, "target", PreviewKind::Reference);
        let id = SemanticSymbolID::new("rust-analyzer cargo sample 1 target().", "gen.rs");
        Arc::make_mut(&mut Arc::get_mut(&mut config).unwrap().index)
            .references
            .insert(id.clone(), vec![("gen.rs".into(), position.range.clone())]);
        let cancel = AtomicBool::new(false);
        let results = query(&config, &id, ReferenceLimits::default(), &cancel).unwrap();
        let hit = &results.hits[0];
        assert_eq!(hit.source_line.number, 2);
        assert_eq!(&config.buffer.text[hit.byte_range.clone()], "target");
        assert!(hit.source_line.text.starts_with('…'));
        assert!(hit.source_line.text.len() <= 403);
        assert_eq!(
            &hit.source_line.text[hit.source_line.emphasis.clone().unwrap()],
            "target"
        );
        let context = prepare(&config, &position, &cancel).unwrap();
        assert_eq!(context.lines[1], hit.source_line);
        assert!(context
            .lines
            .iter()
            .all(|line| line.text.len() <= 403 && !line.text.contains('\u{fffd}')));
        let range = config
            .index
            .byte_range(&config.buffer, &position.range)
            .unwrap();
        let mut definition = position;
        definition.kind = PreviewKind::Definition;
        // Swift explicitly supplies no declaration end to its preview constructor.
        let preview = excerpt(
            &config.buffer,
            &definition,
            &range,
            None,
            &cancel,
            Instant::now() + SOURCE_READ_TIMEOUT,
        )
        .unwrap();
        assert_eq!(preview.lines.len(), 2);
        assert!(preview.lines[0].text.len() <= 403 && !preview.lines[0].text.contains('\u{fffd}'));
        assert_eq!(config.buffer.text, text);
    }

    #[test]
    fn window_markers_identify_only_omitted_edges_and_leave_short_lines_whole() {
        for (prefix, suffix, leading, trailing) in [
            (350, 0, false, false),
            (0, 800, false, true),
            (800, 0, true, false),
            (800, 800, true, true),
        ] {
            let text = format!("{}café{}", "x".repeat(prefix), "y".repeat(suffix));
            let buffer = SourceBuffer::from_str("line.txt", &text);
            let range = prefix..prefix + "café".len();
            let line = preview_line(&buffer, 1, &range);
            assert_eq!(line.text.starts_with('…'), leading);
            assert_eq!(line.text.ends_with('…'), trailing);
            assert_eq!(line.clipped, leading || trailing);
            assert!(line.text.len() <= 400);
            assert_eq!(&line.text[line.emphasis.clone().unwrap()], "café");
            if !line.clipped {
                assert_eq!(line.text, text);
            }
        }
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
        assert!(loader.request(Some(inputs.clone()), Some(first.clone())));
        assert!(loader.request(Some(inputs.clone()), Some(second.clone())));
        settle(&mut loader);
        let PreviewPhase::Loaded(preview) = loader.phase() else {
            panic!("{:?}", loader.phase());
        };
        assert!(preview.lines[0].text.contains("two"));
        assert!(!loader.request(Some(inputs.clone()), Some(second.clone())));
        assert!(!loader.is_running());
        let expected_second = loader.phase().clone();
        loader.request(Some(inputs.clone()), Some(first.clone()));
        assert!(
            loader.is_running(),
            "the retired first request must not have populated the cache"
        );
        settle(&mut loader);
        let expected_first = loader.phase().clone();
        loader.request(Some(inputs.clone()), None);
        loader.request(Some(inputs.clone()), Some(second));
        assert!(!loader.is_running());
        assert_eq!(loader.phase(), &expected_second);
        loader.request(Some(inputs.clone()), Some(first.clone()));
        assert!(
            !loader.is_running(),
            "returning to a prepared definition restarted a worker"
        );
        assert_eq!(loader.phase(), &expected_first);
        loader.request(None, None);
        assert_eq!(loader.phase(), &PreviewPhase::None);
        loader.request(Some(inputs), Some(first));
        assert!(
            loader.is_running(),
            "reset must discard previously cached excerpts"
        );
        settle(&mut loader);
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

#[cfg(test)]
mod memory_tests {
    use super::tests::{assert_preview_lines, inputs, settle, target};
    use super::*;
    use crate::{source::loader::SourceRead, syntax::DocumentSyntax};

    #[test]
    fn prepared_definition_promotes_pending_work_without_io_or_mutex_and_rejects_bad_coordinates() {
        let mut config = inputs(SourceBuffer::from_str(
            "a.rs",
            "fn target() {\n    work();\n}\nfn next() {}\n",
        ));
        let owner = tempfile::tempdir().unwrap();
        let unavailable = owner.path().join("not-created");
        assert!(!unavailable.exists());
        Arc::get_mut(&mut config).unwrap().source = SourceRead::Checkout(unavailable);
        let symbols = DocumentSyntax::analyze(&config.buffer).symbols;
        let position = target(&config.buffer, "target", PreviewKind::Definition);
        let mut loader = SourcePreviewLoader::default();
        loader.request(Some(config.clone()), None);
        let structure = loader.structure.clone();
        let held = structure.lock().unwrap();
        loader.request(Some(config.clone()), Some(position.clone()));
        assert!(loader.is_running());
        loader.request_prepared(
            Some(config.clone()),
            Some(position.clone()),
            Some((&config.buffer, &symbols)),
        );
        assert_preview_lines(&loader, &[1, 2, 3]);
        assert!(!loader.is_running());
        assert_eq!(held.parses, 0);
        drop(held);
        assert!(!loader.poll());
        let mut invalid = position.clone();
        invalid.range.start_col = 999;
        invalid.range.end_col = 1000;
        loader.request_prepared(
            Some(config.clone()),
            Some(invalid),
            Some((&config.buffer, &symbols)),
        );
        assert!(matches!(loader.phase(), PreviewPhase::Unavailable(_)));
        assert!(!loader.is_running());
        let mut cross_file = position;
        cross_file.path = "other.rs".into();
        loader.request_prepared(
            Some(config.clone()),
            Some(cross_file),
            Some((&config.buffer, &symbols)),
        );
        assert!(
            loader.is_running(),
            "cross-file targets must stay on workers even with current-file symbols"
        );
        settle(&mut loader);
        assert!(matches!(loader.phase(), PreviewPhase::Unavailable(_)));
    }

    #[test]
    fn stale_prepared_owner_uses_current_source_and_before_never_publishes_semantic_previews() {
        let old = inputs(SourceBuffer::from_str(
            "a.rs",
            "fn target() {\n    work();\n}\nfn next() {}\n",
        ));
        let old_symbols = DocumentSyntax::analyze(&old.buffer).symbols;
        let current = inputs(SourceBuffer::from_str(
            "a.rs",
            "fn target() {\n    first();\n    second();\n    third();\n}\nfn next() {}\n",
        ));
        let position = target(&current.buffer, "target", PreviewKind::Definition);
        let mut loader = SourcePreviewLoader::default();
        loader.request_prepared(
            Some(current.clone()),
            Some(position.clone()),
            Some((&old.buffer, &old_symbols)),
        );
        assert!(
            loader.is_running(),
            "stale prepared facts must not take the immediate path"
        );
        settle(&mut loader);
        assert_preview_lines(&loader, &[1, 2, 3, 4, 5]);
        assert_eq!(loader.structure.lock().unwrap().parses, 1);
        let mut before = inputs(SourceBuffer::from_str("a.rs", &current.buffer.text));
        Arc::make_mut(&mut Arc::get_mut(&mut before).unwrap().buffer).revision =
            Some(ReviewRevision::Before);
        let symbols = DocumentSyntax::analyze(&before.buffer).symbols;
        loader.request_prepared(
            Some(before.clone()),
            Some(position),
            Some((&before.buffer, &symbols)),
        );
        assert!(
            matches!(loader.phase(), PreviewPhase::Unavailable(message) if message.contains("Before"))
        );
        assert!(!loader.is_running());
    }
}
