use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedGrammar {
    Rust,
    Python,
    Go,
    JavaScript,
    TypeScript,
    C,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    pub grammar: Option<SupportedGrammar>,
}

pub struct LanguageRegistry;

impl LanguageRegistry {
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

        if filename == "makefile" {
            return LanguageInfo {
                id: "makefile",
                display_name: "Makefile",
                grammar: None,
            };
        }
        if filename == "dockerfile" {
            return LanguageInfo {
                id: "dockerfile",
                display_name: "Dockerfile",
                grammar: None,
            };
        }

        match ext.as_str() {
            "rs" => LanguageInfo {
                id: "rust",
                display_name: "Rust",
                grammar: Some(SupportedGrammar::Rust),
            },
            "py" | "pyi" => LanguageInfo {
                id: "python",
                display_name: "Python",
                grammar: Some(SupportedGrammar::Python),
            },
            "go" => LanguageInfo {
                id: "go",
                display_name: "Go",
                grammar: Some(SupportedGrammar::Go),
            },
            "js" | "mjs" | "cjs" => LanguageInfo {
                id: "javascript",
                display_name: "JavaScript",
                grammar: Some(SupportedGrammar::JavaScript),
            },
            "jsx" => LanguageInfo {
                id: "jsx",
                display_name: "JavaScript (JSX)",
                grammar: Some(SupportedGrammar::JavaScript),
            },
            "ts" => LanguageInfo {
                id: "typescript",
                display_name: "TypeScript",
                grammar: Some(SupportedGrammar::TypeScript),
            },
            "tsx" => LanguageInfo {
                id: "tsx",
                display_name: "TypeScript (TSX)",
                grammar: Some(SupportedGrammar::TypeScript),
            },
            "c" | "h" => LanguageInfo {
                id: "c",
                display_name: "C",
                grammar: Some(SupportedGrammar::C),
            },
            "cpp" | "cc" | "cxx" | "hpp" | "hh" => LanguageInfo {
                id: "cpp",
                display_name: "C++",
                grammar: Some(SupportedGrammar::C),
            },
            "swift" => LanguageInfo {
                id: "swift",
                display_name: "Swift",
                grammar: None,
            },
            "kt" | "kts" => LanguageInfo {
                id: "kotlin",
                display_name: "Kotlin",
                grammar: None,
            },
            "java" => LanguageInfo {
                id: "java",
                display_name: "Java",
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
            "sh" | "bash" | "zsh" => LanguageInfo {
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
            "css" | "scss" | "sass" => LanguageInfo {
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
            Some(SupportedGrammar::TypeScript)
        );
        assert_eq!(
            LanguageRegistry::for_path("script.py").grammar,
            Some(SupportedGrammar::Python)
        );
        assert_eq!(LanguageRegistry::for_path("Makefile").id, "makefile");
    }
}
