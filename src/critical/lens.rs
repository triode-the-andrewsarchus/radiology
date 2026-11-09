//! Critical Lens mode and symbol criticality policies.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ops::Range;

/// Mode determining how source text is visually presented to the reader.
///
/// In [`SourceMode::Critical`], standard runtime plumbing recedes so repository
/// definitions, system APIs, and FFI boundaries stand out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SourceMode {
    /// Shows source text exactly as written.
    #[default]
    Normal,
    /// Recedes runtime plumbing and highlights meaningful domain and system operations.
    Critical,
}

impl SourceMode {
    /// Returns the user-facing title of this mode.
    #[must_use]
    pub fn title(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Critical => "Critical",
        }
    }

    /// Returns a short description of this mode's behavior.
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::Normal => "Show the source as written.",
            Self::Critical => {
                "Focus on call names; recede proven ordinary calls and surrounding source."
            }
        }
    }
}

/// Criticality level of a symbol for skimming unfamiliar code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Criticality {
    /// Meaningful operation that should be emphasized.
    Critical,
    /// Non-critical plumbing or boilerplate.
    Ordinary,
    /// Neutral, unclassified symbol.
    Unknown,
}

/// Explanatory reason behind a symbol's criticality classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CriticalityReason {
    /// Manually marked critical by user.
    SymbolOverride,
    /// Enclosing package marked by user.
    PackageOverride,
    /// Local variable or binding.
    Local,
    /// Foreign Function Interface (FFI) boundary.
    FfiPackage,
    /// Extern C/FFI function signature.
    FfiSignature,
    /// Operating system API or system call.
    SystemApi,
    /// Symbol defined within the browsed repository.
    RepositoryDefinition,
    /// Standard language runtime plumbing (clone, drop, fmt, etc.).
    RuntimePlumbing,
    /// External package with no specific profile.
    External,
}

impl CriticalityReason {
    /// Returns a concise description of the reason.
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::SymbolOverride => "marked by you",
            Self::PackageOverride => "package marked by you",
            Self::Local => "local binding",
            Self::FfiPackage => "FFI boundary",
            Self::FfiSignature => "extern signature",
            Self::SystemApi => "system API",
            Self::RepositoryDefinition => "defined in this repository",
            Self::RuntimePlumbing => "runtime plumbing",
            Self::External => "external, no profile",
        }
    }
}

/// Pair containing a [`Criticality`] level and its explanatory [`CriticalityReason`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriticalityClassification {
    /// Assigned criticality level.
    pub criticality: Criticality,
    /// Evidence or rule responsible for the assignment.
    pub reason: CriticalityReason,
}

pub use crate::semantic::index::SCIPSymbolHeader;

/// Conservative, deterministic knowledge about runtime packages and FFI bindings.
pub struct CriticalityProfiles;

impl CriticalityProfiles {
    /// Verified SCIP indexer scheme.
    pub const VERIFIED_SCHEME: &'static str = "rust-analyzer";

    /// Standard language runtime packages considered plumbing by default.
    pub const RUNTIME_PACKAGES: &'static [&'static str] = &["core", "alloc", "std"];

    /// Packages known to be native bindings or platform ABIs.
    pub const FFI_PACKAGES: &'static [&'static str] = &[
        "libc",
        "jni",
        "jni-sys",
        "windows-sys",
        "winapi",
        "ndk-sys",
        "core-foundation-sys",
        "objc",
        "objc2",
        "block2",
        "libsqlite3-sys",
        "openssl-sys",
        "libz-sys",
        "bzip2-sys",
        "zstd-sys",
        "lzma-sys",
        "curl-sys",
        "libgit2-sys",
        "libssh2-sys",
        "ffmpeg-sys-next",
        "v8",
        "deno_core",
    ];

    /// Returns `true` if `name` is a recognized FFI package.
    #[must_use]
    pub fn is_ffi_package(name: &str) -> bool {
        Self::FFI_PACKAGES.contains(&name)
    }

    /// Returns `true` if the descriptor text inside `package` points to an FFI boundary.
    #[must_use]
    pub fn is_ffi_descriptor(package: &str, descriptors: &str) -> bool {
        match package {
            "core" => descriptors.starts_with("ffi/"),
            "std" => descriptors.starts_with("ffi/") || descriptors.starts_with("os/"),
            _ => false,
        }
    }

    /// Returns `true` if the descriptor text inside `package` points to a recognized OS/system API.
    #[must_use]
    pub fn is_system_api(package: &str, descriptors: &str) -> bool {
        match package {
            "std" => {
                let prefixes = [
                    "fs/",
                    "net/",
                    "process/",
                    "env/",
                    "os/",
                    "thread/",
                    "io/",
                    "time/SystemTime",
                    "time/Instant",
                ];
                let exclusions = [
                    "io/error/",
                    "io/cursor/",
                    "io/buffered/",
                    "io/util/",
                    "io/impls/",
                ];
                prefixes.iter().any(|p| descriptors.starts_with(p))
                    && !exclusions.iter().any(|e| descriptors.starts_with(e))
            }
            "tokio" => {
                let prefixes = ["fs/", "net/", "process/", "signal/"];
                prefixes.iter().any(|p| descriptors.starts_with(p))
            }
            _ => false,
        }
    }
}

/// The reader's manual decisions for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriticalityOverrides {
    /// Format version number.
    #[serde(default = "default_format_version")]
    pub format_version: u32,
    /// Canonical repository root path.
    pub repository_root: String,
    /// Per-symbol decisions.
    #[serde(default)]
    pub symbols: HashMap<String, Criticality>,
    /// Per-package decisions keyed by `scheme manager name`.
    #[serde(default)]
    pub packages: HashMap<String, Criticality>,
}

impl Default for CriticalityOverrides {
    fn default() -> Self {
        Self {
            format_version: 1,
            repository_root: String::new(),
            symbols: HashMap::new(),
            packages: HashMap::new(),
        }
    }
}

fn default_format_version() -> u32 {
    1
}

impl CriticalityOverrides {
    /// Returns whether an identity can safely survive semantic reindexing.
    #[must_use]
    pub fn is_overridable(symbol: &str) -> bool {
        !symbol.is_empty() && !symbol.starts_with("local ")
    }

    /// Returns `true` if the overrides contain no symbol or package rules.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty() && self.packages.is_empty()
    }

    /// Sets or clears a manual symbol criticality override.
    pub fn set_symbol(&mut self, symbol: &str, value: Option<Criticality>) {
        if Self::is_overridable(symbol) {
            if let Some(val) = value {
                if val != Criticality::Unknown {
                    self.symbols.insert(symbol.to_string(), val);
                    return;
                }
            }
            self.symbols.remove(symbol);
        }
    }

    /// Sets or clears a manual package criticality override.
    pub fn set_package(&mut self, package_key: &str, value: Option<Criticality>) {
        if !package_key.is_empty() {
            if let Some(val) = value {
                if val != Criticality::Unknown {
                    self.packages.insert(package_key.to_string(), val);
                    return;
                }
            }
            self.packages.remove(package_key);
        }
    }
}

/// Policy evaluator classifying symbols for Critical Lens mode.
///
/// # Examples
///
/// ```rust
/// use radiology::critical::lens::{Criticality, CriticalityPolicy, CriticalityReason};
///
/// let classification = CriticalityPolicy::default()
///     .classify("rust-analyzer cargo core 0 clone().", None, false);
/// assert_eq!(classification.criticality, Criticality::Ordinary);
/// assert_eq!(classification.reason, CriticalityReason::RuntimePlumbing);
/// ```
#[derive(Debug, Clone, Default)]
pub struct CriticalityPolicy {
    /// Package names defined within the indexed repository.
    pub repository_packages: HashSet<String>,
    /// User manual overrides.
    pub overrides: CriticalityOverrides,
}

impl CriticalityPolicy {
    /// Creates a new policy with the given repository packages and manual overrides.
    #[must_use]
    pub fn new(repository_packages: HashSet<String>, overrides: CriticalityOverrides) -> Self {
        Self {
            repository_packages,
            overrides,
        }
    }

    /// Evaluates symbol criticality following the strict precedence table.
    #[must_use]
    pub fn classify(
        &self,
        symbol: &str,
        signature: Option<&str>,
        is_repo_defined: bool,
    ) -> CriticalityClassification {
        if symbol.starts_with("local ") {
            return CriticalityClassification {
                criticality: Criticality::Unknown,
                reason: CriticalityReason::Local,
            };
        }

        // 1. User symbol override
        if let Some(&decision) = self.overrides.symbols.get(symbol) {
            return CriticalityClassification {
                criticality: decision,
                reason: CriticalityReason::SymbolOverride,
            };
        }

        let header = SCIPSymbolHeader::parse(symbol);

        // 2. User package override
        if let Some(key) = header.as_ref().and_then(|h| h.package_key()) {
            if let Some(&decision) = self.overrides.packages.get(&key) {
                return CriticalityClassification {
                    criticality: decision,
                    reason: CriticalityReason::PackageOverride,
                };
            }
        }

        let package = header.as_ref().and_then(|h| h.package_name());
        let is_profiled = header
            .as_ref()
            .map(|h| h.scheme == CriticalityProfiles::VERIFIED_SCHEME)
            .unwrap_or(false);

        // Descriptors are the trailing characters after header fields.
        let descriptors = crate::semantic::index::SCIPSymbolString::descriptor_text(symbol);

        // 3. Profiled FFI packages or descriptors
        if is_profiled {
            if let Some(pkg) = package {
                if CriticalityProfiles::is_ffi_package(pkg)
                    || CriticalityProfiles::is_ffi_descriptor(pkg, descriptors)
                {
                    return CriticalityClassification {
                        criticality: Criticality::Critical,
                        reason: CriticalityReason::FfiPackage,
                    };
                }
            }
        }

        // 4. Extern C/FFI function signature
        if let Some(sig) = signature {
            if sig.contains("extern \"") {
                return CriticalityClassification {
                    criticality: Criticality::Critical,
                    reason: CriticalityReason::FfiSignature,
                };
            }
        }

        // 5. System API
        if is_profiled {
            if let Some(pkg) = package {
                if CriticalityProfiles::is_system_api(pkg, descriptors) {
                    return CriticalityClassification {
                        criticality: Criticality::Critical,
                        reason: CriticalityReason::SystemApi,
                    };
                }
            }
        }

        // 6. Repository definition
        if is_repo_defined
            || package
                .map(|p| self.repository_packages.contains(p))
                .unwrap_or(false)
        {
            return CriticalityClassification {
                criticality: Criticality::Critical,
                reason: CriticalityReason::RepositoryDefinition,
            };
        }

        // 7. Profiled runtime plumbing
        if is_profiled {
            if let Some(pkg) = package {
                if CriticalityProfiles::RUNTIME_PACKAGES.contains(&pkg) {
                    return CriticalityClassification {
                        criticality: Criticality::Ordinary,
                        reason: CriticalityReason::RuntimePlumbing,
                    };
                }
            }
        }

        // 8. Fallback
        CriticalityClassification {
            criticality: Criticality::Unknown,
            reason: CriticalityReason::External,
        }
    }
}

/// Defines disjoint, sorted byte ranges that recede in the viewer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProjection {
    /// Document path.
    pub path: String,
    revision: Option<crate::source::ReviewRevision>,
    content_digest: String,
    /// Disjoint, sorted byte ranges that recede in opacity.
    pub receded_byte_ranges: Vec<Range<usize>>,
}

impl SourceProjection {
    /// Creates a projection bound to exact bytes and revision, dropping invalid ranges.
    #[must_use]
    pub fn new(buffer: &crate::source::SourceBuffer, ranges: Vec<Range<usize>>) -> Self {
        let ranges = ranges
            .into_iter()
            .filter(|range| {
                range.end <= buffer.byte_len()
                    && buffer.text.is_char_boundary(range.start)
                    && buffer.text.is_char_boundary(range.end)
            })
            .collect();
        Self {
            path: buffer.path.clone(),
            revision: buffer.revision,
            content_digest: buffer.content_digest().to_owned(),
            receded_byte_ranges: Self::normalized(ranges),
        }
    }

    /// Returns whether this projection describes the exact displayed document.
    #[must_use]
    pub fn matches(&self, buffer: &crate::source::SourceBuffer) -> bool {
        self.path == buffer.path
            && self.revision == buffer.revision
            && self.content_digest == buffer.content_digest()
    }

    /// Normalizes, sorts, and merges overlapping or adjacent byte ranges.
    #[must_use]
    pub fn normalized(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
        ranges.retain(|r| !r.is_empty());
        ranges.sort_by_key(|r| r.start);
        let mut output: Vec<Range<usize>> = Vec::new();
        for range in ranges {
            if let Some(last) = output.last_mut() {
                if range.start <= last.end {
                    last.end = last.end.max(range.end);
                    continue;
                }
            }
            output.push(range);
        }
        output
    }

    /// Checks if a byte offset falls within any receded range.
    #[must_use]
    pub fn is_receded(&self, byte_offset: usize) -> bool {
        let position = self
            .receded_byte_ranges
            .partition_point(|range| range.end <= byte_offset);
        self.receded_byte_ranges
            .get(position)
            .is_some_and(|range| range.contains(&byte_offset))
    }
}

/// Completeness of reachable call graph operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountCompleteness {
    /// Fully resolved call tree.
    Proven,
    /// Bounded or partially resolved call tree.
    Partial,
    /// Declaration with no body to summarize.
    NoBody,
}

/// Reachable critical operation count annotation for viewer gutters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountAnnotation {
    /// Reachable critical count.
    pub count: usize,
    /// Completeness level.
    pub completeness: CountCompleteness,
}

impl CountAnnotation {
    /// Returns the concise gutter label (`0`, `3+`, or `—`).
    #[must_use]
    pub fn label(&self) -> String {
        match self.completeness {
            CountCompleteness::Proven => self.count.to_string(),
            CountCompleteness::Partial => format!("{}+", self.count),
            CountCompleteness::NoBody => "—".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scip_symbol_header_parsing() {
        let sym = "rust-analyzer cargo libc 0.2.147 io/read().";
        let header = SCIPSymbolHeader::parse(sym).expect("valid header");
        assert_eq!(header.scheme, "rust-analyzer");
        assert_eq!(header.manager, "cargo");
        assert_eq!(header.package, "libc");
        assert_eq!(header.version, "0.2.147");
        assert_eq!(header.package_name(), Some("libc"));
        assert_eq!(
            header.package_key(),
            Some("rust-analyzer cargo libc".to_string())
        );

        assert!(SCIPSymbolHeader::parse("local 12").is_none());
        let placeholder = SCIPSymbolHeader::parse("rust-analyzer cargo . . x/").unwrap();
        assert!(placeholder.package_name().is_none());
        let escaped = SCIPSymbolHeader::parse("a b  c d e f/").unwrap();
        assert_eq!(escaped.manager, "b c");
        let url = "rust-analyzer cargo std https://github.com/rust-lang/rust/library/std fs/OpenOptions#open().";
        assert!(SCIPSymbolHeader::parse(url)
            .unwrap()
            .version
            .starts_with("https://"));
        assert_eq!(
            crate::semantic::index::SCIPSymbolString::descriptor_text(url),
            "fs/OpenOptions#open()."
        );
        assert_eq!(
            SCIPSymbolHeader::parse("rust-analyzer cargo core . option/Option#unwrap().")
                .unwrap()
                .package_name(),
            Some("core")
        );
    }

    #[test]
    fn test_policy_precedence() {
        use crate::semantic::{ScipDocument, ScipIndex, ScipOccurrence, ScipRange, SemanticIndex};
        use Criticality::{Critical as C, Ordinary as O, Unknown as U};
        use CriticalityReason::*;
        let repository = "rust-analyzer cargo app 0.1.0 main/run().";
        let index = SemanticIndex::from_scip(ScipIndex {
            documents: vec![ScipDocument {
                relative_path: "src/main.rs".into(),
                occurrences: vec![ScipOccurrence {
                    symbol: repository.into(),
                    symbol_roles: 1,
                    range: ScipRange::from_packed(&[0, 3, 6]),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        });
        let policy = CriticalityPolicy::for_index(Some(&index), Default::default());
        assert_eq!(policy.repository_packages, HashSet::from(["app".into()]));
        for (symbol, criticality, reason) in [
            (repository, C, RepositoryDefinition),
            ("rust-analyzer cargo app 0.1.0 android/only().", C, RepositoryDefinition),
            ("rust-analyzer cargo core . option/Option#unwrap().", O, RuntimePlumbing),
            ("rust-analyzer cargo std https://github.com/rust-lang/rust/library/std fs/OpenOptions#open().", C, SystemApi),
            ("rust-analyzer cargo std . io/error/impl#[Error]last_os_error().", O, RuntimePlumbing),
            ("rust-analyzer cargo std . io/error/ErrorKind#WouldBlock#", O, RuntimePlumbing),
            ("rust-analyzer cargo std . io/buffered/bufreader/BufReader#new().", O, RuntimePlumbing),
            ("rust-analyzer cargo std . io/stdio/stdout().", C, SystemApi),
            ("rust-analyzer cargo std . io/Read#read().", C, SystemApi),
            ("rust-analyzer cargo std . os/fd/raw/impl#[`fs::File`][AsRawFd]as_raw_fd().", C, FfiPackage),
            ("rust-analyzer cargo libc 0.2.189 flock().", C, FfiPackage),
            ("rust-analyzer cargo libsqlite3-sys 0.35.0 sqlite3_open().", C, FfiPackage),
            ("rust-analyzer cargo mystery-sys 1.0.0 do_thing().", U, External),
            ("rust-analyzer cargo core . ptr/read().", O, RuntimePlumbing),
            ("rust-analyzer cargo serde_json 1.0.151 value/Value#as_object().", U, External),
            ("rust-analyzer cargo tokio 1.53.1 fs/File#open().", C, SystemApi),
            ("rust-analyzer cargo tokio 1.53.1 task/spawn().", U, External),
            ("scip-go gomod std go1.22 fmt/Println().", U, External),
            ("local 3", U, Local),
        ] {
            assert_eq!(policy.classify(symbol, None, symbol == repository),
                CriticalityClassification { criticality, reason }, "{symbol}");
        }
        assert_eq!(
            policy.classify(
                "rust-analyzer cargo app 0.1.0 ffi/v8__IsSandboxEnabled().",
                Some("unsafe extern \"C\" fn v8__IsSandboxEnabled() -> bool"),
                true
            ),
            CriticalityClassification {
                criticality: C,
                reason: FfiSignature
            }
        );
    }

    #[test]
    fn symbol_package_overrides_and_resets_preserve_builtin_precedence() {
        use Criticality::{Critical, Ordinary, Unknown};
        use CriticalityReason::*;
        let system = "rust-analyzer cargo std . fs/OpenOptions#open().";
        let ffi = "rust-analyzer cargo std . os/fd/AsRawFd#as_raw_fd().";
        let repository = "rust-analyzer cargo app 0.1.0 main/run().";
        let mut policy = CriticalityPolicy::new(HashSet::from(["app".into()]), Default::default());
        policy
            .overrides
            .set_package("rust-analyzer cargo std", Some(Ordinary));
        policy.overrides.set_symbol(system, Some(Critical));
        policy.overrides.set_symbol(repository, Some(Ordinary));
        policy.overrides.set_symbol("local 3", Some(Critical));
        assert!(!policy.overrides.symbols.contains_key("local 3"));
        for (symbol, criticality, reason) in [
            (ffi, Ordinary, PackageOverride),
            (system, Critical, SymbolOverride),
            (repository, Ordinary, SymbolOverride),
            ("local 3", Unknown, Local),
        ] {
            assert_eq!(
                policy.classify(symbol, None, symbol == repository),
                CriticalityClassification {
                    criticality,
                    reason
                }
            );
        }
        let builtin =
            CriticalityPolicy::new(policy.repository_packages.clone(), Default::default());
        assert_eq!(
            builtin.classify(repository, None, true).reason,
            RepositoryDefinition
        );
        policy.overrides.set_symbol(system, None);
        assert_eq!(policy.classify(system, None, false).reason, PackageOverride);
        policy
            .overrides
            .set_package("rust-analyzer cargo std", Some(Unknown));
        assert!(policy.overrides.packages.is_empty());
        assert_eq!(
            policy.classify(system, None, false),
            builtin.classify(system, None, false)
        );
        assert_eq!(policy.classify(ffi, None, false).reason, FfiPackage);
    }
}
