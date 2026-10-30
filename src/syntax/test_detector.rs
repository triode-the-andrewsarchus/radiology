//! Conservative inline test detection from native grammar nodes.
//!
//! Rules mirror the Swift reader. Comments, strings, macro token trees and
//! production-reachable Rust cfg predicates never become tests by text scanning.

mod kotlin;
mod rust;
mod script;

use super::language::SupportedGrammar;
use crate::source::SourceBuffer;
use std::ops::Range;
use tree_sitter::Node;

/// One disjoint outer test region in canonical source coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestRegion {
    /// Exact source bytes, expanded only where the declaration owns its line.
    pub byte_range: Range<usize>,
    /// One-based canonical lines, with an exclusive upper bound.
    pub line_range: Range<usize>,
    /// First visible line retained when test bodies are hidden.
    pub header_line: usize,
}

/// Test regions and document-level compile gating for an exact source snapshot.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestDetection {
    /// Normalized outer regions, merging whitespace-adjacent test declarations.
    pub regions: Vec<TestRegion>,
    /// Rust's leading inner cfg excludes the entire file from production.
    pub is_test_only_document: bool,
}

/// Detects inline tests using native Rust, Kotlin, Python and ECMAScript trees.
pub struct TestRegionDetector;

impl TestRegionDetector {
    /// Detects inline regions and whole-file test-only gating.
    #[must_use]
    pub fn detect(buffer: &SourceBuffer) -> TestDetection {
        let control = super::AnalysisControl::new(&|| false);
        let Some((grammar, tree)) = control.parse(buffer).expect("uncancelled test detection")
        else {
            return TestDetection::default();
        };
        Self::from_tree(buffer, grammar, tree.root_node(), &control)
    }

    pub(super) fn from_tree(
        buffer: &SourceBuffer,
        grammar: SupportedGrammar,
        root: Node<'_>,
        control: &super::AnalysisControl<'_>,
    ) -> TestDetection {
        let detector = Detector { buffer, control };
        let mut result = TestDetection::default();
        match grammar {
            SupportedGrammar::Rust => {
                result.is_test_only_document = detector.rust_body_test_only(root);
                if !result.is_test_only_document {
                    detector.rust_regions(root, &mut result.regions);
                }
            }
            SupportedGrammar::Python => detector.python_regions(root, &mut result.regions),
            SupportedGrammar::Kotlin => detector.kotlin_regions(root, &mut result.regions),
            SupportedGrammar::JavaScript | SupportedGrammar::TypeScript | SupportedGrammar::Tsx => {
                detector.script_regions(root, &mut result.regions)
            }
            _ => {}
        }
        result.regions = detector.normalize(result.regions);
        result
    }

    /// Returns inline regions; use [`Self::detect`] for document-level cfg gating.
    #[must_use]
    pub fn detect_test_regions(buffer: &SourceBuffer) -> Vec<TestRegion> {
        Self::detect(buffer).regions
    }
}

struct Detector<'a> {
    buffer: &'a SourceBuffer,
    control: &'a super::AnalysisControl<'a>,
}

fn children(node: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}

/// One iterative traversal policy shared by attribute-based detectors.
fn walk_regions<'tree>(
    root: Node<'tree>,
    control: &super::AnalysisControl<'_>,
    mut visit: impl FnMut(&[Node<'tree>], usize) -> bool,
) {
    let mut pending = vec![root];
    while let Some(parent) = pending.pop() {
        if control.is_cancelled() {
            break;
        }
        let siblings = children(parent);
        for (index, node) in siblings.iter().copied().enumerate() {
            if control.is_cancelled() {
                break;
            }
            if is_comment(node)
                || matches!(
                    node.kind(),
                    "token_tree"
                        | "string_literal"
                        | "raw_string_literal"
                        | "multiline_string_literal"
                )
            {
                continue;
            }
            if visit(&siblings, index) {
                pending.push(node);
            }
        }
    }
}

impl Detector<'_> {
    fn text(&self, node: Node<'_>) -> &str {
        &self.buffer.text[node.byte_range()]
    }

    fn region(&self, siblings: &[Node<'_>], mut first: usize, last: usize) -> TestRegion {
        while first > 0
            && self.doc_comment(siblings[first - 1])
            && self
                .buffer
                .line_at_byte(siblings[first - 1].end_byte().saturating_sub(1))
                + 1
                >= self.buffer.line_at_byte(siblings[first].start_byte())
        {
            first -= 1;
        }
        let mut start = siblings[first].start_byte();
        let mut end = siblings[last].end_byte();
        if let Some(comment) = siblings.get(last + 1).filter(|node| is_comment(**node)) {
            if self.buffer.line_at_byte(comment.start_byte())
                == self.buffer.line_at_byte(end.saturating_sub(1))
            {
                end = comment.end_byte();
            }
        }
        while end > start && matches!(self.buffer.data[end - 1], b'\r' | b'\n') {
            end -= 1;
        }
        let line = self.buffer.line_byte_range(self.buffer.line_at_byte(start));
        if self.buffer.data[line.start..start]
            .iter()
            .all(|byte| matches!(byte, b' ' | b'\t'))
        {
            start = line.start;
        }
        let line = self
            .buffer
            .line_byte_range(self.buffer.line_at_byte(end.saturating_sub(1)));
        if end <= line.end
            && self.buffer.data[end..line.end]
                .iter()
                .all(|byte| matches!(byte, b' ' | b'\t'))
        {
            end = line.end;
        }
        self.range(start..end)
    }

    fn range(&self, bytes: Range<usize>) -> TestRegion {
        let header_line = self.buffer.line_at_byte(bytes.start);
        let last_line = self.buffer.line_at_byte(bytes.end.saturating_sub(1));
        TestRegion {
            byte_range: bytes,
            line_range: header_line..last_line + 1,
            header_line,
        }
    }

    fn doc_comment(&self, node: Node<'_>) -> bool {
        let text = self.text(node);
        is_comment(node)
            && ((text.starts_with("///") && !text.starts_with("////"))
                || (text.starts_with("/**") && !text.starts_with("/**/")))
    }

    fn normalize(&self, mut regions: Vec<TestRegion>) -> Vec<TestRegion> {
        regions.retain(|region| !region.byte_range.is_empty());
        regions.sort_by_key(|region| {
            (
                region.byte_range.start,
                std::cmp::Reverse(region.byte_range.end),
            )
        });
        let mut output: Vec<TestRegion> = Vec::new();
        for region in regions {
            if self.control.is_cancelled() {
                break;
            }
            if let Some(previous) = output.last_mut() {
                let adjacent = previous.byte_range.end >= region.byte_range.start
                    || self.buffer.data[previous.byte_range.end..region.byte_range.start]
                        .iter()
                        .all(u8::is_ascii_whitespace);
                if adjacent {
                    *previous = self.range(
                        previous.byte_range.start
                            ..previous.byte_range.end.max(region.byte_range.end),
                    );
                    continue;
                }
            }
            output.push(region);
        }
        output
    }
}

fn is_comment(node: Node<'_>) -> bool {
    node.kind() == "comment" || node.kind().ends_with("_comment")
}

#[cfg(test)]
mod tests {
    use super::super::LanguageRegistry;
    use super::*;

    fn detected(path: &str, source: &str) -> TestDetection {
        let buffer = SourceBuffer::from_str(path, source);
        let grammar = LanguageRegistry::for_path(path).grammar.unwrap();
        let tree = grammar.parse(&buffer.data).unwrap();
        assert!(
            !tree.root_node().has_error(),
            "{}",
            tree.root_node().to_sexp()
        );
        TestRegionDetector::detect(&buffer)
    }

    #[test]
    fn rust_cfg_requires_test_only_reachability_and_ignores_strings_and_macros() {
        for predicate in [
            "test",
            "all(test, feature = \"optional\")",
            "not(not(test))",
            "doctest",
        ] {
            let text = format!("#[cfg({predicate})]\nmod checks {{ const S: &str = \"}}\"; }}\nfn production() {{}}\n");
            let result = detected("lib.rs", &text);
            assert_eq!(result.regions.len(), 1, "{predicate}");
            let region = &result.regions[0];
            assert!(text[region.byte_range.clone()].contains("const S"));
            assert!(!text[region.byte_range.clone()].contains("production"));
        }
        for predicate in [
            "any(test, feature = \"optional\")",
            "not(test)",
            "all(test, not(test))",
            "any()",
            "feature = \"test\"",
        ] {
            let text = format!("#[cfg({predicate})]\nfn production() {{}}\n");
            assert!(detected("lib.rs", &text).regions.is_empty(), "{predicate}");
        }
        assert!(detected("lib.rs", "const S: &str = \"#[cfg(test)] mod fake {}\";\nmacro_rules! fake { () => { #[test] fn absent() {} }; }\n").regions.is_empty());
        let file = detected(
            "lib.rs",
            "// leading comment\n#![cfg(test)]\nfn only_tests() {}\n",
        );
        assert!(file.is_test_only_document);
        assert!(file.regions.is_empty());
    }

    #[test]
    fn rust_attributes_doc_comments_and_crlf_ranges_preserve_neighbor_code() {
        let text = "fn production() {}\r\n/// retained documentation\r\n#[tokio::test]\r\nasync fn check() { let text = \"}\"; } // test comment\r\nfn next() {}\r\n";
        let result = detected("lib.rs", text);
        assert_eq!(result.regions.len(), 1);
        let region = &result.regions[0];
        assert_eq!(region.header_line, 2);
        assert!(text[region.byte_range.clone()].starts_with("///"));
        assert!(text[region.byte_range.clone()].ends_with("// test comment"));
        assert!(!text[region.byte_range.clone()].contains("fn next"));
    }

    #[test]
    fn python_requires_framework_evidence_and_only_module_definitions_count() {
        assert!(detected(
            "normal.py",
            "def test_connection():\n    return True\nclass TestConnection:\n    pass\n"
        )
        .regions
        .is_empty());
        let pytest = detected("mixed.py", "import pytest\n\n@pytest.fixture\ndef connection():\n    return 1\n\ndef production():\n    def test_nested():\n        pass\n    return 1\n\ndef test_read():\n    assert True\n");
        assert_eq!(pytest.regions.len(), 2);
        assert_eq!(detected("mixed.py", "import unittest\nclass Checks(unittest.TestCase):\n    def test_read(self):\n        pass\n").regions.len(), 1);
    }

    #[test]
    fn ecmascript_checks_callbacks_bindings_chains_and_vitest_guard() {
        assert!(detected(
            "normal.ts",
            "function test(name: string, callback: () => void) {}\ntest('production', () => {});\n"
        )
        .regions
        .is_empty());
        assert!(detected("normal.js", "const test = factory();\ntest('production', () => {});\nconst text = \"describe('fake', () => {})\";\n").regions.is_empty());
        for path in ["mixed.js", "mixed.ts", "mixed.tsx"] {
            let source = "import { test, describe } from 'vitest';\ndescribe.each([1])('group', () => { test('case', () => {}); });\nconst production = 1;\nif (import.meta.vitest) { const value = ')'; }\n";
            assert_eq!(detected(path, source).regions.len(), 2);
        }
        assert!(detected(
            "normal.js",
            "test('no callback');\nif (import.meta.vitest) {} else { production(); }\n"
        )
        .regions
        .is_empty());
    }

    #[test]
    fn kotlin_annotations_own_whole_classes_and_real_function_bodies() {
        let text = "class Production {\n fun read() = \"@Test fake()\"\n}\nclass Checks {\n @org.junit.Test\n fun verifies() { println(\"}\") }\n}\nfun unrelated() = 1\n";
        let result = detected("Mixed.kt", text);
        assert_eq!(result.regions.len(), 1);
        assert!(text[result.regions[0].byte_range.clone()].starts_with("class Checks"));
        assert!(!text[result.regions[0].byte_range.clone()].contains("unrelated"));
        assert_eq!(
            detected(
                "Spec.kt",
                "class Reading : StringSpec({ \"reads\" { println(1) } })\n"
            )
            .regions
            .len(),
            1
        );
    }
}
