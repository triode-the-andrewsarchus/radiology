//! Git and filesystem directory discovery for repository files.

use crate::repository::file_node::RepositoryFile;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Enumerates files in a repository using Git commands when available, with a walkdir fallback.
///
/// Excludes common build, dependency, and tool directories such as `.git`, `target`,
/// and `node_modules`.
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::Path;
/// use radiology::repository::discovery::RepositoryDiscovery;
///
/// let root = RepositoryDiscovery::find_root(Path::new("."));
/// let files = RepositoryDiscovery::discover_files(&root).unwrap();
/// println!("Discovered {} files", files.len());
/// ```
pub struct RepositoryDiscovery;

impl RepositoryDiscovery {
    /// Directory names excluded from repository browsing and file enumeration.
    pub const EXCLUDED_DIRS: &'static [&'static str] = &[
        ".git",
        ".build",
        "build",
        "dist",
        "node_modules",
        "vendor",
        "DerivedData",
        "target",
        ".DS_Store",
        ".cargo",
        ".gemini",
        ".idea",
        ".vscode",
    ];

    /// Locates the canonical repository root, checking `git rev-parse --show-toplevel`
    /// or looking for a parent `.git` directory, falling back to the canonicalized path.
    #[must_use]
    pub fn find_root(path: &Path) -> PathBuf {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        if let Ok(output) = Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .current_dir(&canonical)
            .output()
        {
            if output.status.success() {
                if let Ok(root_str) = String::from_utf8(output.stdout) {
                    let trimmed = root_str.trim();
                    if !trimmed.is_empty() {
                        return PathBuf::from(trimmed);
                    }
                }
            }
        }

        let mut curr = canonical.clone();
        loop {
            if curr.join(".git").exists() {
                return curr;
            }
            if !curr.pop() {
                break;
            }
        }

        canonical
    }

    /// Discovers candidate repository files using `git ls-files -z` when available,
    /// falling back to filesystem traversal.
    ///
    /// # Errors
    ///
    /// Returns an [`std::io::Error`] if filesystem enumeration fails.
    pub fn discover_files(root: &Path) -> Result<Vec<RepositoryFile>, std::io::Error> {
        if !root.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Choose a folder to browse.",
            ));
        }

        if let Ok(files) = Self::discover_git_files(root) {
            return Ok(files);
        }

        Self::discover_walkdir(root)
    }

    fn discover_git_files(root: &Path) -> Result<Vec<RepositoryFile>, std::io::Error> {
        let output = Command::new("git")
            .args([
                "ls-files",
                "-z",
                "--cached",
                "--others",
                "--exclude-standard",
            ])
            .current_dir(root)
            .output()?;

        if !output.status.success() {
            return Err(std::io::Error::other("git ls-files failed"));
        }

        let mut seen = HashSet::new();
        let mut paths = Vec::new();

        for slice in output.stdout.split(|&b| b == 0) {
            if slice.is_empty() {
                continue;
            }
            if let Ok(path_str) = std::str::from_utf8(slice) {
                if Self::is_excluded_path(path_str) {
                    continue;
                }
                // Verify the file actually exists in the working tree (drops unstaged deleted files)
                if root.join(path_str).exists() && seen.insert(path_str.to_string()) {
                    paths.push(path_str.to_string());
                }
            }
        }

        paths.sort_by_key(|a| a.to_lowercase());
        Ok(paths.into_iter().map(RepositoryFile::new).collect())
    }

    fn discover_walkdir(root: &Path) -> Result<Vec<RepositoryFile>, std::io::Error> {
        let mut paths = Vec::new();

        let walker = WalkDir::new(root).follow_links(false).into_iter();
        for entry in walker.filter_entry(|e| !Self::is_excluded_dir_entry(e)) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Skip symbolic links (matches Swift enumerator.skipDescendants())
            if entry.file_type().is_symlink() {
                continue;
            }

            if entry.file_type().is_file() {
                if let Ok(rel) = entry.path().strip_prefix(root) {
                    if let Some(rel_str) = rel.to_str() {
                        if !Self::is_excluded_path(rel_str) {
                            paths.push(rel_str.to_string());
                        }
                    }
                }
            }
        }

        paths.sort_by_key(|a| a.to_lowercase());
        Ok(paths.into_iter().map(RepositoryFile::new).collect())
    }

    fn is_excluded_dir_entry(entry: &walkdir::DirEntry) -> bool {
        if entry.file_type().is_dir() {
            let name = entry.file_name().to_string_lossy();
            Self::EXCLUDED_DIRS.contains(&name.as_ref())
        } else {
            false
        }
    }

    /// Checks if a repository-relative path contains any excluded directory components.
    #[must_use]
    pub fn is_excluded_path(path_str: &str) -> bool {
        for part in path_str.split('/') {
            if Self::EXCLUDED_DIRS.contains(&part) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excluded_paths() {
        assert!(RepositoryDiscovery::is_excluded_path("target/debug/build"));
        assert!(RepositoryDiscovery::is_excluded_path(".git/config"));
        assert!(RepositoryDiscovery::is_excluded_path(
            "node_modules/pkg/index.js"
        ));
        assert!(!RepositoryDiscovery::is_excluded_path(
            "src/target_detector.rs"
        ));
        assert!(!RepositoryDiscovery::is_excluded_path("src/main.rs"));
    }
}
