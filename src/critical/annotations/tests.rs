use super::*;
use crate::critical::{worker::tests::fixture, Criticality};
use crate::syntax::DocumentSyntax;

fn settle(service: &mut CriticalAnnotations) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while service.is_running() {
        service.poll();
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(service.error().is_none(), "{:?}", service.error());
}

#[test]
fn fold_only_counts_bind_to_exact_bytes_and_revision_and_keep_coverage_detail() {
    let (_dir, inputs) = fixture();
    let folds = DocumentSyntax::analyze(&inputs.buffer).folds;
    let mut entries = BTreeMap::new();
    compute(inputs.clone(), &folds, &AtomicBool::new(false), |batch| {
        entries.extend(batch)
    })
    .unwrap();
    let result = SourceAnnotations {
        buffer: inputs.buffer.clone(),
        entries,
    };
    let annotation = result.at_line(&inputs.buffer, 1).unwrap();
    assert_eq!(annotation.count.label(), "1");
    assert!(annotation
        .detail
        .contains("1 critical operations reachable"));
    assert!(result.at_line(&inputs.buffer, 2).is_none());
    let changed = SourceBuffer::from_str("a.rs", "fn root() {\n absent();\n}\n");
    assert!(!result.matches(&changed));
    let before = SourceBuffer::from_bytes(
        "a.rs",
        inputs.buffer.data.clone(),
        Some(crate::source::ReviewRevision::Before),
    )
    .unwrap();
    assert!(result.at_line(&before, 1).is_none());
    assert!(compute(inputs, &[], &AtomicBool::new(true), |_| panic!(
        "cancelled publication"
    ))
    .is_err());
}

#[test]
fn source_policy_and_stop_transitions_cannot_publish_retired_counts() {
    let (_dir, inputs) = fixture();
    let folds = DocumentSyntax::analyze(&inputs.buffer).folds;
    let mut service = CriticalAnnotations::default();
    service.start(inputs.clone(), folds.clone());
    let mut next = inputs.clone();
    next.overrides.set_symbol(
        "rust-analyzer cargo std 0 fs/open().",
        Some(Criticality::Ordinary),
    );
    service.start(next, folds);
    settle(&mut service);
    assert_eq!(
        service
            .current(&inputs.buffer)
            .unwrap()
            .at_line(&inputs.buffer, 1)
            .unwrap()
            .count
            .label(),
        "0"
    );
    service.stop();
    assert!(!service.is_running());
    assert!(service.current(&inputs.buffer).is_none());
    service.poll();
    assert!(service.current(&inputs.buffer).is_none());
}

fn seven_scopes() -> (GraphInputs, Vec<FoldRange>) {
    use crate::{
        semantic::{ScipDocument, ScipIndex, SemanticIndex},
        source::loader::SourceRead,
    };
    let text = (0..7)
        .map(|i| format!("fn function_{i}() {{\n let value = {i};\n}}\n"))
        .collect::<String>();
    let buffer = Arc::new(SourceBuffer::from_str("a.rs", &text));
    let syntax = DocumentSyntax::analyze(&buffer);
    let inputs = GraphInputs {
        source: SourceRead::Buffer(buffer.clone()),
        buffer,
        calls: Arc::new(syntax.calls),
        overrides: Default::default(),
        index: Arc::new(SemanticIndex::from_scip(ScipIndex {
            documents: vec![ScipDocument {
                relative_path: "a.rs".into(),
                ..Default::default()
            }],
            ..Default::default()
        })),
    };
    (inputs, syntax.folds)
}

#[test]
fn six_function_batches_publish_before_completion_and_cancellation_stops_later_roots() {
    use std::sync::atomic::Ordering;
    let (inputs, folds) = seven_scopes();
    assert_eq!(folds.len(), 7);
    let mut batches = Vec::new();
    compute(inputs.clone(), &folds, &AtomicBool::new(false), |batch| {
        batches.push(batch)
    })
    .unwrap();
    assert_eq!(
        batches.iter().map(BTreeMap::len).collect::<Vec<_>>(),
        vec![6, 1]
    );
    let lines: Vec<_> = batches
        .iter()
        .flat_map(|batch| batch.keys().copied())
        .collect();
    assert_eq!(
        lines,
        folds.iter().map(|fold| fold.start_line).collect::<Vec<_>>()
    );
    assert!(batches
        .iter()
        .flat_map(|batch| batch.values())
        .all(|value| value.count.label() == "0"));
    let cancel = AtomicBool::new(false);
    let mut first = None;
    assert!(compute(inputs, &folds, &cancel, |batch| {
        assert!(first.is_none(), "no publication after cancellation");
        first = Some(batch);
        cancel.store(true, Ordering::Relaxed);
    })
    .is_err());
    assert_eq!(first.unwrap().len(), 6);
}

#[test]
fn completed_counts_are_visible_during_a_live_job_and_survive_a_later_failure() {
    use std::sync::mpsc;
    use std::time::Duration;
    let (_dir, inputs) = fixture();
    let folds = DocumentSyntax::analyze(&inputs.buffer).folds;
    let pending = PendingCounts {
        buffer: inputs.buffer.clone(),
        entries: Default::default(),
    };
    let mut service = CriticalAnnotations {
        pending: Some(pending.clone()),
        ..Default::default()
    };
    let (published, observed) = mpsc::channel();
    let (release, proceed) = mpsc::channel();
    service.job.spawn(move |cancel| {
        compute(inputs, &folds, cancel, |batch| {
            pending.publish(batch);
            published.send(()).unwrap();
            proceed.recv_timeout(Duration::from_secs(5)).unwrap();
        })?;
        Err("Later callable analysis failed".into())
    });
    observed.recv_timeout(Duration::from_secs(5)).unwrap();
    service.poll();
    assert!(service.is_running(), "worker is gated before completion");
    let source = service.pending.as_ref().unwrap().buffer.clone();
    assert_eq!(
        service
            .current(&source)
            .unwrap()
            .at_line(&source, 1)
            .unwrap()
            .count
            .label(),
        "1"
    );
    release.send(()).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while service.is_running() {
        service.poll();
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert_eq!(service.error(), Some("Later callable analysis failed"));
    assert_eq!(
        service
            .current(&source)
            .unwrap()
            .at_line(&source, 1)
            .unwrap()
            .count
            .label(),
        "1"
    );
}

#[test]
fn late_progress_from_a_retired_request_cannot_overwrite_a_new_policy() {
    use std::sync::mpsc;
    use std::time::Duration;
    let (_dir, inputs) = fixture();
    let folds = DocumentSyntax::analyze(&inputs.buffer).folds;
    let retired = PendingCounts {
        buffer: inputs.buffer.clone(),
        entries: Default::default(),
    };
    let mut service = CriticalAnnotations {
        pending: Some(retired.clone()),
        ..Default::default()
    };
    let (published, observed) = mpsc::channel();
    let (release, proceed) = mpsc::channel();
    let (finished, done) = mpsc::channel();
    let old_inputs = inputs.clone();
    let old_folds = folds.clone();
    service.job.spawn(move |cancel| {
        let result = compute(old_inputs, &old_folds, cancel, |batch| {
            published.send(()).unwrap();
            proceed.recv_timeout(Duration::from_secs(5)).unwrap();
            retired.publish(batch);
        });
        finished.send(()).unwrap();
        result
    });
    observed.recv_timeout(Duration::from_secs(5)).unwrap();
    let mut replacement = inputs.clone();
    replacement.overrides.set_symbol(
        "rust-analyzer cargo std 0 fs/open().",
        Some(Criticality::Ordinary),
    );
    service.start(replacement, folds);
    assert!(service.current(&inputs.buffer).is_none());
    release.send(()).unwrap();
    done.recv_timeout(Duration::from_secs(5)).unwrap();
    settle(&mut service);
    assert_eq!(
        service
            .current(&inputs.buffer)
            .unwrap()
            .at_line(&inputs.buffer, 1)
            .unwrap()
            .count
            .label(),
        "0"
    );
}
