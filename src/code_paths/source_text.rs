//! Byte-preserving lexical masking and mutually exclusive preprocessor contexts.
use crate::source::SourceBuffer;
use std::collections::BTreeMap;

pub(super) fn masked<E>(bytes: &[u8], check: &impl Fn() -> Result<(), E>) -> Result<Vec<u8>, E> {
    let mut result = bytes.to_vec();
    let mut cursor = 0;
    while cursor < bytes.len() {
        check()?;
        if let Some(end) = hidden_end(bytes, cursor) {
            for byte in &mut result[cursor..end] {
                if !matches!(*byte, b'\r' | b'\n') {
                    *byte = b' ';
                }
            }
            cursor = end;
        } else {
            cursor += 1;
        }
    }
    check()?;
    Ok(result)
}

fn hidden_end(bytes: &[u8], start: usize) -> Option<usize> {
    let tail = &bytes[start..];
    if tail.starts_with(b"//") {
        Some(start + tail.iter().position(|&b| b == b'\n').unwrap_or(tail.len()))
    } else if tail.starts_with(b"/*") {
        Some(
            start
                + tail[2..]
                    .windows(2)
                    .position(|pair| pair == b"*/")
                    .map_or(tail.len(), |end| end + 4),
        )
    } else if let Some(end) = raw_end(tail) {
        Some(start + end)
    } else if matches!(tail[0], b'\'' | b'"') {
        Some(quoted_end(bytes, start))
    } else {
        None
    }
}

fn quoted_end(bytes: &[u8], start: usize) -> usize {
    let mut cursor = start + 1;
    while cursor < bytes.len() {
        if bytes[cursor] == bytes[start] {
            return cursor + 1;
        }
        cursor += if bytes[cursor] == b'\\' { 2 } else { 1 };
    }
    bytes.len()
}

fn raw_end(tail: &[u8]) -> Option<usize> {
    if !tail.starts_with(b"R\"") {
        return None;
    }
    let open = tail.iter().take(19).position(|&b| b == b'(')?;
    let delimiter = &tail[2..open];
    if delimiter
        .iter()
        .any(|b| b.is_ascii_whitespace() || matches!(b, b')' | b'\\'))
    {
        return None;
    }
    let closing: Vec<_> = [b")".as_slice(), delimiter, b"\"".as_slice()].concat();
    Some(
        tail[open + 1..]
            .windows(closing.len())
            .position(|part| part == closing)
            .map_or(tail.len(), |at| open + 1 + at + closing.len()),
    )
}

pub(super) fn alternatives<E>(
    positions: &[usize],
    buffer: &SourceBuffer,
    check: &impl Fn() -> Result<(), E>,
) -> Result<bool, E> {
    if positions.len() < 2 {
        return Ok(false);
    }
    let bytes = masked(&buffer.data, check)?;
    let mut stack = Vec::<(usize, usize)>::new();
    let mut contexts = Vec::<BTreeMap<usize, usize>>::new();
    let mut current = 0;
    let mut start = 0;
    for line in bytes.split_inclusive(|&b| b == b'\n') {
        check()?;
        directive(line, start, &mut stack);
        while positions
            .get(current)
            .is_some_and(|&position| position < start + line.len())
        {
            contexts.push(stack.iter().copied().collect());
            current += 1;
        }
        start += line.len();
    }
    if contexts.len() != positions.len() {
        return Ok(false);
    }
    for (left, context) in contexts.iter().enumerate() {
        for other in &contexts[left + 1..] {
            check()?;
            if !context
                .iter()
                .any(|(group, branch)| other.get(group).is_some_and(|other| branch != other))
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn directive(line: &[u8], start: usize, stack: &mut Vec<(usize, usize)>) {
    let line = line.trim_ascii_start();
    let Some(tail) = line.strip_prefix(b"#") else {
        return;
    };
    let tail = tail.trim_ascii_start();
    let end = tail
        .iter()
        .position(|b| !b.is_ascii_alphabetic())
        .unwrap_or(tail.len());
    match &tail[..end] {
        b"if" | b"ifdef" | b"ifndef" => stack.push((start, 0)),
        b"elif" | b"else" => {
            if let Some((_, branch)) = stack.last_mut() {
                *branch += 1;
            }
        }
        b"endif" => {
            stack.pop();
        }
        _ => {}
    }
}
