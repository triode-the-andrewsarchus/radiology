//! Main application model and EGUI event loop lifecycle.

use egui::{CentralPanel, Context, SidePanel, TopBottomPanel};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::bookmarks::store::BookmarkStore;
use crate::critical::lens::SourceMode;
use crate::pull_request::change::FileChange;
use crate::pull_request::diff::{FileLineChanges, UnifiedDiffParser};
use crate::pull_request::github::{GitHubClient, PullRequestSummary};
use crate::repository::discovery::RepositoryDiscovery;
use crate::repository::file_node::{FileNode, RepositoryFile};
use crate::search::match_item::SearchMatch;
use crate::search::ripgrep::{SearchOptions, SearchRunner};
use crate::semantic::caller_tree::{CallerTreeBuilder, ReferenceCallerNode};
use crate::semantic::index::SemanticIndex;
use crate::session::store::SessionStore;
use crate::source::buffer::SourceBuffer;
use crate::source::history::NavigationHistory;
use crate::source::location::ReviewRevision;
use crate::source::selection::OccurrencePinStore;
use crate::syntax::folds::{FoldDetector, FoldRange, FoldStore};
use crate::syntax::highlighter::{HighlightedSpan, SyntaxHighlighter};
use crate::syntax::outline::{DocumentSymbol, SymbolExtractor};
use crate::theme::AppTheme;
use crate::ui::context_inspector::{ContextInspector, InspectorAction, InspectorTab};
use crate::ui::icons::Icon;
use crate::ui::modals::{
    GoToLineModal, GoToSymbolModal, ModalAction, PullRequestModal, QuickFilePickerModal,
};
use crate::ui::sidebar::{Sidebar, SidebarAction, SidebarTab};
use crate::ui::source_view::{SourceView, SourceViewAction};
use crate::ui::toolbar::{Toolbar, ToolbarAction};

/// Active modal sheet displayed over the main window.
pub enum ActiveModal {
    /// No modal active.
    None,
    /// Quick file search modal (Ctrl+K).
    QuickFilePicker(QuickFilePickerModal),
    /// Jump to line modal (Ctrl+L).
    GoToLine(GoToLineModal),
    /// Symbol search modal (Ctrl+T).
    GoToSymbol(GoToSymbolModal),
    /// GitHub pull request selector modal (Ctrl+P).
    PullRequest(PullRequestModal),
}

/// The top-level application state managing all reader panels and subsystems.
pub struct RadiologyApp {
    /// Active color theme (dark or light).
    pub theme: AppTheme,
    /// Persisted session state.
    pub session: SessionStore,
    /// Canonical filesystem root of the opened repository.
    pub repository_root: Option<PathBuf>,
    /// Display name of the repository.
    pub repository_name: Option<String>,
    /// Current Git branch name if inside a repository.
    pub branch_name: Option<String>,
    /// All discovered candidate files.
    pub all_files: Vec<RepositoryFile>,
    /// Hierarchical directory tree of filtered files.
    pub file_tree: Vec<FileNode>,
    /// Set of expanded directory paths.
    pub expanded_dirs: HashSet<String>,
    /// Relative path of the currently selected file.
    pub selected_path: Option<String>,
    /// Loaded source buffer for the active file.
    pub open_buffer: Option<SourceBuffer>,
    /// 1-based line of the active caret.
    pub caret_line: usize,
    /// 0-based column of the active caret.
    pub caret_col: usize,
    /// Linear navigation history.
    pub history: NavigationHistory,
    /// Syntax highlighting engine.
    pub highlighter: SyntaxHighlighter,
    /// Cached highlighted lines for active source buffer.
    pub highlighted_lines: Vec<Vec<HighlightedSpan>>,
    /// Foldable line ranges in the open file.
    pub fold_ranges: Vec<FoldRange>,
    /// Fold store tracking collapsed line numbers.
    pub fold_store: FoldStore,
    /// Tree-sitter document outline symbols.
    pub outline_symbols: Vec<DocumentSymbol>,
    /// Pinned token occurrence groups.
    pub pin_store: OccurrencePinStore,
    /// SCIP semantic index if loaded.
    pub semantic_index: Option<SemanticIndex>,
    /// Backward caller tree nodes for the symbol under caret.
    pub caller_tree: Vec<ReferenceCallerNode>,
    /// Active pull request summary during PR review.
    pub active_pr: Option<PullRequestSummary>,
    /// Changed files in the pull request under review.
    pub pr_changes: Vec<FileChange>,
    /// Active revision (Before or After) during PR review.
    pub active_revision: Option<ReviewRevision>,
    /// Parsed line-level diff hunks for the open document.
    pub line_changes: Option<FileLineChanges>,
    /// Source presentation mode (Normal or Critical Lens).
    pub mode: SourceMode,
    /// Selected tab in the left sidebar.
    pub sidebar_tab: SidebarTab,
    /// Selected tab in the right context inspector.
    pub inspector_tab: InspectorTab,
    /// Search options for repository ripgrep queries.
    pub search_options: SearchOptions,
    /// Results of the last repository search.
    pub search_matches: Vec<SearchMatch>,
    /// Whether an asynchronous search is currently running.
    pub is_searching: bool,
    /// Cancellation flag for the background search process.
    pub search_cancel: Arc<AtomicBool>,
    /// Background receiver for streaming search results.
    #[allow(clippy::type_complexity)]
    pub search_rx: Option<std::sync::mpsc::Receiver<Vec<SearchMatch>>>,
    /// Repository and PR bookmarks.
    pub bookmarks: BookmarkStore,
    /// Active modal dialog sheet.
    pub modal: ActiveModal,
    /// Target line to scroll and center into view in the code editor.
    pub scroll_to_line: Option<usize>,
    /// Whether the left navigation sidebar is visible.
    pub show_sidebar: bool,
    /// Whether the right context inspector is visible.
    pub show_inspector: bool,
}

impl RadiologyApp {
    /// Creates a new [`RadiologyApp`] instance initialized with session defaults.
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let session = SessionStore::load();
        let mut app = Self {
            theme: AppTheme::dark(),
            session,
            repository_root: None,
            repository_name: None,
            branch_name: None,
            all_files: Vec::new(),
            file_tree: Vec::new(),
            expanded_dirs: HashSet::new(),
            selected_path: None,
            open_buffer: None,
            caret_line: 1,
            caret_col: 0,
            history: NavigationHistory::new(),
            highlighter: SyntaxHighlighter::new(),
            highlighted_lines: Vec::new(),
            fold_ranges: Vec::new(),
            fold_store: FoldStore::new(),
            outline_symbols: Vec::new(),
            pin_store: OccurrencePinStore::new(),
            semantic_index: None,
            caller_tree: Vec::new(),
            active_pr: None,
            pr_changes: Vec::new(),
            active_revision: None,
            line_changes: None,
            mode: SourceMode::Normal,
            sidebar_tab: SidebarTab::Files,
            inspector_tab: InspectorTab::Symbol,
            search_options: SearchOptions::default(),
            search_matches: Vec::new(),
            is_searching: false,
            search_cancel: Arc::new(AtomicBool::new(false)),
            search_rx: None,
            bookmarks: BookmarkStore::new(),
            modal: ActiveModal::None,
            scroll_to_line: None,
            show_sidebar: true,
            show_inspector: true,
        };

        if app.session.is_valid() {
            if let Some(repo_str) = &app.session.last_repository {
                let path = PathBuf::from(repo_str);
                app.open_repository(&path);
                if let Some(pr_num) = app.session.active_pr {
                    if let Ok(pr) = GitHubClient::view_pull_request(&path, pr_num) {
                        app.active_pr = Some(pr);
                        if let Ok(diff) = GitHubClient::get_pr_diff(&path, pr_num) {
                            app.line_changes = Some(UnifiedDiffParser::parse(&diff));
                            app.pr_changes = UnifiedDiffParser::parse_file_changes(&diff);
                            if !app.pr_changes.is_empty() {
                                app.sidebar_tab = SidebarTab::Changes;
                            }
                        }
                    }
                }
            }
        }

        app
    }

    /// Opens the repository located at `path`, discovering files and restoring bookmarks.
    pub fn open_repository(&mut self, path: &Path) {
        let root = RepositoryDiscovery::find_root(path);
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repository")
            .to_string();

        self.repository_root = Some(root.clone());
        self.repository_name = Some(name);
        self.session.record_repository(root.to_string_lossy());

        if let Ok(output) = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&root)
            .output()
        {
            if output.status.success() {
                let b = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !b.is_empty() {
                    self.branch_name = Some(b);
                }
            }
        }

        if let Ok(files) = RepositoryDiscovery::discover_files(&root) {
            self.all_files = files;
            self.refresh_file_tree();
        }

        self.bookmarks = BookmarkStore::load(&root.to_string_lossy(), None);

        for scip_candidate in [root.join("index.scip"), root.join(".scip")] {
            if scip_candidate.exists() {
                if let Ok(idx) = SemanticIndex::load_file(&scip_candidate) {
                    self.semantic_index = Some(idx);
                    break;
                }
            }
        }

        let first_file = self
            .all_files
            .iter()
            .find(|f| f.path.to_lowercase() == "readme.md")
            .or_else(|| self.all_files.iter().find(|f| f.is_source))
            .or_else(|| self.all_files.first());

        if let Some(f) = first_file {
            let path_clone = f.path.clone();
            self.open_file(&path_clone);
        }

        self.refresh_git_changes();
    }

    /// Rebuilds the hierarchical file tree applying active filters.
    pub fn refresh_file_tree(&mut self) {
        let filtered = self.session.source_filters.filter_files(&self.all_files);
        self.file_tree = FileNode::build_tree(&filtered);
    }

    /// Reads and displays the file at `rel_path`.
    pub fn open_file(&mut self, rel_path: &str) {
        self.open_file_internal(rel_path, true);
    }

    fn open_file_internal(&mut self, rel_path: &str, record_history: bool) {
        let Some(root) = &self.repository_root else {
            return;
        };
        let full_path = root.join(rel_path);

        if let Ok(bytes) = std::fs::read(&full_path) {
            if let Ok(buffer) = SourceBuffer::from_bytes(rel_path, bytes, self.active_revision) {
                self.selected_path = Some(rel_path.to_string());
                self.caret_line = 1;
                self.caret_col = 0;
                self.scroll_to_line = Some(1);

                self.outline_symbols = SymbolExtractor::extract(&buffer);
                self.fold_ranges = FoldDetector::detect_folds(&buffer);
                self.fold_store.unfold_all();

                let syntax_ref = self.highlighter.find_syntax_for_path(&buffer.path);
                self.highlighted_lines = self.highlighter.highlight_document(
                    syntax_ref,
                    &buffer.text,
                    self.theme.is_dark,
                );

                if record_history {
                    self.history.visit(buffer.location_at_byte(0));
                }
                self.open_buffer = Some(buffer);

                self.refresh_symbol_context();
            }
        }
    }

    /// Navigates directly to a file, line number, and byte offset.
    pub fn jump_to_location(&mut self, rel_path: &str, line: usize, byte_offset: usize) {
        self.navigate_to_location(rel_path, line, byte_offset, true);
    }

    fn navigate_to_location(
        &mut self,
        rel_path: &str,
        line: usize,
        byte_offset: usize,
        record_history: bool,
    ) {
        if self.selected_path.as_deref() != Some(rel_path) {
            self.open_file_internal(rel_path, record_history);
        }
        if let Some(buf) = &self.open_buffer {
            let target_line = if byte_offset > 0 {
                buf.line_at_byte(byte_offset)
            } else {
                line.clamp(1, buf.line_count())
            };
            self.caret_line = target_line;
            self.caret_col = 0;
            self.scroll_to_line = Some(target_line);
            let offset = if byte_offset > 0 {
                byte_offset
            } else {
                buf.byte_offset(self.caret_line, 0)
            };
            if record_history {
                self.history.visit(buf.location_at_byte(offset));
            }
            self.refresh_symbol_context();
        }
    }

    /// Refreshes working tree Git changes when no pull request is actively being reviewed.
    pub fn refresh_git_changes(&mut self) {
        if self.active_pr.is_some() {
            return;
        }
        let Some(root) = &self.repository_root else {
            return;
        };

        let mut changes = Vec::new();

        if let Ok(output) = std::process::Command::new("git")
            .args(["diff", "HEAD"])
            .current_dir(root)
            .output()
        {
            if output.status.success() {
                let diff_text = String::from_utf8_lossy(&output.stdout);
                changes = UnifiedDiffParser::parse_file_changes(&diff_text);
                if let Some(buf) = &self.open_buffer {
                    if let Ok(file_diff) = std::process::Command::new("git")
                        .args(["diff", "-U0", "HEAD", "--", &buf.path])
                        .current_dir(root)
                        .output()
                    {
                        if file_diff.status.success() {
                            let text = String::from_utf8_lossy(&file_diff.stdout);
                            self.line_changes = Some(UnifiedDiffParser::parse(&text));
                        }
                    }
                }
            }
        }

        if let Ok(output) = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(root)
            .output()
        {
            if output.status.success() {
                let status_text = String::from_utf8_lossy(&output.stdout);
                for line in status_text.lines() {
                    if let Some(rel) = line.strip_prefix("?? ") {
                        let rel = rel.trim();
                        let full = root.join(rel);
                        if full.is_file() {
                            let count = std::fs::read_to_string(&full)
                                .map(|s| s.lines().count())
                                .unwrap_or(0);
                            changes.push(FileChange {
                                path: rel.to_string(),
                                old_path: None,
                                status: crate::pull_request::change::FileChangeStatus::Added,
                                before_blob: String::new(),
                                after_blob: String::new(),
                                additions: count,
                                deletions: 0,
                                is_binary: false,
                            });
                        }
                    }
                }
            }
        }

        self.pr_changes = changes;
    }

    /// Traces incoming callers using ripgrep fallback when SCIP index is not present.
    fn find_callers_fallback(
        &self,
        sym_name: &str,
        def_file: &str,
        def_line: usize,
    ) -> Vec<ReferenceCallerNode> {
        let Some(root) = &self.repository_root else {
            return Vec::new();
        };

        if sym_name.len() < 2 || matches!(sym_name, "new" | "default" | "from" | "into" | "clone") {
            return Vec::new();
        }

        let output = match std::process::Command::new("rg")
            .args(["--json", "-w", sym_name, "."])
            .current_dir(root)
            .output()
        {
            Ok(out) if out.status.success() => out,
            _ => return Vec::new(),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut callers = Vec::new();
        let mut seen = HashSet::new();

        for line in stdout.lines() {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                if v.get("type").and_then(|t| t.as_str()) == Some("match") {
                    if let Some(data) = v.get("data") {
                        let path = data
                            .get("path")
                            .and_then(|p| p.get("text"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .trim_start_matches("./")
                            .to_string();
                        let line_no = data
                            .get("line_number")
                            .and_then(|l| l.as_u64())
                            .unwrap_or(0) as usize;
                        let line_text = data
                            .get("lines")
                            .and_then(|l| l.get("text"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .trim();

                        if path == def_file && line_no == def_line {
                            continue;
                        }

                        let caller_name = if let Some(fn_idx) = line_text.find("fn ") {
                            line_text[fn_idx + 3..]
                                .split(|c: char| !c.is_alphanumeric() && c != '_')
                                .next()
                                .unwrap_or(sym_name)
                                .to_string()
                        } else {
                            line_text
                                .split('(')
                                .next()
                                .and_then(|s| s.split_whitespace().next_back())
                                .and_then(|s| s.rsplit("::").next())
                                .and_then(|s| s.rsplit('.').next())
                                .unwrap_or(sym_name)
                                .to_string()
                        };

                        let key = format!("{path}:{line_no}");
                        if !seen.insert(key) {
                            continue;
                        }

                        let slot =
                            crate::semantic::caller_tree::caller_pill_slot(&caller_name, &path);
                        callers.push(ReferenceCallerNode {
                            name: caller_name,
                            path,
                            line: line_no,
                            depth: 0,
                            color_slot: slot,
                            is_recursive: false,
                            children: Vec::new(),
                        });

                        if callers.len() >= 30 {
                            break;
                        }
                    }
                }
            }
        }

        callers
    }

    /// Updates semantic caller context for the symbol at the caret or enclosing declaration.
    pub fn refresh_symbol_context(&mut self) {
        let Some(buf) = &self.open_buffer else {
            return;
        };

        let line_0 = self.caret_line.saturating_sub(1);
        let caret_byte = buf.byte_offset(self.caret_line, self.caret_col);
        let mut enclosing = Vec::new();
        for sym in &self.outline_symbols {
            sym.breadcrumbs(caret_byte, &mut enclosing);
        }
        let enclosing_sym = enclosing.last().cloned();

        let mut scip_callers = Vec::new();
        if let Some(index) = &self.semantic_index {
            let target_sym = index
                .occurrence_at(&buf.path, line_0, self.caret_col)
                .map(|occ| occ.symbol.clone())
                .or_else(|| {
                    if let Some(enclosing) = &enclosing_sym {
                        index
                            .search_symbols(&enclosing.name, 5)
                            .first()
                            .map(|m| m.id.symbol.clone())
                    } else {
                        None
                    }
                });

            if let Some(sym) = target_sym {
                if !sym.is_empty() {
                    scip_callers = CallerTreeBuilder::build(index, &sym, 4);
                }
            }
        }

        if !scip_callers.is_empty() {
            self.caller_tree = scip_callers;
        } else if let Some(enclosing) = &enclosing_sym {
            let def_line = buf.line_at_byte(enclosing.selection_range.start);
            self.caller_tree = self.find_callers_fallback(&enclosing.name, &buf.path, def_line);
        } else {
            self.caller_tree.clear();
        }
    }

    /// Handles global keyboard shortcuts.
    pub fn handle_shortcuts(&mut self, ctx: &Context) {
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command)
                && i.modifiers.alt
                && i.key_pressed(egui::Key::T)
        }) {
            let new_dark = !self.theme.is_dark;
            self.theme = if new_dark {
                AppTheme::dark()
            } else {
                AppTheme::light()
            };
            ctx.set_visuals(if new_dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            if let Some(buf) = &self.open_buffer {
                let syntax_ref = self.highlighter.find_syntax_for_path(&buf.path);
                self.highlighted_lines = self
                    .highlighter
                    .highlight_document(syntax_ref, &buf.text, new_dark);
            }
        }
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::O)) {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                self.open_repository(&folder);
            }
        }
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command)
                && i.modifiers.shift
                && i.key_pressed(egui::Key::F)
        }) {
            self.sidebar_tab = SidebarTab::Search;
        }
        if ctx
            .input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::Num1))
        {
            self.mode = SourceMode::Normal;
        }
        if ctx
            .input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::Num2))
        {
            self.mode = SourceMode::Critical;
        }
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command)
                && i.modifiers.alt
                && i.key_pressed(egui::Key::B)
        }) {
            if let Some(rev) = self.active_revision {
                self.active_revision = Some(rev.other());
            }
        }
        if ctx.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowDown)) {
            if let Some(changes) = &self.line_changes {
                let rev = self.active_revision.unwrap_or(ReviewRevision::After);
                if let Some(next) = changes.next_change_line(self.caret_line, rev) {
                    if let Some(buf) = &self.open_buffer {
                        let path = buf.path.clone();
                        let offset = buf.byte_offset(next, 0);
                        self.jump_to_location(&path, next, offset);
                    }
                }
            }
        }
        if ctx.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowUp)) {
            if let Some(changes) = &self.line_changes {
                let rev = self.active_revision.unwrap_or(ReviewRevision::After);
                if let Some(prev) = changes.previous_change_line(self.caret_line, rev) {
                    if let Some(buf) = &self.open_buffer {
                        let path = buf.path.clone();
                        let offset = buf.byte_offset(prev, 0);
                        self.jump_to_location(&path, prev, offset);
                    }
                }
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F12)) {
            if let Some(buf) = &self.open_buffer {
                if let Some(idx) = &self.semantic_index {
                    let line_0 = self.caret_line.saturating_sub(1);
                    if let Some(occ) = idx.occurrence_at(&buf.path, line_0, self.caret_col) {
                        if let Some((def_path, def_range)) = idx.find_definition(&occ.symbol) {
                            let p = def_path.to_string();
                            let l = (def_range.start_line + 1) as usize;
                            self.jump_to_location(&p, l, 0);
                        }
                    }
                }
            }
        }
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::O)) {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                self.open_repository(&folder);
            }
        }
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::R)) {
            if let Some(pr) = &self.active_pr {
                let num = pr.number;
                if let Some(root) = &self.repository_root {
                    if let Ok(updated) = GitHubClient::view_pull_request(root, num) {
                        self.active_pr = Some(updated);
                    }
                    if let Ok(diff) = GitHubClient::get_pr_diff(root, num) {
                        self.line_changes = Some(UnifiedDiffParser::parse(&diff));
                        self.pr_changes = UnifiedDiffParser::parse_file_changes(&diff);
                    }
                }
            } else {
                self.refresh_git_changes();
            }
        }
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command)
                && i.modifiers.shift
                && i.key_pressed(egui::Key::F)
        }) {
            self.sidebar_tab = SidebarTab::Search;
            self.show_sidebar = true;
        }
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command)
                && i.modifiers.alt
                && i.key_pressed(egui::Key::S)
        }) {
            self.show_sidebar = !self.show_sidebar;
        }
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command)
                && i.modifiers.alt
                && i.key_pressed(egui::Key::I)
        }) {
            self.show_inspector = !self.show_inspector;
        }
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command)
                && i.modifiers.alt
                && i.key_pressed(egui::Key::Num1)
        }) {
            self.mode = SourceMode::Normal;
        }
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command)
                && i.modifiers.alt
                && i.key_pressed(egui::Key::Num2)
        }) {
            self.mode = SourceMode::Critical;
        }
        if ctx.input(|i| {
            i.key_pressed(egui::Key::Enter)
                && !i.modifiers.ctrl
                && !i.modifiers.command
                && !i.modifiers.alt
                && !i.modifiers.shift
        }) && matches!(self.modal, ActiveModal::None)
        {
            if let Some(buf) = &self.open_buffer {
                if let Some(idx) = &self.semantic_index {
                    let line_0 = self.caret_line.saturating_sub(1);
                    if let Some(occ) = idx.occurrence_at(&buf.path, line_0, self.caret_col) {
                        if let Some((def_path, def_range)) = idx.find_definition(&occ.symbol) {
                            let p = def_path.to_string();
                            let l = (def_range.start_line + 1) as usize;
                            self.jump_to_location(&p, l, 0);
                        }
                    }
                }
            }
        }
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::K)) {
            self.modal = ActiveModal::QuickFilePicker(QuickFilePickerModal::default());
        }
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::L)) {
            self.modal = ActiveModal::GoToLine(GoToLineModal::default());
        }
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::T)) {
            self.modal = ActiveModal::GoToSymbol(GoToSymbolModal::default());
        }
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::P)) {
            self.modal = ActiveModal::PullRequest(PullRequestModal::default());
        }
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command)
                && i.modifiers.alt
                && i.key_pressed(egui::Key::T)
        }) {
            let new_dark = !self.theme.is_dark;
            self.theme = if new_dark {
                AppTheme::dark()
            } else {
                AppTheme::light()
            };
            ctx.set_visuals(if new_dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            if let Some(buf) = &self.open_buffer {
                let syntax_ref = self.highlighter.find_syntax_for_path(&buf.path);
                self.highlighted_lines = self
                    .highlighter
                    .highlight_document(syntax_ref, &buf.text, new_dark);
            }
        }
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command)
                && i.modifiers.alt
                && i.key_pressed(egui::Key::W)
        }) {
            self.session.wrap_lines = !self.session.wrap_lines;
        }
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::D)) {
            if let Some(buf) = &self.open_buffer {
                let offset = buf.byte_offset(self.caret_line, self.caret_col);
                self.bookmarks.toggle(buf, offset);
                if let Some(root) = &self.repository_root {
                    self.bookmarks.save(
                        &root.to_string_lossy(),
                        self.active_pr.as_ref().map(|p| p.number),
                    );
                }
            }
        }
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::OpenBracket)
        }) {
            if let Some(loc) = self.history.back() {
                self.navigate_to_location(&loc.path, 1, loc.byte_offset, false);
            }
        }
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::CloseBracket)
        }) {
            if let Some(loc) = self.history.forward() {
                self.navigate_to_location(&loc.path, 1, loc.byte_offset, false);
            }
        }
        for idx in 0..10 {
            let key = match idx {
                0 => egui::Key::Num0,
                1 => egui::Key::Num1,
                2 => egui::Key::Num2,
                3 => egui::Key::Num3,
                4 => egui::Key::Num4,
                5 => egui::Key::Num5,
                6 => egui::Key::Num6,
                7 => egui::Key::Num7,
                8 => egui::Key::Num8,
                _ => egui::Key::Num9,
            };
            if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(key)) {
                if let Some(b) = self.bookmarks.get_by_index(idx) {
                    let path = b.path.clone();
                    let line = b.line;
                    let offset = b.byte_offset;
                    self.jump_to_location(&path, line, offset);
                }
            }
        }
    }
}

impl eframe::App for RadiologyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.search_rx {
            while let Ok(batch) = rx.try_recv() {
                self.search_matches.extend(batch);
            }
            if let Err(std::sync::mpsc::TryRecvError::Disconnected) = rx.try_recv() {
                self.is_searching = false;
                self.search_rx = None;
            } else if self.is_searching {
                ctx.request_repaint();
            }
        }

        self.handle_shortcuts(ctx);

        let mut close_modal = false;
        match &mut self.modal {
            ActiveModal::None => {}
            ActiveModal::QuickFilePicker(modal) => {
                if let Some(action) = modal.show(ctx, &self.theme, &self.all_files) {
                    match action {
                        ModalAction::OpenFile(path) => {
                            self.open_file(&path);
                            close_modal = true;
                        }
                        ModalAction::Close => {
                            close_modal = true;
                        }
                        _ => {}
                    }
                }
            }
            ActiveModal::GoToLine(modal) => {
                let max_lines = self.open_buffer.as_ref().map_or(1, |b| b.line_count());
                if let Some(action) = modal.show(ctx, &self.theme, max_lines) {
                    match action {
                        ModalAction::JumpToLine(line) => {
                            if let Some(buf) = &self.open_buffer {
                                let offset = buf.byte_offset(line, 0);
                                let p = buf.path.clone();
                                self.jump_to_location(&p, line, offset);
                            }
                            close_modal = true;
                        }
                        ModalAction::Close => {
                            close_modal = true;
                        }
                        _ => {}
                    }
                }
            }
            ActiveModal::GoToSymbol(modal) => {
                if let Some(action) = modal.show(
                    ctx,
                    &self.theme,
                    self.semantic_index.as_ref(),
                    &self.outline_symbols,
                ) {
                    match action {
                        ModalAction::JumpToLocation {
                            path,
                            line,
                            byte_offset,
                        } => {
                            self.jump_to_location(&path, line, byte_offset);
                            close_modal = true;
                        }
                        ModalAction::JumpToSymbol(byte) => {
                            if let Some(buf) = &self.open_buffer {
                                let line = buf.line_at_byte(byte);
                                let p = buf.path.clone();
                                self.jump_to_location(&p, line, byte);
                            }
                            close_modal = true;
                        }
                        ModalAction::Close => {
                            close_modal = true;
                        }
                        _ => {}
                    }
                }
            }
            ActiveModal::PullRequest(modal) => {
                if let Some(action) = modal.show(ctx, &self.theme) {
                    match action {
                        ModalAction::OpenPullRequest(num) => {
                            if let Some(root) = &self.repository_root {
                                if let Ok(pr) = GitHubClient::view_pull_request(root, num) {
                                    self.active_pr = Some(pr);
                                    if let Ok(diff_text) = GitHubClient::get_pr_diff(root, num) {
                                        self.line_changes =
                                            Some(UnifiedDiffParser::parse(&diff_text));
                                        self.pr_changes =
                                            UnifiedDiffParser::parse_file_changes(&diff_text);
                                        if !self.pr_changes.is_empty() {
                                            self.sidebar_tab = SidebarTab::Changes;
                                        }
                                    }
                                }
                            }
                            close_modal = true;
                        }
                        ModalAction::Close => {
                            close_modal = true;
                        }
                        _ => {}
                    }
                }
            }
        }
        if close_modal {
            self.modal = ActiveModal::None;
        }

        TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            if let Some(action) = Toolbar::show(
                ui,
                &self.theme,
                self.show_sidebar,
                self.show_inspector,
                self.repository_name.as_deref(),
                self.branch_name.as_deref(),
                &self.session.recent_repositories,
                self.active_pr.as_ref(),
                self.active_revision,
                self.mode,
                &self.session.source_filters,
                self.session.wrap_lines,
                &self.history,
            ) {
                match action {
                    ToolbarAction::ToggleSidebar => {
                        self.show_sidebar = !self.show_sidebar;
                    }
                    ToolbarAction::ToggleInspector => {
                        self.show_inspector = !self.show_inspector;
                    }
                    ToolbarAction::OpenRecentRepo(p) => {
                        self.open_repository(&PathBuf::from(p));
                    }
                    ToolbarAction::CopyRepoPath => {
                        if let Some(root) = &self.repository_root {
                            ui.output_mut(|o| o.copied_text = root.to_string_lossy().to_string());
                        }
                    }
                    ToolbarAction::RefreshPR => {
                        if let Some(pr) = &self.active_pr {
                            let num = pr.number;
                            if let Some(root) = &self.repository_root {
                                if let Ok(updated) = GitHubClient::view_pull_request(root, num) {
                                    self.active_pr = Some(updated);
                                }
                                if let Ok(diff) = GitHubClient::get_pr_diff(root, num) {
                                    self.line_changes = Some(UnifiedDiffParser::parse(&diff));
                                    self.pr_changes = UnifiedDiffParser::parse_file_changes(&diff);
                                }
                            }
                        } else {
                            self.refresh_git_changes();
                        }
                    }
                    ToolbarAction::OpenSearch => {
                        self.sidebar_tab = SidebarTab::Search;
                        self.show_sidebar = true;
                    }
                    ToolbarAction::OpenRepositoryDialog => {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            self.open_repository(&folder);
                        }
                    }
                    ToolbarAction::NavigateBack => {
                        if let Some(loc) = self.history.back() {
                            self.navigate_to_location(&loc.path, 1, loc.byte_offset, false);
                        }
                    }
                    ToolbarAction::NavigateForward => {
                        if let Some(loc) = self.history.forward() {
                            self.navigate_to_location(&loc.path, 1, loc.byte_offset, false);
                        }
                    }
                    ToolbarAction::PrevChangeBlock => {
                        if let Some(diff) = &self.line_changes {
                            let anchors = diff.change_anchors(
                                self.active_revision.unwrap_or(ReviewRevision::After),
                            );
                            if let Some(&prev) =
                                anchors.iter().rev().find(|&&a| a < self.caret_line)
                            {
                                self.caret_line = prev;
                            }
                        }
                    }
                    ToolbarAction::NextChangeBlock => {
                        if let Some(diff) = &self.line_changes {
                            let anchors = diff.change_anchors(
                                self.active_revision.unwrap_or(ReviewRevision::After),
                            );
                            if let Some(&next) = anchors.iter().find(|&&a| a > self.caret_line) {
                                self.caret_line = next;
                            }
                        }
                    }
                    ToolbarAction::ToggleBookmark => {
                        if let Some(buf) = &self.open_buffer {
                            let offset = buf.byte_offset(self.caret_line, self.caret_col);
                            self.bookmarks.toggle(buf, offset);
                        }
                    }
                    ToolbarAction::OpenQuickFilePicker => {
                        self.modal = ActiveModal::QuickFilePicker(QuickFilePickerModal::default());
                    }
                    ToolbarAction::OpenGoToLine => {
                        self.modal = ActiveModal::GoToLine(GoToLineModal::default());
                    }
                    ToolbarAction::OpenGoToSymbol => {
                        self.modal = ActiveModal::GoToSymbol(GoToSymbolModal::default());
                    }
                    ToolbarAction::OpenPRDialog => {
                        self.modal = ActiveModal::PullRequest(PullRequestModal::default());
                    }
                    ToolbarAction::ExitPRReview => {
                        self.active_pr = None;
                        self.pr_changes.clear();
                        self.line_changes = None;
                        self.active_revision = None;
                    }
                    ToolbarAction::ToggleMode(m) => {
                        self.mode = m;
                    }
                    ToolbarAction::ToggleRevision(r) => {
                        self.active_revision = Some(r);
                        if let Some(p) = self.selected_path.clone() {
                            self.open_file(&p);
                        }
                    }
                    ToolbarAction::ToggleSourceFilesOnly => {
                        self.session.source_filters.source_files_only =
                            !self.session.source_filters.source_files_only;
                        self.refresh_file_tree();
                    }
                    ToolbarAction::ToggleHideTests => {
                        self.session.source_filters.hides_tests =
                            !self.session.source_filters.hides_tests;
                        self.refresh_file_tree();
                    }
                    ToolbarAction::ToggleTheme => {
                        let new_dark = !self.theme.is_dark;
                        self.theme = if new_dark {
                            AppTheme::dark()
                        } else {
                            AppTheme::light()
                        };
                        ctx.set_visuals(if new_dark {
                            egui::Visuals::dark()
                        } else {
                            egui::Visuals::light()
                        });
                        if let Some(buf) = &self.open_buffer {
                            let syntax_ref = self.highlighter.find_syntax_for_path(&buf.path);
                            self.highlighted_lines = self
                                .highlighter
                                .highlight_document(syntax_ref, &buf.text, new_dark);
                        }
                    }
                    ToolbarAction::ToggleWrapLines => {
                        self.session.wrap_lines = !self.session.wrap_lines;
                    }
                }
            }
        });

        if self.show_sidebar {
            SidePanel::left("left_sidebar")
                .resizable(true)
                .default_width(260.0)
                .min_width(180.0)
                .max_width(500.0)
                .show(ctx, |ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    let prev_tab = self.sidebar_tab;
                    let visible_count = self
                        .session
                        .source_filters
                        .filter_files(&self.all_files)
                        .len();
                    let total_count = self.all_files.len();
                    if let Some(action) = Sidebar::show(
                        ui,
                        &self.theme,
                        &mut self.sidebar_tab,
                        &self.file_tree,
                        &mut self.expanded_dirs,
                        self.selected_path.as_deref(),
                        self.active_pr.as_ref(),
                        &self.pr_changes,
                        &mut self.search_options,
                        &self.search_matches,
                        self.is_searching,
                        visible_count,
                        total_count,
                        self.branch_name.as_deref(),
                    ) {
                        match action {
                            SidebarAction::OpenFile(p) => {
                                self.open_file(&p);
                            }
                            SidebarAction::OpenSearchMatch(m) => {
                                self.jump_to_location(&m.path, m.line, m.byte_offset);
                            }
                            SidebarAction::StartSearch => {
                                if let Some(root) = self.repository_root.clone() {
                                    self.search_cancel
                                        .store(true, std::sync::atomic::Ordering::Relaxed);
                                    let cancel = Arc::new(AtomicBool::new(false));
                                    self.search_cancel = cancel.clone();
                                    self.is_searching = true;
                                    self.search_matches.clear();
                                    let opts = self.search_options.clone();

                                    let (tx, rx) = std::sync::mpsc::channel();
                                    self.search_rx = Some(rx);

                                    std::thread::spawn(move || {
                                        if let Ok(rt) =
                                            tokio::runtime::Builder::new_current_thread()
                                                .enable_all()
                                                .build()
                                        {
                                            rt.block_on(async move {
                                                let _ = SearchRunner::search(
                                                    &root,
                                                    &opts,
                                                    cancel,
                                                    |batch| {
                                                        let _ = tx.send(batch);
                                                    },
                                                )
                                                .await;
                                            });
                                        }
                                    });
                                }
                            }
                            SidebarAction::ClearSearch => {
                                self.search_matches.clear();
                            }
                        }
                    }

                    if prev_tab != SidebarTab::Changes && self.sidebar_tab == SidebarTab::Changes {
                        self.refresh_git_changes();
                    }
                });
        }

        if self.show_inspector {
            SidePanel::right("context_inspector")
                .resizable(true)
                .default_width(300.0)
                .min_width(200.0)
                .max_width(600.0)
                .show(ctx, |ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    let current_sym_info = self.open_buffer.as_ref().and_then(|buf| {
                        self.semantic_index.as_ref().and_then(|idx| {
                            let line_0 = self.caret_line.saturating_sub(1);
                            idx.occurrence_at(&buf.path, line_0, self.caret_col)
                                .and_then(|occ| idx.symbol_info(&occ.symbol))
                        })
                    });

                    let def_loc = current_sym_info.and_then(|s| {
                        self.semantic_index.as_ref().and_then(|idx| {
                            idx.find_definition(&s.symbol)
                                .map(|(p, r)| (p, (r.start_line + 1) as usize))
                        })
                    });

                    let (enclosing_sym, enclosing_line) = if let Some(buf) = &self.open_buffer {
                        let caret_byte = buf.byte_offset(self.caret_line, self.caret_col);
                        let mut enclosing = Vec::new();
                        for sym in &self.outline_symbols {
                            sym.breadcrumbs(caret_byte, &mut enclosing);
                        }
                        let sym = enclosing.last().cloned();
                        let line = sym
                            .as_ref()
                            .map(|s| buf.line_at_byte(s.selection_range.start))
                            .unwrap_or(self.caret_line);
                        (sym, line)
                    } else {
                        (None, 1)
                    };
                    let current_file_path = self.open_buffer.as_ref().map(|b| b.path.as_str());

                    if let Some(action) = ContextInspector::show(
                        ui,
                        &self.theme,
                        &mut self.inspector_tab,
                        current_sym_info,
                        def_loc,
                        enclosing_sym.as_ref(),
                        enclosing_line,
                        current_file_path,
                        &self.caller_tree,
                        &self.outline_symbols,
                        &self.bookmarks,
                    ) {
                        match action {
                            InspectorAction::JumpToLocation {
                                path,
                                line,
                                byte_offset,
                            } => {
                                self.jump_to_location(&path, line, byte_offset);
                            }
                            InspectorAction::JumpToSymbol(byte) => {
                                if let Some(buf) = &self.open_buffer {
                                    let line = buf.line_at_byte(byte);
                                    let p = buf.path.clone();
                                    self.jump_to_location(&p, line, byte);
                                }
                            }
                            InspectorAction::RemoveBookmark(id) => {
                                self.bookmarks.remove(id);
                            }
                            InspectorAction::MoveBookmarkUp(idx) => {
                                self.bookmarks.move_up(idx);
                            }
                            InspectorAction::MoveBookmarkDown(idx) => {
                                self.bookmarks.move_down(idx);
                            }
                        }
                    }
                });
        }

        let central_frame = egui::Frame::none()
            .fill(self.theme.bg_base)
            .inner_margin(egui::Margin::symmetric(0.0, 0.0));

        CentralPanel::default()
            .frame(central_frame)
            .show(ctx, |ui| {
                ui.set_clip_rect(ui.max_rect());
                if let Some(buf) = &self.open_buffer {
                    let mut enclosing = Vec::new();
                    let caret_byte = buf.byte_offset(self.caret_line, self.caret_col);
                    for sym in &self.outline_symbols {
                        sym.breadcrumbs(caret_byte, &mut enclosing);
                    }

                    if let Some(action) = SourceView::show(
                        ui,
                        &self.theme,
                        self.mode,
                        buf,
                        &self.highlighted_lines,
                        &mut self.caret_line,
                        &mut self.caret_col,
                        &self.fold_ranges,
                        &mut self.fold_store,
                        self.line_changes.as_ref(),
                        &mut self.pin_store,
                        self.session.wrap_lines,
                        &enclosing,
                        &mut self.scroll_to_line,
                        self.semantic_index.is_some(),
                    ) {
                        match action {
                            SourceViewAction::CaretMoved {
                                line,
                                column,
                                byte_offset,
                            } => {
                                self.caret_line = line;
                                self.caret_col = column;
                                self.history
                                    .update_current(buf.location_at_byte(byte_offset));
                                self.refresh_symbol_context();
                            }
                            SourceViewAction::JumpToDefinition => {
                                if let Some(idx) = &self.semantic_index {
                                    let line_0 = self.caret_line.saturating_sub(1);
                                    if let Some(occ) =
                                        idx.occurrence_at(&buf.path, line_0, self.caret_col)
                                    {
                                        if let Some((def_path, def_range)) =
                                            idx.find_definition(&occ.symbol)
                                        {
                                            let p = def_path.to_string();
                                            let l = (def_range.start_line + 1) as usize;
                                            self.jump_to_location(&p, l, 0);
                                        }
                                    }
                                }
                            }
                            SourceViewAction::ToggleFold(_) => {}
                            SourceViewAction::CollapseFunctions => {
                                self.fold_store
                                    .collapse_all_functions(&self.outline_symbols);
                            }
                            SourceViewAction::NextChangeBlock => {
                                if let Some(diff) = &self.line_changes {
                                    let anchors = diff.change_anchors(
                                        self.active_revision.unwrap_or(ReviewRevision::After),
                                    );
                                    if let Some(&next) =
                                        anchors.iter().find(|&&a| a > self.caret_line)
                                    {
                                        self.caret_line = next;
                                    }
                                }
                            }
                            SourceViewAction::PrevChangeBlock => {
                                if let Some(diff) = &self.line_changes {
                                    let anchors = diff.change_anchors(
                                        self.active_revision.unwrap_or(ReviewRevision::After),
                                    );
                                    if let Some(&prev) =
                                        anchors.iter().rev().find(|&&a| a < self.caret_line)
                                    {
                                        self.caret_line = prev;
                                    }
                                }
                            }
                            SourceViewAction::ToggleRevision(r) => {
                                self.active_revision = Some(r);
                            }
                            SourceViewAction::JumpToSymbol(byte) => {
                                if let Some(buf) = &self.open_buffer {
                                    let line = buf.line_at_byte(byte);
                                    let p = buf.path.clone();
                                    self.jump_to_location(&p, line, byte);
                                }
                            }
                            SourceViewAction::PinOccurrence { symbol, ranges } => {
                                self.pin_store.toggle_pin(&symbol, ranges);
                            }
                            SourceViewAction::OpenGoToLine => {
                                self.modal = ActiveModal::GoToLine(GoToLineModal::default());
                            }
                        }
                    }
                } else if self.repository_root.is_some() {
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("No File Selected");
                            ui.add_space(8.0);
                            ui.label(format!(
                                "Browsing repository: {}",
                                self.repository_name.as_deref().unwrap_or("")
                            ));
                            ui.add_space(8.0);
                            ui.colored_label(
                                self.theme.text_muted,
                                "Select a file from the sidebar (or press Ctrl+K to search files).",
                            );
                        });
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(egui::RichText::new("Read the Code").size(24.0).strong());
                            ui.add_space(8.0);
                            ui.label("Open a repository to browse, search, and follow the source.");
                            ui.add_space(16.0);
                            let open_clicked = ui
                                .horizontal(|ui| {
                                    Icon::Folder.ui(ui, 18.0, self.theme.accent);
                                    ui.button(egui::RichText::new("Open Repository…").size(15.0))
                                        .clicked()
                                })
                                .inner;
                            if open_clicked {
                                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                    self.open_repository(&folder);
                                }
                            }
                            if !self.session.recent_repositories.is_empty() {
                                ui.add_space(24.0);
                                ui.label(egui::RichText::new("Recent Repositories").strong());
                                ui.add_space(8.0);
                                for recent in &self.session.recent_repositories.clone() {
                                    if ui.link(recent).clicked() {
                                        self.open_repository(&PathBuf::from(recent));
                                    }
                                }
                            }
                            ui.add_space(24.0);
                            ui.colored_label(
                                self.theme.text_muted,
                                "Ctrl+K  Go to file     Ctrl+Shift+F  Search     Ctrl+[  Go back",
                            );
                        });
                    });
                }
            });
    }
}
