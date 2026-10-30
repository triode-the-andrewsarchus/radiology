//! Callable summaries from lexical ownership and exact indexed callee identities.
//!
//! Syntax establishes sites and ownership; spelling never establishes an edge.
//! Summaries retain partial coverage instead of allowing missing facts to prove zero.

use super::{callees::CalleeResolver, Criticality, CriticalityClassification, CriticalityPolicy};
use crate::semantic::{index::SemanticSymbolID, ScipRange, SemanticIndex};
use crate::source::{ReviewRevision, SourceBuffer};
use crate::syntax::calls::{CallSites, CallableScope};
use std::{
    collections::HashMap,
    ops::Range,
    sync::atomic::{AtomicBool, Ordering},
};

/// Reasons a callable has no body available to summarize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoBodyReason {
    /// Defined outside the attached repository.
    External,
    /// Parsed declaration without a body.
    Abstract,
    /// Missing or mismatched parsed structure.
    NoSyntax,
    /// Source could not be read.
    Unreadable,
    /// Semantic facts are absent, stale, or use an unsupported encoding/revision.
    NotIndexed,
    /// No call query supports this language.
    UnsupportedLanguage,
}

/// Gaps that make reachable counts lower bounds rather than proven totals.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CoverageGaps {
    /// Syntactic sites without an indexed callee, including dynamic calls.
    pub unresolved_sites: usize,
    /// Macro expansions that were not analyzed.
    pub macro_sites: usize,
    /// Repository callees with missing, abstract, or unreadable bodies.
    pub missing_bodies: usize,
    /// Distinct unclassified callees, which may hide critical behavior.
    pub unknown_callees: usize,
    /// Calls owned by nested functions/closures whose execution is conditional.
    pub conditional_sites: usize,
    /// Syntax errors may hide additional sites.
    pub parse_errors: bool,
    /// A traversal depth/body limit was reached.
    pub truncated: bool,
}

impl CoverageGaps {
    /// Returns whether no gap was found.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }

    pub(super) fn merge(&mut self, other: &Self) {
        self.unresolved_sites += other.unresolved_sites;
        self.macro_sites += other.macro_sites;
        self.missing_bodies += other.missing_bodies;
        self.unknown_callees += other.unknown_callees;
        self.conditional_sites += other.conditional_sites;
        self.parse_errors |= other.parse_errors;
        self.truncated |= other.truncated;
    }
}

/// Coverage of a callable summary or a bounded reachable frontier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallCoverage {
    /// Every relevant site resolved and every traversed body was parsed.
    Complete,
    /// The count is a lower bound for the stated reasons.
    Partial(CoverageGaps),
    /// No body was available to summarize.
    NoBody(NoBodyReason),
}

impl CallCoverage {
    pub(super) fn from_gaps(gaps: CoverageGaps) -> Self {
        if gaps.is_empty() {
            Self::Complete
        } else {
            Self::Partial(gaps)
        }
    }
}

/// A stable in-repository semantic destination, in its document's SCIP encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallDefinition {
    /// Repository-relative source path.
    pub path: String,
    /// Exact indexed declaration coordinates.
    pub range: ScipRange,
}

/// One distinct resolved direct callee, retaining its first source site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectCall {
    /// Global or document-scoped SCIP identity.
    pub callee: SemanticSymbolID,
    /// Indexed display name.
    pub display_name: String,
    /// Decision including policy provenance.
    pub classification: CriticalityClassification,
    /// First callee-name byte in the owner's file.
    pub first_site_byte: usize,
    /// Number of direct sites for this identity.
    pub site_count: usize,
    /// Canonical repository definition, if one exists.
    pub definition: Option<CallDefinition>,
    /// True if at least one site is a macro invocation.
    pub is_macro: bool,
}

/// One directly nested callable, which is not an unconditional call edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalCallable {
    /// Parsed callable scope.
    pub scope: CallableScope,
    /// Sites directly owned by this scope, excluding deeper callables.
    pub site_count: usize,
}

/// Exact source identity and lexical range summarized by a callable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableBody {
    /// Repository-relative path.
    pub path: String,
    /// Revision of the immutable source snapshot.
    pub revision: Option<ReviewRevision>,
    /// SHA-256 digest of original source bytes.
    pub content_digest: String,
    /// Complete callable declaration/closure range.
    pub range: Range<usize>,
}

/// Distinct direct calls and conditional nested callables in source order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableSummary {
    /// Indexed callable identity, when available.
    pub callable: Option<SemanticSymbolID>,
    /// Exact body identity, absent for no-body results.
    pub body: Option<CallableBody>,
    /// One entry per callee in order of its first direct site.
    pub calls: Vec<DirectCall>,
    /// Directly nested closures/functions, excluded from direct edges.
    pub conditional: Vec<ConditionalCallable>,
    /// Reasons a complete count cannot be established.
    pub coverage: CallCoverage,
}

impl CallableSummary {
    /// Creates an explicit no-body result without claiming a proven zero.
    #[must_use]
    pub fn no_body(reason: NoBodyReason, callable: Option<SemanticSymbolID>) -> Self {
        Self {
            callable,
            body: None,
            calls: Vec::new(),
            conditional: Vec::new(),
            coverage: CallCoverage::NoBody(reason),
        }
    }
}

/// Summarizes a parsed callable using the same resolver as source projection.
///
/// `scope_index` selects an exact scope in `sites`, never a guessed line extent.
/// Missing semantic data and Before revisions cannot establish call edges.
///
/// # Errors
/// Returns cancellation before publication; no files or caches are mutated.
pub fn summarize(
    buffer: &SourceBuffer,
    sites: &CallSites,
    scope_index: usize,
    callable: Option<SemanticSymbolID>,
    index: Option<&SemanticIndex>,
    policy: &CriticalityPolicy,
    cancel: &AtomicBool,
) -> Result<CallableSummary, String> {
    check_cancel(cancel)?;
    let fail = |reason| Ok(CallableSummary::no_body(reason, callable.clone()));
    if sites.error.is_some() {
        return fail(NoBodyReason::NoSyntax);
    }
    if !sites.supported {
        return fail(NoBodyReason::UnsupportedLanguage);
    }
    if !sites.matches(buffer) {
        return fail(NoBodyReason::NoSyntax);
    }
    let Some(scope) = sites.scopes.get(scope_index) else {
        return fail(NoBodyReason::NoSyntax);
    };
    if !scope.has_body {
        return fail(NoBodyReason::Abstract);
    }
    let resolver = CalleeResolver::new(buffer, index);
    let Some(index) = resolver
        .index
        .filter(|index| index.column_encoding(&buffer.path).is_some())
    else {
        return fail(NoBodyReason::NotIndexed);
    };
    let mut gaps = CoverageGaps {
        parse_errors: sites.parse_errors,
        ..Default::default()
    };
    let mut calls = Vec::<DirectCall>::new();
    let mut by_id = HashMap::<SemanticSymbolID, usize>::new();
    for site in sites
        .sites
        .iter()
        .filter(|site| site.owner == Some(scope_index))
    {
        check_cancel(cancel)?;
        gaps.macro_sites += usize::from(site.kind == "macro");
        let Some(callee) = site.name.as_ref().and_then(|name| resolver.resolve(name)) else {
            gaps.unresolved_sites += 1;
            continue;
        };
        if let Some(&slot) = by_id.get(callee) {
            calls[slot].site_count += 1;
            calls[slot].is_macro |= site.kind == "macro";
            continue;
        }
        let classification = policy.classify(
            &callee.symbol,
            index
                .symbol_info_for_id(callee)
                .map(|info| info.signature.as_str()),
            index.definitions.contains_key(callee),
        );
        gaps.unknown_callees += usize::from(classification.criticality == Criticality::Unknown);
        by_id.insert(callee.clone(), calls.len());
        calls.push(DirectCall {
            callee: callee.clone(),
            display_name: index.display_name(callee),
            classification,
            first_site_byte: site.name.as_ref().unwrap().start,
            site_count: 1,
            definition: index
                .find_definition_for_id(callee)
                .map(|(path, range)| CallDefinition {
                    path: path.to_owned(),
                    range: range.clone(),
                }),
            is_macro: site.kind == "macro",
        });
    }
    let conditional = conditionals(sites, scope_index, &mut gaps);
    Ok(CallableSummary {
        callable,
        body: Some(CallableBody {
            path: buffer.path.clone(),
            revision: buffer.revision,
            content_digest: buffer.content_digest().to_owned(),
            range: scope.range.clone(),
        }),
        calls,
        conditional,
        coverage: CallCoverage::from_gaps(gaps),
    })
}

fn conditionals(
    sites: &CallSites,
    owner: usize,
    gaps: &mut CoverageGaps,
) -> Vec<ConditionalCallable> {
    let range = &sites.scopes[owner].range;
    let mut counts = vec![0; sites.scopes.len()];
    for site in &sites.sites {
        if let Some(scope) = site.owner {
            counts[scope] += 1;
            let nested = &sites.scopes[scope].range;
            if scope != owner && range.start <= nested.start && nested.end <= range.end {
                gaps.conditional_sites += 1;
            }
        }
    }
    let mut result = Vec::new();
    let mut covered_until = range.start;
    for (i, scope) in sites.scopes.iter().enumerate() {
        if i != owner
            && scope.range.start >= covered_until
            && scope.range.start >= range.start
            && scope.range.end <= range.end
        {
            covered_until = scope.range.end;
            result.push(ConditionalCallable {
                scope: scope.clone(),
                site_count: counts[i],
            });
        }
    }
    result
}

pub(super) fn check_cancel(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("Call graph cancelled".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
