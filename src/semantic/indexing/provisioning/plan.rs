use super::*;

pub(super) const KOTLIN_COMMIT: &str = "9f288dd339c7e9fc257a7b21714d9a402fd4d837";
pub(super) const KOTLIN_VERSION: &str = "0.13.1-radiology-9f288dd339c7";

/// One literal installation command, inspectable before execution.
#[derive(Debug, Clone)]
pub struct ProvisioningPlan {
    /// User-facing scope and pinned package identity.
    pub summary: String,
    /// Command runs in the private tool root, never the viewed repository.
    pub command: crate::process::ProcessCommand,
    /// Private directories the package manager requires before launch.
    pub directories: Vec<PathBuf>,
}

impl Indexer {
    /// Describes the installation scope, or manual prerequisite instructions.
    #[must_use]
    pub fn installation_hint(self) -> &'static str {
        match self {
            Self::Rust => "Add rust-analyzer to the active rustup toolchain. Requires rustup; this updates that toolchain.",
            Self::Go => "Install scip-go v0.2.7 in Radiology's tools directory. Requires Go and network access.",
            Self::TypeScript => "Install scip-typescript 0.4.0 in Radiology's tools directory. Requires Node.js 22+, npm and network access.",
            Self::Python => "Install scip-python 0.6.6 in Radiology's tools directory. Requires Node.js, npm, Python 3.10+ and network access.",
            Self::Kotlin => "Build the pinned Kotlin SCIP compiler indexer in Radiology's tools directory. Requires JDK 17+, curl, tar, unzip and network access.",
            Self::Clang => "Install scip-clang manually and make it available on PATH. Radiology does not install it automatically.",
        }
    }

    /// Builds the same pinned installation plans as Swift without executing them.
    ///
    /// # Errors
    /// Returns missing prerequisites, absent private storage or manual-only tooling.
    pub fn provisioning(self, tools: &ToolDirectory) -> Result<ProvisioningPlan, String> {
        let root = tools
            .managed
            .as_ref()
            .filter(|root| root.is_absolute())
            .ok_or("A private absolute tools directory is required")?;
        let required = |name| {
            tools
                .locate_program(name)
                .ok_or_else(|| format!("Missing {name}. {}", self.installation_hint()))
        };
        let mut environment = tools.child_environment();
        let mut resources = Vec::new();
        let mut directories = Vec::new();
        let (executable, arguments): (PathBuf, Vec<std::ffi::OsString>) = match self {
            Self::Rust => (
                required("rustup")?,
                vec!["component".into(), "add".into(), "rust-analyzer".into()],
            ),
            Self::Go => {
                let bin = root.join("go/bin");
                directories.push(bin.clone());
                environment.insert("GOBIN".into(), bin.into_os_string());
                environment.insert("GOFLAGS".into(), "-mod=mod".into());
                environment.insert("GOCACHE".into(), root.join("go/cache").into_os_string());
                environment.insert(
                    "GOMODCACHE".into(),
                    root.join("go/modules").into_os_string(),
                );
                (
                    required("go")?,
                    vec![
                        "install".into(),
                        "github.com/scip-code/scip-go/cmd/scip-go@v0.2.7".into(),
                    ],
                )
            }
            Self::TypeScript | Self::Python => {
                required("node")?;
                if self == Self::Python {
                    required("python3")?;
                }
                let prefix = root.join("npm").join(self.id());
                directories.push(prefix.clone());
                environment.insert(
                    "npm_config_cache".into(),
                    root.join("npm/cache").into_os_string(),
                );
                let package = if self == Self::TypeScript {
                    "@sourcegraph/scip-typescript@0.4.0"
                } else {
                    "@sourcegraph/scip-python@0.6.6"
                };
                (
                    required("npm")?,
                    vec![
                        "install".into(),
                        "--no-fund".into(),
                        "--no-audit".into(),
                        "--prefix".into(),
                        prefix.into_os_string(),
                        package.into(),
                    ],
                )
            }
            Self::Kotlin => {
                required("java")?;
                for name in ["curl", "tar", "unzip"] {
                    required(name)?;
                }
                let script = root.join("install-kotlin-indexer.sh");
                resources.push((
                    script.clone(),
                    include_bytes!(
                        "../../../../Sources/Radiology/IndexingScripts/install-kotlin-indexer.sh"
                    )
                    .to_vec(),
                ));
                environment.insert(
                    "GRADLE_USER_HOME".into(),
                    root.join("kotlin-scip/gradle-user-home").into_os_string(),
                );
                (
                    PathBuf::from("/bin/sh"),
                    vec![
                        script.into_os_string(),
                        root.join("kotlin-scip").into_os_string(),
                        KOTLIN_COMMIT.into(),
                        KOTLIN_VERSION.into(),
                        tools
                            .gradle()
                            .map_or_else(Default::default, PathBuf::into_os_string),
                        "9.7.1".into(),
                    ],
                )
            }
            Self::Clang => return Err(self.installation_hint().into()),
        };
        Ok(ProvisioningPlan {
            summary: self.installation_hint().into(),
            command: crate::process::ProcessCommand {
                executable,
                arguments,
                directory: root.clone(),
                environment,
                resources,
            },
            directories,
        })
    }
}
