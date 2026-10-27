pub mod match_item;
pub mod ripgrep;

pub use match_item::SearchMatch;
pub use ripgrep::{SearchError, SearchOptions, SearchRunner, SEARCH_RESULT_LIMIT};
