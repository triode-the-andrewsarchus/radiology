//! Pointer-dwell Critical Lens summaries independent of any window toolkit.

use super::{
    frontier::CriticalFrontier,
    graph::{CallCoverage, CallDefinition, CallableSummary, NoBodyReason},
    worker::{CallGraphWorker, GraphInputs},
    Criticality, CriticalityPolicy,
};
use crate::{background::LatestRead, semantic::index::SemanticSymbolID, source::ReviewRevision};
use std::{ops::Range, sync::Arc, time::Duration};

const DWELL: f64 = 0.350;
const GRACE: f64 = 0.700;

/// Exact indexed occurrence captured before pointer dwell begins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverTarget {
    /// Callable identity; document-local variables are excluded.
    pub symbol: SemanticSymbolID,
    /// Original byte range of the hovered occurrence.
    pub range: Range<usize>,
    /// Indexed display name.
    pub display_name: String,
    /// Canonical repository definition, absent for external symbols.
    pub definition: Option<CallDefinition>,
}

/// Current background summary phase, without source/caret mutation.
pub enum HoverPhase {
    /// A repository body is being summarized off-thread.
    Loading,
    /// A complete or explicitly partial/no-body frontier.
    Ready(Box<CriticalFrontier>),
    /// Recoverable worker failure.
    Failed(String),
}

/// Stable popup identity and content across loading/result updates.
pub struct HoverPresentation {
    /// Monotonic presentation identity; unchanged by completion.
    pub serial: u64,
    /// Original occurrence and semantic destination.
    pub target: HoverTarget,
    /// Current summary state.
    pub phase: HoverPhase,
}

/// Owns dwell, grace and generation-safe background publication.
///
/// Hosts supply a monotonic clock in seconds and call `update` on pointer/input
/// changes and requested wakeups. Entering the popup cancels closure; clicking a
/// destination remains a separate host navigation action. No method moves source.
#[derive(Default)]
pub struct CriticalHover {
    inputs: Option<Arc<GraphInputs>>,
    policy: Option<CriticalityPolicy>,
    last_byte: Option<usize>,
    pending: Option<(HoverTarget, f64)>,
    close_at: Option<f64>,
    presentation: Option<HoverPresentation>,
    job: LatestRead<Result<CriticalFrontier, String>>,
    serial: u64,
}

impl CriticalHover {
    /// Retires hover when the captured source/index/storage/policy changes.
    pub fn configure(&mut self, inputs: Option<Arc<GraphInputs>>) {
        let same = match (&self.inputs, &inputs) {
            (Some(old), Some(new)) => Arc::ptr_eq(old, new),
            (None, None) => true,
            _ => false,
        };
        if same {
            return;
        }
        self.dismiss();
        self.last_byte = None;
        self.policy = inputs.as_ref().map(|inputs| {
            CriticalityPolicy::for_index(Some(&inputs.index), inputs.overrides.clone())
        });
        self.inputs = inputs;
    }

    /// Cancels pending work and presentation, retaining idle-pointer suppression.
    pub fn dismiss(&mut self) {
        self.job.stop();
        self.pending = None;
        self.close_at = None;
        self.presentation = None;
    }

    /// Returns the stable current popup, if its dwell completed.
    #[must_use]
    pub fn presentation(&self) -> Option<&HoverPresentation> {
        self.presentation.as_ref()
    }

    /// Returns the source occurrence whose geometry should anchor the next popup.
    #[must_use]
    pub fn anchor_range(&self) -> Option<&Range<usize>> {
        self.pending
            .as_ref()
            .map(|(target, _)| &target.range)
            .or_else(|| self.presentation.as_ref().map(|shown| &shown.target.range))
    }

    /// Advances pointer tracking, deadlines and worker publication.
    ///
    /// `interrupted` means a source/outside click, key or scroll. It suppresses
    /// reopening at the same idle byte until a subsequent pointer movement.
    pub fn update(
        &mut self,
        byte: Option<usize>,
        popup_hovered: bool,
        interrupted: bool,
        now: f64,
    ) {
        self.poll();
        if interrupted {
            self.dismiss();
            self.last_byte = byte;
            return;
        }
        if popup_hovered {
            self.pending = None;
            self.close_at = None;
            self.last_byte = None;
        } else {
            if byte != self.last_byte {
                self.last_byte = byte;
                self.track(byte, now);
            }
            let on_anchor = self
                .presentation
                .as_ref()
                .is_some_and(|shown| byte.is_some_and(|byte| shown.target.range.contains(&byte)));
            if self.pending.is_none()
                && !on_anchor
                && self.presentation.is_some()
                && self.close_at.is_none()
            {
                self.close_at = Some(now + GRACE);
            }
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|(_, deadline)| *deadline <= now)
        {
            let (target, _) = self.pending.take().unwrap();
            self.present(target);
        }
        if self.close_at.is_some_and(|deadline| deadline <= now) {
            self.dismiss();
        }
    }

    /// Returns the next useful repaint deadline, with no idle polling.
    #[must_use]
    pub fn next_wake(&self, now: f64) -> Option<Duration> {
        let deadline = self
            .pending
            .as_ref()
            .map(|(_, time)| *time)
            .into_iter()
            .chain(self.close_at)
            .reduce(f64::min);
        let seconds = deadline.map(|deadline| (deadline - now).max(0.001));
        let seconds = if self.job.is_running() {
            Some(seconds.map_or(0.025, |value| value.min(0.025)))
        } else {
            seconds
        };
        seconds.map(Duration::from_secs_f64)
    }

    fn track(&mut self, byte: Option<usize>, now: f64) {
        let target = byte.and_then(|byte| self.target_at(byte));
        if let Some(target) = target {
            self.close_at = None;
            if self
                .presentation
                .as_ref()
                .is_some_and(|shown| shown.target == target)
            {
                self.pending = None;
            } else if !self
                .pending
                .as_ref()
                .is_some_and(|(pending, _)| *pending == target)
            {
                self.pending = Some((target, now + DWELL));
            }
        } else {
            self.pending = None;
        }
    }

    fn target_at(&self, byte: usize) -> Option<HoverTarget> {
        let inputs = self.inputs.as_ref()?;
        if inputs.buffer.revision == Some(ReviewRevision::Before)
            || !inputs.calls.matches(&inputs.buffer)
        {
            return None;
        }
        let occurrence = inputs.index.occurrence_at_byte(&inputs.buffer, byte)?;
        let symbol = SemanticSymbolID::new(&occurrence.symbol, &inputs.buffer.path);
        if symbol.is_local() || !symbol.is_callable(&inputs.index) {
            return None;
        }
        let definition = inputs
            .index
            .find_definition_for_id(&symbol)
            .map(|(path, range)| CallDefinition {
                path: path.to_owned(),
                range: range.clone(),
            });
        let classification = self.policy.as_ref()?.classify(
            &symbol.symbol,
            inputs
                .index
                .symbol_info_for_id(&symbol)
                .map(|info| info.signature.as_str()),
            definition.is_some(),
        );
        if definition.is_none() && classification.criticality != Criticality::Critical {
            return None;
        }
        Some(HoverTarget {
            range: inputs
                .index
                .byte_range(&inputs.buffer, occurrence.range.as_ref()?)?,
            display_name: inputs.index.display_name(&symbol),
            symbol,
            definition,
        })
    }

    fn present(&mut self, target: HoverTarget) {
        self.job.stop();
        self.serial = self.serial.wrapping_add(1);
        let phase = if target.definition.is_none() {
            HoverPhase::Ready(Box::new(CriticalFrontier {
                summary: CallableSummary::no_body(
                    NoBodyReason::External,
                    Some(target.symbol.clone()),
                ),
                paths: Vec::new(),
                unknown: Vec::new(),
                coverage: CallCoverage::NoBody(NoBodyReason::External),
                traversed_bodies: 0,
            }))
        } else {
            let inputs = self.inputs.as_ref().unwrap().clone();
            let symbol = target.symbol.clone();
            self.job.spawn(move |cancel| {
                CallGraphWorker::new((*inputs).clone()).frontier_of(&symbol, cancel)
            });
            HoverPhase::Loading
        };
        self.presentation = Some(HoverPresentation {
            serial: self.serial,
            target,
            phase,
        });
        self.close_at = None;
    }

    fn poll(&mut self) {
        let phase = match self.job.poll() {
            Ok(Some(Ok(frontier))) => HoverPhase::Ready(Box::new(frontier)),
            Ok(Some(Err(error))) => HoverPhase::Failed(error),
            Err(_) => HoverPhase::Failed("Critical hover worker stopped unexpectedly".into()),
            Ok(None) => return,
        };
        if let Some(shown) = &mut self.presentation {
            shown.phase = phase;
        }
    }
}

#[cfg(test)]
mod tests;
