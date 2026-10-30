//! Git and filesystem directory discovery for repository files.

use super::git::GitRead;
use crate::repository::{
    file_node::{FileNode, RepositoryFile},
    filters::SourceFilters,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{
    io,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
use walkdir::WalkDir;

/// A fully enumerated root, ready for publication by a host.
pub(crate) struct DiscoveredRepository {
    pub root: PathBuf,
    pub files: Vec<RepositoryFile>,
    pub tree: Vec<FileNode>,
    pub filters: SourceFilters,
}

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
        let cancel = AtomicBool::new(false);
        let git = GitRead {
            root: &canonical,
            executable: Path::new("git"),
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(30),
            lease: None,
        };
        Self::find_root_with(&git).unwrap_or(canonical)
    }

    /// Discovers candidate repository files using bounded Git reads or traversal.
    ///
    /// This synchronous API belongs on a worker. GUI hosts use background opening.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid folder, incomplete Git output, failed
    /// traversal, or work exceeding the 30-second deadline.
    pub fn discover_files(root: &Path) -> Result<Vec<RepositoryFile>, io::Error> {
        let cancel = AtomicBool::new(false);
        let git = GitRead {
            root,
            executable: Path::new("git"),
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(30),
            lease: None,
        };
        Self::discover_files_with(&git)
    }

    pub(crate) fn discover(
        path: &Path,
        filters: &SourceFilters,
        cancel: &AtomicBool,
    ) -> Result<DiscoveredRepository, io::Error> {
        Self::discover_with(
            path,
            filters,
            cancel,
            Path::new("git"),
            Duration::from_secs(30),
        )
    }

    fn discover_with(
        path: &Path,
        filters: &SourceFilters,
        cancel: &AtomicBool,
        executable: &Path,
        timeout: Duration,
    ) -> Result<DiscoveredRepository, io::Error> {
        let deadline = Instant::now() + timeout;
        let canonical = path.canonicalize()?;
        let git = GitRead {
            root: &canonical,
            executable,
            cancel,
            deadline,
            lease: None,
        };
        let root = Self::find_root_with(&git)?;
        let git = GitRead { root: &root, ..git };
        let files = Self::discover_files_with(&git)?;
        let tree = FileNode::build_tree(&filters.filter_files(&files));
        git.check().map_err(io::Error::other)?;
        Ok(DiscoveredRepository {
            root,
            files,
            tree,
            filters: filters.clone(),
        })
    }

    fn find_root_with(git: &GitRead<'_>) -> Result<PathBuf, io::Error> {
        git.check().map_err(io::Error::other)?;
        Self::require_directory(git.root)?;
        if let Some(bytes) = git
            .optional(&["rev-parse", "--show-toplevel"])
            .map_err(io::Error::other)?
        {
            let root = std::str::from_utf8(&bytes).map_err(io::Error::other)?;
            // Git appends exactly one newline; whitespace can belong to the path.
            let root = root.strip_suffix('\n').unwrap_or(root);
            if !root.is_empty() {
                return Ok(PathBuf::from(root));
            }
        }
        let mut root = git.root.to_path_buf();
        loop {
            git.check().map_err(io::Error::other)?;
            if root.join(".git").exists() {
                return Ok(root);
            }
            if !root.pop() {
                break;
            }
        }
        Ok(git.root.to_path_buf())
    }

    fn require_directory(root: &Path) -> Result<(), io::Error> {
        if root.is_dir() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Choose a folder to browse.",
            ))
        }
    }

    fn discover_files_with(git: &GitRead<'_>) -> Result<Vec<RepositoryFile>, io::Error> {
        git.check().map_err(io::Error::other)?;
        Self::require_directory(git.root)?;
        let paths = match git
            .optional(&[
                "ls-files",
                "-z",
                "--cached",
                "--others",
                "--exclude-standard",
            ])
            .map_err(io::Error::other)?
        {
            Some(bytes) => Self::git_paths(git, &bytes)?,
            None => Self::walk_paths(git)?,
        };
        let mut paths = paths;
        paths.sort_by_cached_key(|path| path.to_lowercase());
        let files = paths.into_iter().map(RepositoryFile::new).collect();
        git.check().map_err(io::Error::other)?;
        Ok(files)
    }

    fn git_paths(git: &GitRead<'_>, bytes: &[u8]) -> Result<Vec<String>, io::Error> {
        let mut seen = HashSet::new();
        let mut paths = Vec::new();
        for path in crate::pull_request::change::nul_records(bytes).map_err(io::Error::other)? {
            git.check().map_err(io::Error::other)?;
            if !crate::source::SourceLocation::is_repository_path(path) {
                return Err(io::Error::other("Invalid Git source path"));
            }
            if !Self::is_excluded_path(path)
                && git.root.join(path).is_file()
                && seen.insert(path.to_owned())
            {
                paths.push(path.to_owned());
            }
        }
        Ok(paths)
    }

    fn walk_paths(git: &GitRead<'_>) -> Result<Vec<String>, io::Error> {
        let mut paths = Vec::new();
        let walker = WalkDir::new(git.root).follow_links(false).into_iter();
        for entry in walker.filter_entry(|entry| !Self::is_excluded_dir_entry(entry)) {
            git.check().map_err(io::Error::other)?;
            let entry = entry.map_err(io::Error::other)?;
            if entry.file_type().is_file() {
                let rel = entry
                    .path()
                    .strip_prefix(git.root)
                    .map_err(io::Error::other)?;
                let path = rel
                    .to_str()
                    .ok_or_else(|| io::Error::other("Invalid UTF-8 source path"))?;
                if !Self::is_excluded_path(path) {
                    paths.push(path.to_owned());
                }
            }
        }
        Ok(paths)
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
mod tests;
