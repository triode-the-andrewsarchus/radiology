//! Critical Lens mode and symbol criticality policies.

use serde::{Deserialize, Serialize};

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
pub struct CriticalityPolicy;

impl CriticalityPolicy {
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

#[cfg(test)]
mod tests {
    use super::*;

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
