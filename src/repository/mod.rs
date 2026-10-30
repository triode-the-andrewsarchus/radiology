//! Repository discovery, file classification, hierarchical file trees, and fuzzy searching.
//!
//! [`RepositoryDiscovery`] enumerates files using `git ls-files` with NUL delimiters when
//! available, falling back to directory traversal with standard exclusions (`.git`, `target`,
//! `node_modules`, etc.). Files are classified by [`SourceFileClassifier`] and [`TestPathClassifier`],
//! structured into a navigable [`FileNode`] hierarchy, filtered by [`SourceFilters`], and searched
//! via [`QuickFileSearch`] using ordered subsequence fuzzy matching.

pub mod classifier;
pub mod discovery;
pub mod file_node;
pub mod filters;
pub(crate) mod git_refresh;
pub mod quick_search;

pub use classifier::{SourceFileClassifier, TestPathClassifier};
pub use discovery::RepositoryDiscovery;
pub use file_node::{FileNode, RepositoryFile};
pub use filters::SourceFilters;
pub use quick_search::{QuickFileSearch, QuickFileSearchResponse};
