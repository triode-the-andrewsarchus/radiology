//! Pre-frame ownership and ordered replay shared by standalone and hosted dialogs.

use super::ModalAction;
use crate::ui::commands::{keyboard_event, take_input_actions};
use egui::{Context, Event, Key, RawInput};

/// Owns input for one modal lifetime, before egui's native focus processing.
///
/// Call [`Self::capture_input`] before `Context::run` (or from the host's raw input
/// hook), then render the dialog through [`Self::show`]. Keep this instance until
/// the modal closes. The host owns the backdrop and the focus-return destination.
/// Rendering a picker alone cannot undo focus navigation already queued by egui.
///
/// # Examples
///
/// ```no_run
/// use radiology::{theme::AppTheme, ui::modals::{GoToLineModal, ModalInput}};
/// let ctx = egui::Context::default();
/// let mut owner = ModalInput::default();
/// let mut dialog = GoToLineModal::new(1);
/// let mut raw = egui::RawInput::default(); // Input supplied by the host each frame.
/// owner.capture_input(&mut raw);
/// let mut action = None;
/// let output = ctx.run(raw, |ctx| {
///     action = owner.show(ctx, |ctx| dialog.show(ctx, &AppTheme::dark(), 100));
/// });
/// ```
#[derive(Debug, Default, Clone)]
pub struct ModalInput {
    input: Vec<Event>,
    registered: bool,
}

impl ModalInput {
    /// Captures keyboard and IME text before egui can route them to another owner.
    ///
    /// Composed text stays ordered with ordinary text, navigation and submission.
    ///
    /// Initial pointer gestures are retired until the first measurement pass.
    /// Other pointer input remains available to the host's modal surface.
    pub fn capture_input(&mut self, input: &mut RawInput) {
        if !self.registered {
            input.events.retain(|event| {
                !matches!(
                    event,
                    Event::PointerMoved(_)
                        | Event::MouseMoved(_)
                        | Event::PointerButton { .. }
                        | Event::Scroll(_)
                        | Event::Zoom(_)
                        | Event::Touch { .. }
                        | Event::MouseWheel { .. }
                )
            });
        }
        self.input
            .extend(take_input_actions(&mut input.events, |event| {
                keyboard_event(event).then(|| event.clone())
            }));
    }

    pub(crate) fn is_registered(&self) -> bool {
        self.registered
    }

    /// Replays owned input and renders the modal after its first measurement pass.
    ///
    /// Tab is delivered before later editing/submission so dialog traversal can
    /// establish the next owner. Closing retires keyboard, pointer and scroll input
    /// from that frame, and resets this owner for its next modal lifetime.
    pub fn show(
        &mut self,
        ctx: &Context,
        render: impl FnOnce(&Context) -> Option<ModalAction>,
    ) -> Option<ModalAction> {
        if self.registered {
            let end = self
                .input
                .iter()
                .position(|event| {
                    matches!(
                        event,
                        Event::Key {
                            key: Key::Tab,
                            pressed: true,
                            ..
                        }
                    )
                })
                .map_or(self.input.len(), |index| index + 1);
            ctx.input_mut(|input| input.events.extend(self.input.drain(..end)));
            if !self.input.is_empty() {
                ctx.request_repaint();
            }
        } else {
            ctx.stop_dragging();
            self.input.extend(ctx.input_mut(|input| {
                take_input_actions(&mut input.events, |event| {
                    keyboard_event(event).then(|| event.clone())
                })
            }));
            self.registered = true;
            ctx.request_repaint();
        }
        let action = render(ctx);
        ctx.input_mut(|input| input.events.retain(|event| !keyboard_event(event)));
        if action.is_some() {
            ctx.stop_dragging();
            ctx.input_mut(|input| {
                input.events.clear();
                input.pointer = Default::default();
                input.raw_scroll_delta = egui::Vec2::ZERO;
                input.smooth_scroll_delta = egui::Vec2::ZERO;
            });
            *self = Self::default();
        }
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        theme::AppTheme,
        ui::{commands::test_key, modals::GoToLineModal},
    };

    #[test]
    fn hosted_opening_batches_and_tab_submission_never_reach_background_input() {
        let mut select_all = test_key(Key::A);
        if let Event::Key { modifiers, .. } = &mut select_all {
            *modifiers = egui::Modifiers::CTRL;
        }
        let mut backwards = test_key(Key::Tab);
        if let Event::Key { modifiers, .. } = &mut backwards {
            *modifiers = egui::Modifiers::SHIFT;
        }
        let cases = [
            (
                0,
                vec![
                    select_all,
                    Event::Text("7".into()),
                    test_key(Key::Enter),
                    Event::Text("suffix".into()),
                ],
                ModalAction::JumpToLine(7),
            ),
            (
                2,
                vec![test_key(Key::Tab), test_key(Key::Enter)],
                ModalAction::JumpToLine(10),
            ),
            (
                2,
                vec![test_key(Key::Tab), test_key(Key::Tab), test_key(Key::Enter)],
                ModalAction::Close,
            ),
            (2, vec![backwards, test_key(Key::Enter)], ModalAction::Close),
        ];
        for (delivery, events, expected) in cases {
            let ctx = Context::default();
            let mut owner = ModalInput::default();
            let mut dialog = GoToLineModal::new(10);
            let mut background = String::new();
            let mut result = None;
            for frame in 0..12 {
                let mut raw = RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(850.0, 650.0),
                    )),
                    events: if frame == delivery {
                        events.clone()
                    } else {
                        Vec::new()
                    },
                    ..Default::default()
                };
                owner.capture_input(&mut raw);
                assert!(raw.events.iter().all(|event| !keyboard_event(event)));
                let _ = ctx.run(raw, |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.text_edit_singleline(&mut background);
                    });
                    result = owner.show(ctx, |ctx| dialog.show(ctx, &AppTheme::dark(), 100));
                    assert!(
                        ctx.input(|input| input.events.iter().all(|event| !keyboard_event(event)))
                    );
                });
                if result.is_some() {
                    break;
                }
            }
            assert_eq!(result, Some(expected));
            assert!(background.is_empty());
            assert!(owner.input.is_empty());
            assert!(!owner.is_registered());
        }
    }
    #[test]
    fn composed_picker_queries_do_not_edit_the_previous_background_owner() {
        use crate::{repository::file_node::RepositoryFile, ui::modals::QuickFilePickerModal};
        use std::sync::Arc;
        let submit = vec![test_key(Key::Enter), Event::CompositionEnd("suffix".into())];
        for stream in [
            vec![
                vec![
                    Event::CompositionStart,
                    Event::CompositionUpdate("に".into()),
                    Event::CompositionUpdate("にほん".into()),
                    Event::CompositionEnd("日本".into()),
                ],
                Vec::new(),
                submit.clone(),
            ],
            vec![
                vec![Event::CompositionEnd("日本".into())],
                Vec::new(),
                submit.clone(),
            ],
            vec![
                vec![
                    Event::CompositionStart,
                    Event::CompositionUpdate("に".into()),
                ],
                vec![Event::CompositionUpdate("にほん".into())],
                vec![Event::CompositionEnd("日本".into())],
                submit.clone(),
            ],
            vec![
                vec![
                    Event::CompositionStart,
                    Event::CompositionUpdate("に".into()),
                ],
                vec![
                    Event::CompositionUpdate(String::new()),
                    Event::CompositionEnd(String::new()),
                ],
                vec![
                    Event::CompositionStart,
                    Event::CompositionUpdate("にほん".into()),
                    Event::CompositionEnd("日本".into()),
                ],
                submit,
            ],
        ] {
            let ctx = Context::default();
            let parent = egui::Id::new("background query");
            let mut background = String::new();
            let _ = ctx.run(RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.add(egui::TextEdit::singleline(&mut background).id(parent))
                        .request_focus();
                });
            });
            let mut owner = ModalInput::default();
            let mut dialog = QuickFilePickerModal::default();
            let files: Arc<[RepositoryFile]> = vec![RepositoryFile::new("日本.rs")].into();
            let mut action = None;
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            let mut frame = 0;
            while action.is_none() {
                let events = stream.get(frame).cloned().unwrap_or_default();
                let mut raw = RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(850.0, 650.0),
                    )),
                    events,
                    ..Default::default()
                };
                owner.capture_input(&mut raw);
                let _ = ctx.run(raw, |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.add(egui::TextEdit::singleline(&mut background).id(parent));
                    });
                    action = owner.show(ctx, |ctx| dialog.show(ctx, &AppTheme::dark(), &files));
                });
                assert!(
                    background.is_empty(),
                    "composition reached the previous text owner: {background:?}"
                );
                assert!(
                    std::time::Instant::now() < deadline,
                    "composed query did not submit"
                );
                frame += 1;
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            assert_eq!(action, Some(ModalAction::OpenFile("日本.rs".into())));
            assert_eq!(dialog.query, "日本");
        }
    }
}
