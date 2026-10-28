//! Language registry and Tree-sitter grammar associations.

use std::path::Path;

/// Enumerates Tree-sitter grammars supported with native parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedGrammar {
    /// Rust language grammar.
    Rust,
    /// Python language grammar.
    Python,
    /// Go language grammar.
    Go,
    /// JavaScript language grammar.
    JavaScript,
    /// TypeScript language grammar.
    TypeScript,
    /// TSX and JSX language grammar with React element support.
    Tsx,
    /// Swift language grammar.
    Swift,
    /// Kotlin language grammar.
    Kotlin,
    /// C and C++ language grammar.
    C,
}

impl SupportedGrammar {
    /// Returns the statically linked grammar used by all syntax analyses.
    #[must_use]
    pub fn language(self) -> tree_sitter::Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::Go => tree_sitter_go::LANGUAGE.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Self::C => tree_sitter_c::LANGUAGE.into(),
            Self::Swift => tree_sitter_swift::LANGUAGE.into(),
            Self::Kotlin => tree_sitter_kotlin_ng::LANGUAGE.into(),
        }
    }

    /// Parses exact source bytes, preserving Tree-sitter's original coordinates.
    #[must_use]
    pub fn parse(self, bytes: &[u8]) -> Option<tree_sitter::Tree> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&self.language()).ok()?;
        parser.parse(bytes, None)
    }
}

/// Stores display metadata and associated parsing grammar for a language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageInfo {
    /// Stable language identifier (e.g. `"rust"`).
    pub id: &'static str,
    /// Human-readable display name (e.g. `"Rust"`).
    pub display_name: &'static str,
    /// Native Tree-sitter grammar if supported for outline parsing.
    pub grammar: Option<SupportedGrammar>,
}

/// Maps file paths and extensions to canonical language metadata.
///
/// # Examples
///
/// ```rust
/// use radiology::syntax::language::{LanguageRegistry, SupportedGrammar};
///
/// let info = LanguageRegistry::for_path("src/main.rs");
/// assert_eq!(info.display_name, "Rust");
/// assert_eq!(info.grammar, Some(SupportedGrammar::Rust));
/// ```
pub struct LanguageRegistry;

impl LanguageRegistry {
    /// Returns the [`LanguageInfo`] corresponding to the given file path.
    #[must_use]
    pub fn for_path(path: &str) -> LanguageInfo {
        let p = Path::new(path);
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let filename = p
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("")
            .to_lowercase();

        if filename == "dockerfile" {
            return LanguageInfo {
                id: "dockerfile",
                display_name: "Dockerfile",
                grammar: None,
            };
        }
        if filename == "makefile" || filename == "gnumakefile" {
            return LanguageInfo {
                id: "makefile",
                display_name: "Makefile",
                grammar: None,
            };
        }
        if filename == "cmakelists.txt" {
            return LanguageInfo {
                id: "cmake",
                display_name: "CMake",
                grammar: None,
            };
        }

        match ext.as_str() {
            "rs" => LanguageInfo {
                id: "rust",
                display_name: "Rust",
                grammar: Some(SupportedGrammar::Rust),
            },
            "go" => LanguageInfo {
                id: "go",
                display_name: "Go",
                grammar: Some(SupportedGrammar::Go),
            },
            "py" | "pyi" => LanguageInfo {
                id: "python",
                display_name: "Python",
                grammar: Some(SupportedGrammar::Python),
            },
            "ts" | "mts" | "cts" => LanguageInfo {
                id: "typescript",
                display_name: "TypeScript",
                grammar: Some(SupportedGrammar::TypeScript),
            },
            "tsx" | "jsx" => LanguageInfo {
                id: "tsx",
                display_name: "TSX",
                grammar: Some(SupportedGrammar::Tsx),
            },
            "js" | "mjs" | "cjs" => LanguageInfo {
                id: "javascript",
                display_name: "JavaScript",
                grammar: Some(SupportedGrammar::JavaScript),
            },
            "c" | "h" => LanguageInfo {
                id: "c",
                display_name: "C",
                grammar: Some(SupportedGrammar::C),
            },
            "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hh" | "hxx" | "h++" | "inl" | "ipp" => {
                LanguageInfo {
                    id: "cpp",
                    display_name: "C++",
                    grammar: Some(SupportedGrammar::C),
                }
            }
            "swift" => LanguageInfo {
                id: "swift",
                display_name: "Swift",
                grammar: Some(SupportedGrammar::Swift),
            },
            "kt" | "kts" => LanguageInfo {
                id: "kotlin",
                display_name: "Kotlin",
                grammar: Some(SupportedGrammar::Kotlin),
            },
            "java" => LanguageInfo {
                id: "java",
                display_name: "Java",
                grammar: None,
            },
            "scala" | "sc" => LanguageInfo {
                id: "scala",
                display_name: "Scala",
                grammar: None,
            },
            "rb" => LanguageInfo {
                id: "ruby",
                display_name: "Ruby",
                grammar: None,
            },
            "json" => LanguageInfo {
                id: "json",
                display_name: "JSON",
                grammar: None,
            },
            "toml" => LanguageInfo {
                id: "toml",
                display_name: "TOML",
                grammar: None,
            },
            "yaml" | "yml" => LanguageInfo {
                id: "yaml",
                display_name: "YAML",
                grammar: None,
            },
            "md" | "markdown" => LanguageInfo {
                id: "markdown",
                display_name: "Markdown",
                grammar: None,
            },
            "sh" | "bash" | "zsh" | "fish" => LanguageInfo {
                id: "shell",
                display_name: "Shell",
                grammar: None,
            },
            "sql" => LanguageInfo {
                id: "sql",
                display_name: "SQL",
                grammar: None,
            },
            "html" | "htm" => LanguageInfo {
                id: "html",
                display_name: "HTML",
                grammar: None,
            },
            "css" | "scss" | "sass" | "less" => LanguageInfo {
                id: "css",
                display_name: "CSS",
                grammar: None,
            },
            _ => LanguageInfo {
                id: "text",
                display_name: "Plain Text",
                grammar: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(
            LanguageRegistry::for_path("src/lib.rs").display_name,
            "Rust"
        );
        assert_eq!(
            LanguageRegistry::for_path("pkg/service.go").display_name,
            "Go"
        );
        assert_eq!(
            LanguageRegistry::for_path("index.tsx").grammar,
            Some(SupportedGrammar::Tsx)
        );
        assert_eq!(
            LanguageRegistry::for_path("app.ts").grammar,
            Some(SupportedGrammar::TypeScript)
        );
        assert_eq!(
            LanguageRegistry::for_path("script.py").grammar,
            Some(SupportedGrammar::Python)
        );
        assert_eq!(LanguageRegistry::for_path("Makefile").id, "makefile");
    }
}
