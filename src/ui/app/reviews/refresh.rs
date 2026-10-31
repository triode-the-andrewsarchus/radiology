//! Prepare against current source, then commit using the latest caret and pins.

use super::*;
use crate::{
    background::LatestRead,
    pull_request::refresh::{self, RefreshedSource},
    source::loader::{PreparedSource, SourceLoadRequest, SourceRead, SourceTarget},
};

#[derive(Default)]
pub(in crate::ui::app) struct ReviewRefresh {
    candidate: Option<Arc<ReviewSnapshot>>,
    job: LatestRead<Result<PreparedRefresh, String>>,
    pub notice: Option<String>,
}

struct PreparedRefresh {
    original: Option<Arc<SourceBuffer>>,
    mapped: Option<RefreshedSource>,
    source: Option<PreparedSource>,
    dark_mode: bool,
}

impl ReviewRefresh {
    pub(super) fn queue(&mut self, candidate: Arc<ReviewSnapshot>) {
        *self = Self {
            candidate: Some(candidate),
            ..Default::default()
        };
    }
}

impl RadiologyApp {
    pub(in crate::ui::app) fn poll_review_refresh(&mut self, ctx: &Context) {
        let Some(candidate) = self.review_refresh.candidate.clone() else {
            return;
        };
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
        if self.source_loader.is_running() || self.pending_navigation.is_some() {
            return;
        }
        if !self.review_refresh.job.is_running() {
            self.prepare_review_refresh(candidate);
            return;
        }
        match self.review_refresh.job.poll() {
            Ok(Some(Ok(prepared))) if self.refresh_source_is_current(&prepared) => {
                self.review_refresh.candidate = None;
                self.pending_pr = None;
                self.commit_review_refresh(candidate, prepared);
            }
            Ok(Some(Ok(_))) => {} // Source or theme moved: reconcile again next frame.
            Ok(Some(Err(error))) => self.fail_review_refresh(error),
            Err(_) => self.fail_review_refresh("Refresh source worker stopped unexpectedly".into()),
            Ok(None) => {}
        }
    }

    fn prepare_review_refresh(&mut self, candidate: Arc<ReviewSnapshot>) {
        let Some(old) = self.review.clone() else {
            self.cancel_review_preparation();
            return;
        };
        let original = self.open_buffer.clone();
        let highlighter = self.highlighter.clone();
        let dark_mode = self.theme.is_dark;
        self.review_refresh.job.spawn(move |cancel| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            let mapped = original
                .as_ref()
                .map(|original| refresh::prepare(&old, &candidate, original, cancel, deadline))
                .transpose()?
                .flatten();
            let source = mapped
                .as_ref()
                .map(|mapped| {
                    let request = source_request(candidate.clone(), &mapped.buffer);
                    crate::source::loader::prepare_loaded(
                        request,
                        mapped.buffer.clone(),
                        &highlighter,
                        dark_mode,
                        cancel,
                        deadline,
                    )
                })
                .transpose()?;
            Ok(PreparedRefresh {
                original,
                mapped,
                source,
                dark_mode,
            })
        });
    }

    fn refresh_source_is_current(&self, prepared: &PreparedRefresh) -> bool {
        prepared.dark_mode == self.theme.is_dark
            && match (&prepared.original, &self.open_buffer) {
                (Some(old), Some(current)) => Arc::ptr_eq(old, current),
                (None, None) => true,
                _ => false,
            }
    }

    fn fail_review_refresh(&mut self, error: String) {
        self.cancel_review_preparation();
        self.error = Some(format!("Unable to refresh review: {error}"));
    }

    fn commit_review_refresh(
        &mut self,
        candidate: Arc<ReviewSnapshot>,
        mut prepared: PreparedRefresh,
    ) {
        let notice = prepared
            .mapped
            .as_ref()
            .and_then(|mapped| mapped.notice.clone());
        let restoration = prepared
            .original
            .as_ref()
            .zip(prepared.mapped.as_ref())
            .map(|(original, mapped)| {
                let caret = original.byte_offset(self.caret_line, self.caret_col);
                let viewport = self.source_viewport.top_byte(original).unwrap_or(caret);
                (
                    refresh::map_byte(original, &mapped.buffer, mapped.changes.as_ref(), caret),
                    refresh::map_byte(original, &mapped.buffer, mapped.changes.as_ref(), viewport),
                    self.pin_store.retaining_current(original, &mapped.buffer),
                )
            });
        let sidebar = self.sidebar_tab;
        self.install_review(candidate.clone());
        if let (Some((caret, viewport, pins)), Some(mut source)) =
            (restoration, prepared.source.take())
        {
            self.sidebar_tab = sidebar;
            source.destination.byte_offset = caret;
            self.pin_store = pins;
            self.pending_navigation = Some(source::PendingNavigation {
                request: source_request(candidate, &source.buffer),
                intent: source::NavigationIntent {
                    record_history: true,
                    reveal: true,
                    ..Default::default()
                },
            });
            let buffer = source.buffer.clone();
            self.publish_source(source);
            self.scroll_to_line = None;
            self.test_code.reveal_byte(&buffer, viewport);
            self.fold_store
                .reveal_byte(viewport, &buffer, &self.fold_ranges);
            self.source_viewport.restore_top(&buffer, viewport);
            self.review_refresh.notice = notice;
        } else {
            self.review_refresh.notice = prepared.original.is_some().then(|| {
                "Previously open file is absent from the refreshed PR. Choose a changed file."
                    .into()
            });
        }
    }

    pub(in crate::ui::app) fn show_review_notice(&mut self, ctx: &Context) {
        if let Some(notice) = self.review_refresh.notice.clone() {
            TopBottomPanel::top("review_refresh_notice").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(notice);
                    if ui.small_button("Dismiss").clicked() {
                        self.review_refresh.notice = None;
                    }
                });
            });
        }
    }
}

fn source_request(snapshot: Arc<ReviewSnapshot>, buffer: &SourceBuffer) -> SourceLoadRequest {
    SourceLoadRequest {
        source: SourceRead::Review(snapshot),
        location: buffer.location_at_byte(0),
        target: SourceTarget::Byte(0),
        native_highlights: None,
    }
}
