//! Non-semantic function-local spelling groups from an exact shared parse.

use super::SupportedGrammar;
use crate::source::{buffer::SourceIdentity, SourceBuffer};
use std::{collections::BTreeMap, ops::Range};
use tree_sitter::Node;

/// A drawing-only spelling group; never a definition or reference identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntacticSelection {
    /// Innermost captured function or closure containing the clicked token.
    pub scope: Range<usize>,
    /// Complete token under the clicked byte.
    pub active: Range<usize>,
    /// Same-spelling tokens in the scope, excluding highlighted syntax classes.
    pub ranges: Vec<Range<usize>>,
}

/// Prepared syntactic occurrence facts, independent of call-query availability.
#[derive(Debug, Clone, Default)]
pub struct SyntacticOccurrences {
    scopes: Vec<Range<usize>>,
    words: BTreeMap<String, Vec<Range<usize>>>,
    identity: Option<SourceIdentity>,
    /// Query or fact-limit failure; failed queries publish no partial groups.
    pub error: Option<String>,
}

impl SyntacticOccurrences {
    pub(super) fn from_tree(
        buffer: &SourceBuffer,
        grammar: SupportedGrammar,
        root: Node<'_>,
        control: &super::AnalysisControl<'_>,
    ) -> Self {
        match extract(buffer, grammar, root, 200_000, control) {
            Ok((scopes, words)) => Self {
                scopes,
                words,
                identity: Some(SourceIdentity::new(buffer)),
                error: None,
            },
            Err(error) => Self {
                error: Some(error),
                ..Default::default()
            },
        }
    }

    /// Returns a same-spelling group only for the exact source and callable scope.
    ///
    /// Shadowed bindings remain one decoration; semantic navigation must use SCIP.
    #[must_use]
    pub fn selection(&self, buffer: &SourceBuffer, byte: usize) -> Option<SyntacticSelection> {
        if self
            .identity
            .as_ref()
            .is_none_or(|identity| !identity.matches(buffer))
        {
            return None;
        }
        let scope = self
            .scopes
            .iter()
            .filter(|scope| scope.contains(&byte))
            .min_by_key(|scope| scope.len())?;
        let active = token_range(&buffer.data, byte)?;
        let word = buffer.text.get(active.clone())?;
        let tokens = self.words.get(word)?;
        let first = tokens.partition_point(|range| range.start < scope.start);
        let ranges: Vec<_> = tokens[first..]
            .iter()
            .take_while(|range| range.end <= scope.end)
            .cloned()
            .collect();
        ranges.contains(&active).then(|| SyntacticSelection {
            scope: scope.clone(),
            active,
            ranges,
        })
    }
}

type Words = BTreeMap<String, Vec<Range<usize>>>;

fn extract(
    buffer: &SourceBuffer,
    grammar: SupportedGrammar,
    root: Node<'_>,
    limit: usize,
    control: &super::AnalysisControl<'_>,
) -> Result<(Vec<Range<usize>>, Words), String> {
    let mut scopes = Vec::new();
    let mut excluded = Vec::new();
    super::queries::visit_captures(
        buffer,
        grammar,
        super::queries::QueryRole::Occurrences,
        root,
        limit,
        control,
        |name, node, _| {
            if name.starts_with("scope.") {
                scopes.push(node.byte_range());
            } else if ["comment", "string", "number", "keyword"]
                .iter()
                .any(|prefix| name.starts_with(prefix))
            {
                excluded.push(node.byte_range());
            }
        },
    )?;
    scopes.sort_by_key(|range| (range.start, range.end));
    scopes.dedup();
    let excluded = reader_text::union_ranges(excluded);
    let words = collect_words(
        buffer,
        &excluded,
        limit.saturating_sub(scopes.len() + excluded.len()),
        control,
    )?;
    Ok((scopes, words))
}

fn collect_words(
    buffer: &SourceBuffer,
    excluded: &[Range<usize>],
    limit: usize,
    control: &super::AnalysisControl<'_>,
) -> Result<Words, String> {
    let mut words = Words::new();
    let mut byte = 0;
    let mut count = 0;
    while byte < buffer.data.len() {
        if byte.is_multiple_of(4096) {
            control.check()?;
        }
        let start = byte;
        if !identifier_byte(buffer.data[byte]) {
            byte += 1;
            continue;
        }
        while byte < buffer.data.len() && identifier_byte(buffer.data[byte]) {
            if byte.is_multiple_of(4096) {
                control.check()?;
            }
            byte += 1;
        }
        let first = excluded.partition_point(|range| range.end <= start);
        if buffer.data[start].is_ascii_digit()
            || excluded.get(first).is_some_and(|range| range.start < byte)
        {
            continue;
        }
        count += 1;
        if count > limit {
            return Err("Occurrence tokens exceeded their fact limit".into());
        }
        words
            .entry(buffer.text[start..byte].to_owned())
            .or_default()
            .push(start..byte);
    }
    Ok(words)
}

fn token_range(bytes: &[u8], byte: usize) -> Option<Range<usize>> {
    if !identifier_byte(*bytes.get(byte)?) {
        return None;
    }
    let mut start = byte;
    let mut end = byte + 1;
    while start > 0 && identifier_byte(bytes[start - 1]) {
        start -= 1;
    }
    while end < bytes.len() && identifier_byte(bytes[end]) {
        end += 1;
    }
    (!bytes[start].is_ascii_digit()).then_some(start..end)
}

fn identifier_byte(byte: u8) -> bool {
    byte == b'_' || byte >= 0x80 || byte.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests;
