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
        if let Some(pos) = self
            .at_location(buffer, byte_offset)
            .and_then(|b| self.index_of(b.id))
        {
            self.bookmarks.remove(pos);
            false
        } else {
            let line = buffer.line_at_byte(byte_offset);
            let label = format!("{}:{}", buffer.path, line);
            self.bookmarks
                .push(SourceBookmark::new(buffer, byte_offset, label));
            true
        }
    }

    /// Finds a bookmark at the exact canonical byte in the current document and revision.
    ///
    /// Relocation occurs during navigation, never while painting current-location controls.
    #[must_use]
    pub fn at_location(
        &self,
        buffer: &SourceBuffer,
        byte_offset: usize,
    ) -> Option<&SourceBookmark> {
        let byte = buffer.clamp_to_char_boundary(byte_offset.min(buffer.byte_len()));
        self.bookmarks.iter().find(|bookmark| {
            bookmark.path == buffer.path
                && bookmark.revision == buffer.revision
                && bookmark.byte_offset == byte
        })
    }

    /// Returns the current slot of an identity, including after reordering.
    #[must_use]
    pub fn index_of(&self, id: Uuid) -> Option<usize> {
        self.bookmarks.iter().position(|bookmark| bookmark.id == id)
    }

    /// Moves an identity to a final slot while preserving the other entries' order.
    /// Returns false without changing the store for a missing identity or invalid slot.
    pub fn move_to(&mut self, id: Uuid, slot: usize) -> bool {
        let Some(index) = self.index_of(id) else {
            return false;
        };
        if slot >= self.bookmarks.len() || slot == index {
            return false;
        }
        let bookmark = self.bookmarks.remove(index);
        self.bookmarks.insert(slot, bookmark);
        true
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

    /// Loads valid bookmarks from a host-selected configuration directory.
    ///
    /// Discards malformed entries, invalid repository paths, zero lines and duplicate
    /// identities. The first valid entry for each identity retains its original order.
    #[must_use]
    pub fn load_in(config_dir: &Path, scope: &BookmarkScope) -> Self {
        #[derive(Deserialize)]
        struct Archive {
            bookmarks: Vec<serde_json::Value>,
        }
        let archive = std::fs::read(Self::storage_path_in(config_dir, &scope.key()))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Archive>(&bytes).ok());
        let mut seen = std::collections::HashSet::new();
        Self {
            bookmarks: archive
                .into_iter()
                .flat_map(|archive| archive.bookmarks)
                .filter_map(|value| serde_json::from_value::<SourceBookmark>(value).ok())
                .filter(|bookmark| {
                    crate::source::SourceLocation::is_repository_path(&bookmark.path)
                        && bookmark.line > 0
                        && seen.insert(bookmark.id)
                })
                .collect(),
        }
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
    fn identities_survive_slot_assignment_and_invalid_moves_are_noops() {
        let buffer = SourceBuffer::from_str("a.rs", "first\nsecond\nthird\n");
        let mut store = BookmarkStore::new();
        for line in 1..=3 {
            store.toggle(&buffer, buffer.byte_offset(line, 0));
        }
        let original = store.bookmarks.clone();
        let id = original[2].id;
        assert!(store.move_to(id, 0));
        assert_eq!(
            store.bookmarks,
            vec![
                original[2].clone(),
                original[0].clone(),
                original[1].clone()
            ]
        );
        assert_eq!(store.index_of(id), Some(0));
        assert_eq!(
            store
                .at_location(&buffer, buffer.byte_offset(3, 0))
                .unwrap()
                .id,
            id
        );
        assert!(store.move_to(id, 2));
        assert_eq!(store.bookmarks, original);
        assert!(!store.move_to(id, 3));
        assert!(!store.move_to(Uuid::new_v4(), 0));
        assert!(!store.move_to(id, 2));
        assert_eq!(store.bookmarks, original);
        let mut before = buffer.clone();
        before.revision = Some(crate::source::location::ReviewRevision::Before);
        assert!(store.at_location(&before, 0).is_none());
    }

    #[test]
    fn test_scope_separation() {
        let scope1 = BookmarkScope::new("/tmp/repo", None);
        let scope2 = BookmarkScope::new("/tmp/repo", Some(42));
        let scope3 = BookmarkScope::new("/tmp/other", None);

        assert_ne!(scope1.key(), scope2.key());
        assert_ne!(scope1.key(), scope3.key());
    }

    #[test]
    fn exact_unicode_columns_are_independent_bookmarks_and_revisions() {
        let buffer = SourceBuffer::from_str("a.rs", "é🦀漢字\r\nnext\r\n");
        let mut store = BookmarkStore::new();
        for byte in [0, 2, 6] {
            assert!(store.toggle(&buffer, byte));
        }
        let original = store.bookmarks.clone();
        for (slot, byte) in [0, 2, 6].into_iter().enumerate() {
            assert_eq!(
                store.at_location(&buffer, byte).unwrap().id,
                original[slot].id
            );
        }
        assert_eq!(store.at_location(&buffer, 3).unwrap().id, original[1].id);
        assert!(store.at_location(&buffer, 9).is_none());
        let mut other = buffer.clone();
        other.path = "b.rs".into();
        assert!(store.at_location(&other, 2).is_none());
        other.path = buffer.path.clone();
        other.revision = Some(crate::source::ReviewRevision::Before);
        assert!(store.toggle(&other, 2));
        assert!(!store.toggle(&buffer, 2));
        assert_eq!(
            store.bookmarks.iter().map(|b| b.id).collect::<Vec<_>>(),
            vec![
                original[0].id,
                original[2].id,
                store.at_location(&other, 2).unwrap().id
            ]
        );
    }

    #[test]
    fn persisted_records_keep_first_valid_unique_identity_and_recover_other_entries() {
        let dir = tempfile::tempdir().unwrap();
        let scope = BookmarkScope::new(dir.path(), None);
        let buffer = SourceBuffer::from_str("a.rs", "é🦀漢字\r\n");
        let first = SourceBookmark::new(&buffer, 0, "first");
        let second = SourceBookmark::new(&buffer, 2, "second");
        let same_location = SourceBookmark::new(&buffer, 0, "distinct identity");
        let mut zero_line = serde_json::to_value(&second).unwrap();
        zero_line["line"] = 0.into();
        let mut negative = serde_json::to_value(&second).unwrap();
        negative["byte_offset"] = (-1).into();
        let mut bad_id = serde_json::to_value(&second).unwrap();
        bad_id["id"] = "broken".into();
        let mut duplicate = first.clone();
        duplicate.label = "duplicate".into();
        let mut records = vec![
            serde_json::to_value(&first).unwrap(),
            zero_line,
            negative,
            bad_id,
        ];
        for path in ["", "/absolute.rs", "../escape.rs", "src/../escape.rs"] {
            let mut invalid = serde_json::to_value(&second).unwrap();
            invalid["path"] = path.into();
            records.push(invalid);
        }
        records.extend([
            serde_json::to_value(duplicate).unwrap(),
            serde_json::to_value(&second).unwrap(),
            serde_json::to_value(&same_location).unwrap(),
        ]);
        let raw = serde_json::to_vec(&serde_json::json!({"bookmarks":records})).unwrap();
        let path = BookmarkStore::storage_path_in(dir.path(), &scope.key());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &raw).unwrap();
        let loaded = BookmarkStore::load_in(dir.path(), &scope);
        assert_eq!(loaded.bookmarks, vec![first, second, same_location]);
        assert_eq!(
            std::fs::read(&path).unwrap(),
            raw,
            "loading must not rewrite the archive"
        );
        loaded.save_in(dir.path(), &scope).unwrap();
        assert_eq!(
            BookmarkStore::load_in(dir.path(), &scope).bookmarks,
            loaded.bookmarks
        );
        assert!(
            BookmarkStore::load_in(dir.path(), &BookmarkScope::new(dir.path(), Some(7)))
                .bookmarks
                .is_empty()
        );
    }
}
