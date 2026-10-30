//! Interactive Critical Lens popup with stable geometry across loading updates.

use crate::{
    critical::{
        hover::{CriticalHover, HoverPhase},
        Criticality, SCIPSymbolHeader,
    },
    semantic::index::SemanticSymbolID,
    theme::AppTheme,
};
use egui::{pos2, vec2, Align2, Area, Context, Frame, Id, Order, Pos2, Sense, TextStyle, Ui};

#[derive(Default)]
pub(super) struct HoverPopup {
    serial: u64,
    origin: Pos2,
}

#[derive(Default)]
pub(super) struct PopupResponse {
    pub hovered: bool,
    pub destination: Option<SemanticSymbolID>,
}

impl HoverPopup {
    pub fn show(
        &mut self,
        ctx: &Context,
        hover: &CriticalHover,
        anchor: Pos2,
        theme: &AppTheme,
    ) -> PopupResponse {
        let Some(shown) = hover.presentation() else {
            return PopupResponse::default();
        };
        if self.serial != shown.serial {
            self.serial = shown.serial;
            let screen = ctx.screen_rect().shrink(8.0);
            self.origin = pos2(
                anchor
                    .x
                    .clamp(screen.left(), (screen.right() - 380.0).max(screen.left())),
                (anchor.y + 4.0).clamp(screen.top(), (screen.bottom() - 320.0).max(screen.top())),
            );
        }
        let popup = Area::new(Id::new("radiology_critical_hover"))
            .order(Order::Foreground)
            .movable(false)
            .interactable(true)
            .sense(Sense::hover())
            .fixed_pos(self.origin)
            .show(ctx, |ui| {
                Frame::popup(ui.style())
                    .show(ui, |ui| {
                        ui.set_width(364.0);
                        let target = &shown.target;
                        let mut destination = destination_row(
                            ui,
                            &target.display_name,
                            if target.definition.is_some() {
                                "Definition"
                            } else {
                                "external — definition unavailable"
                            },
                            target.definition.as_ref().map(|_| &target.symbol),
                            theme,
                        );
                        ui.separator();
                        match &shown.phase {
                            HoverPhase::Loading => {
                                ui.label("Summarizing…");
                            }
                            HoverPhase::Failed(error) => {
                                ui.label("Summary unavailable").on_hover_text(error);
                            }
                            HoverPhase::Ready(frontier) => {
                                ui.label(egui::RichText::new(frontier.status_line()).small())
                                    .on_hover_text(frontier.status_detail());
                                let rows = frontier
                                    .summary
                                    .calls
                                    .iter()
                                    .filter(|call| {
                                        call.classification.criticality == Criticality::Critical
                                    })
                                    .map(|call| (call, String::new()))
                                    .chain(frontier.paths.iter().map(|path| {
                                        (
                                            &path.call,
                                            path.via
                                                .iter()
                                                .map(|via| via.display_name.as_str())
                                                .collect::<Vec<_>>()
                                                .join(" › "),
                                        )
                                    }));
                                for (call, via) in rows.take(8) {
                                    let detail = call.definition.as_ref().map_or_else(
                                        || {
                                            SCIPSymbolHeader::parse(&call.callee.symbol)
                                                .and_then(|header| {
                                                    header.package_name().map(str::to_owned)
                                                })
                                                .unwrap_or_else(|| "external".into())
                                        },
                                        |definition| {
                                            format!(
                                                "{}:{}",
                                                definition.path,
                                                definition.range.start_line.saturating_add(1)
                                            )
                                        },
                                    );
                                    let detail = if via.is_empty() {
                                        detail
                                    } else {
                                        format!("via {via} · {detail}")
                                    };
                                    destination = destination_row(
                                        ui,
                                        &call.display_name,
                                        &detail,
                                        call.definition.as_ref().map(|_| &call.callee),
                                        theme,
                                    )
                                    .or(destination);
                                }
                                let count = frontier.annotation().count;
                                if count > 8 {
                                    ui.label(
                                        egui::RichText::new(format!("+{} more", count - 8)).small(),
                                    );
                                }
                            }
                        }
                        destination
                    })
                    .inner
            });
        PopupResponse {
            hovered: ctx
                .pointer_hover_pos()
                .is_some_and(|pos| popup.response.rect.contains(pos)),
            destination: popup.inner,
        }
    }
}

fn destination_row(
    ui: &mut Ui,
    name: &str,
    detail: &str,
    destination: Option<&SemanticSymbolID>,
    theme: &AppTheme,
) -> Option<SemanticSymbolID> {
    let label = format!("{name} · {detail}");
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), 24.0),
        if destination.is_some() {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, &label));
    if response.hovered() && destination.is_some() {
        ui.painter()
            .rect_filled(rect, 3.0_f32, ui.visuals().widgets.hovered.bg_fill);
    }
    let color = if destination.is_some() {
        theme.accent
    } else {
        theme.text_secondary
    };
    let mut job = egui::text::LayoutJob::simple_singleline(
        label.clone(),
        TextStyle::Body.resolve(ui.style()),
        color,
    );
    job.wrap.max_width = (rect.width() - 8.0).max(1.0);
    job.wrap.max_rows = 1;
    let galley = ui.fonts(|fonts| fonts.layout_job(job));
    let origin = Align2::LEFT_CENTER
        .align_size_within_rect(galley.size(), rect.shrink2(vec2(4.0, 0.0)))
        .min;
    ui.painter().galley(origin, galley, color);
    let clicked = response
        .on_hover_text(label)
        .on_hover_cursor(if destination.is_some() {
            egui::CursorIcon::PointingHand
        } else {
            egui::CursorIcon::Default
        })
        .clicked();
    clicked.then(|| destination.cloned()).flatten()
}
