use super::*;
use crate::semantic::{ScipDocument, ScipIndex, ScipOccurrence, ScipRange};

#[test]
fn marking_requires_exact_global_semantics_and_uses_cached_package_ownership() {
    let text = "fn f() { call(); }";
    let buffer = SourceBuffer::from_str("a.rs", text);
    let column = text.find("call").unwrap();
    let symbol = "rust-analyzer cargo domain 1 call().";
    let mut index = SemanticIndex::from_scip(ScipIndex {
        documents: vec![ScipDocument {
            relative_path: "a.rs".into(),
            occurrences: vec![ScipOccurrence {
                symbol: symbol.into(),
                symbol_roles: 1,
                range: Some(ScipRange {
                    start_line: 0,
                    end_line: 0,
                    start_col: column as i32,
                    end_col: (column + 4) as i32,
                }),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    });
    assert!(index.repository_packages.contains("domain"));
    let mut overrides = CriticalityOverrides::default();
    overrides.set_package("rust-analyzer cargo domain", Some(Criticality::Ordinary));
    let menu = MarkContext {
        index: &index,
        overrides: &overrides,
    }
    .at_byte(&buffer, column)
    .unwrap();
    assert_eq!(menu.name, "call");
    assert_eq!(
        menu.package_key.as_deref(),
        Some("rust-analyzer cargo domain")
    );
    assert_eq!(menu.built_in.criticality, Criticality::Critical);
    assert_eq!(menu.package_decision, Some(Criticality::Ordinary));
    let before =
        SourceBuffer::from_bytes("a.rs", buffer.data.clone(), Some(ReviewRevision::Before))
            .unwrap();
    assert!(MarkContext {
        index: &index,
        overrides: &overrides
    }
    .at_byte(&before, column)
    .is_none());
    index.source_digests.insert("a.rs".into(), "changed".into());
    assert!(MarkContext {
        index: &index,
        overrides: &overrides
    }
    .at_byte(&buffer, column)
    .is_none());
    index.source_digests.clear();
    index.documents.get_mut("a.rs").unwrap().occurrences[0].symbol = "local 7".into();
    assert!(MarkContext {
        index: &index,
        overrides: &overrides
    }
    .at_byte(&buffer, column)
    .is_none());
}
