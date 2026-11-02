//! Optional derived cache; original bytes and semantic coordinates own freshness.
use super::*;
use sha2::{Digest, Sha256};
use std::{io::Write, path::PathBuf};
const VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub(super) struct Cache {
    version: u32,
    pub files: HashMap<String, FileRecord>,
}
impl Default for Cache {
    fn default() -> Self {
        Self {
            version: VERSION,
            files: HashMap::new(),
        }
    }
}

pub(super) fn path(source: &SourceRead, directory: &Path) -> Option<PathBuf> {
    let destination = crate::semantic::indexing::workspace::destination_path(directory).ok()?;
    let root = match source {
        SourceRead::Checkout(root) => root,
        SourceRead::Review(review) => &review.head_root,
        SourceRead::Buffer(_) => return None,
    }
    .canonicalize()
    .ok()?;
    (!destination.starts_with(root)).then(|| destination.join("rust-trait-implementations.json"))
}

pub(super) fn load(path: &Path, cancel: &AtomicBool) -> Cache {
    crate::semantic::loader::read_index_bytes(path, cancel)
        .ok()
        .flatten()
        .and_then(|bytes| serde_json::from_slice::<Cache>(&bytes).ok())
        .filter(|cache| cache.version == VERSION)
        .unwrap_or_default()
}

pub(super) fn key(
    index: &SemanticIndex,
    document: &super::super::ScipDocument,
    buffer: &SourceBuffer,
    cancel: &AtomicBool,
) -> Result<String, String> {
    let mut hash = Sha256::new();
    hash.update(VERSION.to_le_bytes());
    hash.update(buffer.content_digest());
    hash.update(index.default_encoding.to_le_bytes());
    hash.update(document.position_encoding.to_le_bytes());
    for occurrence in &document.occurrences {
        check(cancel)?;
        hash.update((occurrence.symbol.len() as u64).to_le_bytes());
        hash.update(&occurrence.symbol);
        hash.update(occurrence.symbol_roles.to_le_bytes());
        hash.update([u8::from(occurrence.range.is_some())]);
        if let Some(range) = &occurrence.range {
            for value in [
                range.start_line,
                range.start_col,
                range.end_line,
                range.end_col,
            ] {
                hash.update(value.to_le_bytes());
            }
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub(super) fn save(path: &Path, cache: &Cache, cancel: &AtomicBool) {
    let write = || -> Result<(), String> {
        check(cancel)?;
        let parent = path.parent().ok_or("No cache parent")?;
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let mut temporary =
            tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
        serde_json::to_writer(temporary.as_file_mut(), cache).map_err(|error| error.to_string())?;
        temporary.flush().map_err(|error| error.to_string())?;
        check(cancel)?;
        temporary.persist(path).map_err(|error| error.to_string())?;
        Ok(())
    };
    let _ = write();
}
