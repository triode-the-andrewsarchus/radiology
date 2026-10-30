//! Shared encoded occurrence fixtures for source projection and graph tests.

use crate::semantic::{ScipDocument, ScipIndex, ScipOccurrence, ScipRange, SemanticIndex};
use crate::source::SourceBuffer;
use std::ops::Range;

pub(super) fn index_ranges(
    buffer: &SourceBuffer,
    entries: impl IntoIterator<Item = (Range<usize>, String)>,
) -> SemanticIndex {
    let occurrences = entries
        .into_iter()
        .map(|(range, symbol)| {
            let line = buffer.text[..range.start]
                .bytes()
                .filter(|&b| b == b'\n')
                .count();
            let line_start = buffer.text[..range.start]
                .rfind('\n')
                .map_or(0, |byte| byte + 1);
            let start = buffer.text[line_start..range.start].encode_utf16().count();
            let end = start + buffer.text[range].encode_utf16().count();
            ScipOccurrence {
                symbol,
                range: ScipRange::from_packed(&[line as i32, start as i32, end as i32]),
                ..Default::default()
            }
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
