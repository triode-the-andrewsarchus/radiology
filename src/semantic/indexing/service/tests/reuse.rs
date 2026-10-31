use super::*;

fn project_cache(fixture: &Fixture) -> Vec<(PathBuf, Vec<u8>)> {
    let mut entries: Vec<_> = fs::read_dir(fixture.cache.join("projects"))
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect();
    entries.sort();
    entries
}

#[test]
fn exact_projects_reuse_but_changed_tools_inputs_modes_and_reindex_execute_again() {
    let fixture = Fixture::new(&[
        ("Cargo.toml", "[package]\nname='fixture'"),
        ("lib.rs", "a"),
        ("web/package.json", "{}"),
        ("web/app.ts", "b"),
    ]);
    let runs = fixture.bin.join("runs");
    for (indexer, document) in [(Indexer::Rust, "lib.rs"), (Indexer::TypeScript, "app.ts")] {
        fixture.tool(
            indexer,
            &[document],
            &format!("printf '{}\\n' >> {}", indexer.id(), quote(&runs)),
        );
    }
    run(fixture.request());
    let accepted = project_cache(&fixture);
    let mut request = fixture.request();
    request.reuse = true;
    let reused = run(request.clone());
    assert!(reused
        .report
        .outcomes
        .iter()
        .all(|outcome| outcome.reused_from.as_deref() == Some("checkout")));
    assert!(!reused.report.is_partial());
    assert!(reused.report.summary().starts_with("Updated index"));
    assert_eq!(project_cache(&fixture), accepted);
    assert_eq!(fs::read_to_string(&runs).unwrap().lines().count(), 2);

    let tool = fixture.bin.join("rust-analyzer");
    fs::write(
        &tool,
        fs::read_to_string(&tool)
            .unwrap()
            .replace("fixture-1", "fixture-2"),
    )
    .unwrap();
    let mixed = run(request.clone());
    assert!(mixed.report.outcomes[0].reused_from.is_none());
    assert!(mixed.report.outcomes[1].reused_from.is_some());
    assert!(mixed.report.reuse_warning().is_some());
    assert!(mixed.report.is_partial());
    assert_eq!(fs::read_to_string(&runs).unwrap().lines().count(), 3);
    let cached = super::super::super::cache::load(&request, &AtomicBool::new(false))
        .unwrap()
        .unwrap();
    assert_eq!(cached.report.reuse_warning(), mixed.report.reuse_warning());

    // A sibling's bytes can influence either project, so both must regenerate.
    for (expected_runs, edit_bytes) in [(5, true), (7, false)] {
        if edit_bytes {
            fs::write(fixture.source.join("web/app.ts"), "c").unwrap();
        } else {
            fs::set_permissions(
                fixture.source.join("lib.rs"),
                fs::Permissions::from_mode(0o755),
            )
            .unwrap();
        }
        let changed = run(request.clone());
        assert!(changed
            .report
            .outcomes
            .iter()
            .all(|outcome| outcome.reused_from.is_none()));
        assert_eq!(
            fs::read_to_string(&runs).unwrap().lines().count(),
            expected_runs
        );
    }
    request.reuse = false;
    let forced = run(request);
    assert!(forced
        .report
        .outcomes
        .iter()
        .all(|outcome| outcome.reused_from.is_none()));
    assert_eq!(fs::read_to_string(&runs).unwrap().lines().count(), 9);
    assert_eq!(project_cache(&fixture).len(), 2); // latest records, no revision/version growth
}

#[test]
fn corrupt_project_records_regenerate_and_missing_tools_keep_accepted_records() {
    let files = [("package.json", "{}"), ("a.ts", "a"), ("b.ts", "b")];
    let fixture = Fixture::new(&files);
    fixture.tool(Indexer::TypeScript, &["a.ts"], "");
    run(fixture.request());
    let mut request = fixture.request();
    request.reuse = true;
    let reused = run(request.clone());
    assert_eq!(reused.report.outcomes[0].missing_paths, ["b.ts"]);
    assert!(reused.report.is_partial());
    let (path, pristine) = project_cache(&fixture).pop().unwrap();
    let record: serde_json::Value = serde_json::from_slice(&pristine).unwrap();
    for pointer in ["/source_digests/a.ts", "/scip/documents/0/relative_path"] {
        let mut corrupt = record.clone();
        *corrupt.pointer_mut(pointer).unwrap() = serde_json::json!("../wrong");
        fs::write(&path, serde_json::to_vec(&corrupt).unwrap()).unwrap();
        let recovered = run(request.clone());
        assert!(recovered.report.outcomes[0].reused_from.is_none());
        assert!(recovered.report.outcomes[0].failure.is_none());
        assert!(recovered.report.cache_warning.is_some());
    }
    let other = Fixture::new(&files);
    let mut other_request = request.clone();
    other_request.root = other.source.clone();
    other_request.attached_root = other.source.clone();
    let separate = run(other_request);
    assert!(separate.report.outcomes[0].reused_from.is_none());
    let accepted = project_cache(&fixture);
    fs::remove_file(fixture.bin.join("scip-typescript")).unwrap();
    let missing = run(request);
    assert!(missing.index.is_none());
    assert!(missing.report.outcomes[0].failure.is_some());
    assert_eq!(project_cache(&fixture), accepted);
}

#[test]
fn project_cache_symlink_cannot_write_into_viewed_inputs() {
    let fixture = Fixture::new(&[("package.json", "{}"), ("a.ts", "a")]);
    fs::create_dir(&fixture.cache).unwrap();
    std::os::unix::fs::symlink(&fixture.source, fixture.cache.join("projects")).unwrap();
    fixture.tool(Indexer::TypeScript, &["a.ts"], "");
    let generated = run(fixture.request());
    assert!(generated.index.is_some());
    assert!(generated
        .report
        .cache_warning
        .as_deref()
        .unwrap()
        .contains("escapes"));
    assert_eq!(
        InputInventory::discover(&fixture.source, &AtomicBool::new(false))
            .unwrap()
            .paths,
        ["a.ts", "package.json"]
    );
}

#[test]
fn cancelled_generation_discards_staged_projects_and_preserves_accepted_records() {
    let fixture = Fixture::new(&[
        ("Cargo.toml", "[package]\nname='fixture'"),
        ("lib.rs", "a"),
        ("web/package.json", "{}"),
        ("web/a.ts", "b"),
    ]);
    fixture.tool(Indexer::Rust, &["lib.rs"], "");
    fixture.tool(Indexer::TypeScript, &["a.ts"], "");
    run(fixture.request());
    let accepted = project_cache(&fixture);
    let ready = fixture.bin.join("ready");
    fixture.tool(
        Indexer::TypeScript,
        &["a.ts"],
        &format!("printf ready > {}; /bin/sleep 30", quote(&ready)),
    );
    let mut generator = IndexGenerator::default();
    generator.start(fixture.request());
    wait_for(&ready);
    assert_eq!(
        fs::read_dir(fixture.cache.join("projects"))
            .unwrap()
            .count(),
        3
    ); // two accepted, one staged
    generator.stop();
    let deadline = Instant::now() + Duration::from_secs(5);
    while fs::read_dir(fixture.cache.join("projects"))
        .unwrap()
        .count()
        != 2
    {
        assert!(Instant::now() < deadline, "staged project was not retired");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(project_cache(&fixture), accepted);
    assert!(generator.poll().is_none());
}

#[test]
fn pinned_records_reuse_across_commits_with_equal_bytes_and_stable_attached_identity() {
    let fixture = Fixture::new(&[]);
    use crate::pull_request::review::tests::git;
    let (owner, root, mut request) = pinned_request(&fixture);
    let runs = fixture.bin.join("runs");
    fixture.tool(
        Indexer::Rust,
        &["a file.rs"],
        &format!("printf 'run\\n' >> {}", quote(&runs)),
    );
    git(&root, &["checkout", "-f", "review"]);
    let mut checkout = request.clone();
    checkout.review = None;
    checkout.root = root.clone();
    run(checkout);
    request.reuse = true;
    let from_checkout = run(request.clone());
    assert_eq!(
        from_checkout.report.outcomes[0].reused_from.as_deref(),
        Some("checkout")
    );
    request.reuse = false;
    let first = run(request.clone());
    assert!(first.report.cache_warning.is_none());
    let origin = request.revision().unwrap().to_owned();
    // Prepare a real new commit/tree through the production snapshot lifecycle.
    git(&root, &["commit", "--allow-empty", "-m", "same inputs"]);
    let mut summary = request.review.as_ref().unwrap().summary.clone();
    summary.head_ref_oid = Some(git(&root, &["rev-parse", "HEAD"]));
    request.review = Some(Arc::new(
        crate::pull_request::review::ReviewSnapshot::prepare(
            &root,
            summary,
            &owner.path().join("cache"),
        )
        .unwrap(),
    ));
    request.root = request.review.as_ref().unwrap().head_root.clone();
    request.reuse = true;
    let reused = run(request.clone());
    assert_ne!(reused.report.revision.as_deref(), Some(origin.as_str()));
    assert_eq!(
        reused.report.outcomes[0].reused_from.as_deref(),
        Some(origin.as_str())
    );
    assert!(reused.report.cache_warning.is_none());
    assert_eq!(fs::read_to_string(&runs).unwrap(), "run\nrun\n");
    let cached = super::super::super::cache::load(&request, &AtomicBool::new(false))
        .unwrap()
        .unwrap();
    assert_eq!(
        cached.report.outcomes[0].reused_from.as_deref(),
        Some(origin.as_str())
    );
    assert!(cached.report.stale_reason.is_none());
}
