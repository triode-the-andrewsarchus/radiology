//! Critical Lens mode and criticality classification policies.
//!
//! Provides [`SourceMode`] (Normal vs Critical) and [`CriticalityPolicy`]
//! to distinguish boilerplate runtime plumbing from core domain logic, system APIs, and FFI boundaries.

pub mod annotations;
pub mod frontier;
pub mod graph;
pub mod hover;
pub mod lens;
pub mod marks;
pub mod menu;
pub mod overrides;
pub mod projection;
pub mod service;
pub mod worker;

pub use lens::{
    CountAnnotation, CountCompleteness, Criticality, CriticalityClassification,
    CriticalityOverrides, CriticalityPolicy, CriticalityProfiles, CriticalityReason,
    SCIPSymbolHeader, SourceMode, SourceProjection,
};

#[cfg(test)]
mod test_support;
