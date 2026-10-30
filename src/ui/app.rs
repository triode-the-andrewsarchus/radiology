//! Main application model and EGUI event loop lifecycle.

mod actions;
mod bookmarks;
mod critical_marks;
mod indexing;
mod outline;
mod references;
mod reviews;
mod source;
#[cfg(test)]
mod tests;

use egui::{CentralPanel, Context, SidePanel, TopBottomPanel};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::bookmarks::store::{BookmarkScope, BookmarkStore};
use crate::critical::lens::SourceMode;
use crate::pull_request::change::FileChange;
use crate::pull_request::diff::{FileLineChanges, UnifiedDiffParser};
use crate::pull_request::github::{GitHubClient, PullRequestSummary};
use crate::pull_request::review::ReviewSnapshot;
use crate::repository::discovery::RepositoryDiscovery;
use crate::repository::file_node::{FileNode, RepositoryFile};
use crate::search::{RepositorySearch, SearchOptions};
use crate::semantic::index::{SemanticIndex, SemanticSymbolID};
use crate::semantic::references::{ReferenceContext, ReferenceTarget};
use crate::session::store::SessionStore;
use crate::source::buffer::SourceBuffer;
use crate::source::history::NavigationHistory;
use crate::source::location::{ReviewRevision, SourceLocation};
use crate::source::selection::OccurrencePinStore;
use crate::syntax::folds::{FoldRange, FoldStore};
use crate::syntax::highlighter::{HighlightedSpan, SyntaxHighlighter};
use crate::syntax::outline::DocumentSymbol;
use crate::theme::AppTheme;
use crate::ui::context_inspector::{ContextInspector, InspectorAction, InspectorContent};
use crate::ui::icons::Icon;
use crate::ui::modals::{
    GoToLineModal, GoToSymbolModal, ModalAction, PullRequestModal, QuickFilePickerModal,
};
use crate::ui::sidebar::{Sidebar, SidebarAction, SidebarTab};
use crate::ui::source_view::{SourceView, SourceViewAction, SourceViewContent};
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
    pub open_buffer: Option<Arc<SourceBuffer>>,
    /// 1-based line of the active caret.
    pub caret_line: usize,
    /// 0-based column of the active caret.
    pub caret_col: usize,
    /// Local-find controls and document-scoped results.
    pub source_find: crate::ui::source_find::SourceFindPanel,
    /// Measured source geometry and original-byte selection.
    pub source_viewport: crate::ui::source_view::SourceViewport,
    /// Linear navigation history.
    pub history: NavigationHistory,
    /// Syntax highlighting engine.
    pub highlighter: Arc<SyntaxHighlighter>,
    /// Cached highlighted lines for active source buffer.
    pub highlighted_lines: Vec<Vec<HighlightedSpan>>,
    /// Foldable line ranges in the open file.
    pub fold_ranges: Vec<FoldRange>,
    /// Recoverable structural query failure for the displayed source.
    pub fold_error: Option<String>,
    /// Fold store tracking collapsed line numbers.
    pub fold_store: FoldStore,
    /// Tree-sitter document outline symbols.
    pub outline_symbols: Vec<DocumentSymbol>,
    /// Inline-test visibility independent of manual folds.
    pub test_code: crate::source::test_code::TestCodePresentation,
    /// Pinned token occurrence groups.
    pub pin_store: OccurrencePinStore,
    /// SCIP semantic index if loaded.
    pub semantic_index: Option<Arc<SemanticIndex>>,
    call_sites: Arc<crate::syntax::calls::CallSites>,
    syntactic_occurrences: crate::syntax::occurrences::SyntacticOccurrences,
    critical_lens: crate::critical::service::CriticalLens,
    critical_annotations: crate::critical::annotations::CriticalAnnotations,
    critical_graph_inputs: Option<Arc<crate::critical::worker::GraphInputs>>,
    critical_marks: crate::critical::marks::MarkSession,
    /// Configurable tool lookup context shared by probes and generation.
    pub index_tools: crate::semantic::indexing::tools::ToolDirectory,
    index_importer: crate::semantic::loader::IndexImporter,
    index_generator: crate::semantic::indexing::service::IndexGenerator,
    generation_report: Option<crate::semantic::indexing::service::GenerationReport>,
    index_plan: crate::semantic::indexing::IndexPlan,
    index_plan_job: crate::background::LatestRead<crate::semantic::indexing::IndexPlan>,
    index_projects_open: bool,
    tool_probe_job: crate::background::LatestRead<
        std::collections::BTreeMap<
            crate::semantic::indexing::Indexer,
            crate::semantic::indexing::tools::ToolAvailability,
        >,
    >,
    tool_availability: std::collections::BTreeMap<
        crate::semantic::indexing::Indexer,
        crate::semantic::indexing::tools::ToolAvailability,
    >,
    reference_context: ReferenceContext,
    semantic_inputs: Option<Arc<crate::source::preview::SourceQueryInputs>>,
    definition_preview: crate::source::preview::SourcePreviewLoader,
    under_caret: Option<crate::semantic::context::UnderCaretContext>,
    under_caret_job: crate::background::LatestRead<
        Result<Option<crate::semantic::context::UnderCaretContext>, String>,
    >,
    under_caret_error: Option<String>,
    definition_identity: Option<(SemanticSymbolID, std::ops::Range<usize>)>,
    definition_expanded: bool,
    reference_viewport: crate::ui::reference_list::ReferenceViewport,
    bookmark_list: crate::ui::bookmark_list::BookmarkList,
    outline_list: crate::ui::outline_list::OutlineList,
    /// Active pull request summary during PR review.
    pub active_pr: Option<PullRequestSummary>,
    /// Immutable review objects and revision-specific source routing.
    pub review: Option<Arc<ReviewSnapshot>>,
    /// Review preparation currently running outside the UI thread.
    review_rx: Option<std::sync::mpsc::Receiver<Result<ReviewSnapshot, String>>>,
    /// Pull request being prepared while the current reader remains usable.
    pub pending_pr: Option<usize>,
    /// Explicit launch file to select after asynchronous review preparation.
    pending_open_file: Option<String>,
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
    /// Search options for repository ripgrep queries.
    pub search_options: SearchOptions,
    /// Cancellable search state with query errors and bounded-result status.
    pub repository_search: RepositorySearch,
    /// Repository and PR bookmarks.
    pub bookmarks: BookmarkStore,
    /// Active modal dialog sheet.
    pub modal: ActiveModal,
    source_loader: crate::source::loader::SourceLoader,
    pending_navigation: Option<source::PendingNavigation>,
    /// Target line to scroll and center into view in the code editor.
    pub scroll_to_line: Option<usize>,
    /// Most recent recoverable error shown in the reader.
    pub error: Option<String>,
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
        let app = Self::from_session(SessionStore::load());
        cc.egui_ctx.set_visuals(if app.theme.is_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
        app
    }

    /// Creates an idle reader model without a window or startup side effects.
    ///
    /// Hosts choose when to restore a session or open an explicit repository.
    #[must_use]
    pub fn from_session(session: SessionStore) -> Self {
        let index_tools = crate::semantic::indexing::tools::ToolDirectory::from_environment(
            session.config_directory().map(|path| path.join("tools")),
        );
        let critical_marks =
            crate::critical::marks::MarkSession::new(session.config_directory().map(|path| {
                crate::critical::overrides::OverrideStore::new(path.join("critical-marks"))
            }));
        Self {
            theme: if session.dark_mode {
                AppTheme::dark()
            } else {
                AppTheme::light()
            },
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
            source_find: Default::default(),
            source_viewport: Default::default(),
            history: NavigationHistory::new(),
            highlighter: Arc::new(SyntaxHighlighter::new()),
            highlighted_lines: Vec::new(),
            fold_ranges: Vec::new(),
            fold_error: None,
            fold_store: FoldStore::new(),
            outline_symbols: Vec::new(),
            test_code: Default::default(),
            pin_store: OccurrencePinStore::new(),
            semantic_index: None,
            call_sites: Default::default(),
            syntactic_occurrences: Default::default(),
            critical_lens: Default::default(),
            critical_annotations: Default::default(),
            critical_graph_inputs: None,
            critical_marks,
            index_tools,
            index_importer: Default::default(),
            index_generator: Default::default(),
            generation_report: None,
            index_plan: Default::default(),
            index_plan_job: Default::default(),
            index_projects_open: false,
            tool_probe_job: Default::default(),
            tool_availability: Default::default(),
            reference_context: Default::default(),
            semantic_inputs: None,
            definition_preview: Default::default(),
            under_caret: None,
            under_caret_job: Default::default(),
            under_caret_error: None,
            definition_identity: None,
            definition_expanded: false,
            reference_viewport: Default::default(),
            bookmark_list: Default::default(),
            outline_list: Default::default(),
            active_pr: None,
            review: None,
            review_rx: None,
            pending_pr: None,
            pending_open_file: None,
            pr_changes: Vec::new(),
            active_revision: None,
            line_changes: None,
            mode: SourceMode::Normal,
            sidebar_tab: SidebarTab::Files,
            search_options: SearchOptions::default(),
            repository_search: RepositorySearch::default(),
            bookmarks: BookmarkStore::new(),
            modal: ActiveModal::None,
            source_loader: Default::default(),
            pending_navigation: None,
            scroll_to_line: None,
            error: None,
            show_sidebar: true,
            show_inspector: true,
        }
    }

    /// Restores the last confirmed repository when no explicit launch root was supplied.
    pub fn restore_session(&mut self) {
        self.launch(None, None, None);
    }

    /// Captures the attached root and its private storage for background operations.
    fn cached_repository(&mut self) -> Option<(PathBuf, PathBuf)> {
        let root = self.repository_root.clone()?;
        match self.session.repository_cache(&root) {
            Some(cache) => Some((root, cache)),
            None => {
                self.error =
                    Some("No private cache directory is available for this repository".into());
                None
            }
        }
    }

    /// Opens the repository located at `path`, discovering files and restoring bookmarks.
    ///
    /// Initial source preparation is asynchronous; hosts drive [`Self::poll_source`]
    /// or the normal frame loop to publish the selected document. Semantic index
    /// imports also run asynchronously; hosts drive [`Self::poll_index`].
    pub fn open_repository(&mut self, path: &Path) {
        let root = RepositoryDiscovery::find_root(path);
        let files = match RepositoryDiscovery::discover_files(&root) {
            Ok(files) => files,
            Err(error) => {
                self.error = Some(error.to_string());
                return;
            }
        };
        self.save_bookmarks();
        self.reset_repository_state();
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repository")
            .to_string();

        self.repository_root = Some(root.clone());
        self.critical_marks.open(root.clone());
        self.repository_name = Some(name);
        self.session.active_pr = None;
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

        self.all_files = files;
        self.refresh_file_tree();
        self.load_bookmarks();

        self.load_repository_index();
        self.inspect_index_projects();

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

    /// Requests a file while keeping the current document usable until publication.
    pub fn open_file(&mut self, rel_path: &str) {
        self.request_navigation(
            SourceLocation::with_revision(rel_path, 0, None),
            crate::source::loader::SourceTarget::Byte(0),
            source::NavigationIntent {
                record_history: true,
                ..Default::default()
            },
        );
    }

    fn record_departure(&mut self) {
        if let Some(buffer) = &self.open_buffer {
            let location =
                buffer.location_at_byte(buffer.byte_offset(self.caret_line, self.caret_col));
            if self.history.current().is_some() {
                self.history.update_current(location);
            } else {
                self.history.visit(location);
            }
        }
    }

    /// Requests exact canonical bytes, retaining the revision and departure.
    ///
    /// Existing-document navigation is immediate; new documents publish atomically
    /// through [`Self::poll_source`] after their read and analysis finish.
    pub fn navigate(&mut self, location: SourceLocation, record_history: bool) {
        self.navigate_target(
            location,
            None,
            source::NavigationIntent {
                record_history,
                reveal: true,
                ..Default::default()
            },
        );
    }

    fn navigate_target(
        &mut self,
        location: SourceLocation,
        line: Option<usize>,
        intent: source::NavigationIntent,
    ) {
        let target = line.map_or(
            crate::source::loader::SourceTarget::Byte(location.byte_offset),
            |line| crate::source::loader::SourceTarget::LineColumn { line, column: 0 },
        );
        self.request_navigation(location, target, intent);
    }

    /// Navigates directly to a file, line number, and optional exact byte offset.
    pub fn jump_to_location(&mut self, rel_path: &str, line: usize, byte_offset: usize) {
        let location = SourceLocation::with_revision(rel_path, byte_offset, self.active_revision);
        self.navigate_target(
            location,
            (byte_offset == 0).then_some(line),
            source::NavigationIntent {
                record_history: true,
                reveal: true,
                ..Default::default()
            },
        );
    }

    fn refresh_current_diff(&mut self) {
        self.line_changes = None;
        if let Some(review) = &self.review {
            self.line_changes = self
                .selected_path
                .as_ref()
                .and_then(|path| review.diffs.get(path))
                .cloned();
            return;
        }
        let (Some(root), Some(buffer)) = (&self.repository_root, &self.open_buffer) else {
            return;
        };
        if let Ok(output) = std::process::Command::new("git")
            .args(["diff", "-U0", "HEAD", "--", &buffer.path])
            .current_dir(root)
            .output()
        {
            if output.status.success() {
                self.line_changes = Some(UnifiedDiffParser::parse(&String::from_utf8_lossy(
                    &output.stdout,
                )));
            }
        }
    }

    fn search_root(&self) -> Option<PathBuf> {
        self.review
            .as_ref()
            .map(|review| review.head_root.clone())
            .or_else(|| self.repository_root.clone())
    }

    fn reset_repository_state(&mut self) {
        self.critical_marks.close();
        self.cancel_source_load();
        self.reset_index_state();
        self.source_find = Default::default();
        self.source_viewport = Default::default();
        self.test_code = Default::default();
        self.repository_search = RepositorySearch::default();
        self.search_options = SearchOptions::default();
        self.repository_root = None;
        self.repository_name = None;
        self.branch_name = None;
        self.all_files.clear();
        self.file_tree.clear();
        self.expanded_dirs.clear();
        self.selected_path = None;
        self.open_buffer = None;
        self.history.clear();
        self.pin_store.clear();
        self.call_sites = Default::default();
        self.syntactic_occurrences = Default::default();
        self.critical_lens.stop();
        self.critical_annotations.stop();
        self.critical_graph_inputs = None;
        self.fold_store = FoldStore::default();
        self.fold_ranges.clear();
        self.fold_error = None;
        self.outline_symbols.clear();
        self.highlighted_lines.clear();
        self.reference_context = Default::default();
        self.reset_previews();
        self.active_pr = None;
        self.review = None;
        self.review_rx = None;
        self.pending_pr = None;
        self.pending_open_file = None;
        self.active_revision = None;
        self.pr_changes.clear();
        self.line_changes = None;
        self.bookmarks = BookmarkStore::new();
        self.bookmark_list = Default::default();
        self.caret_line = 1;
        self.caret_col = 0;
        self.scroll_to_line = None;
        self.modal = ActiveModal::None;
        self.sidebar_tab = SidebarTab::Files;
        self.error = None;
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
        self.refresh_current_diff();
    }

    pub(super) fn repository_interaction_ready(&self) -> bool {
        self.repository_root.is_some() && self.pending_pr.is_none() && self.review_rx.is_none()
    }

    pub(super) fn source_location_ready(&self) -> bool {
        self.repository_interaction_ready()
            && self.open_buffer.is_some()
            && !self.is_source_loading()
    }

    /// Dispatches consumed shortcuts through the same commands used by menus.
    pub fn handle_shortcuts(&mut self, ctx: &Context) {
        if matches!(self.modal, ActiveModal::None) && self.show_inspector {
            for action in self.reference_viewport.take_actions(ctx) {
                self.handle_reference_action(action);
            }
            for action in ContextInspector::take_definition_actions(ctx) {
                match action {
                    InspectorAction::ExpandDefinition(expanded) => {
                        self.definition_expanded = expanded
                    }
                    InspectorAction::GoToDefinition => {
                        self.execute_command(ctx, ToolbarAction::GoToDefinition)
                    }
                    _ => {}
                }
            }
        }
        let reader_focused = self.source_viewport.has_focus(ctx)
            || self.reference_viewport.has_focus(ctx)
            || crate::ui::bookmark_list::BookmarkList::has_focus(ctx)
            || crate::ui::outline_list::OutlineList::has_focus(ctx)
            || ctx.memory(|memory| memory.has_focus(ContextInspector::definition_focus_id()));
        for command in super::commands::take_shortcuts(
            ctx,
            !matches!(self.modal, ActiveModal::None),
            reader_focused,
        ) {
            self.execute_command(ctx, command);
        }
    }
}

impl eframe::App for RadiologyApp {
    fn raw_input_hook(&mut self, ctx: &Context, input: &mut egui::RawInput) {
        if matches!(self.modal, ActiveModal::None) && self.show_inspector {
            self.handle_bookmark_keys(ctx, input);
            self.handle_outline_keys(ctx, input);
            for action in self.reference_viewport.take_raw_actions(ctx, input) {
                self.handle_reference_action(action);
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.cancel_source_load();
        self.reference_context = Default::default();
        self.reset_previews();
        self.critical_annotations.stop();
        self.critical_graph_inputs = None;
        self.index_generator.stop();
        self.index_importer.stop();
        if let Err(error) = self.flush_critical_marks(std::time::Duration::from_secs(10)) {
            tracing::warn!(%error, "Critical Lens marks were not saved at shutdown");
        }
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.poll_review(ctx);
        self.poll_source(ctx);
        self.poll_index(ctx);
        if self.repository_search.poll() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
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
                    self.semantic_index
                        .as_deref()
                        .filter(|_| self.active_revision != Some(ReviewRevision::Before)),
                    &self.outline_symbols,
                ) {
                    match action {
                        ModalAction::JumpToIndexedSymbol(id) => {
                            self.navigate_to_indexed_symbol(&id);
                            close_modal = true;
                        }
                        ModalAction::JumpToSymbol(byte) => {
                            if let Some(buffer) = &self.open_buffer {
                                self.navigate(buffer.location_at_byte(byte), true);
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
                            self.open_pull_request(num);
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
                self.execute_command(ctx, action);
            }
        });

        if let Some(number) = self.pending_pr {
            TopBottomPanel::top("review_preparation").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!("Preparing pull request #{number}…"));
                    if ui.button("Cancel").clicked() {
                        self.review_rx = None;
                        self.pending_pr = None;
                        self.pending_open_file = None;
                    }
                });
            });
        }

        self.show_source_loading(ctx);
        self.show_index_status(ctx);

        if let Some(error) = self.error.clone() {
            TopBottomPanel::top("reader_notice").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(220, 90, 60), error);
                    if ui.small_button("Dismiss").clicked() {
                        self.error = None;
                    }
                });
            });
        }

        if self.show_sidebar {
            SidePanel::left("left_sidebar")
                .resizable(true)
                .default_width(260.0)
                .min_width(180.0)
                .max_width(500.0)
                .show(ctx, |ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    let prev_tab = self.sidebar_tab;
                    let previous_options = self.search_options.clone();
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
                        &mut self.session.source_filters,
                        &mut self.expanded_dirs,
                        self.selected_path.as_deref(),
                        self.active_pr.as_ref(),
                        &self.pr_changes,
                        &mut self.search_options,
                        &self.repository_search,
                        visible_count,
                        total_count,
                        self.branch_name.as_deref(),
                    ) {
                        match action {
                            SidebarAction::OpenFile(p) => {
                                self.open_file(&p);
                            }
                            SidebarAction::OpenSearchMatch(m) => {
                                self.navigate(
                                    SourceLocation::with_revision(
                                        &m.path,
                                        m.byte_offset,
                                        self.review.as_ref().map(|_| ReviewRevision::After),
                                    ),
                                    true,
                                );
                            }
                            SidebarAction::StartSearch => {
                                if let Some(root) = self.search_root() {
                                    self.repository_search
                                        .start(&root, self.search_options.clone());
                                }
                            }
                            SidebarAction::StopSearch => self.repository_search.stop(),
                            SidebarAction::ClearSearch => {
                                self.repository_search.stop();
                                self.repository_search.matches.clear();
                            }
                            SidebarAction::FiltersChanged => self.filters_changed(),
                            SidebarAction::ShowAllFiles => {
                                self.execute_command(ctx, ToolbarAction::ShowAllFiles)
                            }
                        }
                    }

                    if previous_options != self.search_options {
                        if let Some(root) = self.search_root() {
                            self.repository_search
                                .schedule(&root, self.search_options.clone());
                            ctx.request_repaint_after(std::time::Duration::from_millis(150));
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
                    let (enclosing_sym, _enclosing_line) = if let Some(buf) = &self.open_buffer {
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
                    let scope = self
                        .active_pr
                        .as_ref()
                        .map_or_else(|| "Checkout".to_string(), |pr| format!("PR #{}", pr.number));
                    self.sync_outline();
                    if let Some(action) = ContextInspector::show(
                        ui,
                        &self.theme,
                        InspectorContent {
                            under_caret: self.under_caret.as_ref(),
                            under_caret_loading: self.under_caret_job.is_running(),
                            under_caret_error: self.under_caret_error.as_deref(),
                            definition_preview: self.definition_preview.phase(),
                            definition_expanded: self.definition_expanded,
                            declaration: enclosing_sym.as_ref(),
                            buffer: self.open_buffer.as_deref(),
                            reference_results: &self.reference_context.results,
                            selected_reference: self.reference_context.selected,
                            reference_disclosures: &self.reference_context.disclosures,
                            reference_target: self
                                .reference_context
                                .explicit()
                                .or(self.reference_context.target())
                                .map(|target| target.name.as_str()),
                            explicit_references: self.reference_context.explicit().is_some(),
                            references_loading: self.reference_context.is_query_running(),
                            reference_error: self.reference_context.error.as_deref(),
                            references_available: self.reference_context.available(),
                            outline: &self.outline_symbols,
                            test_code: Some(&self.test_code),
                            bookmarks: &self.bookmarks,
                            scope: &scope,
                            bookmarks_enabled: self.repository_interaction_ready(),
                            can_toggle_bookmark: self.source_location_ready(),
                            current_bookmarked: self.current_bookmarked(),
                            outline_enabled: self.outline_available(),
                        },
                        &mut self.reference_viewport,
                        &mut self.bookmark_list,
                        &mut self.outline_list,
                    ) {
                        match action {
                            InspectorAction::Reference(action) => {
                                self.handle_reference_action(action)
                            }
                            InspectorAction::FollowCaret => self.follow_reference_caret(),
                            InspectorAction::ExpandDefinition(expanded) => {
                                self.definition_expanded = expanded
                            }
                            InspectorAction::GoToDefinition => {
                                self.execute_command(ctx, ToolbarAction::GoToDefinition)
                            }
                            InspectorAction::JumpToIndexedLocation { path, range } => {
                                self.navigate_to_indexed_range(&path, &range);
                            }
                            InspectorAction::JumpToSymbol(byte) => {
                                if let Some(buffer) = &self.open_buffer {
                                    self.navigate_target(
                                        buffer.location_at_byte(byte),
                                        None,
                                        source::NavigationIntent {
                                            record_history: true,
                                            reveal: true,
                                            flash_destination: true,
                                            ..Default::default()
                                        },
                                    );
                                }
                            }
                            InspectorAction::Outline(action) => self.handle_outline_action(action),
                            InspectorAction::Bookmark(action) => {
                                self.handle_bookmark_action(ctx, action)
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
                if let Some(buf) = self.open_buffer.clone() {
                    let found = ui
                        .add_enabled_ui(matches!(self.modal, ActiveModal::None), |ui| {
                            self.source_find.show(ui, &buf)
                        })
                        .inner;
                    if let Some(range) = found {
                        self.test_code.reveal_range(&buf, &range);
                        self.navigate(buf.location_at_byte(range.start), true);
                        self.fold_store.reveal_byte(
                            range.end.saturating_sub(1).max(range.start),
                            &buf,
                            &self.fold_ranges,
                        );
                    }
                    let mut enclosing = Vec::new();
                    let caret_byte = buf.byte_offset(self.caret_line, self.caret_col);
                    for sym in &self.outline_symbols {
                        sym.breadcrumbs(caret_byte, &mut enclosing);
                    }

                    let caret_before = (self.caret_line, self.caret_col);
                    if let Some(action) = SourceView::show(
                        ui,
                        &self.theme,
                        SourceViewContent {
                            mode: self.mode,
                            mark_context: self.semantic_index.as_deref().map(|index| {
                                crate::critical::menu::MarkContext {
                                    index,
                                    overrides: self.critical_marks.overrides(),
                                }
                            }),
                            mark_error: self.critical_marks.error(),
                            projection: self.critical_lens.projection(&buf),
                            annotations: self.critical_annotations.current(&buf),
                            hover_inputs: self.critical_graph_inputs.clone(),
                            critical_status: self.critical_status(),
                            buffer: &buf,
                            highlighted_lines: &self.highlighted_lines,
                            caret_line: &mut self.caret_line,
                            caret_col: &mut self.caret_col,
                            fold_ranges: &self.fold_ranges,
                            fold_error: self.fold_error.as_deref(),
                            occurrence_error: self.syntactic_occurrences.error.as_deref(),
                            fold_store: &mut self.fold_store,
                            line_changes: self.line_changes.as_ref(),
                            pin_store: &self.pin_store,
                            wrap_lines: self.session.wrap_lines,
                            enclosing_symbols: &enclosing,
                            scroll_to_line: &mut self.scroll_to_line,
                            has_semantic_index: self.semantic_index.as_deref().is_some_and(
                                |index| {
                                    index.documents.contains_key(&buf.path)
                                        && index.matches_buffer(&buf)
                                        && index.column_encoding(&buf.path).is_some()
                                },
                            ) && buf.revision != Some(ReviewRevision::Before),
                            keyboard_enabled: matches!(self.modal, ActiveModal::None),
                            find: &mut self.source_find,
                            font_size: self.session.source_font_size,
                            test_code: &self.test_code,
                        },
                        &mut self.source_viewport,
                    ) {
                        match action {
                            SourceViewAction::CaretMoved { line, column, .. } => {
                                self.caret_line = line;
                                self.caret_col = column;
                            }
                            SourceViewAction::NavigateToDefinition(id) => {
                                self.navigate_to_indexed_symbol(&id)
                            }
                            SourceViewAction::FindReferences(byte) => {
                                self.find_references_at_byte(byte)
                            }
                            SourceViewAction::JumpToDefinition => {
                                self.execute_command(ctx, ToolbarAction::GoToDefinition)
                            }
                            SourceViewAction::ToggleFold(_) => {}
                            SourceViewAction::ToggleTestRegion(id) => {
                                if let Some(buffer) = &self.open_buffer {
                                    self.test_code.toggle(buffer, id);
                                }
                            }
                            SourceViewAction::CollapseFunctions => {
                                self.execute_command(ctx, ToolbarAction::CollapseFunctions)
                            }
                            SourceViewAction::NextChangeBlock => {
                                self.execute_command(ctx, ToolbarAction::NextChangeBlock)
                            }
                            SourceViewAction::PrevChangeBlock => {
                                self.execute_command(ctx, ToolbarAction::PrevChangeBlock)
                            }
                            SourceViewAction::ToggleRevision(revision) => {
                                self.execute_command(ctx, ToolbarAction::ToggleRevision(revision))
                            }
                            SourceViewAction::JumpToSymbol(byte) => {
                                if let Some(buffer) = &self.open_buffer {
                                    self.navigate(buffer.location_at_byte(byte), true);
                                }
                            }
                            SourceViewAction::PinAtByte(byte) => self.pin_at_byte(byte),
                            SourceViewAction::RemovePin(symbol) => {
                                self.pin_store.remove(&symbol);
                            }
                            SourceViewAction::ClearPins => self.pin_store.clear_current(),
                            SourceViewAction::SetCriticality(edit) => {
                                self.apply_critical_mark(edit);
                            }
                            SourceViewAction::OpenGoToLine => {
                                self.modal = ActiveModal::GoToLine(GoToLineModal::default());
                            }
                        }
                    }
                    self.finish_source_interaction(caret_before);
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
