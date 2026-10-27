//! SCIP semantic code intelligence, symbol resolution, and caller call trees.
//!
//! Provides pure-Rust zero-dependency decoding of SCIP Protocol Buffers indexes
//! ([`ScipIndex`]), repository semantic indexing ([`SemanticIndex`]), and multi-level
//! caller hierarchy tracing with cycle prevention and deterministic 12-color
//! caller identity capsules ([`CallerTreeBuilder`]).

pub mod caller_tree;
pub mod index;
pub mod scip_proto;

pub use caller_tree::{
    caller_pill_color, caller_pill_slot, CallerTreeBuilder, ReferenceCallerNode, CALLER_PALETTE,
};
pub use index::SemanticIndex;
pub use scip_proto::{ScipDocument, ScipIndex, ScipOccurrence, ScipRange, ScipSymbolInformation};
