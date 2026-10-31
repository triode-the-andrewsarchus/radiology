//! Commands shared by menus, toolbar controls, and keyboard shortcuts.

use crate::critical::lens::SourceMode;
use crate::source::location::ReviewRevision;
use egui::{Context, Event, Key, Modifiers};

/// Actions triggered by interacting with controls in the reader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReaderCommand {
    /// Opens the native directory chooser dialog to select a repository root.
    OpenRepositoryDialog,
    /// Opens a recent repository path.
    OpenRecentRepo(String),
    /// Copies the repository root canonical path to clipboard.
    CopyRepoPath,
    /// Toggles visibility of the left navigation sidebar.
    ToggleSidebar,
    /// Toggles visibility of the right context inspector.
    ToggleInspector,
    /// Navigates backwards in the file and caret navigation history.
    NavigateBack,
    /// Navigates forward in the file and caret navigation history.
    NavigateForward,
    /// Requests jumping to the next modified hunk in the active pull request review.
    NextChangeBlock,
    /// Requests jumping to the previous modified hunk in the active pull request review.
    PrevChangeBlock,
    /// Toggles a bookmark on the currently focused source line.
    ToggleBookmark,
    /// Opens the quick file picker modal dialog (Ctrl/Cmd+K).
    OpenQuickFilePicker,
    /// Opens the jump-to-line modal dialog (Ctrl/Cmd+L).
    OpenGoToLine,
    /// Opens the jump-to-symbol outline search dialog (Ctrl/Cmd+T).
    OpenGoToSymbol,
    /// Switches to the search tab in the sidebar.
    OpenSearch,
    /// Opens local find in the current source document.
    FindInFile,
    /// Reveals the next local match.
    NextFindMatch,
    /// Reveals the previous local match.
    PreviousFindMatch,
    /// Opens the pull request number prompt dialog.
    OpenPRDialog,
    /// Opens a recent pull request by number using fresh pinned metadata.
    OpenPullRequest(usize),
    /// Refreshes recent PR choices independently of the active review.
    RefreshPRList,
    /// Cancels a list refresh without discarding accepted entries.
    CancelPRList,
    /// Reloads current source bytes while preserving the caret destination.
    ReloadSource,
    /// Generates a fresh index in isolated build-input workspaces.
    GenerateIndex,
    /// Reuses exact version-matched projects and generates missing or stale projects.
    UpdateIndex,
    /// Cancels index generation while retaining published semantic data.
    CancelIndexGeneration,
    /// Probes executable versions for the detected indexing tools.
    CheckIndexTools,
    /// Stops pending executable version probes.
    CancelIndexToolCheck,
    /// Installs one missing tool explicitly, then regenerates the current index.
    InstallIndexer(crate::semantic::indexing::Indexer),
    /// Cancels installation while retaining published semantic data.
    CancelToolInstallation,
    /// Reloads a repository-provided SCIP index in the background.
    ReloadIndexImport,
    /// Cancels an import while retaining any published semantic data.
    CancelIndexImport,
    /// Refreshes the currently reviewed pull request from GitHub.
    RefreshPR,
    /// Exits the active pull request review session and clears diff overlays.
    ExitPRReview,
    /// Changes the active [`SourceMode`] (e.g. Normal vs Critical lens).
    ToggleMode(SourceMode),
    /// Switches the active review revision (Before vs After).
    ToggleRevision(ReviewRevision),
    /// Toggles the filter to hide non-source code files.
    ToggleSourceFilesOnly,
    /// Toggles the filter to hide test directories and test files.
    ToggleHideTests,
    /// Toggles between dark and light appearance themes.
    ToggleTheme,
    /// Opens a numbered bookmark using anchor relocation.
    OpenBookmark(usize),
    /// Follows the indexed symbol at the caret.
    GoToDefinition,
    /// Pins incoming references to the exact indexed symbol at the caret.
    FindReferences,
    /// Collapses function bodies in the current document.
    CollapseFunctions,
    /// Folds declarations in the current document.
    FoldToDefinitions,
    /// Switches to the other available review revision.
    ToggleCurrentRevision,
    /// Clears repository source filters.
    ShowAllFiles,
    /// Increases only the source font size.
    IncreaseSourceFont,
    /// Decreases only the source font size.
    DecreaseSourceFont,
    /// Restores the default source font size.
    ResetSourceFont,
    /// Toggles whether long source code lines are soft-wrapped.
    ToggleWrapLines,
}

pub(crate) use super::context_menu::{
    close as close_context_menu, is_open as context_menu_is_open, show as context_menu,
    take_input as take_context_menu_input, Item as MenuItem,
};

/// Takes reader shortcuts from the input stream with exact modifier matching.
///
/// Bare keys belong to the focused control. While a sheet is open its input is
/// exclusive, so typing in it cannot invoke commands in the source behind it.
/// `reader_focused` includes reader-owned non-text controls; their local bindings
/// must consume their keys before this dispatcher runs.
pub fn take_shortcuts(ctx: &Context, modal_open: bool, reader_focused: bool) -> Vec<ReaderCommand> {
    if modal_open {
        return Vec::new();
    }
    let menu_open = context_menu_is_open(ctx);
    let typing = ctx.wants_keyboard_input() && !reader_focused && !menu_open;
    ctx.input_mut(|input| {
        take_input_actions(&mut input.events, |event| match event {
            Event::Key {
                key,
                modifiers,
                pressed: true,
                ..
            } if !menu_open || super::context_menu::owned_key(event).is_none() => {
                shortcut(*key, *modifiers, typing)
            }
            _ => None,
        })
    })
}

/// Consumes recognized key presses in event order, preserving all other input.
pub(crate) fn take_key_actions<T>(
    ctx: &Context,
    mut resolve: impl FnMut(Key, Modifiers) -> Option<T>,
) -> Vec<T> {
    ctx.input_mut(|input| take_event_actions(&mut input.events, &mut resolve))
}

/// Consumes owned presses before framework focus handling or from a prepared frame.
pub(crate) fn take_event_actions<T>(
    events: &mut Vec<Event>,
    mut resolve: impl FnMut(Key, Modifiers) -> Option<T>,
) -> Vec<T> {
    take_input_actions(events, |event| {
        if let Event::Key {
            key,
            modifiers,
            pressed: true,
            ..
        } = event
        {
            resolve(*key, *modifiers)
        } else {
            None
        }
    })
}

/// Consumes owned input events in order, including platform copy events.
pub(crate) fn take_input_actions<T>(
    events: &mut Vec<Event>,
    mut resolve: impl FnMut(&Event) -> Option<T>,
) -> Vec<T> {
    let mut actions = Vec::new();
    events.retain(|event| {
        if let Some(action) = resolve(event) {
            actions.push(action);
            false
        } else {
            true
        }
    });
    actions
}

pub(super) fn shortcut(key: Key, modifiers: Modifiers, typing: bool) -> Option<ReaderCommand> {
    use ReaderCommand::*;
    let primary = modifiers.command || modifiers.ctrl;
    match (primary, modifiers.alt, modifiers.shift, key) {
        (true, false, false, Key::O) => Some(OpenRepositoryDialog),
        (true, false, false, Key::K) => Some(OpenQuickFilePicker),
        (true, false, false, Key::L) => Some(OpenGoToLine),
        (true, false, false, Key::T) => Some(OpenGoToSymbol),
        (true, false, false, Key::P) => Some(OpenPRDialog),
        (true, false, false, Key::R) => Some(RefreshPR),
        (true, true, false, Key::R) if !typing => Some(FindReferences),
        (true, true, false, Key::J) if !typing => Some(GoToDefinition),
        (true, false, true, Key::I) => Some(GenerateIndex),
        (true, false, false, Key::D) => Some(ToggleBookmark),
        (true, false, false, Key::OpenBracket) => Some(NavigateBack),
        (true, false, false, Key::CloseBracket) => Some(NavigateForward),
        (true, false, false, Key::Plus | Key::Equals)
        | (true, false, true, Key::Plus | Key::Equals) => Some(IncreaseSourceFont),
        (true, false, false, Key::Minus) => Some(DecreaseSourceFont),
        (true, true, false, Key::Equals) => Some(ResetSourceFont),
        (true, false, false, Key::F) => Some(FindInFile),
        (true, false, false, Key::G) => Some(NextFindMatch),
        (true, false, true, Key::G) => Some(PreviousFindMatch),
        (true, false, true, Key::F) => Some(OpenSearch),
        (true, false, true, Key::Num0) => Some(CollapseFunctions),
        (true, true, false, Key::T) => Some(ToggleTheme),
        (true, true, false, Key::S) => Some(ToggleSidebar),
        (true, true, false, Key::I) => Some(ToggleInspector),
        (true, true, false, Key::O) => Some(ToggleSourceFilesOnly),
        (true, true, false, Key::H) => Some(ToggleHideTests),
        (true, true, false, Key::W) => Some(ToggleWrapLines),
        (true, true, false, Key::B) => Some(ToggleCurrentRevision),
        (true, true, false, Key::Num0) => Some(FoldToDefinitions),
        (true, true, false, Key::Num1) => Some(ToggleMode(SourceMode::Normal)),
        (true, true, false, Key::Num2) => Some(ToggleMode(SourceMode::Critical)),
        (false, true, false, Key::ArrowDown) => Some(NextChangeBlock),
        (false, true, false, Key::ArrowUp) => Some(PrevChangeBlock),
        (false, false, false, Key::F12) if !typing => Some(GoToDefinition),
        (false, false, false, Key::Enter) if !typing => Some(GoToDefinition),
        (true, false, false, key) if !typing => digit(key).map(OpenBookmark),
        _ => None,
    }
}

fn digit(key: Key) -> Option<usize> {
    const DIGITS: [Key; 10] = [
        Key::Num0,
        Key::Num1,
        Key::Num2,
        Key::Num3,
        Key::Num4,
        Key::Num5,
        Key::Num6,
        Key::Num7,
        Key::Num8,
        Key::Num9,
    ];
    DIGITS.iter().position(|&digit| digit == key)
}

/// Consumes only a focused list's unmodified keys before egui moves focus.
pub(crate) fn take_focused_keys(
    ctx: &Context,
    input: &mut egui::RawInput,
    focus: egui::Id,
    keys: &[Key],
) -> Vec<Key> {
    if context_menu_is_open(ctx) || !ctx.memory(|memory| memory.has_focus(focus)) {
        return Vec::new();
    }
    take_event_actions(&mut input.events, |key, modifiers| {
        (modifiers == Modifiers::NONE && keys.contains(&key)).then_some(key)
    })
}

/// Resolves standard list movement, including an initially empty selection.
pub(crate) fn list_selection(key: Key, selected: Option<usize>, count: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    match key {
        Key::Home => Some(0),
        Key::End => Some(last),
        Key::ArrowUp => Some(selected.map_or(last, |index| index.saturating_sub(1).min(last))),
        Key::ArrowDown => Some(selected.map_or(0, |index| index.saturating_add(1).min(last))),
        _ => None,
    }
}

/// Reveals a fixed-height row without disturbing an already visible selection.
pub(crate) fn reveal_list_row(
    ui: &egui::Ui,
    scroll: egui::ScrollArea,
    id: egui::Id,
    row: Option<usize>,
    height: f32,
) -> egui::ScrollArea {
    let Some(row) = row else {
        return scroll;
    };
    let offset = egui::scroll_area::State::load(ui.ctx(), ui.make_persistent_id(id))
        .map_or(0.0, |state| state.offset.y);
    let top = row as f32 * (height + ui.spacing().item_spacing.y);
    let bottom = top + height;
    let available = ui.available_height();
    let next = if top < offset {
        top
    } else if bottom > offset + available {
        bottom - available
    } else {
        offset
    };
    scroll.vertical_scroll_offset(next.max(0.0))
}

/// Keeps arrows and Escape under explicit dispatch while leaving Tab available.
pub(crate) fn retain_navigation_keys(ctx: &egui::Context, id: egui::Id) {
    ctx.memory_mut(|memory| {
        memory.set_focus_lock_filter(
            id,
            egui::EventFilter {
                horizontal_arrows: true,
                vertical_arrows: true,
                escape: true,
                ..Default::default()
            },
        )
    });
}

#[cfg(test)]
pub(crate) fn test_key(key: Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

#[cfg(test)]
pub(crate) fn test_input(keys: &[Key]) -> egui::RawInput {
    egui::RawInput {
        events: keys.iter().copied().map(test_key).collect(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dispatch(key: Key, modifiers: Modifiers, modal: bool) -> Vec<ReaderCommand> {
        let ctx = Context::default();
        let mut result = Vec::new();
        let _ = ctx.run(
            egui::RawInput {
                events: vec![Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                }],
                ..Default::default()
            },
            |ctx| {
                result = take_shortcuts(ctx, modal, false);
                if !modal {
                    assert!(!ctx.input(|input| input.key_pressed(key)));
                }
            },
        );
        result
    }

    #[test]
    fn context_menu_escape_closes_without_action_and_leaves_closed_menu_input_unowned() {
        use crate::ui::test_support::{click, painted_frame, text_center};
        use std::cell::Cell;
        let ctx = Context::default();
        let owner = egui::Id::new("reader_owner");
        let paints = Cell::new(0);
        let invoked = Cell::new(false);
        let frame = |events| {
            let (mut open, mut escape_left) = (false, false);
            let output = painted_frame(&ctx, egui::vec2(500.0, 300.0), events, |ui| {
                ui.interact(
                    ui.max_rect(),
                    owner,
                    egui::Sense::focusable_noninteractive(),
                );
                let row = ui.button("Reader row");
                if context_menu(&row, "test", || {
                    paints.set(paints.get() + 1);
                    vec![MenuItem::action("Menu action", ())]
                })
                .is_some()
                {
                    invoked.set(true);
                }
                open = context_menu_is_open(ui.ctx());
                escape_left = ui.input(|input| input.key_pressed(Key::Escape));
            });
            (output, open, escape_left)
        };
        let (output, _, _) = frame(vec![]);
        let row = text_center(&output, "Reader row");
        ctx.memory_mut(|memory| memory.request_focus(owner));
        let secondary = || crate::ui::test_support::secondary_click(row);
        assert!(frame(secondary()).1);
        frame(vec![]);
        let before = paints.get();
        ctx.memory_mut(|memory| memory.request_focus(owner));
        let focus_before_escape = ctx.memory(|memory| memory.focused());
        let mut input = test_input(&[Key::Escape]);
        take_context_menu_input(&ctx, &mut input);
        assert!(
            input.events.is_empty(),
            "menu owns Escape before list dispatch"
        );
        assert!(take_focused_keys(&ctx, &mut input, owner, &[Key::Escape]).is_empty());
        let (_, open, escape_left) = frame(input.events);
        assert!(!open, "Escape must retire the open context menu");
        assert!(!escape_left, "the open menu owns Escape");
        assert_eq!(
            paints.get(),
            before,
            "dismissal must not invoke menu contents"
        );
        assert!(!invoked.get());
        assert_eq!(ctx.memory(|memory| memory.focused()), focus_before_escape);
        let mut input = test_input(&[Key::Escape]);
        take_context_menu_input(&ctx, &mut input);
        assert_eq!(input.events.len(), 1, "closed menus leave raw input alone");
        let (_, open, escape_left) = frame(vec![test_key(Key::Escape)]);
        assert!(!open);
        assert!(
            escape_left,
            "a closed menu must leave Escape for its focused owner"
        );
        frame(secondary());
        let (output, open, _) = frame(vec![]);
        assert!(open);
        let (_, open, _) = frame(click(text_center(&output, "Menu action")));
        assert!(
            invoked.get(),
            "reopened menus must keep their ordinary actions"
        );
        assert!(!open);
    }

    #[test]
    fn modified_commands_do_not_trigger_unmodified_commands() {
        let primary = Modifiers {
            ctrl: true,
            ..Default::default()
        };
        let alternate = Modifiers {
            alt: true,
            ..primary
        };
        assert_eq!(
            dispatch(Key::T, alternate, false),
            vec![ReaderCommand::ToggleTheme]
        );
        assert_eq!(
            dispatch(Key::Num1, alternate, false),
            vec![ReaderCommand::ToggleMode(SourceMode::Normal)]
        );
        assert_eq!(
            dispatch(Key::Num1, primary, false),
            vec![ReaderCommand::OpenBookmark(1)]
        );
        assert_eq!(
            dispatch(Key::O, primary, false),
            vec![ReaderCommand::OpenRepositoryDialog]
        );
        assert!(dispatch(Key::O, primary, true).is_empty());
    }

    #[test]
    fn reference_shortcut_is_distinct_from_refresh_and_respects_text_focus() {
        let modifiers = Modifiers {
            ctrl: true,
            alt: true,
            ..Default::default()
        };
        assert_eq!(
            dispatch(Key::R, modifiers, false),
            vec![ReaderCommand::FindReferences]
        );
        assert_eq!(
            dispatch(Key::R, Modifiers::CTRL, false),
            vec![ReaderCommand::RefreshPR]
        );
        assert!(shortcut(Key::R, modifiers, true).is_none());
        assert!(dispatch(Key::R, modifiers, true).is_empty());
    }

    #[test]
    fn definition_and_reindex_shortcuts_are_distinct_and_modal_safe() {
        let definition = Modifiers::CTRL | Modifiers::ALT;
        let reindex = Modifiers::CTRL | Modifiers::SHIFT;
        assert_eq!(
            dispatch(Key::J, definition, false),
            vec![ReaderCommand::GoToDefinition]
        );
        assert_eq!(
            dispatch(Key::I, reindex, false),
            vec![ReaderCommand::GenerateIndex]
        );
        assert_eq!(
            dispatch(Key::I, definition, false),
            vec![ReaderCommand::ToggleInspector]
        );
        assert!(shortcut(Key::J, definition, true).is_none());
        for (key, modifiers) in [(Key::J, definition), (Key::I, reindex)] {
            assert!(dispatch(key, modifiers, true).is_empty());
            assert!(shortcut(key, modifiers | Modifiers::ALT | Modifiers::SHIFT, false).is_none());
        }
    }

    #[test]
    fn typing_owns_bare_keys_and_number_shortcuts() {
        assert!(shortcut(Key::Enter, Modifiers::NONE, true).is_none());
        assert!(shortcut(Key::Num1, Modifiers::CTRL, true).is_none());
        assert!(shortcut(Key::ArrowDown, Modifiers::NONE, false).is_none());
    }
}
