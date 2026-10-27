//! Quick fuzzy file search within the repository.
//!
//! Provides 5-tier lexicographical candidate ranking, word boundary detection,
//! diacritic/case folding, and prefix-constrained fuzzy matching matching the
//! reference algorithm.

use crate::repository::file_node::RepositoryFile;
use std::collections::HashSet;
use unicode_normalization::UnicodeNormalization;

/// Result of executing a [`QuickFileSearch`] query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickFileSearchResponse {
    /// Matched repository files ranked best-first, capped at the requested limit.
    pub files: Vec<RepositoryFile>,
    /// Total number of candidates matching the query across the entire repository.
    pub total_count: usize,
}

impl QuickFileSearchResponse {
    /// Returns `true` if more files matched the query than were returned within the limit.
    #[must_use]
    pub fn is_limited(&self) -> bool {
        self.total_count > self.files.len()
    }
}

/// Fast subsequence and boundary-aware fuzzy file search for the repository file tree.
pub struct QuickFileSearch;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Candidate {
    file: RepositoryFile,
    tier: usize,
    boundary_misses: usize,
    gaps: usize,
    path_length: usize,
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.tier
            .cmp(&other.tier)
            .then_with(|| self.boundary_misses.cmp(&other.boundary_misses))
            .then_with(|| self.gaps.cmp(&other.gaps))
            .then_with(|| self.path_length.cmp(&other.path_length))
            .then_with(|| self.file.path.cmp(&other.file.path))
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

struct SearchText {
    characters: Vec<char>,
    boundaries: HashSet<usize>,
}

impl QuickFileSearch {
    /// Default maximum number of results returned by [`search`](Self::search).
    pub const VISIBLE_LIMIT: usize = 100;

    /// Searches `files` for candidates matching `query`, returning up to `limit` best matches.
    ///
    /// # Examples
    ///
    /// ```
    /// use radiology::repository::file_node::RepositoryFile;
    /// use radiology::repository::quick_search::QuickFileSearch;
    ///
    /// let files = vec![
    ///     RepositoryFile::new("src/main.rs"),
    ///     RepositoryFile::new("src/lib.rs"),
    ///     RepositoryFile::new("src/buffer.rs"),
    ///     RepositoryFile::new("tests/sample.rs"),
    /// ];
    ///
    /// let resp = QuickFileSearch::search(&files, "main", 10);
    /// assert_eq!(resp.files.len(), 1);
    /// assert_eq!(resp.files[0].path, "src/main.rs");
    /// ```
    #[must_use]
    pub fn search(files: &[RepositoryFile], query: &str, limit: usize) -> QuickFileSearchResponse {
        let trimmed = query.trim();
        let normalized_query = Self::normalize(trimmed);

        if normalized_query.is_empty() {
            let count = files.len().min(limit);
            return QuickFileSearchResponse {
                files: files[..count].to_vec(),
                total_count: files.len(),
            };
        }

        let mut best: Vec<Candidate> = Vec::with_capacity(limit);
        let mut total = 0;

        for file in files {
            if let Some(candidate) = Self::candidate_for(file, &normalized_query) {
                total += 1;
                if limit > 0 {
                    let pos = best.partition_point(|c| c < &candidate);
                    if pos < limit {
                        best.insert(pos, candidate);
                        if best.len() > limit {
                            best.pop();
                        }
                    } else if best.len() < limit {
                        best.push(candidate);
                    }
                }
            }
        }

        QuickFileSearchResponse {
            files: best.into_iter().map(|c| c.file).collect(),
            total_count: total,
        }
    }

    fn candidate_for(file: &RepositoryFile, query: &[char]) -> Option<Candidate> {
        let name = Self::search_text(&file.name());
        let path = Self::search_text(&file.path);

        // Tier 0: Exact filename match
        if name.characters == query {
            return Some(Candidate {
                file: file.clone(),
                tier: 0,
                boundary_misses: 0,
                gaps: 0,
                path_length: path.characters.len(),
            });
        }

        // Tier 1: Filename prefix match
        if name.characters.starts_with(query) {
            return Some(Candidate {
                file: file.clone(),
                tier: 1,
                boundary_misses: 0,
                gaps: name.characters.len().saturating_sub(query.len()),
                path_length: path.characters.len(),
            });
        }

        // Tier 2: Contiguous substring match in filename
        if let Some(start) = Self::contiguous_start(query, &name.characters) {
            return Some(Candidate {
                file: file.clone(),
                tier: 2,
                boundary_misses: if name.boundaries.contains(&start) {
                    0
                } else {
                    1
                },
                gaps: name.characters.len().saturating_sub(query.len()),
                path_length: path.characters.len(),
            });
        }

        // Tier 3: Subsequence match within filename
        if let Some((gaps, boundary_misses)) = Self::subsequence_score(query, &name) {
            return Some(Candidate {
                file: file.clone(),
                tier: 3,
                boundary_misses,
                gaps,
                path_length: path.characters.len(),
            });
        }

        // Tier 4: Subsequence match across full path
        if let Some((gaps, boundary_misses)) = Self::subsequence_score(query, &path) {
            return Some(Candidate {
                file: file.clone(),
                tier: 4,
                boundary_misses,
                gaps,
                path_length: path.characters.len(),
            });
        }

        None
    }

    /// Normalizes text by decomposing diacritics via NFD and folding to lowercase.
    fn normalize(value: &str) -> Vec<char> {
        value
            .nfd()
            .filter(|&c| !Self::is_combining_mark(c))
            .flat_map(|c| c.to_lowercase())
            .collect()
    }

    /// Builds [`SearchText`] tracking word and path boundary positions.
    fn search_text(value: &str) -> SearchText {
        let mut characters = Vec::new();
        let mut boundaries = HashSet::new();
        let mut prev_lower_or_digit = false;

        for ch in value.chars() {
            let start = characters.len();
            let is_upper = ch.is_uppercase();
            let is_sep = matches!(ch, '_' | '-' | '.' | ' ' | '/' | '\\');

            if start == 0 || is_sep || (is_upper && prev_lower_or_digit) {
                boundaries.insert(start);
            }

            for norm in ch.to_lowercase() {
                // Strip combining marks
                for c in norm.nfd().filter(|&c| !Self::is_combining_mark(c)) {
                    characters.push(c);
                }
            }

            prev_lower_or_digit = ch.is_lowercase() || ch.is_ascii_digit();
            if is_sep {
                boundaries.insert(characters.len());
            }
        }

        SearchText {
            characters,
            boundaries,
        }
    }

    fn is_combining_mark(c: char) -> bool {
        matches!(c, '\u{0300}'..='\u{036f}' | '\u{1ab0}'..='\u{1aff}' | '\u{1dc0}'..='\u{1dff}' | '\u{20d0}'..='\u{20ff}' | '\u{fe20}'..='\u{fe2f}')
    }

    fn contiguous_start(needle: &[char], haystack: &[char]) -> Option<usize> {
        if needle.len() > haystack.len() {
            return None;
        }
        for start in 0..=(haystack.len() - needle.len()) {
            if &haystack[start..start + needle.len()] == needle {
                return Some(start);
            }
        }
        None
    }

    fn subsequence_score(needle: &[char], haystack: &SearchText) -> Option<(usize, usize)> {
        let mut matched = Vec::with_capacity(needle.len());
        let mut cursor = 0;

        for &ch in needle {
            let found = haystack.characters[cursor..]
                .iter()
                .position(|&c| c == ch)?;
            let index = cursor + found;
            matched.push(index);
            cursor = index + 1;
        }

        let first = *matched.first()?;
        let last = *matched.last()?;
        let span = last - first + 1;
        let gaps = span.saturating_sub(needle.len());

        let boundary_misses = matched
            .iter()
            .filter(|idx| !haystack.boundaries.contains(idx))
            .count();

        Some((gaps, boundary_misses))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_search_subsequence() {
        let files = vec![
            RepositoryFile::new("src/main.rs"),
            RepositoryFile::new("src/buffer.rs"),
            RepositoryFile::new("src/history.rs"),
            RepositoryFile::new("tests/sample.rs"),
        ];

        let results = QuickFileSearch::search(&files, "buf", 10);
        assert_eq!(results.files.len(), 1);
        assert_eq!(results.files[0].path, "src/buffer.rs");
    }

    #[test]
    fn test_ranking_favors_filename_tiers() {
        let files = vec![
            RepositoryFile::new("z/scan.swift"),
            RepositoryFile::new("scan-tools/other.swift"),
            RepositoryFile::new("a/scanner.swift"),
            RepositoryFile::new("b/rescan.swift"),
            RepositoryFile::new("c/source-canary.swift"),
        ];

        let resp = QuickFileSearch::search(&files, "scan", 10);
        let paths: Vec<&str> = resp.files.iter().map(|f| f.path.as_str()).collect();

        assert_eq!(
            paths,
            vec![
                "z/scan.swift",           // Tier 0: exact filename match
                "a/scanner.swift",        // Tier 1: filename prefix match
                "b/rescan.swift",         // Tier 2: contiguous substring in filename
                "c/source-canary.swift",  // Tier 3: subsequence in filename
                "scan-tools/other.swift", // Tier 4: subsequence in path only
            ]
        );
    }

    #[test]
    fn test_diacritic_folding() {
        let files = vec![
            RepositoryFile::new("b/Café.swift"),
            RepositoryFile::new("a/CAFÉ.swift"),
            RepositoryFile::new("c/other.swift"),
        ];

        let resp = QuickFileSearch::search(&files, "cafe", 10);
        assert_eq!(resp.files.len(), 2);
    }
}
