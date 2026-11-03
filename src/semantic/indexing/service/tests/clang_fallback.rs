//! Whole-service defaults, cache provenance, real-database transitions and PR input scope.
use super::*;

#[test]
fn clang_fallback_full_document_coverage_stays_partial_after_reuse_and_reload() {
    let fixture = Fixture::new(&[("main.c", "int main(void) { return 0; }\n")]);
    let observed = fixture.bin.join("database.json");
    fixture.tool(
        Indexer::Clang,
        &["main.c"],
        &format!("/bin/cp \"$database\" {}", quote(&observed)),
    );
    let generated = run(fixture.request());
    assert!(generated.index.is_some(), "{:?}", generated.report);
    let outcome = &generated.report.outcomes[0];
    assert_eq!(outcome.missing_count, 0);
    assert!(outcome.strategy.is_best_effort() && generated.report.is_partial());
    let database: serde_json::Value = serde_json::from_slice(&fs::read(observed).unwrap()).unwrap();
    assert_eq!(database[0]["arguments"][3], "-std=gnu11");
    let copied = Path::new(database[0]["directory"].as_str().unwrap());
    assert!(copied.starts_with(&fixture.cache) && !copied.exists());
    let args = fixture.bin.join("scip-clang.args");
    let arguments = fs::read_to_string(&args).unwrap();
    assert!(arguments.contains("--jobs\n4\n") && !arguments.contains("--no-progress-report"));
    let loaded = super::super::super::cache::load(&fixture.request(), &AtomicBool::new(false))
        .unwrap()
        .unwrap();
    assert!(loaded.report.summary().starts_with("Partial cached index"));
    fs::remove_file(&args).unwrap();
    let mut reuse = fixture.request();
    reuse.reuse = true;
    let reused = run(reuse);
    assert!(reused.report.is_partial());
    assert!(reused.report.outcomes[0].reused_from.is_some());
    assert!(
        !args.exists(),
        "Exact project reuse must not invoke the compiler"
    );
    assert_eq!(fs::read_dir(&fixture.source).unwrap().count(), 1);
}

#[test]
fn clang_fallback_database_add_remove_changes_strategy_and_prevents_incompatible_reuse() {
    let fixture = Fixture::new(&[("main.cpp", "int main() {}\n")]);
    fixture.tool(Indexer::Clang, &["main.cpp"], "");
    let fallback = run(fixture.request());
    let strategy = &fallback.report.outcomes[0].strategy;
    assert!(strategy.is_best_effort());
    let database = fixture.source.join("compile_commands.json");
    fs::write(
        &database,
        r#"[{"directory":".","file":"main.cpp","arguments":["/usr/bin/clang++","-c","main.cpp"]}]"#,
    )
    .unwrap();
    let mut request = fixture.request();
    request.reuse = true;
    let configured = run(request.clone());
    assert!(configured.index.is_some());
    assert!(!configured.report.outcomes[0].strategy.is_best_effort());
    assert!(configured.report.outcomes[0].reused_from.is_none());
    assert!(!configured.report.is_partial());
    assert_ne!(strategy, &configured.report.outcomes[0].strategy);
    fs::remove_file(database).unwrap();
    let fallback_again = run(request);
    assert!(fallback_again.report.is_partial());
    assert!(fallback_again.report.outcomes[0].strategy.is_best_effort());
    assert!(fallback_again.report.outcomes[0].reused_from.is_none());
}

#[test]
fn clang_errors_with_usable_output_cannot_claim_full_configured_coverage() {
    for message in [
        "sample.c:1: error: missing declaration",
        "num errored TUs: 2",
    ] {
        let fixture = Fixture::new(&[
            ("main.c", "int main(void) {}\n"),
            (
                "compile_commands.json",
                r#"[{"directory":".","file":"main.c","arguments":["/usr/bin/clang","-c","main.c"]}]"#,
            ),
        ]);
        fixture.tool(
            Indexer::Clang,
            &["main.c"],
            &format!("printf '%s\\n' '{}' >&2", message),
        );
        let generated = run(fixture.request());
        assert!(generated.index.is_some());
        assert_eq!(generated.report.outcomes[0].missing_count, 0);
        assert!(generated.report.outcomes[0].partial_build && generated.report.is_partial());
    }
    for good in ["num errored TUs: 0", "num errored TUs: 000", "no errors"] {
        assert!(!clang_reported_errors(good));
    }
    assert!(clang_reported_errors(
        "num errored TUs: 00000000000000000000001"
    ));
}

#[test]
fn clang_fallback_review_does_not_borrow_an_ignored_checkout_compilation_database() {
    use crate::pull_request::review::tests::git;
    let (owner, root, initial) = crate::pull_request::review::tests::fixture();
    git(&root, &["restore", "."]);
    git(&root, &["checkout", "review"]);
    fs::write(root.join("boot.cpp"), "int boot() { return 1; }\n").unwrap();
    git(&root, &["add", "boot.cpp"]);
    git(&root, &["commit", "-m", "C++ inputs"]);
    let mut summary = initial.summary.clone();
    summary.head_ref_oid = Some(git(&root, &["rev-parse", "HEAD"]));
    git(&root, &["checkout", "main"]);
    fs::write(root.join(".git/info/exclude"), "compile_commands.json\n").unwrap();
    fs::write(
        root.join("compile_commands.json"),
        "checkout-only poison database",
    )
    .unwrap();
    let status = git(&root, &["status", "--porcelain"]);
    let review = Arc::new(
        crate::pull_request::review::ReviewSnapshot::prepare(
            &root,
            summary,
            &owner.path().join("review-cache"),
        )
        .unwrap(),
    );
    let fixture = Fixture::new(&[]);
    fixture.tool(Indexer::Clang, &["boot.cpp"], "");
    let mut request = fixture.request();
    request.root = review.head_root.clone();
    request.attached_root = root.clone();
    request.review = Some(review);
    let generated = run(request.clone());
    assert!(generated.report.outcomes[0].strategy.is_best_effort());
    assert!(generated
        .index
        .unwrap()
        .matches_buffer(&SourceBuffer::from_str(
            "boot.cpp",
            "int boot() { return 1; }\n"
        )));
    assert_eq!(generated.report.revision.as_deref(), request.revision());
    assert_eq!(git(&root, &["status", "--porcelain"]), status);
    assert_eq!(
        fs::read_to_string(root.join("compile_commands.json")).unwrap(),
        "checkout-only poison database"
    );
}
