//! Syntect fallback highlighting and styled spans for source rendering.

use egui::Color32;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

/// A styled segment of text within a highlighted line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightedSpan {
    /// UTF-8 byte range within the original source line.
    pub bytes: std::ops::Range<usize>,
    /// Text content of this token or span.
    pub text: String,
    /// EGUI text color mapped from the active syntax theme.
    pub color: Color32,
    /// Whether bold typography should be applied.
    pub is_bold: bool,
    /// Whether italic typography should be applied.
    pub is_italic: bool,
}

/// Manages syntax definitions and themes for source text highlighting.
///
/// # Examples
///
/// ```rust
/// use syntect::easy::HighlightLines;
/// use radiology::syntax::highlighter::SyntaxHighlighter;
///
/// let hl = SyntaxHighlighter::new();
/// let syntax = hl.find_syntax_for_path("test.rs").unwrap();
/// let theme = hl.theme(true);
/// let mut line_hl = HighlightLines::new(syntax, theme);
///
/// let spans = hl.highlight_line(&mut line_hl, "let x = 42;\n");
/// assert!(!spans.is_empty());
/// ```
pub struct SyntaxHighlighter {
    /// Precompiled syntax set containing 100+ standard syntaxes.
    pub syntax_set: SyntaxSet,
    /// Bundled syntax themes.
    pub theme_set: ThemeSet,
    /// Active dark theme key.
    pub dark_theme_name: &'static str,
    /// Active light theme key.
    pub light_theme_name: &'static str,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighter {
    /// Creates a new [`SyntaxHighlighter`] with default bundled syntaxes and themes.
    #[must_use]
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        Self {
            syntax_set,
            theme_set,
            dark_theme_name: "base16-ocean.dark",
            light_theme_name: "InspiredGitHub",
        }
    }

    /// Looks up a [`SyntaxReference`] matching the given file path or extension.
    #[must_use]
    pub fn find_syntax_for_path(&self, path: &str) -> Option<&SyntaxReference> {
        if let Some(syntax) = self.syntax_set.find_syntax_for_file(path).ok().flatten() {
            return Some(syntax);
        }
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "ts" | "tsx" | "mts" | "cts" => self.syntax_set.find_syntax_by_extension("js"),
            "toml" => self
                .syntax_set
                .find_syntax_by_extension("ini")
                .or_else(|| self.syntax_set.find_syntax_by_extension("yaml")),
            _ => None,
        }
    }

    /// Returns the theme corresponding to the current light or dark appearance mode.
    #[must_use]
    pub fn theme(&self, dark_mode: bool) -> &Theme {
        let name = if dark_mode {
            self.dark_theme_name
        } else {
            self.light_theme_name
        };
        self.theme_set.themes.get(name).unwrap_or_else(|| {
            self.theme_set
                .themes
                .values()
                .next()
                .expect("syntect theme available")
        })
    }

    /// Highlights a single line of source code, returning styled [`HighlightedSpan`] elements.
    #[must_use]
    pub fn highlight_line(
        &self,
        highlighter: &mut HighlightLines,
        line: &str,
    ) -> Vec<HighlightedSpan> {
        let mut offset = 0;
        match highlighter.highlight_line(line, &self.syntax_set) {
            Ok(ranges) => ranges
                .into_iter()
                .map(|(style, text)| {
                    let color = Color32::from_rgb(
                        style.foreground.r,
                        style.foreground.g,
                        style.foreground.b,
                    );
                    let is_bold = style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::BOLD);
                    let is_italic = style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::ITALIC);
                    let bytes = offset..offset + text.len();
                    offset = bytes.end;
                    HighlightedSpan {
                        bytes,
                        text: text.to_string(),
                        color,
                        is_bold,
                        is_italic,
                    }
                })
                .collect(),
            Err(_) => vec![HighlightedSpan {
                bytes: 0..line.len(),
                text: line.to_string(),
                color: Color32::from_rgb(220, 220, 220),
                is_bold: false,
                is_italic: false,
            }],
        }
    }

    /// Highlights all lines of `text` using `syntax_ref` and the current appearance theme.
    #[must_use]
    pub fn highlight_document(
        &self,
        syntax_ref: Option<&SyntaxReference>,
        text: &str,
        dark_mode: bool,
    ) -> Vec<Vec<HighlightedSpan>> {
        self.highlight_document_cancellable(syntax_ref, text, dark_mode, || false)
            .unwrap_or_default()
    }

    /// Highlights original lines, checking request retirement between line parses.
    #[must_use]
    pub fn highlight_document_cancellable(
        &self,
        syntax_ref: Option<&SyntaxReference>,
        text: &str,
        dark_mode: bool,
        cancelled: impl Fn() -> bool,
    ) -> Option<Vec<Vec<HighlightedSpan>>> {
        let mut out = Vec::new();
        if let Some(syntax) = syntax_ref {
            let mut hl = HighlightLines::new(syntax, self.theme(dark_mode));
            for line in syntect::util::LinesWithEndings::from(text) {
                if cancelled() {
                    return None;
                }
                out.push(self.highlight_line(&mut hl, line));
            }
        } else {
            let default_color = if dark_mode {
                Color32::from_rgb(220, 220, 220)
            } else {
                Color32::from_rgb(30, 30, 30)
            };
            for line in text.lines() {
                if cancelled() {
                    return None;
                }
                out.push(vec![HighlightedSpan {
                    bytes: 0..line.len(),
                    text: line.to_string(),
                    color: default_color,
                    is_bold: false,
                    is_italic: false,
                }]);
            }
        }
        (!cancelled()).then_some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlighter_rust() {
        let hl = SyntaxHighlighter::new();
        let syntax = hl
            .find_syntax_for_path("main.rs")
            .expect("Rust syntax found");
        let theme = hl.theme(true);
        let mut highlighter = HighlightLines::new(syntax, theme);

        let line = "fn café() { /* 😀 */ }\r\n";
        let spans = hl.highlight_line(&mut highlighter, line);
        assert!(!spans.is_empty());
        let joined: String = spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(joined, line);
        for span in &spans {
            assert_eq!(&line[span.bytes.clone()], span.text);
        }
    }
}
