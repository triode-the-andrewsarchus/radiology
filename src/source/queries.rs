//! Captured source-query ownership and bounded reuse of validated semantic results.
use super::{
    loader::{check_source, SourceRead},
    ReviewRevision, SourceBuffer, SourceLocation,
};
use crate::semantic::SemanticIndex;
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    sync::Arc,
};
use std::{sync::atomic::AtomicBool, time::Instant};

/// Captured storage, displayed bytes and index for exact-source semantic queries.
pub struct SourceQueryInputs {
    /// Checkout or pinned review storage selected before the query starts.
    pub source: SourceRead,
    /// Immutable displayed bytes, which take precedence over storage reads.
    pub buffer: Arc<SourceBuffer>,
    /// Immutable index whose coordinates and source provenance must be validated.
    pub index: Arc<SemanticIndex>,
}

impl SourceQueryInputs {
    pub(crate) fn read(
        &self,
        path: &str,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<Arc<SourceBuffer>, String> {
        check_source(cancel, deadline)?;
        if !SourceLocation::is_repository_path(path) {
            return Err("Invalid repository-relative source path".into());
        }
        if path == self.buffer.path {
            Ok(self.buffer.clone())
        } else {
            self.source.read_cancellable(
                &SourceLocation::with_revision(path, 0, self.buffer.revision),
                cancel,
                deadline,
            )
        }
    }
}

const BYTE_LIMIT: usize = 32 * 1024 * 1024;

pub(crate) struct QueryCache<K, V, const CAPACITY: usize> {
    inputs: Option<Arc<SourceQueryInputs>>,
    sources: HashMap<String, String>,
    entries: VecDeque<(K, V, usize)>,
    bytes: usize,
}

impl<K, V, const CAPACITY: usize> Default for QueryCache<K, V, CAPACITY> {
    fn default() -> Self {
        Self {
            inputs: None,
            sources: HashMap::new(),
            entries: VecDeque::new(),
            bytes: 0,
        }
    }
}

impl<K: PartialEq, V, const CAPACITY: usize> QueryCache<K, V, CAPACITY> {
    /// Changes repository/index/revision ownership, or observes accepted source edits.
    /// No filesystem reads occur here; unseen checkout edits are validated by queries.
    pub(crate) fn prepare(&mut self, inputs: Option<&Arc<SourceQueryInputs>>) -> bool {
        let same_scope = match (&self.inputs, inputs) {
            (Some(old), Some(new)) => {
                Arc::ptr_eq(&old.index, &new.index)
                    && old.buffer.revision == new.buffer.revision
                    && match (&old.source, &new.source) {
                        (SourceRead::Checkout(a), SourceRead::Checkout(b)) => a == b,
                        (SourceRead::Review(a), SourceRead::Review(b)) => Arc::ptr_eq(a, b),
                        (SourceRead::Buffer(a), SourceRead::Buffer(b)) => {
                            super::buffer::SourceIdentity::new(a).matches(b)
                        }
                        _ => false,
                    }
            }
            (None, None) => true,
            _ => false,
        };
        if !same_scope {
            self.invalidate();
            self.sources.clear();
        }
        self.inputs = inputs.cloned();
        let changed_source = inputs.is_some_and(|inputs| {
            self.observe_source(
                &inputs.buffer.path,
                inputs.buffer.revision,
                inputs.buffer.content_digest(),
            )
        });
        !same_scope || changed_source
    }

    pub(crate) fn invalidate(&mut self) {
        self.entries.clear();
        self.bytes = 0;
    }

    /// Tracks source read by excerpts/references/callers, including before first visit.
    pub(crate) fn observe_source(
        &mut self,
        path: &str,
        revision: Option<ReviewRevision>,
        digest: &str,
    ) -> bool {
        let key = SourceLocation::format_position_key(path, revision);
        let changed = self
            .sources
            .insert(key, digest.to_owned())
            .is_some_and(|old| old != digest);
        if changed {
            self.invalidate();
        }
        changed
    }

    pub(crate) fn observe_sources(
        &mut self,
        revision: Option<ReviewRevision>,
        sources: &BTreeMap<String, String>,
    ) {
        for (path, digest) in sources {
            self.observe_source(path, revision, digest);
        }
    }

    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        self.entries
            .iter()
            .find(|entry| &entry.0 == key)
            .map(|entry| &entry.1)
    }

    /// Bounds both entry count and retained payload; reads do not reorder entries.
    pub(crate) fn insert(&mut self, key: K, value: V, bytes: usize) {
        let bytes = bytes.saturating_add(std::mem::size_of::<(K, V, usize)>());
        if bytes > BYTE_LIMIT {
            if let Some(position) = self.entries.iter().position(|entry| entry.0 == key) {
                let retired = self
                    .entries
                    .remove(position)
                    .expect("located cached result");
                self.bytes -= retired.2;
            }
            return;
        }
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.0 == key) {
            self.bytes -= entry.2;
            *entry = (key, value, bytes);
        } else {
            self.entries.push_back((key, value, bytes));
        }
        self.bytes += bytes;
        while self.entries.len() > CAPACITY || self.bytes > BYTE_LIMIT {
            if let Some((_, _, bytes)) = self.entries.pop_front() {
                self.bytes -= bytes;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{semantic::SemanticIndex, source::SourceBuffer};

    fn inputs(root: &str, text: &str, index: Arc<SemanticIndex>) -> Arc<SourceQueryInputs> {
        Arc::new(SourceQueryInputs {
            source: SourceRead::Checkout(root.into()),
            buffer: Arc::new(SourceBuffer::from_str("a.rs", text)),
            index,
        })
    }

    #[test]
    fn insertion_order_and_payload_caps_bound_retention_without_caching_oversized_values() {
        let mut cache = QueryCache::<usize, &str, 2>::default();
        cache.insert(1, "first", 1);
        cache.insert(2, "second", 1);
        assert_eq!(cache.get(&1), Some(&"first"));
        cache.insert(3, "third", 1);
        assert!(
            cache.get(&1).is_none(),
            "reads must not change insertion order"
        );
        cache.insert(2, "updated", 2);
        assert_eq!(cache.entries.len(), 2);
        assert_eq!(cache.get(&2), Some(&"updated"));
        cache.insert(4, "oversized", usize::MAX);
        assert_eq!(cache.entries.len(), 2);
        cache.insert(5, "large", BYTE_LIMIT - 64);
        assert!(cache.get(&2).is_none() && cache.get(&3).is_none());
        assert!(cache.bytes <= BYTE_LIMIT);
        let mut disabled = QueryCache::<usize, (), 0>::default();
        disabled.insert(1, (), 1);
        assert!(disabled.entries.is_empty() && disabled.bytes == 0);
        cache.insert(5, "replacement too large", usize::MAX);
        assert!(cache.get(&5).is_none() && cache.bytes == 0);
    }

    #[test]
    fn immutable_scope_and_observed_source_changes_control_reuse_across_documents() {
        let index = Arc::new(SemanticIndex::default());
        let first = inputs("/repo", "first", index.clone());
        let mut cache = QueryCache::<usize, &str, 24>::default();
        assert!(cache.prepare(Some(&first)));
        cache.insert(1, "kept", 4);
        let same = inputs("/repo", "first", index.clone());
        assert!(!cache.prepare(Some(&same)));
        assert_eq!(cache.get(&1), Some(&"kept"));
        let other = Arc::new(SourceQueryInputs {
            source: same.source.clone(),
            buffer: Arc::new(SourceBuffer::from_str("b.rs", "other")),
            index: index.clone(),
        });
        assert!(!cache.prepare(Some(&other)));
        assert_eq!(cache.get(&1), Some(&"kept"));
        cache.observe_source("a.rs", None, first.buffer.content_digest());
        assert!(cache.prepare(Some(&inputs("/repo", "edited", index.clone()))));
        assert!(cache.get(&1).is_none());
        cache.insert(2, "old root", 8);
        assert!(cache.prepare(Some(&inputs("/elsewhere", "edited", index.clone()))));
        assert!(cache.get(&2).is_none());
        cache.insert(3, "old index", 9);
        let replacement = inputs("/elsewhere", "edited", Arc::new((*index).clone()));
        assert!(cache.prepare(Some(&replacement)));
        assert!(cache.get(&3).is_none());
        let mut before = (*replacement.buffer).clone();
        before.revision = Some(ReviewRevision::Before);
        let before = Arc::new(SourceQueryInputs {
            buffer: Arc::new(before),
            source: replacement.source.clone(),
            index: replacement.index.clone(),
        });
        cache.insert(4, "after", 5);
        assert!(cache.prepare(Some(&before)));
        assert!(cache.get(&4).is_none());
        cache.insert(5, "before", 6);
        assert!(cache.prepare(None));
        assert!(cache.get(&5).is_none() && cache.sources.is_empty());
    }
}

#[cfg(test)]
mod storage_tests {
    use super::*;
    use crate::{
        semantic::SemanticIndex,
        source::{SourceBuffer, SourceLocation},
    };

    #[test]
    fn pinned_storage_and_preview_only_source_observations_cannot_leak_cached_excerpts() {
        let (_owner, root, snapshot) = crate::pull_request::review::tests::fixture_with_sources(
            "fn old() {}\n",
            "fn target() {}\n",
        );
        let snapshot = Arc::new(snapshot);
        let source = SourceRead::Review(snapshot.clone());
        let buffer = source
            .read(&SourceLocation::with_revision(
                "a file.rs",
                0,
                Some(ReviewRevision::After),
            ))
            .unwrap();
        let first = Arc::new(SourceQueryInputs {
            source,
            buffer: buffer.clone(),
            index: Arc::new(SemanticIndex::default()),
        });
        let mut cache = QueryCache::<&str, &str, 64>::default();
        cache.prepare(Some(&first));
        cache.insert("target", "pinned", 6);
        let same = Arc::new(SourceQueryInputs {
            source: SourceRead::Review(snapshot.clone()),
            buffer: buffer.clone(),
            index: first.index.clone(),
        });
        assert!(!cache.prepare(Some(&same)));
        assert_eq!(cache.get(&"target"), Some(&"pinned"));
        let mut observed = SourceBuffer::from_str("preview.rs", "old preview bytes");
        observed.revision = Some(ReviewRevision::After);
        cache.observe_source(&observed.path, observed.revision, observed.content_digest());
        let mut edited = SourceBuffer::from_str("preview.rs", "new preview bytes");
        edited.revision = observed.revision;
        let changed = Arc::new(SourceQueryInputs {
            source: same.source.clone(),
            buffer: Arc::new(edited),
            index: same.index.clone(),
        });
        assert!(cache.prepare(Some(&changed)));
        assert!(cache.get(&"target").is_none());
        cache.insert("target", "changed", 7);
        let checkout = Arc::new(SourceQueryInputs {
            source: SourceRead::Checkout(root),
            buffer,
            index: first.index.clone(),
        });
        assert!(cache.prepare(Some(&checkout)));
        assert!(cache.get(&"target").is_none());
    }
}
