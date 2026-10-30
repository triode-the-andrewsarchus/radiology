//! Managed worktrees with exact reuse checks, shared reader leases and bounded LRU.

use super::{
    lease::{self, StoreLease},
    store::{self, GitStore},
    verify_tree, ReviewControl,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

const LIMIT: usize = 3;

#[derive(Debug, Clone)]
struct Context {
    cache: PathBuf,
    git: PathBuf,
}

#[derive(Debug)]
pub(crate) struct TreeLease {
    file: Option<File>,
    context: Context,
}

impl Drop for TreeLease {
    fn drop(&mut self) {
        drop(self.file.take());
        let context = self.context.clone();
        crate::background::READ_WORKERS.spawn_blocking(move || {
            let cancel = AtomicBool::new(false);
            let mut control = ReviewControl::new(&cancel);
            control.git = &context.git;
            control.deadline = Instant::now() + Duration::from_secs(5);
            if let Ok(Some(writer)) = StoreLease::try_acquire(&context.cache, &control) {
                let control = ReviewControl {
                    lease: Some(writer.file()),
                    ..control
                };
                if let Err(error) = prune(&context.cache, &control) {
                    tracing::debug!(%error,"Review tree cleanup deferred");
                }
            }
        });
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct State {
    format: u32,
    head: String,
    name: String,
    used: u64,
    complete: bool,
}

impl State {
    fn tree(&self, cache: &Path) -> PathBuf {
        cache.join("revisions").join(&self.head).join(&self.name)
    }
    fn metadata(&self, cache: &Path) -> PathBuf {
        cache
            .join("revisions")
            .join(&self.head)
            .join("tree-cache")
            .join(&self.name)
    }
}

pub(super) struct PreparedTree {
    pub path: PathBuf,
    pub lease: Arc<TreeLease>,
    state: State,
    context: Context,
    temporary: Option<tempfile::TempDir>,
}

impl Drop for PreparedTree {
    fn drop(&mut self) {
        if let Some(directory) = self.temporary.take() {
            drop(directory);
            let metadata = self.state.metadata(&self.context.cache);
            let _ = fs::remove_dir_all(&metadata);
            if let Some(parent) = metadata.parent() {
                let _ = fs::remove_dir(parent);
            }
            let _ = fs::remove_dir(self.path.parent().expect("tree revision parent"));
        }
    }
}

impl PreparedTree {
    pub fn prepare(
        store: &GitStore<'_>,
        head: &str,
        control: &ReviewControl<'_>,
    ) -> Result<Self, String> {
        let context = Context {
            cache: store
                .0
                .root
                .parent()
                .ok_or("Private store has no parent")?
                .to_owned(),
            git: control.git.into(),
        };
        let mut states = scan(&context.cache, control)?;
        states.sort_by_key(|state| std::cmp::Reverse(state.used));
        for state in states
            .iter()
            .filter(|state| state.head == head && state.complete)
        {
            let path = state.tree(&context.cache);
            if verify_tree::verify(store, head, &path, control).is_ok() {
                if let Ok(lease) = shared(state, &context, false) {
                    return Ok(Self {
                        path,
                        lease,
                        state: state.clone(),
                        context,
                        temporary: None,
                    });
                }
            }
            control.check()?;
        }
        Self::create(store, head, control, context)
    }

    fn create(
        store: &GitStore<'_>,
        head: &str,
        control: &ReviewControl<'_>,
        context: Context,
    ) -> Result<Self, String> {
        let revision = context.cache.join("revisions").join(head);
        fs::create_dir_all(&revision).map_err(|error| error.to_string())?;
        let temporary = tempfile::Builder::new()
            .prefix("tree-")
            .tempdir_in(&revision)
            .map_err(|error| error.to_string())?;
        let path = temporary.path().to_owned();
        let state = State {
            format: 1,
            head: head.into(),
            name: path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or("Invalid tree name")?
                .into(),
            used: 0,
            complete: false,
        };
        fs::create_dir_all(state.metadata(&context.cache)).map_err(|error| error.to_string())?;
        let lease = shared(&state, &context, true)?;
        let tree = Self {
            path,
            lease,
            state,
            context,
            temporary: Some(temporary),
        };
        write_state(&tree.state, &tree.context.cache)?;
        store.materialize(head, &tree.path)?;
        verify_tree::verify(store, head, &tree.path, control)?;
        Ok(tree)
    }

    pub fn commit(&mut self, control: &ReviewControl<'_>) -> Result<(), String> {
        control.check()?;
        self.state.used = scan(&self.context.cache, control)?
            .iter()
            .map(|state| state.used)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        self.state.complete = true;
        write_state(&self.state, &self.context.cache)?;
        control.check()?;
        if let Some(directory) = self.temporary.take() {
            let _ = directory.keep();
        }
        let maintenance = ReviewControl {
            deadline: control
                .deadline
                .min(Instant::now() + Duration::from_secs(5)),
            ..*control
        };
        if let Err(error) = prune(&self.context.cache, &maintenance) {
            tracing::debug!(%error,"Review retention deferred");
        }
        control.check()
    }
}

fn shared(state: &State, context: &Context, create: bool) -> Result<Arc<TreeLease>, String> {
    let file = lease::open_lock(&state.metadata(&context.cache).join("lease"), create)?;
    if !lease::try_lock(&file, true)? {
        return Err("Review tree is being removed; retry".into());
    }
    Ok(Arc::new(TreeLease {
        file: Some(file),
        context: context.clone(),
    }))
}

fn write_state(state: &State, cache: &Path) -> Result<(), String> {
    let directory = state.metadata(cache);
    let mut temporary =
        tempfile::NamedTempFile::new_in(&directory).map_err(|error| error.to_string())?;
    serde_json::to_writer(temporary.as_file_mut(), state).map_err(|error| error.to_string())?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| error.to_string())?;
    temporary
        .persist(directory.join("state.json"))
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn real_directory(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir() && !metadata.is_symlink())
}

fn scan(cache: &Path, control: &ReviewControl<'_>) -> Result<Vec<State>, String> {
    let revisions = cache.join("revisions");
    if !real_directory(&revisions) {
        return Ok(vec![]);
    }
    let mut states = Vec::new();
    for revision in fs::read_dir(&revisions).map_err(|error| error.to_string())? {
        control.check()?;
        let revision = revision.map_err(|error| error.to_string())?;
        states.extend(scan_revision(&revision, cache, control)?);
    }
    Ok(states)
}

fn scan_revision(
    revision: &std::fs::DirEntry,
    cache: &Path,
    control: &ReviewControl<'_>,
) -> Result<Vec<State>, String> {
    let revisions = cache.join("revisions");
    let mut states = Vec::new();
    let Some(head) = revision.file_name().to_str().map(str::to_owned) else {
        return Ok(states);
    };
    if super::required_oid(Some(&head)).is_err() || !real_directory(&revision.path()) {
        return Ok(states);
    }
    let directory = revision.path().join("tree-cache");
    if !real_directory(&directory) {
        return Ok(states);
    }
    for item in fs::read_dir(&directory).map_err(|error| error.to_string())? {
        control.check()?;
        let item = item.map_err(|error| error.to_string())?;
        if !real_directory(&item.path()) {
            continue;
        }
        if let Some(state) = read_state(&item.path().join("state.json")) {
            if state.head == head
                && item.file_name().to_str() == Some(state.name.as_str())
                && store::owned_tree(&state.tree(cache), &revisions)
            {
                states.push(state);
            }
        }
    }
    Ok(states)
}

fn read_state(path: &Path) -> Option<State> {
    let mut bytes = Vec::new();
    crate::repository::identity::open_regular_file(path)
        .ok()?
        .take(4097)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > 4096 {
        return None;
    }
    let state: State = serde_json::from_slice(&bytes).ok()?;
    (state.format == 1).then_some(state)
}

fn prune(cache: &Path, control: &ReviewControl<'_>) -> Result<(), String> {
    let (protected, mut idle) = idle_states(cache, control)?;
    idle.sort_by(|a, b| b.used.cmp(&a.used).then_with(|| a.name.cmp(&b.name)));
    let mut remaining = LIMIT.saturating_sub(protected);
    let objects = cache.join("review.git");
    let store = GitStore(control.git(&objects));
    for state in idle {
        control.check()?;
        if state.complete && remaining > 0 {
            remaining -= 1;
            continue;
        }
        let file = lease::open_lock(&state.metadata(cache).join("lease"), false)?;
        if !lease::try_lock(&file, false)? {
            continue;
        }
        let path = state.tree(cache);
        if fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.is_symlink()) {
            continue;
        }
        store.remove_tree(&path)?;
        fs::remove_dir_all(state.metadata(cache)).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn idle_states(cache: &Path, control: &ReviewControl<'_>) -> Result<(usize, Vec<State>), String> {
    let mut protected = 0usize;
    let mut idle = Vec::new();
    for state in scan(cache, control)? {
        control.check()?;
        let Ok(file) = lease::open_lock(&state.metadata(cache).join("lease"), false) else {
            continue;
        };
        if lease::try_lock(&file, false)? {
            idle.push(state);
        } else {
            protected += 1;
        }
    }
    Ok((protected, idle))
}

#[cfg(all(test, unix))]
mod tests;
