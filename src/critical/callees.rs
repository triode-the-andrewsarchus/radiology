//! Exact callee resolution shared by source projection and callable summaries.

use crate::semantic::{index::SemanticSymbolID, SemanticIndex};
use crate::source::{ReviewRevision, SourceBuffer};
use std::{collections::HashMap, ops::Range};

pub(super) struct CalleeResolver<'a> {
    pub index: Option<&'a SemanticIndex>,
    occurrences: IndexedCalls,
}

impl<'a> CalleeResolver<'a> {
    pub fn new(buffer: &SourceBuffer, index: Option<&'a SemanticIndex>) -> Self {
        let index = index.filter(|index| {
            buffer.revision != Some(ReviewRevision::Before) && index.matches_buffer(buffer)
        });
        Self {
            index,
            occurrences: indexed_calls(buffer, index),
        }
    }
    pub fn resolve(&self, name: &Range<usize>) -> Option<&SemanticSymbolID> {
        resolve(name, &self.occurrences)
    }
}

type IndexedCalls = HashMap<usize, Vec<(Range<usize>, SemanticSymbolID)>>;

fn indexed_calls(buffer: &SourceBuffer, index: Option<&SemanticIndex>) -> IndexedCalls {
    let mut occurrences: IndexedCalls = HashMap::new();
    if let Some(index) = index {
        if let Some(document) = index.documents.get(&buffer.path) {
            for occurrence in &document.occurrences {
                let Some(range) = occurrence
                    .range
                    .as_ref()
                    .filter(|range| range.start_line == range.end_line)
                    .and_then(|range| index.byte_range(buffer, range))
                    .filter(|range| !range.is_empty())
                else {
                    continue;
                };
                occurrences.entry(range.start).or_default().push((
                    range,
                    SemanticSymbolID::new(occurrence.symbol.clone(), &buffer.path),
                ));
            }
        }
    }
    occurrences
}

fn resolve<'a>(name: &Range<usize>, occurrences: &'a IndexedCalls) -> Option<&'a SemanticSymbolID> {
    let hits = occurrences.get(&name.start)?;
    hits.iter()
        .find(|(range, _)| range == name)
        .or_else(|| hits.first())
        .map(|(_, id)| id)
}
