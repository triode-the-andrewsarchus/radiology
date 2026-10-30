use super::*;
use crate::semantic::{ScipDocument, ScipIndex, ScipOccurrence, ScipRange};
use crate::syntax::DocumentSyntax;

const RUNTIME: &str = "rust-analyzer cargo core 0 clone().";
const SYSTEM: &str = "rust-analyzer cargo std 0 fs/open().";

fn index_for(buffer: &SourceBuffer, entries: &[(&str, &str)]) -> SemanticIndex {
    let occurrences = entries
        .iter()
        .map(|(spelling, symbol)| {
            let byte = buffer.text.find(spelling).unwrap();
            let column: usize = buffer.text[..byte].chars().map(char::len_utf16).sum();
            ScipOccurrence {
                symbol: (*symbol).into(),
                range: Some(ScipRange {
                    start_line: 0,
                    end_line: 0,
                    start_col: column as i32,
                    end_col: (column + spelling.chars().map(char::len_utf16).sum::<usize>()) as i32,
                }),
                ..Default::default()
            }
        })
        .collect();
    SemanticIndex::from_scip(ScipIndex {
        documents: vec![ScipDocument {
            relative_path: buffer.path.clone(),
            position_encoding: 2,
            occurrences,
            ..Default::default()
        }],
        ..Default::default()
    })
}

fn projection(
    buffer: &SourceBuffer,
    index: Option<&SemanticIndex>,
    overrides: CriticalityOverrides,
) -> SourceProjection {
    let syntax = DocumentSyntax::analyze(buffer);
    let policy = CriticalityPolicy::for_index(index, overrides);
    make(
        buffer,
        &syntax.calls,
        index,
        &policy,
        &AtomicBool::new(false),
    )
    .unwrap()
}

#[test]
fn call_projection_uses_encoded_identity_and_keeps_unknown_calls_prominent() {
    let buffer = SourceBuffer::from_str(
        "a.rs",
        "fn demo() { let café = \"😀\"; clone(); open(); mystery(); }",
    );
    let index = index_for(&buffer, &[("clone", RUNTIME), ("open", SYSTEM)]);
    let projected = projection(&buffer, Some(&index), Default::default());
    assert!(projected.matches(&buffer));
    assert!(projected.is_receded(buffer.text.find("clone").unwrap()));
    assert!(!projected.is_receded(buffer.text.find("open").unwrap()));
    assert!(!projected.is_receded(buffer.text.find("mystery").unwrap()));
    assert!(projected.is_receded(buffer.text.find("demo").unwrap()));
    assert!(projected.is_receded(buffer.text.find('😀').unwrap()));
    assert!(projected.is_receded(buffer.text.find("open()").unwrap() + 4));
    let unresolved = projection(&buffer, None, Default::default());
    assert!(!unresolved.is_receded(buffer.text.find("clone").unwrap())); // Spelling is not evidence.
}

#[test]
fn overrides_apply_by_identity_and_symbol_decisions_precede_package_decisions() {
    let buffer = SourceBuffer::from_str("a.rs", "fn f() { clone(); open(); }");
    let index = index_for(&buffer, &[("clone", RUNTIME), ("open", SYSTEM)]);
    let mut overrides = CriticalityOverrides::default();
    overrides.set_package("rust-analyzer cargo core", Some(Criticality::Critical));
    overrides.set_symbol(SYSTEM, Some(Criticality::Ordinary));
    let projected = projection(&buffer, Some(&index), overrides.clone());
    assert!(!projected.is_receded(buffer.text.find("clone").unwrap()));
    assert!(projected.is_receded(buffer.text.find("open").unwrap()));
    overrides.set_symbol(RUNTIME, Some(Criticality::Ordinary));
    assert!(
        projection(&buffer, Some(&index), overrides).is_receded(buffer.text.find("clone").unwrap())
    );
}

#[test]
fn stale_unsupported_and_before_semantics_never_demote_unresolved_call_names() {
    let buffer = SourceBuffer::from_str("a.rs", "fn f() { clone(); }");
    let mut index = index_for(&buffer, &[("clone", RUNTIME)]);
    index
        .source_digests
        .insert("a.rs".into(), "older-digest".into());
    assert!(!projection(&buffer, Some(&index), Default::default())
        .is_receded(buffer.text.find("clone").unwrap()));
    index.source_digests.clear();
    let before =
        SourceBuffer::from_bytes("a.rs", buffer.data.clone(), Some(ReviewRevision::Before))
            .unwrap();
    assert!(!projection(&before, Some(&index), Default::default())
        .is_receded(buffer.text.find("clone").unwrap()));
    index.documents.get_mut("a.rs").unwrap().position_encoding = 99;
    assert!(!projection(&buffer, Some(&index), Default::default())
        .is_receded(buffer.text.find("clone").unwrap()));
}

#[test]
fn exact_callee_occurrences_win_over_same_start_broader_candidates() {
    let buffer = SourceBuffer::from_str("a.rs", "fn f() { clone(); }");
    let mut index = index_for(&buffer, &[("clone", RUNTIME)]);
    let mut broader = index.documents["a.rs"].occurrences[0].clone();
    broader.range.as_mut().unwrap().end_col += 2;
    broader.symbol = "unknown-package callable".into();
    index
        .documents
        .get_mut("a.rs")
        .unwrap()
        .occurrences
        .insert(0, broader);
    assert!(projection(&buffer, Some(&index), Default::default())
        .is_receded(buffer.text.find("clone").unwrap()));
}

#[test]
fn projections_bind_to_content_path_revision_and_supported_call_coverage() {
    let buffer = SourceBuffer::from_str("a.rs", "fn f() { clone(); }");
    let projected = projection(&buffer, None, Default::default());
    assert!(!projected.matches(&SourceBuffer::from_str("b.rs", &buffer.text)));
    assert!(!projected.matches(&SourceBuffer::from_str(
        "a.rs",
        &buffer.text.replace("clone", "other")
    )));
    assert!(!projected.matches(
        &SourceBuffer::from_bytes("a.rs", buffer.data.clone(), Some(ReviewRevision::After))
            .unwrap()
    ));
    let stale_syntax =
        DocumentSyntax::analyze(&SourceBuffer::from_str("a.rs", "fn other() { other(); }"));
    assert!(make(
        &buffer,
        &stale_syntax.calls,
        None,
        &Default::default(),
        &AtomicBool::new(false)
    )
    .unwrap()
    .receded_byte_ranges
    .is_empty());
    let plain = SourceBuffer::from_str("notes.txt", "something()");
    assert!(projection(&plain, None, Default::default())
        .receded_byte_ranges
        .is_empty());
    let no_calls = SourceBuffer::from_str("a.rs", "fn f() { let a = 1; }");
    let projected = projection(&no_calls, None, Default::default());
    assert_eq!(
        projected.receded_byte_ranges,
        std::iter::once(0..no_calls.byte_len()).collect::<Vec<_>>()
    );
    assert!(make(
        &buffer,
        &DocumentSyntax::analyze(&buffer).calls,
        None,
        &Default::default(),
        &AtomicBool::new(true)
    )
    .is_err());
    let unicode = SourceBuffer::from_str("a.rs", "😀abc");
    let projected = SourceProjection::new(&unicode, vec![1..3, 4..5, 5..7]);
    assert_eq!(
        projected.receded_byte_ranges,
        std::iter::once(4..7).collect::<Vec<_>>()
    );
}
