use super::*;
use crate::critical::graph::NoBodyReason;
use crate::critical::{CriticalityClassification, CriticalityReason};
use crate::semantic::ScipRange;
use std::collections::HashMap;

fn id(name: &str) -> SemanticSymbolID {
    SemanticSymbolID::new(name, "a.rs")
}

fn call(name: &str, criticality: Criticality, repository: bool) -> DirectCall {
    DirectCall {
        callee: id(name),
        display_name: name.into(),
        classification: CriticalityClassification {
            criticality,
            reason: CriticalityReason::External,
        },
        first_site_byte: 0,
        site_count: 1,
        is_macro: false,
        definition: repository.then(|| CallDefinition {
            path: "a.rs".into(),
            range: ScipRange::from_packed(&[0, 0, 1]).unwrap(),
        }),
    }
}

fn summary(name: &str, calls: Vec<DirectCall>) -> CallableSummary {
    CallableSummary {
        callable: Some(id(name)),
        body: None,
        calls,
        conditional: Vec::new(),
        coverage: CallCoverage::Complete,
    }
}

#[test]
fn shortest_paths_are_stable_and_cycles_diamonds_and_direct_calls_do_not_inflate_counts() {
    use Criticality::{Critical as C, Ordinary as O};
    let root = summary(
        "root",
        vec![
            call("a", O, true),
            call("b", O, true),
            call("direct", C, false),
        ],
    );
    let graph = HashMap::from([
        (
            id("a"),
            summary(
                "a",
                vec![
                    call("root", O, true),
                    call("shared", O, true),
                    call("direct", C, false),
                ],
            ),
        ),
        (
            id("b"),
            summary("b", vec![call("shared", O, true), call("deep", C, false)]),
        ),
        (
            id("shared"),
            summary(
                "shared",
                vec![call("deep", C, false), call("leaf", C, false)],
            ),
        ),
    ]);
    let mut loads = Vec::new();
    let result = build(root, Default::default(), &AtomicBool::new(false), |id| {
        loads.push(id.clone());
        Ok(graph[id].clone())
    })
    .unwrap();
    assert_eq!(loads, vec![id("a"), id("b"), id("shared")]);
    assert_eq!(result.annotation().label(), "3");
    assert_eq!(
        result
            .paths
            .iter()
            .map(|p| p.call.display_name.as_str())
            .collect::<Vec<_>>(),
        vec!["deep", "leaf"]
    );
    assert_eq!(result.paths[0].via[0].display_name, "b");
    assert_eq!(
        result.paths[1]
            .via
            .iter()
            .map(|s| s.display_name.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "shared"]
    );
}

#[test]
fn unknowns_are_distinct_leaves_and_external_plumbing_does_not_request_a_body() {
    let root = summary(
        "root",
        vec![
            call("unknown", Criticality::Unknown, true),
            call("external", Criticality::Ordinary, false),
        ],
    );
    let result = build(root, Default::default(), &AtomicBool::new(false), |_| {
        panic!("leaves must not expand")
    })
    .unwrap();
    assert_eq!(result.annotation().label(), "0+");
    assert_eq!(result.unknown.len(), 1);
    assert!(matches!(
        result.coverage,
        CallCoverage::Partial(CoverageGaps {
            unknown_callees: 1,
            ..
        })
    ));
    assert!(result.paths.is_empty());
}

#[test]
fn bounds_missing_bodies_and_conditional_only_calls_cannot_prove_zero() {
    let root = summary("root", vec![call("a", Criticality::Ordinary, true)]);
    for limits in [
        FrontierLimits {
            depth: 1,
            bodies: 48,
        },
        FrontierLimits {
            depth: 4,
            bodies: 0,
        },
    ] {
        let result = build(root.clone(), limits, &AtomicBool::new(false), |_| {
            panic!("bounded")
        })
        .unwrap();
        assert_eq!(result.annotation().label(), "0+");
        assert!(matches!(
            result.coverage,
            CallCoverage::Partial(CoverageGaps {
                truncated: true,
                ..
            })
        ));
    }
    let result = build(root, Default::default(), &AtomicBool::new(false), |id| {
        Ok(CallableSummary::no_body(
            NoBodyReason::Abstract,
            Some(id.clone()),
        ))
    })
    .unwrap();
    assert!(matches!(
        result.coverage,
        CallCoverage::Partial(CoverageGaps {
            missing_bodies: 1,
            ..
        })
    ));
    let mut conditional = summary("conditional", vec![]);
    conditional.coverage = CallCoverage::Partial(CoverageGaps {
        conditional_sites: 2,
        ..Default::default()
    });
    let result = build(
        conditional,
        Default::default(),
        &AtomicBool::new(false),
        |_| panic!("no unconditional edges"),
    )
    .unwrap();
    assert_eq!(result.annotation().label(), "0+");
    let no_body = CallableSummary::no_body(NoBodyReason::External, Some(id("external")));
    assert_eq!(
        build(
            no_body,
            Default::default(),
            &AtomicBool::new(false),
            |_| panic!()
        )
        .unwrap()
        .annotation()
        .label(),
        "—"
    );
}

#[test]
fn cancellation_and_provider_failure_do_not_publish_partial_success() {
    let root = summary("root", vec![call("a", Criticality::Ordinary, true)]);
    let cancel = AtomicBool::new(false);
    assert!(build(root.clone(), Default::default(), &cancel, |_| {
        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(summary("a", vec![]))
    })
    .is_err());
    assert!(build(root.clone(), Default::default(), &cancel, |_| panic!(
        "cancelled"
    ))
    .is_err());
    assert!(
        build(root, Default::default(), &AtomicBool::new(false), |_| Err(
            "provider failed".into()
        ))
        .unwrap_err()
        .contains("provider failed")
    );
}
