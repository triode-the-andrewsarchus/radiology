use super::{Personality, Settings, MAX_BYTES};
use crate::semantic::{
    indexing::{inventory::check_cancel, workspace::destination_path},
    loader::read_bytes_limited,
};
use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

/// App-owned editable templates with non-overwriting first-use installation.
///
/// Constructing a store and asking for a file path perform no I/O. Hosts load on
/// a worker and pass both attached checkout and pinned source roots as protected.
///
/// # Examples
/// ```
/// use radiology::semantic::indexing::personality::{Personality, PersonalityStore};
/// use std::sync::atomic::AtomicBool;
/// let owner = tempfile::tempdir()?;
/// let store = PersonalityStore::new(owner.path().join("settings"));
/// assert!(!store.path(Personality::Default).exists());
/// let settings = store.load(Personality::Default, &[], &AtomicBool::new(false))?;
/// assert_eq!(settings.bytes(), Personality::Default.template());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct PersonalityStore {
    directory: PathBuf,
}

impl PersonalityStore {
    /// Selects a private configuration directory without installing templates.
    #[must_use]
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    /// Returns the selected file location even if its contents are invalid.
    #[must_use]
    pub fn path(&self, personality: Personality) -> PathBuf {
        self.directory.join(format!("{}.toml", personality.id()))
    }

    /// Loads current settings, installing the bundled template only if absent.
    ///
    /// User edits, including invalid files, survive concurrent first-use loads.
    /// Cancellation never publishes settings; a completed template installation
    /// may remain if cancellation arrives during that atomic operation.
    ///
    /// # Errors
    /// Rejects storage inside protected repositories, missing protected roots,
    /// nonregular/symlink files, oversized/invalid settings and cancellation.
    pub fn load(
        &self,
        personality: Personality,
        protected_roots: &[PathBuf],
        cancel: &AtomicBool,
    ) -> Result<Settings, String> {
        check_cancel(cancel)?;
        let directory = destination_path(&self.directory)?;
        for root in protected_roots {
            if directory.starts_with(root.canonicalize().map_err(|error| error.to_string())?) {
                return Err(
                    "Personality settings must be stored outside viewed repositories".into(),
                );
            }
        }
        let path = directory.join(format!("{}.toml", personality.id()));
        let bytes = match read(&path, cancel)? {
            Some(bytes) => bytes,
            None => {
                check_cancel(cancel)?;
                crate::storage::create_if_missing(&path, personality.template())
                    .map_err(|error| error.to_string())?;
                read(&path, cancel)?.ok_or("Personality settings disappeared after installation")?
            }
        };
        check_cancel(cancel)?;
        let settings = Settings::parse(personality, path.clone(), &bytes)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        check_cancel(cancel)?;
        Ok(settings)
    }
}

fn read(path: &Path, cancel: &AtomicBool) -> Result<Option<Vec<u8>>, String> {
    read_bytes_limited(
        path,
        cancel,
        MAX_BYTES,
        "Personality settings exceed the 2 MiB limit",
    )
}
