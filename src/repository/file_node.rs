//! Hierarchical file tree structure and repository file metadata.

use crate::repository::classifier::{SourceFileClassifier, TestPathClassifier};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Stores metadata for a single repository file.
///
/// # Examples
///
/// ```rust
/// use radiology::repository::file_node::RepositoryFile;
///
/// let file = RepositoryFile::new("src/main.rs");
/// assert!(file.is_source);
/// assert!(!file.is_test);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryFile {
    /// Repository-relative file path.
    pub path: String,
    /// Whether this file contains source code or configuration.
    pub is_source: bool,
    /// Whether this file represents a test or test fixture.
    pub is_test: bool,
}

impl RepositoryFile {
    /// Creates a [`RepositoryFile`], automatically classifying whether it is source or test code.
    pub fn new(path: impl Into<String>) -> Self {
        let path = path.into();
        let is_source = SourceFileClassifier::is_source_file(&path);
        let is_test = TestPathClassifier::is_test_path(&path);
        Self {
            path,
            is_source,
            is_test,
        }
    }

    /// Returns the file name (last path component).
    #[must_use]
    pub fn name(&self) -> String {
        std::path::Path::new(&self.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&self.path)
            .to_string()
    }
}

/// Represents a directory or leaf file node in the repository tree hierarchy.
///
/// # Examples
///
/// ```rust
/// use radiology::repository::file_node::{FileNode, RepositoryFile};
///
/// let files = vec![
///     RepositoryFile::new("src/main.rs"),
///     RepositoryFile::new("Cargo.toml"),
/// ];
/// let tree = FileNode::build_tree(&files);
/// assert_eq!(tree.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileNode {
    /// Directory or file basename.
    pub name: String,
    /// Repository-relative path to this node.
    pub path: String,
    /// `true` if this node represents a directory.
    pub is_directory: bool,
    /// Child directory and file nodes.
    pub children: Vec<FileNode>,
    /// Associated [`RepositoryFile`] metadata if this is a leaf file.
    pub file: Option<RepositoryFile>,
}

impl FileNode {
    /// Creates a leaf file node.
    #[must_use]
    pub fn new_file(file: RepositoryFile) -> Self {
        let name = file
            .path
            .rsplit('/')
            .next()
            .unwrap_or(&file.path)
            .to_string();
        Self {
            name,
            path: file.path.clone(),
            is_directory: false,
            children: Vec::new(),
            file: Some(file),
        }
    }

    /// Creates a directory node.
    pub fn new_dir(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            is_directory: true,
            children: Vec::new(),
            file: None,
        }
    }

    /// Builds a sorted tree hierarchy from a slice of [`RepositoryFile`] records.
    ///
    /// Directories appear first in alphabetical order, followed by files in alphabetical order.
    #[must_use]
    pub fn build_tree(files: &[RepositoryFile]) -> Vec<FileNode> {
        enum NodeItem {
            File(RepositoryFile),
            Dir(BTreeMap<String, NodeItem>),
        }

        let mut root_dirs: BTreeMap<String, NodeItem> = BTreeMap::new();

        for file in files {
            let parts: Vec<&str> = file.path.split('/').collect();
            if parts.is_empty() {
                continue;
            }

            let mut current = &mut root_dirs;
            for (idx, &part) in parts.iter().enumerate() {
                if idx == parts.len() - 1 {
                    current.insert(part.to_string(), NodeItem::File(file.clone()));
                } else {
                    let entry = current
                        .entry(part.to_string())
                        .or_insert_with(|| NodeItem::Dir(BTreeMap::new()));
                    match entry {
                        NodeItem::Dir(sub) => {
                            current = sub;
                        }
                        NodeItem::File(_) => break,
                    }
                }
            }
        }

        fn convert_map(map: BTreeMap<String, NodeItem>, prefix: &str) -> Vec<FileNode> {
            let mut dirs = Vec::new();
            let mut leaf_files = Vec::new();

            for (name, item) in map {
                let full_path = if prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{prefix}/{name}")
                };

                match item {
                    NodeItem::Dir(sub) => {
                        let children = convert_map(sub, &full_path);
                        let mut dir_node = FileNode::new_dir(name, full_path);
                        dir_node.children = children;
                        dirs.push(dir_node);
                    }
                    NodeItem::File(f) => {
                        leaf_files.push(FileNode::new_file(f));
                    }
                }
            }

            dirs.sort_by_key(|a| a.name.to_lowercase());
            leaf_files.sort_by_key(|a| a.name.to_lowercase());
            dirs.extend(leaf_files);
            dirs
        }

        convert_map(root_dirs, "")
    }

    /// Recursively counts all files within this node and its descendants.
    #[must_use]
    pub fn file_count(&self) -> usize {
        if !self.is_directory {
            1
        } else {
            self.children.iter().map(|c| c.file_count()).sum()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tree() {
        let files = vec![
            RepositoryFile::new("src/main.rs"),
            RepositoryFile::new("src/source/buffer.rs"),
            RepositoryFile::new("Cargo.toml"),
            RepositoryFile::new("README.md"),
        ];

        let tree = FileNode::build_tree(&files);
        assert_eq!(tree.len(), 3);
        assert_eq!(tree[0].name, "src");
        assert!(tree[0].is_directory);
        assert_eq!(tree[0].children.len(), 2);
        assert_eq!(tree[1].name, "Cargo.toml");
        assert_eq!(tree[2].name, "README.md");
    }
}
