use super::*;
use crate::{critical::Criticality, source::ReviewRevision, syntax::DocumentSyntax};
use std::time::{Duration, Instant};

fn request(
    lens: &mut CriticalLens,
    buffer: Arc<SourceBuffer>,
    index: Option<Arc<SemanticIndex>>,
    overrides: CriticalityOverrides,
) {
    let calls = Arc::new(DocumentSyntax::analyze(&buffer).calls);
    lens.start(buffer, calls, index, overrides);
    settle(lens);
}

fn settle(lens: &mut CriticalLens) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while lens.is_running() {
        assert!(lens.poll().is_none());
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(lens.error().is_none());
}

#[test]
fn revisits_and_mode_toggles_reuse_exact_facts_but_bytes_paths_revisions_and_syntax_do_not() {
    let mut lens = CriticalLens::default();
    let text = "fn f() { call(); }";
    for path in ["a.rs", "b.rs", "a.rs"] {
        request(
            &mut lens,
            Arc::new(SourceBuffer::from_str(path, text)),
            None,
            Default::default(),
        );
    }
    assert_eq!(lens.cache_counts(), (2, 1));
    lens.stop();
    assert!(lens
        .projection(&SourceBuffer::from_str("a.rs", text))
        .is_none());
    request(
        &mut lens,
        Arc::new(SourceBuffer::from_str("a.rs", text)),
        None,
        Default::default(),
    );
    assert_eq!(lens.cache_counts(), (2, 2));
    for revision in [Some(ReviewRevision::Before), Some(ReviewRevision::After)] {
        let buffer =
            Arc::new(SourceBuffer::from_bytes("a.rs", text.as_bytes().to_vec(), revision).unwrap());
        request(&mut lens, buffer, None, Default::default());
    }
    let changed = Arc::new(SourceBuffer::from_str("a.rs", "fn f() { next(); }"));
    request(&mut lens, changed.clone(), None, Default::default());
    let mut calls = DocumentSyntax::analyze(&changed).calls;
    calls.supported = false;
    lens.start(changed.clone(), Arc::new(calls), None, Default::default());
    settle(&mut lens);
    assert!(lens
        .projection(&changed)
        .unwrap()
        .receded_byte_ranges
        .is_empty());
    assert_eq!(lens.cache_counts(), (6, 2));
}

#[test]
fn index_and_policy_changes_invalidate_decisions_without_retaining_old_indexes() {
    let buffer = Arc::new(SourceBuffer::from_str(
        "a.rs",
        "fn clone() {} fn f() { clone(); }",
    ));
    let start = buffer.text.rfind("clone").unwrap();
    let symbol = "rust-analyzer cargo core 0 clone().";
    let index = Arc::new(crate::critical::test_support::index_ranges(
        &buffer,
        [(start..start + 5, symbol.into())],
    ));
    let weak = Arc::downgrade(&index);
    let mut lens = CriticalLens::default();
    for (decision, receded) in [
        (None, true),
        (Some(Criticality::Critical), false),
        (None, true),
    ] {
        let mut overrides = CriticalityOverrides::default();
        overrides.set_symbol(symbol, decision);
        request(&mut lens, buffer.clone(), Some(index.clone()), overrides);
        assert_eq!(lens.projection(&buffer).unwrap().is_receded(start), receded);
    }
    // Same bytes, policy and symbol, but a new index proves a repository definition.
    let mut replacement = (*index).clone();
    replacement.definitions.insert(
        crate::semantic::index::SemanticSymbolID::new(symbol, "a.rs"),
        vec![(
            "a.rs".into(),
            crate::semantic::ScipRange::from_packed(&[0, 3, 8]).unwrap(),
        )],
    );
    drop(index);
    assert!(
        weak.upgrade().is_none(),
        "cache must not retain the semantic index"
    );
    request(
        &mut lens,
        buffer.clone(),
        Some(Arc::new(replacement)),
        Default::default(),
    );
    assert!(!lens.projection(&buffer).unwrap().is_receded(start));
    assert_eq!(lens.cache_counts(), (4, 0));
    lens.clear();
    assert_eq!(lens.cache_counts(), (0, 0));
}

#[test]
fn bounded_lru_evicts_the_least_recent_document_and_skips_oversized_facts() {
    let mut lens = CriticalLens::default();
    for n in 0..8 {
        request(
            &mut lens,
            Arc::new(SourceBuffer::from_str(format!("{n}.rs"), "fn f() {}")),
            None,
            Default::default(),
        );
    }
    for (path, counts) in [
        ("0.rs", (8, 1)),
        ("8.rs", (9, 1)),
        ("0.rs", (9, 2)),
        ("1.rs", (10, 2)),
    ] {
        request(
            &mut lens,
            Arc::new(SourceBuffer::from_str(path, "fn f() {}")),
            None,
            Default::default(),
        );
        assert_eq!(lens.cache_counts(), counts);
    }
    let buffer = Arc::new(SourceBuffer::from_str("large.rs", "fn f() {}"));
    let mut calls = DocumentSyntax::analyze(&buffer).calls;
    calls.error = Some("x".repeat(17 * 1024 * 1024));
    let calls = Arc::new(calls);
    for _ in 0..2 {
        lens.start(buffer.clone(), calls.clone(), None, Default::default());
        settle(&mut lens);
    }
    assert_eq!(lens.cache_counts(), (12, 2));
    drop(calls);
    lens.clear();
    for path in ["large-a.rs", "large-b.rs", "large-b.rs", "large-a.rs"] {
        let buffer = Arc::new(SourceBuffer::from_str(path, "fn f() {}"));
        let mut calls = DocumentSyntax::analyze(&buffer).calls;
        calls.error = Some("x".repeat(9 * 1024 * 1024));
        lens.start(buffer, Arc::new(calls), None, Default::default());
        settle(&mut lens);
    }
    assert_eq!(lens.cache_counts(), (3, 1)); // byte budget evicts before the entry cap
}

#[test]
fn cancelled_lock_wait_cannot_reconfigure_or_publish_a_retired_request() {
    let mut lens = CriticalLens::default();
    let cache = lens.cache.clone();
    let held = cache.lock().unwrap();
    let index = Arc::new(SemanticIndex::default());
    let weak = Arc::downgrade(&index);
    let buffer = Arc::new(SourceBuffer::from_str("a.rs", "fn f() {}"));
    lens.start(
        buffer.clone(),
        Arc::new(DocumentSyntax::analyze(&buffer).calls),
        Some(index),
        Default::default(),
    );
    lens.stop();
    let deadline = Instant::now() + Duration::from_secs(5);
    while weak.upgrade().is_some() {
        assert!(
            Instant::now() < deadline,
            "cancelled cache waiter kept its index"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(lens.projection(&buffer).is_none());
    drop(held);
    assert_eq!(lens.cache_counts(), (0, 0));
    request(&mut lens, buffer, None, Default::default());
    assert_eq!(lens.cache_counts(), (1, 0));
}
