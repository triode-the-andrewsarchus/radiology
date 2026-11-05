//! Explicit network acceptance; ordinary tests never install packages.

use super::*;
use crate::semantic::indexing::provisioning::{ProvisioningRequest, ToolProvisioner};
use crate::source::ColumnEncoding;

#[test]
#[ignore = "requires real scip-clang; optionally set RADIOLOGY_REAL_SCIP_CLANG"]
fn real_scip_clang_fallback_resolves_cross_file_call_despite_missing_header_and_retains_partial() {
    clang_fallback_case(false);
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[ignore = "downloads pinned scip-clang 0.4.0; requires curl, sha256sum and network access"]
fn pinned_clang_install_cross_file_fallback_navigation_reuse_and_reload() {
    clang_fallback_case(true);
}

fn clang_fallback_case(install: bool) {
    let files = [
        ("support/include/layout/api.hpp", "int increment(int value);\n"),
        ("src/api.cpp", "#include <layout/api.hpp>\nint increment(int value) { return value + 1; }\n"),
        ("src/reader.cpp", "#include <layout/api.hpp>\r\n#include <missing_header.hpp>\r\nint run() { /* 🦀 */ return increment(4); }\r\n"),
    ];
    let navigation = Navigation {
        caller: "src/reader.cpp",
        callee: "increment",
        target: "src/api.cpp",
        span: (1, 4, 13),
        encoding: ColumnEncoding::Utf8,
        documents: 3,
        partial_build: true,
    };
    let artifacts = &["compile_commands.json", "index.scip", "src/api.o"];
    if install {
        installed_case(Indexer::Clang, "0.4.0", &files, navigation, artifacts);
    } else {
        let fixture = Fixture::new(&files);
        let request = external_tool_request(&fixture, Indexer::Clang, "RADIOLOGY_REAL_SCIP_CLANG");
        verify_lifecycle(&fixture, request, &files, navigation, artifacts);
    }
}

#[test]
#[ignore = "requires real scip-clang; optionally set RADIOLOGY_REAL_SCIP_CLANG"]
fn real_scip_clang_response_inputs_navigation_unicode_reuse_and_reload() {
    for (extension, standard, database_path) in [
        ("c", "c11", "compile_commands.json"),
        ("cpp", "c++17", "compile_commands.json"),
        ("c", "c11", "build/compile_commands.json"),
        ("cpp", "c++17", "build/compile_commands.json"),
    ] {
        let main = format!("main.{extension}");
        let database = serde_json::json!([{
            "directory": ".", "file": main,
            "arguments": ["/usr/bin/clang", "@compile.rsp"]
        }])
        .to_string();
        let response = format!("-I. -std={standard} -c {main} -o main.o\n");
        let files = [
            (database_path, database.as_str()),
            ("compile.rsp", response.as_str()),
            (
                "helper.h",
                "inline int twice(int value) { return value * 2; }\n",
            ),
            (
                main.as_str(),
                "#include \"helper.h\"\nint main() { /* 🦀 */ return twice(21); }\n",
            ),
        ];
        let fixture = Fixture::new(&files);
        let request = external_tool_request(&fixture, Indexer::Clang, "RADIOLOGY_REAL_SCIP_CLANG");
        verify_lifecycle(
            &fixture,
            request,
            &files,
            Navigation {
                caller: &main,
                callee: "twice",
                target: "helper.h",
                span: (0, 11, 16),
                encoding: ColumnEncoding::Utf8,
                documents: 2,
                partial_build: false,
            },
            &["main.o", "index.scip"],
        );
    }
}

#[test]
#[ignore = "builds pinned scip-java and downloads Kotlin/Gradle; requires JDK 17+ and network access"]
fn pinned_kotlin_java_install_navigation_unicode_reuse_and_reload() {
    let files = [
        ("settings.gradle.kts", "rootProject.name = \"radiology-acceptance\"\n"),
        ("build.gradle.kts", "plugins { kotlin(\"jvm\") version \"2.4.20\" }\nrepositories { mavenCentral() }\ngroup = \"radiology.fixture\"\nversion = \"1.0.0\"\n"),
        ("src/main/kotlin/Helper.kt", "package fixture\n\nfun twice(value: Int): Int = JavaHelper.twice(value)\n"),
        ("src/main/java/fixture/JavaHelper.java", "package fixture;\n\npublic class JavaHelper {\n    public static int twice(int value) { return value * 2; }\n}\n"),
        ("src/main/kotlin/Caller.kt", "package fixture\n\nfun caller(): Int { /* 🦀 */ return twice(21) }\n"),
    ];
    let index = installed_case(
        Indexer::Kotlin,
        "0.13.1-radiology-9f288dd339c7",
        &files,
        Navigation {
            caller: "src/main/kotlin/Caller.kt",
            callee: "twice",
            target: "src/main/kotlin/Helper.kt",
            span: (2, 4, 9),
            encoding: ColumnEncoding::Utf16,
            documents: 3,
            partial_build: false,
        },
        &[
            ".gradle",
            ".kotlin",
            "build",
            ".radiology-scip",
            ".radiology-index-tools",
        ],
    );
    let (path, text) = files
        .iter()
        .find(|(path, _)| path.ends_with("Helper.kt"))
        .unwrap();
    let buffer = SourceBuffer::from_str(*path, text);
    let byte = text.rfind("twice").unwrap();
    let call = index
        .occurrence_at_byte(&buffer, byte)
        .expect("Java call from Kotlin");
    let (path, range) = index
        .find_definition(&call.symbol)
        .expect("Java definition");
    assert_eq!(path, "src/main/java/fixture/JavaHelper.java");
    assert_eq!(
        (range.start_line, range.start_col, range.end_col),
        (3, 22, 27)
    );
    assert_eq!(
        index.byte_range(&buffer, call.range.as_ref().unwrap()),
        Some(byte..byte + 5)
    );
}

#[test]
#[ignore = "downloads pinned scip-python; requires Node/npm and Python 3.10+ with pip"]
fn pinned_python_install_cross_module_navigation_unicode_reuse_and_reload() {
    let files = [
        (
            "pyproject.toml",
            "[project]\nname='radiology-acceptance'\nversion='0.1.0'\n",
        ),
        (
            "helper.py",
            "def twice(value: int) -> int:\n    return value * 2\n",
        ),
        (
            "caller.py",
            "from helper import twice\n\nlabel = '🦀'; result = twice(21)\n",
        ),
    ];
    installed_case(
        Indexer::Python,
        "0.6.6",
        &files,
        Navigation {
            caller: "caller.py",
            callee: "twice",
            target: "helper.py",
            span: (0, 4, 9),
            encoding: ColumnEncoding::Utf16,
            documents: 2,
            partial_build: false,
        },
        &["__pycache__", ".venv"],
    );
}

#[test]
#[ignore = "downloads and builds pinned scip-go; requires Go and network access"]
fn pinned_go_install_cross_package_navigation_unicode_reuse_and_reload() {
    let files = [
        ("go.mod", "module radiology.example/acceptance\n\ngo 1.20\n"),
        ("double/double.go", "package double\n\nfunc Twice(value int) int { return value * 2 }\n"),
        ("main.go", "package main\n\nimport \"radiology.example/acceptance/double\"\n\nfunc main() { /* 🦀 */ _ = double.Twice(21) }\n"),
    ];
    installed_case(
        Indexer::Go,
        "0.2.7",
        &files,
        Navigation {
            caller: "main.go",
            callee: "Twice",
            target: "double/double.go",
            span: (2, 5, 10),
            encoding: ColumnEncoding::Utf8,
            documents: 2,
            partial_build: false,
        },
        &["go.sum", "vendor"],
    );
}

#[test]
#[ignore = "requires a real rust-analyzer component; optionally set RADIOLOGY_REAL_RUST_ANALYZER"]
fn real_rust_analyzer_indexes_feature_gated_workspace_calls_and_reuses_the_result() {
    let files = [
        ("Cargo.toml", "[package]\nname='radiology-rust-acceptance'\nversion='0.1.0'\nedition='2021'\n[workspace]\nmembers=['member']\nresolver='2'\n[features]\nroot_feature=[]\n"),
        ("src/lib.rs", "#[cfg(all(feature = \"member_gate\", miri))]\nmod gated;\npub fn run() -> i32 { gated::double(21) }\n"),
        ("src/gated.rs", "pub fn double(value: i32) -> i32 { value * 2 }\n"),
        ("member/Cargo.toml", "[package]\nname='radiology-member'\nversion='0.1.0'\nedition='2021'\n[features]\nmember_gate=[]\n"),
        ("member/src/lib.rs", "pub fn member() {}\n"),
    ];
    let fixture = Fixture::new(&files);
    let request = external_tool_request(&fixture, Indexer::Rust, "RADIOLOGY_REAL_RUST_ANALYZER");
    verify_lifecycle(
        &fixture,
        request,
        &files,
        Navigation {
            caller: "src/lib.rs",
            callee: "double",
            target: "src/gated.rs",
            span: (0, 7, 13),
            encoding: ColumnEncoding::Utf8,
            documents: 3,
            partial_build: false,
        },
        &["Cargo.lock", "target"],
    );
}

#[test]
#[ignore = "downloads the pinned TypeScript indexer; requires Node.js 22+ and npm"]
fn pinned_typescript_install_generation_navigation_and_cache_reload() {
    let files = [
        (
            "package.json",
            "{\"name\":\"radiology-acceptance\",\"version\":\"1.0.0\"}\n",
        ),
        (
            "tsconfig.json",
            "{\"compilerOptions\":{\"noLib\":true},\"include\":[\"*.ts\"]}\n",
        ),
        (
            "a.ts",
            "export function double(value: number): number { return value * 2; }\n",
        ),
        (
            "b.ts",
            "import { double } from './a';\r\n/* 🦀 */ export const result = double(21);\r\n",
        ),
    ];
    installed_case(
        Indexer::TypeScript,
        "0.4.0",
        &files,
        Navigation {
            caller: "b.ts",
            callee: "double",
            target: "a.ts",
            span: (0, 16, 22),
            encoding: ColumnEncoding::Utf16,
            documents: 2,
            partial_build: false,
        },
        &["node_modules"],
    );
}

#[test]
#[ignore = "requires real scip-clang; optionally set RADIOLOGY_REAL_SCIP_CLANG"]
fn real_clang_vendor_dependencies_invalidate_reuse_and_preserve_partial_builds() {
    use crate::pull_request::review::tests::git;
    let mut owned = vec![
        (".gitignore".to_owned(), "build/\n".to_owned()),
        ("include/api.hpp".into(), "int transform(int value);\n".into()),
        ("vendor/dependency/config.hpp".into(), "#define DEPENDENCY_OFFSET 5\n".into()),
        ("src/api.cpp".into(), "#include <api.hpp>\nint transform(int value) { return dependency::stage23<7>::apply(value) + DEPENDENCY_OFFSET; }\n".into()),
        ("compile.rsp".into(), "@nested.rsp -Iinclude -Ivendor -include dependency/stage23.hpp -imacros dependency/config.hpp\n".into()),
        ("nested.rsp".into(), "-std=c++17 -DNAME='literal spaces;not shell'\n".into()),
    ];
    for number in 0..24 {
        let text = if number == 0 {
            "#pragma once\nnamespace dependency { template<int N> struct stage0 { static int apply(int value) { return value + N; } }; }\n".into()
        } else {
            format!("#pragma once\n#include \"stage{}.hpp\"\nnamespace dependency {{ template<int N> struct stage{number} {{ static int apply(int value) {{ return stage{}<N>::apply(value); }} }}; }}\n", number - 1, number - 1)
        };
        owned.push((format!("vendor/dependency/stage{number}.hpp"), text));
    }
    let mut commands = vec![serde_json::json!({
        "directory": ".", "file": "src/api.cpp",
        "arguments": ["clang++", "@compile.rsp", "-c", "src/api.cpp", "-o", "build/api.o"]
    })];
    for number in 0..32 {
        let path = format!("src/caller{number:02}.cpp");
        owned.push((path.clone(), format!("#include <api.hpp>\nint caller{number}() {{ /* 🦀 */ return transform({number}); }}\n")));
        commands.push(serde_json::json!({
            "directory": ".", "file": path,
            "arguments": ["clang++", "@compile.rsp", "-c", path, "-o", format!("build/caller{number}.o")]
        }));
    }
    owned.push((
        "build/compile_commands.json".into(),
        serde_json::to_string(&commands).unwrap(),
    ));
    let files: Vec<_> = owned
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect();
    let fixture = Fixture::new(&files);
    git(&fixture.source, &["init", "-q"]);
    git(&fixture.source, &["add", "."]);
    assert!(git(&fixture.source, &["ls-files", "vendor"]).contains("stage23.hpp"));
    assert!(git(&fixture.source, &["ls-files", "build"]).is_empty());
    let status = git(&fixture.source, &["status", "--porcelain"]);
    let mut request = external_tool_request(&fixture, Indexer::Clang, "RADIOLOGY_REAL_SCIP_CLANG");
    let accepted = verify_lifecycle(
        &fixture,
        request.clone(),
        &files,
        Navigation {
            caller: "src/caller00.cpp",
            callee: "transform",
            target: "src/api.cpp",
            span: (1, 4, 13),
            encoding: ColumnEncoding::Utf8,
            documents: 59,
            partial_build: false,
        },
        &["build/api.o", "build/caller0.o", "index.scip"],
    );
    assert_eq!(git(&fixture.source, &["status", "--porcelain"]), status);
    // A dependency-only edit must invalidate both normalized and project reuse.
    fs::write(
        fixture.source.join("vendor/dependency/config.hpp"),
        "#define DEPENDENCY_OFFSET 9\n",
    )
    .unwrap();
    request.reuse = true;
    let stale = super::super::super::cache::load(&request, &AtomicBool::new(false))
        .unwrap()
        .unwrap();
    assert!(stale.report.stale_reason.is_some());
    let changed = run(request.clone());
    eprintln!("Dependency edit: {:#?}", changed.report);
    assert!(changed.report.outcomes[0].reused_from.is_none());
    assert!(!changed.report.is_partial());
    assert_eq!(changed.report.documents, 59);
    let config = SourceBuffer::from_str(
        "vendor/dependency/config.hpp",
        "#define DEPENDENCY_OFFSET 9\n",
    );
    assert!(!accepted.matches_buffer(&config));
    assert!(changed.index.unwrap().matches_buffer(&config));
    // An unavailable generated header must not erase independent compiler data.
    let broken = "#include <missing/generated.hpp>\nint unavailable() { return 1; }\n";
    fs::write(fixture.source.join("src/caller31.cpp"), broken).unwrap();
    let partial = run(request.clone());
    eprintln!("Missing generated dependency: {:#?}", partial.report);
    let outcome = &partial.report.outcomes[0];
    assert!(outcome.partial_build && partial.report.is_partial());
    assert!(outcome.reused_from.is_none());
    assert!(outcome.diagnostics.contains("missing/generated.hpp"));
    assert!(partial.report.stale_reason.is_none() && partial.report.cache_warning.is_none());
    let partial_index = partial
        .index
        .expect("successful independent translation units survive");
    let text = files
        .iter()
        .find(|(path, _)| *path == "src/caller00.cpp")
        .unwrap()
        .1;
    let caller = SourceBuffer::from_str("src/caller00.cpp", text);
    let call = partial_index
        .occurrence_at_byte(&caller, text.find("transform").unwrap())
        .unwrap();
    assert_eq!(
        partial_index.find_definition(&call.symbol).unwrap().0,
        "src/api.cpp"
    );
    let cached = super::super::super::cache::load(&request, &AtomicBool::new(false))
        .unwrap()
        .unwrap();
    assert!(cached.report.cached && cached.report.is_partial());
    assert!(cached.report.outcomes[0].partial_build);
    assert_eq!(
        cached.index.unwrap().find_definition(&call.symbol),
        partial_index.find_definition(&call.symbol)
    );
    let reused = run(request.clone());
    assert!(reused.report.is_partial() && reused.report.outcomes[0].partial_build);
    assert!(reused.report.outcomes[0].reused_from.is_some());
    // Escaping response inputs fail explicitly before executing arbitrary build inputs.
    fs::write(
        fixture.source.join("compile.rsp"),
        "@/outside/not-copied.rsp\n",
    )
    .unwrap();
    let failed = run(request);
    assert!(failed.index.is_none());
    assert!(failed.report.outcomes[0]
        .failure
        .as_ref()
        .unwrap()
        .contains("outside"));
    let modified: Vec<_> = files
        .iter()
        .map(|(path, original)| {
            let expected = match *path {
                "vendor/dependency/config.hpp" => "#define DEPENDENCY_OFFSET 9\n",
                "src/caller31.cpp" => broken,
                "compile.rsp" => "@/outside/not-copied.rsp\n",
                _ => original,
            };
            (*path, expected)
        })
        .collect();
    assert_viewed_inputs_unchanged(&fixture, &modified);
    assert!(!fixture.source.join("build/api.o").exists());
    assert!(!fixture.source.join("index.scip").exists());
    assert!(walkdir::WalkDir::new(&fixture.cache)
        .into_iter()
        .map(Result::unwrap)
        .all(|entry| !entry
            .file_name()
            .to_string_lossy()
            .starts_with("radiology-build-inputs-")));
}

#[test]
#[ignore = "requires RADIOLOGY_REAL_KOTLIN_TOOLS pinned installation, full JDK 17 and network access"]
fn real_gradle_modules_keep_partial_shards_and_report_missing_android_sdk() {
    let tool_root = std::env::var_os("RADIOLOGY_REAL_KOTLIN_TOOLS")
        .map(PathBuf::from)
        .expect("set RADIOLOGY_REAL_KOTLIN_TOOLS to the private managed tool root");
    let settings = "pluginManagement { repositories { google(); gradlePluginPortal(); mavenCentral() } }\nrootProject.name = \"radiology-modules\"\ninclude(\":app\", \":core\", \":broken\")\n";
    let root_build = "plugins { kotlin(\"jvm\") version \"2.4.20\" apply false }\nallprojects { repositories { mavenCentral() }; group = \"radiology.fixture\"; version = \"1.0.0\" }\n";
    let broken_path = "broken/src/main/kotlin/Broken.kt";
    let good_broken = "package broken\nfun value(): Int = 5\n";
    let bad_broken = "package broken\nfun value(): Int = missingSymbol\n";
    let files = [
        ("settings.gradle.kts", settings),
        ("build.gradle.kts", root_build),
        ("app/build.gradle.kts", "plugins { kotlin(\"jvm\") }\ndependencies { implementation(project(\":core\")) }\n"),
        ("core/build.gradle.kts", "plugins { kotlin(\"jvm\") }\n"),
        ("broken/build.gradle.kts", "plugins { kotlin(\"jvm\") }\n"),
        ("core/src/main/kotlin/Helper.kt", "package support\n\nfun twice(value: Int): Int = JavaHelper.twice(value)\n"),
        ("core/src/main/java/support/JavaHelper.java", "package support;\npublic class JavaHelper { public static int twice(int value) { return value * 2; } }\n"),
        ("app/src/main/kotlin/Caller.kt", "package fixture\nimport support.twice\nfun caller(): Int { /* 🦀 */ return twice(21) }\n"),
        (broken_path, good_broken),
    ];
    let fixture = Fixture::new(&files);
    let mut request = fixture.request();
    request.tools = ToolDirectory::from_environment(Some(tool_root.canonicalize().unwrap()));
    request.timeout = Duration::from_secs(600);
    let artifacts = [
        ".gradle",
        ".kotlin",
        "app/build",
        "core/build",
        "broken/build",
        ".radiology-scip",
        ".radiology-index-tools",
    ];
    for (text, documents, partial_build) in [(good_broken, 4, false), (bad_broken, 3, true)] {
        fs::write(fixture.source.join(broken_path), text).unwrap();
        let current: Vec<_> = files
            .iter()
            .map(|(path, original)| (*path, if *path == broken_path { text } else { original }))
            .collect();
        let index = verify_lifecycle(
            &fixture,
            request.clone(),
            &current,
            Navigation {
                caller: "app/src/main/kotlin/Caller.kt",
                callee: "twice",
                target: "core/src/main/kotlin/Helper.kt",
                span: (2, 4, 9),
                encoding: ColumnEncoding::Utf16,
                documents,
                partial_build,
            },
            &artifacts,
        );
        let helper = SourceBuffer::from_str("core/src/main/kotlin/Helper.kt", files[5].1);
        let call = index
            .occurrence_at_byte(&helper, files[5].1.rfind("twice").unwrap())
            .unwrap();
        assert_eq!(
            index.find_definition(&call.symbol).unwrap().0,
            "core/src/main/java/support/JavaHelper.java"
        );
    }
    // Real AGP configuration must identify a missing SDK rather than publish success.
    let cache_path = super::super::super::cache::path(&request).unwrap();
    let accepted_cache = fs::read(&cache_path).unwrap();
    let android_build = "plugins { id(\"com.android.library\") }\nandroid { namespace = \"radiology.fixture\"; compileSdk = 36; defaultConfig { minSdk = 26 } }\n";
    let android_root = root_build.replace(
        "apply false }",
        "apply false; id(\"com.android.library\") version \"9.4.1\" apply false }",
    );
    let local_properties = format!(
        "sdk.dir={}\n",
        fixture._owner.path().join("missing-sdk").display()
    );
    fs::write(fixture.source.join("build.gradle.kts"), &android_root).unwrap();
    fs::write(
        fixture.source.join("broken/build.gradle.kts"),
        android_build,
    )
    .unwrap();
    fs::write(fixture.source.join(broken_path), good_broken).unwrap();
    fs::write(fixture.source.join("local.properties"), &local_properties).unwrap();
    let failed = run(request);
    eprintln!("Real AGP missing SDK: {:#?}", failed.report);
    assert!(failed.index.is_none());
    let outcome = &failed.report.outcomes[0];
    assert!(outcome.failure.is_some());
    assert!(
        outcome.diagnostics.contains("SDK location not found")
            || outcome.diagnostics.contains("SDK directory"),
        "{}",
        outcome.diagnostics
    );
    assert_eq!(fs::read(cache_path).unwrap(), accepted_cache);
    let mut final_files: Vec<_> = files
        .iter()
        .map(|(path, original)| {
            (
                *path,
                match *path {
                    "build.gradle.kts" => android_root.as_str(),
                    "broken/build.gradle.kts" => android_build,
                    path if path == broken_path => good_broken,
                    _ => original,
                },
            )
        })
        .collect();
    final_files.push(("local.properties", &local_properties));
    assert_viewed_inputs_unchanged(&fixture, &final_files);
    for artifact in artifacts {
        assert!(!fixture.source.join(artifact).exists());
    }
}

struct Navigation<'a> {
    caller: &'a str,
    callee: &'a str,
    target: &'a str,
    span: (i32, i32, i32),
    encoding: ColumnEncoding,
    documents: usize,
    partial_build: bool,
}

fn external_tool_request(
    fixture: &Fixture,
    indexer: Indexer,
    override_variable: &str,
) -> GenerationRequest {
    let mut request = fixture.request();
    request.tools = ToolDirectory::from_environment(None);
    if let Some(executable) = std::env::var_os(override_variable) {
        let executable = PathBuf::from(executable).canonicalize().unwrap();
        assert_eq!(executable.file_name().unwrap(), indexer.id());
        request
            .tools
            .search_path
            .insert(0, executable.parent().unwrap().into());
    }
    request.timeout = Duration::from_secs(120);
    request
}

fn installed_case(
    indexer: Indexer,
    version: &str,
    files: &[(&str, &str)],
    navigation: Navigation<'_>,
    artifacts: &[&str],
) -> SemanticIndex {
    let fixture = Fixture::new(files);
    let mut request = fixture.request();
    request.tools = ToolDirectory::from_environment(Some(fixture.bin.clone()));
    request.timeout = Duration::from_secs(if indexer == Indexer::Kotlin { 600 } else { 120 });
    let ToolAvailability::Available {
        executable,
        version: actual,
    } = install_pinned(&request, indexer)
    else {
        panic!("installation did not verify its executable");
    };
    assert!(executable.starts_with(&fixture.bin));
    assert!(actual.contains(version), "expected {version}, got {actual}");
    verify_lifecycle(&fixture, request, files, navigation, artifacts)
}

fn verify_lifecycle(
    fixture: &Fixture,
    mut request: GenerationRequest,
    files: &[(&str, &str)],
    expected: Navigation<'_>,
    artifacts: &[&str],
) -> SemanticIndex {
    let generated = run(request.clone());
    eprintln!("Real generation: {:#?}", generated.report);
    let best_effort = IndexPlan::inspect(files.iter().map(|(path, _)| *path))
        .selected
        .iter()
        .any(|strategy| strategy.is_best_effort());
    let partial = best_effort || expected.partial_build;
    assert_eq!(
        generated.report.outcomes[0].partial_build,
        expected.partial_build
    );
    assert_eq!(
        generated.report.is_partial(),
        partial,
        "{:#?}",
        generated.report
    );
    if best_effort {
        assert!(generated.report.outcomes[0]
            .diagnostics
            .contains("missing_header.hpp"));
    }
    assert!(generated.report.stale_reason.is_none() && generated.report.cache_warning.is_none());
    assert_eq!(generated.report.documents, expected.documents);
    let index = generated.index.expect("real index");
    let text = files
        .iter()
        .find(|(path, _)| *path == expected.caller)
        .expect("caller fixture")
        .1;
    let buffer = SourceBuffer::from_str(expected.caller, text);
    let byte = text.rfind(expected.callee).unwrap();
    let call = index
        .occurrence_at_byte(&buffer, byte)
        .expect("call occurrence");
    let bytes = byte..byte + expected.callee.len();
    assert_eq!(
        index.column_encoding(expected.caller),
        Some(expected.encoding)
    );
    assert_eq!(
        index.byte_range(&buffer, call.range.as_ref().unwrap()),
        Some(bytes.clone())
    );
    let (path, range) = index
        .find_definition(&call.symbol)
        .expect("cross-file definition");
    assert_eq!(path, expected.target);
    assert_eq!(
        (range.start_line, range.start_col, range.end_col),
        expected.span
    );
    assert!(index
        .find_references(&call.symbol)
        .iter()
        .any(|(path, range)| *path == expected.caller
            && index.byte_range(&buffer, range) == Some(bytes.clone())));
    request.reuse = true;
    let updated = run(request.clone());
    assert_eq!(updated.report.is_partial(), partial);
    assert_eq!(
        updated.report.outcomes[0].partial_build,
        generated.report.outcomes[0].partial_build
    );
    assert_eq!(
        updated.report.outcomes[0].reused_from.as_deref(),
        Some("checkout")
    );
    assert_eq!(
        updated.index.unwrap().find_definition(&call.symbol),
        Some((path, range))
    );
    request.tools = ToolDirectory {
        managed: None,
        home: None,
        search_path: Vec::new(),
    };
    let mut loader = IndexGenerator::default();
    loader.start_cached(request);
    let cached = settle(&mut loader);
    assert!(cached.report.cached && cached.report.stale_reason.is_none());
    assert_eq!(cached.report.is_partial(), partial);
    assert_eq!(
        cached.report.outcomes[0].partial_build,
        generated.report.outcomes[0].partial_build
    );
    if best_effort {
        assert!(cached.report.summary().starts_with("Partial cached index"));
    }
    let restored = cached.index.unwrap();
    assert_eq!(restored.find_definition(&call.symbol), Some((path, range)));
    assert!(restored.matches_buffer(&buffer));
    assert!(!restored.matches_buffer(&SourceBuffer::from_str(expected.caller, "changed\n")));
    assert_viewed_inputs_unchanged(fixture, files);
    for artifact in artifacts {
        assert!(
            !fixture.source.join(artifact).exists(),
            "viewed artifact: {artifact}"
        );
    }
    assert!(walkdir::WalkDir::new(&fixture.cache)
        .into_iter()
        .map(Result::unwrap)
        .all(|entry| {
            let name = entry.file_name().to_string_lossy();
            !name.starts_with("radiology-build-inputs-")
                && !name.starts_with(".radiology-index-stage-")
        }));
    restored
}

fn assert_viewed_inputs_unchanged(fixture: &Fixture, files: &[(&str, &str)]) {
    for (path, text) in files {
        assert_eq!(
            fs::read_to_string(fixture.source.join(path)).unwrap(),
            *text
        );
    }
    let mut expected: Vec<_> = files.iter().map(|(path, _)| path.to_string()).collect();
    expected.sort();
    assert_eq!(
        InputInventory::discover(&fixture.source, &AtomicBool::new(false))
            .unwrap()
            .paths,
        expected
    );
}

fn install_pinned(request: &GenerationRequest, indexer: Indexer) -> ToolAvailability {
    let mut installer = ToolProvisioner::default();
    let timeout = Duration::from_secs(if indexer == Indexer::Kotlin {
        1800
    } else {
        600
    });
    installer.start(ProvisioningRequest {
        indexer,
        tools: request.tools.clone(),
        protected_roots: vec![request.root.clone()],
        timeout,
    });
    let deadline = Instant::now() + timeout + Duration::from_secs(10);
    let mut previous = String::new();
    loop {
        if let Some(result) = installer.poll() {
            return result
                .expect("real pinned installation failed")
                .availability;
        }
        let phase = installer.progress();
        if phase != previous {
            eprintln!("{} installation: {phase}", indexer.id());
            previous = phase;
        }
        assert!(Instant::now() < deadline, "{}", installer.progress());
        std::thread::sleep(Duration::from_millis(20));
    }
}
