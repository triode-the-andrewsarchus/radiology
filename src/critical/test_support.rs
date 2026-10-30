//! Shared encoded occurrence fixtures for source projection and graph tests.

use crate::semantic::{ScipDocument, ScipIndex, SemanticIndex};
use crate::source::SourceBuffer;
use std::ops::Range;

pub(super) fn index_ranges(
    buffer: &SourceBuffer,
    entries: impl IntoIterator<Item = (Range<usize>, String)>,
) -> SemanticIndex {
    let occurrences = entries
        .into_iter()
        .map(|(range, symbol)| {
            crate::semantic::test_support::occurrence(buffer, range, &symbol, false)
        })
        .collect();
    SemanticIndex::from_scip(ScipIndex {
        documents: vec![ScipDocument {
            relative_path: buffer.path.clone(),
            position_encoding: 2,
            occurrences,
            ..Default::default()
        }],
        ..Default::default()
    })
}
