//! Explicit network acceptance; ordinary tests never install packages.

use super::*;
use crate::semantic::indexing::provisioning::{ProvisioningRequest, ToolProvisioner};
use crate::source::ColumnEncoding;

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
    let mut request = fixture.request();
    request.tools = ToolDirectory::from_environment(None);
    if let Some(executable) = std::env::var_os("RADIOLOGY_REAL_RUST_ANALYZER") {
        let executable = PathBuf::from(executable).canonicalize().unwrap();
        assert_eq!(executable.file_name().unwrap(), "rust-analyzer");
        request
            .tools
            .search_path
            .insert(0, executable.parent().unwrap().into());
    }
    request.timeout = Duration::from_secs(120);
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
        },
        &["node_modules"],
    );
}

struct Navigation<'a> {
    caller: &'a str,
    callee: &'a str,
    target: &'a str,
    span: (i32, i32, i32),
    encoding: ColumnEncoding,
    documents: usize,
}

fn installed_case(
    indexer: Indexer,
    version: &str,
    files: &[(&str, &str)],
    navigation: Navigation<'_>,
    artifacts: &[&str],
) {
    let fixture = Fixture::new(files);
    let mut request = fixture.request();
    request.tools = ToolDirectory::from_environment(Some(fixture.bin.clone()));
    request.timeout = Duration::from_secs(120);
    let ToolAvailability::Available {
        executable,
        version: actual,
    } = install_pinned(&request, indexer)
    else {
        panic!("installation did not verify its executable");
    };
    assert!(executable.starts_with(&fixture.bin));
    assert!(actual.contains(version), "expected {version}, got {actual}");
    verify_lifecycle(&fixture, request, files, navigation, artifacts);
}

fn verify_lifecycle(
    fixture: &Fixture,
    mut request: GenerationRequest,
    files: &[(&str, &str)],
    expected: Navigation<'_>,
    artifacts: &[&str],
) {
    let generated = run(request.clone());
    eprintln!("Real generation: {:#?}", generated.report);
    assert!(!generated.report.is_partial(), "{:#?}", generated.report);
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
    installer.start(ProvisioningRequest {
        indexer,
        tools: request.tools.clone(),
        protected_roots: vec![request.root.clone()],
        timeout: Duration::from_secs(600),
    });
    let deadline = Instant::now() + Duration::from_secs(610);
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
