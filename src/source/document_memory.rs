//! Session presentation memory keyed by exact source bytes and revision.

use super::SourceBuffer;
use std::collections::HashMap;

/// Moves nondefault presentation choices between active immutable documents.
/// Hosts clear this memory at repository/review-snapshot boundaries.
#[derive(Debug, Clone, Default)]
pub(crate) struct DocumentMemory<T> {
    active: Option<(String, String)>,
    saved: HashMap<String, (String, T)>,
}

impl<T: Default + PartialEq> DocumentMemory<T> {
    /// Activates successfully published source; returns whether identity changed.
    /// A changed digest invalidates old choices for that path/revision. Failed
    /// source reads must not call this method or modify the current payload.
    pub(crate) fn activate(&mut self, buffer: &SourceBuffer, current: &mut T) -> bool {
        let identity = (buffer.position_key(), buffer.content_digest().to_owned());
        if self.active.as_ref() == Some(&identity) {
            return false;
        }
        let departing = std::mem::take(current);
        if let Some((key, digest)) = self.active.take() {
            if departing != T::default() {
                self.saved.insert(key, (digest, departing));
            }
        }
        *current = self
            .saved
            .remove(&identity.0)
            .filter(|(digest, _)| digest == &identity.1)
            .map(|(_, value)| value)
            .unwrap_or_default();
        self.active = Some(identity);
        true
    }
}
