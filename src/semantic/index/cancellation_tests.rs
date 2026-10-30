use super::*;
use crate::semantic::test_support::cancel_at;
use std::sync::atomic::{AtomicUsize, Ordering};

const GLOBAL: &str = "scip cargo fixture 1 work().";

fn fixture() -> ScipIndex {
    let mut occurrences = (0..128)
        .map(|line| ScipOccurrence {
            symbol: GLOBAL.into(),
            symbol_roles: i32::from(line == 0),
            range: ScipRange::from_packed(&[line, 3, 7]),
            enclosing_range: (line == 0)
                .then(|| ScipRange::from_packed(&[0, 0, 1, 0]))
                .flatten(),
        })
        .collect::<Vec<_>>();
    let local = ScipOccurrence {
        symbol: "local 1".into(),
        symbol_roles: 1,
        range: ScipRange::from_packed(&[0, 1, 2]),
        enclosing_range: None,
    };
    occurrences.push(local.clone());
    ScipIndex {
        text_document_encoding: 2,
        documents: vec![
            ScipDocument {
                relative_path: "a.rs".into(),
                occurrences,
                symbols: vec![ScipSymbolInformation {
                    symbol: GLOBAL.into(),
                    display_name: "café".into(),
                    signature: "fn café()".into(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            ScipDocument {
                relative_path: "b.rs".into(),
                occurrences: vec![
                    local,
                    ScipOccurrence {
                        symbol: GLOBAL.into(),
                        symbol_roles: 0,
                        range: ScipRange::from_packed(&[0, 3, 7]),
                        enclosing_range: None,
                    },
                ],
                ..Default::default()
            },
        ],
        external_symbols: vec![
            ScipSymbolInformation {
                symbol: GLOBAL.into(),
                ..Default::default()
            },
            ScipSymbolInformation {
                symbol: "local 99".into(),
                ..Default::default()
            },
        ],
    }
}

#[test]
fn interrupted_map_construction_never_returns_partial_indexes_and_fresh_builds_keep_semantics() {
    let scip = fixture();
    let checks = AtomicUsize::new(0);
    SemanticIndex::from_scip_cancellable(scip.clone(), cancel_at(&checks, usize::MAX)).unwrap();
    let total = checks.load(Ordering::Relaxed);
    assert!(
        total > 128,
        "checkpoints must reach occurrence and derived-map processing"
    );
    for stop in 1..=total {
        let checks = AtomicUsize::new(0);
        assert_eq!(
            SemanticIndex::from_scip_cancellable(scip.clone(), cancel_at(&checks, stop))
                .unwrap_err(),
            ProtobufError::Cancelled
        );
        assert_eq!(checks.load(Ordering::Relaxed), stop);
    }
    let fresh = SemanticIndex::from_scip(scip);
    let global = SemanticSymbolID::new(GLOBAL, "");
    assert_eq!(fresh.default_encoding, 2);
    assert_eq!(fresh.documents.len(), 2);
    assert_eq!(fresh.find_definition_for_id(&global).unwrap().0, "a.rs");
    assert_eq!(fresh.find_references_for_id(&global).len(), 128);
    assert_eq!(fresh.symbols[&global].signature, "fn café()");
    for path in ["a.rs", "b.rs"] {
        assert_eq!(
            fresh.definitions[&SemanticSymbolID::new("local 1", path)].len(),
            1
        );
    }
    assert!(!fresh.symbols.keys().any(|id| id.symbol == "local 99"));
    assert_eq!(fresh.search_entries.len(), 1);
    assert_eq!(fresh.search_entries[0].name, "café");
    assert_eq!(
        fresh.definition_extents[&global].1,
        ScipRange::from_packed(&[0, 0, 1, 0]).unwrap()
    );
    assert!(fresh.repository_packages.contains("fixture"));
    println!("SCIP map construction: all {total} retirement checkpoints abort; fresh scoped definitions/references/search recover");
}
