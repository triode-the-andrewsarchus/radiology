//! Selection ownership for one repository and immutable semantic snapshot.
use super::SemanticSymbolID;
use crate::semantic::{trait_impls::TraitImplementation, SemanticIndex};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

/// Trait rows and independent selection retained while following the same trait.
///
/// ```
/// use radiology::semantic::{SemanticIndex, index::SemanticSymbolID,
///     trait_context::TraitImplementationContext};
/// use std::{path::Path, sync::Arc};
/// let index = Arc::new(SemanticIndex::default());
/// let target = SemanticSymbolID::new("scip test fixture 1 Read#", "a.rs");
/// let mut context = TraitImplementationContext::default();
/// assert!(context.update(Some(Path::new("repo")), Some(index.clone()), Some(target.clone())));
/// context.navigation_error = Some("Old destination changed".into());
/// assert!(!context.update(Some(Path::new("repo")), Some(index.clone()), Some(target.clone())));
/// assert!(context.navigation_error.is_some());
/// assert!(context.update(Some(Path::new("other-repo")), Some(index), Some(target)));
/// assert!(context.navigation_error.is_none());
/// ```
#[derive(Default)]
pub struct TraitImplementationContext {
    root: Option<PathBuf>,
    index: Option<Arc<SemanticIndex>>,
    target: Option<SemanticSymbolID>,
    /// Position in the immutable, source-sorted row slice; None means no selection.
    pub selected: Option<usize>,
    /// Last source-verified activation failure, retained until a new selection/context.
    pub navigation_error: Option<String>,
}

impl TraitImplementationContext {
    /// Updates ownership, clearing selection/errors only when root, index or trait changes.
    /// Returns whether ownership changed. Source/caret movement within a trait retains rows.
    pub fn update(
        &mut self,
        root: Option<&Path>,
        index: Option<Arc<SemanticIndex>>,
        target: Option<SemanticSymbolID>,
    ) -> bool {
        let target = target.filter(|_| index.is_some());
        let same_index = match (&self.index, &index) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        if self.root.as_deref() == root && same_index && self.target == target {
            return false;
        }
        self.root = root.map(Path::to_owned);
        self.index = index;
        self.target = target;
        self.selected = None;
        self.navigation_error = None;
        true
    }

    /// Returns the exact trait identity, including traits with no indexed impls.
    pub fn target(&self) -> Option<&SemanticSymbolID> {
        self.target.as_ref()
    }

    /// Returns the accepted catalog owner, for provenance and coverage presentation.
    pub fn index(&self) -> Option<&Arc<SemanticIndex>> {
        self.index.as_ref()
    }

    /// Returns source-sorted rows without copying or regrouping on each frame.
    pub fn rows(&self) -> &[TraitImplementation] {
        self.index
            .as_ref()
            .zip(self.target.as_ref())
            .and_then(|(index, target)| index.trait_implementations.by_trait.get(target))
            .map_or(&[], Vec::as_slice)
    }

    /// Selects a listed row, or clears selection, without moving the source caret.
    pub fn select(&mut self, row: Option<usize>) {
        self.selected = row.filter(|&row| row < self.rows().len());
        self.navigation_error = None;
    }
}
