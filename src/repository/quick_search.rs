//! Fuzzy file searcher based on ordered subsequence matching with ergonomic scoring.

use crate::repository::file_node::RepositoryFile;

/// Represents a single search match result from [`QuickFileSearch`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickSearchResult {
    /// Matched file metadata.
    pub file: RepositoryFile,
    /// Computed match score, where higher values indicate stronger relevance.
    pub score: i64,
    /// Character indices within the file path that matched the search query.
    pub matched_indices: Vec<usize>,
}

/// Provides fuzzy subsequence matching for fast file navigation (Ctrl+K).
///
/// Matches are scored with bonuses for:
/// - Matching within the filename component rather than directory prefix
/// - Consecutive character sequences
/// - Word boundaries (after `/`, `_`, `-`, `.`) and CamelCase boundaries
///
/// # Examples
///
/// ```rust
/// use radiology::repository::file_node::RepositoryFile;
/// use radiology::repository::quick_search::QuickFileSearch;
///
/// let files = vec![
///     RepositoryFile::new("src/main.rs"),
///     RepositoryFile::new("engine-foo.rs"),
/// ];
/// let results = QuickFileSearch::search(&files, "engrs", 10);
/// assert_eq!(results[0].file.path, "engine-foo.rs");
/// ```
pub struct QuickFileSearch;

impl QuickFileSearch {
    /// Searches `files` for the ordered subsequence `query`, returning up to `limit` sorted results.
    #[must_use]
    pub fn search(files: &[RepositoryFile], query: &str, limit: usize) -> Vec<QuickSearchResult> {
        let query = query.trim();
        if query.is_empty() {
            return files
                .iter()
                .take(limit)
                .map(|f| QuickSearchResult {
                    file: f.clone(),
                    score: 0,
                    matched_indices: Vec::new(),
                })
                .collect();
        }

        let query_chars: Vec<char> = query.to_lowercase().chars().collect();
        let mut results = Vec::new();

        for file in files {
            if let Some((score, matched_indices)) =
                Self::match_subsequence(&file.path, &query_chars)
            {
                results.push(QuickSearchResult {
                    file: file.clone(),
                    score,
                    matched_indices,
                });
            }
        }

        results.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.file.path.len().cmp(&b.file.path.len()))
                .then_with(|| a.file.path.cmp(&b.file.path))
        });

        results.truncate(limit);
        results
    }

    fn match_subsequence(path: &str, query: &[char]) -> Option<(i64, Vec<usize>)> {
        let path_chars: Vec<char> = path.chars().collect();
        let path_lower: Vec<char> = path.to_lowercase().chars().collect();

        let filename_start = path.rfind('/').map_or(0, |i| i + 1);
        let mut matched_indices = Vec::with_capacity(query.len());

        let mut query_idx = 0;
        let mut score: i64 = 0;
        let mut prev_match_idx: Option<usize> = None;

        for (idx, &ch) in path_lower.iter().enumerate() {
            if query_idx < query.len() && ch == query[query_idx] {
                matched_indices.push(idx);

                let mut match_score = 10;

                if idx >= filename_start {
                    match_score += 15;
                }

                if idx == 0 || idx == filename_start {
                    match_score += 30;
                } else {
                    let prev_ch = path_chars[idx - 1];
                    if prev_ch == '/' || prev_ch == '_' || prev_ch == '-' || prev_ch == '.' {
                        match_score += 20;
                    } else if prev_ch.is_lowercase() && path_chars[idx].is_uppercase() {
                        match_score += 15;
                    }
                }

                if let Some(prev) = prev_match_idx {
                    if idx == prev + 1 {
                        match_score += 25;
                    }
                }

                score += match_score;
                prev_match_idx = Some(idx);
                query_idx += 1;
            }
        }

        if query_idx == query.len() {
            score -= (path.len() as i64) / 4;
            Some((score, matched_indices))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_search_subsequence() {
        let files = vec![
            RepositoryFile::new("src/main.rs"),
            RepositoryFile::new("src/source/buffer.rs"),
            RepositoryFile::new("src/repository/quick_search.rs"),
            RepositoryFile::new("engine-foo.rs"),
        ];

        let matches = QuickFileSearch::search(&files, "engrs", 10);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].file.path, "engine-foo.rs");

        let buf_matches = QuickFileSearch::search(&files, "buf", 10);
        assert_eq!(buf_matches[0].file.path, "src/source/buffer.rs");
    }
}
