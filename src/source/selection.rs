use std::ops::Range;

/// The four distinct occurrence pin colors supported in Radiology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OccurrenceColor {
    Teal,
    Amber,
    Violet,
    Rose,
}

impl OccurrenceColor {
    pub const ALL: [Self; 4] = [Self::Teal, Self::Amber, Self::Violet, Self::Rose];

    pub fn title(&self) -> &'static str {
        match self {
            Self::Teal => "Teal",
            Self::Amber => "Amber",
            Self::Violet => "Violet",
            Self::Rose => "Rose",
        }
    }
}

/// A pinned set of occurrences for a symbol/identifier across the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OccurrencePinGroup {
    pub symbol_name: String,
    pub ranges: Vec<Range<usize>>,
    pub color: OccurrenceColor,
}

/// Store for managing pinned occurrence groups and ephemeral single-click highlights.
#[derive(Debug, Clone, Default)]
pub struct OccurrencePinStore {
    pub groups: Vec<OccurrencePinGroup>,
    /// Ephemeral highlight for the currently hovered or single-clicked symbol
    pub ephemeral: Option<(String, Vec<Range<usize>>)>,
}

impl OccurrencePinStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_pinned(&self, symbol: &str) -> bool {
        self.groups.iter().any(|g| g.symbol_name == symbol)
    }

    pub fn toggle_pin(&mut self, symbol: &str, ranges: Vec<Range<usize>>) {
        if let Some(pos) = self.groups.iter().position(|g| g.symbol_name == symbol) {
            self.groups.remove(pos);
        } else {
            // Assign next available color from the 4-color palette
            let used_colors: Vec<OccurrenceColor> = self.groups.iter().map(|g| g.color).collect();
            let next_color = OccurrenceColor::ALL
                .iter()
                .copied()
                .find(|c| !used_colors.contains(c))
                .unwrap_or(OccurrenceColor::Teal);

            if self.groups.len() >= 4 {
                self.groups.remove(0); // Evict oldest if reaching capacity
            }

            self.groups.push(OccurrencePinGroup {
                symbol_name: symbol.to_string(),
                ranges,
                color: next_color,
            });
        }
    }

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

        // Toggle foo off
        store.toggle_pin("foo", vec![]);
        assert!(!store.is_pinned("foo"));
        assert_eq!(store.groups.len(), 1);

        // Unpin by byte offset in bar (offset 21)
        assert!(store.unpin_group_containing(21));
        assert_eq!(store.groups.len(), 0);
    }
}
