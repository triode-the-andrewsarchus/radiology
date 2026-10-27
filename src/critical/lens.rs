use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SourceMode {
    #[default]
    Normal,
    Critical,
}

impl SourceMode {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Critical => "Critical",
        }
    }

    pub fn help(&self) -> &'static str {
        match self {
            Self::Normal => "Show the source as written.",
            Self::Critical => {
                "Recede runtime plumbing so repository, system, and FFI operations stand out."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Criticality {
    Critical,
    Ordinary,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CriticalityReason {
    SymbolOverride,
    PackageOverride,
    Local,
    FfiPackage,
    FfiSignature,
    SystemApi,
    RepositoryDefinition,
    RuntimePlumbing,
    External,
}

impl CriticalityReason {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriticalityClassification {
    pub criticality: Criticality,
    pub reason: CriticalityReason,
}

pub struct CriticalityPolicy;

impl CriticalityPolicy {
    pub fn classify_symbol(name: &str, is_repo_defined: bool) -> CriticalityClassification {
        // Known runtime plumbing methods
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

        // FFI / Sys call indicators
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
