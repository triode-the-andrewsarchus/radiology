//! Private-tool discovery and cancellable executable version probes.

use super::command::IndexCommand;
use super::process;
use super::Indexer;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

/// Actual executable usability, rather than a filename's mere presence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolAvailability {
    /// The executable returned a successful, nonempty version response.
    Available {
        /// Absolute invocation path; symlink identity is retained for rustup.
        executable: PathBuf,
        /// First version line, bounded to 256 characters.
        version: String,
    },
    /// No usable executable was found or its probe failed.
    Unavailable {
        /// Located executable, if a probe was attempted.
        executable: Option<PathBuf>,
        /// Human-readable failure category.
        reason: String,
        /// Bounded stdout and stderr from an attempted probe.
        diagnostics: String,
    },
}

/// Explicit search context, usable by standalone readers, hosts and fixture tests.
#[derive(Debug, Clone)]
pub struct ToolDirectory {
    /// Radiology-owned tool root, when persistent storage is available.
    pub managed: Option<PathBuf>,
    /// User home used for standard Cargo and Go executable locations.
    pub home: Option<PathBuf>,
    /// Ordered executable search directories.
    pub search_path: Vec<PathBuf>,
}

impl ToolDirectory {
    /// Captures the host search environment without changing it.
    #[must_use]
    pub fn from_environment(managed: Option<PathBuf>) -> Self {
        let home = dirs::home_dir();
        let mut search_path: Vec<_> = std::env::var_os("PATH")
            .map_or_else(Vec::new, |path| std::env::split_paths(&path).collect());
        let mut additions = vec![
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/local/go/bin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/bin"),
        ];
        if let Some(home) = &home {
            additions.extend([home.join(".cargo/bin"), home.join("go/bin")]);
        }
        for path in additions {
            if !search_path.contains(&path) {
                search_path.push(path);
            }
        }
        Self {
            managed,
            home,
            search_path,
        }
    }

    /// Finds an executable in managed locations first, then conventional paths.
    #[must_use]
    pub fn locate(&self, indexer: Indexer) -> Option<PathBuf> {
        let name = if indexer == Indexer::Kotlin {
            "scip-java"
        } else {
            indexer.id()
        };
        let mut candidates = Vec::new();
        if let Some(root) = &self.managed {
            let relative = match indexer {
                Indexer::Go => "go/bin/scip-go",
                Indexer::TypeScript => "npm/scip-typescript/node_modules/.bin/scip-typescript",
                Indexer::Python => "npm/scip-python/node_modules/.bin/scip-python",
                Indexer::Kotlin => "kotlin-scip/distribution/bin/scip-java",
                Indexer::Rust => "bin/rust-analyzer",
                Indexer::Clang => "bin/scip-clang",
            };
            candidates.push(root.join(relative));
        }
        if let Some(home) = &self.home {
            match indexer {
                Indexer::Rust => candidates.push(home.join(".cargo/bin/rust-analyzer")),
                Indexer::Go => candidates.push(home.join("go/bin/scip-go")),
                _ => {}
            }
        }
        candidates.extend(self.search_path.iter().map(|root| root.join(name)));
        candidates
            .into_iter()
            .find(|path| executable(path))
            .and_then(|path| {
                // Do not canonicalize executable symlinks: rustup dispatches by argv[0].
                if path.is_absolute() {
                    Some(path)
                } else {
                    std::env::current_dir().ok().map(|root| root.join(path))
                }
            })
    }

    /// Returns child-only PATH overrides matching this lookup context.
    #[must_use]
    pub fn child_environment(&self) -> BTreeMap<std::ffi::OsString, std::ffi::OsString> {
        std::env::join_paths(&self.search_path)
            .ok()
            .map(|path| ("PATH".into(), path))
            .into_iter()
            .collect()
    }

    /// Probes one tool using the bounded runner and the provided working directory.
    #[must_use]
    pub fn probe(
        &self,
        indexer: Indexer,
        directory: &Path,
        cancel: &AtomicBool,
        timeout: Duration,
    ) -> ToolAvailability {
        let Some(executable) = self.locate(indexer) else {
            return ToolAvailability::Unavailable {
                executable: None,
                reason: format!(
                    "{} is not installed in the tool search locations",
                    indexer.id()
                ),
                diagnostics: String::new(),
            };
        };
        let command = IndexCommand {
            executable: executable.clone(),
            arguments: vec!["--version".into()],
            directory: directory.into(),
            environment: self.child_environment(),
            resources: Vec::new(),
        };
        match process::run(&command, cancel, timeout) {
            Ok(output) => {
                let version = output
                    .stdout
                    .text
                    .lines()
                    .chain(output.stderr.text.lines())
                    .map(str::trim)
                    .find(|line| !line.is_empty());
                let valid = output.succeeded()
                    && version.is_some()
                    && (indexer != Indexer::Rust
                        || output.stdout.text.starts_with("rust-analyzer"));
                if valid {
                    // The compiler script locates Kotlin plugin jars relative to
                    // the actual distribution, while rustup needs its proxy name.
                    let executable = if indexer == Indexer::Kotlin {
                        std::fs::canonicalize(&executable).unwrap_or(executable)
                    } else {
                        executable
                    };
                    ToolAvailability::Available {
                        executable,
                        version: version.unwrap().chars().take(256).collect(),
                    }
                } else {
                    ToolAvailability::Unavailable {
                        executable: Some(executable),
                        reason: if output.succeeded() {
                            "Version response was empty or did not identify the expected tool"
                                .into()
                        } else if let Some(code) = output.exit_code {
                            format!("Version check exited with status {code}")
                        } else {
                            format!(
                                "Version check terminated by signal {}",
                                output.signal.unwrap_or_default()
                            )
                        },
                        diagnostics: diagnostics(&output.stdout.text, &output.stderr.text),
                    }
                }
            }
            Err(failure) => ToolAvailability::Unavailable {
                executable: Some(executable),
                reason: format!("Version probe {failure}"),
                diagnostics: diagnostics(&failure.stdout.text, &failure.stderr.text),
            },
        }
    }
}

fn diagnostics(stdout: &str, stderr: &str) -> String {
    format!("stdout:\n{stdout}\n\nstderr:\n{stderr}")
}

#[cfg(unix)]
fn executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::{symlink, PermissionsExt};

    fn script(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, format!("#!/bin/sh\n{text}\n")).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn managed_tools_win_and_rustup_symlink_identity_is_preserved() {
        let root = tempfile::tempdir().unwrap();
        let tools = ToolDirectory {
            managed: Some(root.path().join("tools")),
            home: None,
            search_path: vec![root.path().join("bin")],
        };
        let managed = tools.managed.as_ref().unwrap().join("go/bin/scip-go");
        script(&managed, "printf 'managed-go 1.0'");
        script(&root.path().join("bin/scip-go"), "printf 'path-go 0.1'");
        assert_eq!(tools.locate(Indexer::Go), Some(managed));
        let proxy = root.path().join("bin/rustup");
        script(
            &proxy,
            "case \"$0\" in *rust-analyzer) printf 'rust-analyzer 1.0';; *) exit 1;; esac",
        );
        let link = root.path().join("bin/rust-analyzer");
        symlink(&proxy, &link).unwrap();
        assert_eq!(
            tools.probe(
                Indexer::Rust,
                root.path(),
                &AtomicBool::new(false),
                Duration::from_secs(5)
            ),
            ToolAvailability::Available {
                executable: link,
                version: "rust-analyzer 1.0".into()
            }
        );
    }

    #[test]
    fn missing_components_empty_responses_and_nonexecutables_are_unavailable() {
        let root = tempfile::tempdir().unwrap();
        let tools = ToolDirectory {
            managed: None,
            home: None,
            search_path: vec![root.path().into()],
        };
        assert!(matches!(
            tools.probe(
                Indexer::Rust,
                root.path(),
                &AtomicBool::new(false),
                Duration::from_secs(1)
            ),
            ToolAvailability::Unavailable {
                executable: None,
                ..
            }
        ));
        let path = root.path().join("rust-analyzer");
        script(
            &path,
            "printf \"error: Unknown binary 'rust-analyzer'\" >&2; exit 1",
        );
        assert!(
            matches!(tools.probe(Indexer::Rust, root.path(), &AtomicBool::new(false), Duration::from_secs(5)), ToolAvailability::Unavailable { diagnostics, .. } if diagnostics.contains("Unknown binary"))
        );
        script(&path, "exit 0");
        assert!(matches!(
            tools.probe(
                Indexer::Rust,
                root.path(),
                &AtomicBool::new(false),
                Duration::from_secs(5)
            ),
            ToolAvailability::Unavailable { .. }
        ));
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(tools.locate(Indexer::Rust).is_none());
    }

    #[test]
    fn probe_deadlines_and_stderr_versions_are_reported() {
        let root = tempfile::tempdir().unwrap();
        let tools = ToolDirectory {
            managed: None,
            home: None,
            search_path: vec![root.path().into()],
        };
        script(
            &root.path().join("scip-python"),
            "printf 'python-indexer 1.0' >&2",
        );
        let available = tools.probe(
            Indexer::Python,
            root.path(),
            &AtomicBool::new(false),
            Duration::from_secs(5),
        );
        assert!(
            matches!(&available, ToolAvailability::Available { version, .. } if version == "python-indexer 1.0"),
            "{available:?}"
        );
        script(&root.path().join("scip-python"), "while :; do :; done");
        assert!(
            matches!(tools.probe(Indexer::Python, root.path(), &AtomicBool::new(false), Duration::from_millis(50)), ToolAvailability::Unavailable { reason, .. } if reason.contains("timed out"))
        );
    }
}
