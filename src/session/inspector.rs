//! App-wide inspector presentation preferences, independent of source state.
use serde::{Deserialize, Serialize};

/// Independently configurable inspector content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InspectorPane {
    /// Enclosing declaration and incoming references.
    References,
    /// Definition details or the document outline.
    Definition,
    /// Contextual Rust trait implementations.
    Implementations,
    /// Saved source locations.
    Bookmarks,
}

impl InspectorPane {
    /// All configurable panes, in menu order.
    pub const ALL: [Self; 4] = [
        Self::References,
        Self::Definition,
        Self::Implementations,
        Self::Bookmarks,
    ];

    /// Returns the menu title for this pane.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::References => "References",
            Self::Definition => "Definition / Outline",
            Self::Implementations => "Trait Implementations",
            Self::Bookmarks => "Bookmarks",
        }
    }
}

/// Persistent display choices; absent settings default to enabled.
///
/// ```
/// use radiology::session::inspector::{InspectorPane, InspectorPanes};
/// let mut panes = InspectorPanes::default();
/// panes.toggle(InspectorPane::References);
/// assert_eq!(panes.visible(false), vec![InspectorPane::Definition, InspectorPane::Bookmarks]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct InspectorPanes {
    /// Displays the enclosing declaration and references.
    pub references: bool,
    /// Displays definition details or outline when no trait pane is active.
    pub definition: bool,
    /// Displays trait implementations when a trait context is available.
    pub implementations: bool,
    /// Displays bookmarks.
    pub bookmarks: bool,
}

impl Default for InspectorPanes {
    fn default() -> Self {
        Self {
            references: true,
            definition: true,
            implementations: true,
            bookmarks: true,
        }
    }
}

impl InspectorPanes {
    fn choice(&mut self, pane: InspectorPane) -> &mut bool {
        match pane {
            InspectorPane::References => &mut self.references,
            InspectorPane::Definition => &mut self.definition,
            InspectorPane::Implementations => &mut self.implementations,
            InspectorPane::Bookmarks => &mut self.bookmarks,
        }
    }

    /// Returns whether a preference is enabled, independently of current context.
    #[must_use]
    pub fn enabled(mut self, pane: InspectorPane) -> bool {
        *self.choice(pane)
    }

    /// Toggles one preference without changing any source or contextual data.
    pub fn toggle(&mut self, pane: InspectorPane) {
        let value = self.choice(pane);
        *value = !*value;
    }

    /// Returns whether every preference is enabled.
    #[must_use]
    pub fn all_enabled(self) -> bool {
        self == Self::default()
    }

    /// Returns the active regions, with trait results sharing the definition slot.
    #[must_use]
    pub fn visible(self, has_trait: bool) -> Vec<InspectorPane> {
        let mut result = Vec::new();
        if self.references {
            result.push(InspectorPane::References);
        }
        if self.implementations && has_trait {
            result.push(InspectorPane::Implementations);
        } else if self.definition {
            result.push(InspectorPane::Definition);
        }
        if self.bookmarks {
            result.push(InspectorPane::Bookmarks);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pane_preferences_default_old_partial_sessions_and_share_the_middle_slot() {
        let partial: InspectorPanes = serde_json::from_str(r#"{"references":false}"#).unwrap();
        assert!(
            !partial.references
                && partial.definition
                && partial.implementations
                && partial.bookmarks
        );
        for mask in 0..16 {
            let mut panes = InspectorPanes::default();
            for (bit, pane) in InspectorPane::ALL.into_iter().enumerate() {
                if mask & (1 << bit) == 0 {
                    panes.toggle(pane);
                }
                assert_eq!(panes.enabled(pane), mask & (1 << bit) != 0);
            }
            for has_trait in [false, true] {
                let visible = panes.visible(has_trait);
                assert!(visible.len() <= 3);
                assert_eq!(
                    visible.contains(&InspectorPane::References),
                    panes.references
                );
                assert_eq!(visible.contains(&InspectorPane::Bookmarks), panes.bookmarks);
                assert_eq!(
                    visible.contains(&InspectorPane::Implementations),
                    panes.implementations && has_trait
                );
                assert_eq!(
                    visible.contains(&InspectorPane::Definition),
                    panes.definition && !(panes.implementations && has_trait)
                );
                let encoded = serde_json::to_vec(&panes).unwrap();
                assert_eq!(
                    serde_json::from_slice::<InspectorPanes>(&encoded).unwrap(),
                    panes
                );
            }
        }
        let owner = tempfile::tempdir().unwrap();
        let path = owner.path().join("session.json");
        let mut old = serde_json::to_value(crate::session::SessionStore::default()).unwrap();
        old.as_object_mut().unwrap().remove("inspector_panes");
        std::fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
        let mut session = crate::session::SessionStore::load_from(&path);
        assert!(session.inspector_panes.all_enabled());
        session.inspector_panes = partial;
        session.save();
        assert_eq!(
            crate::session::SessionStore::load_from(&path).inspector_panes,
            partial
        );
    }
}
