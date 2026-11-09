use super::*;

pub(super) fn project_cache(fixture: &Fixture) -> Vec<(PathBuf, Vec<u8>)> {
    let mut entries: Vec<_> = walkdir::WalkDir::new(fixture.cache.join("projects"))
        .into_iter()
        .map(Result::unwrap)
        .filter(|entry| {
            entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "json")
        })
        .map(|entry| {
            let path = entry.into_path();
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
    let first = run(fixture.request());
    let sources = first.index.unwrap().sources;
    let invocations: Vec<_> = first
        .report
        .outcomes
        .iter()
        .map(|outcome| {
            (
                outcome.command_line.clone(),
                outcome.working_directory.clone(),
            )
        })
        .collect();
    assert!(invocations
        .iter()
        .all(|(command, directory)| command.is_some()
            && directory
                .as_ref()
                .is_some_and(|directory| !directory.exists())));
    assert_eq!(sources.len(), 2);
    let accepted = project_cache(&fixture);
    let mut request = fixture.request();
    request.reuse = true;
    let reused = run(request.clone());
    assert_eq!(reused.index.as_ref().unwrap().sources, sources);
    assert!(reused
        .report
        .outcomes
        .iter()
        .all(|outcome| outcome.reused_from.as_deref() == Some("checkout")));
    assert!(!reused.report.is_partial());
    assert_eq!(
        reused
            .report
            .outcomes
            .iter()
            .map(|outcome| (
                outcome.command_line.clone(),
                outcome.working_directory.clone()
            ))
            .collect::<Vec<_>>(),
        invocations
    );
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
    let cached = load(&request).unwrap();
    assert_eq!(cached.report.reuse_warning(), mixed.report.reuse_warning());
    assert_eq!(
        cached
            .report
            .outcomes
            .iter()
            .map(|outcome| (&outcome.command_line, &outcome.working_directory))
            .collect::<Vec<_>>(),
        mixed
            .report
            .outcomes
            .iter()
            .map(|outcome| (&outcome.command_line, &outcome.working_directory))
            .collect::<Vec<_>>()
    );
    assert_eq!(cached.index.unwrap().sources, mixed.index.unwrap().sources);

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
    for (pointer, value) in [
        ("/source_digests/a.ts", serde_json::json!("../wrong")),
        (
            "/scip/documents/0/relative_path",
            serde_json::json!("../wrong"),
        ),
        ("/format", serde_json::json!(2)),
    ] {
        let mut corrupt = record.clone();
        *corrupt.pointer_mut(pointer).unwrap() = value;
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
    let staged = || {
        walkdir::WalkDir::new(fixture.cache.join("projects"))
            .into_iter()
            .map(Result::unwrap)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".radiology-index-stage-")
            })
            .count()
    };
    assert_eq!(staged(), 1); // first project is staged while the second is live
    generator.stop();
    let deadline = Instant::now() + Duration::from_secs(5);
    while staged() != 0 {
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
    let cached = load(&request).unwrap();
    assert_eq!(
        cached.report.outcomes[0].reused_from.as_deref(),
        Some(origin.as_str())
    );
    assert!(cached.report.stale_reason.is_none());
}

#[test]
fn explicit_checkout_reuse_publishes_owned_records_and_survives_source_removal() {
    let source = Fixture::new(&[("package.json", "{}"), ("a.ts", "new"), ("b.ts", "other")]);
    source.tool(Indexer::TypeScript, &["a.ts", "b.ts"], "");
    run(source.request());
    let accepted = project_cache(&source);
    let target = Fixture::new(&[("package.json", "{}"), ("a.ts", "old"), ("b.ts", "other")]);
    target.tool(Indexer::TypeScript, &["a.ts", "b.ts"], "");
    run(target.request()); // An incompatible local record must be replaced.
    fs::write(target.source.join("a.ts"), "new").unwrap();
    let ran = target.bin.join("unexpected-run");
    target.tool(
        Indexer::TypeScript,
        &["a.ts", "b.ts"],
        &format!("printf ran > {}; exit 9", quote(&ran)),
    );
    let mut request = target.request();
    request.reuse = true;
    request.reuse_sources = vec![
        IndexReuseSource {
            root: source.source.clone(),
            cache: target.source.join("forbidden-cache"),
            label: "invalid source".into(),
        },
        IndexReuseSource {
            root: source.source.clone(),
            cache: source.cache.clone(),
            label: "other checkout".into(),
        },
    ];
    let imported = run(request.clone());
    assert!(imported
        .index
        .as_ref()
        .unwrap()
        .matches_buffer(&SourceBuffer::from_str("a.ts", "new")));
    assert_eq!(
        imported.report.outcomes[0].reused_from.as_deref(),
        Some("other checkout")
    );
    assert!(imported.report.outcomes[0].failure.is_none());
    assert!(!ran.exists() && !target.source.join("forbidden-cache").exists());
    assert_eq!(project_cache(&source), accepted);
    let destination = project_cache(&target);
    assert_eq!(destination.len(), 1);
    assert_eq!(imported.index.as_ref().unwrap().documents.len(), 2);
    let record: serde_json::Value = serde_json::from_slice(&destination[0].1).unwrap();
    assert_eq!(
        record["root"],
        fs::canonicalize(&target.source).unwrap().to_str().unwrap()
    );
    fs::remove_dir_all(&source.cache).unwrap();
    request.reuse_sources.clear();
    let local = run(request.clone());
    assert!(local.report.outcomes[0].failure.is_none());
    assert_eq!(
        local.report.outcomes[0].reused_from.as_deref(),
        Some("other checkout")
    );
    assert!(!ran.exists());
    assert_eq!(project_cache(&target), destination);
    let cached = load(&request).unwrap();
    assert!(cached.report.cached && cached.report.stale_reason.is_none());
}

#[test]
fn explicit_sources_do_not_bypass_full_context_or_forced_regeneration() {
    let files = [
        ("Cargo.toml", "[package]\nname='fixture'"),
        ("lib.rs", "a"),
        ("web/package.json", "{}"),
        ("web/a.ts", "b"),
        ("shared/include.h", "original"),
    ];
    let source = Fixture::new(&files);
    let target = Fixture::new(&files);
    for fixture in [&source, &target] {
        for (indexer, document) in [
            (Indexer::Rust, "lib.rs"),
            (Indexer::TypeScript, "a.ts"),
            (Indexer::Clang, "shared/include.h"),
        ] {
            fixture.tool(indexer, &[document], "");
        }
    }
    run(source.request());
    let mut request = target.request();
    request.reuse_sources.push(IndexReuseSource {
        root: source.source.clone(),
        cache: source.cache.clone(),
        label: "source".into(),
    });
    let tool = target.bin.join("rust-analyzer");
    fs::write(
        &tool,
        fs::read_to_string(&tool)
            .unwrap()
            .replace("fixture-1", "fixture-2"),
    )
    .unwrap();
    request.reuse = true;
    let mixed = run(request.clone());
    assert!(mixed
        .report
        .outcomes
        .iter()
        .find(|outcome| outcome.strategy.indexer == Indexer::Rust)
        .unwrap()
        .reused_from
        .is_none());
    assert_eq!(
        mixed
            .report
            .outcomes
            .iter()
            .find(|outcome| outcome.strategy.indexer == Indexer::TypeScript)
            .unwrap()
            .reused_from
            .as_deref(),
        Some("source")
    );
    assert!(mixed.report.reuse_warning().is_some() && mixed.report.is_partial());
    let cached = load(&request).unwrap();
    assert_eq!(cached.report.reuse_warning(), mixed.report.reuse_warning());
    let sources = std::mem::take(&mut request.reuse_sources);
    let local = run(request.clone());
    assert!(local
        .report
        .outcomes
        .iter()
        .all(|outcome| outcome.reused_from.is_some()));
    assert!(local.report.is_partial() && local.report.reuse_warning().is_some());
    request.reuse_sources = sources;
    for forced in [false, true] {
        request.reuse = !forced;
        fs::write(
            target.source.join("shared/include.h"),
            if forced { "original" } else { "changed" },
        )
        .unwrap();
        let generated = run(request.clone());
        assert_eq!(generated.report.outcomes.len(), 3);
        assert!(generated
            .report
            .outcomes
            .iter()
            .all(|outcome| outcome.failure.is_none() && outcome.reused_from.is_none()));
    }
}
