use super::{CodePath, CodePathStep};
use uuid::Uuid;

/// Maximum UTF-8 byte length of one pasted or file-imported trace.
pub const MAX_IMPORT_BYTES: usize = 2 * 1024 * 1024;

/// An atomic import failure with original input-line diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ImportError {
    /// Input exceeds the same two-megabyte limit used by the Swift importer.
    #[error("Code paths must be no larger than 2 MB.")]
    TooLarge,
    /// The first physical input line has no title.
    #[error("Line 1: enter a title for this code path.")]
    MissingTitle,
    /// A record lacks two unescaped field delimiters.
    #[error("Line {line}: expected file:function:description.")]
    Fields {
        /// One-based physical input line, including ignored blank lines.
        line: usize,
    },
    /// A field is empty or a filename is absolute, contains NUL or traverses parents.
    #[error(
        "Line {line}: use a repository-relative file path and nonempty function and description."
    )]
    InvalidStep {
        /// One-based physical input line.
        line: usize,
    },
    /// No records follow the title.
    #[error("Add at least one file:function:description after the title.")]
    MissingSteps,
}

pub(super) fn parse(text: &str) -> Result<CodePath, ImportError> {
    if text.len() > MAX_IMPORT_BYTES {
        return Err(ImportError::TooLarge);
    }
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = normalized.split('\n');
    let title = lines
        .next()
        .unwrap_or_default()
        .trim()
        .trim_matches('\u{feff}');
    if title.is_empty() {
        return Err(ImportError::MissingTitle);
    }
    let mut steps = Vec::new();
    for (offset, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = fields(line);
        let [path, function, description]: [String; 3] = fields
            .try_into()
            .map_err(|_| ImportError::Fields { line: offset + 2 })?;
        let step = CodePathStep {
            id: Uuid::new_v4(),
            path,
            function,
            description,
        };
        if !step.is_valid() {
            return Err(ImportError::InvalidStep { line: offset + 2 });
        }
        steps.push(step);
    }
    if steps.is_empty() {
        return Err(ImportError::MissingSteps);
    }
    Ok(CodePath {
        id: Uuid::new_v4(),
        title: title.into(),
        steps,
    })
}

fn fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut characters = line.chars().peekable();
    while let Some(character) = characters.next() {
        if fields.len() < 2
            && character == '\\'
            && characters
                .peek()
                .is_some_and(|next| matches!(next, ':' | '\\'))
        {
            field.push(characters.next().expect("peeked escape"));
        } else if character == ':' && fields.len() < 2 {
            fields.push(field.trim().to_owned());
            field.clear();
        } else {
            field.push(character);
        }
    }
    fields.push(field.trim().to_owned());
    fields
}
