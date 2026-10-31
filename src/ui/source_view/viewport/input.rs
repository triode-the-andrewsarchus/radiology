//! Ordered source gestures in document coordinates, independent of layout timing.

use super::*;
use egui::{Event, PointerButton, Pos2, Vec2};

#[derive(Clone, Copy)]
enum PointerKind {
    Press,
    Drag,
    Release { click: bool },
    Context,
}

#[derive(Clone, Copy)]
pub(super) struct PointerEvent {
    kind: PointerKind,
    row: usize,
    // A provisional reveal paints this exact fragment before its prefix is ready.
    // Retain fragment-local coordinates so refinement cannot change the hit byte.
    fragment: Option<usize>,
    local: Vec2,
    screen: Pos2,
    modifiers: Modifiers,
    time: f64,
}

#[derive(Default)]
pub(super) struct PointerCapture {
    primary: Option<Pos2>,
    secondary: Option<Pos2>,
    menu_waiting: bool,
    seen: Option<u64>,
}

struct InputFrame {
    code_rect: Rect,
    scroll: Vec2,
    time: f64,
    available: Vec<Pos2>,
}

impl PointerCapture {
    fn interrupted(&self, ui: &Ui, code_rect: Rect, scrollbars: &[Rect]) -> bool {
        let owner_gap = self
            .seen
            .is_some_and(|frame| frame.saturating_add(1) < ui.ctx().frame_nr());
        owner_gap
            || ui.input(|input| {
                (self.menu_waiting && input.pointer.any_pressed())
                    || !input.raw.focused
                    || input.raw_scroll_delta != Vec2::ZERO
                    || (input.pointer.any_pressed()
                        && input.pointer.interact_pos().is_none_or(|point| {
                            !code_rect.contains(point)
                                || scrollbars.iter().any(|rect| rect.contains(point))
                        }))
            })
    }
}

impl SourceViewport {
    fn cancel_source_input(&mut self, ctx: &egui::Context) {
        self.pending_input.clear();
        self.pointer = PointerCapture::default();
        self.dragging = false;
        self.occurrence_clicks = None;
        self.menu_request = None;
        crate::ui::context_menu::cancel_opening(ctx, Self::focus_id());
    }

    pub(super) fn retire_input(
        &mut self,
        ui: &Ui,
        content: &CanvasContent<'_>,
        geometry: &CanvasGeometry,
    ) {
        let interrupted = self
            .pointer
            .interrupted(ui, geometry.code_rect, &self.scrollbars);
        let lost_focus = !self.has_focus(ui.ctx()) && ui.ctx().wants_keyboard_input();
        let manual_override = ui.input(|input| input.pointer.any_pressed())
            && !self.pending_input.is_empty()
            && !self
                .pending_input
                .iter()
                .any(|event| matches!(event, SourceInput::Pointer(_)));
        let menu_retired = self.pointer.menu_waiting
            && !crate::ui::context_menu::owns_opening(ui.ctx(), Self::focus_id());
        if interrupted
            || menu_retired
            || !content.keyboard_enabled
            || lost_focus
            || (content.target.is_some() && !self.pending_input.is_empty())
        {
            self.cancel_source_input(ui.ctx());
            self.pending_reveal = None;
        } else if manual_override {
            // Retire queued keys, retaining the visible source anchor until the
            // overriding pointer has captured its fragment-local coordinates.
            self.cancel_source_input(ui.ctx());
        }
        if !self.pointer.menu_waiting {
            crate::ui::context_menu::cancel_opening(ui.ctx(), Self::focus_id());
        }
    }

    pub(super) fn source_events(
        &mut self,
        ui: &Ui,
        content: &mut CanvasContent<'_>,
        geometry: &CanvasGeometry,
        scroll_y: f32,
    ) -> (Option<SourceViewAction>, bool) {
        if !content.keyboard_enabled || crate::ui::commands::context_menu_is_open(ui.ctx()) {
            return (None, false);
        }
        self.pointer.seen = Some(ui.ctx().frame_nr());
        let events = ui.input(|input| input.events.clone());
        let available = events
            .iter()
            .filter_map(|event| match event {
                Event::PointerButton { pos, .. } | Event::PointerMoved(pos) => Some(*pos),
                _ => None,
            })
            .filter(|point| {
                ui.ctx()
                    .layer_id_at(*point)
                    .is_none_or(|layer| layer == ui.layer_id())
            })
            .collect();
        let frame = InputFrame {
            code_rect: geometry.code_rect,
            scroll: vec2(if content.wrap { 0.0 } else { self.scroll_x }, scroll_y),
            time: ui.input(|input| input.time),
            available,
        };
        let mut focused = self.has_focus(ui.ctx()) || !ui.ctx().wants_keyboard_input();
        let mut clicked = false;
        let actions = ui.input_mut(|input| {
            crate::ui::commands::take_input_actions(&mut input.events, |event| {
                let before = (self.pointer.primary, self.pointer.secondary);
                let pointer = self.capture_pointer(event, content, &frame);
                if before != (self.pointer.primary, self.pointer.secondary)
                    && (self.pointer.primary.is_some() || self.pointer.secondary.is_some())
                {
                    clicked = true;
                    focused = true;
                }
                pointer.or_else(|| {
                    if self.pointer.menu_waiting {
                        crate::ui::context_menu::owned_key(event)
                            .map(|(key, modifiers)| SourceInput::MenuKey(key, modifiers))
                    } else {
                        focused.then(|| source_input(event)).flatten()
                    }
                })
            })
        });
        if clicked {
            self.pending_reveal = None;
            ui.memory_mut(|memory| memory.request_focus(Self::focus_id()));
            crate::ui::commands::retain_navigation_keys(ui.ctx(), Self::focus_id());
        }
        self.enqueue_events(ui, content, &frame, &events, actions);
        if self.pointer.menu_waiting {
            crate::ui::context_menu::hold_opening(ui.ctx(), Self::focus_id(), None);
        }
        self.dispatch_source_events(ui, content, geometry)
    }

    fn enqueue_events(
        &mut self,
        ui: &Ui,
        content: &CanvasContent<'_>,
        frame: &InputFrame,
        events: &[Event],
        mut actions: Vec<SourceInput>,
    ) {
        // A stationary pointer outside the viewport still extends the selection
        // as vertical autoscroll advances beneath it.
        if self.pointer.primary.is_some()
            && !events
                .iter()
                .any(|event| matches!(event, Event::PointerMoved(_)))
        {
            if let Some(point) = ui
                .ctx()
                .pointer_interact_pos()
                .filter(|point| !frame.code_rect.contains(*point))
            {
                actions.extend(self.capture_pointer(&Event::PointerMoved(point), content, frame));
            }
        }
        for event in actions {
            if matches!(
                event,
                SourceInput::Pointer(PointerEvent {
                    kind: PointerKind::Drag,
                    ..
                })
            ) && matches!(
                self.pending_input.back(),
                Some(SourceInput::Pointer(PointerEvent {
                    kind: PointerKind::Drag,
                    ..
                }))
            ) {
                self.pending_input.pop_back();
            }
            self.pending_input.push_back(event);
        }
    }

    fn capture_pointer(
        &mut self,
        event: &Event,
        content: &CanvasContent<'_>,
        frame: &InputFrame,
    ) -> Option<SourceInput> {
        let (kind, screen, modifiers) = match event {
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers,
            } if self.source_text_contains(*pos, content, frame)
                && frame.available.contains(pos) =>
            {
                self.pointer.primary = Some(*pos);
                (PointerKind::Press, *pos, *modifiers)
            }
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers,
            } => {
                let start = self.pointer.primary.take()?;
                (
                    PointerKind::Release {
                        click: start.distance(*pos) <= 6.0,
                    },
                    *pos,
                    *modifiers,
                )
            }
            Event::PointerMoved(pos) if self.pointer.primary.is_some() => {
                (PointerKind::Drag, *pos, Modifiers::NONE)
            }
            Event::PointerButton {
                pos,
                button: PointerButton::Secondary,
                pressed: true,
                ..
            } if self.source_text_contains(*pos, content, frame)
                && frame.available.contains(pos) =>
            {
                self.pointer.secondary = Some(*pos);
                self.pointer.menu_waiting = true;
                return None;
            }
            Event::PointerButton {
                pos,
                button: PointerButton::Secondary,
                pressed: false,
                modifiers,
            } => {
                let start = self.pointer.secondary.take()?;
                if !frame.code_rect.contains(*pos) || start.distance(*pos) > 6.0 {
                    self.pointer.menu_waiting = false;
                    return None;
                }
                (PointerKind::Context, *pos, *modifiers)
            }
            _ => return None,
        };
        let point = frame.code_rect.shrink(0.1).clamp(screen);
        let y = point.y - frame.code_rect.top() + frame.scroll.y;
        let row = self.rows.row_at(y);
        if row >= self.visible.len() {
            return None;
        }
        let mut local = vec2(
            point.x - frame.code_rect.left() - TEXT_INSET + frame.scroll.x,
            y - self.rows.offset(row) - 2.0,
        );
        let fragment = (!content.wrap && self.pending_reveal.is_some())
            .then(|| self.unwrapped.fragment_at_x(row, f64::from(local.x)));
        if let Some(fragment) = fragment {
            local.x -= self.fragment_x(fragment, false);
        }
        Some(SourceInput::Pointer(PointerEvent {
            kind,
            row,
            fragment,
            local,
            screen,
            modifiers,
            time: frame.time,
        }))
    }

    fn source_text_contains(
        &self,
        point: Pos2,
        content: &CanvasContent<'_>,
        frame: &InputFrame,
    ) -> bool {
        if !frame.code_rect.contains(point)
            || self.visible.is_empty()
            || self.scrollbars.iter().any(|rect| rect.contains(point))
        {
            return false;
        }
        let y = point.y - frame.code_rect.top() + frame.scroll.y;
        let row = self.rows.row_at(y);
        let group = if content.wrap {
            row..row + 1
        } else {
            self.unwrapped.row(row)
        };
        let first = &self.fragments[self.visible[group.start]];
        let label = if first.first_in_line && content.test_code.header(first.line).is_some() {
            18.0
        } else {
            0.0
        };
        y < self.rows.offset(group.end) - label
    }

    fn dispatch_source_events(
        &mut self,
        ui: &Ui,
        content: &mut CanvasContent<'_>,
        geometry: &CanvasGeometry,
    ) -> (Option<SourceViewAction>, bool) {
        let (mut action, mut keyboard_reveal) = (None, false);
        while let Some(event) = self.pending_input.front().copied() {
            let next = match event {
                SourceInput::Copy => {
                    self.copy(ui.ctx(), content.buffer);
                    None
                }
                SourceInput::SelectAll => {
                    self.select(0..content.buffer.byte_len());
                    None
                }
                SourceInput::MenuKey(key, modifiers) => {
                    crate::ui::context_menu::hold_opening(
                        ui.ctx(),
                        Self::focus_id(),
                        Some((key, modifiers)),
                    );
                    None
                }
                SourceInput::Move(key, shift) => {
                    let Some(next) = self.move_key(
                        ui,
                        content,
                        &geometry.font,
                        geometry.width,
                        geometry.height,
                        (key, shift),
                    ) else {
                        break;
                    };
                    keyboard_reveal = true;
                    Some(next)
                }
                SourceInput::Pointer(pointer) => {
                    let Some(byte) = self.pointer_byte(ui, content, geometry, pointer) else {
                        break;
                    };
                    let next = self.apply_pointer(ui, content, pointer, byte);
                    if pointer.fragment.is_some()
                        && !matches!(next, Some(SourceViewAction::NavigateToDefinition(_)))
                    {
                        self.pending_reveal = Some((byte, false, true));
                    }
                    next
                }
            };
            self.pending_input.pop_front();
            if matches!(next, Some(SourceViewAction::NavigateToDefinition(_))) {
                self.cancel_source_input(ui.ctx());
                action = next;
                break;
            }
            action = next.or(action);
        }
        if !self.pending_input.is_empty() {
            ui.ctx().request_repaint();
        }
        (action, keyboard_reveal)
    }

    fn pointer_byte(
        &mut self,
        ui: &Ui,
        content: &CanvasContent<'_>,
        geometry: &CanvasGeometry,
        pointer: PointerEvent,
    ) -> Option<usize> {
        let row = pointer.fragment.unwrap_or_else(|| {
            if content.wrap {
                pointer.row
            } else {
                self.unwrapped
                    .fragment_at_x(pointer.row, f64::from(pointer.local.x))
            }
        });
        if pointer.fragment.is_none() && !content.wrap && !self.unwrapped.prefix_measured(row) {
            self.measurement_priority = Some(row);
            return None;
        }
        let part = &self.fragments[self.visible[row]];
        let galley = layout(ui, content, part, &geometry.font, geometry.width);
        Some(hit_byte(
            &galley,
            pointer.local
                - vec2(
                    if pointer.fragment.is_some() {
                        0.0
                    } else {
                        self.fragment_x(row, content.wrap)
                    },
                    0.0,
                ),
            part,
            &content.buffer.text,
        ))
    }

    fn apply_pointer(
        &mut self,
        ui: &Ui,
        content: &mut CanvasContent<'_>,
        pointer: PointerEvent,
        byte: usize,
    ) -> Option<SourceViewAction> {
        match pointer.kind {
            PointerKind::Press => {
                let modifiers = pointer.modifiers;
                if (modifiers.ctrl || modifiers.command) && !modifiers.alt && !modifiers.shift {
                    if let Some(target) = content.definition_at_byte(byte) {
                        self.dragging = false;
                        self.occurrence_clicks = None;
                        self.hover.dismiss();
                        return Some(SourceViewAction::NavigateToDefinition(target));
                    }
                }
                self.begin_selection(content, byte, modifiers.shift, false);
                self.dragging = true;
            }
            PointerKind::Drag if self.dragging => {
                self.selection.as_mut()?.focus = byte;
            }
            PointerKind::Release { click } if self.dragging => {
                self.selection.as_mut()?.focus = byte;
                self.dragging = false;
                let selected = self
                    .selection
                    .is_some_and(|selection| !selection.range().is_empty());
                if click && !selected && pointer.modifiers == Modifiers::NONE {
                    return self
                        .occurrence_click(pointer.screen, pointer.time, byte)
                        .or_else(|| Some(move_caret(content, byte)));
                }
                self.occurrence_clicks = None;
            }
            PointerKind::Context => {
                self.begin_selection(content, byte, pointer.modifiers.shift, true);
                self.dragging = false;
                self.pointer.menu_waiting = false;
                self.context_byte = Some(byte);
                self.menu_request = Some(pointer.screen);
            }
            _ => return None,
        }
        ui.ctx().request_repaint();
        Some(move_caret(content, byte))
    }

    fn begin_selection(
        &mut self,
        content: &CanvasContent<'_>,
        byte: usize,
        extending: bool,
        secondary: bool,
    ) {
        if secondary
            && self
                .selection
                .is_some_and(|selection| selection.range().contains(&byte))
        {
            return;
        }
        let previous = content
            .buffer
            .byte_offset(*content.caret_line, *content.caret_col);
        let anchor = if extending {
            self.selection
                .map_or(previous, |selection| selection.anchor)
        } else {
            byte
        };
        self.selection = Some(ByteSelection {
            anchor,
            focus: byte,
        });
    }
}
