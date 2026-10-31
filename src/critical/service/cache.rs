//! Bounded off-thread projection reuse within an immutable index/policy scope.

use super::*;
use crate::critical::{
    projection::{check_cancel, ClassificationMemo},
    CriticalityPolicy,
};
use std::collections::VecDeque;
use std::sync::{atomic::AtomicBool, MutexGuard, TryLockError, Weak};
use std::time::Duration;

const ENTRY_LIMIT: usize = 8;
const BYTE_LIMIT: usize = 16 * 1024 * 1024;

struct Scope {
    index: Option<Weak<SemanticIndex>>,
    policy: CriticalityPolicy,
}

impl Scope {
    fn matches(
        &self,
        index: Option<&Arc<SemanticIndex>>,
        overrides: &CriticalityOverrides,
    ) -> bool {
        let same_index = match (&self.index, index) {
            (Some(old), Some(new)) => old.ptr_eq(&Arc::downgrade(new)),
            (None, None) => true,
            _ => false,
        };
        same_index && self.policy.overrides == *overrides
    }
}

struct Entry {
    calls: Arc<CallSites>,
    projection: SourceProjection,
    bytes: usize,
}

#[derive(Default)]
pub(super) struct ProjectionCache {
    scope: Option<Scope>,
    classifications: ClassificationMemo,
    entries: VecDeque<Entry>,
    bytes: usize,
    #[cfg(test)]
    pub(super) builds: usize,
    #[cfg(test)]
    pub(super) hits: usize,
}

pub(super) fn project(
    cache: &Mutex<ProjectionCache>,
    buffer: &SourceBuffer,
    calls: Arc<CallSites>,
    index: Option<&Arc<SemanticIndex>>,
    overrides: CriticalityOverrides,
    cancel: &AtomicBool,
) -> Result<SourceProjection, String> {
    let mut cache = lock(cache, cancel)?;
    check_cancel(cancel)?;
    cache.prepare(index, overrides);
    if let Some(projection) = cache.get(buffer, &calls) {
        check_cancel(cancel)?;
        tracing::debug!(cache_hit = true, source = %buffer.path, "Critical Lens projection");
        return Ok(projection);
    }
    let ProjectionCache {
        scope,
        classifications,
        ..
    } = &mut *cache;
    let policy = &scope.as_ref().expect("prepared scope").policy;
    let projection = crate::critical::projection::make_with_memo(
        buffer,
        &calls,
        index.map(Arc::as_ref),
        policy,
        classifications,
        cancel,
    )?;
    check_cancel(cancel)?;
    #[cfg(test)]
    {
        cache.builds += 1;
    }
    cache.insert(calls, projection.clone());
    tracing::debug!(cache_hit = false, source = %buffer.path, "Critical Lens projection");
    Ok(projection)
}

fn lock<'a>(
    cache: &'a Mutex<ProjectionCache>,
    cancel: &AtomicBool,
) -> Result<MutexGuard<'a, ProjectionCache>, String> {
    loop {
        check_cancel(cancel)?;
        match cache.try_lock() {
            Ok(guard) => return Ok(guard),
            Err(TryLockError::Poisoned(error)) => return Ok(error.into_inner()),
            Err(TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(2)),
        }
    }
}

impl ProjectionCache {
    fn prepare(&mut self, index: Option<&Arc<SemanticIndex>>, overrides: CriticalityOverrides) {
        if self
            .scope
            .as_ref()
            .is_some_and(|scope| scope.matches(index, &overrides))
        {
            return;
        }
        self.entries.clear();
        self.bytes = 0;
        self.classifications = Default::default();
        self.scope = Some(Scope {
            index: index.map(Arc::downgrade),
            policy: CriticalityPolicy::for_index(index.map(Arc::as_ref), overrides),
        });
    }

    fn get(&mut self, buffer: &SourceBuffer, calls: &Arc<CallSites>) -> Option<SourceProjection> {
        let position = self.entries.iter().position(|entry| {
            entry.projection.matches(buffer)
                && (Arc::ptr_eq(&entry.calls, calls) || *entry.calls == **calls)
        })?;
        let entry = self.entries.remove(position).expect("located entry");
        let projection = entry.projection.clone();
        self.entries.push_back(entry);
        #[cfg(test)]
        {
            self.hits += 1;
        }
        Some(projection)
    }

    fn insert(&mut self, calls: Arc<CallSites>, projection: SourceProjection) {
        let bytes = weight(&calls, &projection);
        if bytes > BYTE_LIMIT {
            return;
        }
        while self.entries.len() >= ENTRY_LIMIT || bytes > BYTE_LIMIT.saturating_sub(self.bytes) {
            let Some(retired) = self.entries.pop_front() else {
                break;
            };
            self.bytes -= retired.bytes;
        }
        self.bytes += bytes;
        self.entries.push_back(Entry {
            calls,
            projection,
            bytes,
        });
    }
}

fn weight(calls: &CallSites, projection: &SourceProjection) -> usize {
    let sites = calls
        .sites
        .capacity()
        .saturating_mul(std::mem::size_of::<crate::syntax::calls::CallSite>());
    let scopes = calls
        .scopes
        .capacity()
        .saturating_mul(std::mem::size_of::<crate::syntax::calls::CallableScope>());
    calls
        .sites
        .iter()
        .fold(sites, |bytes, site| {
            bytes.saturating_add(site.kind.capacity())
        })
        .saturating_add(scopes)
        .saturating_add(calls.error.as_ref().map_or(0, String::capacity))
        .saturating_add(
            projection
                .receded_byte_ranges
                .capacity()
                .saturating_mul(std::mem::size_of::<std::ops::Range<usize>>()),
        )
        .saturating_add(projection.path.capacity().saturating_mul(2))
        .saturating_add(512)
}
