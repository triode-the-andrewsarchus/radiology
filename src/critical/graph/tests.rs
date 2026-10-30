use super::*;
use crate::critical::CriticalityOverrides;
use crate::syntax::DocumentSyntax;

fn indexed(buffer: &SourceBuffer, sites: &CallSites) -> SemanticIndex {
    super::super::test_support::index_ranges(
        buffer,
        sites.sites.iter().filter_map(|site| {
            let name = site.name.as_ref()?;
            let text = &buffer.text[name.clone()];
            (text != "unknown").then(|| {
                (
                    name.clone(),
                    format!("rust-analyzer cargo std 0 fs/{text}()."),
                )
            })
        }),
    )
}

fn summary(
    buffer: &SourceBuffer,
    sites: &CallSites,
    index: &SemanticIndex,
    scope: usize,
) -> CallableSummary {
    let policy = CriticalityPolicy::for_index(Some(index), CriticalityOverrides::default());
    summarize(
        buffer,
        sites,
        scope,
        None,
        Some(index),
        &policy,
        &AtomicBool::new(false),
    )
    .unwrap()
}

#[test]
fn direct_calls_deduplicate_exact_utf16_identities_and_exclude_nested_execution() {
    let buffer = SourceBuffer::from_str("a.rs", "fn outer() { let café = \"😀\"; open(); open(); let c = || { close(); let inner = || hidden(); }; fn nested() { nested_call(); } unknown(); log!(); (f)(); } fn next() { open(); }");
    let sites = DocumentSyntax::analyze(&buffer).calls;
    let index = indexed(&buffer, &sites);
    let result = summary(&buffer, &sites, &index, 0);
    assert_eq!(
        result
            .calls
            .iter()
            .map(|call| call.display_name.as_str())
            .collect::<Vec<_>>(),
        vec!["open", "log"]
    );
    assert_eq!(result.calls[0].site_count, 2);
    assert_eq!(
        result.calls[0].first_site_byte,
        buffer.text.find("open()").unwrap()
    );
    assert_eq!(
        result.calls[0].classification.criticality,
        Criticality::Critical
    );
    assert!(result.calls[1].is_macro);
    assert_eq!(result.conditional.len(), 2);
    assert!(result.conditional[0].scope.closure);
    assert_eq!(result.conditional[0].site_count, 1);
    assert!(!result.conditional[1].scope.closure);
    assert!(matches!(
        result.coverage,
        CallCoverage::Partial(CoverageGaps {
            unresolved_sites: 2,
            macro_sites: 1,
            conditional_sites: 3,
            parse_errors: false,
            ..
        })
    ));
    let body = result.body.unwrap();
    assert_eq!(body.content_digest, buffer.content_digest());
    assert!(!body.range.contains(&buffer.text.find("fn next").unwrap()));
}

#[test]
fn closure_and_nested_function_summaries_keep_their_own_sites() {
    let buffer = SourceBuffer::from_str(
        "a.rs",
        "fn outer() { let a = || { let b = || open(); }; fn nested() { close(); } }",
    );
    let sites = DocumentSyntax::analyze(&buffer).calls;
    let index = indexed(&buffer, &sites);
    let outer = summary(&buffer, &sites, &index, 0);
    assert!(outer.calls.is_empty());
    assert!(matches!(
        outer.coverage,
        CallCoverage::Partial(CoverageGaps {
            conditional_sites: 2,
            ..
        })
    ));
    for (scope, expected) in [(1, vec![]), (2, vec!["open"]), (3, vec!["close"])] {
        let result = summary(&buffer, &sites, &index, scope);
        assert_eq!(
            result
                .calls
                .iter()
                .map(|c| c.display_name.as_str())
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn abstract_stale_before_missing_encoding_and_parse_errors_never_prove_zero() {
    let buffer = SourceBuffer::from_str(
        "a.rs",
        "trait T { fn abstract_method(); } fn concrete() { open(); let broken = ; }",
    );
    let sites = DocumentSyntax::analyze(&buffer).calls;
    let mut index = indexed(&buffer, &sites);
    assert_eq!(
        summary(&buffer, &sites, &index, 0).coverage,
        CallCoverage::NoBody(NoBodyReason::Abstract)
    );
    assert!(matches!(
        summary(&buffer, &sites, &index, 1).coverage,
        CallCoverage::Partial(CoverageGaps {
            parse_errors: true,
            ..
        })
    ));
    index.source_digests.insert("a.rs".into(), "stale".into());
    assert_eq!(
        summary(&buffer, &sites, &index, 1).coverage,
        CallCoverage::NoBody(NoBodyReason::NotIndexed)
    );
    index.source_digests.clear();
    index.documents.get_mut("a.rs").unwrap().position_encoding = 99;
    assert_eq!(
        summary(&buffer, &sites, &index, 1).coverage,
        CallCoverage::NoBody(NoBodyReason::NotIndexed)
    );
    let before =
        SourceBuffer::from_bytes("a.rs", buffer.data.clone(), Some(ReviewRevision::Before))
            .unwrap();
    let before_sites = DocumentSyntax::analyze(&before).calls;
    assert_eq!(
        summary(&before, &before_sites, &index, 1).coverage,
        CallCoverage::NoBody(NoBodyReason::NotIndexed)
    );
    assert_eq!(
        summary(&before, &sites, &index, 1).coverage,
        CallCoverage::NoBody(NoBodyReason::NoSyntax)
    );
    let cancel = AtomicBool::new(true);
    assert!(summarize(
        &buffer,
        &sites,
        1,
        None,
        Some(&index),
        &CriticalityPolicy::for_index(Some(&index), Default::default()),
        &cancel
    )
    .is_err());
}
