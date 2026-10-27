//! Multi-occurrence token highlighting and pin group management.

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
    /// Byte ranges in the file corresponding to occurrences.
    pub ranges: Vec<Range<usize>>,
    /// Theme color assigned to this group.
    pub color: OccurrenceColor,
}

/// Stores and manages up to four persistent pinned occurrence groups and ephemeral highlights.
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
}

impl OccurrencePinStore {
    /// Creates an empty [`OccurrencePinStore`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

    /// Toggles pinning for a symbol, adding it with the next free color or removing it.
    pub fn toggle_pin(&mut self, symbol: &str, ranges: Vec<Range<usize>>) {
        if let Some(pos) = self.groups.iter().position(|g| g.symbol_name == symbol) {
            self.groups.remove(pos);
        } else {
            let used_colors: Vec<OccurrenceColor> = self.groups.iter().map(|g| g.color).collect();
            let next_color = OccurrenceColor::ALL
                .iter()
                .copied()
                .find(|c| !used_colors.contains(c))
                .unwrap_or(OccurrenceColor::Teal);

            if self.groups.len() >= 4 {
                self.groups.remove(0);
            }

            self.groups.push(OccurrencePinGroup {
                symbol_name: symbol.to_string(),
                ranges,
                color: next_color,
            });
        }
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
        self.groups.clear();
        self.ephemeral = None;
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
}
