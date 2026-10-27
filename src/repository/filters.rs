use crate::repository::file_node::RepositoryFile;
use serde::{Deserialize, Serialize};

/// User preferences for filtering the repository file tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SourceFilters {
    pub source_files_only: bool,
    pub hides_tests: bool,
    #[serde(skip)]
    pub path_query: String,
}

impl SourceFilters {
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
}
