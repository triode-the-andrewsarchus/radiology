//! Bounded breadth-first critical frontiers over exact callable summaries.

use super::graph::{
    check_cancel, CallCoverage, CallDefinition, CallableSummary, CoverageGaps, DirectCall,
};
use super::{CountAnnotation, CountCompleteness, Criticality};
use crate::semantic::index::SemanticSymbolID;
use std::{
    collections::{HashSet, VecDeque},
    sync::atomic::AtomicBool,
};

/// An ordinary repository callable on the shortest route to an operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViaStep {
    /// Exact SCIP identity of the intermediary.
    pub callee: SemanticSymbolID,
    /// Indexed display name.
    pub display_name: String,
    /// Canonical repository definition.
    pub definition: Option<CallDefinition>,
}

/// A critical or unknown callee and the ordinary intermediaries that reach it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticalPath {
    /// Callee identity, decision and first site in its immediate owner's source.
    pub call: DirectCall,
    /// Shortest intermediary chain, empty for a direct call.
    pub via: Vec<ViaStep>,
}

/// Reachable critical operations with explicit bounds and coverage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticalFrontier {
    /// Original direct summary of the root callable.
    pub summary: CallableSummary,
    /// Distinct indirect critical callees, in breadth-first source order.
    pub paths: Vec<CriticalPath>,
    /// Distinct unclassified callees at any depth, never traversed or counted.
    pub unknown: Vec<CriticalPath>,
    /// Complete, partial lower bound, or no-body result.
    pub coverage: CallCoverage,
    /// Number of repository bodies requested beyond the root.
    pub traversed_bodies: usize,
}

impl CriticalFrontier {
    /// Returns the distinct union of direct and indirectly reachable critical callees.
    #[must_use]
    pub fn annotation(&self) -> CountAnnotation {
        let count = self
            .summary
            .calls
            .iter()
            .filter(|call| call.classification.criticality == Criticality::Critical)
            .map(|call| &call.callee)
            .chain(self.paths.iter().map(|path| &path.call.callee))
            .collect::<HashSet<_>>()
            .len();
        CountAnnotation {
            count,
            completeness: match self.coverage {
                CallCoverage::Complete => CountCompleteness::Proven,
                CallCoverage::Partial(_) => CountCompleteness::Partial,
                CallCoverage::NoBody(_) => CountCompleteness::NoBody,
            },
        }
    }
}

/// Limits match Swift's default four-level, forty-eight-body frontier.
#[derive(Debug, Clone, Copy)]
pub struct FrontierLimits {
    /// Maximum call depth, including the root's direct calls as depth one.
    pub depth: usize,
    /// Maximum additional bodies loaded beyond the root.
    pub bodies: usize,
}

impl Default for FrontierLimits {
    fn default() -> Self {
        Self {
            depth: 4,
            bodies: 48,
        }
    }
}

struct Traversal {
    gaps: CoverageGaps,
    visited: HashSet<SemanticSymbolID>,
    critical_seen: HashSet<SemanticSymbolID>,
    unknown_seen: HashSet<SemanticSymbolID>,
    paths: Vec<CriticalPath>,
    unknown: Vec<CriticalPath>,
    traversed: usize,
    limits: FrontierLimits,
}

impl Traversal {
    fn merge(&mut self, coverage: &CallCoverage) {
        match coverage {
            CallCoverage::Complete => {}
            CallCoverage::Partial(gaps) => {
                // Count unknown identities once across diamonds and cycles.
                let mut gaps = gaps.clone();
                gaps.unknown_callees = 0;
                self.gaps.merge(&gaps);
            }
            CallCoverage::NoBody(_) => self.gaps.missing_bodies += 1,
        }
    }

    fn emit(&mut self, call: &DirectCall, via: &[ViaStep]) {
        let path = || CriticalPath {
            call: call.clone(),
            via: via.to_vec(),
        };
        match call.classification.criticality {
            Criticality::Critical if self.critical_seen.insert(call.callee.clone()) => {
                self.paths.push(path())
            }
            Criticality::Unknown if self.unknown_seen.insert(call.callee.clone()) => {
                self.unknown.push(path())
            }
            _ => {}
        }
    }

    fn should_expand(&mut self, call: &DirectCall, depth: usize) -> bool {
        if call.is_macro || call.definition.is_none() || !self.visited.insert(call.callee.clone()) {
            return false;
        }
        if depth + 1 >= self.limits.depth || self.traversed >= self.limits.bodies {
            self.gaps.truncated = true;
            return false;
        }
        self.traversed += 1;
        true
    }
}

/// Traverses ordinary repository callees, stopping at critical and unknown leaves.
///
/// Cycles and diamonds load each ordinary body once. First discovery wins, making
/// paths shortest and stable in breadth-first source order. External ordinary
/// plumbing is a leaf; missing repository bodies and bounds make coverage partial.
/// Conditional nested callables are never traversed as execution edges.
///
/// # Errors
/// Returns cancellation or a provider error without publishing a partial result.
pub fn build(
    root: CallableSummary,
    limits: FrontierLimits,
    cancel: &AtomicBool,
    mut body: impl FnMut(&SemanticSymbolID) -> Result<CallableSummary, String>,
) -> Result<CriticalFrontier, String> {
    check_cancel(cancel)?;
    let mut state = Traversal {
        gaps: Default::default(),
        visited: root.callable.iter().cloned().collect(),
        critical_seen: root
            .calls
            .iter()
            .filter(|call| call.classification.criticality == Criticality::Critical)
            .map(|call| call.callee.clone())
            .collect(),
        unknown_seen: Default::default(),
        paths: Vec::new(),
        unknown: Vec::new(),
        traversed: 0,
        limits,
    };
    state.merge(&root.coverage);
    let mut queue = VecDeque::from([(root.clone(), Vec::new(), 0)]);
    while let Some((summary, via, depth)) = queue.pop_front() {
        check_cancel(cancel)?;
        for call in &summary.calls {
            check_cancel(cancel)?;
            if call.classification.criticality != Criticality::Ordinary {
                state.emit(call, &via);
                continue;
            }
            if !state.should_expand(call, depth) {
                continue;
            }
            let summary = body(&call.callee)?;
            check_cancel(cancel)?;
            if summary.callable.as_ref() != Some(&call.callee) {
                state.gaps.missing_bodies += 1;
                continue;
            }
            state.merge(&summary.coverage);
            let mut chain = via.clone();
            chain.push(ViaStep {
                callee: call.callee.clone(),
                display_name: call.display_name.clone(),
                definition: call.definition.clone(),
            });
            queue.push_back((summary, chain, depth + 1));
        }
    }
    state.gaps.unknown_callees = state.unknown.len();
    let coverage = match root.coverage {
        CallCoverage::NoBody(reason) => CallCoverage::NoBody(reason),
        _ => CallCoverage::from_gaps(state.gaps),
    };
    Ok(CriticalFrontier {
        summary: root,
        paths: state.paths,
        unknown: state.unknown,
        coverage,
        traversed_bodies: state.traversed,
    })
}

#[cfg(test)]
mod tests;
