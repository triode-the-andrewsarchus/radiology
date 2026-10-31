use super::*;
use command::CommandPaths;

#[test]
fn multilingual_plan_preserves_evidence_counts_and_registry_order() {
    let plan = IndexPlan::inspect([
        "Cargo.toml",
        "src/main.rs",
        "go.mod",
        "pkg/main.go",
        "tsconfig.json",
        "src/a.mts",
        "package.json",
        "pyproject.toml",
        "pkg/main.py",
        "types/api.pyi",
        "build.gradle.kts",
        "settings.gradle.kts",
        "src/App.kt",
        "src/App.java",
        "build/compile_commands.json",
        "native/api.cpp",
    ]);
    assert!(plan.skipped.is_empty());
    assert_eq!(
        plan.selected.iter().map(|s| s.indexer).collect::<Vec<_>>(),
        [
            Indexer::Rust,
            Indexer::Go,
            Indexer::TypeScript,
            Indexer::Python,
            Indexer::Clang,
            Indexer::Kotlin
        ]
    );
    assert_eq!(
        plan.selected
            .iter()
            .map(|s| s.source_files)
            .collect::<Vec<_>>(),
        [1, 1, 1, 2, 1, 2]
    );
    assert_eq!(plan.selected[2].evidence, ["tsconfig.json"]);
    assert_eq!(plan.selected[4].evidence, ["build/compile_commands.json"]);
    assert_eq!(
        plan.selected[5].evidence,
        ["build.gradle.kts", "settings.gradle.kts"]
    );
}

#[test]
fn nested_projects_and_sibling_prefixes_have_deterministic_boundaries() {
    let paths = [
        "py/child/pyproject.toml",
        "py/child/main.py",
        "py/requirements.txt",
        "py/peer/setup.py",
        "py2/setup.cfg",
        "py2/app.py",
        "web/tsconfig.json",
        "web/nested/tsconfig.json",
        "web/nested/package.json",
        "web/nested/app.ts",
        "web2/package.json",
        "web2/main.mjs",
        "web2/nested/package.json",
        "rust/Cargo.toml",
        "rust/member/Cargo.toml",
        "rust/member/lib.rs",
        "empty/Cargo.toml",
        "gradle/build.gradle",
        "gradle/sub/settings.gradle.kts",
        "gradle/sub/Main.java",
    ];
    let mut reversed = paths;
    reversed.reverse();
    let plan = IndexPlan::inspect(paths);
    assert_eq!(plan, IndexPlan::inspect(reversed));
    assert_eq!(
        plan.selected.iter().map(|s| s.id()).collect::<Vec<_>>(),
        [
            "rust-analyzer@rust",
            "scip-typescript@web",
            "scip-typescript@web2",
            "scip-python@py",
            "scip-python@py2",
            "scip-kotlin@gradle",
        ]
    );
    assert_eq!(plan.selected[3].evidence, ["py/requirements.txt"]);
}

#[test]
fn plan_cap_prefers_shallow_projects_and_retains_skipped_work() {
    let mut paths = vec!["deep/project/go.mod".into(), "deep/project/main.go".into()];
    for n in 0..40 {
        paths.extend([format!("p{n:02}/Cargo.toml"), format!("p{n:02}/main.rs")]);
    }
    let plan = IndexPlan::inspect(paths.iter().map(String::as_str));
    assert_eq!(plan.selected.len(), 32);
    assert_eq!(plan.skipped.len(), 9);
    assert_eq!(plan.selected[0].id(), "rust-analyzer@p00");
    assert_eq!(plan.skipped.last().unwrap().id(), "scip-go@deep/project");
    assert_eq!(
        IndexPlan::with_limit(paths.iter().map(String::as_str), 0)
            .skipped
            .len(),
        41
    );
}

#[test]
fn inventory_deduplicates_and_rejects_invalid_paths_without_inventing_strategies() {
    let plan = IndexPlan::inspect([
        "../Cargo.toml",
        "/go.mod",
        "src/main.rs",
        "package.json",
        "api/types.CTS",
        "api/types.CTS",
    ]);
    assert_eq!(plan.selected.len(), 1);
    assert_eq!(plan.selected[0].indexer, Indexer::TypeScript);
    assert_eq!(plan.selected[0].source_files, 1);
    assert_eq!(
        IndexPlan::inspect(["hello.swift", "orphan.rs", "README.md"])
            .selected
            .len(),
        0
    );
    assert_eq!(
        IndexPlan::inspect(["empty/tsconfig.json"]).selected[0].source_files,
        0
    );
}

#[test]
fn structured_commands_retain_literal_paths_and_language_specific_options() {
    let root = Path::new("/isolated/project with spaces;$(echo surprise)");
    let output = Path::new("/cache/run/index.scip");
    let run = Path::new("/cache/run");
    let executable = Path::new("/tools/indexer");
    for strategy in IndexPlan::inspect([
        "Cargo.toml",
        "lib.rs",
        "go.mod",
        "app.go",
        "package.json",
        "app.js",
        "pyproject.toml",
        "app.py",
        "compile_commands.json",
        "native.c",
        "build.gradle",
        "App.kt",
    ])
    .selected
    {
        let command = strategy
            .command(CommandPaths {
                workspace: root,
                source_root: root,
                executable,
                output,
                run,
                gradle: None,
                rust_features: &[],
                compilation_database: Some(&run.join("compile_commands.json")),
            })
            .unwrap();
        assert_eq!(command.directory, root);
        assert!(command.arguments.contains(&output.as_os_str().to_owned()));
        assert!(command
            .resources
            .iter()
            .all(|(path, _)| path.starts_with(run)));
        if strategy.indexer == Indexer::TypeScript {
            assert!(command.arguments.contains(&"--infer-tsconfig".into()));
        }
        if strategy.indexer == Indexer::Rust {
            assert_eq!(Path::new(&command.arguments[1]), root);
        }
        if strategy.indexer == Indexer::Kotlin {
            assert_eq!(command.executable, Path::new("/bin/sh"));
            assert_eq!(command.arguments[1], executable.as_os_str());
            assert_eq!(command.resources.len(), 2);
        }
    }
}

#[test]
fn command_rejects_escaping_strategy_inputs() {
    let strategy = IndexingStrategy {
        indexer: Indexer::Clang,
        directory: "../escape".into(),
        evidence: vec![],
        source_files: 0,
    };
    assert!(strategy
        .command(CommandPaths {
            workspace: Path::new("/work"),
            source_root: Path::new("/original"),
            executable: Path::new("tool"),
            output: Path::new("/output.scip"),
            run: Path::new("/run"),
            gradle: None,
            compilation_database: None,
            rust_features: &[],
        })
        .is_err());
}

#[test]
fn clang_command_requires_a_prepared_absolute_database_in_the_private_run() {
    let strategy = IndexPlan::inspect(["build/compile_commands.json", "main.c"])
        .selected
        .remove(0);
    let workspace = Path::new("/private/workspace");
    let run = workspace.join(".run");
    let prepared = run.join("compile_commands.json");
    let escape = run.join("../compile_commands.json");
    for database in [
        None,
        Some(Path::new("build/compile_commands.json")),
        Some(Path::new("/original/compile_commands.json")),
        Some(escape.as_path()),
        Some(prepared.as_path()),
    ] {
        let command = strategy.command(CommandPaths {
            workspace,
            source_root: Path::new("/original"),
            executable: Path::new("/tools/scip-clang"),
            output: &run.join("index.scip"),
            run: &run,
            gradle: None,
            compilation_database: database,
            rust_features: &[],
        });
        if database == Some(prepared.as_path()) {
            let command = command.unwrap();
            assert_eq!(command.arguments[0], "--compdb-path");
            assert_eq!(command.arguments[1], prepared.as_os_str());
            assert_eq!(command.directory, workspace);
        } else {
            assert!(command.is_err());
        }
    }
}
