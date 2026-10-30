use super::*;
use crate::source::SourceBuffer;
use std::{
    cell::Cell,
    time::{Duration, Instant},
};

fn large_rust() -> SourceBuffer {
    SourceBuffer::from_str(
        "large.rs",
        &"fn work() { let value = 1; work(); }\n".repeat(20_000),
    )
}

fn stop_at(checks: &Cell<usize>, step: usize) -> impl Fn() -> bool + '_ {
    move || {
        checks.set(checks.get() + 1);
        checks.get() == step
    }
}

#[test]
fn parser_progress_interrupts_native_work_and_latches_a_stop_pulse() {
    let buffer = large_rust();
    let checks = Cell::new(0);
    let cancelled = stop_at(&checks, 8);
    let control = AnalysisControl::new(&cancelled);
    let started = Instant::now();
    assert!(SupportedGrammar::Rust
        .parse_cancellable(&buffer.data, &control)
        .unwrap_err()
        .contains("cancelled"));
    // Only the initial guard runs before parsing. Eight calls therefore prove
    // interruption from the native progress callback, rather than pre-retirement.
    assert_eq!(checks.get(), 8);
    assert!(control.check().is_err());
    assert_eq!(
        checks.get(),
        8,
        "stop stays latched without rechecking the predicate"
    );
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(!SupportedGrammar::Rust
        .parse(b"fn retry() {}\n")
        .unwrap()
        .root_node()
        .has_error());
    println!(
        "native parser: interrupted after {} guard callbacks; fresh parse succeeds",
        checks.get()
    );
}

#[test]
fn query_progress_interrupts_native_scanning_even_without_any_matches() {
    let buffer = large_rust();
    let tree = SupportedGrammar::Rust.parse(&buffer.data).unwrap();
    let query = tree_sitter::Query::new(
        &SupportedGrammar::Rust.language(),
        "(string_literal) @string",
    )
    .unwrap();
    let checks = Cell::new(0);
    let visited = Cell::new(0);
    let cancelled = stop_at(&checks, 8);
    let control = AnalysisControl::new(&cancelled);
    let started = Instant::now();
    let result = queries::visit_matches(
        &query,
        tree.root_node(),
        &buffer.data,
        &control,
        "Absent strings",
        |_| {
            visited.set(visited.get() + 1);
            Ok(())
        },
    );
    assert!(result.unwrap_err().contains("cancelled"));
    assert_eq!(visited.get(), 0);
    assert_eq!(
        checks.get(),
        8,
        "progress must run while native matching finds no captures"
    );
    assert!(started.elapsed() < Duration::from_secs(2));
    println!(
        "native no-match query: interrupted after {} guard callbacks with zero captures",
        checks.get()
    );
}

#[test]
fn cancellation_at_derived_fact_stages_never_publishes_a_fallback_document() {
    for (path, text, name) in [
        ("a.rs", "fn work() { let value = 1; work(); }\n", "work"),
        (
            "a.swift",
            "func work(value: Int) -> Int { return value + 1 }\n",
            "work",
        ),
        (
            "a.kt",
            "fun work(value: Int): Int { return value + 1 }\n",
            "work",
        ),
    ] {
        let buffer = SourceBuffer::from_str(path, text);
        let total = Cell::new(0);
        DocumentSyntax::analyze_cancellable(&buffer, || {
            total.set(total.get() + 1);
            false
        })
        .unwrap();
        for stop in [1, total.get() / 4, total.get() / 2, total.get() - 1] {
            let checks = Cell::new(0);
            let result = DocumentSyntax::analyze_cancellable(&buffer, stop_at(&checks, stop));
            assert!(
                result.err().unwrap().contains("cancelled"),
                "{path}: stop {stop}"
            );
        }
        let fresh = DocumentSyntax::analyze_cancellable(&buffer, || false).unwrap();
        assert!(fresh.folds_supported && fresh.calls.supported);
        assert_eq!(fresh.symbols[0].name, name);
        assert!(fresh.occurrences.error.is_none());
        assert!(fresh.native_highlights.error.is_none());
    }
    let plain = SourceBuffer::from_str("a.txt", "fn fallback() {}\n");
    assert!(DocumentSyntax::analyze_cancellable(&plain, || true).is_err());
    assert_eq!(DocumentSyntax::analyze(&plain).symbols[0].name, "fallback");
}

#[test]
fn outline_and_test_walkers_stop_between_parsed_nodes() {
    let buffer = SourceBuffer::from_str("a.rs", &"fn work() { work(); }\n".repeat(100));
    let tree = SupportedGrammar::Rust.parse(&buffer.data).unwrap();
    for tests in [false, true] {
        let checks = Cell::new(0);
        let cancelled = stop_at(&checks, 3);
        let control = AnalysisControl::new(&cancelled);
        if tests {
            let _ = TestRegionDetector::from_tree(
                &buffer,
                SupportedGrammar::Rust,
                tree.root_node(),
                &control,
            );
        } else {
            let _ = SymbolExtractor::from_tree(
                &buffer,
                SupportedGrammar::Rust,
                tree.root_node(),
                &control,
            );
        }
        assert!(control.check().is_err());
        assert_eq!(checks.get(), 3);
    }
}
