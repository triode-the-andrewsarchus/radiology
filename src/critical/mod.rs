//! Critical Lens mode and criticality classification policies.
//!
//! Provides [`SourceMode`] (Normal vs Critical) and [`CriticalityPolicy`]
//! to distinguish boilerplate runtime plumbing from core domain logic, system APIs, and FFI boundaries.

pub mod lens;

pub use lens::{
    CountAnnotation, CountCompleteness, Criticality, CriticalityClassification,
    CriticalityOverrides, CriticalityPolicy, CriticalityProfiles, CriticalityReason,
    SCIPSymbolHeader, SourceMode, SourceProjection,
};
