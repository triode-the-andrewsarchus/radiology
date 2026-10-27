use crate::bookmarks::bookmark::SourceBookmark;
use crate::source::buffer::SourceBuffer;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookmarkStore {
    pub bookmarks: Vec<SourceBookmark>,
}

impl BookmarkStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle(&mut self, buffer: &SourceBuffer, byte_offset: usize) -> bool {
        let line = buffer.line_at_byte(byte_offset);
        if let Some(pos) = self
            .bookmarks
            .iter()
            .position(|b| b.path == buffer.path && b.line == line && b.revision == buffer.revision)
        {
            self.bookmarks.remove(pos);
            false
        } else {
            let label = format!("{}:{}", buffer.path, line);
            self.bookmarks
                .push(SourceBookmark::new(buffer, byte_offset, label));
            true
        }
    }

    pub fn remove(&mut self, id: Uuid) {
        self.bookmarks.retain(|b| b.id != id);
    }

    pub fn move_up(&mut self, idx: usize) {
        if idx > 0 && idx < self.bookmarks.len() {
            self.bookmarks.swap(idx - 1, idx);
        }
    }

    pub fn move_down(&mut self, idx: usize) {
        if idx + 1 < self.bookmarks.len() {
            self.bookmarks.swap(idx, idx + 1);
        }
    }

    pub fn get_by_index(&self, index: usize) -> Option<&SourceBookmark> {
        self.bookmarks.get(index)
    }

    fn storage_path(repo_path: &str, pr: Option<usize>) -> Option<PathBuf> {
        let config_dir = dirs::config_dir()?.join("radiology").join("bookmarks");
        let _ = std::fs::create_dir_all(&config_dir);

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        repo_path.hash(&mut hasher);
        pr.hash(&mut hasher);
        let filename = format!("{:x}.json", hasher.finish());

        Some(config_dir.join(filename))
    }

    pub fn save(&self, repo_path: &str, pr: Option<usize>) {
        if let Some(path) = Self::storage_path(repo_path, pr) {
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    pub fn load(repo_path: &str, pr: Option<usize>) -> Self {
        if let Some(path) = Self::storage_path(repo_path, pr) {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(store) = serde_json::from_slice::<BookmarkStore>(&bytes) {
                    return store;
                }
            }
        }
        Self::new()
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

        // Toggle on line 1
        assert!(store.toggle(&buf, 0));
        assert_eq!(store.bookmarks.len(), 1);

        // Toggle on line 2
        let byte2 = buf.byte_offset(2, 0);
        assert!(store.toggle(&buf, byte2));
        assert_eq!(store.bookmarks.len(), 2);

        // Reorder
        store.move_up(1);
        assert_eq!(store.bookmarks[0].line, 2);

        // Toggle line 2 off
        assert!(!store.toggle(&buf, byte2));
        assert_eq!(store.bookmarks.len(), 1);
    }
}
