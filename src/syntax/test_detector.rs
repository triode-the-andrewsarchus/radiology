//! AST-aware inline test region detection for source files.
//!
//! Detects inline test modules, unit test functions, and test fixtures across Rust,
//! Python, TypeScript/JavaScript, and Kotlin, allowing them to be automatically folded
//! or receded when the "Hide Tests" filter is active.

use crate::source::buffer::SourceBuffer;
use crate::syntax::language::{LanguageRegistry, SupportedGrammar};
use std::ops::Range;

/// Represents an inline test region (e.g. `mod tests { ... }` or `#[test] fn ...`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestRegion {
    /// Absolute UTF-8 byte range of the test region in the buffer.
    pub byte_range: Range<usize>,
    /// 1-based line number range spanning the entire test region.
    pub line_range: Range<usize>,
    /// 1-based line number of the test region's header line.
    pub header_line: usize,
}

/// Detects inline test code blocks within source buffers.
pub struct TestRegionDetector;

impl TestRegionDetector {
    /// Detects all inline test regions in `buffer`.
    #[must_use]
    pub fn detect_test_regions(buffer: &SourceBuffer) -> Vec<TestRegion> {
        let lang = LanguageRegistry::for_path(&buffer.path);
        match lang.grammar {
            Some(SupportedGrammar::Rust) => Self::detect_rust_tests(buffer),
            Some(SupportedGrammar::Python) => Self::detect_python_tests(buffer),
            Some(
                SupportedGrammar::TypeScript | SupportedGrammar::Tsx | SupportedGrammar::JavaScript,
            ) => Self::detect_js_tests(buffer),
            _ => Self::detect_generic_tests(buffer),
        }
    }

    fn detect_rust_tests(buffer: &SourceBuffer) -> Vec<TestRegion> {
        let mut regions = Vec::new();
        let total_lines = buffer.line_count();
        let mut line = 1;

        while line <= total_lines {
            let text = buffer.line_text(line).trim();

            // Detect #[cfg(test)] mod tests { ... }
            if text.contains("#[cfg(test)]") || text.contains("#![cfg(test)]") {
                let start_line = line;
                let mut brace_count = 0;
                let mut found_open = false;
                let mut end_line = line;

                for cur in line..=total_lines {
                    let cur_text = buffer.line_text(cur);
                    for ch in cur_text.chars() {
                        if ch == '{' {
                            brace_count += 1;
                            found_open = true;
                        } else if ch == '}' && found_open {
                            brace_count -= 1;
                            if brace_count == 0 {
                                end_line = cur;
                                break;
                            }
                        }
                    }
                    if found_open && brace_count == 0 {
                        break;
                    }
                }

                if end_line >= start_line {
                    let start_byte = buffer.byte_offset(start_line, 0);
                    let end_byte = buffer.byte_offset(end_line + 1, 0);
                    regions.push(TestRegion {
                        byte_range: start_byte..end_byte,
                        line_range: start_line..(end_line + 1),
                        header_line: start_line,
                    });
                    line = end_line + 1;
                    continue;
                }
            }

            // Detect standalone #[test] / #[tokio::test] functions
            if text.starts_with("#[test]")
                || text.starts_with("#[tokio::test]")
                || text.starts_with("#[rstest]")
            {
                let start_line = line;
                let mut brace_count = 0;
                let mut found_open = false;
                let mut end_line = line;

                for cur in line..=total_lines {
                    let cur_text = buffer.line_text(cur);
                    for ch in cur_text.chars() {
                        if ch == '{' {
                            brace_count += 1;
                            found_open = true;
                        } else if ch == '}' && found_open {
                            brace_count -= 1;
                            if brace_count == 0 {
                                end_line = cur;
                                break;
                            }
                        }
                    }
                    if found_open && brace_count == 0 {
                        break;
                    }
                }

                if end_line >= start_line {
                    let start_byte = buffer.byte_offset(start_line, 0);
                    let end_byte = buffer.byte_offset(end_line + 1, 0);
                    regions.push(TestRegion {
                        byte_range: start_byte..end_byte,
                        line_range: start_line..(end_line + 1),
                        header_line: start_line,
                    });
                    line = end_line + 1;
                    continue;
                }
            }

            line += 1;
        }

        regions
    }

    fn detect_python_tests(buffer: &SourceBuffer) -> Vec<TestRegion> {
        let mut regions = Vec::new();
        let total_lines = buffer.line_count();
        let mut line = 1;

        while line <= total_lines {
            let raw_text = buffer.line_text(line);
            let trimmed = raw_text.trim();

            if trimmed.starts_with("def test_") || trimmed.starts_with("class Test") {
                let start_line = line;
                let base_indent = raw_text.chars().take_while(|c| c.is_whitespace()).count();
                let mut end_line = line;

                for cur in (line + 1)..=total_lines {
                    let cur_raw = buffer.line_text(cur);
                    if cur_raw.trim().is_empty() {
                        continue;
                    }
                    let cur_indent = cur_raw.chars().take_while(|c| c.is_whitespace()).count();
                    if cur_indent <= base_indent {
                        break;
                    }
                    end_line = cur;
                }

                let start_byte = buffer.byte_offset(start_line, 0);
                let end_byte = buffer.byte_offset(end_line + 1, 0);
                regions.push(TestRegion {
                    byte_range: start_byte..end_byte,
                    line_range: start_line..(end_line + 1),
                    header_line: start_line,
                });
                line = end_line + 1;
                continue;
            }

            line += 1;
        }

        regions
    }

    fn detect_js_tests(buffer: &SourceBuffer) -> Vec<TestRegion> {
        let mut regions = Vec::new();
        let total_lines = buffer.line_count();
        let mut line = 1;

        while line <= total_lines {
            let trimmed = buffer.line_text(line).trim();

            if trimmed.starts_with("describe(")
                || trimmed.starts_with("it(")
                || trimmed.starts_with("test(")
                || trimmed.starts_with("suite(")
            {
                let start_line = line;
                let mut paren_count = 0;
                let mut found_open = false;
                let mut end_line = line;

                for cur in line..=total_lines {
                    let cur_text = buffer.line_text(cur);
                    for ch in cur_text.chars() {
                        if ch == '(' {
                            paren_count += 1;
                            found_open = true;
                        } else if ch == ')' && found_open {
                            paren_count -= 1;
                            if paren_count == 0 {
                                end_line = cur;
                                break;
                            }
                        }
                    }
                    if found_open && paren_count == 0 {
                        break;
                    }
                }

                let start_byte = buffer.byte_offset(start_line, 0);
                let end_byte = buffer.byte_offset(end_line + 1, 0);
                regions.push(TestRegion {
                    byte_range: start_byte..end_byte,
                    line_range: start_line..(end_line + 1),
                    header_line: start_line,
                });
                line = end_line + 1;
                continue;
            }

            line += 1;
        }

        regions
    }

    fn detect_generic_tests(buffer: &SourceBuffer) -> Vec<TestRegion> {
        let mut regions = Vec::new();
        let total_lines = buffer.line_count();

        for line in 1..=total_lines {
            let text = buffer.line_text(line).trim();
            if text.starts_with("@Test") {
                let start_byte = buffer.byte_offset(line, 0);
                let end_byte = buffer.byte_offset(line + 1, 0);
                regions.push(TestRegion {
                    byte_range: start_byte..end_byte,
                    line_range: line..(line + 1),
                    header_line: line,
                });
            }
        }

        regions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rust_test_module() {
        let code = r#"
fn prod() -> i32 { 1 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prod() {
        assert_eq!(prod(), 1);
    }
}
"#;
        let buf = SourceBuffer::from_str("src/lib.rs", code);
        let regions = TestRegionDetector::detect_test_regions(&buf);
        assert!(!regions.is_empty());
        assert_eq!(regions[0].header_line, 4);
    }
}
