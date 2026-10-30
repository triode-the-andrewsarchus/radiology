//! Shared exact UTF-16 occurrence construction for source-backed semantic tests.

use super::{ScipOccurrence, ScipRange};
use crate::source::SourceBuffer;
use std::ops::Range;

pub(crate) fn occurrence(
    buffer: &SourceBuffer,
    range: Range<usize>,
    symbol: &str,
    definition: bool,
) -> ScipOccurrence {
    let line = buffer.line_at_byte(range.start);
    let start = buffer.text[buffer.line_starts[line - 1]..range.start]
        .encode_utf16()
        .count() as i32;
    let end = start + buffer.text[range].encode_utf16().count() as i32;
    ScipOccurrence {
        symbol: symbol.into(),
        symbol_roles: i32::from(definition),
        range: ScipRange::from_packed(&[line as i32 - 1, start, end]),
        ..Default::default()
    }
}
