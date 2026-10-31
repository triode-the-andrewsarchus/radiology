use super::*;

#[test]
fn rust_feature_cfgs_use_isolated_nested_manifests_without_leaking_sibling_projects() {
    let fixture = Fixture::new(&[
        ("rust/Cargo.toml", "[package]\nname='fixture'\n[features] # real TOML comments\ndefault=[]\napp-runtime=[]\n'quoted.feature'=[]\n"),
        ("rust/lib.rs", "a"),
        ("rust/member/Cargo.toml", "[features]\nembedding_gate=[]\napp-runtime=[]\n[dependencies]\nnot_a_feature='1'\n"),
        ("rust/unused/Cargo.toml", "not a valid manifest"),
        ("rust-extra/Cargo.toml", "[features]\nforeign_feature=[]\n"),
        ("rust-extra/lib.rs", "b"),
    ]);
    let configs = fixture.bin.join("configs");
    fs::create_dir(&configs).unwrap();
    fixture.tool(
        Indexer::Rust,
        &["lib.rs"],
        &format!(
            "/bin/cp \"$configuration\" {}/\"$(/usr/bin/basename \"$PWD\")\".json",
            quote(&configs)
        ),
    );
    let generated = run(fixture.request());
    assert_eq!(generated.report.documents, 2, "{:?}", generated.report);
    for (project, features) in [
        (
            "rust",
            vec![
                "feature=app-runtime",
                "feature=embedding_gate",
                "feature=quoted.feature",
            ],
        ),
        ("rust-extra", vec!["feature=foreign_feature"]),
    ] {
        let configuration: serde_json::Value =
            serde_json::from_slice(&fs::read(configs.join(format!("{project}.json"))).unwrap())
                .unwrap();
        let expected: Vec<_> = ["debug_assertions", "miri"]
            .into_iter()
            .chain(features)
            .collect();
        assert_eq!(configuration["cargo"]["cfgs"], serde_json::json!(expected));
        assert_eq!(configuration["cargo"]["features"], "all");
        assert_eq!(configuration["cargo"]["buildScripts"]["enable"], false);
        assert_eq!(configuration["procMacro"]["enable"], false);
    }
    assert!(!fixture.source.join("rust/rust-analyzer.json").exists());
    assert_eq!(
        fs::read_to_string(fixture.source.join("rust/lib.rs")).unwrap(),
        "a"
    );
}

#[test]
fn oversized_rust_manifests_fail_before_indexing_and_internal_manifest_links_work() {
    let fixture = Fixture::new(&[("Cargo.toml", "[features]\n"), ("lib.rs", "a")]);
    let manifest = fixture.source.join("Cargo.toml");
    let file = fs::OpenOptions::new().write(true).open(&manifest).unwrap();
    file.set_len(4 * 1024 * 1024 + 1).unwrap();
    fixture.tool(Indexer::Rust, &["lib.rs"], "");
    let generated = run(fixture.request());
    assert!(generated.report.outcomes[0]
        .failure
        .as_deref()
        .unwrap()
        .contains("4 MiB"));
    assert!(!fixture.bin.join("rust-analyzer.args").exists());
    fs::remove_file(&manifest).unwrap();
    fs::write(
        fixture.source.join("real-manifest.toml"),
        "[features]\nlinked=[]\n",
    )
    .unwrap();
    std::os::unix::fs::symlink("real-manifest.toml", &manifest).unwrap();
    let config = fixture.bin.join("linked.json");
    fixture.tool(
        Indexer::Rust,
        &["lib.rs"],
        &format!("/bin/cp \"$configuration\" {}", quote(&config)),
    );
    assert_eq!(run(fixture.request()).report.documents, 1);
    let configuration: serde_json::Value =
        serde_json::from_slice(&fs::read(config).unwrap()).unwrap();
    assert_eq!(
        configuration["cargo"]["cfgs"],
        serde_json::json!(["debug_assertions", "miri", "feature=linked"])
    );
    assert_eq!(
        fs::read_link(manifest).unwrap(),
        Path::new("real-manifest.toml")
    );
}
