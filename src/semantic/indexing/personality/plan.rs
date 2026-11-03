use super::{Personality, Settings};
use crate::semantic::indexing::{IndexPlan, Indexer, IndexingStrategy};

impl Settings {
    /// Plans actual input paths using this capture's adapter filter and project cap.
    ///
    /// Filtering precedes the cap, so disallowed adapters cannot hide allowed
    /// projects. seL4 selects its one explicit pipeline; its runner validates that
    /// the checkout contains the configured seL4 build inputs.
    #[must_use]
    pub fn plan<'a>(&self, paths: impl IntoIterator<Item = &'a str>) -> IndexPlan {
        let paths: Vec<_> = paths
            .into_iter()
            .filter(|path| crate::source::location::SourceLocation::is_repository_path(path))
            .collect();
        let mut plan = if self.personality() == Personality::Sel4 {
            IndexPlan {
                selected: vec![IndexingStrategy {
                    indexer: Indexer::Sel4,
                    directory: String::new(),
                    evidence: vec!["CMakeLists.txt".into()],
                    source_files: paths
                        .iter()
                        .filter(|path| Indexer::Sel4.covers_path(path))
                        .count(),
                    personality_digest: None,
                }],
                ..Default::default()
            }
        } else {
            let mut plan = IndexPlan::with_limit(paths.iter().copied(), usize::MAX);
            plan.selected.retain(|strategy| {
                self.indexers().is_empty()
                    || self.indexers().iter().any(|id| id == strategy.indexer.id())
            });
            plan
        };
        plan.skipped = plan
            .selected
            .split_off(self.project_limit().min(plan.selected.len()));
        for strategy in plan.selected.iter_mut().chain(&mut plan.skipped) {
            strategy.personality_digest = Some(self.digest().into());
        }
        plan.uncovered_languages = super::super::uncovered_languages(paths, &plan.selected);
        plan
    }
}
