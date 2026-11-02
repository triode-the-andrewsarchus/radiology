//! Ordered, named source traces independent of semantic indexing and UI toolkits.
//!
//! Text imports use a title followed by `file:function:description` records, not
//! persisted source coordinates. Resolving declarations and presenting traces are
//! separate from the format and app-owned storage APIs here.

mod parser;
pub mod store;
pub use parser::{ImportError, MAX_IMPORT_BYTES};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// One named source destination and its freeform explanation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodePathStep {
    /// Stable identity, independent of repeated destination names.
    pub id: Uuid,
    /// Original repository-relative filename, without source coordinates.
    pub path: String,
    /// Declaration name, qualified name or full declaration header.
    pub function: String,
    /// User-authored explanation; colons and backslashes are literal.
    pub description: String,
}

impl CodePathStep {
    /// Checks the import contract without requiring the source file to exist.
    /// Dot components and repeated slashes are retained for later path resolution.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.path.is_empty()
            && !self.path.starts_with('/')
            && !self.path.contains('\0')
            && !self.path.split('/').any(|part| part == "..")
            && !self.function.is_empty()
            && !self.description.is_empty()
    }
}

/// An ordered source trace; same-title traces and repeated steps remain distinct.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodePath {
    /// Stable saved trace identity.
    pub id: Uuid,
    /// Freeform title from the first input line.
    pub title: String,
    /// Destinations in their authored order.
    pub steps: Vec<CodePathStep>,
}

impl CodePath {
    /// Parses a complete trace atomically, assigning fresh trace and step identities.
    ///
    /// # Examples
    /// ```
    /// use radiology::code_paths::CodePath;
    /// let trace = CodePath::parse("Boot\nsrc/boot.cc:Kernel\\:\\:boot:Starts: CPU setup")?;
    /// assert_eq!(trace.steps[0].function, "Kernel::boot");
    /// assert_eq!(trace.steps[0].description, "Starts: CPU setup");
    /// assert_eq!(CodePath::parse(&trace.text())?.title, "Boot");
    /// # Ok::<(), radiology::code_paths::ImportError>(())
    /// ```
    ///
    /// # Errors
    /// Rejects oversized input, missing title/steps or malformed records, reporting
    /// the original input line when a record is invalid. No records are published.
    pub fn parse(text: &str) -> Result<Self, ImportError> {
        parser::parse(text)
    }

    /// Checks persisted record validity, including unique step identities.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let mut seen = HashSet::new();
        !self.title.is_empty()
            && !self.steps.is_empty()
            && self
                .steps
                .iter()
                .all(|step| step.is_valid() && seen.insert(step.id))
    }

    /// Exports the title and ordered records in the import escape format.
    /// Stored identities are preserved in storage, not embedded in copied text.
    #[must_use]
    pub fn text(&self) -> String {
        let mut lines = vec![self.title.clone()];
        lines.extend(self.steps.iter().map(|step| {
            format!(
                "{}:{}:{}",
                escape(&step.path),
                escape(&step.function),
                step.description
            )
        }));
        lines.join("\n")
    }
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace(':', "\\:")
}

#[cfg(test)]
mod tests;
