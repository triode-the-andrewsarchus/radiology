use crate::repository::classifier::{SourceFileClassifier, TestPathClassifier};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryFile {
    pub path: String,
    pub is_source: bool,
    pub is_test: bool,
}

impl RepositoryFile {
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
}

/// A node in the hierarchical repository file tree (directory or file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub children: Vec<FileNode>,
    pub file: Option<RepositoryFile>,
}

impl FileNode {
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

    pub fn new_dir(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            is_directory: true,
            children: Vec::new(),
            file: None,
        }
    }

    /// Builds a sorted tree hierarchy from a slice of files.
    /// Directories appear first, ordered alphabetically. Files follow, ordered alphabetically.
    pub fn build_tree(files: &[RepositoryFile]) -> Vec<FileNode> {
        // Internal builder tree representation
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
                        NodeItem::File(_) => break, // unexpected conflict
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

            dirs.extend(leaf_files);
            dirs
        }

        convert_map(root_dirs, "")
    }

    /// Recursively counts all files within this node.
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
        assert_eq!(tree.len(), 3); // src/ (dir), Cargo.toml, README.md
        assert_eq!(tree[0].name, "src");
        assert!(tree[0].is_directory);
        assert_eq!(tree[0].children.len(), 2); // source/ (dir), main.rs
        assert_eq!(tree[1].name, "Cargo.toml");
        assert_eq!(tree[2].name, "README.md");
    }
}
