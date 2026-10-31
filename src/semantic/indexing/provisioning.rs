//! Explicit, cancellable indexer installation with verified staged publication.

mod plan;
use super::{
    tools::{ToolAvailability, ToolDirectory},
    Indexer,
};
use crate::{background::LatestRead, process};
pub use plan::ProvisioningPlan;
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

/// Installation inputs owned independently of the reader's current repository.
#[derive(Clone)]
pub struct ProvisioningRequest {
    /// Selected indexer; Clang is manual-only.
    pub indexer: Indexer,
    /// Tool discovery and private installation destination.
    pub tools: ToolDirectory,
    /// Viewed trees installation must not write into.
    pub protected_roots: Vec<PathBuf>,
    /// Total budget, including writer waiting and verification.
    pub timeout: Duration,
}

/// Verified outcome of an explicit installation request.
pub struct ProvisionedTool {
    /// The selected adapter.
    pub indexer: Indexer,
    /// Actual post-install executable and version probe.
    pub availability: ToolAvailability,
}

/// Owns one installation and bounded latest progress; accepted indexes are host-owned.
#[derive(Default)]
pub struct ToolProvisioner {
    job: LatestRead<Result<ProvisionedTool, String>>,
    progress: Arc<Mutex<String>>,
    cancelled: bool,
}

impl ToolProvisioner {
    /// Starts an explicit install, retiring any older installation publication.
    pub fn start(&mut self, request: ProvisioningRequest) {
        self.cancelled = false;
        self.progress = Arc::new(Mutex::new("Preparing tool installation…".into()));
        let progress = self.progress.clone();
        self.job
            .spawn(move |cancel| install(request, cancel, &progress));
    }
    /// Cancels the installer process group without discarding the host's accepted index.
    pub fn stop(&mut self) {
        if self.job.is_running() {
            self.job.stop();
            self.cancelled = true;
        }
    }
    /// Returns whether a current installation is pending.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.job.is_running()
    }
    /// Returns whether installation was explicitly cancelled.
    #[must_use]
    pub fn was_cancelled(&self) -> bool {
        self.cancelled
    }
    /// Returns the bounded latest installation phase or diagnostic line.
    #[must_use]
    pub fn progress(&self) -> String {
        self.progress
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    /// Takes the current verified result; retired results never publish.
    pub fn poll(&mut self) -> Option<Result<ProvisionedTool, String>> {
        match self.job.poll() {
            Ok(result) => result,
            Err(_) => Some(Err("Tool installer stopped unexpectedly".into())),
        }
    }
}

fn check(cancel: &AtomicBool, deadline: Instant) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("Tool installation cancelled".into())
    } else if Instant::now() >= deadline {
        Err("Tool installation timed out".into())
    } else {
        Ok(())
    }
}

fn install(
    mut request: ProvisioningRequest,
    cancel: &AtomicBool,
    progress: &Mutex<String>,
) -> Result<ProvisionedTool, String> {
    let deadline = Instant::now() + request.timeout;
    check(cancel, deadline)?;
    let root = super::workspace::destination_path(
        request
            .tools
            .managed
            .as_ref()
            .ok_or("No private tool storage is available")?,
    )?;
    validate_roots(&request, &root)?;
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let lease = writer(&root, cancel, deadline)?;
    request.tools.managed = Some(root.clone());
    let availability = if request.indexer == Indexer::Rust {
        execute(
            request.indexer.provisioning(&request.tools)?,
            &lease,
            cancel,
            deadline,
            progress,
        )?;
        let availability = request.tools.probe(
            request.indexer,
            &root,
            cancel,
            Duration::from_secs(10).min(deadline.saturating_duration_since(Instant::now())),
        );
        require_available(&availability)?;
        availability
    } else {
        staged(&request, &root, &lease, cancel, deadline, progress)?
    };
    check(cancel, deadline)?;
    Ok(ProvisionedTool {
        indexer: request.indexer,
        availability,
    })
}

fn validate_roots(request: &ProvisioningRequest, root: &Path) -> Result<(), String> {
    let rustup = if request.indexer == Indexer::Rust {
        std::env::var_os("RUSTUP_HOME")
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|home| home.join(".rustup")))
            .map(|path| super::workspace::destination_path(&path))
            .transpose()?
    } else {
        None
    };
    for protected in &request.protected_roots {
        let protected = std::fs::canonicalize(protected).map_err(|e| e.to_string())?;
        if root.starts_with(&protected) {
            return Err("Tool installation must be outside viewed repositories".into());
        }
        if request.indexer == Indexer::Rust
            && rustup.as_ref().is_some_and(|rustup| {
                rustup.starts_with(&protected) || protected.starts_with(rustup)
            })
        {
            return Err("The active rustup toolchain overlaps the viewed repository".into());
        }
    }
    Ok(())
}

fn writer(root: &Path, cancel: &AtomicBool, deadline: Instant) -> Result<File, String> {
    let file = process::lease::open_lock(&root.join("install.lock"), true)?;
    loop {
        check(cancel, deadline)?;
        if process::lease::try_lock(&file, false)? {
            return Ok(file);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn staged(
    request: &ProvisioningRequest,
    root: &Path,
    lease: &File,
    cancel: &AtomicBool,
    deadline: Instant,
    progress: &Mutex<String>,
) -> Result<ToolAvailability, String> {
    let stage = tempfile::Builder::new()
        .prefix(".tool-install-")
        .tempdir_in(root)
        .map_err(|e| e.to_string())?;
    let tools = ToolDirectory {
        managed: Some(stage.path().into()),
        ..request.tools.clone()
    };
    execute(
        request.indexer.provisioning(&tools)?,
        lease,
        cancel,
        deadline,
        progress,
    )?;
    verify_managed(&tools, request.indexer, stage.path(), cancel, deadline)?;
    check(cancel, deadline)?;
    let relative = match request.indexer {
        Indexer::Go => PathBuf::from("go/bin/scip-go"),
        Indexer::TypeScript | Indexer::Python => Path::new("npm").join(request.indexer.id()),
        Indexer::Kotlin => PathBuf::from("kotlin-scip"),
        _ => return Err("Unsupported staged installer".into()),
    };
    publish(
        &stage.path().join(&relative),
        &root.join(relative),
        root,
        || verify_managed(&request.tools, request.indexer, root, cancel, deadline),
    )
}

fn verify_managed(
    tools: &ToolDirectory,
    indexer: Indexer,
    directory: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<ToolAvailability, String> {
    check(cancel, deadline)?;
    let lookup = ToolDirectory {
        managed: tools.managed.clone(),
        home: None,
        search_path: vec![],
    };
    let executable = lookup
        .locate(indexer)
        .ok_or("Installer did not produce the managed executable")?;
    let availability = tools.probe_executable(
        indexer,
        executable,
        directory,
        cancel,
        Duration::from_secs(10).min(deadline.saturating_duration_since(Instant::now())),
    );
    require_available(&availability)?;
    Ok(availability)
}

fn publish(
    source: &Path,
    destination: &Path,
    root: &Path,
    verify: impl FnOnce() -> Result<ToolAvailability, String>,
) -> Result<ToolAvailability, String> {
    let resolved = super::workspace::destination_path(destination)?;
    if !resolved.starts_with(root) {
        return Err("Managed installation destination escapes its tool root".into());
    }
    std::fs::create_dir_all(destination.parent().ok_or("Missing tool parent")?)
        .map_err(|e| e.to_string())?;
    let retired = tempfile::Builder::new()
        .prefix(".tool-retired-")
        .tempdir_in(root)
        .map_err(|e| e.to_string())?;
    let backup = retired.path().join("previous");
    let previous = destination.exists();
    if previous {
        std::fs::rename(destination, &backup).map_err(|e| e.to_string())?;
    }
    let result = std::fs::rename(source, destination)
        .map_err(|e| e.to_string())
        .and_then(|_| verify());
    match result {
        Ok(available) => Ok(available),
        Err(error) => {
            if let Err(restore) = restore_previous(destination, &backup, previous) {
                let saved = retired.keep();
                return Err(format!("Tool publication failed: {error}; rollback failed: {restore}; previous files retained at {}", saved.display()));
            }
            Err(error)
        }
    }
}

fn restore_previous(destination: &Path, backup: &Path, previous: bool) -> Result<(), String> {
    if destination.exists() {
        if destination.is_dir() {
            std::fs::remove_dir_all(destination)
        } else {
            std::fs::remove_file(destination)
        }
        .map_err(|e| e.to_string())?;
    }
    if previous {
        std::fs::rename(backup, destination).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn require_available(availability: &ToolAvailability) -> Result<(), String> {
    match availability {
        ToolAvailability::Available { .. } => Ok(()),
        ToolAvailability::Unavailable {
            reason,
            diagnostics,
            ..
        } => Err(format!(
            "Installed tool could not run: {reason}\n{diagnostics}"
        )),
    }
}

fn execute(
    plan: ProvisioningPlan,
    lease: &File,
    cancel: &AtomicBool,
    deadline: Instant,
    progress: &Mutex<String>,
) -> Result<(), String> {
    check(cancel, deadline)?;
    for directory in &plan.directories {
        std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    }
    for (path, bytes) in &plan.command.resources {
        std::fs::write(path, bytes).map_err(|e| e.to_string())?;
    }
    *progress.lock().unwrap_or_else(|e| e.into_inner()) = plan.summary;
    let output = process::run_observed_with_lease(
        &plan.command,
        cancel,
        deadline.saturating_duration_since(Instant::now()),
        Some(lease),
        |_, chunk| {
            if let Some(line) = String::from_utf8_lossy(chunk)
                .lines()
                .rev()
                .find(|line| !line.trim().is_empty())
            {
                *progress.lock().unwrap_or_else(|e| e.into_inner()) =
                    line.chars().take(300).collect();
            }
            Ok(())
        },
    )
    .map_err(|e| format!("Installer {e}\n{}\n{}", e.stdout.text, e.stderr.text))?;
    if output.succeeded() {
        Ok(())
    } else {
        Err(format!(
            "Installer {}\n{}\n{}",
            output.status_description(),
            output.stdout.text,
            output.stderr.text
        ))
    }
}

#[cfg(test)]
pub(crate) mod tests;
