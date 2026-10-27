//! Repository and pull request bookmarks with line-anchor relocation.
//!
//! [`SourceBookmark`] records line coordinates, text anchors, and surrounding lines.
//! If file contents change (e.g. after editing or pulling fresh commits),
//! [`SourceBookmark::resolve`] relocates the bookmark to its new canonical position.
//! [`BookmarkStore`] persists bookmarks to user configuration storage.

pub mod bookmark;
pub mod store;

pub use bookmark::SourceBookmark;
pub use store::BookmarkStore;
