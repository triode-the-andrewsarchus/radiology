use egui::Color32;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightedSpan {
    pub text: String,
    pub color: Color32,
    pub is_bold: bool,
    pub is_italic: bool,
}

pub struct SyntaxHighlighter {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
    pub dark_theme_name: &'static str,
    pub light_theme_name: &'static str,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighter {
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

    pub fn find_syntax_for_path(&self, path: &str) -> Option<&SyntaxReference> {
        self.syntax_set.find_syntax_for_file(path).ok().flatten()
    }

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

    /// Highlights a single line of text given the syntax and active theme.
    pub fn highlight_line(
        &self,
        highlighter: &mut HighlightLines,
        line: &str,
    ) -> Vec<HighlightedSpan> {
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
                    HighlightedSpan {
                        text: text.to_string(),
                        color,
                        is_bold,
                        is_italic,
                    }
                })
                .collect(),
            Err(_) => vec![HighlightedSpan {
                text: line.to_string(),
                color: Color32::from_rgb(220, 220, 220),
                is_bold: false,
                is_italic: false,
            }],
        }
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

        let spans = hl.highlight_line(&mut highlighter, "fn main() {\n");
        assert!(!spans.is_empty());
        let joined: String = spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(joined, "fn main() {\n");
    }
}
