//! File filtering options for the repository browser.

use crate::repository::file_node::RepositoryFile;
use serde::{Deserialize, Serialize};

/// Filters applied to the repository file tree.
///
/// # Examples
///
/// ```rust
/// use radiology::repository::file_node::RepositoryFile;
/// use radiology::repository::filters::SourceFilters;
///
/// let files = vec![
///     RepositoryFile::new("src/main.rs"),
///     RepositoryFile::new("tests/test_main.rs"),
/// ];
/// let mut filters = SourceFilters::default();
/// filters.hides_tests = true;
///
/// let filtered = filters.filter_files(&files);
/// assert_eq!(filtered.len(), 1);
/// assert_eq!(filtered[0].path, "src/main.rs");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SourceFilters {
    /// Hides generated, binary, and media files, showing only source code.
    pub source_files_only: bool,
    /// Hides test files, test suites, and test fixtures across all languages.
    pub hides_tests: bool,
    /// Live search query filter applied across file paths.
    #[serde(skip)]
    pub path_query: String,
}

impl SourceFilters {
    /// Checks whether an individual path passes the active filters.
    #[must_use]
    pub fn includes_path(&self, path: &str) -> bool {
        if self.source_files_only
            && !crate::repository::classifier::SourceFileClassifier::is_source_file(path)
        {
            return false;
        }
        if self.hides_tests && crate::repository::classifier::TestPathClassifier::is_test_path(path)
        {
            return false;
        }
        if !self.path_query.trim().is_empty()
            && !path
                .to_lowercase()
                .contains(&self.path_query.trim().to_lowercase())
        {
            return false;
        }
        true
    }

    /// Returns a new vector containing only files matching the active filter criteria.
    #[must_use]
    pub fn filter_files(&self, files: &[RepositoryFile]) -> Vec<RepositoryFile> {
        let query_lower = self.path_query.trim().to_lowercase();
        files
            .iter()
            .filter(|f| {
                if self.source_files_only && !f.is_source {
                    return false;
                }
                if self.hides_tests && f.is_test {
                    return false;
                }
                if !query_lower.is_empty() && !f.path.to_lowercase().contains(&query_lower) {
                    return false;
                }
                true
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filters() {
        let files = vec![
            RepositoryFile::new("src/main.rs"),
            RepositoryFile::new("tests/test_main.rs"),
            RepositoryFile::new("assets/logo.png"),
        ];

        let mut filters = SourceFilters::default();
        assert_eq!(filters.filter_files(&files).len(), 3);

        filters.source_files_only = true;
        let src_only = filters.filter_files(&files);
        assert_eq!(src_only.len(), 2);

        filters.hides_tests = true;
        let no_tests = filters.filter_files(&files);
        assert_eq!(no_tests.len(), 1);
        assert_eq!(no_tests[0].path, "src/main.rs");

        filters.hides_tests = false;
        filters.source_files_only = false;
        filters.path_query = "logo".to_string();
        let query_match = filters.filter_files(&files);
        assert_eq!(query_match.len(), 1);
        assert_eq!(query_match[0].path, "assets/logo.png");
    }
    #[test]
    fn reference_filter_combinations_prune_empty_directories_and_preserve_tree_order() {
        use crate::repository::file_node::FileNode;
        let files = [
            "README.md",
            "Cargo.toml",
            "docs/guide.md",
            "src/app.rs",
            "src/contest.rs",
            "src/app/bundle_tests.rs",
            "src/main/kotlin/A.kt",
            "src/test/kotlin/ATest.kt",
            "tests/fixtures/data.json",
            "tests/integration.rs",
            "web/App.tsx",
            "web/App.test.tsx",
            "web/package.json",
        ]
        .map(RepositoryFile::new);
        assert_eq!(SourceFilters::default().filter_files(&files), files);
        for (hides_tests, source_files_only, expected) in [
            (
                true,
                false,
                vec![
                    "docs",
                    "docs/guide.md",
                    "src",
                    "src/main",
                    "src/main/kotlin",
                    "src/main/kotlin/A.kt",
                    "src/app.rs",
                    "src/contest.rs",
                    "web",
                    "web/App.tsx",
                    "web/package.json",
                    "Cargo.toml",
                    "README.md",
                ],
            ),
            (
                false,
                true,
                vec![
                    "src",
                    "src/app",
                    "src/app/bundle_tests.rs",
                    "src/main",
                    "src/main/kotlin",
                    "src/main/kotlin/A.kt",
                    "src/test",
                    "src/test/kotlin",
                    "src/test/kotlin/ATest.kt",
                    "src/app.rs",
                    "src/contest.rs",
                    "tests",
                    "tests/integration.rs",
                    "web",
                    "web/App.test.tsx",
                    "web/App.tsx",
                ],
            ),
            (
                true,
                true,
                vec![
                    "src",
                    "src/main",
                    "src/main/kotlin",
                    "src/main/kotlin/A.kt",
                    "src/app.rs",
                    "src/contest.rs",
                    "web",
                    "web/App.tsx",
                ],
            ),
        ] {
            let filters = SourceFilters {
                hides_tests,
                source_files_only,
                ..Default::default()
            };
            let filtered = filters.filter_files(&files);
            assert_eq!(
                filtered,
                files
                    .iter()
                    .filter(|file| filters.includes_path(&file.path))
                    .cloned()
                    .collect::<Vec<_>>()
            );
            let tree = FileNode::build_tree(&filtered);
            let mut stack: Vec<_> = tree.iter().rev().collect();
            let mut paths = Vec::new();
            while let Some(node) = stack.pop() {
                paths.push(node.path.as_str());
                stack.extend(node.children.iter().rev());
            }
            assert_eq!(
                paths, expected,
                "hide tests={hides_tests}, source only={source_files_only}"
            );
        }
    }
}
