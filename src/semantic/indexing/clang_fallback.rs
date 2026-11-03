//! Source-only Clang defaults and inferred repository include paths.
use super::{
    compilation_database::DatabaseCommand,
    inventory::{check_cancel, validate_input_path, InputInventory},
    Indexer,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, LazyLock},
};

/// Versioned strategy evidence, separate from every real compilation database.
pub const MARKER: &str = "Radiology best-effort C/C++ defaults v1";
/// Coverage explanation retained for generated, reused and cached fallback results.
pub const EXPLANATION: &str = "Best effort: no compilation database. Uses default GNU C/C++ settings and inferred repository include paths. Missing headers, defines, targets or custom preprocessing can leave symbols unresolved. A real compile_commands.json improves coverage.";
const MAX_DIRECTORIES: usize = 64;
static INCLUDE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"(?m)^\s*#\s*include\s*[<"]([^>"]+)[>"]"#).expect("literal include regex")
});

fn is_unit(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "c" | "cc" | "cpp" | "cxx" | "c++"
            )
        })
}

/// Generates a standard argument-array database from an isolated source inventory.
/// Every translation unit is included; headers substitute only for header-only inputs.
/// This writes nothing and never runs a build. The caller stages bytes under its run
/// directory and must always retain the Best effort/Partial coverage explanation.
///
/// # Examples
/// ```
/// use radiology::semantic::indexing::{clang_fallback, inventory::InputInventory};
/// use std::{path::Path, sync::atomic::AtomicBool};
/// let copy = tempfile::tempdir()?;
/// std::fs::write(copy.path().join("boot.c"), "void boot(void) {}\n")?;
/// let inventory = InputInventory::from_paths(vec!["boot.c".into()])?;
/// let bytes = clang_fallback::database(copy.path(), &inventory, Path::new("/usr/bin/clang"), &AtomicBool::new(false))?;
/// let entries: serde_json::Value = serde_json::from_slice(&bytes)?;
/// assert_eq!(entries[0]["arguments"][3], "-std=gnu11");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// Rejects invalid roots/paths, absent C/C++ inputs, non-UTF-8 compiler paths,
/// serialization failures and cancellation. Unreadable includes stay compiler diagnostics.
pub fn database(
    root: &Path,
    inventory: &InputInventory,
    compiler: &Path,
    cancel: &AtomicBool,
) -> Result<Vec<u8>, String> {
    check_cancel(cancel)?;
    if !root.is_absolute() || !compiler.is_absolute() {
        return Err("Clang defaults require absolute workspace/compiler paths".into());
    }
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let compiler = compiler.to_str().ok_or("Compiler path is not UTF-8")?;
    let mut inputs = BTreeSet::new();
    let mut known_directories = BTreeSet::new();
    for path in &inventory.paths {
        check_cancel(cancel)?;
        validate_input_path(path)?;
        for parent in Path::new(path)
            .parent()
            .expect("relative file parent")
            .ancestors()
        {
            known_directories.insert(parent.to_str().expect("UTF-8 parent"));
        }
        if Indexer::Clang.covers_path(path) {
            inputs.insert(path.as_str());
        }
    }
    let mut inference = Inference {
        root: &root,
        headers: BTreeMap::new(),
        known_directories,
        include_cache: HashMap::new(),
        directory_cache: HashMap::new(),
        cancel,
    };
    for path in inputs.iter().copied().filter(|path| !is_unit(path)) {
        inference
            .headers
            .entry(
                Path::new(path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .expect("UTF-8 file name"),
            )
            .or_default()
            .push(path);
    }
    let units: Vec<_> = inputs
        .iter()
        .copied()
        .filter(|path| is_unit(path))
        .collect();
    let sources: Vec<_> = if units.is_empty() {
        inputs.into_iter().collect()
    } else {
        units
    };
    if sources.is_empty() {
        return Err("No C/C++ sources are available for best-effort indexing".into());
    }
    let mut entries = Vec::with_capacity(sources.len());
    for path in sources {
        check_cancel(cancel)?;
        let directories = inference.directories(path)?;
        let c = crate::syntax::LanguageRegistry::for_path(path).id == "c";
        let mut arguments = vec![
            compiler.into(),
            "-x".into(),
            if c { "c" } else { "c++" }.into(),
            if c { "-std=gnu11" } else { "-std=gnu++20" }.into(),
            "-w".into(),
            "-ferror-limit=5".into(),
        ];
        for directory in directories {
            arguments.extend([
                "-I".into(),
                root.join(directory)
                    .to_str()
                    .ok_or("Include path is not UTF-8")?
                    .into(),
            ]);
        }
        arguments.extend([
            "-c".into(),
            if path.starts_with('-') {
                format!("./{path}")
            } else {
                path.into()
            },
        ]);
        entries.push(DatabaseCommand {
            directory: root.clone(),
            file: PathBuf::from(path),
            arguments,
            output: None,
        });
    }
    check_cancel(cancel)?;
    serde_json::to_vec(&entries).map_err(|error| error.to_string())
}

struct Inference<'a> {
    root: &'a Path,
    headers: BTreeMap<&'a str, Vec<&'a str>>,
    known_directories: BTreeSet<&'a str>,
    include_cache: HashMap<&'a str, Vec<String>>,
    directory_cache: HashMap<&'a str, Vec<String>>,
    cancel: &'a AtomicBool,
}

impl<'a> Inference<'a> {
    fn directories(&mut self, path: &'a str) -> Result<Vec<String>, String> {
        let mut directories = Vec::new();
        for parent in Path::new(path).parent().expect("source parent").ancestors() {
            add_directory(&mut directories, parent.to_str().expect("UTF-8 parent"));
            for name in ["include", "inc", "src"] {
                let directory = parent.join(name);
                let directory = directory.to_str().expect("UTF-8 directory");
                if self.known_directories.contains(directory) {
                    add_directory(&mut directories, directory);
                }
            }
        }
        for directory in self.referenced(path, &mut HashSet::new())? {
            add_directory(&mut directories, &directory);
        }
        Ok(directories)
    }

    fn includes(&mut self, path: &'a str) -> Result<Vec<String>, String> {
        if let Some(cached) = self.include_cache.get(path) {
            return Ok(cached.clone());
        }
        let bytes = self
            .root
            .join(path)
            .canonicalize()
            .ok()
            .filter(|path| path.starts_with(self.root))
            .and_then(|path| {
                crate::semantic::loader::read_index_bytes(&path, self.cancel)
                    .ok()
                    .flatten()
            })
            .unwrap_or_default();
        check_cancel(self.cancel)?;
        let text = String::from_utf8_lossy(&bytes);
        let names: Vec<_> = INCLUDE
            .captures_iter(&text)
            .map(|capture| capture[1].to_owned())
            .collect();
        self.include_cache.insert(path, names.clone());
        Ok(names)
    }

    fn referenced(
        &mut self,
        path: &'a str,
        active: &mut HashSet<&'a str>,
    ) -> Result<Vec<String>, String> {
        check_cancel(self.cancel)?;
        if let Some(cached) = self.directory_cache.get(path) {
            return Ok(cached.clone());
        }
        if active.len() >= 128 || !active.insert(path) {
            return Ok(Vec::new());
        }
        let mut directories = Vec::new();
        for name in self.includes(path)? {
            for header in self.candidates(path, &name) {
                add_directory(
                    &mut directories,
                    header[..header.len() - name.len()].trim_end_matches('/'),
                );
                for nested in self.referenced(header, active)? {
                    add_directory(&mut directories, &nested);
                }
                if directories.len() >= MAX_DIRECTORIES {
                    break;
                }
            }
            if directories.len() >= MAX_DIRECTORIES {
                break;
            }
        }
        active.remove(path);
        self.directory_cache.insert(path, directories.clone());
        Ok(directories)
    }

    fn candidates(&self, input: &str, name: &str) -> Vec<&'a str> {
        let basename = Path::new(name)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        let suffix = format!("/{name}");
        let mut headers: Vec<_> = self
            .headers
            .get(basename)
            .into_iter()
            .flatten()
            .copied()
            .filter(|header| *header == name || header.ends_with(&suffix))
            .collect();
        let parent = input.rsplit_once('/').map(|(parent, _)| parent);
        let proximity = |header: &str| {
            parent.map_or(0, |parent| {
                parent
                    .split('/')
                    .zip(header.split('/'))
                    .take_while(|(left, right)| left == right)
                    .count()
            })
        };
        headers.sort_by(|left, right| {
            proximity(right)
                .cmp(&proximity(left))
                .then_with(|| left.cmp(right))
        });
        headers.truncate(8);
        headers
    }
}

fn add_directory(directories: &mut Vec<String>, directory: &str) {
    if directories.len() < MAX_DIRECTORIES && !directories.iter().any(|known| known == directory) {
        directories.push(directory.into());
    }
}

#[cfg(test)]
mod tests;
