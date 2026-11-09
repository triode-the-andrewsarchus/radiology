use super::*;
use crate::semantic::test_support::cancel_at;
use std::sync::atomic::{AtomicUsize, Ordering};

fn varint(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    while value >= 128 {
        bytes.push((value as u8) | 128);
        value >>= 7;
    }
    bytes.push(value as u8);
    bytes
}

fn field(number: u64, bytes: &[u8]) -> Vec<u8> {
    [
        varint((number << 3) | 2),
        varint(bytes.len() as u64),
        bytes.to_vec(),
    ]
    .concat()
}

fn nested_index() -> Vec<u8> {
    let typed = [8, 0, 16, 3, 24, 7];
    let occurrence = [
        field(1, &[0, 1, 2]),
        field(8, &typed),
        field(11, &[8, 2, 16, 0, 24, 6, 32, 1]),
        [varint(99 << 3), varint(7)].concat(),
        field(2, b"local 1"),
        vec![24, 1],
    ]
    .concat();
    let relation = [field(1, b"scip cargo fixture 1 other()."), vec![16, 1]].concat();
    let symbol = [
        field(1, b"local 1"),
        field(4, &relation),
        field(7, &field(5, b"fn work()")),
    ]
    .concat();
    let document = [
        field(1, b"a.rs"),
        field(2, &occurrence),
        field(3, &symbol),
        vec![48, 2],
    ]
    .concat();
    let tool = [
        field(1, b"fixture-indexer"),
        field(2, b"1.2.3"),
        field(3, "--café".as_bytes()),
        field(3, "🦀".as_bytes()),
        vec![0x98, 0x06, 7],
    ]
    .concat();
    let metadata = [
        field(2, &tool),
        field(3, b"file:///compiler-stage"),
        vec![32, 2],
        field(42, b"unknown metadata"),
    ]
    .concat();
    [
        field(1, &metadata),
        field(2, &document),
        field(3, &symbol),
        field(20, b"ignored unknown field"),
    ]
    .concat()
}

#[test]
fn nested_messages_stop_at_every_retirement_checkpoint_and_recover_exact_fields() {
    let bytes = nested_index();
    let checks = AtomicUsize::new(0);
    ScipIndex::decode_cancellable(&bytes, cancel_at(&checks, usize::MAX)).unwrap();
    for stop in 1..=checks.load(Ordering::Relaxed) {
        let checks = AtomicUsize::new(0);
        let error = ScipIndex::decode_cancellable(&bytes, cancel_at(&checks, stop)).unwrap_err();
        assert_eq!(error, ProtobufError::Cancelled, "checkpoint {stop}");
        assert_eq!(checks.load(Ordering::Relaxed), stop);
    }
    let fresh = ScipIndex::decode(&bytes).unwrap();
    assert_eq!(fresh.text_document_encoding, 2);
    assert_eq!(
        fresh.sources,
        vec![ScipIndexSource {
            tool_name: "fixture-indexer".into(),
            tool_version: "1.2.3".into(),
            tool_arguments: vec!["--café".into(), "🦀".into()],
            project_root: "file:///compiler-stage".into(),
            document_count: 1,
        }]
    );
    let doc = &fresh.documents[0];
    assert_eq!(doc.relative_path, "a.rs");
    assert_eq!(doc.position_encoding, 2);
    assert_eq!(doc.occurrences[0].range, ScipRange::from_packed(&[0, 3, 7]));
    assert_eq!(
        doc.occurrences[0].enclosing_range,
        ScipRange::from_packed(&[2, 0, 6, 1])
    );
    assert!(doc.occurrences[0].is_definition());
    assert_eq!(doc.symbols[0].signature, "fn work()");
    assert!(fresh.external_symbols[0].relationships[0].is_reference);
    let normalized = crate::semantic::SemanticIndex::from_scip(fresh);
    assert_eq!(
        normalized.column_encoding("a.rs"),
        Some(crate::source::ColumnEncoding::Utf16)
    );
    assert_eq!(
        normalized.display_name(&crate::semantic::index::SemanticSymbolID::new(
            "local 1", "a.rs"
        )),
        "1"
    );
}

#[test]
fn packed_arrays_and_large_unicode_strings_inherit_the_parent_cancellation() {
    let packed = field(2, &field(2, &field(1, &vec![0; 128 * 1024])));
    let checks = AtomicUsize::new(0);
    assert_eq!(
        ScipIndex::decode_cancellable(&packed, cancel_at(&checks, 256)).unwrap_err(),
        ProtobufError::Cancelled
    );
    assert_eq!(checks.load(Ordering::Relaxed), 256);
    assert_eq!(
        ScipIndex::decode(&packed).unwrap().documents[0]
            .occurrences
            .len(),
        1
    );
    let text = format!("{}{}", "x".repeat(65_535), "😀".repeat(512 * 1024));
    let bytes = field(
        3,
        &[field(1, b"global symbol"), field(3, text.as_bytes())].concat(),
    );
    let checks = AtomicUsize::new(0);
    assert_eq!(
        ScipIndex::decode_cancellable(&bytes, cancel_at(&checks, 20)).unwrap_err(),
        ProtobufError::Cancelled
    );
    assert_eq!(checks.load(Ordering::Relaxed), 20);
    assert_eq!(
        ScipIndex::decode(&bytes).unwrap().external_symbols[0].documentation[0],
        text
    );
    println!(
        "SCIP: interrupted nested packed values and UTF-8 chunk copying; fresh decode succeeds"
    );
}

#[test]
fn malformed_lengths_and_overflowing_varints_return_errors_without_panicking() {
    let maximum = varint(u64::MAX);
    assert_eq!(
        ProtobufReader::new(&maximum)
            .read_length_delimited()
            .unwrap_err(),
        ProtobufError::Truncated
    );
    assert_eq!(
        ProtobufReader::new(&maximum)
            .skip_field(WireType::LengthDelimited)
            .unwrap_err(),
        ProtobufError::Truncated
    );
    let mut overflow = vec![255; 9];
    overflow.push(2);
    assert_eq!(
        ProtobufReader::new(&overflow).read_varint().unwrap_err(),
        ProtobufError::MalformedVarint
    );
    fn send_sync<T: Send + Sync>() {}
    send_sync::<ProtobufReader<'static>>();
}

#[test]
fn metadata_uses_the_last_message_and_counts_documents_independent_of_wire_order() {
    let document = field(2, &field(1, b"kept.rs"));
    assert!(ScipIndex::decode(&document).unwrap().sources.is_empty());
    let first = field(1, &field(2, &field(1, b"first-indexer")));
    let last = field(1, &field(3, "file:///café/🦀".as_bytes()));
    let decoded = ScipIndex::decode(&[document, first, last].concat()).unwrap();
    assert_eq!(
        decoded.sources,
        vec![ScipIndexSource {
            project_root: "file:///café/🦀".into(),
            document_count: 1,
            ..Default::default()
        }]
    );
    assert_eq!(decoded.documents[0].relative_path, "kept.rs");
    assert_eq!(decoded.text_document_encoding, 0);
    let invalid_tool = field(1, &field(2, &field(1, &[0xff])));
    assert_eq!(
        ScipIndex::decode(&invalid_tool).unwrap_err(),
        ProtobufError::InvalidUtf8
    );
}
