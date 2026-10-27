//! Session state persistence and recent repository history.
//!
//! Provides [`SessionStore`] for saving and restoring the last visited repository,
//! active pull request number, and user filter preferences in XDG configuration storage.

pub mod store;

pub use store::SessionStore;
