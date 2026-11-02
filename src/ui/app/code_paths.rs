//! Selected-trace navigation through the existing source transaction and worker pool.
use super::*;
use crate::{
    background::LatestRead,
    code_paths::{resolver::CodePathDestination, CodePath},
    source::loader::{PreparedSource, SourceLoadRequest, SourceRead, SourceTarget},
};
use uuid::Uuid;

#[derive(Default)]
pub(super) struct CodePaths {
    selected: Option<Arc<CodePath>>,
    selected_step: Option<Uuid>,
    destinations: Vec<CodePathDestination>,
    error: Option<String>,
    job: LatestRead<Result<ReadyStep, String>>,
    owner: Option<Owner>,
}

struct Owner {
    root: PathBuf,
    review: Option<Arc<ReviewSnapshot>>,
    trace: Arc<CodePath>,
    origin: Option<Arc<SourceBuffer>>,
    caret: (usize, usize),
    step: Uuid,
}

struct ReadyStep {
    request: SourceLoadRequest,
    source: PreparedSource,
    destinations: Vec<CodePathDestination>,
}

impl RadiologyApp {
    /// Selects a valid trace for the attached repository, retiring pending navigation.
    ///
    /// The host loads/imports saved traces separately. Same-ID replacements also
    /// retire work so edited records cannot accept a previous request's completion.
    /// Returns `false` for invalid records or when no repository is ready.
    pub fn select_code_path(&mut self, trace: Option<Arc<CodePath>>) -> bool {
        if !self.repository_interaction_ready()
            || self.is_repository_opening()
            || trace.as_ref().is_some_and(|trace| !trace.is_valid())
        {
            return false;
        }
        self.code_paths = CodePaths {
            selected: trace,
            ..Default::default()
        };
        true
    }

    /// Returns the currently selected trace, independent of sidebar visibility.
    #[must_use]
    pub fn selected_code_path(&self) -> Option<&CodePath> {
        self.code_paths.selected.as_deref()
    }

    /// Returns the last successfully opened step in the selected trace.
    #[must_use]
    pub fn selected_code_path_step(&self) -> Option<Uuid> {
        self.code_paths.selected_step
    }

    /// Returns source-ordered conditional alternatives from the last accepted step.
    #[must_use]
    pub fn code_path_destinations(&self) -> &[CodePathDestination] {
        &self.code_paths.destinations
    }

    /// Returns the current step's visible resolution/preparation failure.
    #[must_use]
    pub fn code_path_error(&self) -> Option<&str> {
        self.code_paths.error.as_deref()
    }

    /// Returns whether the current step still owns a background request.
    #[must_use]
    pub fn is_code_path_loading(&self) -> bool {
        self.code_paths.job.is_running()
    }

    /// Resolves and opens a trace step without requiring a semantic index.
    ///
    /// `variant` is a zero-based conditional alternative in source order; use zero
    /// for the first definition. Selection/history change only after a successful
    /// source transaction. New navigation, caret movement, trace/root/review changes
    /// and cancellation retire both late success and late error publication.
    pub fn go_to_code_path_step(&mut self, id: Uuid, variant: usize) {
        if !self.repository_interaction_ready() || self.is_repository_opening() {
            return;
        }
        let (Some(root), Some(trace)) = (
            self.repository_root.clone(),
            self.code_paths.selected.clone(),
        ) else {
            return;
        };
        let Some(step) = trace.steps.iter().find(|step| step.id == id).cloned() else {
            return;
        };
        self.cancel_source_load();
        self.source_viewport.cancel_deferred_input();
        let owner = Owner {
            root: root.clone(),
            review: self.review.clone(),
            trace,
            origin: self.open_buffer.clone(),
            caret: (self.caret_line, self.caret_col),
            step: id,
        };
        let source = owner.review.as_ref().map_or_else(
            || SourceRead::Checkout(root),
            |review| SourceRead::Review(review.clone()),
        );
        let open = owner.origin.clone();
        let highlighter = self.highlighter.clone();
        let dark = self.theme.is_dark;
        self.code_paths.destinations.clear();
        self.code_paths.error = None;
        self.code_paths.owner = Some(owner);
        self.code_paths.job.spawn(move |cancel| {
            let resolved =
                crate::code_paths::resolver::resolve_step(&step, &source, open.as_ref(), cancel)
                    .map_err(|error| error.to_string())?;
            let destination = resolved
                .destinations
                .get(variant)
                .ok_or("This conditional definition is no longer available")?;
            let request = SourceLoadRequest {
                source,
                location: destination.location.clone(),
                target: SourceTarget::Byte(destination.location.byte_offset),
                native_highlights: None,
            };
            let prepared = crate::source::loader::prepare_loaded(
                request.clone(),
                resolved.buffer,
                Some(resolved.syntax),
                &highlighter,
                dark,
                cancel,
                std::time::Instant::now() + crate::source::loader::SOURCE_READ_TIMEOUT,
            )?;
            Ok(ReadyStep {
                request,
                source: prepared,
                destinations: resolved.destinations,
            })
        });
    }

    pub(super) fn cancel_code_path_navigation(&mut self) {
        self.code_paths.job.stop();
        self.code_paths.owner = None;
    }

    fn code_path_owner_is_current(&self, owner: &Owner) -> bool {
        self.repository_interaction_ready()
            && !self.is_repository_opening()
            && self.repository_root.as_ref() == Some(&owner.root)
            && same_snapshot(&owner.review, &self.review)
            && same_snapshot(&owner.origin, &self.open_buffer)
            && self
                .code_paths
                .selected
                .as_ref()
                .is_some_and(|trace| Arc::ptr_eq(trace, &owner.trace))
            && owner.caret == (self.caret_line, self.caret_col)
    }

    pub(super) fn poll_code_path_navigation(&mut self, ctx: &Context) {
        if self
            .code_paths
            .owner
            .as_ref()
            .is_some_and(|owner| !self.code_path_owner_is_current(owner))
        {
            self.cancel_code_path_navigation();
        }
        let result = match self.code_paths.job.poll() {
            Ok(result) => result,
            Err(_) => Some(Err("Code path worker stopped unexpectedly".into())),
        };
        if let Some(result) = result {
            if let Some(owner) = self.code_paths.owner.take() {
                match result {
                    Ok(ready) => {
                        let recolor = ready.source.dark_mode != self.theme.is_dark;
                        self.pending_navigation = Some(source::PendingNavigation {
                            request: ready.request,
                            intent: source::NavigationIntent {
                                record_history: true,
                                reveal: true,
                                flash_destination: true,
                                ..Default::default()
                            },
                            trait_owner: None,
                        });
                        self.publish_source(ready.source);
                        self.code_paths.selected_step = Some(owner.step);
                        self.code_paths.destinations = ready.destinations;
                        if recolor {
                            self.queue_rehighlight();
                        }
                    }
                    Err(error) => self.code_paths.error = Some(error),
                }
            }
            ctx.request_repaint();
        }
        if self.is_code_path_loading() {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
        }
    }
}

fn same_snapshot<T>(a: &Option<Arc<T>>, b: &Option<Arc<T>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests;
