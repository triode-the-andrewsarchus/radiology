//! EGUI desktop application user interface components and layouts.
//!
//! Provides the top [`Toolbar`], the left [`Sidebar`] with Files/Changes/Search tabs,
//! the central [`SourceView`] reader surface with compact line number gutter, the right
//! [`ContextInspector`] with symbol definition details, caller trees, outline, and bookmarks,
//! and modal dialogs for quick file jumping and symbol navigation.

mod context_menu;
mod critical_hover;
mod critical_menu;
mod reference_callers;
mod source_preview;
#[cfg(test)]
mod test_support;
mod text_fit;
mod under_caret;

pub mod app;
pub mod bookmark_list;
pub mod commands;
pub mod context_inspector;
pub mod error_notice;
pub mod icons;
pub mod modals;
pub mod outline_list;
pub mod pull_request_menu;
pub mod reference_list;
pub mod sidebar;
pub mod source_find;
pub mod source_view;
pub mod toolbar;

pub use app::RadiologyApp;
pub use context_inspector::{ContextInspector, InspectorAction, InspectorContent};
pub use icons::Icon;
pub use modals::{
    GoToLineModal, GoToSymbolModal, ModalAction, PullRequestModal, QuickFilePickerModal,
};
pub use sidebar::{Sidebar, SidebarAction, SidebarTab};
pub use source_view::{SourceView, SourceViewAction, SourceViewContent, SourceViewport};
pub use toolbar::{Toolbar, ToolbarAction};
