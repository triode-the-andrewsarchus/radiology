//! Headless smoke verification of the syntax resources used by the reader.

use radiology::{
    source::SourceBuffer,
    syntax::{DocumentSyntax, LanguageRegistry, SyntaxHighlighter},
};
use std::collections::HashSet;

const NATIVE: &[(&str, &str, bool)] = &[
    ("probe.rs", "fn probe(value: i32) -> i32 {\n target(value)\n}\n#[test]\nfn test_probe() { probe(1); }\n", false),
    ("probe.py", "def probe(value):\n result = target(value)\n return result\n", false),
    ("probe.go", "package main\nfunc probe(value int) int {\n return target(value)\n}\n", false),
    ("probe.js", "function probe(value) {\n return target(value);\n}\n", true),
    ("probe.ts", "export function probe(value: number): number {\n return target(value);\n}\n", true),
    ("probe.tsx", "export function probe(value: number) {\n return <section title={target(value)}>hello</section>;\n}\n", true),
    ("probe.swift", "func probe(_ value: Int) -> Int {\n return target(value)\n}\n", true),
    ("probe.kt", "fun probe(value: Int): Int {\n return target(value)\n}\n", true),
    ("probe.c", "int target(int);\nint probe(int value) {\n return target(value);\n}\n", false),
];

pub(super) fn verify_resources() -> bool {
    let highlighter = SyntaxHighlighter::new();
    let mut ok = true;
    println!("Checking native parsers and reader queries...");
    for &(path, text, native_colors) in NATIVE {
        match verify_native(path, text, native_colors, &highlighter) {
            Ok(()) => println!("ok   {path}: parse, outline, folds, calls, occurrences, colors"),
            Err(error) => {
                println!("FAIL {path}: {error}");
                ok = false;
            }
        }
    }
    println!("Checking syntax color fallbacks (no native grammar claim)...");
    for (path, text) in [
        ("probe.cpp", "class Probe { public: int value = 1; };\n"),
        ("probe.html", "<h1 class=\"probe\">hello</h1>\n"),
        ("probe.json", "{\"probe\": true}\n"),
        ("probe.md", "# Probe\nSome **text**.\n"),
        ("probe.yaml", "probe: true\n"),
        ("probe.toml", "[probe]\nvalue = 1\n"),
    ] {
        let buffer = SourceBuffer::from_str(path, text);
        match verify_colors(&buffer, None, &highlighter) {
            Ok(()) => println!("ok   {path}: fallback colors"),
            Err(error) => {
                println!("FAIL {path}: {error}");
                ok = false;
            }
        }
    }
    println!(
        "Syntax resource smoke checks {}.",
        if ok { "passed" } else { "failed" }
    );
    ok
}

fn verify_native(
    path: &str,
    text: &str,
    native_colors: bool,
    highlighter: &SyntaxHighlighter,
) -> Result<(), String> {
    if LanguageRegistry::for_path(path).grammar.is_none() {
        return Err("native grammar is not registered".into());
    }
    let buffer = SourceBuffer::from_str(path, text);
    let facts = DocumentSyntax::analyze(&buffer);
    let checks = [
        ("parse", !facts.calls.parse_errors),
        (
            "outline",
            facts.symbols.iter().any(|symbol| symbol.name == "probe"),
        ),
        ("folds", facts.folds_supported && !facts.folds.is_empty()),
        (
            "calls",
            facts.calls.supported
                && facts.calls.sites.iter().any(|site| {
                    site.name.as_ref().and_then(|range| text.get(range.clone())) == Some("target")
                }),
        ),
        (
            "occurrences",
            facts.occurrences.error.is_none()
                && text
                    .find("value")
                    .and_then(|byte| facts.occurrences.selection(&buffer, byte))
                    .is_some(),
        ),
        (
            "native colors",
            !native_colors || facts.native_highlights.matches(&buffer),
        ),
        (
            "test detection",
            path != "probe.rs" || !facts.tests.regions.is_empty(),
        ),
    ];
    let failed: Vec<_> = checks
        .into_iter()
        .filter_map(|(name, passed)| (!passed).then_some(name))
        .collect();
    if !failed.is_empty() {
        return Err(format!(
            "missing/invalid {} (folds: {:?}; calls: {:?}; occurrences: {:?}; highlights: {:?})",
            failed.join(", "),
            facts.fold_error,
            facts.calls.error,
            facts.occurrences.error,
            facts.native_highlights.error
        ));
    }
    verify_colors(
        &buffer,
        native_colors.then_some(facts.native_highlights.as_ref()),
        highlighter,
    )
}

fn verify_colors(
    buffer: &SourceBuffer,
    native: Option<&radiology::syntax::native_highlights::NativeHighlights>,
    highlighter: &SyntaxHighlighter,
) -> Result<(), String> {
    for dark in [false, true] {
        let lines = if let Some(native) = native {
            native
                .recolor(buffer, dark, || false)
                .ok_or("native recoloring failed")?
        } else {
            let syntax = highlighter
                .find_syntax_for_path(&buffer.path)
                .ok_or("syntax fallback is missing")?;
            highlighter.highlight_document(Some(syntax), &buffer.text, dark)
        };
        let text: String = lines
            .iter()
            .flatten()
            .map(|span| span.text.as_str())
            .collect();
        let colors: HashSet<_> = lines
            .iter()
            .flatten()
            .map(|span| span.color.to_array())
            .collect();
        if text != buffer.text || colors.len() < 2 {
            return Err(format!(
                "{} colors lost source bytes or produced no syntax roles",
                if dark { "dark" } else { "light" }
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_native_resources_and_fallbacks_pass_the_real_reader_paths() {
        assert!(verify_resources());
    }

    #[test]
    fn a_fallback_cannot_hide_missing_native_structure() {
        let highlighter = SyntaxHighlighter::new();
        let error =
            verify_native("probe.tsx", "const value = 1;\n", true, &highlighter).unwrap_err();
        assert!(error.contains("outline"), "{error}");
        assert!(error.contains("calls"), "{error}");
        assert!(verify_native("unknown.txt", "probe(value)", false, &highlighter).is_err());
    }
}
