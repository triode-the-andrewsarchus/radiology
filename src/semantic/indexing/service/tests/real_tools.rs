//! Explicit network acceptance; ordinary tests never install packages.

use super::*;
use crate::semantic::indexing::provisioning::{ProvisioningRequest, ToolProvisioner};

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
    let fixture = Fixture::new(&files);
    let mut request = fixture.request();
    request.tools = ToolDirectory::from_environment(Some(fixture.bin.clone()));
    request.timeout = Duration::from_secs(120);
    let mut installer = ToolProvisioner::default();
    installer.start(ProvisioningRequest {
        indexer: Indexer::TypeScript,
        tools: request.tools.clone(),
        protected_roots: vec![fixture.source.clone()],
        timeout: Duration::from_secs(300),
    });
    let deadline = Instant::now() + Duration::from_secs(310);
    let installed = loop {
        if let Some(result) = installer.poll() {
            break result.expect("real pinned installation failed");
        }
        assert!(Instant::now() < deadline, "{}", installer.progress());
        std::thread::sleep(Duration::from_millis(20));
    };
    let ToolAvailability::Available {
        executable,
        version,
    } = installed.availability
    else {
        panic!("installer did not verify its output");
    };
    assert!(executable.starts_with(&fixture.bin));
    assert!(version.contains("0.4.0"), "{version}");
    let generated = run(request.clone());
    eprintln!("Real TypeScript generation: {:#?}", generated.report);
    assert!(!generated.report.is_partial(), "{:#?}", generated.report);
    assert!(generated.report.stale_reason.is_none());
    assert!(generated.report.cache_warning.is_none());
    assert_eq!(generated.report.documents, 2);
    let index = generated
        .index
        .expect("real indexer emitted no usable data");
    let buffer = SourceBuffer::from_str("b.ts", files[3].1);
    let call_byte = files[3].1.rfind("double").unwrap();
    let call = index
        .occurrence_at_byte(&buffer, call_byte)
        .expect("call occurrence");
    assert_eq!(
        index.column_encoding("b.ts"),
        Some(crate::source::ColumnEncoding::Utf16)
    );
    assert_eq!(
        index.byte_range(&buffer, call.range.as_ref().unwrap()),
        Some(call_byte..call_byte + 6)
    );
    let (path, range) = index
        .find_definition(&call.symbol)
        .expect("call definition");
    assert_eq!(path, "a.ts");
    assert_eq!(
        (range.start_line, range.start_col, range.end_col),
        (0, 16, 22)
    );
    assert!(index
        .find_references(&call.symbol)
        .iter()
        .any(|(path, range)| {
            *path == "b.ts" && index.byte_range(&buffer, range) == Some(call_byte..call_byte + 6)
        }));

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

    // Cache-only loading must succeed even when no executable can be discovered.
    request.tools = ToolDirectory {
        managed: None,
        home: None,
        search_path: Vec::new(),
    };
    let cached = crate::semantic::indexing::cache::load(&request, &AtomicBool::new(false))
        .unwrap()
        .expect("persistent real index");
    assert!(cached.report.cached && cached.report.stale_reason.is_none());
    let restored = cached.index.unwrap();
    assert_eq!(restored.find_definition(&call.symbol), Some((path, range)));
    for (path, text) in files {
        assert_eq!(fs::read_to_string(fixture.source.join(path)).unwrap(), text);
    }
    let inventory = InputInventory::discover(&fixture.source, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        inventory.paths,
        ["a.ts", "b.ts", "package.json", "tsconfig.json"]
    );
    assert!(restored.matches_buffer(&SourceBuffer::from_str("b.ts", files[3].1)));
    assert!(!restored.matches_buffer(&SourceBuffer::from_str("b.ts", "changed\n")));
    assert_eq!(fs::read_dir(&fixture.cache).unwrap().count(), 2);
    assert!(fs::read_dir(fixture.cache.join("projects"))
        .unwrap()
        .all(|entry| entry.unwrap().path().is_file()));
}
