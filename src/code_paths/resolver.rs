//! Named source navigation independent of semantic indexes and saved line numbers.
use super::{source_text, CodePathStep};
use crate::{
    source::{
        loader::{SourceRead, SOURCE_READ_TIMEOUT},
        ReviewRevision, SourceBuffer, SourceLocation,
    },
    syntax::{DocumentSymbol, DocumentSyntax, LanguageRegistry},
};
use std::{
    collections::HashSet,
    path::{Component, Path},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, LazyLock,
    },
    time::Instant,
};

/// A canonical destination resolved against one exact immutable source snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodePathDestination {
    /// Original source byte and checkout/After revision.
    pub location: SourceLocation,
    /// One-based source line for conditional-variant presentation.
    pub line: usize,
    /// Full-source identity for guarded subsequent source loading.
    pub content_digest: String,
}

/// Source and already-computed structure owned by one completed resolution job.
pub struct ResolvedStep {
    /// Exact bytes used for every destination; the matching displayed buffer wins.
    pub buffer: Arc<SourceBuffer>,
    /// The same source's structure, available for later reader publication reuse.
    pub syntax: DocumentSyntax,
    /// Source-ordered alternatives; more than one means opposing conditional branches.
    pub destinations: Vec<CodePathDestination>,
}

/// A failed resolution never substitutes text matches or arbitrary overload choices.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ResolveError {
    /// The step is not a valid repository-relative record.
    #[error("This code path step is invalid.")]
    InvalidStep,
    /// The requested language has neither native declarations nor assembly labels.
    #[error("Symbol navigation is not supported for {0}.")]
    Unsupported(String),
    /// The current source has no such declaration.
    #[error("‘{selector}’ was not found in {path} in this revision. Update the trace if it was renamed or moved.")]
    Missing {
        /// Authored selector.
        selector: String,
        /// Repository-relative filename.
        path: String,
    },
    /// Multiple ordinary declarations match; the caller must supply a qualifier/header.
    #[error("‘{selector}’ is ambiguous in {path}. Use an enclosing type/module name or the exact declaration header to distinguish it.")]
    Ambiguous {
        /// Authored selector.
        selector: String,
        /// Repository-relative filename.
        path: String,
    },
    /// The owning request was retired.
    #[error("Code path resolution cancelled")]
    Cancelled,
    /// Source or syntax preparation failed; no partial result was returned.
    #[error("{0}")]
    Preparation(String),
}

pub(super) fn check(cancel: &AtomicBool) -> Result<(), ResolveError> {
    if cancel.load(Ordering::Relaxed) {
        Err(ResolveError::Cancelled)
    } else {
        Ok(())
    }
}

/// Reads and resolves a step, preferring the exact matching displayed source buffer.
/// Checkout reads enforce canonical containment; reviews explicitly read pinned After.
/// The host runs this on a worker and guards publication against its current scope,
/// navigation/caret revisions and selected trace. No semantic index is required.
///
/// # Examples
/// ```
/// use radiology::{code_paths::{CodePath, resolver::resolve_step},
///     source::{SourceBuffer, loader::SourceRead}};
/// use std::sync::{Arc, atomic::AtomicBool};
/// let source = Arc::new(SourceBuffer::from_str("boot.rs", "fn boot() {}\n"));
/// let trace = CodePath::parse("Boot\nboot.rs:boot:Begin")?;
/// let resolved = resolve_step(&trace.steps[0], &SourceRead::Buffer(source.clone()),
///     Some(&source), &AtomicBool::new(false))?;
/// assert!(Arc::ptr_eq(&resolved.buffer, &source));
/// assert_eq!(resolved.destinations[0].location.byte_offset, 3);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// Rejects invalid/unreadable/unsupported source, missing/ambiguous selectors and
/// cancellation. A Before-only buffer cannot stand in for an After snapshot.
pub fn resolve_step(
    step: &CodePathStep,
    source: &SourceRead,
    open: Option<&Arc<SourceBuffer>>,
    cancel: &AtomicBool,
) -> Result<ResolvedStep, ResolveError> {
    check(cancel)?;
    if !step.is_valid() {
        return Err(ResolveError::InvalidStep);
    }
    let path: std::path::PathBuf = Path::new(&step.path)
        .components()
        .filter(|part| matches!(part, Component::Normal(_)))
        .collect();
    let path = path.to_str().ok_or(ResolveError::InvalidStep)?;
    if !SourceLocation::is_repository_path(path) {
        return Err(ResolveError::InvalidStep);
    }
    let revision = match source {
        SourceRead::Review(_) => Some(ReviewRevision::After),
        SourceRead::Buffer(buffer) => buffer.revision,
        _ => None,
    };
    if revision == Some(ReviewRevision::Before) {
        return Err(ResolveError::Preparation(
            "Code paths require checkout or After source".into(),
        ));
    }
    let buffer = match open.filter(|buffer| buffer.path == path && buffer.revision == revision) {
        Some(buffer) => buffer.clone(),
        None => {
            let loaded = source.read_contained_cancellable(
                &SourceLocation::with_revision(path, 0, revision),
                cancel,
                Instant::now() + SOURCE_READ_TIMEOUT,
            );
            check(cancel)?;
            loaded.map_err(ResolveError::Preparation)?
        }
    };
    let syntax = DocumentSyntax::analyze_cancellable(&buffer, || cancel.load(Ordering::Relaxed));
    check(cancel)?;
    let syntax = syntax.map_err(ResolveError::Preparation)?;
    let destinations = destinations(&step.function, &buffer, &syntax, cancel)?;
    Ok(ResolvedStep {
        buffer,
        syntax,
        destinations,
    })
}

/// Resolves a selector using structure from the supplied exact source snapshot.
/// Bare names, suffix-qualified enclosing names (`::`/`.`), and whitespace-normalized
/// full declaration headers are supported. Definitions outrank prototypes; ordinary
/// ambiguity fails. Opposing preprocessor branches produce selectable alternatives.
/// Assembly labels and the documented entry macros are supported without a grammar.
///
/// # Examples
/// ```
/// use radiology::{code_paths::resolver::destinations, source::SourceBuffer,
///     syntax::DocumentSyntax};
/// use std::sync::atomic::AtomicBool;
/// let source = SourceBuffer::from_str("boot.rs", "// café 🦀\nfn boot() {}\n");
/// let syntax = DocumentSyntax::analyze(&source);
/// let targets = destinations("boot", &source, &syntax, &AtomicBool::new(false))?;
/// assert_eq!(targets[0].line, 2);
/// assert!(source.text[targets[0].location.byte_offset..].starts_with("boot"));
/// # Ok::<(), radiology::code_paths::resolver::ResolveError>(())
/// ```
///
/// # Errors
/// Returns unsupported/missing/ambiguous/preparation errors or cancellation. The
/// caller must pair `syntax` with `buffer`; approximate fallback outlines are refused.
pub fn destinations(
    selector: &str,
    buffer: &SourceBuffer,
    syntax: &DocumentSyntax,
    cancel: &AtomicBool,
) -> Result<Vec<CodePathDestination>, ResolveError> {
    check(cancel)?;
    let language = LanguageRegistry::for_path(&buffer.path);
    let positions = if language.grammar.is_some() {
        if let Some(error) = &syntax.parse_error {
            return Err(ResolveError::Preparation(error.clone()));
        }
        declaration_positions(selector, buffer, syntax, cancel)?
    } else if Path::new(&buffer.path)
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "s" | "asm"))
    {
        assembly_positions(selector, buffer, cancel)?
    } else {
        return Err(ResolveError::Unsupported(buffer.path.clone()));
    };
    if positions.is_empty() {
        return Err(ResolveError::Missing {
            selector: selector.into(),
            path: buffer.path.clone(),
        });
    }
    check(cancel)?;
    Ok(positions
        .into_iter()
        .map(|byte| CodePathDestination {
            location: buffer.location_at_byte(byte),
            line: buffer.line_at_byte(byte),
            content_digest: buffer.content_digest().into(),
        })
        .collect())
}

fn declaration_positions(
    selector: &str,
    buffer: &SourceBuffer,
    syntax: &DocumentSyntax,
    cancel: &AtomicBool,
) -> Result<Vec<usize>, ResolveError> {
    let mut symbols = Vec::new();
    flatten(&syntax.symbols, &mut symbols, cancel)?;
    let known: HashSet<_> = symbols
        .iter()
        .map(|symbol| symbol.selection_range.clone())
        .collect();
    let candidates = symbols.iter().copied().map(|s| (s, false)).chain(
        syntax
            .recovered_declarations
            .iter()
            .filter(|s| !known.contains(&s.selection_range))
            .map(|s| (s, true)),
    );
    let body_ends: HashSet<_> = syntax
        .calls
        .scopes
        .iter()
        .filter(|scope| !scope.closure && scope.has_body())
        .map(|scope| scope.range.end)
        .collect();
    let mut matches = Vec::new();
    for (symbol, recovered) in candidates {
        check(cancel)?;
        if matches_selector(selector, symbol, &symbols, buffer) {
            matches.push((symbol, recovered || body_ends.contains(&symbol.range.end)));
        }
    }
    if matches.len() > 1 && matches.iter().any(|(_, definition)| *definition) {
        matches.retain(|(_, definition)| *definition);
    }
    let mut positions: Vec<_> = matches
        .iter()
        .map(|(s, _)| s.selection_range.start)
        .collect();
    positions.sort_unstable();
    positions.dedup();
    if positions.len() > 1 {
        let c = matches!(LanguageRegistry::for_path(&buffer.path).id, "c" | "cpp");
        if !(c
            && matches.iter().all(|(s, _)| s.kind.is_callable())
            && source_text::alternatives(&positions, buffer, &|| check(cancel))?)
        {
            return Err(ResolveError::Ambiguous {
                selector: selector.into(),
                path: buffer.path.clone(),
            });
        }
    }
    Ok(positions)
}

fn flatten<'a>(
    symbols: &'a [DocumentSymbol],
    out: &mut Vec<&'a DocumentSymbol>,
    cancel: &AtomicBool,
) -> Result<(), ResolveError> {
    for symbol in symbols {
        check(cancel)?;
        out.push(symbol);
        flatten(&symbol.children, out, cancel)?;
    }
    Ok(())
}

fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn matches_selector(
    selector: &str,
    symbol: &DocumentSymbol,
    all: &[&DocumentSymbol],
    buffer: &SourceBuffer,
) -> bool {
    let wanted = compact(selector);
    let mut parents: Vec<_> = all
        .iter()
        .filter(|parent| {
            parent.range != symbol.range
                && parent.range.start <= symbol.range.start
                && parent.range.end >= symbol.range.end
        })
        .collect();
    parents.sort_by_key(|p| std::cmp::Reverse(p.range.len()));
    let prefix = buffer
        .text
        .get(symbol.range.start..symbol.selection_range.start)
        .unwrap_or_default();
    let qualified = prefix
        .split_whitespace()
        .next_back()
        .filter(|prefix| prefix.ends_with("::"))
        .map(|prefix| format!("{prefix}{}", symbol.name));
    let mut names = vec![symbol.name.clone()];
    if let Some(qualified) = &qualified {
        names.push(qualified.clone());
    }
    for start in 0..parents.len() {
        let mut parts: Vec<_> = parents[start..].iter().map(|s| s.name.as_str()).collect();
        parts.push(&symbol.name);
        names.push(parts.join("::"));
        names.push(parts.join("."));
        if let Some(qualified) = &qualified {
            parts.pop();
            parts.push(qualified);
            names.push(parts.join("::"));
        }
    }
    if names.iter().any(|name| compact(name) == wanted) {
        return true;
    }
    let declaration = buffer.text.get(symbol.range.clone()).unwrap_or_default();
    let header = declaration
        .split(|c| {
            c == '{'
                || c == ';'
                || (c == '\n' && LanguageRegistry::for_path(&buffer.path).id == "python")
        })
        .next()
        .unwrap_or_default();
    selector.contains('(') && compact(header) == wanted
}

fn assembly_positions(
    selector: &str,
    buffer: &SourceBuffer,
    cancel: &AtomicBool,
) -> Result<Vec<usize>, ResolveError> {
    static MACRO: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"^\s*(?:BEGIN_FUNC|ENTRY|ENTRY_NP|SYM_FUNC_START|SYM_CODE_START)\s*\(\s*([A-Za-z_.$][A-Za-z0-9_.$]*)\s*\)").expect("assembly declaration pattern")
    });
    let bytes = source_text::masked(&buffer.data, &|| check(cancel))?;
    let text = std::str::from_utf8(&bytes).expect("mask preserves UTF-8");
    let mut positions = Vec::new();
    let mut start = 0;
    for line in text.split_inclusive('\n') {
        check(cancel)?;
        let trimmed = line.trim_start();
        if trimmed.starts_with(&format!("{selector}:")) {
            positions.push(start + line.len() - trimmed.len());
        } else if let Some(name) = MACRO
            .captures(line)
            .and_then(|captures| captures.get(1))
            .filter(|name| name.as_str() == selector)
        {
            positions.push(start + name.start());
        }
        start += line.len();
    }
    if positions.len() > 1 && !source_text::alternatives(&positions, buffer, &|| check(cancel))? {
        return Err(ResolveError::Ambiguous {
            selector: selector.into(),
            path: buffer.path.clone(),
        });
    }
    Ok(positions)
}

#[cfg(test)]
mod tests;
