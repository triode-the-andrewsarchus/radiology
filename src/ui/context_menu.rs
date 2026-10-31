//! Reader context menus with owned keyboard input and ordinary egui controls.

mod bar;
pub(crate) use bar::MenuBar;

use egui::{Context, Event, Id, Key, Modifiers, Pos2, Rect, Response};
use std::collections::VecDeque;

pub(crate) struct Item<T> {
    label: String,
    enabled: bool,
    checked: bool,
    hint: String,
    tooltip: String,
    shortcut: Option<egui::KeyboardShortcut>,
    action: Option<T>,
    children: Vec<Self>,
}

impl<T> Item<T> {
    pub fn action(label: impl Into<String>, action: T) -> Self {
        Self {
            label: label.into(),
            enabled: true,
            checked: false,
            hint: String::new(),
            tooltip: String::new(),
            shortcut: None,
            action: Some(action),
            children: vec![],
        }
    }
    pub fn submenu(label: impl Into<String>, children: Vec<Self>) -> Self {
        Self {
            label: label.into(),
            enabled: !children.is_empty(),
            checked: false,
            hint: "▶".into(),
            tooltip: String::new(),
            shortcut: None,
            action: None,
            children,
        }
    }
    pub fn separator() -> Self {
        Self {
            label: String::new(),
            enabled: false,
            checked: false,
            hint: String::new(),
            tooltip: String::new(),
            shortcut: None,
            action: None,
            children: vec![],
        }
    }
    pub fn label(text: impl Into<String>) -> Self {
        let mut item = Self::separator();
        item.label = text.into();
        item
    }
    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = text.into();
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }
    pub fn shortcut(mut self, key: Key, modifiers: Modifiers, hint: &str) -> Self {
        self.shortcut = Some(egui::KeyboardShortcut::new(modifiers, key));
        self.hint = hint.into();
        self
    }
    pub fn map<U>(self, convert: impl Fn(T) -> U + Copy) -> Item<U> {
        Item {
            label: self.label,
            enabled: self.enabled,
            checked: self.checked,
            hint: self.hint,
            tooltip: self.tooltip,
            shortcut: self.shortcut,
            action: self.action.map(convert),
            children: self
                .children
                .into_iter()
                .map(|item| item.map(convert))
                .collect(),
        }
    }
}

#[derive(Clone)]
struct State {
    owner: Id,
    scope: Id,
    return_focus: Option<Id>,
    origin: Pos2,
    open: Vec<usize>,
    selected: Option<usize>,
    pending: VecDeque<(Key, Modifiers)>,
    rects: Vec<Rect>,
    seen: u64,
}

#[derive(Clone, Default)]
struct Opening {
    focus: Option<Id>,
    pending: VecDeque<(Key, Modifiers)>,
    armed: bool,
    primary: bool,
    released: Option<u64>,
    held: Option<(Id, u64)>,
}

impl Opening {
    fn capture(ctx: &Context, input: &egui::RawInput) -> Option<Self> {
        let id = state_id().with("opening");
        let primary = bar::primary_press(ctx, input);
        let pressed = primary
            || input.events.iter().any(|event| {
                matches!(
                    event,
                    Event::PointerButton {
                        button: egui::PointerButton::Secondary,
                        pressed: true,
                        ..
                    }
                )
            });
        let mut opening = if pressed {
            Self {
                focus: ctx.memory(|memory| memory.focused()),
                primary,
                ..Default::default()
            }
        } else {
            ctx.data(|data| data.get_temp::<Self>(id))?
        };
        if opening.held.is_some() && input.events.iter().any(|event| matches!(event,
            Event::PointerButton { button, pressed: true, .. } if *button != egui::PointerButton::Secondary)) {
            ctx.data_mut(|data| data.remove::<Self>(id));
            return None;
        }
        if opening
            .held
            .is_some_and(|(_, frame)| frame.saturating_add(1) < ctx.frame_nr())
            || (opening.held.is_none()
                && opening.released.is_some_and(|frame| frame < ctx.frame_nr()))
        {
            ctx.data_mut(|data| data.remove::<Self>(id));
            return None;
        }
        if input.events.iter().any(|event| {
            matches!(
                event,
                Event::PointerButton {
                    button,
                    pressed: false,
                    ..
                } if *button == if opening.primary { egui::PointerButton::Primary } else { egui::PointerButton::Secondary }
            )
        }) {
            opening.released = Some(ctx.frame_nr());
        }
        ctx.data_mut(|data| data.insert_temp(id, opening.clone()));
        Some(opening)
    }
}

fn state_id() -> Id {
    Id::new("reader_context_menu")
}

fn current(ctx: &Context) -> Option<State> {
    ctx.data(|data| data.get_temp::<State>(state_id()))
        .filter(|state| {
            state.seen.saturating_add(1) >= ctx.frame_nr()
                && ctx.read_response(state.owner).is_some()
        })
}

pub(crate) fn is_open(ctx: &Context) -> bool {
    current(ctx).is_some()
}

pub(crate) fn close(ctx: &Context) {
    if current(ctx).is_some() {
        ctx.request_repaint();
    }
    ctx.data_mut(|data| data.remove::<Opening>(state_id().with("opening")));
    let state = ctx.data_mut(|data| {
        let state = data.get_temp::<State>(state_id());
        data.remove::<State>(state_id());
        state
    });
    if let Some(focus) = state
        .and_then(|state| state.return_focus)
        .filter(|id| ctx.read_response(*id).is_some())
    {
        ctx.memory_mut(|memory| {
            if !memory.has_focus(focus) {
                memory.request_focus(focus);
            }
        });
    }
}

pub(crate) fn owned_key(event: &Event) -> Option<(Key, Modifiers)> {
    match event {
        Event::Key {
            key,
            modifiers,
            pressed: true,
            ..
        } => {
            let global = (modifiers.ctrl || modifiers.command || modifiers.alt || *key == Key::F12)
                && super::commands::shortcut(*key, *modifiers, false).is_some();
            (!global).then_some((*key, *modifiers))
        }
        Event::Copy => Some((Key::C, Modifiers::CTRL)),
        Event::Text(_) | Event::Paste(_) | Event::Cut => Some((Key::Escape, Modifiers::SHIFT)),
        _ => None,
    }
}

/// Captures menu keys before any reader/list or egui focus handling sees them.
pub(crate) fn take_input(ctx: &Context, input: &mut egui::RawInput) {
    let opening = Opening::capture(ctx, input);
    let state = current(ctx).filter(|state| !input.events.iter().any(|event| matches!(event,
        Event::PointerButton { pos, pressed: true, .. } if !state.rects.iter().any(|rect| rect.contains(*pos)))));
    if let Some(mut state) = state {
        state.pending.extend(super::commands::take_input_actions(
            &mut input.events,
            owned_key,
        ));
        ctx.data_mut(|data| data.insert_temp(state_id(), state));
    } else {
        close(ctx);
        if let Some(mut opening) = opening {
            let pending = super::commands::take_input_actions(&mut input.events, |event| {
                if matches!(
                    event,
                    Event::PointerButton {
                        button,
                        pressed: true,
                        ..
                    } if *button == if opening.primary { egui::PointerButton::Primary } else { egui::PointerButton::Secondary }
                ) {
                    opening.armed = true;
                }
                opening.armed.then(|| owned_key(event)).flatten()
            });
            opening.pending.extend(pending);
            ctx.data_mut(|data| data.insert_temp(state_id().with("opening"), opening));
        }
    }
}

/// Keeps opening-frame keys scoped to a reader awaiting exact hit geometry.
pub(crate) fn hold_opening(ctx: &Context, owner: Id, key: Option<(Key, Modifiers)>) {
    let frame = ctx.frame_nr();
    ctx.data_mut(|data| {
        let id = state_id().with("opening");
        let mut opening = data.get_temp::<Opening>(id).unwrap_or_default();
        opening.held = Some((owner, frame));
        opening.armed = true;
        opening.pending.extend(key);
        data.insert_temp(id, opening);
    });
}

pub(crate) fn owns_opening(ctx: &Context, owner: Id) -> bool {
    let frame = ctx.frame_nr();
    ctx.data(|data| data.get_temp::<Opening>(state_id().with("opening")))
        .is_some_and(|opening| {
            opening
                .held
                .is_some_and(|(held, seen)| held == owner && seen.saturating_add(1) >= frame)
        })
}

pub(crate) fn cancel_opening(ctx: &Context, owner: Id) {
    ctx.data_mut(|data| {
        let id = state_id().with("opening");
        if data
            .get_temp::<Opening>(id)
            .is_some_and(|opening| opening.held.is_some_and(|(held, _)| held == owner))
        {
            data.remove::<Opening>(id);
        }
    });
}

fn entries<'a, T>(mut items: &'a [Item<T>], path: &[usize]) -> &'a [Item<T>] {
    for &index in path {
        let Some(item) = items.get(index).filter(|item| item.enabled) else {
            return &[];
        };
        items = &item.children;
    }
    items
}

impl State {
    fn point_to(&mut self, level: usize, index: usize, submenu: bool) {
        self.open.truncate(level);
        self.selected = Some(index);
        if submenu {
            self.open.push(index);
            self.selected = None;
        }
    }

    fn move_selection<T>(&mut self, items: &[Item<T>], key: Key, shift: bool) {
        let enabled: Vec<_> = entries(items, &self.open)
            .iter()
            .enumerate()
            .filter_map(|(index, item)| item.enabled.then_some(index))
            .collect();
        let Some(last) = enabled.len().checked_sub(1) else {
            self.selected = None;
            return;
        };
        let selected = self
            .selected
            .and_then(|index| enabled.iter().position(|item| *item == index));
        let position = match key {
            Key::Home => 0,
            Key::End => last,
            Key::ArrowUp => selected.map_or(last, |index| (index + last) % enabled.len()),
            Key::Tab if shift => selected.map_or(last, |index| (index + last) % enabled.len()),
            _ => selected.map_or(0, |index| (index + 1) % enabled.len()),
        };
        self.selected = Some(enabled[position]);
    }

    fn enter<T: Clone>(&mut self, items: &[Item<T>], activate: bool) -> Option<T> {
        let index = self.selected?;
        let item = entries(items, &self.open)
            .get(index)
            .filter(|item| item.enabled)?;
        if !item.children.is_empty() {
            self.point_to(self.open.len(), index, true);
            self.move_selection(items, Key::Home, false);
            None
        } else if activate {
            item.action.clone()
        } else {
            None
        }
    }

    fn keys<T: Clone>(&mut self, items: &[Item<T>], bar: &[bar::Anchor]) -> (bool, Option<T>) {
        while let Some((key, modifiers)) = self.pending.pop_front() {
            if modifiers.ctrl || modifiers.command || modifiers.alt {
                if let Some(action) = entries(items, &self.open)
                    .iter()
                    .filter(|item| item.enabled)
                    .find_map(|item| {
                        item.shortcut
                            .filter(|shortcut| {
                                shortcut.logical_key == key
                                    && modifiers.matches_logically(shortcut.modifiers)
                            })
                            .and(item.action.clone())
                    })
                {
                    return (true, Some(action));
                }
                continue;
            }
            if self.switch_bar(items, bar, key) {
                return (false, None);
            }
            match key {
                Key::Escape if modifiers == Modifiers::NONE => return (true, None),
                Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End | Key::Tab => {
                    self.move_selection(items, key, modifiers.shift)
                }
                Key::ArrowLeft => {
                    if let Some(index) = self.open.pop() {
                        self.selected = Some(index);
                    }
                }
                Key::Enter | Key::Space | Key::ArrowRight => {
                    if let Some(action) = self.enter(items, key != Key::ArrowRight) {
                        return (true, Some(action));
                    }
                }
                _ => {}
            }
        }
        (false, None)
    }
}

/// Builds actions only for the live owner; actions never outlive their source model.
pub(crate) fn show<T: Clone>(
    response: &Response,
    scope: impl std::hash::Hash,
    build: impl FnOnce() -> Vec<Item<T>>,
) -> Option<T> {
    show_at(
        response,
        scope,
        response
            .secondary_clicked()
            .then(|| response.interact_pointer_pos())
            .flatten(),
        build,
    )
}

/// Opens a menu after a source-owned gesture has resolved its canonical target.
pub(crate) fn show_at<T: Clone>(
    response: &Response,
    scope: impl std::hash::Hash,
    opening: Option<Pos2>,
    build: impl FnOnce() -> Vec<Item<T>>,
) -> Option<T> {
    show_owned(
        response,
        scope,
        opening.map(|origin| (origin, response.ctx.memory(|memory| memory.focused()))),
        &[],
        None,
        build,
    )
}

fn show_owned<T: Clone>(
    response: &Response,
    scope: impl std::hash::Hash,
    opening: Option<(Pos2, Option<Id>)>,
    bar: &[bar::Anchor],
    initial: Option<Key>,
    build: impl FnOnce() -> Vec<Item<T>>,
) -> Option<T> {
    let ctx = &response.ctx;
    let scope = response.id.with(scope);
    if current(ctx).is_some_and(|state| state.owner == response.id && state.scope != scope) {
        close(ctx);
    }
    let mut state = if let Some((origin, focus)) = opening {
        let opening = ctx
            .data_mut(|data| data.remove_temp::<Opening>(state_id().with("opening")))
            .unwrap_or_default();
        State {
            owner: response.id,
            scope,
            return_focus: focus.or(opening.focus),
            origin,
            open: vec![],
            selected: None,
            pending: opening.pending,
            rects: vec![],
            seen: ctx.frame_nr(),
        }
    } else {
        current(ctx).filter(|state| state.owner == response.id)?
    };
    // Also support embeddable readers whose host only supplies prepared egui frames.
    state.pending.extend(
        ctx.input_mut(|input| super::commands::take_input_actions(&mut input.events, owned_key)),
    );
    if state
        .pending
        .front()
        .is_some_and(|(key, modifiers)| *key == Key::Escape && *modifiers == Modifiers::NONE)
    {
        ctx.data_mut(|data| data.insert_temp(state_id(), state));
        close(ctx);
        return None;
    }
    let items = build();
    let reveal_selection = initial.is_some() || !state.pending.is_empty();
    if let Some(key) = initial {
        state.move_selection(&items, key, false);
    }
    let (dismiss, action) = state.keys(&items, bar);
    if state.owner != response.id {
        state.seen = ctx.frame_nr();
        ctx.data_mut(|data| data.insert_temp(state_id(), state));
        ctx.request_repaint();
        return None;
    }
    if dismiss {
        ctx.data_mut(|data| data.insert_temp(state_id(), state));
        close(ctx);
        return action;
    }
    let action = paint(ctx, &items, &mut state, reveal_selection);
    if !bar.is_empty() {
        state.rects.push(response.rect);
    }
    state.seen = ctx.frame_nr();
    ctx.data_mut(|data| data.insert_temp(state_id(), state));
    if action.is_some() {
        close(ctx);
    }
    action
}

fn paint<T: Clone>(
    ctx: &Context,
    items: &[Item<T>],
    state: &mut State,
    reveal_selection: bool,
) -> Option<T> {
    state.rects.clear();
    let pointer_changed =
        ctx.input(|input| input.pointer.delta() != egui::Vec2::ZERO || input.pointer.any_pressed());
    let reveal_selection = reveal_selection && !pointer_changed;
    let mut origin = state.origin;
    let mut level = 0;
    loop {
        let path = state.open[..level].to_vec();
        let rows = entries(items, &path);
        if rows.is_empty() {
            state.open.truncate(level.saturating_sub(1));
            break;
        }
        let (rect, child_y, action) = paint_level(
            ctx,
            rows,
            state,
            level,
            origin,
            pointer_changed,
            reveal_selection,
        );
        state.rects.push(rect);
        if action.is_some() {
            return action;
        }
        if state.open.len() <= level {
            break;
        }
        let right = Pos2::new(rect.right() + 2.0, child_y);
        origin = if right.x + rect.width() <= ctx.screen_rect().right() {
            right
        } else {
            Pos2::new(
                (rect.left() - rect.width() - 2.0).max(ctx.screen_rect().left()),
                child_y,
            )
        };
        level += 1;
    }
    None
}

fn paint_level<T: Clone>(
    ctx: &Context,
    rows: &[Item<T>],
    state: &mut State,
    level: usize,
    origin: Pos2,
    pointer_changed: bool,
    reveal_selection: bool,
) -> (Rect, f32, Option<T>) {
    let mut child_y = origin.y;
    let mut action = None;
    let area = egui::Area::new(state.scope.with(("reader_menu", level)))
        .order(egui::Order::Foreground)
        .movable(false)
        .sense(egui::Sense::hover())
        .fixed_pos(origin)
        .constrain_to(ctx.screen_rect())
        .show(ctx, |ui| {
            egui::Frame::menu(ui.style()).show(ui, |ui| {
                ui.set_min_width(180.0);
                ui.set_max_width(300.0);
                egui::ScrollArea::vertical()
                    .id_source(state.scope.with(("menu_scroll", level)))
                    .max_height((ctx.screen_rect().height() - 32.0).max(48.0))
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                            for (index, item) in rows.iter().enumerate() {
                                let Some((response, chosen)) = paint_item(
                                    ui,
                                    item,
                                    state,
                                    index,
                                    level,
                                    pointer_changed,
                                    reveal_selection,
                                ) else {
                                    continue;
                                };
                                if state.open.get(level) == Some(&index) {
                                    child_y = response.rect.top();
                                }
                                action = chosen.or(action.take());
                            }
                        });
                    });
            });
        });
    (area.response.rect, child_y, action)
}

fn paint_item<T: Clone>(
    ui: &mut egui::Ui,
    item: &Item<T>,
    state: &mut State,
    index: usize,
    level: usize,
    pointer_changed: bool,
    reveal_selection: bool,
) -> Option<(Response, Option<T>)> {
    if item.label.is_empty() {
        ui.separator();
        return None;
    }
    if item.action.is_none() && item.children.is_empty() {
        return Some((ui.add(egui::Label::new(&item.label).wrap(true)), None));
    }
    let selected = state.open.get(level) == Some(&index)
        || (state.open.len() == level && state.selected == Some(index));
    let label = if item.checked {
        format!("✓ {}", item.label)
    } else {
        item.label.clone()
    };
    let button = egui::Button::new(label)
        .wrap(true)
        .shortcut_text(&item.hint)
        .selected(selected)
        .frame(selected)
        .sense(egui::Sense {
            focusable: false,
            ..egui::Sense::click()
        });
    let response = ui.add_enabled(item.enabled, button);
    let response = if item.tooltip.is_empty() {
        response
    } else {
        response.on_hover_text(&item.tooltip)
    };
    if selected && reveal_selection {
        response.scroll_to_me(None);
    }
    if !item.enabled {
        return Some((response, None));
    }
    if response.hovered() && pointer_changed {
        state.point_to(level, index, !item.children.is_empty());
    }
    let mut action = None;
    if response.clicked() {
        if item.children.is_empty() {
            action = item.action.clone();
        } else {
            state.point_to(level, index, true);
        }
    }
    Some((response, action))
}

#[cfg(test)]
mod tests;
