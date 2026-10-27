use std::path::Path;

/// Determines if a file path represents source code, markup, or configuration
/// rather than binary, media, or build artifacts.
pub struct SourceFileClassifier;

impl SourceFileClassifier {
    const SOURCE_EXTENSIONS: &'static [&'static str] = &[
        "rs", "swift", "go", "py", "pyi", "js", "jsx", "ts", "tsx", "mjs", "cjs",
        "c", "h", "cc", "cpp", "cxx", "hpp", "hh", "m", "mm",
        "java", "kt", "kts", "scala", "rb", "php", "cs", "fs",
        "sh", "bash", "zsh", "fish",
        "lua", "zig", "nim", "dart", "ex", "exs", "erl", "clj", "cljs",
        "html", "htm", "css", "scss", "sass", "less",
        "json", "yaml", "yml", "toml", "xml", "plist", "sql", "graphql",
        "md", "markdown", "rst", "adoc", "txt", "diff", "patch",
        "dockerfile", "makefile", "cmake",
    ];

    const IGNORED_EXTENSIONS: &'static [&'static str] = &[
        "png", "jpg", "jpeg", "gif", "webp", "ico", "bmp", "tiff", "svg",
        "mp3", "wav", "flac", "ogg", "aac", "m4a", "mid", "midi",
        "mp4", "mkv", "avi", "mov", "webm",
        "zip", "tar", "gz", "bz2", "xz", "7z", "rar",
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        "o", "a", "so", "dylib", "dll", "exe", "bin", "wasm",
        "lock", "min.js", "min.css", "map",
    ];

    pub fn is_source_file(path: &str) -> bool {
        let p = Path::new(path);
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let filename = p.file_name().and_then(|f| f.to_str()).unwrap_or("").to_lowercase();

        if filename == "cargo.lock" || filename == "package-lock.json" || filename == "yarn.lock" {
            return false;
        }

        if filename == "dockerfile" || filename == "makefile" || filename == "cmakelists.txt" {
            return true;
        }

        if Self::IGNORED_EXTENSIONS.contains(&ext.as_str()) {
            return false;
        }

        Self::SOURCE_EXTENSIONS.contains(&ext.as_str())
    }
}

/// Identifies whether a file path belongs to tests or test fixtures across languages.
pub struct TestPathClassifier;

impl TestPathClassifier {
    pub fn is_test_path(path: &str) -> bool {
        let p = Path::new(path);

        // Check path components for common test directory names
        for component in p.components() {
            let comp_str = component.as_os_str().to_string_lossy().to_lowercase();
            if matches!(
                comp_str.as_str(),
                "tests" | "test" | "spec" | "specs" | "testing" | "fixtures" | "mocks" | "__tests__"
            ) {
                return true;
            }
        }

        // Check filename patterns
        let filename = p.file_name().and_then(|f| f.to_str()).unwrap_or("");
        let lower = filename.to_lowercase();

        if lower.starts_with("test_") || lower.ends_with("_test.go") || lower.ends_with("_test.rs")
            || lower.ends_with(".test.ts") || lower.ends_with(".test.js")
            || lower.ends_with(".test.tsx") || lower.ends_with(".test.jsx")
            || lower.ends_with(".spec.ts") || lower.ends_with(".spec.js")
            || lower.ends_with(".spec.tsx") || lower.ends_with(".spec.jsx")
            || lower.ends_with("_spec.rb") || lower.ends_with("_test.py")
            || filename.ends_with("Test.java") || filename.ends_with("Tests.java")
            || filename.ends_with("Test.kt") || filename.ends_with("Tests.kt")
            || filename.ends_with("Tests.swift") || filename.ends_with("Test.swift")
        {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_classifier() {
        assert!(SourceFileClassifier::is_source_file("src/main.rs"));
        assert!(SourceFileClassifier::is_source_file("Package.swift"));
        assert!(SourceFileClassifier::is_source_file("app.tsx"));
        assert!(SourceFileClassifier::is_source_file("Makefile"));
        assert!(!SourceFileClassifier::is_source_file("image.png"));
        assert!(!SourceFileClassifier::is_source_file("Cargo.lock"));
        assert!(!SourceFileClassifier::is_source_file("binary.exe"));
    }

    #[test]
    fn test_test_classifier() {
        assert!(TestPathClassifier::is_test_path("tests/integration_test.rs"));
        assert!(TestPathClassifier::is_test_path("src/tests/helper.rs"));
        assert!(TestPathClassifier::is_test_path("pkg/service_test.go"));
        assert!(TestPathClassifier::is_test_path("components/Button.test.tsx"));
        assert!(TestPathClassifier::is_test_path("Tests/RadiologyTests/AppTests.swift"));
        assert!(TestPathClassifier::is_test_path("tests/test_model.py"));
        assert!(!TestPathClassifier::is_test_path("src/testing_utils.rs"));
        assert!(!TestPathClassifier::is_test_path("src/core/model.rs"));
    }
}
