//! Critical Lens mode and criticality classification policies.
//!
//! Provides [`SourceMode`] (Normal vs Critical) and [`CriticalityPolicy`]
//! to distinguish boilerplate runtime plumbing from core domain logic, system APIs, and FFI boundaries.

pub mod lens;

pub use lens::{
    Criticality, CriticalityClassification, CriticalityPolicy, CriticalityReason, SourceMode,
};
