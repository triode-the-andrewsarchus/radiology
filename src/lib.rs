//! Radiology is a native, read-only code browser designed for understanding unfamiliar repositories.
//!
//! Originally developed for macOS, this crate provides a standalone, Linux-native rewrite
//! using Rust and EGUI ([`eframe`] and [`egui`]). It combines Git-aware repository discovery,
//! fast syntax highlighting with `syntect`, Tree-sitter outline and code folding, asynchronous
//! `ripgrep` search, SCIP semantic cross-referencing, multi-level caller trees, persistent bookmarks,
//! and GitHub pull request reviews with unified diff navigation.
//!
//! # Architecture and Subsystems
//!
//! - [`source`]: Canonical byte-addressed source buffer, line indexing, coordinate translation,
//!   and linear navigation history.
//! - [`repository`]: Git discovery, directory traversal, file classification, and fuzzy file search.
//! - [`syntax`]: Multi-language syntax highlighting, Tree-sitter AST queries, and code folding.
//! - [`search`]: Asynchronous streaming `ripgrep` repository search.
//! - [`semantic`]: Zero-dependency pure-Rust SCIP protobuf wire reader, symbol resolution, and caller trees.
//! - [`bookmarks`]: Repository and pull-request bookmarks with line-anchor relocation.
//! - [`critical`]: Critical Lens mode to distinguish runtime plumbing from core business and system logic.
//! - [`pull_request`]: GitHub pull request integration, unified diff parsing, and Before/After viewing.
//! - [`theme`]: Adaptive dark and light palettes, caller identity colors, and occurrence highlights.
//! - [`ui`]: EGUI application state, toolbar, sidebar, central source reader, and context inspector.
//!
//! # Examples
//!
//! ```rust
//! use radiology::source::SourceBuffer;
//!
//! let buffer = SourceBuffer::from_str("example.rs", "fn main() {\n    println!(\"Hello!\");\n}\n");
//! assert_eq!(buffer.line_count(), 4);
//! assert_eq!(buffer.line_text(1), "fn main() {");
//! ```

mod background;
pub mod bookmarks;
pub mod critical;
pub mod process;
pub mod pull_request;
pub mod repository;
pub mod search;
pub mod semantic;
pub mod session;
pub mod source;
pub mod syntax;
pub mod theme;
pub mod ui;

pub use ui::RadiologyApp;
