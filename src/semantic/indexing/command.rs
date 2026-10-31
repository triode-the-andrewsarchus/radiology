//! Structured indexer invocations; repository paths never become shell source.

use super::{Indexer, IndexingStrategy};
use crate::source::location::SourceLocation;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::Path;

/// Indexer invocation using the shared literal subprocess description.
pub use crate::process::ProcessCommand as IndexCommand;

/// Runtime inputs for a single isolated indexing run.
pub struct CommandPaths<'a> {
    /// Root of the input workspace, with repository-relative paths preserved.
    pub workspace: &'a Path,
    /// Original repository root supplying stable project identity.
    pub source_root: &'a Path,
    /// Executable selected by a successful version probe.
    pub executable: &'a Path,
    /// Absolute output index path, outside the viewed repository.
    pub output: &'a Path,
    /// Private per-run directory for auxiliary files.
    pub run: &'a Path,
    /// Optional fallback Gradle when a project has no executable wrapper.
    pub gradle: Option<&'a Path>,
    /// Prepared C/C++ compilation database, exclusively under the private run.
    pub compilation_database: Option<&'a Path>,
    /// Explicit Cargo feature cfg overrides discovered from isolated manifests.
    pub rust_features: &'a [String],
}

impl IndexingStrategy {
    /// Builds the adapter's invocation without running a process or writing files.
    ///
    /// The caller materializes resources in the private run directory before
    /// launch. The workspace must be isolated from the viewed repository.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid project/evidence paths or non-absolute run paths.
    pub fn command(&self, paths: CommandPaths<'_>) -> Result<IndexCommand, String> {
        if !self.directory.is_empty() && !SourceLocation::is_repository_path(&self.directory) {
            return Err("Invalid repository-relative project directory".into());
        }
        if !paths.workspace.is_absolute()
            || !paths.source_root.is_absolute()
            || !paths.output.is_absolute()
            || !paths.run.is_absolute()
        {
            return Err("Index source, workspace, output and run paths must be absolute".into());
        }
        if self
            .evidence
            .iter()
            .any(|p| !SourceLocation::is_repository_path(p))
        {
            return Err("Invalid repository-relative strategy evidence".into());
        }
        let project = paths.workspace.join(&self.directory);
        let output = paths.output.as_os_str().to_owned();
        let mut command = IndexCommand {
            executable: paths.executable.into(),
            arguments: Vec::new(),
            directory: project.clone(),
            environment: BTreeMap::new(),
            resources: Vec::new(),
        };
        command.arguments = match self.indexer {
            Indexer::Rust => {
                let configuration = paths.run.join("rust-analyzer.json");
                let mut cfgs = vec!["debug_assertions".to_owned(), "miri".to_owned()];
                cfgs.extend(
                    paths
                        .rust_features
                        .iter()
                        .map(|name| format!("feature={name}")),
                );
                command.resources.push((configuration.clone(), serde_json::to_vec(&serde_json::json!({
                    "cargo": {"buildScripts": {"enable": false}, "features": "all", "cfgs": cfgs},
                    "procMacro": {"enable": false}
                })).expect("serializable rust-analyzer configuration")));
                vec![
                    "scip".into(),
                    project.into_os_string(),
                    "--output".into(),
                    output,
                    "--exclude-vendored-libraries".into(),
                    "--config-path".into(),
                    configuration.into_os_string(),
                ]
            }
            Indexer::Go => vec!["--output".into(), output],
            Indexer::TypeScript => {
                let mut arguments = vec![
                    "index".into(),
                    "--output".into(),
                    output,
                    "--no-progress-bar".into(),
                ];
                if self.evidence.iter().any(|p| p.ends_with("package.json")) {
                    arguments.push("--infer-tsconfig".into());
                }
                arguments
            }
            Indexer::Python => vec![
                "index".into(),
                ".".into(),
                "--project-name".into(),
                paths
                    .source_root
                    .join(&self.directory)
                    .file_name()
                    .unwrap_or_default()
                    .to_owned(),
                "--output".into(),
                output,
                "--quiet".into(),
            ],
            Indexer::Clang => {
                let database = paths.compilation_database
                    .filter(|database| database.is_absolute() && database.starts_with(paths.run)
                            && !database.components().any(|part| part == std::path::Component::ParentDir))
                    .ok_or("C/C++ indexing requires a rebased database under the private run directory")?;
                vec![
                    "--compdb-path".into(),
                    database.as_os_str().to_owned(),
                    "--index-output-path".into(),
                    output,
                    "--no-progress-report".into(),
                ]
            }
            Indexer::Kotlin => kotlin_command(&mut command, &paths),
        };
        Ok(command)
    }
}

fn kotlin_command(command: &mut IndexCommand, paths: &CommandPaths<'_>) -> Vec<OsString> {
    let script = paths.run.join("kotlin-index.sh");
    let init = paths.run.join("kotlin-index.gradle");
    // Reuse the existing compiler integration byte for byte. The script extracts
    // plugins from the selected CLI distribution when these paths do not exist.
    command.resources.extend([
        (
            script.clone(),
            include_bytes!("../../../Sources/Radiology/IndexingScripts/kotlin-index.sh").to_vec(),
        ),
        (
            init.clone(),
            include_bytes!("../../../Sources/Radiology/IndexingScripts/kotlin-index.gradle")
                .to_vec(),
        ),
    ]);
    command.executable = "/bin/sh".into();
    vec![
        script.into_os_string(),
        paths.executable.into(),
        paths.output.into(),
        init.into_os_string(),
        paths.run.join("scip-kotlinc.jar").into_os_string(),
        paths.run.join("scip-plugin.jar").into_os_string(),
        paths.gradle.map_or_else(OsString::new, |path| path.into()),
    ]
}
