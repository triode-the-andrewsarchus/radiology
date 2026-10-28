//! Asynchronous streaming ripgrep search subsystem.
//!
//! Executes `rg --json` asynchronously using [`SearchRunner`], decoding stream JSON
//! lines into [`SearchMatch`] records with support for regex, case-sensitivity, and glob filters.

pub mod match_item;
pub mod ripgrep;
pub mod session;

pub use match_item::SearchMatch;
pub use ripgrep::{SearchError, SearchOptions, SearchRunner, SEARCH_RESULT_LIMIT};
pub use session::RepositorySearch;
