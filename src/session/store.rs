//! User session persistence across application restarts.

use crate::repository::filters::SourceFilters;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Manages persistent reader session state across application restarts.
///
/// Records the last opened repository, recent repository list, active PR, and filter settings.
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
    /// Chronological list of recently opened repository paths (most recent first, capped at 8).
    pub recent_repositories: Vec<String>,
    /// Pull request number active during last session, if any.
    pub active_pr: Option<usize>,
    /// Persisted file tree filters.
    pub source_filters: SourceFilters,
    /// Whether source code lines wrap at the viewport boundary or scroll horizontally.
    #[serde(default)]
    pub wrap_lines: bool,
}

impl SessionStore {
    /// Maximum number of recent repository paths stored in the history.
    pub const RECENT_LIMIT: usize = 8;

    fn session_file_path() -> Option<PathBuf> {
        let dir = dirs::config_dir()?.join("radiology");
        let _ = std::fs::create_dir_all(&dir);
        Some(dir.join("session.json"))
    }

    /// Validates whether the session data represents a valid, existing repository on disk.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        if let Some(repo) = &self.last_repository {
            if !repo.starts_with('/') || !Path::new(repo).is_dir() {
                return false;
            }
        }
        if let Some(pr) = self.active_pr {
            if pr == 0 {
                return false;
            }
        }
        true
    }

    /// Loads the persisted session from `~/.config/radiology/session.json`.
    #[must_use]
    pub fn load() -> Self {
        if let Some(path) = Self::session_file_path() {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(store) = serde_json::from_slice::<SessionStore>(&bytes) {
                    if store.is_valid() {
                        return store;
                    }
                }
            }
        }
        Self::default()
    }

    /// Saves the current session state to disk if valid.
    pub fn save(&self) {
        if !self.is_valid() {
            return;
        }
        if let Some(path) = Self::session_file_path() {
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    /// Records a confirmed repository path, updating recent history capped at 8 entries.
    pub fn record_repository(&mut self, path: impl Into<String>) {
        let path = path.into();
        let canonical = Path::new(&path)
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(&path))
            .to_string_lossy()
            .to_string();

        self.last_repository = Some(canonical.clone());
        self.recent_repositories.retain(|p| p != &canonical);
        self.recent_repositories.insert(0, canonical);
        if self.recent_repositories.len() > Self::RECENT_LIMIT {
            self.recent_repositories.truncate(Self::RECENT_LIMIT);
        }
        self.save();
    }

    /// Updates the confirmed active pull request number and saves session state.
    pub fn set_active_pr(&mut self, pr: Option<usize>) {
        if let Some(num) = pr {
            if num > 0 {
                self.active_pr = Some(num);
            }
        } else {
            self.active_pr = None;
        }
        self.save();
    }
}
