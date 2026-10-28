//! Persistent bookmark store and ordering management.

use crate::bookmarks::bookmark::SourceBookmark;
use crate::source::buffer::SourceBuffer;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Scope identifying whether bookmarks belong to the local repository checkout or a pull request.
///
/// Ensures checkout bookmarks and PR review bookmarks remain completely isolated.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BookmarkScope {
    /// Canonical repository path.
    pub repository_path: String,
    /// Pull request number if viewing a PR.
    pub pull_request: Option<usize>,
}

impl BookmarkScope {
    /// Creates a new checkout or pull request scope.
    #[must_use]
    pub fn new(root: impl AsRef<Path>, pull_request: Option<usize>) -> Self {
        let path = std::fs::canonicalize(root.as_ref())
            .unwrap_or_else(|_| root.as_ref().to_path_buf())
            .to_string_lossy()
            .to_string();
        Self {
            repository_path: path,
            pull_request,
        }
    }

    /// Scope storage key isolating checkout from PR numbers.
    #[must_use]
    pub fn key(&self) -> String {
        format!(
            "{}\0{}",
            self.repository_path,
            self.pull_request
                .map_or_else(|| "checkout".to_string(), |pr| format!("pr:{pr}"))
        )
    }
}

/// Manages ordered bookmarks for a repository checkout or pull request.
///
/// Supports keyboard shortcuts ⌘0..⌘9 to jump to the first ten bookmarks.
///
/// # Examples
///
/// ```rust
/// use radiology::bookmarks::store::BookmarkStore;
/// use radiology::source::SourceBuffer;
///
/// let buf = SourceBuffer::from_str("src/main.rs", "fn main() {}\n");
/// let mut store = BookmarkStore::new();
/// assert!(store.toggle(&buf, 0));
/// assert_eq!(store.bookmarks.len(), 1);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookmarkStore {
    /// Ordered list of bookmarks.
    pub bookmarks: Vec<SourceBookmark>,
}

impl BookmarkStore {
    /// Creates an empty [`BookmarkStore`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggles a bookmark at `byte_offset` in `buffer`. Returns `true` if added, `false` if removed.
    pub fn toggle(&mut self, buffer: &SourceBuffer, byte_offset: usize) -> bool {
        let line = buffer.line_at_byte(byte_offset);
        if let Some(pos) = self.bookmarks.iter().position(|bookmark| {
            bookmark
                .resolve(buffer)
                .ok()
                .is_some_and(|location| buffer.line_at_byte(location.byte_offset) == line)
        }) {
            self.bookmarks.remove(pos);
            false
        } else {
            let label = format!("{}:{}", buffer.path, line);
            self.bookmarks
                .push(SourceBookmark::new(buffer, byte_offset, label));
            true
        }
    }

    /// Removes the bookmark matching `id`.
    pub fn remove(&mut self, id: Uuid) {
        self.bookmarks.retain(|b| b.id != id);
    }

    /// Swaps the bookmark at `idx` with the preceding bookmark.
    pub fn move_up(&mut self, idx: usize) {
        if idx > 0 && idx < self.bookmarks.len() {
            self.bookmarks.swap(idx - 1, idx);
        }
    }

    /// Swaps the bookmark at `idx` with the following bookmark.
    pub fn move_down(&mut self, idx: usize) {
        if idx + 1 < self.bookmarks.len() {
            self.bookmarks.swap(idx, idx + 1);
        }
    }

    /// Returns a reference to the bookmark at the given zero-based index.
    #[must_use]
    pub fn get_by_index(&self, index: usize) -> Option<&SourceBookmark> {
        self.bookmarks.get(index)
    }

    fn storage_path_in(config_dir: &Path, key: &str) -> PathBuf {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let filename = format!("{:x}.json", hasher.finish());

        config_dir.join("bookmarks").join(filename)
    }

    /// Persists bookmarks for a given [`BookmarkScope`].
    pub fn save_scope(&self, scope: &BookmarkScope) {
        if let Some(dir) = dirs::config_dir() {
            let _ = self.save_in(&dir.join("radiology"), scope);
        }
    }

    /// Saves bookmarks in a host-selected configuration directory.
    ///
    /// # Errors
    /// Returns an error if serialization or writing the store fails.
    pub fn save_in(&self, config_dir: &Path, scope: &BookmarkScope) -> std::io::Result<()> {
        let path = Self::storage_path_in(config_dir, &scope.key());
        std::fs::create_dir_all(config_dir.join("bookmarks"))?;
        let json = serde_json::to_vec_pretty(self)?;
        let temporary = path.with_extension("json.tmp");
        std::fs::write(&temporary, json)?;
        std::fs::rename(temporary, path)
    }

    /// Loads persisted bookmarks for a given [`BookmarkScope`].
    #[must_use]
    pub fn load_scope(scope: &BookmarkScope) -> Self {
        dirs::config_dir().map_or_else(Self::new, |dir| {
            Self::load_in(&dir.join("radiology"), scope)
        })
    }

    /// Loads bookmarks from a host-selected configuration directory.
    #[must_use]
    pub fn load_in(config_dir: &Path, scope: &BookmarkScope) -> Self {
        std::fs::read(Self::storage_path_in(config_dir, &scope.key()))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// Persists bookmarks to disk for the given repository and optional pull request number.
    pub fn save(&self, repo_path: &str, pr: Option<usize>) {
        let scope = BookmarkScope::new(repo_path, pr);
        self.save_scope(&scope);
    }

    /// Loads persisted bookmarks from disk for the given repository and optional PR number.
    #[must_use]
    pub fn load(repo_path: &str, pr: Option<usize>) -> Self {
        let scope = BookmarkScope::new(repo_path, pr);
        Self::load_scope(&scope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_toggle_and_reorder() {
        let code = "line 1\nline 2\nline 3\n";
        let buf = SourceBuffer::from_str("src/main.rs", code);
        let mut store = BookmarkStore::new();

        assert!(store.toggle(&buf, 0));
        assert_eq!(store.bookmarks.len(), 1);

        let byte2 = buf.byte_offset(2, 0);
        assert!(store.toggle(&buf, byte2));
        assert_eq!(store.bookmarks.len(), 2);

        store.move_up(1);
        assert_eq!(store.bookmarks[0].line, 2);

        assert!(!store.toggle(&buf, byte2));
        assert_eq!(store.bookmarks.len(), 1);
    }

    #[test]
    fn test_scope_separation() {
        let scope1 = BookmarkScope::new("/tmp/repo", None);
        let scope2 = BookmarkScope::new("/tmp/repo", Some(42));
        let scope3 = BookmarkScope::new("/tmp/other", None);

        assert_ne!(scope1.key(), scope2.key());
        assert_ne!(scope1.key(), scope3.key());
    }
}
