use super::*;
use crate::ui::commands::test_key;
use crate::ui::test_support::{click, menu_frame, text_center};

#[derive(Default)]
struct BarReader {
    ctx: Context,
    choice: Option<&'static str>,
    scope: usize,
}

impl BarReader {
    fn frame(&mut self, events: Vec<Event>) -> egui::FullOutput {
        menu_frame(&self.ctx, egui::vec2(600.0, 500.0), events, |ui| {
            let mut bar = MenuBar::default();
            ui.horizontal(|ui| {
                bar.button(
                    ui,
                    "Alpha",
                    ("alpha", self.scope),
                    vec![
                        Item::action("Disabled", "disabled").enabled(false),
                        Item::action("First", "first"),
                        Item::submenu("More", vec![Item::action("Child", "child")]),
                        Item::action("Last", "last"),
                    ],
                );
                bar.button(
                    ui,
                    "Beta",
                    "beta",
                    vec![
                        Item::action("Beta First", "beta"),
                        Item::action("Beta Last", "beta-last"),
                    ],
                );
                bar.button(
                    ui,
                    "Gamma",
                    "gamma",
                    vec![Item::action("Gamma First", "gamma")],
                );
            });
            self.choice = bar.show();
            ui.interact(
                ui.available_rect_before_wrap(),
                Id::new("source"),
                egui::Sense::focusable_noninteractive(),
            );
        })
    }
    fn open(&mut self) {
        let output = self.frame(vec![]);
        self.ctx
            .memory_mut(|memory| memory.request_focus(Id::new("source")));
        self.frame(click(text_center(&output, "Alpha")));
        assert!(is_open(&self.ctx));
    }
    fn expect_action(&self, choice: &'static str) {
        assert_eq!(self.choice, Some(choice));
        assert!(!is_open(&self.ctx));
        assert!(self
            .ctx
            .memory(|memory| memory.has_focus(Id::new("source"))));
    }
    fn keys(&mut self, keys: &[Key]) {
        self.frame(keys.iter().copied().map(test_key).collect());
    }
}

#[test]
fn toolbar_arrows_switch_live_menus_in_order_and_keep_submenu_ownership() {
    let mut reader = BarReader::default();
    reader.open();
    reader.keys(&[Key::ArrowRight, Key::End, Key::Enter]);
    reader.expect_action("beta-last");
    reader.open();
    reader.keys(&[Key::ArrowLeft, Key::ArrowDown, Key::Enter]);
    assert_eq!(reader.choice, Some("gamma"));
    reader.open();
    reader.keys(&[Key::Home, Key::ArrowDown, Key::ArrowRight, Key::Enter]);
    assert_eq!(reader.choice, Some("child"));
}

#[test]
fn primary_press_reserves_coalesced_keys_and_its_release_does_not_reopen_the_menu() {
    let mut reader = BarReader::default();
    let output = reader.frame(vec![]);
    reader
        .ctx
        .memory_mut(|memory| memory.request_focus(Id::new("source")));
    let mut events = click(text_center(&output, "Alpha"));
    let release = events.pop().unwrap();
    events.extend([test_key(Key::End), test_key(Key::Enter)]);
    reader.frame(events);
    assert_eq!(reader.choice, Some("last"));
    reader.frame(vec![release]);
    assert!(!is_open(&reader.ctx));
    assert!(reader.choice.is_none());
}

#[test]
fn toolbar_toggle_escape_and_scope_retirement_never_dispatch_stale_choices() {
    let mut reader = BarReader::default();
    reader.open();
    let output = reader.frame(vec![]);
    reader.frame(click(text_center(&output, "Alpha")));
    assert!(!is_open(&reader.ctx));
    reader.open();
    reader.keys(&[Key::Escape]);
    assert!(reader
        .ctx
        .memory(|memory| memory.has_focus(Id::new("source"))));
    reader.open();
    reader.scope += 1;
    reader.keys(&[Key::End, Key::Enter]);
    assert!(!is_open(&reader.ctx));
    assert!(reader.choice.is_none());
}

#[test]
fn a_tab_focused_toolbar_button_opens_with_the_keyboard_and_returns_to_that_button() {
    for (opening, expected) in [
        (Key::ArrowDown, "first"),
        (Key::ArrowUp, "last"),
        (Key::Enter, "first"),
        (Key::Space, "first"),
    ] {
        let mut reader = BarReader::default();
        reader.frame(vec![]);
        let button = reader
            .ctx
            .data(|data| data.get_temp::<Anchors>(state_id().with("bar")))
            .unwrap()
            .buttons[0]
            .owner;
        reader.ctx.memory_mut(|memory| memory.request_focus(button));
        reader.frame(vec![]);
        reader.frame(vec![]);
        reader.keys(&[opening]);
        assert!(is_open(&reader.ctx));
        reader.keys(&[Key::Enter]);
        assert_eq!(reader.choice, Some(expected));
        assert!(reader.ctx.memory(|memory| memory.has_focus(button)));
        reader.keys(&[opening, Key::ArrowRight]);
        reader.keys(&[Key::Escape]);
        assert!(reader.ctx.memory(|memory| memory.has_focus(button)));
    }
}

#[test]
fn hovering_another_toolbar_anchor_replaces_the_popup_and_keeps_return_focus() {
    let mut reader = BarReader::default();
    reader.open();
    let output = reader.frame(vec![]);
    let beta = text_center(&output, "Beta");
    reader.frame(vec![Event::PointerMoved(beta)]);
    let output = reader.frame(vec![]);
    assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == "Beta First")));
    assert!(!output.shapes.iter().any(
        |shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == "First")
    ));
    reader.keys(&[Key::End, Key::Enter]);
    reader.expect_action("beta-last");
}

#[test]
fn the_active_menu_title_tracks_popup_ownership_without_changing_return_focus() {
    for dark in [true, false] {
        for scale in [1.0, 2.0] {
            let mut reader = BarReader::default();
            let theme = if dark {
                crate::theme::AppTheme::dark()
            } else {
                crate::theme::AppTheme::light()
            };
            reader.ctx.set_visuals(theme.visuals());
            reader.ctx.set_pixels_per_point(scale);
            reader.open();
            for (movement, selected) in [
                (None, 0),
                (Some(Key::ArrowRight), 1),
                (Some(Key::ArrowLeft), 0),
            ] {
                if let Some(key) = movement {
                    reader.keys(&[key]);
                }
                let output = reader.frame(vec![]);
                let anchors = reader
                    .ctx
                    .data(|data| data.get_temp::<Anchors>(state_id().with("bar")))
                    .unwrap()
                    .buttons;
                assert_eq!(current(&reader.ctx).unwrap().owner, anchors[selected].owner);
                assert!(
                    reader
                        .ctx
                        .memory(|memory| memory.has_focus(anchors[selected].owner)),
                    "keyboard focus should follow the active title"
                );
                let visuals = reader.ctx.style().visuals.clone();
                for (index, anchor) in anchors.iter().enumerate() {
                    let highlighted = output.shapes.iter().any(|shape| matches!(&shape.shape,
                        egui::Shape::Rect(rect) if rect.fill == visuals.selection.bg_fill && rect.rect.contains(anchor.rect.center())));
                    if index == selected {
                        assert!(
                            highlighted,
                            "the active title needs its selected background"
                        );
                    }
                    let label = ["Alpha", "Beta", "Gamma"][index];
                    let accented = output.shapes.iter().any(|shape| matches!(&shape.shape,
                        egui::Shape::Text(text) if text.galley.job.text == label
                            && text.galley.job.sections.iter().any(|section| section.format.color == visuals.selection.stroke.color)));
                    assert_eq!(
                        accented,
                        index == selected,
                        "only the active title should use the accent caption"
                    );
                }
            }
            reader.keys(&[Key::Escape]);
            assert!(!is_open(&reader.ctx));
            assert!(reader
                .ctx
                .memory(|memory| memory.has_focus(Id::new("source"))));
        }
    }
}

#[test]
fn prepared_hosts_keep_bare_activation_in_the_menu_and_global_shortcuts_in_the_reader() {
    fn frame(
        ctx: &Context,
        events: Vec<Event>,
    ) -> (
        egui::FullOutput,
        Option<&'static str>,
        Vec<crate::ui::commands::ReaderCommand>,
    ) {
        let mut choice = None;
        let mut commands = Vec::new();
        let output =
            crate::ui::test_support::painted_frame(ctx, egui::vec2(600.0, 500.0), events, |ui| {
                commands = crate::ui::commands::take_shortcuts(ctx, false, false);
                let mut bar = MenuBar::default();
                bar.button(ui, "Menu", "prepared", vec![Item::action("First", "first")]);
                choice = bar.show();
            });
        (output, choice, commands)
    }
    let ctx = Context::default();
    let (output, _, _) = frame(&ctx, vec![]);
    let position = text_center(&output, "Menu");
    frame(&ctx, click(position));
    frame(&ctx, vec![test_key(Key::Home)]);
    let (_, choice, commands) = frame(&ctx, vec![test_key(Key::Enter)]);
    assert_eq!(choice, Some("first"));
    assert!(commands.is_empty());
    frame(&ctx, click(position));
    let mut shortcut = test_key(Key::J);
    if let Event::Key { modifiers, .. } = &mut shortcut {
        *modifiers = Modifiers::CTRL | Modifiers::ALT;
    }
    let (_, choice, commands) = frame(&ctx, vec![shortcut]);
    assert!(choice.is_none());
    assert_eq!(
        commands,
        [crate::ui::commands::ReaderCommand::GoToDefinition]
    );
}
