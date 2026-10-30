//! Source-validated incoming call paths for one selected reference or definition.

use super::{
    callees::CalleeResolver,
    index::SemanticSymbolID,
    reference_results::{check_cancel, ReferenceHit},
};
use crate::{
    source::{preview::SourceQueryInputs, ReviewRevision, SourceBuffer, SourceLocation},
    syntax::{calls::CallSites, DocumentSyntax},
};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    sync::{atomic::AtomicBool, Arc},
};

/// One containing callable or an explicit boundary in an incoming call path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallerRow {
    /// Zero-based nesting depth, with the selected site's owner at zero.
    pub depth: usize,
    /// Parsed name, never inferred from a preceding indexed declaration.
    pub name: String,
    /// Exact call-site location in captured source storage.
    pub location: SourceLocation,
    /// One-based source line of this site.
    pub line: usize,
    /// Zero-based Unicode scalar column, validated with the tree source digest.
    pub column: usize,
    /// Scoped semantic identity, when the parsed declaration is indexed.
    pub symbol: Option<SemanticSymbolID>,
    /// Why traversal stopped or why a recursive continuation was suppressed.
    pub note: Option<String>,
}

/// A bounded depth-first traversal, preserving repeated callers on sibling paths.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CallerTree {
    /// Source-ordered incoming branches, flattened for virtualized presentation.
    pub rows: Vec<CallerRow>,
    /// Sorted coverage caveats, including unreadable or stale source.
    pub notices: Vec<String>,
    /// Whether row or source limits prevented a complete traversal.
    pub has_more: bool,
    /// SHA-256 identities of every accepted file read.
    pub source_identities: std::collections::BTreeMap<String, String>,
}

/// Resource limits for one incoming-call request.
#[derive(Debug, Clone, Copy)]
pub struct CallerLimits {
    /// Maximum published rows; there is no arbitrary depth limit.
    pub rows: usize,
    /// Maximum attempted distinct source files, including failures.
    pub files: usize,
    /// Maximum accepted source bytes across parsed files.
    pub source_bytes: usize,
}

impl Default for CallerLimits {
    fn default() -> Self {
        Self {
            rows: 256,
            files: 512,
            source_bytes: 64 * 1024 * 1024,
        }
    }
}

impl CallerLimits {
    /// Doubles request bounds without overflowing.
    #[must_use]
    pub fn expanded(self) -> Self {
        Self {
            rows: self.rows.saturating_mul(2),
            files: self.files.saturating_mul(2),
            source_bytes: self.source_bytes.saturating_mul(2),
        }
    }
}

struct File {
    buffer: Arc<SourceBuffer>,
    calls: CallSites,
    callees: HashMap<usize, SemanticSymbolID>,
}

#[derive(Clone)]
struct Owner {
    row: CallerRow,
    declaration: Option<SourceLocation>,
}

struct Incoming {
    owners: Vec<Owner>,
    partial: bool,
}

struct Pending {
    owner: Owner,
    symbols: HashSet<SemanticSymbolID>,
    declarations: HashSet<SourceLocation>,
}

struct Trace<'a> {
    inputs: &'a SourceQueryInputs,
    cancel: &'a AtomicBool,
    limits: CallerLimits,
    files: HashMap<String, Result<Arc<File>, String>>,
    source_bytes: usize,
    notices: BTreeSet<String>,
    limited: bool,
}

/// Traces callers backward from the parsed owner of one exact listed site.
///
/// The supplied digest must come from the site's reference group. Displayed
/// bytes take precedence over disk; other files use the captured checkout or PR
/// revision. Only parsed invocations whose semantic callee matches establish an
/// edge. Callback references and closures remain explicit terminal rows.
///
/// # Errors
/// Returns cancellation, Before-revision use, or a changed/unreadable initial
/// site. Failures on later branches appear in the returned coverage notices.
pub fn trace(
    inputs: &SourceQueryInputs,
    hit: &ReferenceHit,
    digest: &str,
    limits: CallerLimits,
    cancel: &AtomicBool,
) -> Result<CallerTree, String> {
    check_cancel(cancel)?;
    if inputs.buffer.revision == Some(ReviewRevision::Before) {
        return Err("After call data is unavailable in Before source.".into());
    }
    let mut trace = Trace {
        inputs,
        cancel,
        limits,
        files: HashMap::new(),
        source_bytes: 0,
        notices: BTreeSet::new(),
        limited: false,
    };
    let file = trace.load(&hit.path)?;
    if file.buffer.content_digest() != digest
        || inputs.index.byte_range(&file.buffer, &hit.range).as_ref() != Some(&hit.byte_range)
    {
        return Err("This reference changed. Reload or reindex to trace its callers.".into());
    }
    let first = owner(&file, hit.byte_range.start, inputs);
    let mut pending = vec![Pending {
        owner: first,
        symbols: HashSet::new(),
        declarations: HashSet::new(),
    }];
    let mut tree = CallerTree::default();
    while tree.rows.len() < limits.rows {
        check_cancel(cancel)?;
        let Some(next) = pending.pop() else { break };
        trace.visit(next, &mut pending, &mut tree.rows)?;
    }
    check_cancel(cancel)?;
    tree.has_more = !pending.is_empty() || trace.limited;
    tree.notices = trace.notices.into_iter().collect();
    for (path, file) in trace.files {
        if let Ok(file) = file {
            tree.source_identities
                .insert(path, file.buffer.content_digest().to_owned());
        }
    }
    Ok(tree)
}

impl Trace<'_> {
    fn load(&mut self, path: &str) -> Result<Arc<File>, String> {
        check_cancel(self.cancel)?;
        if let Some(file) = self.files.get(path) {
            return file.clone();
        }
        if self.files.len() >= self.limits.files || self.source_bytes >= self.limits.source_bytes {
            self.limited = true;
            return Err("caller source budget reached".into());
        }
        let loaded = self.read(path);
        check_cancel(self.cancel)?;
        self.files.insert(path.to_owned(), loaded.clone());
        loaded
    }

    fn read(&mut self, path: &str) -> Result<Arc<File>, String> {
        let buffer = self.inputs.read(path)?;
        check_cancel(self.cancel)?;
        if buffer.byte_len() > self.limits.source_bytes.saturating_sub(self.source_bytes) {
            self.limited = true;
            return Err("caller source budget reached".into());
        }
        self.source_bytes += buffer.byte_len();
        if !self.inputs.index.matches_buffer(&buffer) {
            return Err("stale semantic source identity".into());
        }
        let calls = DocumentSyntax::analyze(&buffer).calls;
        let resolver = CalleeResolver::new(&buffer, Some(&self.inputs.index));
        let callees = calls
            .sites
            .iter()
            .filter_map(|site| {
                let name = site.name.as_ref()?;
                Some((name.start, resolver.resolve(name)?.clone()))
            })
            .collect();
        if calls.parse_errors {
            self.notices.insert(format!("{path}: parse errors"));
        }
        Ok(Arc::new(File {
            buffer,
            calls,
            callees,
        }))
    }

    fn visit(
        &mut self,
        next: Pending,
        pending: &mut Vec<Pending>,
        rows: &mut Vec<CallerRow>,
    ) -> Result<(), String> {
        let mut row = next.owner.row;
        if row.note.is_none() {
            if let Some(symbol) = &row.symbol {
                let callers = self.callers(symbol)?;
                if callers.owners.is_empty() {
                    row.note = Some(
                        if !callers.partial {
                            "No indexed callers"
                        } else {
                            "Caller coverage incomplete"
                        }
                        .into(),
                    );
                }
                let mut symbols = next.symbols;
                symbols.insert(symbol.clone());
                let mut declarations = next.declarations;
                declarations.extend(next.owner.declaration);
                let mut continuations = Vec::new();
                for mut caller in callers.owners {
                    if caller
                        .row
                        .symbol
                        .as_ref()
                        .is_some_and(|id| symbols.contains(id))
                        || caller
                            .declaration
                            .as_ref()
                            .is_some_and(|location| declarations.contains(location))
                    {
                        row.note = Some("Recursive branch stopped".into());
                    } else {
                        caller.row.depth = row.depth + 1;
                        continuations.push(Pending {
                            owner: caller,
                            symbols: symbols.clone(),
                            declarations: declarations.clone(),
                        });
                    }
                }
                pending.extend(continuations.into_iter().rev());
                let remaining = self.limits.rows.saturating_sub(rows.len());
                if pending.len() > remaining {
                    // Keep the next DFS prefix (the end of this stack).
                    pending.drain(..pending.len() - remaining);
                    self.limited = true;
                }
            }
        }
        rows.push(row);
        Ok(())
    }

    fn callers(&mut self, symbol: &SemanticSymbolID) -> Result<Incoming, String> {
        let mut positions = self.inputs.index.find_references_for_id(symbol);
        positions.sort_by_key(|(path, range)| {
            (
                *path,
                range.start_line,
                range.start_col,
                range.end_line,
                range.end_col,
            )
        });
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        let mut partial = false;
        for (path, range) in positions {
            check_cancel(self.cancel)?;
            let file = match self.load(path) {
                Ok(file) => file,
                Err(error) => {
                    partial = true;
                    check_cancel(self.cancel)?;
                    self.notices.insert(format!(
                        "{path}: {}",
                        error.chars().take(1024).collect::<String>()
                    ));
                    if self.limited {
                        break;
                    }
                    continue;
                }
            };
            let Some(bytes) = self
                .inputs
                .index
                .byte_range(&file.buffer, range)
                .filter(|bytes| !bytes.is_empty())
            else {
                partial = true;
                self.notices
                    .insert(format!("{path}: stale index coordinates"));
                continue;
            };
            if !seen.insert((path, bytes.start)) {
                continue;
            }
            let mut caller = owner(&file, bytes.start, self.inputs);
            if !file.calls.supported {
                caller.row.note = Some("Call-site structure unavailable".into());
            } else if file.callees.get(&bytes.start) != Some(symbol) {
                caller.row.note = Some("Non-call reference; invocation unknown".into());
            }
            result.push(caller);
            if result.len() > self.limits.rows {
                partial = true;
                self.limited = true;
                break;
            }
        }
        Ok(Incoming {
            owners: result,
            partial,
        })
    }
}

fn owner(file: &File, byte: usize, inputs: &SourceQueryInputs) -> Owner {
    let buffer = &file.buffer;
    let mut row = CallerRow {
        depth: 0,
        name: "Top level".into(),
        location: SourceLocation::with_revision(&buffer.path, byte, buffer.revision),
        line: buffer.line_at_byte(byte),
        column: buffer.column_at_byte(byte),
        symbol: None,
        note: None,
    };
    let scope = file
        .calls
        .scopes
        .iter()
        .filter(|scope| scope.range.contains(&byte))
        .min_by_key(|scope| scope.range.len());
    let Some(scope) = scope else {
        if !file.calls.supported {
            row.name = "Reference".into();
        }
        row.note = Some(
            if file.calls.supported {
                "No enclosing callable"
            } else {
                "No parsed structure"
            }
            .into(),
        );
        return Owner {
            row,
            declaration: None,
        };
    };
    if scope.closure {
        row.name = file
            .calls
            .scopes
            .iter()
            .filter(|outer| {
                !outer.closure
                    && outer.range.start <= scope.range.start
                    && outer.range.end >= scope.range.end
            })
            .min_by_key(|outer| outer.range.len())
            .and_then(|outer| outer.name.as_ref())
            .and_then(|name| buffer.text.get(name.clone()))
            .map_or_else(|| "Closure".into(), |name| format!("Closure in {name}"));
        row.note = Some("Closure invocation is not indexed".into());
        return Owner {
            row,
            declaration: None,
        };
    }
    let name = scope.name.as_ref();
    row.name = name
        .and_then(|name| buffer.text.get(name.clone()))
        .unwrap_or("Callable")
        .to_owned();
    let occurrence = name.and_then(|name| {
        inputs
            .index
            .occurrence_at_byte(buffer, name.start)
            .filter(|occ| {
                occ.is_definition()
                    && occ
                        .range
                        .as_ref()
                        .and_then(|range| inputs.index.byte_range(buffer, range))
                        .is_some_and(|bytes| bytes.start == name.start)
            })
    });
    row.symbol = occurrence.map(|occ| SemanticSymbolID::new(&occ.symbol, &buffer.path));
    if row.symbol.is_none() {
        row.note = Some("Callable is not indexed".into());
    }
    Owner {
        row,
        declaration: name
            .map(|name| SourceLocation::with_revision(&buffer.path, name.start, buffer.revision)),
    }
}

#[cfg(test)]
mod tests;
