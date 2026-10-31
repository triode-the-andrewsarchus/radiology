//! Primary menu anchors adapt toolbar controls to the reader's menu owner.
use super::*;

#[derive(Clone)]
pub(super) struct Anchor {
    owner: Id,
    scope: Id,
    rect: Rect,
}

#[derive(Clone)]
struct Anchors {
    seen: u64,
    buttons: Vec<Anchor>,
}

pub(super) fn primary_press(ctx: &Context, input: &egui::RawInput) -> bool {
    let Some(anchors) = ctx.data(|data| data.get_temp::<Anchors>(state_id().with("bar"))) else {
        return false;
    };
    anchors.seen.saturating_add(1) >= ctx.frame_nr() && input.events.iter().any(|event| matches!(event,
        Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: true, .. }
        if anchors.buttons.iter().any(|anchor| anchor.rect.contains(*pos) && ctx.read_response(anchor.owner).is_some())))
}

struct Menu<T> {
    response: Response,
    scope: Id,
    opening: Option<(Pos2, Option<Id>)>,
    items: Vec<Item<T>>,
    initial: Option<Key>,
}

/// Collects current toolbar anchors before dispatching an owned popup.
pub(crate) struct MenuBar<T> {
    menus: Vec<Menu<T>>,
}

impl<T> Default for MenuBar<T> {
    fn default() -> Self {
        Self { menus: Vec::new() }
    }
}

impl<T: Clone> MenuBar<T> {
    pub fn button(
        &mut self,
        ui: &mut egui::Ui,
        label: impl Into<egui::WidgetText>,
        scope: impl std::hash::Hash,
        items: Vec<Item<T>>,
    ) -> Response {
        let previous = ui.memory(|memory| memory.focused());
        let id = ui.next_auto_id();
        let current_owner = current(ui.ctx()).is_some_and(|state| state.owner == id);
        let label = label.into();
        let label = if current_owner {
            label.color(ui.visuals().selection.stroke.color)
        } else {
            label
        };
        let response = ui.add(egui::Button::new(label).selected(current_owner));
        crate::ui::commands::retain_navigation_keys(ui.ctx(), response.id);
        let pressed = response.contains_pointer() && ui.input(|input| input.pointer.any_pressed());
        let keyboard_open = if !current_owner && response.has_focus() {
            ui.input_mut(|input| {
                [Key::Enter, Key::Space, Key::ArrowDown, Key::ArrowUp]
                    .into_iter()
                    .find(|key| input.consume_key(Modifiers::NONE, *key))
            })
        } else {
            None
        };
        let keyboard_click =
            response.clicked() && !ui.input(|input| input.pointer.any_released()) && !pressed;
        let opening = (pressed || keyboard_open.is_some() || keyboard_click)
            .then_some((response.rect.left_bottom(), previous));
        if current_owner && pressed {
            close(ui.ctx());
        }
        self.menus.push(Menu {
            scope: Id::new(scope),
            response: response.clone(),
            opening: opening.filter(|_| !(current_owner && pressed)),
            items,
            initial: keyboard_open
                .or(keyboard_click.then_some(Key::Home))
                .map(|key| {
                    if key == Key::ArrowUp {
                        Key::End
                    } else {
                        Key::Home
                    }
                }),
        });
        response
    }

    pub fn show(mut self) -> Option<T> {
        let anchors: Vec<_> = self
            .menus
            .iter()
            .map(|menu| Anchor {
                owner: menu.response.id,
                scope: menu.response.id.with(menu.scope),
                rect: menu.response.rect,
            })
            .collect();
        let ctx = self.menus.first().map(|menu| menu.response.ctx.clone());
        if let Some(ctx) = &ctx {
            let seen = ctx.frame_nr();
            ctx.data_mut(|data| {
                data.insert_temp(
                    state_id().with("bar"),
                    Anchors {
                        seen,
                        buttons: anchors.clone(),
                    },
                )
            });
            if let Some(state) = current(ctx)
                .filter(|state| anchors.iter().any(|anchor| anchor.owner == state.owner))
            {
                if ctx.input(|input| input.pointer.delta() != egui::Vec2::ZERO) {
                    if let Some(menu) = self
                        .menus
                        .iter_mut()
                        .find(|menu| menu.response.hovered() && menu.response.id != state.owner)
                    {
                        menu.opening = Some((menu.response.rect.left_bottom(), state.return_focus));
                    }
                }
            }
        }
        // Paint a newly chosen anchor first so the previous popup cannot linger.
        if let Some(index) = self.menus.iter().position(|menu| menu.opening.is_some()) {
            let opening = self.menus.remove(index);
            self.menus.insert(0, opening);
        }
        let mut action = None;
        for menu in self.menus {
            if let Some(next) = show_owned(
                &menu.response,
                menu.scope,
                menu.opening,
                &anchors,
                menu.initial,
                || menu.items,
            ) {
                action = Some(next);
            }
        }
        if let Some(ctx) = ctx {
            if let Some(state) = current(&ctx)
                .filter(|state| anchors.iter().any(|anchor| anchor.owner == state.owner))
            {
                ctx.memory_mut(|memory| {
                    if !memory.has_focus(state.owner) {
                        memory.request_focus(state.owner);
                    }
                });
            }
        }
        action
    }
}

impl State {
    pub(super) fn switch_bar<T>(
        &mut self,
        items: &[Item<T>],
        anchors: &[Anchor],
        key: Key,
    ) -> bool {
        if !self.open.is_empty() {
            return false;
        }
        let forward = match key {
            Key::ArrowLeft => false,
            Key::ArrowRight
                if self
                    .selected
                    .and_then(|index| items.get(index))
                    .is_none_or(|item| item.children.is_empty()) =>
            {
                true
            }
            _ => return false,
        };
        let Some(index) = anchors.iter().position(|anchor| anchor.owner == self.owner) else {
            return false;
        };
        let target = if forward {
            (index + 1) % anchors.len()
        } else {
            (index + anchors.len() - 1) % anchors.len()
        };
        let anchor = &anchors[target];
        self.owner = anchor.owner;
        self.scope = anchor.scope;
        self.origin = anchor.rect.left_bottom();
        self.selected = None;
        self.open.clear();
        self.rects = vec![anchor.rect];
        self.pending.push_front((Key::Home, Modifiers::NONE));
        true
    }
}

#[cfg(test)]
mod tests;
