//! Source text management, canonical byte addressing, and navigation coordinates.
//!
//! Byte offsets serve as canonical coordinates throughout Radiology. [`SourceBuffer`]
//! provides efficient conversions between byte offsets, 1-based line numbers, and 0-based
//! character columns. [`NavigationHistory`] maintains the Back/Forward navigation stack,
//! and [`OccurrencePinStore`] manages multi-occurrence token highlighting groups.

pub mod buffer;
pub mod find;
pub mod history;
pub mod location;
pub mod selection;

pub use buffer::{SourceBuffer, SourceBufferError};
pub use history::NavigationHistory;
pub use location::{ColumnEncoding, ReviewRevision, SourceLocation};
pub use selection::{OccurrenceColor, OccurrencePinGroup, OccurrencePinStore};
