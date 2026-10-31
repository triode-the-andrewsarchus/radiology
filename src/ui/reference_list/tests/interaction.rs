use super::*;
use crate::ui::test_support::{click, painted_frame};

struct Row {
    node: ReferenceHit,
    digest: &'static str,
    offset: f32,
    action: Option<ReferenceAction>,
}

impl Row {
    fn new() -> Self {
        Self {
            node: test_hit(0),
            digest: "original",
            offset: 0.0,
            action: None,
        }
    }

    fn frame(&mut self, ctx: &Context, events: Vec<Event>) {
        painted_frame(ctx, egui::vec2(260.0, 240.0), events, |ui| {
            ui.add_space(self.offset);
            self.action = render_row(
                ui,
                &AppTheme::dark(),
                &self.node,
                0,
                false,
                false,
                Some(self.digest),
            );
        });
    }
}

#[test]
fn reference_press_retires_on_replacement_source_change_or_row_movement() {
    for disclosure in [false, true] {
        for change in 0..3 {
            let ctx = Context::default();
            let mut row = Row::new();
            row.frame(&ctx, vec![]);
            let position = egui::pos2(if disclosure { 16.0 } else { 140.0 }, 20.0);
            let mut events = click(position);
            let release = events.split_off(2);
            row.frame(&ctx, events);
            match change {
                0 => row.node = test_hit(1),
                1 => row.digest = "new",
                _ => row.offset = 24.0,
            }
            row.frame(&ctx, release);
            assert_eq!(
                row.action, None,
                "old press activated replacement {change} (disclosure={disclosure})"
            );
            row.frame(&ctx, vec![]);
            row.frame(&ctx, click(position + egui::vec2(0.0, row.offset)));
            assert!(matches!(
                row.action,
                Some(
                    ReferenceAction::Select(0)
                        | ReferenceAction::Commit(Some(0))
                        | ReferenceAction::SetExpanded(0, true)
                )
            ));
        }
    }
}

#[test]
fn explicit_reference_and_disclosure_ids_remain_keyboard_usable() {
    let ctx = Context::default();
    let mut row = Row::new();
    row.frame(&ctx, vec![]);
    row.frame(&ctx, vec![crate::ui::commands::test_key(Key::Tab)]);
    let row_focus = ctx.memory(|memory| memory.focused()).unwrap();
    row.frame(&ctx, vec![crate::ui::commands::test_key(Key::Space)]);
    assert_eq!(row.action, Some(ReferenceAction::Select(0)));
    // Enter the disclosure from the row; full-list arrow ownership has separate coverage.
    ctx.memory_mut(|memory| memory.request_focus(row_focus));
    row.frame(&ctx, vec![crate::ui::commands::test_key(Key::Tab)]);
    row.frame(&ctx, vec![crate::ui::commands::test_key(Key::Space)]);
    assert_eq!(row.action, Some(ReferenceAction::SetExpanded(0, true)));
}
