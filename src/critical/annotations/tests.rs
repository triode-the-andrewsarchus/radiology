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
    let result = compute(inputs.clone(), &folds, &AtomicBool::new(false)).unwrap();
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
    assert!(compute(inputs, &[], &AtomicBool::new(true)).is_err());
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
