use super::*;
use crate::{
    semantic::{
        reference_results::{query, ReferenceLimits},
        ScipDocument, ScipIndex, ScipOccurrence, ScipRange, SemanticIndex,
    },
    source::loader::SourceRead,
};

const NAMES: &[&str] = &[
    "café",
    "middle",
    "left",
    "right",
    "entry",
    "callbacks",
    "nested",
    "inner",
    "launch",
    "relay",
    "other",
];

fn fixture(texts: &[(&str, &str)]) -> (tempfile::TempDir, SourceQueryInputs) {
    let root = tempfile::tempdir().unwrap();
    let mut documents = Vec::new();
    let mut buffers = Vec::new();
    for &(path, text) in texts {
        std::fs::write(root.path().join(path), text).unwrap();
        let buffer = Arc::new(SourceBuffer::from_str(path, text));
        let mut occurrences = Vec::new();
        for (line, text) in text.lines().enumerate() {
            for name in NAMES {
                for (byte, _) in text.match_indices(name) {
                    let column = text[..byte].encode_utf16().count() as i32;
                    occurrences.push(ScipOccurrence {
                        symbol: format!("test cargo fixture 1 {name}()."),
                        symbol_roles: i32::from(text[..byte].ends_with("fn ")),
                        range: ScipRange::from_packed(&[
                            line as i32,
                            column,
                            column + name.encode_utf16().count() as i32,
                        ]),
                        ..Default::default()
                    });
                }
            }
        }
        documents.push(ScipDocument {
            relative_path: path.into(),
            position_encoding: 2,
            occurrences,
            ..Default::default()
        });
        buffers.push(buffer);
    }
    let mut index = SemanticIndex::from_scip(ScipIndex {
        documents,
        ..Default::default()
    });
    for buffer in &buffers {
        index
            .source_digests
            .insert(buffer.path.clone(), buffer.content_digest().into());
    }
    let inputs = SourceQueryInputs {
        source: SourceRead::Checkout(root.path().into()),
        buffer: buffers.remove(0),
        index: Arc::new(index),
    };
    (root, inputs)
}

fn branches() -> (tempfile::TempDir, SourceQueryInputs) {
    fixture(&[
        ("leaf.rs", "fn café() {}\nfn middle() { let s = \"🚀\"; café(); }\n"),
        ("entry.rs", "fn left() { middle(); }\nfn right() { middle(); }\nfn entry() { left(); right(); }\n"),
        ("boundary.rs", "fn callbacks() { let f = middle; let c = || middle(); }\nfn nested() { fn inner() { middle(); } inner(); }\n"),
    ])
}

fn selected(inputs: &SourceQueryInputs, name: &str, definition: bool) -> (ReferenceHit, String) {
    let results = query(
        inputs,
        &SemanticSymbolID::new(
            format!("test cargo fixture 1 {name}()."),
            &inputs.buffer.path,
        ),
        ReferenceLimits::default(),
        &AtomicBool::new(false),
    )
    .unwrap();
    let hit = results
        .hits
        .iter()
        .find(|hit| hit.is_definition == definition)
        .unwrap()
        .clone();
    let digest = results.source_digest(&hit.path).unwrap().to_owned();
    (hit, digest)
}

fn run(inputs: &SourceQueryInputs, hit: &ReferenceHit, digest: &str, rows: usize) -> CallerTree {
    trace(
        inputs,
        hit,
        digest,
        CallerLimits {
            rows,
            ..Default::default()
        },
        &AtomicBool::new(false),
    )
    .unwrap()
}

#[test]
fn parsed_branches_match_swift_and_keep_callback_closure_and_nested_boundaries() {
    let (_root, inputs) = branches();
    let (hit, digest) = selected(&inputs, "café", false);
    let tree = run(&inputs, &hit, &digest, 256);
    assert_eq!(
        tree.rows
            .iter()
            .map(|row| (row.name.as_str(), row.depth))
            .collect::<Vec<_>>(),
        [
            ("middle", 0),
            ("callbacks", 1),
            ("Closure in callbacks", 1),
            ("inner", 1),
            ("nested", 2),
            ("left", 1),
            ("entry", 2),
            ("right", 1),
            ("entry", 2)
        ]
    );
    assert_eq!(
        tree.rows[1].note.as_deref(),
        Some("Non-call reference; invocation unknown")
    );
    assert_eq!(
        tree.rows[2].note.as_deref(),
        Some("Closure invocation is not indexed")
    );
    assert_eq!(tree.rows[4].note.as_deref(), Some("No indexed callers"));
    assert_eq!(
        tree.rows[0].location.byte_offset,
        inputs.buffer.text.find("café();").unwrap()
    );
    assert!(tree.notices.is_empty() && !tree.has_more);
    assert_eq!(tree.source_identities.len(), 3);
}

#[test]
fn row_limit_expansion_preserves_the_dfs_prefix() {
    let (_root, inputs) = branches();
    let (hit, digest) = selected(&inputs, "café", false);
    let short = run(&inputs, &hit, &digest, 3);
    let all = run(&inputs, &hit, &digest, 512);
    assert_eq!(short.rows, all.rows[..3]);
    assert!(short.has_more && !all.has_more);
    for rows in 1..all.rows.len() {
        assert_eq!(run(&inputs, &hit, &digest, rows).rows, all.rows[..rows]);
    }
    let limits = CallerLimits {
        rows: usize::MAX,
        ..Default::default()
    }
    .expanded();
    assert_eq!(limits.rows, usize::MAX);
}

#[test]
fn recursion_stops_only_the_current_path_and_preserves_other_entry_branches() {
    let (_root, inputs) = fixture(&[("cycle.rs", "fn café() {}\nfn launch() { café(); relay(); }\nfn relay() { launch(); }\nfn entry() { relay(); }\nfn other() { launch(); }\n")]);
    let (hit, digest) = selected(&inputs, "café", true);
    let tree = run(&inputs, &hit, &digest, 256);
    assert_eq!(
        tree.rows
            .iter()
            .map(|row| (row.name.as_str(), row.depth))
            .collect::<Vec<_>>(),
        [
            ("café", 0),
            ("launch", 1),
            ("relay", 2),
            ("entry", 3),
            ("other", 2)
        ]
    );
    assert_eq!(
        tree.rows[2].note.as_deref(),
        Some("Recursive branch stopped")
    );
    let prefix = run(&inputs, &hit, &digest, 3);
    assert_eq!(prefix.rows, tree.rows[..3]);
    assert!(prefix.has_more && !tree.has_more);
}

#[test]
fn missing_changed_and_invalid_sites_leave_coverage_not_fabricated_edges() {
    let (root, mut inputs) = branches();
    let (hit, digest) = selected(&inputs, "café", false);
    std::fs::remove_file(root.path().join("entry.rs")).unwrap();
    std::fs::write(root.path().join("boundary.rs"), "fn callbacks() {}\n").unwrap();
    let index = Arc::make_mut(&mut inputs.index);
    index
        .references
        .get_mut(&SemanticSymbolID::new(
            "test cargo fixture 1 middle().",
            "leaf.rs",
        ))
        .unwrap()
        .push((
            "leaf.rs".into(),
            ScipRange::from_packed(&[99, 0, 6]).unwrap(),
        ));
    let tree = run(&inputs, &hit, &digest, 256);
    assert_eq!(tree.rows.len(), 1);
    assert_eq!(
        tree.rows[0].note.as_deref(),
        Some("Caller coverage incomplete")
    );
    assert!(tree
        .notices
        .iter()
        .any(|notice| notice.starts_with("entry.rs:")));
    assert!(tree
        .notices
        .contains(&"boundary.rs: stale semantic source identity".into()));
    assert!(tree
        .notices
        .contains(&"leaf.rs: stale index coordinates".into()));
}

#[test]
fn initial_identity_revision_and_cancellation_are_checked_before_publication() {
    let (_root, mut inputs) = branches();
    let (hit, digest) = selected(&inputs, "café", false);
    for (expected_digest, cancelled, message) in [
        ("different", false, "changed"),
        (digest.as_str(), true, "cancelled"),
    ] {
        assert!(trace(
            &inputs,
            &hit,
            expected_digest,
            CallerLimits::default(),
            &AtomicBool::new(cancelled)
        )
        .unwrap_err()
        .contains(message));
    }
    Arc::make_mut(&mut inputs.buffer).revision = Some(ReviewRevision::Before);
    assert!(trace(
        &inputs,
        &hit,
        &digest,
        CallerLimits::default(),
        &AtomicBool::new(false)
    )
    .unwrap_err()
    .contains("Before"));
}

#[test]
fn source_budget_is_explicit_and_displayed_bytes_are_primed() {
    let (root, inputs) = branches();
    let (hit, digest) = selected(&inputs, "café", false);
    std::fs::remove_file(root.path().join("leaf.rs")).unwrap();
    let tree = trace(
        &inputs,
        &hit,
        &digest,
        CallerLimits {
            files: 1,
            ..Default::default()
        },
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(tree.has_more);
    assert_eq!(tree.rows.len(), 1);
    assert!(tree.notices.iter().any(|notice| notice.contains("budget")));
    assert_eq!(run(&inputs, &hit, &digest, 256).rows.len(), 9);
}

#[test]
fn unindexed_callable_and_top_level_do_not_borrow_a_preceding_definition() {
    let (_root, mut inputs) = fixture(&[(
        "a.rs",
        "fn café() {}\nfn middle() { café(); }\nconst CALLBACK: fn() = café;\n",
    )]);
    let (hit, digest) = selected(&inputs, "café", true);
    let index = Arc::make_mut(&mut inputs.index);
    index
        .documents
        .get_mut("a.rs")
        .unwrap()
        .occurrences
        .retain(|occ| !occ.symbol.ends_with("middle()."));
    let tree = run(&inputs, &hit, &digest, 256);
    assert_eq!(
        tree.rows
            .iter()
            .map(|row| row.name.as_str())
            .collect::<Vec<_>>(),
        ["café", "middle", "Top level"]
    );
    assert_eq!(
        tree.rows[1].note.as_deref(),
        Some("Callable is not indexed")
    );
    assert_eq!(
        tree.rows[2].note.as_deref(),
        Some("Non-call reference; invocation unknown")
    );
}

#[test]
fn identical_offsets_in_distinct_documents_keep_document_local_owners_separate() {
    let (_root, mut inputs) = fixture(&[
        ("leaf.rs", "fn café() {}\n"),
        (
            "a.rs",
            "fn middle() { café(); }\nfn entry() { middle(); }\n",
        ),
        (
            "b.rs",
            "fn middle() { café(); }\nfn other() { middle(); }\n",
        ),
    ]);
    let mut documents = inputs.index.documents.values().cloned().collect::<Vec<_>>();
    for document in &mut documents {
        for occurrence in &mut document.occurrences {
            if occurrence.symbol.ends_with("middle().") {
                occurrence.symbol = "local 1".into();
            }
        }
    }
    let mut index = SemanticIndex::from_scip(ScipIndex {
        documents,
        ..Default::default()
    });
    index.source_digests = inputs.index.source_digests.clone();
    inputs.index = Arc::new(index);
    let (hit, digest) = selected(&inputs, "café", true);
    let tree = run(&inputs, &hit, &digest, 256);
    let labels = tree
        .rows
        .iter()
        .map(|row| (row.name.as_str(), row.location.path.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        labels,
        [
            ("café", "leaf.rs"),
            ("middle", "a.rs"),
            ("entry", "a.rs"),
            ("middle", "b.rs"),
            ("other", "b.rs")
        ]
    );
    assert_ne!(tree.rows[1].symbol, tree.rows[3].symbol);
    assert!(!tree.has_more);
}

#[test]
fn traversal_can_exceed_legacy_depth_limits_without_recursive_stack_calls() {
    let text = format!(
        "fn café() {{}}\n{}",
        (0..300)
            .map(|index| {
                let previous = if index == 0 {
                    "café".into()
                } else {
                    format!("caller{}", index - 1)
                };
                format!("fn caller{index}() {{ {previous}(); }}\n")
            })
            .collect::<String>()
    );
    let (_root, mut inputs) = fixture(&[("chain.rs", &text)]);
    let buffer = &inputs.buffer;
    let mut document = inputs.index.documents["chain.rs"].clone();
    for (line, text) in text.lines().enumerate() {
        for word in text.split(|ch: char| !ch.is_alphanumeric()) {
            if !word.starts_with("caller") {
                continue;
            }
            for (byte, _) in text.match_indices(word) {
                let column = text[..byte].encode_utf16().count() as i32;
                document.occurrences.push(ScipOccurrence {
                    symbol: format!("test cargo fixture 1 {word}()."),
                    symbol_roles: i32::from(text[..byte].ends_with("fn ")),
                    range: ScipRange::from_packed(&[
                        line as i32,
                        column,
                        column + word.len() as i32,
                    ]),
                    ..Default::default()
                });
            }
        }
    }
    let mut index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![document],
        ..Default::default()
    });
    index
        .source_digests
        .insert(buffer.path.clone(), buffer.content_digest().into());
    inputs.index = Arc::new(index);
    let (hit, digest) = selected(&inputs, "café", true);
    let limited = run(&inputs, &hit, &digest, 256);
    assert!(limited.has_more && limited.rows.len() == 256);
    let all = run(&inputs, &hit, &digest, 512);
    assert_eq!(all.rows.len(), 301);
    assert_eq!(all.rows.last().unwrap().depth, 300);
    assert_eq!(limited.rows, all.rows[..256]);
    assert!(!all.has_more);
}
