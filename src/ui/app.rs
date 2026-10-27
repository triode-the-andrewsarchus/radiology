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
use crate::syntax::highlighter::SyntaxHighlighter;
use crate::syntax::outline::{DocumentSymbol, SymbolExtractor};
use crate::theme::AppTheme;
use crate::ui::context_inspector::{ContextInspector, InspectorAction, InspectorTab};
use crate::ui::modals::{
    GoToLineModal, GoToSymbolModal, ModalAction, PullRequestModal, QuickFilePickerModal,
};
use crate::ui::sidebar::{Sidebar, SidebarAction, SidebarTab};
use crate::ui::source_view::{SourceView, SourceViewAction};
use crate::ui::toolbar::{Toolbar, ToolbarAction};

pub enum ActiveModal {
    None,
    QuickFilePicker(QuickFilePickerModal),
    GoToLine(GoToLineModal),
    GoToSymbol(GoToSymbolModal),
    PullRequest(PullRequestModal),
}

pub struct RadiologyApp {
    pub theme: AppTheme,
    pub session: SessionStore,
    pub repository_root: Option<PathBuf>,
    pub repository_name: Option<String>,
    pub branch_name: Option<String>,
    pub all_files: Vec<RepositoryFile>,
    pub file_tree: Vec<FileNode>,
    pub expanded_dirs: HashSet<String>,
    pub selected_path: Option<String>,
    pub open_buffer: Option<SourceBuffer>,
    pub caret_line: usize,
    pub caret_col: usize,
    pub history: NavigationHistory,
    pub highlighter: SyntaxHighlighter,
    pub fold_ranges: Vec<FoldRange>,
    pub fold_store: FoldStore,
    pub outline_symbols: Vec<DocumentSymbol>,
    pub pin_store: OccurrencePinStore,
    pub semantic_index: Option<SemanticIndex>,
    pub caller_tree: Vec<ReferenceCallerNode>,
    pub active_pr: Option<PullRequestSummary>,
    pub pr_changes: Vec<FileChange>,
    pub active_revision: Option<ReviewRevision>,
    pub line_changes: Option<FileLineChanges>,
    pub mode: SourceMode,
    pub sidebar_tab: SidebarTab,
    pub inspector_tab: InspectorTab,
    pub search_options: SearchOptions,
    pub search_matches: Vec<SearchMatch>,
    pub is_searching: bool,
    pub search_cancel: Arc<AtomicBool>,
    pub bookmarks: BookmarkStore,
    pub modal: ActiveModal,
}

impl RadiologyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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
            bookmarks: BookmarkStore::new(),
            modal: ActiveModal::None,
        };

        // Automatically open the current directory or last opened repository
        let initial_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        app.open_repository(&initial_dir);

        app
    }

    pub fn open_repository(&mut self, path: &Path) {
        let root = RepositoryDiscovery::find_root(path);
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repository")
            .to_string();

        self.repository_root = Some(root.clone());
        self.repository_name = Some(name.clone());
        self.session.record_repository(root.to_string_lossy());

        // Resolve Git branch
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

        // Discover files
        if let Ok(files) = RepositoryDiscovery::discover_files(&root) {
            self.all_files = files;
            self.refresh_file_tree();
        }

        // Load persistent bookmarks for repository
        self.bookmarks = BookmarkStore::load(&root.to_string_lossy(), None);

        // Try loading SCIP index if present (.scip or index.scip)
        for scip_candidate in [root.join("index.scip"), root.join(".scip")] {
            if scip_candidate.exists() {
                if let Ok(idx) = SemanticIndex::load_file(&scip_candidate) {
                    self.semantic_index = Some(idx);
                    break;
                }
            }
        }

        // Open first file if available (e.g. README.md, main.rs, or first source file)
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
    }

    pub fn refresh_file_tree(&mut self) {
        let filtered = self.session.source_filters.filter_files(&self.all_files);
        self.file_tree = FileNode::build_tree(&filtered);
    }

    pub fn open_file(&mut self, rel_path: &str) {
        let Some(root) = &self.repository_root else {
            return;
        };
        let full_path = root.join(rel_path);

        if let Ok(bytes) = std::fs::read(&full_path) {
            if let Ok(buffer) = SourceBuffer::from_bytes(rel_path, bytes, self.active_revision) {
                self.selected_path = Some(rel_path.to_string());
                self.caret_line = 1;
                self.caret_col = 0;

                // Extract syntax outline & folds
                self.outline_symbols = SymbolExtractor::extract(&buffer);
                self.fold_ranges = FoldDetector::detect_folds(&buffer);
                self.fold_store.unfold_all();

                self.history.visit(buffer.location_at_byte(0));
                self.open_buffer = Some(buffer);

                self.refresh_symbol_context();
            }
        }
    }

    pub fn jump_to_location(&mut self, rel_path: &str, line: usize, byte_offset: usize) {
        if self.selected_path.as_deref() != Some(rel_path) {
            self.open_file(rel_path);
        }
        if let Some(buf) = &self.open_buffer {
            self.caret_line = line.clamp(1, buf.line_count());
            self.caret_col = 0;
            let offset = if byte_offset > 0 {
                byte_offset
            } else {
                buf.byte_offset(self.caret_line, 0)
            };
            self.history.visit(buf.location_at_byte(offset));
            self.refresh_symbol_context();
        }
    }

    pub fn refresh_symbol_context(&mut self) {
        let Some(buf) = &self.open_buffer else {
            return;
        };
        let Some(index) = &self.semantic_index else {
            return;
        };

        // Find occurrence at caret
        let line_0 = self.caret_line.saturating_sub(1);
        if let Some(occ) = index.occurrence_at(&buf.path, line_0, self.caret_col) {
            if !occ.symbol.is_empty() {
                self.caller_tree = CallerTreeBuilder::build(index, &occ.symbol, 4);
            }
        }
    }

    pub fn handle_shortcuts(&mut self, ctx: &Context) {
        // Ctrl+K -> Quick File Picker
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::K)) {
            self.modal = ActiveModal::QuickFilePicker(QuickFilePickerModal::default());
        }
        // Ctrl+L -> Go to Line
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::L)) {
            self.modal = ActiveModal::GoToLine(GoToLineModal::default());
        }
        // Ctrl+T -> Go to Symbol
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::T)) {
            self.modal = ActiveModal::GoToSymbol(GoToSymbolModal::default());
        }
        // Ctrl+P -> PR Picker
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::P)) {
            self.modal = ActiveModal::PullRequest(PullRequestModal::default());
        }
        // Ctrl+D -> Toggle Bookmark
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
        // Ctrl+[ -> Back
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::OpenBracket)
        }) {
            if let Some(loc) = self.history.back() {
                self.jump_to_location(&loc.path, 1, loc.byte_offset);
            }
        }
        // Ctrl+] -> Forward
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::CloseBracket)
        }) {
            if let Some(loc) = self.history.forward() {
                self.jump_to_location(&loc.path, 1, loc.byte_offset);
            }
        }
        // Ctrl+0..9 -> Jump to Bookmark index
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
        self.handle_shortcuts(ctx);

        // Modals
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
                if let Some(action) = modal.show(ctx, &self.theme, &self.outline_symbols) {
                    match action {
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

        // Top Toolbar
        TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            if let Some(action) = Toolbar::show(
                ui,
                &self.theme,
                self.repository_name.as_deref(),
                self.branch_name.as_deref(),
                self.active_pr.as_ref(),
                self.active_revision,
                self.mode,
                &self.session.source_filters,
                &self.history,
            ) {
                match action {
                    ToolbarAction::OpenRepositoryDialog => {
                        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
                        self.open_repository(&home);
                    }
                    ToolbarAction::NavigateBack => {
                        if let Some(loc) = self.history.back() {
                            self.jump_to_location(&loc.path, 1, loc.byte_offset);
                        }
                    }
                    ToolbarAction::NavigateForward => {
                        if let Some(loc) = self.history.forward() {
                            self.jump_to_location(&loc.path, 1, loc.byte_offset);
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
                }
            }
        });

        // Left Sidebar
        SidePanel::left("left_sidebar")
            .resizable(true)
            .default_width(260.0)
            .min_width(180.0)
            .max_width(500.0)
            .show(ctx, |ui| {
                if let Some(action) = Sidebar::show(
                    ui,
                    &self.theme,
                    &mut self.sidebar_tab,
                    &self.file_tree,
                    &mut self.expanded_dirs,
                    self.selected_path.as_deref(),
                    &self.pr_changes,
                    &mut self.search_options,
                    &self.search_matches,
                    self.is_searching,
                ) {
                    match action {
                        SidebarAction::OpenFile(p) => {
                            self.open_file(&p);
                        }
                        SidebarAction::OpenSearchMatch(m) => {
                            self.jump_to_location(&m.path, m.line, m.byte_offset);
                        }
                        SidebarAction::StartSearch => {
                            // Run search in background
                            if let Some(root) = self.repository_root.clone() {
                                self.is_searching = true;
                                self.search_matches.clear();
                                let opts = self.search_options.clone();
                                let cancel = self.search_cancel.clone();

                                // Synchronous batch for UI
                                let mut matches = Vec::new();
                                let _ = tokio::runtime::Builder::new_current_thread()
                                    .enable_all()
                                    .build()
                                    .map(|rt| {
                                        rt.block_on(async {
                                            let _ = SearchRunner::search(
                                                &root,
                                                &opts,
                                                cancel,
                                                |batch| {
                                                    matches.extend(batch);
                                                },
                                            )
                                            .await;
                                        });
                                    });
                                self.search_matches = matches;
                                self.is_searching = false;
                            }
                        }
                        SidebarAction::ClearSearch => {
                            self.search_matches.clear();
                        }
                    }
                }
            });

        // Right Sidebar (Context Inspector)
        SidePanel::right("context_inspector")
            .resizable(true)
            .default_width(300.0)
            .min_width(200.0)
            .max_width(600.0)
            .show(ctx, |ui| {
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

                if let Some(action) = ContextInspector::show(
                    ui,
                    &self.theme,
                    &mut self.inspector_tab,
                    current_sym_info,
                    def_loc,
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

        // Center Panel (Source View)
        CentralPanel::default().show(ctx, |ui| {
            if let Some(buf) = &self.open_buffer {
                let syntax_ref = self.highlighter.find_syntax_for_path(&buf.path);

                let mut enclosing = Vec::new();
                let caret_byte = buf.byte_offset(self.caret_line, self.caret_col);
                for sym in &self.outline_symbols {
                    sym.breadcrumbs(caret_byte, &mut enclosing);
                }

                if let Some(action) = SourceView::show(
                    ui,
                    &self.theme,
                    &self.highlighter,
                    syntax_ref,
                    buf,
                    &mut self.caret_line,
                    &mut self.caret_col,
                    &self.fold_ranges,
                    &mut self.fold_store,
                    self.line_changes.as_ref(),
                    &mut self.pin_store,
                    &enclosing,
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
                        SourceViewAction::NextChangeBlock => {
                            if let Some(diff) = &self.line_changes {
                                let anchors = diff.change_anchors(
                                    self.active_revision.unwrap_or(ReviewRevision::After),
                                );
                                if let Some(&next) = anchors.iter().find(|&&a| a > self.caret_line)
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
                        SourceViewAction::PinOccurrence { symbol, ranges } => {
                            self.pin_store.toggle_pin(&symbol, ranges);
                        }
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Open a repository or select a file to view source.");
                });
            }
        });
    }
}
