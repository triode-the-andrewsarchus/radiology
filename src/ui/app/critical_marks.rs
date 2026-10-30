//! Source marking entry points and repository-scoped policy persistence.

use super::*;
use crate::critical::overrides::{MarkEdit, MarkTarget};

impl RadiologyApp {
    /// Applies a menu decision only to the current exact semantic source target.
    /// Returns whether current-session policy changed; source/history stay intact.
    pub fn apply_critical_mark(&mut self, edit: MarkEdit) -> bool {
        let menu = self.open_buffer.as_ref().and_then(|buffer| {
            crate::critical::menu::MarkContext {
                index: self.semantic_index.as_deref()?,
                overrides: self.critical_marks.overrides(),
            }
            .at_byte(buffer, buffer.byte_offset(self.caret_line, self.caret_col))
        });
        let Some(menu) = menu else {
            return false;
        };
        let matches = match &edit.target {
            MarkTarget::Symbol(symbol) => *symbol == menu.symbol,
            MarkTarget::Package(package) => Some(package) == menu.package_key.as_ref(),
        };
        if !matches {
            return false;
        }
        let changed = self.critical_marks.edit(edit);
        if changed {
            self.refresh_critical_projection();
        }
        changed
    }

    /// Returns current repository decisions, including unsaved session changes.
    #[must_use]
    pub fn critical_overrides(&self) -> &crate::critical::CriticalityOverrides {
        self.critical_marks.overrides()
    }

    /// Returns whether mark loading or accepted durable writes remain.
    #[must_use]
    pub fn is_marks_loading(&self) -> bool {
        self.critical_marks.is_busy()
    }

    /// Returns a recoverable mark load/save error for the current repository.
    #[must_use]
    pub fn critical_mark_error(&self) -> Option<&str> {
        self.critical_marks.error()
    }

    /// Flushes accepted edits before a host closes; never used during normal frames.
    ///
    /// # Errors
    /// Returns persistence failures or a shutdown deadline error.
    pub fn flush_critical_marks(&mut self, timeout: std::time::Duration) -> Result<(), String> {
        self.critical_marks.flush(timeout)
    }
}
