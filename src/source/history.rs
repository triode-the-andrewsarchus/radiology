//! Linear navigation history stack with Back and Forward branch management.

use crate::source::location::SourceLocation;

/// Manages linear navigation history with Back and Forward capabilities.
///
/// Navigating to a new location truncates any forward history, matching
/// standard browser and editor navigation paradigms.
///
/// # Examples
///
/// ```rust
/// use radiology::source::history::NavigationHistory;
/// use radiology::source::location::SourceLocation;
///
/// let mut history = NavigationHistory::new();
/// history.visit(SourceLocation::new("a.rs", 10));
/// history.visit(SourceLocation::new("b.rs", 20));
///
/// assert!(history.can_go_back());
/// assert_eq!(history.back().unwrap().path, "a.rs");
/// assert!(history.can_go_forward());
/// assert_eq!(history.forward().unwrap().path, "b.rs");
/// ```
#[derive(Debug, Clone, Default)]
pub struct NavigationHistory {
    entries: Vec<SourceLocation>,
    index: Option<usize>,
}

impl NavigationHistory {
    /// Creates an empty [`NavigationHistory`] stack.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if there are entries prior to the current location.
    #[must_use]
    pub fn can_go_back(&self) -> bool {
        match self.index {
            Some(i) => i > 0,
            None => false,
        }
    }

    /// Returns `true` if there are entries ahead of the current location.
    #[must_use]
    pub fn can_go_forward(&self) -> bool {
        match self.index {
            Some(i) => i + 1 < self.entries.len(),
            None => false,
        }
    }

    /// Returns a reference to the active [`SourceLocation`].
    #[must_use]
    pub fn current(&self) -> Option<&SourceLocation> {
        self.index.and_then(|i| self.entries.get(i))
    }

    /// Appends a new destination, truncating forward branches and deduplicating identical visits.
    pub fn visit(&mut self, location: SourceLocation) {
        if let Some(i) = self.index {
            if let Some(curr) = self.entries.get(i) {
                if curr == &location {
                    return;
                }
            }
            self.entries.truncate(i + 1);
        } else {
            self.entries.clear();
        }

        self.entries.push(location);
        self.index = Some(self.entries.len() - 1);
    }

    /// Updates the current entry's coordinates without creating a new history step.
    pub fn update_current(&mut self, location: SourceLocation) {
        if let Some(i) = self.index {
            if i < self.entries.len() {
                self.entries[i] = location;
            }
        }
    }

    /// Steps backward in history, returning the previous [`SourceLocation`].
    pub fn back(&mut self) -> Option<SourceLocation> {
        if !self.can_go_back() {
            return None;
        }
        let i = self.index.unwrap() - 1;
        self.index = Some(i);
        Some(self.entries[i].clone())
    }

    /// Steps forward in history, returning the next [`SourceLocation`].
    pub fn forward(&mut self) -> Option<SourceLocation> {
        if !self.can_go_forward() {
            return None;
        }
        let i = self.index.unwrap() + 1;
        self.index = Some(i);
        Some(self.entries[i].clone())
    }

    /// Clears all navigation history.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.index = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_history() {
        let mut history = NavigationHistory::new();
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());

        let loc1 = SourceLocation::new("a.rs", 10);
        let loc2 = SourceLocation::new("b.rs", 20);
        let loc3 = SourceLocation::new("c.rs", 30);

        history.visit(loc1.clone());
        assert!(!history.can_go_back());
        assert_eq!(history.current(), Some(&loc1));

        history.visit(loc1.clone());
        assert_eq!(history.entries.len(), 1);

        history.visit(loc2.clone());
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());
        assert_eq!(history.current(), Some(&loc2));

        history.visit(loc3);
        assert_eq!(history.entries.len(), 3);

        let b = history.back();
        assert_eq!(b, Some(loc2.clone()));
        assert!(history.can_go_forward());
        assert!(history.can_go_back());

        let b2 = history.back();
        assert_eq!(b2, Some(loc1.clone()));
        assert!(!history.can_go_back());

        let f = history.forward();
        assert_eq!(f, Some(loc2.clone()));

        let loc4 = SourceLocation::new("d.rs", 40);
        history.visit(loc4.clone());
        assert_eq!(history.entries.len(), 3);
        assert!(!history.can_go_forward());
        assert_eq!(history.current(), Some(&loc4));
    }
}
