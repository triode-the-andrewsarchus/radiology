//! Worker-owned destination structure, independent of displayed excerpt identity.
use super::SourceQueryInputs;
use crate::{
    background::lock_checked,
    source::{buffer::SourceIdentity, loader::check_source, queries::QueryCache, SourceBuffer},
    syntax::{DocumentSymbol, SymbolExtractor},
};
use std::{
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::Instant,
};

#[derive(Default)]
pub(super) struct Cache {
    entries: QueryCache<SourceIdentity, Arc<[DocumentSymbol]>, 4>,
    #[cfg(test)]
    pub(super) parses: usize,
}

pub(super) fn end_line(
    cache: &Mutex<Cache>,
    inputs: &Arc<SourceQueryInputs>,
    buffer: &SourceBuffer,
    byte: usize,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<Option<usize>, String> {
    let mut cache = lock_checked(cache, || check_source(cancel, deadline))?;
    check_source(cancel, deadline)?;
    cache.entries.prepare(Some(inputs));
    cache
        .entries
        .observe_source(&buffer.path, buffer.revision, buffer.content_digest());
    let key = SourceIdentity::new(buffer);
    let symbols = if let Some(symbols) = cache.entries.get(&key) {
        symbols.clone()
    } else {
        #[cfg(test)]
        {
            cache.parses += 1;
        }
        let symbols = SymbolExtractor::extract_cancellable(buffer, || {
            check_source(cancel, deadline).is_err()
        });
        check_source(cancel, deadline)?;
        let symbols: Arc<[DocumentSymbol]> = symbols?.into();
        let bytes = retained_bytes(&symbols, cancel, deadline)?
            + buffer.path.len()
            + buffer.content_digest().len();
        cache.entries.insert(key, symbols.clone(), bytes);
        symbols
    };
    check_source(cancel, deadline)?;
    Ok(declaration_end(&symbols, byte).map(|end| buffer.line_at_byte(end.saturating_sub(1))))
}

pub(super) fn declaration_end(symbols: &[DocumentSymbol], byte: usize) -> Option<usize> {
    symbols
        .iter()
        .filter_map(|symbol| symbol.find_innermost(byte))
        .filter(|symbol| symbol.selection_range.contains(&byte))
        .min_by_key(|symbol| (symbol.selection_range.start != byte, symbol.range.len()))
        .map(|symbol| symbol.range.end)
}

fn retained_bytes(
    symbols: &[DocumentSymbol],
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<usize, String> {
    let mut bytes = std::mem::size_of_val(symbols);
    let mut pending: Vec<_> = symbols.iter().collect();
    while let Some(symbol) = pending.pop() {
        check_source(cancel, deadline)?;
        bytes += symbol.name.capacity()
            + symbol.detail.as_ref().map_or(0, String::capacity)
            + symbol.children.capacity() * std::mem::size_of::<DocumentSymbol>();
        pending.extend(symbol.children.iter());
    }
    Ok(bytes)
}
