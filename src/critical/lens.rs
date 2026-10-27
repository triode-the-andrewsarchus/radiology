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
                "Recede runtime plumbing so repository, system, and FFI operations stand out."
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

/// The four header fields of a SCIP symbol string (`scheme manager package version`).
///
/// Double spaces inside a field are escapes and never split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SCIPSymbolHeader {
    /// SCIP indexer scheme (e.g. `rust-analyzer`).
    pub scheme: String,
    /// Package manager name (e.g. `cargo`).
    pub manager: String,
    /// Package or crate name (e.g. `libc` or `.` for unowned).
    pub package: String,
    /// Package version string.
    pub version: String,
}

impl SCIPSymbolHeader {
    /// Parses a SCIP symbol string into header components.
    ///
    /// Returns `None` for `local N` symbols or malformed strings.
    #[must_use]
    pub fn parse(symbol: &str) -> Option<Self> {
        if symbol.starts_with("local ") {
            return None;
        }
        let mut fields = Vec::with_capacity(4);
        let mut current = String::new();
        let mut chars = symbol.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == ' ' {
                if chars.peek() == Some(&' ') {
                    chars.next();
                    current.push(' ');
                    continue;
                }
                fields.push(current);
                current = String::new();
                if fields.len() == 3 {
                    break;
                }
            } else {
                current.push(ch);
            }
        }

        // The remainder is the version plus descriptor, but the 4th field is up to the next unescaped space.
        let mut version = String::new();
        while let Some(ch) = chars.next() {
            if ch == ' ' {
                if chars.peek() == Some(&' ') {
                    chars.next();
                    version.push(' ');
                    continue;
                }
                break;
            }
            version.push(ch);
        }
        fields.push(version);

        if fields.len() != 4 {
            return None;
        }

        Some(Self {
            scheme: fields.remove(0),
            manager: fields.remove(0),
            package: fields.remove(0),
            version: fields.remove(0),
        })
    }

    /// The package name alone: `.` is SCIP's placeholder for "no package".
    #[must_use]
    pub fn package_name(&self) -> Option<&str> {
        if self.package.is_empty() || self.package == "." {
            None
        } else {
            Some(&self.package)
        }
    }

    /// `scheme manager name`, without the version: the key for package-level decisions.
    #[must_use]
    pub fn package_key(&self) -> Option<String> {
        self.package_name()
            .map(|name| format!("{} {} {}", self.scheme, self.manager, name))
    }
}

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
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
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

fn default_format_version() -> u32 {
    1
}

impl CriticalityOverrides {
    /// Returns `true` if the overrides contain no symbol or package rules.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty() && self.packages.is_empty()
    }

    /// Sets or clears a manual symbol criticality override.
    pub fn set_symbol(&mut self, symbol: &str, value: Option<Criticality>) {
        if !symbol.starts_with("local ") && !symbol.is_empty() {
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
/// let classification = CriticalityPolicy::classify_symbol("clone", true);
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
        let descriptors = symbol.rsplit(' ').next().unwrap_or("");

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

    /// Classifies a symbol name based on repository ownership and runtime plumbing heuristics.
    #[must_use]
    pub fn classify_symbol(name: &str, is_repo_defined: bool) -> CriticalityClassification {
        let plumbing = [
            "clone",
            "to_string",
            "into",
            "from",
            "as_ref",
            "borrow",
            "deref",
            "default",
            "fmt",
            "display",
            "debug",
            "drop",
            "hash",
            "eq",
        ];

        if plumbing.contains(&name.to_lowercase().as_str()) {
            return CriticalityClassification {
                criticality: Criticality::Ordinary,
                reason: CriticalityReason::RuntimePlumbing,
            };
        }

        if is_repo_defined {
            return CriticalityClassification {
                criticality: Criticality::Critical,
                reason: CriticalityReason::RepositoryDefinition,
            };
        }

        if name.starts_with("sys_") || name.starts_with("libc::") || name.starts_with("syscall") {
            return CriticalityClassification {
                criticality: Criticality::Critical,
                reason: CriticalityReason::SystemApi,
            };
        }

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
    /// Disjoint, sorted byte ranges that recede in opacity.
    pub receded_byte_ranges: Vec<Range<usize>>,
}

impl SourceProjection {
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
        self.receded_byte_ranges
            .iter()
            .any(|r| r.contains(&byte_offset))
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
    }

    #[test]
    fn test_policy_precedence() {
        let mut policy = CriticalityPolicy::default();
        policy.repository_packages.insert("my_crate".to_string());

        // FFI package
        let ffi_sym = "rust-analyzer cargo libc 0.2.147 io/read().";
        let c_ffi = policy.classify(ffi_sym, None, false);
        assert_eq!(c_ffi.criticality, Criticality::Critical);
        assert_eq!(c_ffi.reason, CriticalityReason::FfiPackage);

        // System API
        let sys_sym = "rust-analyzer cargo std 1.70.0 fs/File#open().";
        let c_sys = policy.classify(sys_sym, None, false);
        assert_eq!(c_sys.criticality, Criticality::Critical);
        assert_eq!(c_sys.reason, CriticalityReason::SystemApi);

        // Runtime plumbing
        let plumb_sym = "rust-analyzer cargo core 1.70.0 clone/Clone#clone().";
        let c_plumb = policy.classify(plumb_sym, None, false);
        assert_eq!(c_plumb.criticality, Criticality::Ordinary);
        assert_eq!(c_plumb.reason, CriticalityReason::RuntimePlumbing);

        // Repository crate
        let repo_sym = "rust-analyzer cargo my_crate 0.1.0 engine/start().";
        let c_repo = policy.classify(repo_sym, None, false);
        assert_eq!(c_repo.criticality, Criticality::Critical);
        assert_eq!(c_repo.reason, CriticalityReason::RepositoryDefinition);

        // User override beats policy
        policy
            .overrides
            .set_symbol(plumb_sym, Some(Criticality::Critical));
        let c_over = policy.classify(plumb_sym, None, false);
        assert_eq!(c_over.criticality, Criticality::Critical);
        assert_eq!(c_over.reason, CriticalityReason::SymbolOverride);
    }

    #[test]
    fn test_criticality_classification() {
        let c1 = CriticalityPolicy::classify_symbol("clone", true);
        assert_eq!(c1.criticality, Criticality::Ordinary);
        assert_eq!(c1.reason, CriticalityReason::RuntimePlumbing);

        let c2 = CriticalityPolicy::classify_symbol("processPayment", true);
        assert_eq!(c2.criticality, Criticality::Critical);
        assert_eq!(c2.reason, CriticalityReason::RepositoryDefinition);

        let c3 = CriticalityPolicy::classify_symbol("sys_open", false);
        assert_eq!(c3.criticality, Criticality::Critical);
        assert_eq!(c3.reason, CriticalityReason::SystemApi);
    }
}
