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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStore {
    /// Explicit settings file for embedded hosts and isolated test sessions.
    #[serde(skip)]
    storage_path: Option<PathBuf>,
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
    /// Source font size in points; metadata typography remains independent.
    #[serde(default = "default_source_font_size")]
    pub source_font_size: f32,
    /// Whether the reader uses dark appearance.
    #[serde(default = "default_dark")]
    pub dark_mode: bool,
}

impl Default for SessionStore {
    fn default() -> Self {
        Self {
            storage_path: None,
            last_repository: None,
            recent_repositories: Vec::new(),
            active_pr: None,
            source_filters: SourceFilters::default(),
            wrap_lines: false,
            source_font_size: default_source_font_size(),
            dark_mode: true,
        }
    }
}

fn default_source_font_size() -> f32 {
    14.0
}

fn default_dark() -> bool {
    true
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
        let mut store = Self::session_file_path().map_or_else(Self::default, Self::load_from);
        // Only an explicit host override redirects cache storage beside settings.
        store.storage_path = None;
        store
    }

    /// Loads settings from an explicit file and keeps subsequent saves there.
    #[must_use]
    pub fn load_from(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let mut store = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Self>(&bytes).ok())
            .filter(Self::is_valid)
            .unwrap_or_default();
        store.source_font_size = if store.source_font_size.is_finite() {
            store.source_font_size.clamp(8.0, 32.0)
        } else {
            default_source_font_size()
        };
        store.storage_path = Some(path);
        store
    }

    /// Returns the directory containing this host's reader settings and bookmarks.
    pub fn config_directory(&self) -> Option<PathBuf> {
        self.storage_path
            .clone()
            .or_else(Self::session_file_path)
            .and_then(|path| path.parent().map(Path::to_path_buf))
    }

    /// Returns a stable private cache directory for an attached repository.
    pub fn repository_cache(&self, root: &Path) -> Option<PathBuf> {
        use std::hash::{Hash, Hasher};
        let mut identity = std::collections::hash_map::DefaultHasher::new();
        root.hash(&mut identity);
        let cache = match &self.storage_path {
            Some(path) => path.parent()?.join("cache"),
            None => dirs::cache_dir()?.join("radiology"),
        };
        Some(
            cache
                .join("repositories")
                .join(format!("{:x}", identity.finish())),
        )
    }

    /// Saves the current session state to disk if valid.
    pub fn save(&self) {
        if !self.is_valid() {
            return;
        }
        if let Some(path) = self.storage_path.clone().or_else(Self::session_file_path) {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
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
