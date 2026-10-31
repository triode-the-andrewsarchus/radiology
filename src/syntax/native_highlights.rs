//! Immutable native highlight captures and source-bound theme recoloring.

use super::{highlighter::HighlightedSpan, SupportedGrammar};
use crate::{
    source::{buffer::SourceIdentity, SourceBuffer},
    theme::AppTheme,
};
use egui::Color32;
use std::{collections::BTreeSet, ops::Range};
use tree_sitter::Node;

#[derive(Debug, Clone)]
struct Capture {
    bytes: Range<usize>,
    name: String,
    pattern: usize,
}

/// Native language captures retained across appearance changes.
#[derive(Debug, Clone, Default)]
pub struct NativeHighlights {
    captures: Vec<Capture>,
    identity: Option<SourceIdentity>,
    /// Recoverable capture failure; callers can keep basic fallback colors.
    pub error: Option<String>,
}

impl NativeHighlights {
    pub(super) fn from_tree(
        buffer: &SourceBuffer,
        grammar: SupportedGrammar,
        root: Node<'_>,
        control: &super::AnalysisControl<'_>,
    ) -> Self {
        if !matches!(
            grammar,
            SupportedGrammar::Rust
                | SupportedGrammar::Swift
                | SupportedGrammar::Kotlin
                | SupportedGrammar::JavaScript
                | SupportedGrammar::TypeScript
                | SupportedGrammar::Tsx
        ) {
            return Self::default();
        }
        match captures(buffer, grammar, root, 200_000, control) {
            Ok(captures) => Self {
                captures,
                identity: Some(SourceIdentity::new(buffer)),
                error: None,
            },
            Err(error) => Self {
                error: Some(error),
                ..Default::default()
            },
        }
    }

    /// Returns whether these native captures describe the exact source and revision.
    #[must_use]
    pub fn matches(&self, buffer: &SourceBuffer) -> bool {
        self.identity
            .as_ref()
            .is_some_and(|identity| identity.matches(buffer))
    }

    /// Recolors exact source spans without another parse or query.
    ///
    /// Returns `None` for mismatched bytes or a retired request.
    #[must_use]
    pub fn recolor(
        &self,
        buffer: &SourceBuffer,
        dark: bool,
        cancelled: impl Fn() -> bool,
    ) -> Option<Vec<Vec<HighlightedSpan>>> {
        if !self.matches(buffer) || cancelled() {
            return None;
        }
        let theme = if dark {
            AppTheme::dark()
        } else {
            AppTheme::light()
        };
        let colors: Vec<_> = self
            .captures
            .iter()
            .map(|capture| role_color(&capture.name, &theme))
            .collect();
        let mut events: Vec<_> = self
            .captures
            .iter()
            .enumerate()
            .flat_map(|(rank, capture)| {
                [
                    (capture.bytes.start, rank, true),
                    (capture.bytes.end, rank, false),
                ]
            })
            .collect();
        events.sort_unstable();
        let mut active = BTreeSet::new();
        let mut event = 0;
        let mut offset = 0;
        let mut output = Vec::new();
        for line in buffer.text.split_inclusive('\n') {
            let end = offset + line.len();
            let mut byte = offset;
            let mut spans: Vec<HighlightedSpan> = Vec::new();
            while byte < end {
                if cancelled() {
                    return None;
                }
                while events.get(event).is_some_and(|entry| entry.0 <= byte) {
                    let (_, rank, start) = events[event];
                    if start {
                        active.insert(rank);
                    } else {
                        active.remove(&rank);
                    }
                    event += 1;
                }
                let stop = events.get(event).map_or(end, |entry| entry.0.min(end));
                let color = active
                    .last()
                    .map_or(theme.text_primary, |&rank| colors[rank]);
                if let Some(last) = spans.last_mut().filter(|last| last.color == color) {
                    last.bytes.end = stop - offset;
                    last.text.push_str(&buffer.text[byte..stop]);
                } else {
                    spans.push(HighlightedSpan {
                        bytes: byte - offset..stop - offset,
                        text: buffer.text[byte..stop].to_owned(),
                        color,
                        is_bold: false,
                        is_italic: false,
                    });
                }
                byte = stop;
            }
            output.push(spans);
            offset = end;
        }
        (!cancelled()).then_some(output)
    }
}

fn captures(
    buffer: &SourceBuffer,
    grammar: SupportedGrammar,
    root: Node<'_>,
    limit: usize,
    control: &super::AnalysisControl<'_>,
) -> Result<Vec<Capture>, String> {
    let mut output = Vec::new();
    super::queries::visit_captures(
        buffer,
        grammar,
        super::queries::QueryRole::Highlights,
        root,
        limit,
        control,
        |name, node, pattern| {
            let bytes = node.byte_range();
            if !name.starts_with('_') && !bytes.is_empty() {
                output.push(Capture {
                    bytes,
                    name: name.to_owned(),
                    pattern,
                });
            }
        },
    )?;
    output.sort_by_key(|capture| {
        (
            capture.bytes.start,
            std::cmp::Reverse(capture.bytes.len()),
            capture.pattern,
        )
    });
    Ok(output)
}

// Corresponding desktop role colors; macOS resolves its dynamic system colors
// at runtime. Every capture uses the reader's regular monospace type.
fn role_color(name: &str, theme: &AppTheme) -> Color32 {
    let role = match name {
        "string.escape" | "string.special" => "number",
        "variable.builtin" => "keyword",
        _ => name.split('.').next().unwrap_or(name),
    };
    let (light, dark) = match role {
        "comment" => return theme.text_secondary,
        "string" => ([255, 59, 48], [255, 69, 58]),
        "number" | "constant" => ([255, 149, 0], [255, 159, 10]),
        "keyword" => ([255, 45, 85], [255, 55, 95]),
        "type" | "constructor" | "namespace" | "tag" => ([48, 176, 199], [64, 200, 224]),
        "function" => return theme.accent,
        "property" => ([88, 86, 214], [94, 92, 230]),
        "attribute" => ([175, 82, 222], [191, 90, 242]),
        "label" => ([162, 132, 94], [172, 142, 104]),
        _ => return theme.text_primary,
    };
    let [r, g, b] = if theme.is_dark { dark } else { light };
    Color32::from_rgb(r, g, b)
}

#[cfg(test)]
mod tests;
