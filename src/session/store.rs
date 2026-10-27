use crate::repository::filters::SourceFilters;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStore {
    pub last_repository: Option<String>,
    pub recent_repositories: Vec<String>,
    pub active_pr: Option<usize>,
    pub source_filters: SourceFilters,
}

impl SessionStore {
    fn session_file_path() -> Option<PathBuf> {
        let dir = dirs::config_dir()?.join("radiology");
        let _ = std::fs::create_dir_all(&dir);
        Some(dir.join("session.json"))
    }

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

    pub fn save(&self) {
        if let Some(path) = Self::session_file_path() {
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(path, json);
            }
        }
    }

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
