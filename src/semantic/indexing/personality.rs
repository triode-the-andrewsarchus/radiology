//! Editable indexing settings captured with their exact pipeline provenance.
//!
//! Loading is blocking and belongs on an existing host worker. This module does
//! not select a repository personality, execute tools or publish semantic data.
mod parser;
mod plan;
mod store;
pub use store::PersonalityStore;

use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

const MAX_BYTES: u64 = 2 * 1024 * 1024;

/// Built-in indexing behavior selected independently for each attached checkout.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Personality {
    /// Detects ordinary projects and available build settings.
    #[default]
    Default,
    /// Unions separate seL4 architecture/configuration compiler runs.
    Sel4,
}

impl Personality {
    /// Returns the stable settings-file and cache namespace identifier.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sel4 => "sel4",
        }
    }

    /// Returns the built-in picker label, available even when user settings fail.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Sel4 => "seL4",
        }
    }

    fn digest(self, bytes: &[u8]) -> String {
        let mut hash = Sha256::new();
        hash.update(bytes);
        if self == Self::Sel4 {
            for (name, contents) in pipeline_resources() {
                hash.update(name.as_bytes());
                hash.update(b"\0");
                hash.update(contents);
            }
        }
        format!("{:x}", hash.finalize())
    }

    /// Returns the bundled, editable template without reading files or running Python.
    #[must_use]
    pub const fn template(self) -> &'static [u8] {
        match self {
            Self::Default => {
                include_bytes!("../../../Sources/Radiology/IndexingPersonalities/default.toml")
            }
            Self::Sel4 => {
                include_bytes!("../../../Sources/Radiology/IndexingPersonalities/sel4.toml")
            }
        }
    }
}

/// Returns the shared seL4 runner resources in stable provenance order.
///
/// Hosts stage these bytes beside the captured TOML, never inside viewed source.
#[must_use]
pub const fn pipeline_resources() -> &'static [(&'static str, &'static [u8])] {
    &[
        (
            "sel4_pipeline.py",
            include_bytes!("../../../Sources/Radiology/IndexingScripts/sel4_pipeline.py"),
        ),
        (
            "sel4_prepare.py",
            include_bytes!("../../../Sources/Radiology/IndexingScripts/sel4_prepare.py"),
        ),
        (
            "sel4_index.py",
            include_bytes!("../../../Sources/Radiology/IndexingScripts/sel4_index.py"),
        ),
        (
            "sel4_sources.py",
            include_bytes!("../../../Sources/Radiology/IndexingScripts/sel4_sources.py"),
        ),
        (
            "sel4_scip_union.py",
            include_bytes!("../../../Sources/Radiology/IndexingScripts/sel4_scip_union.py"),
        ),
    ]
}

/// Immutable validated settings and the exact bytes used to identify one pipeline.
///
/// A host keeps this value for a whole run, including its digest and staged TOML.
/// Editing the user file cannot change an already captured value.
///
/// # Examples
/// ```
/// use radiology::semantic::indexing::personality::{Personality, Settings};
/// let settings = Settings::parse(Personality::Default, "default.toml".into(), Personality::Default.template())?;
/// assert_eq!(settings.project_limit(), 32);
/// assert_eq!(settings.clang_jobs(), 4);
/// assert!(settings.indexers().is_empty());
/// let plan = settings.plan(["main.c"]);
/// assert!(plan.selected[0].is_best_effort());
/// assert_eq!(plan.selected[0].personality_digest.as_deref(), Some(settings.digest()));
/// # Ok::<(), String>(())
/// ```
#[derive(Debug, Clone)]
pub struct Settings {
    personality: Personality,
    path: PathBuf,
    bytes: Arc<[u8]>,
    digest: String,
    name: String,
    summary: String,
    indexers: Vec<String>,
    project_limit: usize,
    clang_jobs: usize,
    bootstrap_python: Vec<String>,
}

impl Settings {
    /// Validates typed root TOML and captures its exact bytes and pipeline digest.
    ///
    /// Nested recipe syntax is parsed by TOML; recipe semantics belong to the shared
    /// Python runner. Default needs no Python executable to parse or load settings.
    ///
    /// # Errors
    /// Rejects oversized/non-UTF-8 TOML, duplicate or unknown root values, unsupported
    /// root syntax, mismatched identity/engine/schema and out-of-range limits.
    pub fn parse(personality: Personality, path: PathBuf, bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() as u64 > MAX_BYTES {
            return Err("Personality settings exceed the 2 MiB limit".into());
        }
        let text =
            std::str::from_utf8(bytes).map_err(|_| "Personality settings must be UTF-8 TOML")?;
        let document = toml_edit::Document::parse(text)
            .map_err(|error| format!("Invalid personality TOML: {error}"))?;
        let root = parser::Root::new(document.as_table(), text)?;
        root.validate_identity(personality)?;
        let indexers = root.strings("indexers")?;
        if personality == Personality::Sel4 && indexers != ["scip-sel4"] {
            return Err(
                "The seL4 personality requires the scip-sel4 configuration-union indexer".into(),
            );
        }
        Ok(Self {
            personality,
            path,
            bytes: bytes.into(),
            digest: personality.digest(bytes),
            name: root.string("name")?,
            summary: root.string("summary")?,
            indexers,
            project_limit: root.integer("project_limit", 1..=256)?,
            clang_jobs: root.integer("clang_jobs", 1..=64)?,
            bootstrap_python: root.bootstrap_python()?,
        })
    }

    /// Returns the selected built-in pipeline.
    #[must_use]
    pub fn personality(&self) -> Personality {
        self.personality
    }
    /// Returns the source settings location without rereading its contents.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Returns the pinned TOML bytes for staging beside compiler output.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Returns SHA-256 of the TOML and applicable shared pipeline resources.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
    /// Returns the user-editable display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the user-editable explanation.
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }
    /// Returns Default's adapter allowlist; an empty list permits all adapters.
    #[must_use]
    pub fn indexers(&self) -> &[String] {
        &self.indexers
    }
    /// Returns the validated project count limit, in 1–256.
    #[must_use]
    pub fn project_limit(&self) -> usize {
        self.project_limit
    }
    /// Returns the validated per-configuration Clang worker count, in 1–64.
    #[must_use]
    pub fn clang_jobs(&self) -> usize {
        self.clang_jobs
    }
    /// Returns ordered bootstrap Python candidates, without resolving or executing them.
    #[must_use]
    pub fn bootstrap_python(&self) -> &[String] {
        &self.bootstrap_python
    }
}

#[cfg(test)]
mod tests;
