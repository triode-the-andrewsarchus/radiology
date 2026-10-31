//! Background index generation with isolated inputs and atomic publication.

use super::command::CommandPaths;
use super::inventory::{check_cancel, InputInventory};
use super::process::{self, OutputStream};
use super::tools::{ToolAvailability, ToolDirectory};
use super::workspace::{IndexingWorkspace, WorkspaceLimits};
use super::{IndexPlan, Indexer, IndexingStrategy};
use crate::background::LatestRead;
use crate::semantic::{ScipIndex, SemanticIndex};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, Arc, Mutex};
use std::time::{Duration, Instant};

/// Inputs owned by one checkout or pinned-review generation request.
#[derive(Clone)]
pub struct GenerationRequest {
    /// Stable attached checkout identity, including when `root` is a pinned tree.
    pub attached_root: PathBuf,
    /// Reuses exact project records with matching tool versions; false forces all tools.
    pub reuse: bool,
    /// Viewed repository whose bytes must remain unchanged.
    pub root: PathBuf,
    /// Pinned review owning this root and its tree lease, or None for a checkout.
    pub review: Option<Arc<crate::pull_request::review::ReviewSnapshot>>,
    /// Private cache parent, outside the viewed repository.
    pub cache: PathBuf,
    /// Explicit managed/PATH search context.
    pub tools: ToolDirectory,
    /// Deadline for each indexer process; probes have their own 10-second limit.
    pub timeout: Duration,
    /// Input-copy limits applied to the pristine and per-strategy workspaces.
    pub limits: WorkspaceLimits,
}

impl GenerationRequest {
    pub(super) fn inventory(&self, cancel: &AtomicBool) -> Result<InputInventory, String> {
        let Some(review) = &self.review else {
            return InputInventory::discover(&self.root, cancel);
        };
        if std::fs::canonicalize(&self.root).map_err(|e| e.to_string())?
            != std::fs::canonicalize(&review.head_root).map_err(|e| e.to_string())?
        {
            return Err("Index input root differs from its pinned review".into());
        }
        let inputs = review.head_inputs(cancel)?;
        let mut inventory = InputInventory::from_paths(inputs.blobs.into_keys())?;
        inventory.omitted_directories = inputs.omitted_directories;
        Ok(inventory)
    }

    fn prepare(&self, cancel: &AtomicBool) -> Result<(InputInventory, IndexingWorkspace), String> {
        super::cache::directory(self)?;
        let inventory = self.inventory(cancel)?;
        let baseline =
            IndexingWorkspace::create(&self.root, &inventory, &self.cache, self.limits, cancel)?;
        if let Some(review) = &self.review {
            baseline.verify_snapshot(&review.head_inputs(cancel)?.blobs)?;
        }
        Ok((inventory, baseline))
    }

    pub(super) fn identity_root(&self) -> Result<PathBuf, String> {
        std::fs::canonicalize(
            self.review
                .as_ref()
                .map_or(&self.root, |review| &review.object_dir),
        )
        .map_err(|error| error.to_string())
    }

    pub(super) fn revision(&self) -> Option<&str> {
        self.review.as_ref().map(|review| review.head.as_str())
    }
}

/// Latest bounded progress snapshot, without an event backlog.
#[derive(Debug, Default, Clone)]
pub struct GenerationProgress {
    /// One-based project step, or zero while discovering inputs.
    pub step: usize,
    /// Projects selected by the actual build-input inventory.
    pub total: usize,
    /// Active strategy identity, if any.
    pub project: Option<String>,
    /// Current phase or bounded last stderr line.
    pub message: String,
}

/// Outcome of one independently isolated project run.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StrategyOutcome {
    /// Detected project and source coverage denominator.
    pub strategy: IndexingStrategy,
    /// Actual successful tool version, if its probe succeeded.
    pub version: Option<String>,
    /// Original generating After commit or `checkout`, when this project was reused.
    #[serde(default)]
    pub reused_from: Option<String>,
    /// A project cache could not be read or staged; generated data remains usable.
    #[serde(default)]
    pub cache_warning: Option<String>,
    /// Emitted navigable source documents.
    pub documents: usize,
    /// Source files not emitted by the tool.
    pub missing_count: usize,
    /// At most 100 missing paths for the diagnostic view.
    pub missing_paths: Vec<String>,
    /// Failure explanation, if no usable index was produced for this strategy.
    pub failure: Option<String>,
    /// Bounded stdout/stderr retained from the process.
    pub diagnostics: String,
    /// Tool reported a partial compiler build.
    pub partial_build: bool,
    /// Elapsed time for preparation, probe and execution.
    pub elapsed: Duration,
}

/// Provenance and coverage of the last completed generation.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GenerationReport {
    /// Exact pinned After commit, or None for a mutable checkout index.
    #[serde(default)]
    pub revision: Option<String>,
    /// Plan derived from the full build-input inventory.
    pub plan: IndexPlan,
    /// Project outcomes in plan order.
    pub outcomes: Vec<StrategyOutcome>,
    /// SHA-256 identity of the pristine source inputs.
    pub fingerprint: String,
    /// Submodules/directories that were not materialized.
    pub omitted_directories: Vec<String>,
    /// Working tree changed while generation was running.
    pub stale_reason: Option<String>,
    /// Overlapping document paths were emitted by independent projects.
    pub overlapping_documents: usize,
    /// Distinct navigable source documents after project merging.
    pub documents: usize,
    /// Data was restored from this repository's persistent cache.
    pub cached: bool,
    /// Persistence failed; this session's generated data remains usable.
    pub cache_warning: Option<String>,
}

impl GenerationReport {
    /// Returns whether generation omitted work, failed projects or source coverage.
    #[must_use]
    pub fn is_partial(&self) -> bool {
        !self.plan.skipped.is_empty()
            || !self.omitted_directories.is_empty()
            || self.overlapping_documents != 0
            || self.reuse_warning().is_some()
            || self.outcomes.iter().any(|outcome| {
                outcome.failure.is_some() || outcome.missing_count != 0 || outcome.partial_build
            })
    }

    /// Returns the cross-project warning when generation mixes index origins.
    #[must_use]
    pub fn reuse_warning(&self) -> Option<&'static str> {
        let origins: HashSet<_> = self
            .outcomes
            .iter()
            .filter(|outcome| outcome.failure.is_none())
            .map(|outcome| outcome.reused_from.as_deref().unwrap_or("generated here"))
            .collect();
        (origins.len() > 1).then_some("Projects were reused and regenerated, or reused from different revisions. Cross-project references may be outdated; Reindex regenerates everything.")
    }

    /// Returns a short reader-facing status without claiming full semantic resolution.
    #[must_use]
    pub fn summary(&self) -> String {
        let documents = self.documents;
        let succeeded = self
            .outcomes
            .iter()
            .filter(|outcome| outcome.failure.is_none())
            .count();
        let state = if self.stale_reason.is_some() && self.cached {
            "Stale cached index"
        } else if self.stale_reason.is_some() {
            "Stale generated index"
        } else if documents == 0 {
            "No index generated"
        } else if self.cached && self.is_partial() {
            "Partial cached index"
        } else if self.cached {
            "Cached index"
        } else {
            self.generation_label()
        };
        format!(
            "{state} · {documents} document{} · {succeeded}/{} project{}",
            if documents == 1 { "" } else { "s" },
            self.outcomes.len(),
            if self.outcomes.len() == 1 { "" } else { "s" }
        )
    }

    fn generation_label(&self) -> &'static str {
        let reused = self
            .outcomes
            .iter()
            .any(|outcome| outcome.reused_from.is_some());
        match (reused, self.is_partial()) {
            (true, true) => "Partial updated index",
            (true, false) => "Updated index",
            (false, true) => "Partial generated index",
            (false, false) => "Generated index",
        }
    }
}

/// One publication from a completed generation; failures may retain no new index.
pub struct GeneratedIndex {
    /// New semantic data, or None when all project runs failed or none applied.
    pub index: Option<SemanticIndex>,
    /// Full bounded project outcomes and source provenance.
    pub report: GenerationReport,
}

/// State of a cache-only request, allowing hosts to choose an import fallback.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum CacheLoadStatus {
    /// No cache load has been requested.
    #[default]
    Idle,
    /// Reading and validating persisted data and source inputs.
    Loading,
    /// No persisted index exists for this repository.
    Missing,
    /// A current or explicitly stale index was published.
    Loaded,
    /// The persisted data could not be read or validated.
    Failed(String),
    /// The host cancelled the load.
    Cancelled,
}

/// Generation ownership shared by standalone readers and embedded hosts.
#[derive(Default)]
pub struct IndexGenerator {
    job: LatestRead<Result<Option<GeneratedIndex>, String>>,
    cache_status: CacheLoadStatus,
    progress: Arc<Mutex<GenerationProgress>>,
    error: Option<String>,
    cancelled: bool,
}

impl IndexGenerator {
    /// Starts generation and retires any older process/publication.
    pub fn start(&mut self, request: GenerationRequest) {
        self.error = None;
        self.cache_status = CacheLoadStatus::Idle;
        self.cancelled = false;
        self.progress = Arc::new(Mutex::new(GenerationProgress {
            message: "Discovering build inputs…".into(),
            ..Default::default()
        }));
        let progress = self.progress.clone();
        self.job
            .spawn(move |cancel| generate(request, cancel, &progress).map(Some));
    }

    /// Starts a cache-only load; never executes an indexer or creates a workspace.
    pub fn start_cached(&mut self, request: GenerationRequest) {
        self.error = None;
        self.cancelled = false;
        self.cache_status = CacheLoadStatus::Loading;
        self.progress = Arc::new(Mutex::new(GenerationProgress {
            message: "Loading cached index…".into(),
            ..Default::default()
        }));
        self.job
            .spawn(move |cancel| super::cache::load(&request, cancel));
    }

    /// Returns the cache lifecycle independently of tool generation errors.
    #[must_use]
    pub fn cache_status(&self) -> &CacheLoadStatus {
        &self.cache_status
    }

    /// Cancels processes and publication; previously published semantic data is host-owned.
    pub fn stop(&mut self) {
        if self.job.is_running() {
            self.job.stop();
            self.cancelled = true;
            if self.cache_status == CacheLoadStatus::Loading {
                self.cache_status = CacheLoadStatus::Cancelled;
            }
        }
    }

    /// Returns whether generation is awaiting a current completion.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.job.is_running()
    }

    /// Returns whether the last pending generation was explicitly cancelled.
    #[must_use]
    pub fn was_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Returns the last preparation/worker failure.
    #[must_use]
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Returns a bounded live progress snapshot.
    #[must_use]
    pub fn progress(&self) -> GenerationProgress {
        self.progress
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    /// Polls only the current generation, retaining preparation errors for the UI.
    pub fn poll(&mut self) -> Option<GeneratedIndex> {
        match self.job.poll() {
            Ok(Some(Ok(generated))) => {
                if self.cache_status == CacheLoadStatus::Loading {
                    self.cache_status = if generated.is_some() {
                        CacheLoadStatus::Loaded
                    } else {
                        CacheLoadStatus::Missing
                    };
                }
                generated
            }
            Ok(Some(Err(error))) => {
                if self.cache_status == CacheLoadStatus::Loading {
                    self.cache_status = CacheLoadStatus::Failed(error.clone());
                }
                self.error = Some(error);
                None
            }
            Ok(None) => None,
            Err(_) => {
                if self.cache_status == CacheLoadStatus::Loading {
                    self.cache_status =
                        CacheLoadStatus::Failed("Cache worker stopped unexpectedly".into());
                }
                self.error = Some("Index generation stopped unexpectedly".into());
                None
            }
        }
    }
}

fn progress(state: &Mutex<GenerationProgress>, message: impl Into<String>) {
    state
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .message = message.into();
}

fn generate(
    request: GenerationRequest,
    cancel: &AtomicBool,
    state: &Mutex<GenerationProgress>,
) -> Result<GeneratedIndex, String> {
    let (inventory, baseline) = request.prepare(cancel)?;
    let plan = IndexPlan::inspect(inventory.paths.iter().map(String::as_str));
    let mut report = GenerationReport {
        revision: request.revision().map(str::to_owned),
        fingerprint: baseline.fingerprint(),
        omitted_directories: inventory.omitted_directories.clone(),
        plan,
        outcomes: Vec::new(),
        stale_reason: None,
        overlapping_documents: 0,
        documents: 0,
        cached: false,
        cache_warning: None,
    };
    let mut merged = ScipIndex::default();
    let mut source_digests = HashMap::new();
    let mut pending_projects = Vec::new();
    for (position, strategy) in report.plan.selected.iter().enumerate() {
        check_cancel(cancel)?;
        *state.lock().unwrap_or_else(|error| error.into_inner()) = GenerationProgress {
            step: position + 1,
            total: report.plan.selected.len(),
            project: Some(strategy.id()),
            message: "Preparing isolated project…".into(),
        };
        let (mut outcome, scip) =
            run_strategy(strategy, &request, &inventory, &baseline, cancel, state);
        if let Some(scip) = scip {
            match super::cache::projects::stage_project(
                &request, &baseline, &outcome, &scip, cancel,
            ) {
                Ok(pending) => pending_projects.extend(pending),
                Err(error) => outcome.cache_warning = Some(error),
            }
            report.overlapping_documents +=
                merge_project(&mut merged, scip, &baseline, &mut source_digests);
        }
        report.outcomes.push(outcome);
    }
    check_cancel(cancel)?;
    progress(state, "Verifying source provenance…");
    report.stale_reason = source_staleness(&request, &inventory, &baseline, cancel)?;
    check_cancel(cancel)?;
    report.documents = merged.documents.len();
    let mut warnings = super::cache::projects::publish(pending_projects, &report, cancel);
    report.cache_warning = (!warnings.is_empty()).then(|| warnings.join("\n"));
    if let Err(error) = super::cache::save(
        &request,
        &inventory,
        &merged,
        &source_digests,
        &report,
        cancel,
    ) {
        warnings.push(error);
        report.cache_warning = Some(warnings.join("\n"));
    }
    check_cancel(cancel)?;
    let index = if merged.documents.is_empty() {
        None
    } else {
        let mut index = SemanticIndex::from_scip_cancellable(merged, || {
            cancel.load(std::sync::atomic::Ordering::Relaxed)
        })
        .map_err(|error| error.to_string())?;
        index.source_digests = source_digests;
        Some(index)
    };
    Ok(GeneratedIndex { index, report })
}

fn merge_project(
    merged: &mut ScipIndex,
    scip: ScipIndex,
    baseline: &IndexingWorkspace,
    source_digests: &mut HashMap<String, String>,
) -> usize {
    let mut overlaps = 0;
    for document in scip.documents {
        let digest = baseline
            .source_digest(&document.relative_path)
            .expect("validated document")
            .to_owned();
        if source_digests
            .insert(document.relative_path.clone(), digest)
            .is_none()
        {
            merged.documents.push(document);
        } else {
            overlaps += 1;
        }
    }
    merged.external_symbols.extend(scip.external_symbols);
    overlaps
}

fn source_staleness(
    request: &GenerationRequest,
    inventory: &InputInventory,
    baseline: &IndexingWorkspace,
    cancel: &AtomicBool,
) -> Result<Option<String>, String> {
    baseline.verify_inputs(cancel)?;
    if request.review.is_some() {
        request.inventory(cancel)?;
        baseline.verify_source(cancel)?;
    }
    if let Err(reason) = baseline.verify_source(cancel) {
        return Ok(Some(reason));
    }
    Ok((request.inventory(cancel)? != *inventory)
        .then(|| "Repository inputs were added or removed during generation".into()))
}

fn run_strategy(
    strategy: &IndexingStrategy,
    request: &GenerationRequest,
    inventory: &InputInventory,
    baseline: &IndexingWorkspace,
    cancel: &AtomicBool,
    state: &Mutex<GenerationProgress>,
) -> (StrategyOutcome, Option<ScipIndex>) {
    let started = Instant::now();
    let mut outcome = StrategyOutcome {
        strategy: strategy.clone(),
        version: None,
        reused_from: None,
        cache_warning: None,
        documents: 0,
        missing_count: strategy.source_files,
        missing_paths: Vec::new(),
        failure: None,
        diagnostics: String::new(),
        partial_build: false,
        elapsed: Duration::ZERO,
    };
    let result = prepare_and_run(
        strategy,
        request,
        inventory,
        baseline,
        cancel,
        state,
        &mut outcome,
    );
    let scip = match result {
        Ok(scip) => Some(scip),
        Err(error) => {
            outcome.failure = Some(error);
            None
        }
    };
    outcome.elapsed = started.elapsed();
    (outcome, scip)
}

fn prepare_and_run(
    strategy: &IndexingStrategy,
    request: &GenerationRequest,
    inventory: &InputInventory,
    baseline: &IndexingWorkspace,
    cancel: &AtomicBool,
    state: &Mutex<GenerationProgress>,
    outcome: &mut StrategyOutcome,
) -> Result<ScipIndex, String> {
    let workspace = IndexingWorkspace::create(
        baseline.root(),
        inventory,
        &request.cache,
        request.limits,
        cancel,
    )?;
    let project = workspace.root().join(&strategy.directory);
    progress(state, "Checking tool version…");
    let executable =
        match request
            .tools
            .probe(strategy.indexer, &project, cancel, Duration::from_secs(10))
        {
            ToolAvailability::Available {
                executable,
                version,
            } => {
                outcome.version = Some(version);
                executable
            }
            ToolAvailability::Unavailable {
                reason,
                diagnostics,
                ..
            } => {
                outcome.diagnostics = diagnostics;
                return Err(reason);
            }
        };
    check_cancel(cancel)?;
    if request.reuse {
        progress(state, "Checking exact project cache…");
        match super::cache::projects::load(request, inventory, baseline, outcome, cancel) {
            Ok(Some((mut cached, scip))) => {
                record_coverage(&scip, inventory, &mut cached);
                *outcome = cached;
                return Ok(scip);
            }
            Ok(None) => {}
            Err(error) => outcome.cache_warning = Some(error),
        }
    }
    check_cancel(cancel)?;
    execute_strategy(
        request,
        inventory,
        &workspace,
        &executable,
        cancel,
        state,
        outcome,
    )
}

fn execute_strategy(
    request: &GenerationRequest,
    inventory: &InputInventory,
    workspace: &IndexingWorkspace,
    executable: &std::path::Path,
    cancel: &AtomicBool,
    state: &Mutex<GenerationProgress>,
    outcome: &mut StrategyOutcome,
) -> Result<ScipIndex, String> {
    let run = tempfile::Builder::new()
        .prefix(".radiology-index-run-")
        .tempdir_in(workspace.root())
        .map_err(|e| e.to_string())?;
    let output = run.path().join("index.scip");
    progress(state, "Preparing indexer inputs…");
    let compilation_database = prepare_compilation_database(
        &outcome.strategy,
        &request.root,
        workspace.root(),
        run.path(),
        cancel,
    )?;
    let gradle = request.tools.gradle();
    let mut command = outcome.strategy.command(CommandPaths {
        workspace: workspace.root(),
        source_root: &request.attached_root,
        executable,
        output: &output,
        run: run.path(),
        gradle: gradle.as_deref(),
        compilation_database: compilation_database.as_deref(),
    })?;
    command
        .environment
        .extend(request.tools.child_environment());
    command.environment.insert(
        "CARGO_TARGET_DIR".into(),
        run.path().join("cargo-target").into(),
    );
    let jobs = std::thread::available_parallelism().map_or(1, |cores| (cores.get() / 2).max(1));
    command
        .environment
        .insert("CARGO_BUILD_JOBS".into(), jobs.to_string().into());
    for (path, bytes) in &command.resources {
        std::fs::write(path, bytes).map_err(|e| e.to_string())?;
    }
    run_command(&command, cancel, request.timeout, state, outcome)?;
    check_cancel(cancel)?;
    workspace.verify_inputs(cancel)?;
    progress(state, "Loading generated index…");
    let scip = crate::semantic::loader::read_generated(&output, cancel)?;
    let normalized = normalize(scip, &outcome.strategy, inventory, workspace)?;
    record_coverage(&normalized, inventory, outcome);
    Ok(normalized)
}

fn prepare_compilation_database(
    strategy: &IndexingStrategy,
    original: &std::path::Path,
    workspace: &std::path::Path,
    run: &std::path::Path,
    cancel: &AtomicBool,
) -> Result<Option<PathBuf>, String> {
    if strategy.indexer != Indexer::Clang {
        return Ok(None);
    }
    let evidence = strategy
        .evidence
        .first()
        .ok_or("C/C++ indexing requires a compilation database")?;
    super::inventory::validate_input_path(evidence)?;
    let bytes = crate::semantic::loader::read_index_bytes(&workspace.join(evidence), cancel)?
        .ok_or("Copied compilation database is missing")?;
    let database = super::compilation_database::rebase(&bytes, original, workspace, run, cancel)?;
    for directory in database.directories {
        check_cancel(cancel)?;
        std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    }
    check_cancel(cancel)?;
    let path = run.join("compile_commands.json");
    std::fs::write(&path, database.bytes).map_err(|e| e.to_string())?;
    Ok(Some(path))
}

fn record_coverage(scip: &ScipIndex, inventory: &InputInventory, outcome: &mut StrategyOutcome) {
    outcome.documents = scip.documents.len();
    let covered: HashSet<_> = scip
        .documents
        .iter()
        .map(|doc| doc.relative_path.as_str())
        .collect();
    let strategy = &outcome.strategy;
    let missing: Vec<_> = inventory
        .paths
        .iter()
        .filter(|path| {
            super::contains(&strategy.directory, path)
                && strategy.indexer.covers_path(path)
                && !covered.contains(path.as_str())
        })
        .collect();
    outcome.missing_count = missing.len();
    outcome.missing_paths = missing.into_iter().take(100).cloned().collect();
}

fn run_command(
    command: &super::command::IndexCommand,
    cancel: &AtomicBool,
    timeout: Duration,
    state: &Mutex<GenerationProgress>,
    outcome: &mut StrategyOutcome,
) -> Result<(), String> {
    progress(state, "Running indexer…");
    let mut marker = Vec::new();
    let result = process::run_observed(command, cancel, timeout, |stream, bytes| {
        if stream == OutputStream::Stderr {
            marker.extend_from_slice(bytes);
            outcome.partial_build |= marker
                .windows(b"RADIOLOGY_KOTLIN_PARTIAL_BUILD".len())
                .any(|window| window == b"RADIOLOGY_KOTLIN_PARTIAL_BUILD");
            let keep = marker.len().saturating_sub(64);
            marker.drain(..keep);
            let message: String = String::from_utf8_lossy(bytes)
                .lines()
                .last()
                .unwrap_or("")
                .chars()
                .take(256)
                .collect();
            progress(state, message);
        }
        Ok(())
    });
    let output_log = match result {
        Ok(result) => result,
        Err(error) => {
            outcome.diagnostics = format!(
                "stdout:\n{}\n\nstderr:\n{}",
                error.stdout.text, error.stderr.text
            );
            return Err(error.to_string());
        }
    };
    outcome.diagnostics = format!(
        "stdout:\n{}\n\nstderr:\n{}",
        output_log.stdout.text, output_log.stderr.text
    );
    if !output_log.succeeded() {
        return Err(format!(
            "Indexer exited with code {:?}, signal {:?}",
            output_log.exit_code, output_log.signal
        ));
    }
    Ok(())
}

fn normalize(
    mut scip: ScipIndex,
    strategy: &IndexingStrategy,
    inventory: &InputInventory,
    baseline: &IndexingWorkspace,
) -> Result<ScipIndex, String> {
    // Metadata describes file bytes, not occurrence columns. In particular,
    // TypeScript emits UTF-16 columns in UTF-8 files without position_encoding.
    let fallback = match strategy.indexer.default_encoding() {
        crate::source::ColumnEncoding::Utf8 => 1,
        crate::source::ColumnEncoding::Utf16 => 2,
        crate::source::ColumnEncoding::Utf32 => 3,
    };
    let paths: HashSet<_> = inventory.paths.iter().map(String::as_str).collect();
    for document in &mut scip.documents {
        if !strategy.directory.is_empty() {
            document.relative_path = format!("{}/{}", strategy.directory, document.relative_path);
        }
        if document.position_encoding == 0 {
            document.position_encoding = fallback;
        }
        if !(1..=3).contains(&document.position_encoding) {
            return Err("Indexer emitted an unsupported column encoding".into());
        }
    }
    scip.documents.retain(|document| {
        paths.contains(document.relative_path.as_str())
            && strategy.indexer.covers_path(&document.relative_path)
            && baseline.source_digest(&document.relative_path).is_some()
    });
    if scip.documents.is_empty() {
        return Err("Generated index contains no navigable source input documents".into());
    }
    Ok(scip)
}

#[cfg(all(test, unix))]
mod tests;
