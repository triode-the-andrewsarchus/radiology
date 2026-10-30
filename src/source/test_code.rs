//! Test-code projection, independent of manual folds and source mutation.

use super::SourceBuffer;
use crate::{
    repository::classifier::TestPathClassifier,
    syntax::{TestDetection, TestRegion},
};
use reader_text::SourceFragment;
use std::{collections::HashSet, ops::Range};

/// Presentation of detected tests for the currently loaded exact document.
#[derive(Default)]
pub struct TestCodePresentation {
    detection: TestDetection,
    revealed: HashSet<usize>,
    hidden: Vec<Range<usize>>,
    enabled: bool,
    test_document: bool,
    generation: u64,
}

impl TestCodePresentation {
    /// Replaces syntax for a loaded document and retires its previous reveal state.
    pub fn load(&mut self, buffer: &SourceBuffer, detection: TestDetection, enabled: bool) {
        self.test_document =
            detection.is_test_only_document || TestPathClassifier::is_test_path(&buffer.path);
        self.detection = detection;
        self.enabled = enabled;
        self.revealed.clear();
        self.refresh(buffer);
    }

    /// Updates the preference while preserving unrelated manual fold state.
    pub fn set_enabled(&mut self, buffer: &SourceBuffer, enabled: bool) {
        if self.enabled == enabled {
            return;
        }
        self.enabled = enabled;
        if enabled {
            self.revealed.clear();
        }
        self.refresh(buffer);
    }

    /// Version used by a viewport to invalidate only its test projection.
    #[must_use]
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Region header controls are available only in mixed source with Hide Tests on.
    #[must_use]
    pub fn header(&self, line: usize) -> Option<&TestRegion> {
        if !self.enabled || self.test_document {
            return None;
        }
        self.detection
            .regions
            .binary_search_by_key(&line, |region| region.header_line)
            .ok()
            .map(|index| &self.detection.regions[index])
    }

    /// Returns whether one region is currently hidden.
    #[must_use]
    pub fn is_hidden(&self, id: usize) -> bool {
        self.enabled && !self.test_document && !self.revealed.contains(&id)
    }

    /// Toggles one canonical region, leaving manual folds untouched.
    pub fn toggle(&mut self, buffer: &SourceBuffer, id: usize) {
        if !self
            .detection
            .regions
            .iter()
            .any(|region| region.byte_range.start == id)
        {
            return;
        }
        if !self.revealed.remove(&id) {
            self.revealed.insert(id);
        }
        self.refresh(buffer);
    }

    /// Reveals any region containing an explicit navigation destination.
    pub fn reveal_byte(&mut self, buffer: &SourceBuffer, byte: usize) {
        self.reveal_range(buffer, &(byte..byte.saturating_add(1)));
    }

    /// Reveals every region intersecting an exact local-find selection.
    pub fn reveal_range(&mut self, buffer: &SourceBuffer, range: &Range<usize>) {
        let mut changed = false;
        for region in &self.detection.regions {
            if region.byte_range.start < range.end && range.start < region.byte_range.end {
                changed |= self.revealed.insert(region.byte_range.start);
            }
        }
        if changed {
            self.refresh(buffer);
        }
    }

    /// Omits hidden declarations from the visible outline, retaining full syntax.
    #[must_use]
    pub fn hides_declaration(&self, range: &Range<usize>) -> bool {
        self.enabled
            && !self.test_document
            && self.detection.regions.iter().any(|region| {
                self.is_hidden(region.byte_range.start)
                    && region.byte_range.start <= range.start
                    && range.end <= region.byte_range.end
            })
    }

    /// Quiet reader status for explicit test files and hidden mixed-source regions.
    #[must_use]
    pub fn status(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }
        if self.test_document {
            return Some("Test file".into());
        }
        let count = self
            .detection
            .regions
            .iter()
            .filter(|region| self.is_hidden(region.byte_range.start))
            .count();
        (count > 0).then(|| {
            format!(
                "{count} test region{} hidden",
                if count == 1 { "" } else { "s" }
            )
        })
    }

    /// Returns disjoint canonical intervals hidden by test presentation.
    #[must_use]
    pub fn hidden_ranges(&self) -> &[Range<usize>] {
        &self.hidden
    }

    /// Removes only hidden byte intervals, preserving visible tails on shared lines.
    pub fn project_into(&self, fragment: &SourceFragment, output: &mut Vec<SourceFragment>) {
        if fragment.bytes.is_empty() {
            if !self
                .hidden
                .iter()
                .any(|hidden| hidden.contains(&fragment.bytes.start))
            {
                output.push(fragment.clone());
            }
            return;
        }
        let mut start = fragment.bytes.start;
        let first = self.hidden.partition_point(|hidden| hidden.end <= start);
        for hidden in &self.hidden[first..] {
            if hidden.start >= fragment.bytes.end {
                break;
            }
            if hidden.start > start {
                output.push(SourceFragment {
                    bytes: start..hidden.start.min(fragment.bytes.end),
                    ..fragment.clone()
                });
            }
            start = start.max(hidden.end);
            if start >= fragment.bytes.end {
                break;
            }
        }
        if start < fragment.bytes.end {
            output.push(SourceFragment {
                bytes: start..fragment.bytes.end,
                ..fragment.clone()
            });
        }
    }

    fn refresh(&mut self, buffer: &SourceBuffer) {
        self.hidden = self
            .detection
            .regions
            .iter()
            .filter(|region| self.is_hidden(region.byte_range.start))
            .filter_map(|region| {
                let header_end = buffer
                    .line_byte_range(region.header_line)
                    .end
                    .min(region.byte_range.end);
                (header_end < region.byte_range.end).then_some(header_end..region.byte_range.end)
            })
            .collect();
        self.generation = self.generation.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::TestRegionDetector;

    fn visible(presentation: &TestCodePresentation, buffer: &SourceBuffer) -> Vec<SourceFragment> {
        let mut output = Vec::new();
        for fragment in reader_text::source_fragments(&buffer.text, 4096) {
            presentation.project_into(&fragment, &mut output);
        }
        output
    }

    #[test]
    fn projection_preserves_headers_production_tails_and_navigation_reveal() {
        let text = "fn production() {}\r\n#[cfg(test)]\r\nmod checks {\r\n fn check() {}\r\n} fn tail() {}\r\n";
        let buffer = SourceBuffer::from_str("src/lib.rs", text);
        let detection = TestRegionDetector::detect(&buffer);
        let id = detection.regions[0].byte_range.start;
        let mut state = TestCodePresentation::default();
        state.load(&buffer, detection, true);
        let parts = visible(&state, &buffer);
        let joined: String = parts.iter().map(|part| &text[part.bytes.clone()]).collect();
        assert!(joined.contains("#[cfg(test)]"));
        assert!(joined.contains("fn production"));
        assert!(joined.contains("fn tail"));
        assert!(!joined.contains("fn check"));
        assert_eq!(state.status().as_deref(), Some("1 test region hidden"));
        state.reveal_byte(&buffer, text.find("fn check").unwrap());
        assert!(!state.is_hidden(id));
        assert!(state.status().is_none());
        assert_eq!(visible(&state, &buffer).len(), buffer.line_count());
        state.set_enabled(&buffer, false);
        state.set_enabled(&buffer, true);
        assert!(state.is_hidden(id));
        state.toggle(&buffer, id);
        assert!(!state.is_hidden(id));
    }

    #[test]
    fn explicitly_opened_test_documents_remain_readable() {
        for (path, text) in [
            ("tests/check.rs", "#[test]\nfn checks() {}\n"),
            ("src/lib.rs", "#![cfg(test)]\nfn checks() {}\n"),
        ] {
            let buffer = SourceBuffer::from_str(path, text);
            let mut state = TestCodePresentation::default();
            state.load(&buffer, TestRegionDetector::detect(&buffer), true);
            assert_eq!(state.status().as_deref(), Some("Test file"));
            assert_eq!(
                visible(&state, &buffer),
                reader_text::source_fragments(text, 4096)
            );
        }
    }
}
