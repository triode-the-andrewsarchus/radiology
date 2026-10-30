//! Isolated environment and real pinned-store I/O; the parent test PATH stays unchanged.
use super::*;
use crate::{
    critical::worker::{CallGraphWorker, GraphInputs},
    process::{self, ProcessCommand},
    semantic::{
        incoming_calls::{self, CallerLimits},
        index::SemanticSymbolID,
        reference_results::{self, ReferenceHit, ReferenceLimits},
        ScipDocument, ScipIndex,
    },
    syntax::DocumentSyntax,
};
use std::{path::Path, time::Duration};

const CONTROL: &str = "RADIOLOGY_ANALYSIS_CANCEL_CONTROL";
const SYMBOL: &str = "rust-analyzer cargo app 0 target().";

fn pinned_inputs() -> (
    tempfile::TempDir,
    SourceQueryInputs,
    SourceBuffer,
    SemanticSymbolID,
) {
    let text = "fn target() {}\n";
    let (owner, _checkout, review) =
        crate::pull_request::review::tests::fixture_with_sources(text, text);
    let target = SourceBuffer::from_bytes(
        "a file.rs",
        text.as_bytes().to_vec(),
        Some(ReviewRevision::After),
    )
    .unwrap();
    let buffer = Arc::new(
        SourceBuffer::from_bytes(
            "shown.rs",
            b"fn root() {}\n".to_vec(),
            Some(ReviewRevision::After),
        )
        .unwrap(),
    );
    let mut index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![ScipDocument {
            relative_path: target.path.clone(),
            position_encoding: 2,
            occurrences: vec![crate::semantic::test_support::occurrence(
                &target,
                3..9,
                SYMBOL,
                true,
            )],
            ..Default::default()
        }],
        ..Default::default()
    });
    index
        .source_digests
        .insert(target.path.clone(), target.content_digest().into());
    let id = SemanticSymbolID::new(SYMBOL, &target.path);
    (
        owner,
        SourceQueryInputs {
            source: SourceRead::Review(Arc::new(review)),
            buffer,
            index: Arc::new(index),
        },
        target,
        id,
    )
}

fn blocked_pid(control: &Path, cancel: &AtomicBool) -> i32 {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(text) = std::fs::read_to_string(control.join("pid")) {
            if let Ok(pid) = text.trim().parse() {
                return pid;
            }
        }
        if Instant::now() >= deadline {
            cancel.store(true, Ordering::Relaxed);
            panic!("pinned blob read never reached the private Git wrapper");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn run_consumers(control: &Path) {
    let (_owner, inputs, target, id) = pinned_inputs();
    let range = ScipRange::from_packed(&[0, 3, 9]).unwrap();
    let preview = PreviewTarget {
        path: target.path.clone(),
        range: range.clone(),
        kind: PreviewKind::Definition,
    };
    let hit = ReferenceHit {
        path: target.path.clone(),
        range,
        byte_range: 3..9,
        source_line: preview_line(&target, 1, &(3..9)),
        is_definition: true,
    };
    for consumer in ["preview", "references", "callers", "critical"] {
        let _ = std::fs::remove_file(control.join("pid"));
        std::fs::write(control.join("hold"), "").unwrap();
        let cancel = AtomicBool::new(false);
        let mut graph = CallGraphWorker::new(GraphInputs {
            source: inputs.source.clone(),
            buffer: inputs.buffer.clone(),
            index: inputs.index.clone(),
            calls: Arc::new(DocumentSyntax::analyze(&inputs.buffer).calls),
            overrides: Default::default(),
        });
        std::thread::scope(|scope| {
            let worker = scope.spawn(|| match consumer {
                "preview" => prepare(&inputs, &preview, &cancel).map(|_| ()),
                "references" => {
                    reference_results::query(&inputs, &id, ReferenceLimits::default(), &cancel)
                        .map(|_| ())
                }
                "callers" => incoming_calls::trace(
                    &inputs,
                    &hit,
                    target.content_digest(),
                    CallerLimits::default(),
                    &cancel,
                )
                .map(|_| ()),
                "critical" => graph.frontier_of(&id, &cancel).map(|_| ()),
                _ => unreachable!(),
            });
            let pid = blocked_pid(control, &cancel);
            assert_eq!(
                unsafe { libc::kill(pid, 0) },
                0,
                "{consumer}: child must be live before cancellation"
            );
            let started = Instant::now();
            cancel.store(true, Ordering::Relaxed);
            let error = worker.join().unwrap().unwrap_err();
            assert!(error.contains("cancelled"), "{consumer}: {error}");
            assert!(
                started.elapsed() < Duration::from_secs(2),
                "{consumer}: cancellation must interrupt Git"
            );
            let status = unsafe { libc::kill(pid, 0) };
            let errno = std::io::Error::last_os_error().raw_os_error();
            assert_eq!(status, -1, "{consumer}: direct Git child must be reaped");
            assert_eq!(errno, Some(libc::ESRCH));
            println!("{consumer}: live pinned-blob child {pid} cancelled and reaped");
        });
        std::fs::remove_file(control.join("hold")).unwrap();
        let recovered = graph.frontier_of(&id, &AtomicBool::new(false)).unwrap();
        assert!(
            recovered.summary.body.is_some(),
            "{consumer}: retirement must not cache an unreadable body"
        );
    }
}

#[test]
fn cancelled_semantic_reads_reap_pinned_blob_children() {
    if let Some(control) = std::env::var_os(CONTROL) {
        run_consumers(Path::new(&control));
        return;
    }
    let owner = tempfile::tempdir().unwrap();
    let directory = owner.path();
    let wrapper = crate::repository::test_support::fake_git(
        directory,
        r#"
if [ "$5" = cat-file ] && [ "$6" = blob ] && [ -f "$RADIOLOGY_ANALYSIS_CANCEL_CONTROL/hold" ]; then
    echo $$ > "$RADIOLOGY_ANALYSIS_CANCEL_CONTROL/pid"
    exec sleep 30
fi
exec /usr/bin/git "$@"
"#,
    );
    std::fs::rename(wrapper, directory.join("git")).unwrap();
    let mut search = vec![directory.to_path_buf()];
    search.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let command = ProcessCommand {
        executable: std::env::current_exe().unwrap(),
        arguments: ["--exact", "source::preview::cancellation_tests::cancelled_semantic_reads_reap_pinned_blob_children", "--nocapture"].map(Into::into).into(),
        directory: directory.into(),
        environment: [(CONTROL.into(), directory.into()), ("PATH".into(), std::env::join_paths(search).unwrap())].into(),
        resources: Vec::new(),
    };
    let output = process::run(&command, &AtomicBool::new(false), Duration::from_secs(20));
    if !matches!(&output, Ok(result) if result.succeeded()) {
        // On a broken cancellation path, the isolated test runner can time out
        // before its separately grouped Git child. Clean that owned group too.
        if let Ok(text) = std::fs::read_to_string(directory.join("pid")) {
            if let Ok(pid) = text.trim().parse::<i32>() {
                unsafe {
                    libc::kill(-pid, libc::SIGKILL);
                }
            }
        }
    }
    let output = output.unwrap();
    print!("{}", output.stdout.text);
    assert!(
        output.succeeded(),
        "{}\n{}",
        output.stdout.text,
        output.stderr.text
    );
}
