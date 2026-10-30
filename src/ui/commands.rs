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
    /// Reloads current source bytes while preserving the caret destination.
    ReloadSource,
    /// Generates a fresh index in isolated build-input workspaces.
    GenerateIndex,
    /// Cancels index generation while retaining published semantic data.
    CancelIndexGeneration,
    /// Probes executable versions for the detected indexing tools.
    CheckIndexTools,
    /// Stops pending executable version probes.
    CancelIndexToolCheck,
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

/// Takes reader shortcuts from the input stream with exact modifier matching.
///
/// Bare keys belong to the focused control. While a sheet is open its input is
/// exclusive, so typing in it cannot invoke commands in the source behind it.
pub fn take_shortcuts(ctx: &Context, modal_open: bool, source_focused: bool) -> Vec<ReaderCommand> {
    if modal_open {
        return Vec::new();
    }
    let typing = ctx.wants_keyboard_input() && !source_focused;
    let mut commands = Vec::new();
    ctx.input_mut(|input| {
        input.events.retain(|event| {
            let Event::Key {
                key,
                modifiers,
                pressed: true,
                ..
            } = event
            else {
                return true;
            };
            let Some(command) = shortcut(*key, *modifiers, typing) else {
                return true;
            };
            commands.push(command);
            false
        });
    });
    commands
}

fn shortcut(key: Key, modifiers: Modifiers, typing: bool) -> Option<ReaderCommand> {
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
    fn typing_owns_bare_keys_and_number_shortcuts() {
        assert!(shortcut(Key::Enter, Modifiers::NONE, true).is_none());
        assert!(shortcut(Key::Num1, Modifiers::CTRL, true).is_none());
        assert!(shortcut(Key::ArrowDown, Modifiers::NONE, false).is_none());
    }
}
