use super::*;
use crate::source::ReviewRevision;
#[test]
fn native_highlight_queries_compile() {
    for grammar in [SupportedGrammar::Swift, SupportedGrammar::Kotlin] {
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

#[test]
fn swift_and_kotlin_captures_color_roles_and_preserve_unicode_crlf_source() {
    for (path, text, function, keyword, string) in [
        ("a.swift", "// café 😀\r\nfunc run(value: Int) {\r\n let text = \"hi\\n\"\r\n print(text)\r\n}\r\n", "run", "func", "hi"),
        ("a.kt", "// café 😀\r\nfun run(value: Int) {\r\n val text = \"hi\\n\"\r\n println(text)\r\n}\r\n", "run", "fun", "hi"),
    ] {
        let buffer = SourceBuffer::from_str(path, text);
        let facts = super::super::DocumentSyntax::analyze(&buffer).native_highlights;
        assert!(facts.error.is_none(), "{path}: {:?}", facts.error);
        assert!(facts.matches(&buffer));
        for dark in [false, true] {
            let theme = if dark { AppTheme::dark() } else { AppTheme::light() };
            let lines = facts.recolor(&buffer, dark, || false).unwrap();
            assert_eq!(lines.iter().flatten().map(|span| span.text.as_str()).collect::<String>(), text);
            for (line, spans) in text.split_inclusive('\n').zip(lines.iter()) {
                let mut offset = 0;
                for span in spans {
                    assert_eq!(span.bytes.start, offset);
                    assert_eq!(&line[span.bytes.clone()], span.text);
                    assert!(!span.is_bold && !span.is_italic);
                    offset = span.bytes.end;
                }
                assert_eq!(offset, line.len());
            }
            assert_eq!(color_at(&lines, &buffer, text.find(function).unwrap()), role_color("function", &theme));
            assert_eq!(color_at(&lines, &buffer, text.find(keyword).unwrap()), role_color("keyword", &theme));
            assert_eq!(color_at(&lines, &buffer, text.find(string).unwrap()), role_color("string", &theme));
            assert_eq!(color_at(&lines, &buffer, text.find("\\n\"").unwrap()), if path.ends_with(".kt") { role_color("string.escape", &theme) } else { theme.text_primary });
            assert_eq!(color_at(&lines, &buffer, text.find("café").unwrap()), theme.text_secondary);
        }
        assert!(facts.recolor(&buffer, true, || true).is_none());
        assert!(!facts.matches(&SourceBuffer::from_str("other.kt", text)));
        assert!(!facts.matches(&SourceBuffer::from_str(path, &(text.to_owned()+" "))));
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
    let text = "@Fancy\nclass Sample {\n val count: Int = 42\n fun run() { val yes = true; println(count) }\n}\n";
    let buffer = SourceBuffer::from_str("a.kt", text);
    let facts = super::super::DocumentSyntax::analyze(&buffer).native_highlights;
    assert!(facts.error.is_none(), "{:?}", facts.error);
    let lines = facts.recolor(&buffer, true, || false).unwrap();
    let theme = AppTheme::dark();
    for (word, role) in [
        ("Fancy", "attribute"),
        ("Sample", "type"),
        ("count", "property"),
        ("42", "number"),
        ("run", "function"),
        ("true", "constant"),
    ] {
        assert_eq!(
            color_at(&lines, &buffer, text.find(word).unwrap()),
            role_color(role, &theme),
            "{word}"
        );
    }
}
