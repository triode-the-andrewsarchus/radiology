//! User session persistence across application launches.

use crate::repository::filters::SourceFilters;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Manages persistent reader session state across application restarts.
///
/// Records the last opened repository, recent repository list, and filter settings.
///
/// # Examples
///
/// ```rust
/// use radiology::session::store::SessionStore;
///
/// let mut session = SessionStore::default();
/// session.record_repository("/home/user/project");
/// assert_eq!(session.last_repository, Some("/home/user/project".to_string()));
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStore {
    /// Canonical path of the most recently browsed repository.
    pub last_repository: Option<String>,
    /// Chronological list of recently opened repository paths (most recent first).
    pub recent_repositories: Vec<String>,
    /// Pull request number active during last session, if any.
    pub active_pr: Option<usize>,
    /// Persisted file tree filters.
    pub source_filters: SourceFilters,
}

impl SessionStore {
    fn session_file_path() -> Option<PathBuf> {
        let dir = dirs::config_dir()?.join("radiology");
        let _ = std::fs::create_dir_all(&dir);
        Some(dir.join("session.json"))
    }

    /// Loads the persisted session from `~/.config/radiology/session.json`.
    #[must_use]
    pub fn load() -> Self {
        if let Some(path) = Self::session_file_path() {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(store) = serde_json::from_slice::<SessionStore>(&bytes) {
                    return store;
                }
            }
        }
        Self::default()
    }

    /// Saves the current session state to disk.
    pub fn save(&self) {
        if let Some(path) = Self::session_file_path() {
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    /// Records a visited repository path, updating the recent list and saving to disk.
    pub fn record_repository(&mut self, path: impl Into<String>) {
        let path = path.into();
        self.last_repository = Some(path.clone());
        self.recent_repositories.retain(|p| p != &path);
        self.recent_repositories.insert(0, path);
        if self.recent_repositories.len() > 10 {
            self.recent_repositories.truncate(10);
        }
        self.save();
    }
}
