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
