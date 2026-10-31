use super::*;
use crate::ui::test_support::{click, menu_frame, painted_frame, text_center};

struct Menu {
    ctx: Context,
    focus: Id,
    choice: Option<&'static str>,
    first: &'static str,
    first_enabled: bool,
    scope: usize,
}

impl Menu {
    fn new() -> Self {
        Self {
            ctx: Context::default(),
            focus: Id::new("reader"),
            choice: None,
            first: "first",
            first_enabled: true,
            scope: 0,
        }
    }
    fn frame(&mut self, events: Vec<Event>) -> egui::FullOutput {
        menu_frame(&self.ctx, egui::vec2(600.0, 500.0), events, |ui| {
            ui.interact(
                ui.max_rect(),
                self.focus,
                egui::Sense::focusable_noninteractive(),
            );
            let row = ui.button("Owner");
            self.choice = show(&row, self.scope, || {
                vec![
                    Item::action("Disabled", "disabled").enabled(false),
                    Item::action("First", self.first)
                        .enabled(self.first_enabled)
                        .shortcut(Key::C, Modifiers::CTRL, "Ctrl+C"),
                    Item::separator(),
                    Item::submenu(
                        "More",
                        vec![
                            Item::action("Unavailable", "unavailable").enabled(false),
                            Item::action("Child", "child"),
                            Item::action("Other child", "other child"),
                        ],
                    ),
                    Item::action("Last", "last"),
                ]
            });
        })
    }
    fn open(&mut self) {
        let output = self.frame(vec![]);
        self.ctx
            .memory_mut(|memory| memory.request_focus(self.focus));
        let events = crate::ui::test_support::secondary_click(text_center(&output, "Owner"));
        self.frame(events);
        self.frame(vec![]);
        assert!(is_open(&self.ctx));
    }
    fn keys(&mut self, keys: &[Key]) -> egui::FullOutput {
        self.frame(
            keys.iter()
                .copied()
                .map(crate::ui::commands::test_key)
                .collect(),
        )
    }
}

#[test]
fn keys_skip_disabled_actions_wrap_and_never_reach_the_reader() {
    let mut menu = Menu::new();
    menu.open();
    let mut raw = egui::RawInput {
        events: vec![
            crate::ui::commands::test_key(Key::ArrowDown),
            crate::ui::commands::test_key(Key::Delete),
        ],
        ..Default::default()
    };
    take_input(&menu.ctx, &mut raw);
    assert!(raw.events.is_empty());
    menu.frame(raw.events);
    assert_eq!(current(&menu.ctx).unwrap().selected, Some(1));
    menu.keys(&[Key::ArrowUp, Key::Enter]);
    assert_eq!(menu.choice, Some("last"));
    assert!(!is_open(&menu.ctx));
    assert!(menu.ctx.memory(|memory| memory.has_focus(menu.focus)));
    let mut closed = egui::RawInput {
        events: vec![crate::ui::commands::test_key(Key::ArrowDown)],
        ..Default::default()
    };
    take_input(&menu.ctx, &mut closed);
    assert_eq!(closed.events.len(), 1);
}

#[test]
fn submenu_arrows_tabs_activation_and_escape_preserve_owner_focus() {
    let mut menu = Menu::new();
    menu.open();
    menu.keys(&[Key::ArrowDown, Key::ArrowDown, Key::ArrowRight]);
    assert_eq!(current(&menu.ctx).unwrap().open, vec![3]);
    menu.keys(&[Key::ArrowLeft]);
    assert_eq!(current(&menu.ctx).unwrap().selected, Some(3));
    menu.keys(&[Key::ArrowRight, Key::End, Key::Home, Key::Tab, Key::Enter]);
    assert_eq!(menu.choice, Some("other child"));
    menu.open();
    menu.keys(&[Key::ArrowDown, Key::ArrowRight]);
    assert!(menu.choice.is_none(), "Right must not activate a leaf");
    menu.keys(&[Key::Escape]);
    assert!(!is_open(&menu.ctx));
    assert!(menu.choice.is_none());
    assert!(menu.ctx.memory(|memory| memory.has_focus(menu.focus)));
}

#[test]
fn pointer_and_copy_shortcut_share_current_actions_and_outside_click_dismisses() {
    let mut menu = Menu::new();
    menu.open();
    let mut copy = crate::ui::commands::test_key(Key::C);
    if let Event::Key { modifiers, .. } = &mut copy {
        *modifiers = Modifiers::CTRL;
    }
    menu.frame(vec![copy]);
    assert_eq!(menu.choice, Some("first"));
    menu.open();
    let output = menu.frame(vec![]);
    menu.frame(click(text_center(&output, "Last")));
    assert_eq!(menu.choice, Some("last"));
    menu.open();
    menu.frame(click(egui::pos2(580.0, 480.0)));
    assert!(!is_open(&menu.ctx));
    assert!(menu.choice.is_none());
}

#[test]
fn actions_follow_the_current_model_and_retired_owners_release_input() {
    let mut menu = Menu::new();
    menu.open();
    menu.keys(&[Key::ArrowDown]);
    menu.first_enabled = false;
    menu.keys(&[Key::Enter]);
    assert!(menu.choice.is_none());
    assert!(is_open(&menu.ctx));
    menu.first_enabled = true;
    menu.first = "replacement";
    menu.keys(&[Key::Enter]);
    assert_eq!(menu.choice, Some("replacement"));
    menu.open();
    painted_frame(&menu.ctx, egui::vec2(600.0, 500.0), vec![], |ui| {
        ui.label("Owner removed");
    });
    let mut input = egui::RawInput {
        events: vec![crate::ui::commands::test_key(Key::ArrowDown)],
        ..Default::default()
    };
    take_input(&menu.ctx, &mut input);
    assert!(!is_open(&menu.ctx));
    assert_eq!(input.events.len(), 1);
}

#[test]
fn shift_tab_wraps_and_global_shortcuts_remain_available() {
    let mut menu = Menu::new();
    menu.open();
    let mut tab = crate::ui::commands::test_key(Key::Tab);
    if let Event::Key { modifiers, .. } = &mut tab {
        *modifiers = Modifiers::SHIFT;
    }
    menu.frame(vec![tab]);
    menu.keys(&[Key::Space]);
    assert_eq!(menu.choice, Some("last"));
    menu.open();
    let mut global = crate::ui::commands::test_key(Key::K);
    if let Event::Key { modifiers, .. } = &mut global {
        *modifiers = Modifiers::CTRL;
    }
    let mut input = egui::RawInput {
        events: vec![global],
        ..Default::default()
    };
    take_input(&menu.ctx, &mut input);
    assert_eq!(input.events.len(), 1);
    close(&menu.ctx);
    assert!(!is_open(&menu.ctx));
}

#[test]
fn a_reused_widget_id_cannot_retarget_an_open_menu_to_another_source() {
    let mut menu = Menu::new();
    menu.open();
    menu.keys(&[Key::ArrowDown]);
    menu.scope += 1;
    menu.keys(&[Key::Enter]);
    assert!(menu.choice.is_none());
    assert!(!is_open(&menu.ctx));
}
