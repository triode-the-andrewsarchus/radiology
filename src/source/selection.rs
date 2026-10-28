//! Multi-occurrence token highlighting and pin group management.

use super::SourceBuffer;
use std::collections::HashMap;
use std::ops::Range;

/// The four distinct occurrence pin colors supported in Radiology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OccurrenceColor {
    /// Teal occurrence highlight.
    Teal,
    /// Amber occurrence highlight.
    Amber,
    /// Violet occurrence highlight.
    Violet,
    /// Rose occurrence highlight.
    Rose,
}

impl OccurrenceColor {
    /// Array of all four occurrence colors in rotation order.
    pub const ALL: [Self; 4] = [Self::Teal, Self::Amber, Self::Violet, Self::Rose];

    /// Returns the human-readable color name.
    #[must_use]
    pub fn title(&self) -> &'static str {
        match self {
            Self::Teal => "Teal",
            Self::Amber => "Amber",
            Self::Violet => "Violet",
            Self::Rose => "Rose",
        }
    }
}

/// A pinned group of occurrences for a symbol across the current file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OccurrencePinGroup {
    /// Identifier or symbol name being highlighted.
    pub symbol_name: String,
    /// Source spelling displayed by the group controls.
    pub label: String,
    /// Byte ranges in the file corresponding to occurrences.
    pub ranges: Vec<Range<usize>>,
    /// Theme color assigned to this group.
    pub color: OccurrenceColor,
}

/// Stores unlimited session pin groups with stable colors and exact document identity.
///
/// # Examples
///
/// ```rust
/// use radiology::source::selection::{OccurrenceColor, OccurrencePinStore};
///
/// let mut store = OccurrencePinStore::new();
/// store.toggle_pin("my_var", vec![10..16, 40..46]);
///
/// assert!(store.is_pinned("my_var"));
/// assert_eq!(store.groups[0].color, OccurrenceColor::Teal);
/// ```
#[derive(Debug, Clone, Default)]
pub struct OccurrencePinStore {
    /// Active locked occurrence groups.
    pub groups: Vec<OccurrencePinGroup>,
    /// Ephemeral highlight for the currently hovered or single-clicked symbol.
    pub ephemeral: Option<(String, Vec<Range<usize>>)>,
    documents: HashMap<String, DocumentPins>,
    active_document: Option<(String, String)>,
    next_color_slot: usize,
}

#[derive(Debug, Clone)]
struct DocumentPins {
    digest: String,
    groups: Vec<OccurrencePinGroup>,
}

impl OccurrencePinStore {
    /// Creates an empty [`OccurrencePinStore`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Activates a document, restoring pins only for identical bytes and revision.
    ///
    /// Call this only when a source read successfully publishes. Ephemeral highlights
    /// do not cross documents. A changed digest discards the older document's pins.
    pub fn activate(&mut self, buffer: &SourceBuffer) {
        let identity = (buffer.position_key(), buffer.content_digest().to_owned());
        if self.active_document.as_ref() == Some(&identity) {
            return;
        }
        if let Some((key, digest)) = self.active_document.take() {
            let groups = std::mem::take(&mut self.groups);
            if !groups.is_empty() {
                self.documents.insert(key, DocumentPins { digest, groups });
            }
        } else {
            self.groups.clear();
        }
        self.groups = self
            .documents
            .remove(&identity.0)
            .filter(|document| document.digest == identity.1)
            .map_or_else(Vec::new, |document| document.groups);
        self.ephemeral = None;
        self.active_document = Some(identity);
    }

    /// Returns `true` if no symbols are pinned and no ephemeral highlight is active.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty() && self.ephemeral.is_none()
    }

    /// Returns `true` if the given symbol name is already pinned.
    #[must_use]
    pub fn is_pinned(&self, symbol: &str) -> bool {
        self.groups.iter().any(|g| g.symbol_name == symbol)
    }

    /// Toggles a group using its identity as its display label.
    pub fn toggle_pin(&mut self, symbol: &str, ranges: Vec<Range<usize>>) {
        let clicked = ranges.first().map_or(0, |range| range.start);
        self.toggle_labeled_pin(symbol, symbol, ranges, clicked);
    }

    /// Toggles a source group without evicting or recoloring existing pins.
    ///
    /// Equivalent canonical ranges reconcile a lexical group that later gains a
    /// semantic identity. Colors advance for each new group and cycle freely.
    pub fn toggle_labeled_pin(
        &mut self,
        symbol: &str,
        label: &str,
        mut ranges: Vec<Range<usize>>,
        clicked: usize,
    ) {
        ranges.retain(|range| range.start < range.end);
        ranges.sort_unstable_by_key(|range| (range.start, range.end));
        ranges.dedup();
        let existing = self.groups.iter().position(|group| {
            group.symbol_name == symbol
                || (group.ranges == ranges && ranges.iter().any(|range| range.contains(&clicked)))
        });
        if let Some(position) = existing {
            self.groups.remove(position);
        } else if !ranges.is_empty() {
            let color = OccurrenceColor::ALL[self.next_color_slot % OccurrenceColor::ALL.len()];
            self.next_color_slot = self.next_color_slot.wrapping_add(1);
            self.groups.push(OccurrencePinGroup {
                symbol_name: symbol.to_owned(),
                label: label.to_owned(),
                ranges,
                color,
            });
        }
    }

    /// Removes a group by its stable identity; returns whether it was present.
    pub fn remove(&mut self, symbol: &str) -> bool {
        let length = self.groups.len();
        self.groups.retain(|group| group.symbol_name != symbol);
        self.groups.len() != length
    }

    /// Clears pins in the active document, retaining other documents and color slots.
    pub fn clear_current(&mut self) {
        self.groups.clear();
        self.ephemeral = None;
    }

    /// Unpins any active occurrence group containing the given byte offset.
    pub fn unpin_group_containing(&mut self, byte_offset: usize) -> bool {
        if let Some(idx) = self.groups.iter().position(|g| {
            g.ranges
                .iter()
                .any(|r| r.contains(&byte_offset) || r.start == byte_offset)
        }) {
            self.groups.remove(idx);
            true
        } else {
            false
        }
    }

    /// Clears all pinned groups and ephemeral highlights.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_occurrence_pins() {
        let mut store = OccurrencePinStore::new();
        assert!(!store.is_pinned("foo"));

        store.toggle_pin("foo", vec![10..13, 50..53]);
        assert!(store.is_pinned("foo"));
        assert_eq!(store.groups.len(), 1);
        assert_eq!(store.groups[0].color, OccurrenceColor::Teal);

        store.toggle_pin("bar", vec![Range { start: 20, end: 23 }]);
        assert_eq!(store.groups.len(), 2);
        assert_eq!(store.groups[1].color, OccurrenceColor::Amber);

        store.toggle_pin("foo", vec![]);
        assert!(!store.is_pinned("foo"));
        assert_eq!(store.groups.len(), 1);

        assert!(store.unpin_group_containing(21));
        assert_eq!(store.groups.len(), 0);
    }
    #[test]
    fn pin_groups_have_no_eviction_limit_and_removal_does_not_recolor_survivors() {
        let mut store = OccurrencePinStore::new();
        for number in 0..12 {
            store.toggle_pin(
                &format!("symbol-{number}"),
                std::iter::once(number * 10..number * 10 + 3).collect(),
            );
        }
        assert_eq!(store.groups.len(), 12);
        let colors: Vec<_> = store
            .groups
            .iter()
            .map(|group| (group.symbol_name.clone(), group.color))
            .collect();
        assert!(store.remove("symbol-3"));
        assert!(!store.remove("absent"));
        for group in &store.groups {
            assert!(colors.contains(&(group.symbol_name.clone(), group.color)));
        }
        store.toggle_pin("new", std::iter::once(200..203).collect());
        assert_eq!(store.groups.last().unwrap().color, OccurrenceColor::Teal);
        assert!(store.is_pinned("symbol-0"));
    }

    #[test]
    fn document_restoration_requires_identical_content_and_revision_and_clears_ephemeral_state() {
        use crate::source::location::ReviewRevision;
        let a = SourceBuffer::from_str("a.rs", "same source");
        let b = SourceBuffer::from_str("b.rs", "same source");
        let mut store = OccurrencePinStore::new();
        store.activate(&a);
        store.toggle_pin("a", std::iter::once(0..4).collect());
        let color = store.groups[0].color;
        store.ephemeral = Some(("temporary".into(), std::iter::once(5..11).collect()));
        store.activate(&b);
        assert!(store.is_empty());
        store.toggle_pin("b", std::iter::once(5..11).collect());
        store.activate(&a);
        assert!(store.is_pinned("a"));
        assert_eq!(store.groups[0].color, color);
        let before = SourceBuffer::from_bytes(
            "a.rs",
            b"same source".to_vec(),
            Some(ReviewRevision::Before),
        )
        .unwrap();
        store.activate(&before);
        assert!(store.is_empty());
        store.toggle_pin("before", std::iter::once(0..4).collect());
        store.activate(&a);
        assert!(store.is_pinned("a"));
        let changed = SourceBuffer::from_str("a.rs", "new! source"); // Same byte length.
        store.activate(&changed);
        assert!(store.is_empty());
        store.activate(&a);
        assert!(store.is_empty()); // Returning to old bytes does not resurrect discarded pins.
        store.activate(&b);
        assert!(store.is_pinned("b"));
        store.clear_current();
        store.activate(&before);
        assert!(store.is_pinned("before"));
        store.clear();
        store.activate(&b);
        assert!(store.is_empty());
    }

    #[test]
    fn canonical_ranges_reconcile_lexical_and_semantic_groups_without_duplicate_highlights() {
        let mut store = OccurrencePinStore::new();
        store.toggle_labeled_pin("word@0", "word", vec![10..14, 1..5, 10..14], 2);
        assert_eq!(store.groups[0].label, "word");
        assert_eq!(store.groups[0].ranges, vec![1..5, 10..14]);
        store.toggle_labeled_pin("scip-package word.", "word", vec![1..5, 10..14], 11);
        assert!(store.is_empty());
        store.toggle_labeled_pin("empty", "empty", std::iter::once(4..4).collect(), 4);
        assert!(store.is_empty());
    }
}
