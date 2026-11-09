use super::*;
use crate::critical::{
    graph::{CallCoverage, CoverageGaps},
    Criticality,
};
use crate::semantic::{ScipDocument, ScipIndex};
use std::{ops::Range, time::Duration};

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

pub(crate) fn fixture() -> (tempfile::TempDir, GraphInputs) {
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
        _tree_lease: None,
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

#[test]
fn retired_or_expired_file_reads_do_not_poison_a_later_frontier() {
    let (_dir, inputs) = fixture();
    let mut worker = CallGraphWorker::new(inputs);
    for (cancel, deadline) in [
        (
            AtomicBool::new(true),
            Instant::now() + Duration::from_secs(1),
        ),
        (AtomicBool::new(false), Instant::now()),
    ] {
        assert!(worker.load("b.rs", &cancel, deadline).is_err());
        assert!(!worker.files.contains_key("b.rs"));
        assert!(worker.summaries.is_empty());
    }
    let result = worker
        .frontier_in_source(0, &AtomicBool::new(false))
        .unwrap();
    assert_eq!(result.annotation().label(), "1");
    assert!(worker.files["b.rs"].is_ok());
}

#[test]
fn swift_fixture_resolves_cross_file_bodies_cache_and_ordinary_marks() {
    let owner = crate::semantic::test_support::scip_rust_fixture();
    let buffer = Arc::new(
        SourceBuffer::from_bytes(
            "src/lib.rs",
            std::fs::read(owner.path().join("src/lib.rs")).unwrap(),
            None,
        )
        .unwrap(),
    );
    let index = SemanticIndex::from_scip(
        ScipIndex::decode(&std::fs::read(owner.path().join("index.scip")).unwrap()).unwrap(),
    );
    let mut inputs = GraphInputs {
        source: SourceRead::Checkout(owner.path().into()),
        calls: Arc::new(DocumentSyntax::analyze(&buffer).calls),
        buffer,
        index: Arc::new(index),
        overrides: Default::default(),
    };
    let cancel = AtomicBool::new(false);
    let mut worker = CallGraphWorker::new(inputs.clone());
    let root = worker.frontier_in_source(0, &cancel).unwrap();
    assert_eq!(root.status_line(), "1 direct · complete");
    assert_eq!(root.summary.calls.len(), 1);
    let area = &root.summary.calls[0];
    assert_eq!(area.display_name, "area");
    assert_eq!(
        area.classification.reason,
        crate::critical::CriticalityReason::RepositoryDefinition
    );
    assert_eq!(area.definition.as_ref().unwrap().path, "src/shapes.rs");
    assert_eq!(area.site_count, 1);
    let frontier = worker.frontier_of(&area.callee, &cancel).unwrap();
    assert_eq!(
        frontier.summary.body.as_ref().unwrap().path,
        "src/shapes.rs"
    );
    assert!(frontier.summary.calls.is_empty());
    assert_eq!(frontier.annotation().label(), "0");
    let bytes = worker.source_bytes;
    let path = owner.path().join("src/shapes.rs");
    let original = std::fs::read(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(worker.frontier_of(&area.callee, &cancel).unwrap(), frontier);
    assert_eq!(worker.source_bytes, bytes);
    std::fs::write(path, original).unwrap();
    inputs
        .overrides
        .set_symbol(&area.callee.symbol, Some(Criticality::Ordinary));
    let mut marked = CallGraphWorker::new(inputs);
    let root = marked.frontier_in_source(0, &cancel).unwrap();
    assert!(root.paths.is_empty());
    assert_eq!(root.traversed_bodies, 1);
    assert_eq!(root.annotation().label(), "0");
    let external = marked
        .frontier_of(
            &SemanticSymbolID::new("rust-analyzer cargo core . option/Option#unwrap().", ""),
            &cancel,
        )
        .unwrap();
    assert_eq!(
        external.coverage,
        CallCoverage::NoBody(NoBodyReason::External)
    );
}

#[test]
fn same_length_callee_replacement_retires_dependent_summaries_with_the_worker() {
    let (owner, mut inputs) = fixture();
    let cancel = AtomicBool::new(false);
    let mut original = CallGraphWorker::new(inputs.clone());
    let before = original.frontier_in_source(0, &cancel).unwrap();
    assert_eq!(before.paths[0].call.callee.symbol, OPEN);
    let path = owner.path().join("b.rs");
    let old = std::fs::read_to_string(&path).unwrap();
    let edited = SourceBuffer::from_str("b.rs", &old.replace("open()", "read()"));
    assert_eq!(old.len(), edited.byte_len());
    assert_ne!(edited.content_digest(), inputs.index.source_digests["b.rs"]);
    std::fs::write(&path, &edited.text).unwrap();
    // A request keeps its own coherent file snapshot, even after disk replacement.
    let bytes = original.source_bytes;
    assert_eq!(original.frontier_in_source(0, &cancel).unwrap(), before);
    assert_eq!(original.source_bytes, bytes);
    // Retiring the request cannot turn the stale old index into a proven zero.
    let stale = CallGraphWorker::new(inputs.clone())
        .frontier_in_source(0, &cancel)
        .unwrap();
    assert_eq!(stale.annotation().label(), "0+");
    assert!(matches!(
        stale.coverage,
        CallCoverage::Partial(CoverageGaps {
            missing_bodies: 1,
            ..
        })
    ));
    let read = "rust-analyzer cargo std 0 fs/read().";
    let index = Arc::make_mut(&mut inputs.index);
    index.documents.get_mut("b.rs").unwrap().occurrences[1].symbol = read.into();
    index
        .source_digests
        .insert("b.rs".into(), edited.content_digest().into());
    let mut current = CallGraphWorker::new(inputs);
    let after = current.frontier_in_source(0, &cancel).unwrap();
    assert_eq!(after.summary, before.summary);
    assert_eq!(after.annotation().label(), "1");
    assert_eq!(after.paths[0].via[0].callee.symbol, HELPER);
    assert_eq!(after.paths[0].call.callee.symbol, read);
    let body = current
        .frontier_of(&SemanticSymbolID::new(HELPER, ""), &cancel)
        .unwrap();
    assert_eq!(
        body.summary.body.unwrap().content_digest,
        edited.content_digest()
    );
    std::fs::remove_file(path).unwrap();
    let bytes = current.source_bytes;
    assert_eq!(current.frontier_in_source(0, &cancel).unwrap(), after);
    assert_eq!(current.source_bytes, bytes);
}

#[test]
fn dynamic_and_unresolved_sites_remain_partial_with_an_explicit_empty_document() {
    let (_owner, mut inputs) = fixture();
    let buffer = Arc::new(SourceBuffer::from_str(
        "other.rs",
        "fn go(f: fn()) { (f)(); helper(); }\nfn helper() {}\n",
    ));
    inputs.calls = Arc::new(DocumentSyntax::analyze(&buffer).calls);
    inputs.index = Arc::new(SemanticIndex::from_scip(ScipIndex {
        documents: vec![crate::semantic::test_support::document(&buffer, vec![])],
        ..Default::default()
    }));
    inputs.buffer = buffer;
    let cancel = AtomicBool::new(false);
    let result = CallGraphWorker::new(inputs.clone())
        .frontier_in_source(0, &cancel)
        .unwrap();
    assert!(result.summary.calls.is_empty());
    assert!(result.summary.body.is_some());
    assert_eq!(
        result.coverage,
        CallCoverage::Partial(CoverageGaps {
            unresolved_sites: 2,
            ..Default::default()
        })
    );
    assert_eq!(result.annotation().label(), "0+");
    // Missing structure is different from absent indexed occurrences.
    let parsed = inputs.calls.clone();
    Arc::make_mut(&mut inputs.calls).error = Some("Structure not available".into());
    let result = CallGraphWorker::new(inputs.clone())
        .frontier_in_source(0, &cancel)
        .unwrap();
    assert_eq!(
        result.coverage,
        CallCoverage::NoBody(NoBodyReason::NoSyntax)
    );
    inputs.calls = parsed;
    assert_eq!(
        CallGraphWorker::new(inputs)
            .frontier_in_source(0, &cancel)
            .unwrap()
            .annotation()
            .label(),
        "0+"
    );
}
