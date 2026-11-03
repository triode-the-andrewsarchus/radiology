use super::*;
use crate::source::ReviewRevision;
#[test]
fn native_highlight_queries_compile() {
    for grammar in [
        SupportedGrammar::Rust,
        SupportedGrammar::Go,
        SupportedGrammar::Python,
        SupportedGrammar::Swift,
        SupportedGrammar::Kotlin,
        SupportedGrammar::JavaScript,
        SupportedGrammar::TypeScript,
        SupportedGrammar::Tsx,
        SupportedGrammar::C,
        SupportedGrammar::Cpp,
    ] {
        assert!(
            super::super::queries::query(grammar, super::super::queries::QueryRole::Highlights)
                .is_ok(),
            "{grammar:?}: {:?}",
            super::super::queries::query(grammar, super::super::queries::QueryRole::Highlights)
                .err()
        );
    }
}

fn color_at(lines: &[Vec<HighlightedSpan>], buffer: &SourceBuffer, byte: usize) -> Color32 {
    let line = buffer.line_at_byte(byte);
    let local = byte - buffer.byte_offset(line, 0);
    lines[line - 1]
        .iter()
        .find(|span| span.bytes.contains(&local))
        .unwrap()
        .color
}

fn assert_original_bytes(lines: &[Vec<HighlightedSpan>], buffer: &SourceBuffer) {
    assert_eq!(
        lines
            .iter()
            .flatten()
            .map(|span| span.text.as_str())
            .collect::<String>(),
        buffer.text
    );
    assert_eq!(lines.len(), buffer.text.split_inclusive('\n').count());
    for (line, spans) in buffer.text.split_inclusive('\n').zip(lines) {
        let mut offset = 0;
        for span in spans {
            assert_eq!(span.bytes.start, offset);
            assert_eq!(&line[span.bytes.clone()], span.text);
            assert!(!span.is_bold && !span.is_italic);
            offset = span.bytes.end;
        }
        assert_eq!(offset, line.len());
    }
}

fn recolored_source(
    facts: &NativeHighlights,
    buffer: &SourceBuffer,
    dark: bool,
) -> (Vec<Vec<HighlightedSpan>>, AppTheme) {
    assert!(facts.matches(buffer));
    assert!(facts.error.is_none(), "{:?}", facts.error);
    let lines = facts.recolor(buffer, dark, || false).unwrap();
    assert_original_bytes(&lines, buffer);
    let theme = if dark {
        AppTheme::dark()
    } else {
        AppTheme::light()
    };
    (lines, theme)
}

#[test]
fn native_dialects_color_roles_and_preserve_unicode_crlf_source() {
    let typed = "// café 😀\r\ninterface User { readonly name: string; }\r\nexport function greet(user: User): string {\r\n  return user.name;\r\n}\r\n";
    let jsx = "// café 😀\r\nexport function View() {\r\n  return <section title=\"hi\">Hello</section>;\r\n}\r\n";
    let javascript =
        "// café 😀\r\nexport function greet(value) {\r\n  return value + \"hi\";\r\n}\r\n";
    for (path, text, expected) in [
        ("a.rs", "// café 😀\r\n#[derive(Debug)]\r\nstruct Record<'borrow> { field: &'borrow str }\r\nfn run<'borrow>(value: &'borrow str) -> Record<'borrow> {\r\n let text = \"hi\\n\"; let raw = r#\"raw\"#; let count = 42; let yes = true;\r\n println!(\"{}\", text); Record { field: value }\r\n}\r\n", vec![("derive", "attribute"), ("struct", "keyword"), ("Record", "type"), ("borrow", "label"), ("field:", "property"), ("str }", "type"), ("run", "function"), ("hi", "string"), ("\\n\"", "string.escape"), ("raw\"#", "string"), ("42", "number"), ("true", "constant"), ("println", "function")]),
        ("a.go", "// café 😀\r\npackage reader\r\ntype Record struct { Field string }\r\nfunc run(value string) string {\r\n text := \"hi\\n\"; count := 42; _ = count; println(text)\r\n return value\r\n}\r\n", vec![("package", "keyword"), ("reader", "namespace"), ("Record", "type"), ("Field", "property"), ("run", "function"), ("string", "type"), ("println", "function.builtin"), ("hi", "string"), ("\\n\"", "string.escape"), ("42", "number")]),
        ("a.py", "# café 😀\r\n@decorator\r\nclass Record:\r\n def run(self, value: str):\r\n  text = \"hi\\n\"; COUNT = 42\r\n  print(self.field)\r\n  return value\r\n", vec![("decorator", "attribute"), ("class", "keyword"), ("Record", "type"), ("run", "function"), ("self", "variable.builtin"), ("str", "type"), ("print", "function.builtin"), ("field", "property"), ("COUNT", "constant"), ("42", "number"), ("hi", "string"), ("\\n\"", "string.escape")]),
        ("a.c", "// café 😀\r\nint run(int value) { return value; }\r\n", vec![("run", "function"), ("int", "type.builtin"), ("return", "keyword.return")]),
        ("a.cpp", "// café 😀\r\ntemplate <typename T> T run(T value) { return value; }\r\n", vec![("template", "keyword"), ("run", "function"), ("return", "keyword.return")]),
        ("a.swift", "// café 😀\r\nfunc run(value: Int) {\r\n let text = \"hi\\n\"\r\n print(text)\r\n}\r\n", vec![("run", "function"), ("func", "keyword"), ("hi", "string"), ("\\n\"", "primary")]),
        ("a.kt", "// café 😀\r\nfun run(value: Int) {\r\n val text = \"hi\\n\"\r\n println(text)\r\n}\r\n", vec![("run", "function"), ("fun", "keyword"), ("hi", "string"), ("\\n\"", "string.escape")]),
        (
            "a.ts",
            typed,
            vec![
                ("interface", "keyword"),
                ("User", "type"),
                ("readonly", "keyword"),
                ("string", "type"),
                ("greet", "function"),
                ("name:", "property"),
            ],
        ),
        (
            "a.mts",
            typed,
            vec![("interface", "keyword"), ("User", "type")],
        ),
        (
            "a.cts",
            typed,
            vec![("readonly", "keyword"), ("string", "type")],
        ),
        (
            "a.tsx",
            jsx,
            vec![("section", "tag"), ("title", "attribute"), ("hi", "string")],
        ),
        (
            "a.jsx",
            jsx,
            vec![("section", "tag"), ("title", "attribute")],
        ),
        (
            "a.js",
            javascript,
            vec![
                ("export", "keyword"),
                ("greet", "function"),
                ("hi", "string"),
            ],
        ),
    ] {
        let buffer = SourceBuffer::from_str(path, text);
        let grammar = super::super::LanguageRegistry::for_path(path)
            .grammar
            .unwrap();
        assert!(
            !grammar.parse(&buffer.data).unwrap().root_node().has_error(),
            "{path}"
        );
        let facts = super::super::DocumentSyntax::analyze(&buffer).native_highlights;
        assert!(facts.error.is_none(), "{path}: {:?}", facts.error);
        assert!(facts.matches(&buffer), "{path} must use native captures");
        for dark in [false, true] {
            let (lines, theme) = recolored_source(&facts, &buffer, dark);
            for (token, role) in &expected {
                assert_eq!(
                    color_at(&lines, &buffer, text.find(token).unwrap()),
                    role_color(role, &theme),
                    "{path}: {token}"
                );
            }
            assert_eq!(
                color_at(&lines, &buffer, text.find("café").unwrap()),
                theme.text_secondary
            );
        }
        assert!(facts.recolor(&buffer, true, || true).is_none());
        assert!(!facts.matches(&SourceBuffer::from_str(path, &(text.to_owned() + " "))));
        assert!(!facts.matches(&SourceBuffer::from_str("other.file", text)));
        let before = SourceBuffer::from_bytes(path, buffer.data.clone(), Some(ReviewRevision::Before)).unwrap();
        assert!(!facts.matches(&before));
    }
}

#[test]
fn later_and_nested_captures_win_without_leaking_a_role_into_following_text() {
    let text = "func run() { value.after; }\n";
    let buffer = SourceBuffer::from_str("a.swift", text);
    let mut facts = super::super::DocumentSyntax::analyze(&buffer)
        .native_highlights
        .as_ref()
        .clone();
    let start = text.find("value").unwrap();
    facts.captures = vec![
        Capture {
            bytes: start..start + 11,
            name: "type".into(),
            pattern: 0,
        },
        Capture {
            bytes: start..start + 5,
            name: "string".into(),
            pattern: 1,
        },
        Capture {
            bytes: start..start + 5,
            name: "function".into(),
            pattern: 2,
        },
    ];
    let lines = facts.recolor(&buffer, true, || false).unwrap();
    let theme = AppTheme::dark();
    assert_eq!(color_at(&lines, &buffer, start), theme.accent);
    assert_eq!(
        color_at(&lines, &buffer, start + 6),
        role_color("type", &theme)
    );
    assert_eq!(color_at(&lines, &buffer, start + 11), theme.text_primary);
}

#[test]
fn native_capture_limits_return_no_partial_color_set() {
    let buffer = SourceBuffer::from_str("a.kt", "fun run() { val x = 12; println(x) }\n");
    let grammar = SupportedGrammar::Kotlin;
    let tree = grammar.parse(&buffer.data).unwrap();
    assert!(captures(
        &buffer,
        grammar,
        tree.root_node(),
        1,
        &crate::syntax::AnalysisControl::new(&|| false)
    )
    .unwrap_err()
    .contains("fact limit"));
    let syntax = super::super::DocumentSyntax::analyze(&buffer);
    assert!(syntax.folds_supported && syntax.calls.supported && syntax.occurrences.error.is_none());
}

#[test]
fn kotlin_native_nodes_supply_type_property_constant_and_annotation_roles() {
    let source = "// café 漢字 😀\n@file:JvmName(\"Scans\")\n@file:Marker\n@Fancy\nclass Sample {\n val count: Int = 42\n fun run(value: Any, items: List<Int>) {\n  val yes = true; println(count)\n  val absent = 2 !in items\n  val other = value !is String\n  val cast = value as? String\n  val literal = \"!is !in\"\n }\n}\n";
    for crlf in [false, true] {
        let text = if crlf {
            source.replace('\n', "\r\n")
        } else {
            source.to_owned()
        };
        let buffer = SourceBuffer::from_str("a.kt", &text);
        let tree = SupportedGrammar::Kotlin.parse(&buffer.data).unwrap();
        for operator in ["!is", "!in", "as?"] {
            let start = text.find(operator).unwrap();
            let range = start..start + operator.len();
            let node = tree
                .root_node()
                .descendant_for_byte_range(range.start, range.end)
                .unwrap();
            assert_eq!(
                node.kind(),
                operator,
                "raw token must own every operator byte"
            );
            assert_eq!(node.byte_range(), range);
        }
        let syntax = super::super::DocumentSyntax::analyze(&buffer);
        assert!(!syntax.calls.parse_errors && syntax.parse_error.is_none());
        let facts = syntax.native_highlights;
        assert!(facts.error.is_none(), "{:?}", facts.error);
        for dark in [false, true] {
            let (lines, theme) = recolored_source(&facts, &buffer, dark);
            for (word, role) in [
                ("JvmName", "attribute"),
                ("Marker", "attribute"),
                ("Fancy", "attribute"),
                ("Sample", "type"),
                ("count", "property"),
                ("42", "number"),
                ("run", "function"),
                ("true", "constant"),
                ("!in", "keyword"),
                ("!is", "keyword"),
                ("as?", "keyword"),
                ("\"!is !in\"", "string"),
            ] {
                let start = text.find(word).unwrap();
                for byte in start..start + word.len() {
                    assert_eq!(
                        color_at(&lines, &buffer, byte),
                        role_color(role, &theme),
                        "{word}: byte={byte}, CRLF={crlf}, dark={dark}"
                    );
                }
            }
        }
    }
}

#[test]
fn rust_native_long_comment_preserves_full_unicode_crlf_and_recolors_without_a_parse() {
    let text = format!(
        "// {} LONG_END\r\nfn tail() {{}}\r\n",
        "abcdé🦀漢字 ".repeat(370_000)
    );
    let buffer = SourceBuffer::from_str("long.rs", &text);
    assert!(buffer.byte_len() > 6_000_000);
    let facts = super::super::DocumentSyntax::analyze(&buffer).native_highlights;
    assert!(
        facts.matches(&buffer),
        "Rust must retain exact native captures"
    );
    assert!(facts.error.is_none(), "{:?}", facts.error);
    for dark in [false, true] {
        let (lines, theme) = recolored_source(&facts, &buffer, dark);
        for token in ["é", "🦀", "LONG_END"] {
            assert_eq!(
                color_at(&lines, &buffer, text.find(token).unwrap()),
                theme.text_secondary
            );
        }
        assert_eq!(
            color_at(&lines, &buffer, text.find("tail").unwrap()),
            theme.accent
        );
    }
    assert!(facts.recolor(&buffer, true, || true).is_none());
}

#[test]
fn swift_corpus_fixtures_preserve_native_facts_and_colors_in_both_line_endings() {
    for crlf in [false, true] {
        let snapshots = crate::syntax::test_fixtures::SWIFT_SOURCE_FIXTURES
            .iter()
            .map(|&(path, text, _)| {
                let text = if crlf {
                    text.replace('\n', "\r\n")
                } else {
                    text.to_string()
                };
                let buffer = SourceBuffer::from_str(path, &text);
                let facts = super::super::DocumentSyntax::analyze(&buffer);
                (buffer, facts)
            })
            .collect::<Vec<_>>();
        let parse_errors = snapshots
            .iter()
            .filter(|(_, facts)| facts.calls.parse_errors)
            .map(|(buffer, _)| buffer.path.as_str())
            .collect::<Vec<_>>();
        assert!(
            parse_errors.is_empty(),
            "unexpected corpus parse errors (CRLF={crlf}): {parse_errors:?}"
        );
        for (buffer, facts) in snapshots {
            assert!(
                facts.parse_error.is_none() && facts.fold_error.is_none(),
                "{}: {:?} {:?}",
                buffer.path,
                facts.parse_error,
                facts.fold_error
            );
            assert!(
                facts.calls.supported && facts.occurrences.error.is_none(),
                "{}",
                buffer.path
            );
            assert!(
                facts.native_highlights.matches(&buffer),
                "{} must retain native colors",
                buffer.path
            );
            for row in crate::syntax::outline::outline_rows(&facts.symbols, &buffer) {
                let symbol = row.symbol;
                assert!(buffer.text.get(symbol.range.clone()).is_some());
                assert_eq!(
                    buffer.text.get(symbol.selection_range.clone()),
                    Some(symbol.name.as_str())
                );
                assert!(
                    symbol.range.start <= symbol.selection_range.start
                        && symbol.selection_range.end <= symbol.range.end
                );
            }
            for range in facts
                .folds
                .iter()
                .map(|fold| &fold.byte_range)
                .chain(facts.calls.scopes.iter().map(|scope| &scope.range))
                .chain(facts.calls.sites.iter().map(|site| &site.range))
                .chain(
                    facts
                        .calls
                        .sites
                        .iter()
                        .filter_map(|site| site.name.as_ref()),
                )
                .chain(
                    facts
                        .native_highlights
                        .captures
                        .iter()
                        .map(|capture| &capture.bytes),
                )
            {
                assert!(
                    buffer.text.get(range.clone()).is_some(),
                    "{}: {range:?}",
                    buffer.path
                );
            }
            for dark in [false, true] {
                assert_original_bytes(
                    &facts
                        .native_highlights
                        .recolor(&buffer, dark, || false)
                        .unwrap(),
                    &buffer,
                );
            }
        }
    }
}
