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
    [
        field(1, &[32, 2]),
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
    let doc = &fresh.documents[0];
    assert_eq!(doc.relative_path, "a.rs");
    assert_eq!(doc.position_encoding, 2);
    assert_eq!(doc.occurrences[0].range, ScipRange::from_packed(&[0, 3, 7]));
    assert_eq!(doc.symbols[0].signature, "fn work()");
    assert!(fresh.external_symbols[0].relationships[0].is_reference);
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
