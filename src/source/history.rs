use crate::source::location::SourceLocation;

/// Linear navigation history with Back and Forward stacks.
/// Navigating to a new location discards the forward branch.
#[derive(Debug, Clone, Default)]
pub struct NavigationHistory {
    entries: Vec<SourceLocation>,
    index: Option<usize>,
}

impl NavigationHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn can_go_back(&self) -> bool {
        match self.index {
            Some(i) => i > 0,
            None => false,
        }
    }

    pub fn can_go_forward(&self) -> bool {
        match self.index {
            Some(i) => i + 1 < self.entries.len(),
            None => false,
        }
    }

    pub fn current(&self) -> Option<&SourceLocation> {
        self.index.and_then(|i| self.entries.get(i))
    }

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

    pub fn update_current(&mut self, location: SourceLocation) {
        if let Some(i) = self.index {
            if i < self.entries.len() {
                self.entries[i] = location;
            }
        }
    }

    pub fn back(&mut self) -> Option<SourceLocation> {
        if !self.can_go_back() {
            return None;
        }
        let i = self.index.unwrap() - 1;
        self.index = Some(i);
        Some(self.entries[i].clone())
    }

    pub fn forward(&mut self) -> Option<SourceLocation> {
        if !self.can_go_forward() {
            return None;
        }
        let i = self.index.unwrap() + 1;
        self.index = Some(i);
        Some(self.entries[i].clone())
    }

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

        // Re-visiting exact same location does not duplicate
        history.visit(loc1.clone());
        assert_eq!(history.entries.len(), 1);

        history.visit(loc2.clone());
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());
        assert_eq!(history.current(), Some(&loc2));

        history.visit(loc3.clone());
        assert_eq!(history.entries.len(), 3);

        // Back to loc2
        let b = history.back();
        assert_eq!(b, Some(loc2.clone()));
        assert!(history.can_go_forward());
        assert!(history.can_go_back());

        // Back to loc1
        let b2 = history.back();
        assert_eq!(b2, Some(loc1.clone()));
        assert!(!history.can_go_back());

        // Forward to loc2
        let f = history.forward();
        assert_eq!(f, Some(loc2.clone()));

        // Branching: visit loc4 from loc2 discards loc3
        let loc4 = SourceLocation::new("d.rs", 40);
        history.visit(loc4.clone());
        assert_eq!(history.entries.len(), 3); // loc1, loc2, loc4
        assert!(!history.can_go_forward());
        assert_eq!(history.current(), Some(&loc4));
    }
}
