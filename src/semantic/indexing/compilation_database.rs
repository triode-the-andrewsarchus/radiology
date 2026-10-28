//! Compilation commands rebased to disposable build inputs, without shell execution.

use super::inventory::check_cancel;
use super::workspace::normalize;
use crate::semantic::loader::open_regular_file;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

const MAX_BYTES: usize = 32 * 1024 * 1024;
const MAX_ARGUMENTS: usize = 200_000;
const MAX_RESPONSE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Deserialize)]
struct Command {
    directory: PathBuf,
    file: PathBuf,
    arguments: Option<Vec<String>>,
    command: Option<String>,
    output: Option<PathBuf>,
}

#[derive(Serialize)]
struct RebasedCommand {
    directory: PathBuf,
    file: PathBuf,
    arguments: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<PathBuf>,
}

/// Materializable database plus working directories that may have been empty originally.
pub struct RebasedDatabase {
    /// JSON argument arrays; no command string is executed by Radiology.
    pub bytes: Vec<u8>,
    /// Directories exclusively within the disposable workspace.
    pub directories: BTreeSet<PathBuf>,
}

/// Rewrites a Linux compilation database from original paths into a private copy.
///
/// Uses argument arrays preferentially, POSIX splitting for command strings and
/// GNU response-file quoting. Responses are expanded from copied repository files,
/// with bounded depth/bytes/arguments. System/toolchain paths remain external;
/// repository paths and compiler outputs are relocated. Duplicate configurations
/// and non-path macro values are preserved.
///
/// # Errors
///
/// Rejects malformed/oversized databases, escaping main files/working directories,
/// absent copied source or response inputs, invalid quoting, response recursion,
/// unsupported embedded repository-path options and cancellation.
pub fn rebase(
    bytes: &[u8],
    original: &Path,
    workspace: &Path,
    run: &Path,
    cancel: &AtomicBool,
) -> Result<RebasedDatabase, String> {
    check_cancel(cancel)?;
    let context = RebaseContext::new(original, workspace, run, cancel)?;
    let commands = decode_commands(bytes)?;
    let mut result = vec![b'['];
    let mut directories = BTreeSet::new();
    for (position, command) in commands.into_iter().enumerate() {
        check_cancel(cancel)?;
        let command = context
            .command(command)
            .map_err(|error| format!("Compilation command {}: {error}", position + 1))?;
        directories.insert(command.directory.clone());
        let encoded = serde_json::to_vec(&command).map_err(|e| e.to_string())?;
        if result.len() + encoded.len() + 2 > MAX_BYTES {
            return Err("Rebased compilation database exceeds 32 MiB".into());
        }
        if position != 0 {
            result.push(b',');
        }
        result.extend(encoded);
    }
    result.push(b']');
    Ok(RebasedDatabase {
        bytes: result,
        directories,
    })
}

fn decode_commands(bytes: &[u8]) -> Result<Vec<Command>, String> {
    if bytes.len() > MAX_BYTES {
        return Err("Compilation database exceeds 32 MiB".into());
    }
    let commands: Vec<Command> = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if commands.is_empty() || commands.len() > MAX_ARGUMENTS {
        return Err("Compilation database is empty or exceeds 200,000 commands".into());
    }
    Ok(commands)
}

struct RebaseContext<'a> {
    original: PathBuf,
    workspace: PathBuf,
    run: PathBuf,
    cancel: &'a AtomicBool,
}

impl<'a> RebaseContext<'a> {
    fn new(
        original: &Path,
        workspace: &Path,
        run: &Path,
        cancel: &'a AtomicBool,
    ) -> Result<Self, String> {
        let original = normalize(original)?;
        let workspace = normalize(workspace)?;
        let run = normalize(run)?;
        if !original.is_absolute()
            || !workspace.is_absolute()
            || workspace.starts_with(&original)
            || !run.starts_with(&workspace)
        {
            return Err("Compilation database requires absolute original/private run roots".into());
        }
        Ok(Self {
            original,
            workspace,
            run,
            cancel,
        })
    }

    fn absolute(&self, directory: &Path, path: &Path) -> Result<PathBuf, String> {
        normalize(&directory.join(path))
    }

    fn repository_path(&self, path: &Path) -> Result<PathBuf, String> {
        let relative = path.strip_prefix(&self.original).map_err(|_| {
            format!(
                "Compilation input is outside the repository: {}",
                path.display()
            )
        })?;
        Ok(self.workspace.join(relative))
    }

    fn input_path(&self, directory: &Path, path: &Path) -> Result<PathBuf, String> {
        let copied = self.repository_path(&self.absolute(directory, path)?)?;
        let canonical = std::fs::canonicalize(&copied).map_err(|e| format!("Copied input: {e}"))?;
        if !canonical.starts_with(&self.workspace) {
            return Err("Compilation input escapes the copied workspace".into());
        }
        Ok(canonical)
    }

    fn path(&self, directory: &Path, value: &str, output: bool) -> Result<String, String> {
        if value.is_empty() || value.starts_with('=') {
            return Ok(value.into());
        }
        let absolute = self.absolute(directory, Path::new(value))?;
        let rebased = if let Ok(relative) = absolute.strip_prefix(&self.original) {
            self.workspace.join(relative)
        } else if output {
            let key = format!(
                "{:x}",
                Sha256::digest(absolute.as_os_str().as_encoded_bytes())
            );
            self.run.join(format!("external-output-{key}"))
        } else {
            absolute
        };
        rebased
            .to_str()
            .map(str::to_owned)
            .ok_or("Compilation path is not UTF-8".into())
    }

    fn command(&self, command: Command) -> Result<RebasedCommand, String> {
        let directory = self.absolute(&self.original, &command.directory)?;
        let copied_directory = self.repository_path(&directory)?;
        let file = self.input_path(&directory, &command.file)?;
        if !file.is_file() {
            return Err(format!(
                "Source was not copied into the workspace: {}",
                command.file.display()
            ));
        }
        let arguments = match command.arguments {
            Some(arguments) => arguments,
            None => shell_arguments(
                command
                    .command
                    .as_deref()
                    .ok_or("Missing arguments or command")?,
            )?,
        };
        if arguments.is_empty() || arguments[0].is_empty() {
            return Err("Missing compiler executable".into());
        }
        let mut expanded = Vec::new();
        let mut response_bytes = 0;
        self.expand(&directory, arguments, 0, &mut response_bytes, &mut expanded)?;
        let arguments = self.arguments(&directory, expanded)?;
        let output = command
            .output
            .as_ref()
            .map(|output| {
                self.path(
                    &directory,
                    output.to_str().ok_or("Output is not UTF-8")?,
                    true,
                )
                .map(PathBuf::from)
            })
            .transpose()?;
        Ok(RebasedCommand {
            directory: copied_directory,
            file,
            arguments,
            output,
        })
    }

    fn expand(
        &self,
        directory: &Path,
        arguments: Vec<String>,
        depth: usize,
        response_bytes: &mut u64,
        expanded: &mut Vec<String>,
    ) -> Result<(), String> {
        if depth > 16 {
            return Err("Response files exceed 16 nesting levels (possible cycle)".into());
        }
        for argument in arguments {
            check_cancel(self.cancel)?;
            if let Some(path) = argument.strip_prefix('@') {
                let arguments = self.response(directory, path, response_bytes)?;
                self.expand(directory, arguments, depth + 1, response_bytes, expanded)?;
            } else {
                if expanded.len() >= MAX_ARGUMENTS || argument.contains('\0') {
                    return Err("Compilation arguments exceed limits or contain NUL".into());
                }
                expanded.push(argument);
            }
        }
        Ok(())
    }

    fn response(
        &self,
        directory: &Path,
        path: &str,
        total: &mut u64,
    ) -> Result<Vec<String>, String> {
        let canonical = self.input_path(directory, Path::new(path))?;
        let file = open_regular_file(&canonical).map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        file.take(MAX_RESPONSE_BYTES.saturating_sub(*total) + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        *total += bytes.len() as u64;
        if *total > MAX_RESPONSE_BYTES {
            return Err("Response files exceed 8 MiB per compilation command".into());
        }
        gnu_arguments(std::str::from_utf8(&bytes).map_err(|e| e.to_string())?)
    }

    fn arguments(&self, directory: &Path, arguments: Vec<String>) -> Result<Vec<String>, String> {
        let mut rebased = Vec::with_capacity(arguments.len());
        let mut operands = arguments.into_iter();
        if let Some(executable) = operands.next() {
            // Resolve repository-local wrappers; retain normal PATH/toolchain identity.
            rebased.push(self.bare_argument(directory, &executable)?);
        }
        while let Some(argument) = operands.next() {
            check_cancel(self.cancel)?;
            if let Some(output) = path_option(&argument) {
                let operand = operands
                    .next()
                    .ok_or_else(|| format!("Missing operand for {argument}"))?;
                rebased.push(argument);
                rebased.push(self.path(directory, &operand, output)?);
            } else if let Some((prefix, value, output)) = joined_path_option(&argument) {
                rebased.push(format!("{prefix}{}", self.path(directory, value, output)?));
            } else if ["-D", "-U", "-x", "-MT", "-MQ", "-target"].contains(&argument.as_str()) {
                let operand = operands
                    .next()
                    .ok_or_else(|| format!("Missing operand for {argument}"))?;
                rebased.extend([argument, operand]);
            } else {
                rebased.push(self.bare_argument(directory, &argument)?);
            }
        }
        Ok(rebased)
    }

    fn bare_argument(&self, directory: &Path, argument: &str) -> Result<String, String> {
        if argument.starts_with('-') {
            if !argument.starts_with("-D")
                && argument.contains(self.original.to_str().ok_or("Root is not UTF-8")?)
            {
                return Err(format!("Unsupported repository-path option: {argument}"));
            }
            return Ok(argument.into());
        }
        let absolute = self.absolute(directory, Path::new(argument))?;
        if Path::new(argument).is_absolute()
            || self
                .repository_path(&absolute)
                .is_ok_and(|path| path.exists())
        {
            self.path(directory, argument, false)
        } else {
            Ok(argument.into())
        }
    }
}

fn path_option(option: &str) -> Option<bool> {
    match option {
        "-o" | "-MF" | "-MJ" | "-fmodules-cache-path" | "-fmodule-output" => Some(true),
        "-I" | "-isystem" | "-iquote" | "-idirafter" | "-include" | "-include-pch" | "-imacros"
        | "-isysroot" | "--sysroot" | "-resource-dir" | "-fmodule-map-file" => Some(false),
        _ => None,
    }
}

fn joined_path_option(argument: &str) -> Option<(&str, &str, bool)> {
    for (prefix, output) in [
        ("-fmodules-cache-path=", true),
        ("-fmodule-output=", true),
        ("-fmodule-map-file=", false),
        ("--sysroot=", false),
        ("-resource-dir=", false),
        ("-isystem", false),
        ("-iquote", false),
        ("-idirafter", false),
        ("-isysroot", false),
        ("-include", false),
        ("-imacros", false),
        ("-MF", true),
        ("-MJ", true),
        ("-I", false),
    ] {
        if let Some(value) = argument.strip_prefix(prefix) {
            if !value.is_empty() {
                return Some((prefix, value, output));
            }
        }
    }
    None
}

// Compilation commands use shell quoting but have no shell expansion or comments.
// Protect unquoted # from shlex's shell-comment extension without changing quoted
// literals; the established parser still owns quote/backslash decoding.
fn shell_arguments(command: &str) -> Result<Vec<String>, String> {
    let mut literal = String::with_capacity(command.len());
    let mut quote = None;
    let mut chars = command.chars();
    while let Some(character) = chars.next() {
        literal.push(character);
        match character {
            '\\' if quote != Some('\'') => {
                if let Some(next) = chars.next() {
                    literal.push(next);
                }
            }
            '#' if quote.is_none() => {
                literal.pop();
                literal.push_str("\\#");
            }
            '\'' | '"' if quote.is_none() => quote = Some(character),
            _ if Some(character) == quote => quote = None,
            _ => {}
        }
    }
    shlex::split(&literal).ok_or("Invalid command quoting".into())
}

// LLVM GNU response quoting differs from POSIX shell quoting (notably backslashes
// inside single quotes, literal # and whitespace). Do not use shell evaluation.
fn gnu_arguments(text: &str) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut active = false;
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\\' {
            let next = chars.peek().copied();
            if next
                .is_some_and(|next| next.is_ascii_whitespace() || matches!(next, '\\' | '\'' | '"'))
            {
                word.push(chars.next().expect("peeked character"));
            } else {
                word.push(character);
            }
            active = true;
        } else if Some(character) == quote {
            quote = None;
        } else if quote.is_none() && matches!(character, '\'' | '"') {
            quote = Some(character);
            active = true;
        } else if quote.is_none() && character.is_ascii_whitespace() {
            if active {
                words.push(std::mem::take(&mut word));
                active = false;
            }
        } else {
            word.push(character);
            active = true;
        }
    }
    if quote.is_some() {
        return Err("Unterminated response-file quote".into());
    }
    if active {
        words.push(word);
    }
    Ok(words)
}

#[cfg(test)]
mod tests;
