use super::*;
use crate::semantic::test_support::cancel_at;
use crate::semantic::{ScipDocument, ScipSymbolInformation};
use std::sync::atomic::AtomicUsize;
use std::time::{Duration, Instant};

fn entry(root: PathBuf, documentation: String) -> StoredEntry {
    let inventory = InputInventory::from_paths(["lib.rs".into()]).unwrap();
    let plan = super::super::IndexPlan::inspect(["lib.rs"]);
    Entry {
        format: FORMAT,
        root,
        inventory,
        scip: ScipIndex {
            documents: vec![ScipDocument {
                relative_path: "lib.rs".into(),
                position_encoding: 2,
                symbols: vec![ScipSymbolInformation {
                    symbol: "local 0".into(),
                    documentation: vec![documentation],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        },
        report: GenerationReport {
            plan,
            outcomes: vec![],
            fingerprint: "fixture".into(),
            omitted_directories: vec![],
            stale_reason: None,
            overlapping_documents: 0,
            documents: 1,
            cached: false,
            cache_warning: None,
        },
        source_digests: HashMap::from([("lib.rs".into(), "a".repeat(64))]),
    }
}

#[test]
fn json_decoding_stops_inside_large_strings_and_recovers_exact_unicode_and_provenance() {
    let entry = entry("/fixture".into(), "é🦀\"\\\n".repeat(IO_CHUNK));
    let bytes = serde_json::to_vec(&entry).unwrap();
    let checks = AtomicUsize::new(0);
    let decoded = decode(&bytes, cancel_at(&checks, usize::MAX)).unwrap();
    let count = checks.load(Ordering::Relaxed);
    assert!(count > 10, "large string must span multiple guarded reads");
    for step in 1..=count {
        checks.store(0, Ordering::Relaxed);
        let error = decode(&bytes, cancel_at(&checks, step)).err().unwrap();
        assert!(error.contains("cancelled"), "{step}: {error}");
        assert_eq!(checks.load(Ordering::Relaxed), step);
    }
    assert_eq!(
        serde_json::to_value(decoded).unwrap(),
        serde_json::to_value(entry).unwrap()
    );
    let mut trailing = bytes.clone();
    trailing.extend_from_slice(b" false");
    assert!(decode(&trailing, || false).is_err());
    assert!(decode(&bytes[..bytes.len() - 1], || false).is_err());
    println!("Cache JSON: {count} bounded read checks; nested Unicode, cancellation and fresh recovery verified");
}

struct CancelWriter<'a> {
    bytes: Vec<u8>,
    cancel: &'a AtomicBool,
    on_flush: bool,
}
impl Write for CancelWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.bytes.extend_from_slice(bytes);
        if !self.on_flush {
            self.cancel.store(true, Ordering::Relaxed);
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        if self.on_flush {
            self.cancel.store(true, Ordering::Relaxed);
        }
        Ok(())
    }
}

#[test]
fn cache_writes_stop_between_chunks_and_detect_cancellation_during_final_flush() {
    let cancel = AtomicBool::new(false);
    let mut writer = LimitedWriter {
        file: CancelWriter {
            bytes: vec![],
            cancel: &cancel,
            on_flush: false,
        },
        written: 0,
        cancel: &cancel,
    };
    assert!(writer
        .write_all(&vec![b'x'; IO_CHUNK * 3])
        .unwrap_err()
        .to_string()
        .contains("cancelled"));
    assert_eq!(writer.file.bytes.len(), IO_CHUNK);
    assert!(writer.flush().is_err());
    cancel.store(false, Ordering::Relaxed);
    writer.file.on_flush = true;
    assert!(writer
        .flush()
        .unwrap_err()
        .to_string()
        .contains("cancelled"));
    cancel.store(false, Ordering::Relaxed);
    writer.written = MAX_BYTES;
    assert!(writer
        .write(b"x")
        .unwrap_err()
        .to_string()
        .contains("limit"));
}

#[test]
fn cancelled_serialization_removes_temporary_output_and_retains_the_accepted_cache() {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path().join("source");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("lib.rs"), "fn work() {}\n").unwrap();
    let request = GenerationRequest {
        root: root.clone(),
        cache: owner.path().join("cache"),
        tools: crate::semantic::indexing::tools::ToolDirectory {
            managed: None,
            home: None,
            search_path: vec![],
        },
        timeout: Duration::from_secs(5),
        limits: Default::default(),
    };
    let mut entry = entry(fs::canonicalize(&root).unwrap(), "accepted".into());
    let cancel = AtomicBool::new(false);
    let save_entry = |entry: &StoredEntry| {
        save(
            &request,
            &entry.inventory,
            &entry.scip,
            &entry.source_digests,
            &entry.report,
            &cancel,
        )
    };
    save_entry(&entry).unwrap();
    let destination = path(&request).unwrap();
    let pristine = fs::read(&destination).unwrap();
    entry.scip.documents[0].symbols[0].documentation = vec!["🦀\"\\\n".repeat(4 * 1024 * 1024)];
    std::thread::scope(|scope| {
        let observer = scope.spawn(|| {
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                let writing = fs::read_dir(&request.cache).unwrap().any(|file| {
                    let file = file.unwrap();
                    file.path() != destination && file.metadata().unwrap().len() >= IO_CHUNK as u64
                });
                if writing {
                    cancel.store(true, Ordering::Relaxed);
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "serialization never produced observable temporary output"
                );
                std::thread::yield_now();
            }
        });
        let error = save_entry(&entry).unwrap_err();
        observer.join().unwrap();
        assert!(error.contains("cancelled"), "{error}");
    });
    assert_eq!(fs::read(&destination).unwrap(), pristine);
    assert_eq!(fs::read_dir(&request.cache).unwrap().count(), 1);
    cancel.store(false, Ordering::Relaxed);
    entry.scip.documents[0].symbols[0].documentation = vec!["fresh".into()];
    save_entry(&entry).unwrap();
    let restored = load(&request, &cancel).unwrap().unwrap();
    assert_eq!(
        restored
            .index
            .unwrap()
            .symbols
            .values()
            .next()
            .unwrap()
            .documentation,
        ["fresh"]
    );
    cancel.store(true, Ordering::Relaxed);
    assert!(validate(&entry, &entry.root, &cancel)
        .unwrap_err()
        .contains("cancelled"));
    assert!(
        valid_document(&entry.scip.documents[0], &entry.source_digests, &cancel)
            .unwrap_err()
            .contains("cancelled")
    );
    println!("Cache serialization: live temporary write interrupted, accepted file retained, temporary removed, fresh save/load succeeds");
}
