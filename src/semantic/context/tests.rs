use super::*;
use crate::semantic::{ScipDocument, ScipIndex, ScipOccurrence, ScipSymbolInformation};

fn site(
    buffer: &SourceBuffer,
    token: &str,
    last: bool,
    symbol: &str,
    definition: bool,
) -> ScipOccurrence {
    let byte = if last {
        buffer.text.rfind(token)
    } else {
        buffer.text.find(token)
    }
    .unwrap();
    super::super::test_support::occurrence(buffer, byte..byte + token.len(), symbol, definition)
}

fn document(buffer: &SourceBuffer, occurrences: Vec<ScipOccurrence>) -> ScipDocument {
    ScipDocument {
        relative_path: buffer.path.clone(),
        position_encoding: 2,
        occurrences,
        ..Default::default()
    }
}

fn prepare(buffer: &SourceBuffer, index: &SemanticIndex, byte: usize) -> UnderCaretContext {
    UnderCaretContext::prepare(buffer, index, byte, &AtomicBool::new(false))
        .unwrap()
        .unwrap()
}

#[test]
fn global_definitions_here_and_index_facts_have_independent_destinations() {
    let a = SourceBuffer::from_str(
        "a.rs",
        "fn target() {}\nfn main() { let emoji = \"😀\"; target(); }\n",
    );
    let b = SourceBuffer::from_str("b.rs", "fn target() {}\n");
    let symbol = "rust-analyzer cargo app 1 target().";
    let mut first = document(
        &a,
        vec![
            site(&a, "target", false, symbol, true),
            site(&a, "target", true, symbol, false),
        ],
    );
    first.symbols = vec![ScipSymbolInformation {
        symbol: symbol.into(),
        display_name: "target".into(),
        kind: 17,
        signature: "fn target()".into(),
        documentation: vec![
            "```rust\nfn target()\n```\n\nDoes useful work.\n\nOther paragraph.".into(),
        ],
        ..Default::default()
    }];
    let second = document(&b, vec![site(&b, "target", false, symbol, true)]);
    let index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![second, first],
        ..Default::default()
    });
    let use_context = prepare(&a, &index, a.text.rfind("target").unwrap());
    assert_eq!(
        (
            use_context.definition_count,
            use_context.reference_count,
            use_context.file_occurrence_count
        ),
        (2, 1, 2)
    );
    assert_eq!(use_context.destination.as_ref().unwrap().path, "a.rs");
    assert_eq!(
        (
            &*use_context.kind,
            &*use_context.signature,
            use_context.documentation.as_deref()
        ),
        ("Function", "fn target()", Some("Does useful work."))
    );
    assert!(!use_context.is_on_definition);
    let here = prepare(&a, &index, 4);
    assert!(here.is_on_definition);
    assert_eq!(here.destination.unwrap().path, "b.rs");
    assert!(here.external_package.is_none());
}

#[test]
fn locals_use_exact_scoped_unicode_occurrences_and_primed_definition_lines() {
    let a = SourceBuffer::from_str("a.rs", "fn main() { let café = \"😀\"; café; }\n");
    let b = SourceBuffer::from_str("b.rs", "let other = 1; other;\n");
    let mut first = document(
        &a,
        vec![
            site(&a, "café", false, "local 1", true),
            site(&a, "café", true, "local 1", false),
        ],
    );
    first.symbols.push(ScipSymbolInformation {
        symbol: "local 1".into(),
        kind: 61,
        enclosing_symbol: "main".into(),
        ..Default::default()
    });
    let mut index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![
            first,
            document(
                &b,
                vec![
                    site(&b, "other", false, "local 1", true),
                    site(&b, "other", true, "local 1", false),
                ],
            ),
        ],
        ..Default::default()
    });
    index.symbols.insert(
        SemanticSymbolID::new("main", "a.rs"),
        ScipSymbolInformation {
            display_name: "main".into(),
            ..Default::default()
        },
    );
    let context = prepare(&a, &index, a.text.rfind("café").unwrap() + 1);
    assert_eq!(
        (
            context.name.as_str(),
            context.kind.as_str(),
            context.file_occurrence_count
        ),
        ("café", "Variable", 2)
    );
    assert_eq!(context.id.document.as_deref(), Some("a.rs"));
    assert_eq!(context.enclosing_name.as_deref(), Some("main"));
    let line = context.local_definition_line.unwrap();
    assert_eq!(&line.text[line.emphasis.unwrap()], "café");
    assert!(context.external_package.is_none());
}

#[test]
fn no_repository_definition_retains_package_facts_without_inventing_a_destination() {
    let buffer = SourceBuffer::from_str("a.rs", "target();");
    let symbol = "rust-analyzer cargo dependency 2.3 target().";
    let index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![document(
            &buffer,
            vec![site(&buffer, "target", false, symbol, false)],
        )],
        ..Default::default()
    });
    let context = prepare(&buffer, &index, 1);
    assert_eq!(context.external_package.as_deref(), Some("dependency 2.3"));
    assert_eq!(context.definition_count, 0);
    assert!(context.destination.is_none());
    assert_eq!(context.name, "target");
}

#[test]
fn import_before_stale_multiline_and_cancelled_contexts_do_not_publish_facts() {
    let mut buffer = SourceBuffer::from_str("a.rs", "target();\n");
    let mut index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![document(
            &buffer,
            vec![site(&buffer, "target", false, "target", false)],
        )],
        ..Default::default()
    });
    let cancelled = AtomicBool::new(false);
    index.documents.get_mut("a.rs").unwrap().occurrences[0].symbol_roles =
        crate::semantic::scip_proto::SCIP_ROLE_IMPORT;
    assert!(UnderCaretContext::prepare(&buffer, &index, 1, &cancelled)
        .unwrap()
        .is_none());
    index.documents.get_mut("a.rs").unwrap().occurrences[0].symbol_roles = 0;
    buffer.revision = Some(ReviewRevision::Before);
    assert!(UnderCaretContext::prepare(&buffer, &index, 1, &cancelled)
        .unwrap()
        .is_none());
    buffer.revision = None;
    index.source_digests.insert("a.rs".into(), "changed".into());
    assert!(UnderCaretContext::prepare(&buffer, &index, 1, &cancelled)
        .unwrap()
        .is_none());
    index.source_digests.clear();
    index.documents.get_mut("a.rs").unwrap().occurrences[0].range =
        ScipRange::from_packed(&[0, 0, 1, 0]);
    assert!(UnderCaretContext::prepare(&buffer, &index, 1, &cancelled)
        .unwrap()
        .is_none());
    assert!(UnderCaretContext::prepare(&buffer, &index, 1, &AtomicBool::new(true)).is_err());
}

#[test]
fn documentation_and_signature_bounds_preserve_unicode_and_known_kind_names() {
    let paragraph = first_paragraph(&[
        "```rust\nignored\n```".into(),
        format!("{}\n\nLater", "é".repeat(500)),
    ])
    .unwrap();
    assert_eq!(paragraph.chars().count(), 280);
    assert!(paragraph.ends_with('…'));
    let signature = bounded_signature(&format!("fn café({})", "é".repeat(500)));
    assert!(signature.len() <= 600 && signature.is_char_boundary(signature.len()));
    assert_eq!(bounded_signature(&"line\r\n".repeat(12)).lines().count(), 8);
    assert_eq!(kind_name(84), "Extension");
    assert_eq!(kind_name(83), "Kind 83");
    assert_eq!(kind_name(-1), "Kind -1");
}
