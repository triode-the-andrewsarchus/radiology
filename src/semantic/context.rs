//! Exact under-caret index facts shared by inspector presentation and navigation.

use super::{
    index::{SCIPSymbolHeader, SCIPSymbolString, SemanticSymbolID},
    reference_results::check_cancel,
    ScipRange, SemanticIndex,
};
use crate::source::{
    preview::{preview_line, PreviewLine},
    ReviewRevision, SourceBuffer,
};
use std::{ops::Range, sync::atomic::AtomicBool};

/// An indexed definition, resolved against exact destination bytes on activation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionDestination {
    /// Repository-relative source path.
    pub path: String,
    /// Original coordinates in the destination document's declared encoding.
    pub range: ScipRange,
}

impl DefinitionDestination {
    fn indexed(path: &str, range: &ScipRange) -> Option<Self> {
        (crate::source::SourceLocation::is_repository_path(path)
            && range.start_line >= 0
            && range.start_col >= 0
            && range.end_col >= 0
            && (range.end_line, range.end_col) > (range.start_line, range.start_col))
            .then(|| Self {
                path: path.into(),
                range: range.clone(),
            })
    }
}

/// Bounded metadata for one exact, single-line indexed occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnderCaretContext {
    /// Global or document-scoped identity.
    pub id: SemanticSymbolID,
    /// Exact occurrence bytes in the captured displayed document.
    pub occurrence: Range<usize>,
    /// Index display name, with a source-token fallback for unnamed locals.
    pub name: String,
    /// Human-readable SCIP kind, preserving unknown numeric kinds.
    pub kind: String,
    /// Index signature, bounded to eight lines and 600 UTF-8 bytes.
    pub signature: String,
    /// First prose paragraph, excluding fenced code and bounded to 280 scalars.
    pub documentation: Option<String>,
    /// Whether the indexed occurrence is a definition.
    pub is_on_definition: bool,
    /// Number of distinct indexed in-repository definitions, including this one.
    pub definition_count: usize,
    /// Canonical indexed definition other than the occurrence under the caret.
    pub destination: Option<DefinitionDestination>,
    /// Indexed reference sites, excluding definition roles; not loaded-source coverage.
    pub reference_count: usize,
    /// Validated nonempty occurrences of this identity in the displayed document.
    pub file_occurrence_count: usize,
    /// Indexed enclosing-symbol display name, if recorded.
    pub enclosing_name: Option<String>,
    /// Package and version when no in-repository definition is indexed.
    pub external_package: Option<String>,
    /// Exact bounded first definition line in the displayed document.
    pub local_definition_line: Option<PreviewLine>,
}

impl UnderCaretContext {
    /// Prepares facts without reading files; hosts should run this off the UI thread.
    ///
    /// Imports, Before revisions, stale source, multiline/empty occurrences and
    /// positions without semantic data return `None`. Destination coordinates
    /// remain index facts until navigation validates the destination source.
    ///
    /// # Errors
    /// Returns cancellation without publishing partial metadata.
    pub fn prepare(
        buffer: &SourceBuffer,
        index: &SemanticIndex,
        byte: usize,
        cancel: &AtomicBool,
    ) -> Result<Option<Self>, String> {
        check_cancel(cancel)?;
        if buffer.revision == Some(ReviewRevision::Before) {
            return Ok(None);
        }
        let Some(occurrence) = index.occurrence_at_byte(buffer, byte).filter(|occ| {
            !occ.symbol.is_empty() && occ.symbol_roles & super::scip_proto::SCIP_ROLE_IMPORT == 0
        }) else {
            return Ok(None);
        };
        let Some(range) = occurrence
            .range
            .as_ref()
            .filter(|range| range.start_line == range.end_line)
        else {
            return Ok(None);
        };
        let Some(bytes) = index
            .byte_range(buffer, range)
            .filter(|range| !range.is_empty())
        else {
            return Ok(None);
        };
        let id = SemanticSymbolID::new(&occurrence.symbol, &buffer.path);
        let info = index.symbol_info_for_id(&id);
        let mut definitions = index.definition_locations_for_id(&id);
        definitions.sort_by_key(|(path, range)| {
            (
                *path,
                range.start_line,
                range.start_col,
                range.end_line,
                range.end_col,
            )
        });
        definitions.dedup();
        let destination = definitions
            .iter()
            .filter(|(path, range)| {
                *path != buffer.path
                    || index
                        .byte_range(buffer, range)
                        .is_some_and(|range| range != bytes)
            })
            .find_map(|(path, range)| DefinitionDestination::indexed(path, range));
        let local_definition_line = definitions
            .iter()
            .filter(|(path, _)| *path == buffer.path)
            .find_map(|(_, range)| {
                index
                    .byte_range(buffer, range)
                    .filter(|range| !range.is_empty())
            })
            .map(|range| preview_line(buffer, buffer.line_at_byte(range.start), &range));
        let file_occurrence_count = file_occurrences(buffer, index, &id, cancel)?;
        let mut name = index.display_name(&id);
        if id.is_local() && name == SCIPSymbolString::fallback_name(&id.symbol) {
            name = buffer.text[bytes.clone()].into();
        }
        let external_package = (definitions.is_empty() && !id.is_local())
            .then(|| SCIPSymbolHeader::parse(&id.symbol))
            .flatten()
            .and_then(|header| {
                header.package_name().map(|package| {
                    if header.version.is_empty() || header.version == "." {
                        package.to_owned()
                    } else {
                        format!("{package} {}", header.version)
                    }
                })
            });
        let context = Self {
            id: id.clone(),
            occurrence: bytes,
            name,
            kind: kind_name(info.map_or(0, |info| info.kind)),
            signature: info.map_or_else(String::new, |info| bounded_signature(&info.signature)),
            documentation: info.and_then(|info| first_paragraph(&info.documentation)),
            is_on_definition: occurrence.is_definition(),
            definition_count: definitions.len(),
            destination,
            reference_count: index.references.get(&id).map_or(0, Vec::len),
            file_occurrence_count,
            enclosing_name: info
                .filter(|info| !info.enclosing_symbol.is_empty())
                .map(|info| {
                    index.display_name(&SemanticSymbolID::new(&info.enclosing_symbol, &buffer.path))
                }),
            external_package,
            local_definition_line,
        };
        check_cancel(cancel)?;
        Ok(Some(context))
    }
}

fn file_occurrences(
    buffer: &SourceBuffer,
    index: &SemanticIndex,
    id: &SemanticSymbolID,
    cancel: &AtomicBool,
) -> Result<usize, String> {
    let mut file_occurrence_count = 0;
    if let Some(document) = index.documents.get(&buffer.path) {
        for site in &document.occurrences {
            check_cancel(cancel)?;
            if site.symbol == id.symbol
                && site
                    .range
                    .as_ref()
                    .and_then(|range| index.byte_range(buffer, range))
                    .is_some_and(|range| !range.is_empty())
            {
                file_occurrence_count += 1;
            }
        }
    }
    Ok(file_occurrence_count)
}

fn bounded_signature(text: &str) -> String {
    let mut end = text.len().min(600);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].lines().take(8).collect::<Vec<_>>().join("\n")
}

fn first_paragraph(entries: &[String]) -> Option<String> {
    for entry in entries {
        let mut fenced = false;
        let mut paragraph = String::new();
        for line in entry.lines() {
            let line = line.trim();
            if line.starts_with("```") {
                fenced = !fenced;
                continue;
            }
            if fenced {
                continue;
            }
            if line.is_empty() {
                if !paragraph.is_empty() {
                    break;
                }
                continue;
            }
            if !paragraph.is_empty() {
                paragraph.push(' ');
            }
            let remaining = 281_usize.saturating_sub(paragraph.chars().count());
            paragraph.extend(line.chars().take(remaining));
            if paragraph.chars().count() > 280 {
                paragraph = paragraph.chars().take(279).chain(['…']).collect();
                break;
            }
        }
        if !paragraph.is_empty() {
            return Some(paragraph);
        }
    }
    None
}

/// Returns the SCIP kind name, retaining unknown values as numeric descriptions.
#[must_use]
pub fn kind_name(kind: i32) -> String {
    const NAMES: &[&str] = &[
        "Symbol",
        "Array",
        "Assertion",
        "Associated Type",
        "Attribute",
        "Axiom",
        "Boolean",
        "Class",
        "Constant",
        "Constructor",
        "Data Family",
        "Enum",
        "Enum Member",
        "Event",
        "Fact",
        "Field",
        "File",
        "Function",
        "Getter",
        "Grammar",
        "Instance",
        "Interface",
        "Key",
        "Lang",
        "Lemma",
        "Macro",
        "Method",
        "Method Receiver",
        "Message",
        "Module",
        "Namespace",
        "Null",
        "Number",
        "Object",
        "Operator",
        "Package",
        "Package Object",
        "Parameter",
        "Parameter Label",
        "Pattern",
        "Predicate",
        "Property",
        "Protocol",
        "Quasiquoter",
        "Self Parameter",
        "Setter",
        "Signature",
        "Subscript",
        "String",
        "Struct",
        "Tactic",
        "Theorem",
        "This Parameter",
        "Trait",
        "Type",
        "Type Alias",
        "Type Class",
        "Type Family",
        "Type Parameter",
        "Union",
        "Value",
        "Variable",
        "Contract",
        "Error",
        "Library",
        "Modifier",
        "Abstract Method",
        "Method Specification",
        "Protocol Method",
        "Pure Virtual Method",
        "Trait Method",
        "Type Class Method",
        "Accessor",
        "Delegate",
        "Method Alias",
        "Singleton Class",
        "Singleton Method",
        "Static Data Member",
        "Static Event",
        "Static Field",
        "Static Method",
        "Static Property",
        "Static Variable",
        "",
        "Extension",
        "Mixin",
        "Concept",
    ];
    usize::try_from(kind)
        .ok()
        .and_then(|index| NAMES.get(index))
        .filter(|name| !name.is_empty())
        .map_or_else(|| format!("Kind {kind}"), |name| (*name).into())
}

#[cfg(test)]
mod tests;
