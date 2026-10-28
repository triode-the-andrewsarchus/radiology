//! Project strategies and tool invocations for semantic index generation.
//!
//! Planning is pure and uses the repository inventory rather than reader filters.
//! No command is executed and no repository file is changed during inspection.

pub mod cache;
pub mod command;
pub mod inventory;
pub mod process;
pub mod service;
pub mod tools;
pub mod workspace;

use crate::source::location::SourceLocation;
use std::collections::BTreeSet;
use std::path::Path;

/// Indexers supported by the Swift implementation, in registry priority order.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Indexer {
    /// Rust Cargo projects.
    Rust,
    /// Go modules.
    Go,
    /// JavaScript and TypeScript projects.
    TypeScript,
    /// Python packages and environments.
    Python,
    /// C/C++ compilation databases.
    Clang,
    /// Compiler-resolved Gradle Kotlin and Java projects.
    Kotlin,
}

impl Indexer {
    /// Returns the persistent adapter identifier.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Rust => "rust-analyzer",
            Self::Go => "scip-go",
            Self::TypeScript => "scip-typescript",
            Self::Python => "scip-python",
            Self::Clang => "scip-clang",
            Self::Kotlin => "scip-kotlin",
        }
    }

    /// Returns the languages covered by this adapter.
    #[must_use]
    pub fn languages(self) -> &'static [&'static str] {
        match self {
            Self::Rust => &["Rust"],
            Self::Go => &["Go"],
            Self::TypeScript => &["TypeScript", "JavaScript"],
            Self::Python => &["Python"],
            Self::Clang => &["C", "C++"],
            Self::Kotlin => &["Kotlin", "Java"],
        }
    }

    fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Rust => &["rs"],
            Self::Go => &["go"],
            Self::TypeScript => &["ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"],
            Self::Python => &["py", "pyi"],
            Self::Clang => &["c", "h", "cpp", "cc", "cxx", "hpp", "hh", "hxx"],
            Self::Kotlin => &["kt", "java"],
        }
    }

    /// Returns whether a path's extension belongs to this adapter's source languages.
    #[must_use]
    pub fn covers_path(self, path: &str) -> bool {
        Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                self.extensions()
                    .contains(&extension.to_ascii_lowercase().as_str())
            })
    }

    /// Returns the column encoding when the index omits an explicit encoding.
    #[must_use]
    pub fn default_encoding(self) -> crate::source::ColumnEncoding {
        use crate::source::ColumnEncoding::{Utf16, Utf8};
        match self {
            Self::TypeScript | Self::Python | Self::Kotlin => Utf16,
            _ => Utf8,
        }
    }
}

/// One adapter applied to a repository-relative project directory.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IndexingStrategy {
    /// Tool adapter selected for this project.
    pub indexer: Indexer,
    /// Repository-relative directory; empty means the root.
    pub directory: String,
    /// Actual manifest paths that justify this strategy.
    pub evidence: Vec<String>,
    /// Source files under this directory with a covered extension.
    pub source_files: usize,
}

impl IndexingStrategy {
    /// Returns an unambiguous persistent project identity.
    #[must_use]
    pub fn id(&self) -> String {
        if self.directory.is_empty() {
            self.indexer.id().into()
        } else {
            format!("{}@{}", self.indexer.id(), self.directory)
        }
    }
}

/// Deterministic bounded plan, including projects beyond the launch cap.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IndexPlan {
    /// Projects eligible to run, ordered by depth then adapter then path.
    pub selected: Vec<IndexingStrategy>,
    /// Projects omitted because the plan reached its run limit.
    pub skipped: Vec<IndexingStrategy>,
}

impl IndexPlan {
    /// Inspects an unfiltered repository inventory using Swift's 32-project cap.
    #[must_use]
    pub fn inspect<'a>(paths: impl IntoIterator<Item = &'a str>) -> Self {
        Self::with_limit(paths, 32)
    }

    /// Inspects with an explicit run cap, retaining all skipped evidence.
    #[must_use]
    pub fn with_limit<'a>(paths: impl IntoIterator<Item = &'a str>, limit: usize) -> Self {
        let inventory: BTreeSet<_> = paths
            .into_iter()
            .filter(|p| SourceLocation::is_repository_path(p))
            .collect();
        let mut selected = Vec::new();
        for (indexer, markers) in [
            (Indexer::Rust, &["Cargo.toml"][..]),
            (Indexer::Go, &["go.mod"][..]),
            (
                Indexer::Python,
                &[
                    "pyproject.toml",
                    "setup.py",
                    "setup.cfg",
                    "requirements.txt",
                ][..],
            ),
            (
                Indexer::Kotlin,
                &[
                    "settings.gradle",
                    "settings.gradle.kts",
                    "build.gradle",
                    "build.gradle.kts",
                ][..],
            ),
        ] {
            add_projects(&mut selected, &inventory, indexer, markers, true);
        }
        add_projects(
            &mut selected,
            &inventory,
            Indexer::TypeScript,
            &["tsconfig.json"],
            false,
        );
        let configured: Vec<_> = selected
            .iter()
            .filter(|s| s.indexer == Indexer::TypeScript)
            .map(|s| s.directory.clone())
            .collect();
        let mut packages = Vec::new();
        add_projects(
            &mut packages,
            &inventory,
            Indexer::TypeScript,
            &["package.json"],
            true,
        );
        selected.extend(
            packages
                .into_iter()
                .filter(|s| !configured.iter().any(|root| contains(root, &s.directory))),
        );
        if let Some(database) = inventory
            .iter()
            .filter(|p| name(p) == "compile_commands.json")
            .min_by_key(|p| (p.split('/').count(), **p))
        {
            selected.push(IndexingStrategy {
                indexer: Indexer::Clang,
                directory: String::new(),
                evidence: vec![(*database).into()],
                source_files: source_count(&inventory, Indexer::Clang, ""),
            });
        }
        selected.sort_by(|a, b| {
            (depth(&a.directory), a.indexer, &a.directory).cmp(&(
                depth(&b.directory),
                b.indexer,
                &b.directory,
            ))
        });
        let skipped = selected.split_off(limit.min(selected.len()));
        Self { selected, skipped }
    }
}

fn name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn depth(directory: &str) -> usize {
    if directory.is_empty() {
        0
    } else {
        directory.split('/').count()
    }
}

fn contains(directory: &str, path: &str) -> bool {
    directory.is_empty()
        || path == directory
        || path
            .strip_prefix(directory)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn source_count(paths: &BTreeSet<&str>, indexer: Indexer, directory: &str) -> usize {
    paths
        .iter()
        .filter(|path| contains(directory, path) && indexer.covers_path(path))
        .count()
}

fn add_projects(
    result: &mut Vec<IndexingStrategy>,
    paths: &BTreeSet<&str>,
    indexer: Indexer,
    markers: &[&str],
    require_sources: bool,
) {
    let mut directories: Vec<_> = paths
        .iter()
        .filter(|path| markers.contains(&name(path)))
        .map(|p| p.rsplit_once('/').map_or("", |(directory, _)| directory))
        .collect();
    directories.sort_by(|a, b| (depth(a), a).cmp(&(depth(b), b)));
    directories.dedup();
    let mut roots: Vec<&str> = Vec::new();
    for directory in directories {
        if roots.iter().any(|root| contains(root, directory)) {
            continue;
        }
        // Suppress nested manifests even when a parent has no matching sources,
        // matching the inventory's top-level project selection.
        roots.push(directory);
        let count = source_count(paths, indexer, directory);
        if require_sources && count == 0 {
            continue;
        }
        let evidence = paths
            .iter()
            .filter(|path| {
                markers.contains(&name(path))
                    && path.rsplit_once('/').map_or("", |(parent, _)| parent) == directory
            })
            .map(|p| (*p).into())
            .collect();
        result.push(IndexingStrategy {
            indexer,
            directory: directory.into(),
            evidence,
            source_files: count,
        });
    }
}

#[cfg(test)]
mod tests;
