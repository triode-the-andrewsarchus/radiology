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
    let mut root = summary(
        "root",
        vec![
            call("a", O, true),
            call("b", O, true),
            call("direct", C, false),
            call("unknown", Criticality::Unknown, true),
            call("plumbing", O, false),
        ],
    );
    let graph = HashMap::from([
        (
            id("a"),
            summary(
                "a",
                vec![
                    call("root", O, true),
                    call("a", O, true),
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
                vec![
                    call("deep", C, false),
                    call("leaf", C, false),
                    call("unknown", Criticality::Unknown, true),
                ],
            ),
        ),
    ]);
    let mut loads = Vec::new();
    let result = build(
        root.clone(),
        Default::default(),
        &AtomicBool::new(false),
        |id| {
            loads.push(id.clone());
            Ok(graph[id].clone())
        },
    )
    .unwrap();
    assert_eq!(loads, vec![id("a"), id("b"), id("shared")]);
    assert_eq!(result.annotation().label(), "3+");
    assert_eq!(result.traversed_bodies, 3);
    assert_eq!(result.unknown.len(), 1);
    assert_eq!(
        result.coverage,
        CallCoverage::Partial(CoverageGaps {
            unknown_callees: 1,
            ..Default::default()
        })
    );
    assert_eq!(result.status_line(), "1 direct · 3 reachable · partial");
    assert_eq!(
        result
            .paths
            .iter()
            .map(|p| p.call.display_name.as_str())
            .collect::<Vec<_>>(),
        vec!["deep", "leaf"]
    );
    root.calls
        .retain(|call| call.classification.criticality != Criticality::Unknown);
    let nested_unknown = build(root, Default::default(), &AtomicBool::new(false), |id| {
        Ok(graph[id].clone())
    })
    .unwrap();
    assert_eq!(nested_unknown.coverage, result.coverage);
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
    assert_eq!(result.status_line(), "0 direct · partial");
    assert!(result.status_detail().contains("unknown callees"));
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
    let empty = build(
        summary("empty", vec![]),
        Default::default(),
        &AtomicBool::new(false),
        |_| panic!("empty"),
    )
    .unwrap();
    assert_eq!(
        empty.annotation(),
        CountAnnotation {
            count: 0,
            completeness: CountCompleteness::Proven
        }
    );
    assert_eq!(empty.status_line(), "0 direct · proven");
    assert!(empty.status_detail().contains("proven"));
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

#[test]
fn conditional_metadata_never_proves_zero_or_double_counts_prepared_gaps() {
    use crate::critical::graph::ConditionalCallable;
    use crate::{source::SourceBuffer, syntax::DocumentSyntax};
    let source =
        SourceBuffer::from_str("a.rs", "fn outer() { let c = || { first(); second(); }; }");
    let scope = DocumentSyntax::analyze(&source).calls.scopes[1].clone();
    for (coverage, expected) in [
        (CallCoverage::Complete, 2),
        (
            CallCoverage::Partial(CoverageGaps {
                conditional_sites: 2,
                ..Default::default()
            }),
            2,
        ),
        // A prepared summary can know about deeper sites beyond the immediate scopes.
        (
            CallCoverage::Partial(CoverageGaps {
                conditional_sites: 3,
                ..Default::default()
            }),
            3,
        ),
    ] {
        let mut conditional = summary("closure", vec![]);
        conditional.conditional.push(ConditionalCallable {
            scope: scope.clone(),
            site_count: 2,
        });
        conditional.coverage = coverage;
        let root = build(
            conditional.clone(),
            Default::default(),
            &AtomicBool::new(false),
            |_| panic!("conditional sites are not edges"),
        )
        .unwrap();
        assert_eq!(
            root.annotation(),
            CountAnnotation {
                count: 0,
                completeness: CountCompleteness::Partial
            }
        );
        assert!(root.summary.calls.is_empty() && root.paths.is_empty());
        assert_eq!(root.status_line(), "0 direct · 2 conditional · partial");
        assert!(root.status_detail().contains("closures"));
        let through = build(
            summary("outer", vec![call("closure", Criticality::Ordinary, true)]),
            Default::default(),
            &AtomicBool::new(false),
            |_| Ok(conditional.clone()),
        )
        .unwrap();
        for frontier in [root, through] {
            assert_eq!(
                frontier.coverage,
                CallCoverage::Partial(CoverageGaps {
                    conditional_sites: expected,
                    ..Default::default()
                })
            );
            assert_eq!(frontier.annotation().label(), "0+");
        }
    }
}

#[test]
fn depth_and_body_caps_keep_macro_gaps_and_expand_the_full_chain_only_when_allowed() {
    use Criticality::{Critical as C, Ordinary as O};
    let mut mac = call("mac", O, true);
    mac.is_macro = true;
    let mut root = summary("root", vec![call("a", O, true), mac]);
    root.coverage = CallCoverage::Partial(CoverageGaps {
        unresolved_sites: 1,
        macro_sites: 1,
        ..Default::default()
    });
    let graph = HashMap::from([
        (id("a"), summary("a", vec![call("b", O, true)])),
        (id("b"), summary("b", vec![call("c", O, true)])),
        (id("c"), summary("c", vec![call("z", C, true)])),
    ]);
    for (depth, bodies, truncated) in [(2, 48, true), (6, 1, true), (6, 48, false)] {
        let result = build(
            root.clone(),
            FrontierLimits { depth, bodies },
            &AtomicBool::new(false),
            |id| Ok(graph[id].clone()),
        )
        .unwrap();
        assert_eq!(
            result.coverage,
            CallCoverage::Partial(CoverageGaps {
                unresolved_sites: 1,
                macro_sites: 1,
                truncated,
                ..Default::default()
            })
        );
        if truncated {
            assert!(result.paths.is_empty());
            assert_eq!(result.status_line(), "0 direct · partial");
        } else {
            assert_eq!(result.paths.len(), 1);
            assert_eq!(
                result.paths[0]
                    .via
                    .iter()
                    .map(|step| step.display_name.as_str())
                    .collect::<Vec<_>>(),
                ["a", "b", "c"]
            );
            assert_eq!(result.traversed_bodies, 3);
            assert_eq!(result.annotation().label(), "1+");
        }
    }
}
