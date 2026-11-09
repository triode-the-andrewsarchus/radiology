//! Repository file classifiers for source files and test paths.
//!
//! Provides [`SourceFileClassifier`] to filter out build artifacts, caches, documentation,
//! manifests, and generated files, and [`TestPathClassifier`] to identify test files and directories.

use std::collections::HashSet;
use std::path::Path;
use std::sync::LazyLock;

/// Determines whether a file path qualifies as a source code file for "Source Files Only" filtering.
pub struct SourceFileClassifier;

static SOURCE_EXTENSIONS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "swift", "rs", "go", "py", "pyi", "js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts",
        "kt", "kts", "java", "scala", "sc", "groovy", "c", "h", "cc", "cpp", "cxx", "c++", "hh",
        "hpp", "hxx", "h++", "inl", "ipp", "tpp", "tcc", "m", "mm", "cs", "fs", "fsi", "fsx", "rb",
        "php", "lua", "zig", "nim", "dart", "ex", "exs", "erl", "hrl", "hs", "lhs", "ml", "mli",
        "clj", "cljs", "cljc", "elm", "jl", "r", "pl", "pm", "sh", "bash", "zsh", "fish", "sql",
        "proto", "thrift", "graphql", "gql", "vue", "svelte", "astro", "s", "asm", "cu", "cuh",
        "metal", "glsl", "vert", "frag", "hlsl", "wgsl", "sv", "vhd", "vhdl", "ada", "adb", "ads",
        "f90", "f95", "pas", "cr", "gleam", "odin", "sol", "move", "cairo",
    ]
    .into_iter()
    .collect()
});

static EXCLUDED_DIRECTORIES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "target",
        "build",
        "dist",
        "out",
        "DerivedData",
        "node_modules",
        "vendor",
        "Pods",
        "Carthage",
        "bower_components",
        "jspm_packages",
        "coverage",
        "__pycache__",
        "venv",
        "site-packages",
        "generated",
        "__generated__",
    ]
    .into_iter()
    .collect()
});

static MANIFEST_NAMES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "Package.swift",
        "build.gradle.kts",
        "settings.gradle.kts",
        "setup.py",
    ]
    .into_iter()
    .collect()
});

static CONFIG_SCRIPT_EXTENSIONS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    ["js", "ts", "mjs", "cjs", "mts", "cts"]
        .into_iter()
        .collect()
});

const GENERATED_SUFFIXES: &[&str] = &[
    ".min.js",
    ".min.css",
    ".bundle.js",
    ".g.dart",
    ".pb.go",
    "_pb2.py",
    "_pb2_grpc.py",
    ".pb.swift",
];

impl SourceFileClassifier {
    /// Returns `true` if `path` is handwritten source code, excluding build outputs,
    /// documentation, data files, manifests, and generated files.
    ///
    /// # Examples
    ///
    /// ```
    /// use radiology::repository::classifier::SourceFileClassifier;
    ///
    /// assert!(SourceFileClassifier::is_source_file("src/lib.rs"));
    /// assert!(SourceFileClassifier::is_source_file("pkg/service.go"));
    /// assert!(!SourceFileClassifier::is_source_file("README.md"));
    /// assert!(!SourceFileClassifier::is_source_file("Cargo.toml"));
    /// assert!(!SourceFileClassifier::is_source_file("Package.swift"));
    /// assert!(!SourceFileClassifier::is_source_file("target/debug/build.rs"));
    /// ```
    #[must_use]
    pub fn is_source_file(path: &str) -> bool {
        let components: Vec<&str> = path.split('/').collect();
        let Some(&name) = components.last() else {
            return false;
        };

        if name.starts_with('.') || name.is_empty() {
            return false;
        }

        // Check directory components: no hidden directories and no excluded directories
        for &dir in components.iter().take(components.len().saturating_sub(1)) {
            if dir.starts_with('.') || EXCLUDED_DIRECTORIES.contains(dir) {
                return false;
            }
        }

        let p = Path::new(name);
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !SOURCE_EXTENSIONS.contains(ext.as_str()) {
            return false;
        }

        if MANIFEST_NAMES.contains(name) {
            return false;
        }

        for &suffix in GENERATED_SUFFIXES {
            if name.ends_with(suffix) {
                return false;
            }
        }

        if name.contains(".generated.") {
            return false;
        }

        // Config scripts: webpack.config.js, vite.config.ts, jest.config.cjs, etc.
        if CONFIG_SCRIPT_EXTENSIONS.contains(ext.as_str()) {
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                if stem.ends_with(".config") {
                    return false;
                }
            }
        }

        true
    }
}

/// Identifies whether a file path or directory belongs to test code.
pub struct TestPathClassifier;

static TEST_DIRECTORIES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "test",
        "tests",
        "Tests",
        "__tests__",
        "__mocks__",
        "__fixtures__",
        "testdata",
        "test-support",
        "test_support",
        "testsupport",
        "test-utils",
        "test_utils",
        "testutils",
        "androidTest",
        "testFixtures",
    ]
    .into_iter()
    .collect()
});

static TEST_SOURCE_EXTENSIONS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts", "py", "rs", "go", "kt", "kts",
        "java", "swift", "rb", "c", "cc", "cpp", "cxx", "m", "mm", "scala", "dart",
    ]
    .into_iter()
    .collect()
});

impl TestPathClassifier {
    /// Returns `true` if `path` is a whole test file or lies within a test directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use radiology::repository::classifier::TestPathClassifier;
    ///
    /// assert!(TestPathClassifier::is_test_path("tests/common.rs"));
    /// assert!(TestPathClassifier::is_test_path("src/lib_test.go"));
    /// assert!(TestPathClassifier::is_test_path("src/app.test.ts"));
    /// assert!(TestPathClassifier::is_test_path("src/conftest.py"));
    /// assert!(!TestPathClassifier::is_test_path("src/main.rs"));
    /// assert!(!TestPathClassifier::is_test_path("pkg/testing/helpers.go"));
    /// assert!(!TestPathClassifier::is_test_path("src/main/kotlin/inc/SpeedTest.kt"));
    /// ```
    #[must_use]
    pub fn is_test_path(path: &str) -> bool {
        let components: Vec<&str> = path.split('/').collect();
        let Some(&name) = components.last() else {
            return false;
        };

        let dirs = &components[..components.len().saturating_sub(1)];

        // Check exact test directory names
        for &dir in dirs {
            if TEST_DIRECTORIES.contains(dir) {
                return true;
            }
        }

        // Gradle/Kotlin source sets: src/jvmTest, src/commonTest, src/androidUnitTest, etc.
        for (i, &dir) in dirs.iter().enumerate() {
            if i > 0
                && dirs[i - 1] == "src"
                && dir.len() > 4
                && dir.ends_with("Test")
                && dir.chars().next().is_some_and(|c| c.is_lowercase())
            {
                return true;
            }
        }

        Self::is_test_file_name(name)
    }

    /// Determines whether the given filename follows test naming conventions for known source extensions.
    #[must_use]
    pub fn is_test_file_name(name: &str) -> bool {
        let p = Path::new(name);
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !TEST_SOURCE_EXTENSIONS.contains(ext.as_str()) {
            return false;
        }

        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        // Generic patterns: a.test.ts, a.spec.tsx, build.test.py, a.test-d.ts
        if stem.ends_with(".test") || stem.ends_with(".spec") || stem.ends_with(".test-d") {
            return true;
        }

        match ext.as_str() {
            "go" => stem.ends_with("_test"),
            "rs" => stem == "tests" || stem.ends_with("_test") || stem.ends_with("_tests"),
            "py" => {
                stem.starts_with("test_")
                    || stem.ends_with("_test")
                    || stem == "tests"
                    || stem == "conftest"
            }
            "c" | "cc" | "cpp" | "cxx" => stem.ends_with("_test") || stem.ends_with("_unittest"),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Full checked-in Swift SourceFilterTests path matrix, plus prior Rust cases.
    #[test]
    fn test_source_classifier() {
        for path in [
            "modules/agent/ffi/src/app.rs",
            "fonts/src/main/kotlin/inc/tk/os/fonts/FontFiles.kt",
            "scripts/pr-local.py",
            "web/App.tsx",
            "include/codec.h",
            "native/codec.hpp",
            "native/codec.cc",
            "native/templates.tpp",
            "native/templates.tcc",
            "ios/Bridge.m",
            "ios/Bridge.mm",
            "Sources/Radiology/AppModel.swift",
            "java/Main.java",
            "tools/run.sh",
            "db/schema.sql",
            "api/service.proto",
            "build.rs",
            "src/bin/cli.rs",
            "lib/src/test/kotlin/X.kt",
            "types/global.d.ts",
            "src/contest.rs",
            "web/src/main.mjs",
            "scripts/release.kts",
            "src/lib.rs",
            "frontend/components/App.tsx",
            "backend/main.go",
            "scripts/deploy.py",
            "pkg/service.go",
        ] {
            assert!(SourceFileClassifier::is_source_file(path), "{path}");
        }
        for path in [
            "target/debug/build/foo/out/bindings.rs",
            "crate/target/release/x.rs",
            "node_modules/react/index.js",
            "web/node_modules/a/b.ts",
            "app/build/generated/source/R.kt",
            "dist/bundle.js",
            "out/main.js",
            "vendor/github.com/x/y.go",
            ".venv/lib/python3.12/site-packages/x.py",
            "pkg/__pycache__/m.py",
            ".gradle/caches/x.kt",
            ".next/server/page.js",
            "coverage/lcov-report/a.js",
            "Pods/AFNetworking/A.m",
            "proto/generated/a.pb.go",
            "api/a.pb.go",
            "api/a_pb2.py",
            "web/vendor.min.js",
            "tools/tk-apps/home/bricks.generated.ts",
            "README.md",
            "docs/guide.md",
            "notes.txt",
            "Cargo.toml",
            "Cargo.lock",
            "package.json",
            "tsconfig.json",
            "Package.swift",
            "app/build.gradle.kts",
            "settings.gradle.kts",
            "setup.py",
            "vite.config.ts",
            "jest.config.cjs",
            ".eslintrc.js",
            "Makefile",
            "Dockerfile",
            "web/index.html",
            "web/site.css",
            "assets/logo.png",
            ".github/workflows/ci.yml",
            "build.gradle.kts",
            "dist/bundle.min.js",
            "vendor.min.js",
            "proto/api.pb.go",
            "types.generated.ts",
            "target/debug/build.rs",
            "target/debug/build.rs",
            "node_modules/pkg/index.js",
        ] {
            assert!(!SourceFileClassifier::is_source_file(path), "{path}");
        }
    }
    #[test]
    fn test_test_classifier() {
        for path in [
            "engine/rust/tests/snapshot.rs",
            "app/src/test/kotlin/inc/AppTest.kt",
            "app/src/androidTest/X.kt",
            "lib/src/jvmTest/kotlin/A.kt",
            "lib/src/commonTest/A.kt",
            "lib/src/androidUnitTest/A.kt",
            "lib/src/testFixtures/kotlin/F.kt",
            "web/__tests__/a.ts",
            "web/__mocks__/fs.ts",
            "web/__fixtures__/x.json",
            "pkg/testdata/golden.txt",
            "tools/tk-apps/test-support/developer.ts",
            "Tests/RadiologyTests/A.swift",
            "scripts/home-design-inspect.test.ts",
            "ui/View.spec.tsx",
            "scripts/build-tree-state.test.py",
            "types/api.test-d.ts",
            "server/handler_test.go",
            "src/catalog/tests.rs",
            "src/app/bundle_tests.rs",
            "src/snapshot_profiles_test.rs",
            "scripts/test_pr_local.py",
            "pkg/util_test.py",
            "app/tests.py",
            "conftest.py",
            "native/codec_unittest.cc",
            "native/codec_test.cpp",
            "tests/integration.rs",
            "src/lib_test.go",
            "components/Button.test.tsx",
            "components/Button.spec.ts",
            "test/api/conftest.py",
            "app/tests.rs",
            "tests/common.rs",
            "src/app.test.ts",
            "src/conftest.py",
        ] {
            assert!(TestPathClassifier::is_test_path(path), "{path}");
        }
        for path in [
            "src/contest.rs",
            "src/latest.ts",
            "src/attestation.py",
            "pkg/testing/helpers.go",
            "spec/openapi.yaml",
            "e2e/run.ts",
            "fixtures/seed.json",
            "src/main/kotlin/inc/SpeedTest.kt",
            "src/test.rs",
            "src/testable.ts",
            "scripts/pr-local.py",
            "scripts/home-design-inspect.ts",
            "src/main/kotlin/tested/Thing.kt",
            "docs/testing.md",
            "src/protest.go",
            "README.test.md",
            "lib/src/Test/A.kt",
            "lib/src/main/kotlin/jvmTest/A.kt",
            "src/main.rs",
        ] {
            assert!(!TestPathClassifier::is_test_path(path), "{path}");
        }
    }
}
