use super::*;
use crate::critical::{
    graph::{CallCoverage, CoverageGaps},
    Criticality,
};
use crate::semantic::{ScipDocument, ScipIndex};
use std::ops::Range;

const ROOT: &str = "rust-analyzer cargo app 0 root().";
const HELPER: &str = "rust-analyzer cargo app 0 helper().";
const OPEN: &str = "rust-analyzer cargo std 0 fs/open().";

fn occurrence(buffer: &SourceBuffer, name: &str) -> Range<usize> {
    let byte = buffer.text.find(name).unwrap();
    byte..byte + name.len()
}

fn document(buffer: &SourceBuffer, names: &[(&str, &str)]) -> ScipDocument {
    let index = super::super::test_support::index_ranges(
        buffer,
        names
            .iter()
            .map(|(name, id)| (occurrence(buffer, name), (*id).to_owned())),
    );
    let mut document = index.documents[&buffer.path].clone();
    document.occurrences[0].symbol_roles = 1;
    document
}

pub(in crate::critical) fn fixture() -> (tempfile::TempDir, GraphInputs) {
    let dir = tempfile::tempdir().unwrap();
    let root = Arc::new(SourceBuffer::from_str(
        "a.rs",
        "fn root() {\n helper();\n}\n",
    ));
    let child = SourceBuffer::from_str("b.rs", "fn helper() {\n open();\n}\n");
    std::fs::write(dir.path().join("a.rs"), &root.text).unwrap();
    std::fs::write(dir.path().join("b.rs"), &child.text).unwrap();
    let mut index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![
            document(&root, &[("root", ROOT), ("helper", HELPER)]),
            document(&child, &[("helper", HELPER), ("open", OPEN)]),
        ],
        ..Default::default()
    });
    for buffer in [&*root, &child] {
        index
            .source_digests
            .insert(buffer.path.clone(), buffer.content_digest().to_owned());
    }
    let mut overrides = CriticalityOverrides::default();
    overrides.set_symbol(HELPER, Some(Criticality::Ordinary));
    let inputs = GraphInputs {
        source: SourceRead::Checkout(dir.path().into()),
        calls: Arc::new(DocumentSyntax::analyze(&root).calls),
        buffer: root,
        index: Arc::new(index),
        overrides,
    };
    (dir, inputs)
}

#[test]
fn exact_cross_file_bodies_are_loaded_once_and_primed_displayed_bytes_win_over_disk() {
    let (dir, inputs) = fixture();
    std::fs::write(dir.path().join("a.rs"), "fn root() { bogus(); }").unwrap();
    let mut worker = CallGraphWorker::new(inputs.clone());
    let result = worker
        .frontier_in_source(0, &AtomicBool::new(false))
        .unwrap();
    assert_eq!(result.annotation().label(), "1");
    assert_eq!(result.paths[0].call.callee.symbol, OPEN);
    assert_eq!(result.paths[0].via[0].callee.symbol, HELPER);
    assert_eq!(
        result.summary.body.as_ref().unwrap().content_digest,
        inputs.buffer.content_digest()
    );
    let bytes = worker.source_bytes;
    std::fs::remove_file(dir.path().join("b.rs")).unwrap();
    assert_eq!(
        worker
            .frontier_in_source(0, &AtomicBool::new(false))
            .unwrap(),
        result
    );
    assert_eq!(worker.source_bytes, bytes);
    // A fresh request cannot reuse another snapshot's loaded file or cached summary.
    let mut fresh = CallGraphWorker::new(inputs);
    assert!(matches!(
        fresh
            .frontier_in_source(0, &AtomicBool::new(false))
            .unwrap()
            .coverage,
        CallCoverage::Partial(CoverageGaps {
            missing_bodies: 1,
            ..
        })
    ));
}

#[test]
fn stale_source_and_resource_limits_are_explicit_and_new_policy_has_no_old_cache() {
    let (dir, inputs) = fixture();
    let mut limited = CallGraphWorker::new(inputs.clone());
    limited.source_budget = 1;
    let result = limited
        .frontier_in_source(0, &AtomicBool::new(false))
        .unwrap();
    assert_eq!(result.annotation().label(), "0+");
    assert!(matches!(
        result.coverage,
        CallCoverage::Partial(CoverageGaps {
            truncated: true,
            ..
        })
    ));
    std::fs::write(dir.path().join("b.rs"), "fn helper() {\n fake();\n}\n").unwrap();
    let result = CallGraphWorker::new(inputs.clone())
        .frontier_in_source(0, &AtomicBool::new(false))
        .unwrap();
    assert!(matches!(
        result.coverage,
        CallCoverage::Partial(CoverageGaps {
            missing_bodies: 1,
            ..
        })
    ));
    let mut changed = inputs;
    changed
        .overrides
        .set_symbol(HELPER, Some(Criticality::Critical));
    let result = CallGraphWorker::new(changed)
        .frontier_in_source(0, &AtomicBool::new(false))
        .unwrap();
    assert_eq!(result.annotation().label(), "1");
    assert!(result.paths.is_empty());
    assert_eq!(result.traversed_bodies, 0);
}

#[test]
fn external_abstract_and_cancelled_bodies_remain_distinct() {
    let (_dir, mut inputs) = fixture();
    let mut worker = CallGraphWorker::new(inputs.clone());
    let external = worker
        .frontier_of(&SemanticSymbolID::new(OPEN, ""), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(
        external.coverage,
        CallCoverage::NoBody(NoBodyReason::External)
    );
    assert!(worker
        .frontier_in_source(0, &AtomicBool::new(true))
        .is_err());
    let buffer = Arc::new(SourceBuffer::from_str("a.rs", "trait T { fn root(); }"));
    let mut index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![document(&buffer, &[("root", ROOT)])],
        ..Default::default()
    });
    index
        .source_digests
        .insert("a.rs".into(), buffer.content_digest().to_owned());
    inputs.source = SourceRead::Buffer(buffer.clone());
    inputs.calls = Arc::new(DocumentSyntax::analyze(&buffer).calls);
    inputs.buffer = buffer;
    inputs.index = Arc::new(index);
    assert_eq!(
        CallGraphWorker::new(inputs)
            .frontier_of(&SemanticSymbolID::new(ROOT, ""), &AtomicBool::new(false))
            .unwrap()
            .coverage,
        CallCoverage::NoBody(NoBodyReason::Abstract)
    );
}

#[test]
fn review_bodies_come_from_pinned_git_objects_and_before_does_not_reuse_after_edges() {
    use crate::pull_request::review::ReviewSnapshot;
    use crate::source::ReviewRevision;
    let (dir, mut inputs) = fixture();
    for args in [
        vec!["init", "-q"],
        vec!["add", "a.rs", "b.rs"],
        vec!["commit", "-qm", "Pinned fixture"],
    ] {
        let output = std::process::Command::new("git")
            .args([
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
            ])
            .args(args)
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let head = String::from_utf8(output.stdout).unwrap().trim().to_owned();
    let review = ReviewSnapshot {
        summary: serde_json::from_value(
            serde_json::json!({"number":1,"title":"Fixture","state":"OPEN"}),
        )
        .unwrap(),
        base: head.clone(),
        head,
        object_dir: dir.path().join(".git"),
        head_root: dir.path().into(),
        files: vec![],
        changes: vec![],
        diffs: HashMap::new(),
    };
    inputs.source = SourceRead::Review(Arc::new(review));
    inputs.buffer = Arc::new(
        SourceBuffer::from_bytes(
            "a.rs",
            inputs.buffer.data.clone(),
            Some(ReviewRevision::After),
        )
        .unwrap(),
    );
    inputs.calls = Arc::new(DocumentSyntax::analyze(&inputs.buffer).calls);
    std::fs::write(
        dir.path().join("b.rs"),
        "fn helper() {
 changed_checkout();
}
",
    )
    .unwrap();
    let result = CallGraphWorker::new(inputs.clone())
        .frontier_in_source(0, &AtomicBool::new(false))
        .unwrap();
    assert_eq!(result.annotation().label(), "1");
    assert_eq!(result.paths[0].call.callee.symbol, OPEN);
    inputs.buffer = Arc::new(
        SourceBuffer::from_bytes(
            "a.rs",
            inputs.buffer.data.clone(),
            Some(ReviewRevision::Before),
        )
        .unwrap(),
    );
    inputs.calls = Arc::new(DocumentSyntax::analyze(&inputs.buffer).calls);
    assert_eq!(
        CallGraphWorker::new(inputs)
            .frontier_in_source(0, &AtomicBool::new(false))
            .unwrap()
            .coverage,
        CallCoverage::NoBody(NoBodyReason::NotIndexed)
    );
}
