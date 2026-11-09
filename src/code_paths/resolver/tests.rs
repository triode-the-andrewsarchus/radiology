use super::*;

fn resolve(
    path: &str,
    text: &str,
    selector: &str,
) -> Result<Vec<CodePathDestination>, ResolveError> {
    let buffer = SourceBuffer::from_str(path, text);
    let syntax = DocumentSyntax::analyze(&buffer);
    destinations(selector, &buffer, &syntax, &AtomicBool::new(false))
}

#[test]
fn current_source_lines_shift_without_a_semantic_index() {
    let text = "// boot\nvoid start() {}\nvoid boot() { start(); }\n";
    let first = resolve("boot.cpp", text, "boot").unwrap();
    let shifted = resolve("boot.cpp", &format!("// café 🦀\n// added\n{text}"), "boot").unwrap();
    assert_eq!(first[0].line, 3);
    assert_eq!(shifted[0].line, 5);
    assert_eq!(
        shifted[0].location.byte_offset,
        first[0].location.byte_offset + "// café 🦀\n// added\n".len()
    );
}

#[test]
fn qualified_cpp_methods_namespaces_and_out_of_class_definitions_disambiguate() {
    let text = "struct A { void boot() {} };\nstruct B { void boot() {} };\nvoid C::boot() {}\nnamespace N { class D { void boot() {} }; }\n";
    for (selector, line) in [
        ("A::boot", 1),
        ("B.boot", 2),
        ("C::boot", 3),
        ("N::D::boot", 4),
    ] {
        let result = resolve("boot.cpp", text, selector).unwrap();
        assert_eq!(result[0].line, line, "{selector}");
        assert!(text[result[0].location.byte_offset..].starts_with("boot"));
    }
    assert!(matches!(
        resolve("boot.cpp", text, "boot"),
        Err(ResolveError::Ambiguous { .. })
    ));
}

#[test]
fn exact_headers_select_overloads_and_real_definitions_outrank_prototypes() {
    let text = "void boot(int cpu);\nvoid boot(int cpu) {}\nvoid boot(bool ready) {}\n";
    assert_eq!(
        resolve("boot.cpp", text, "void boot( int cpu )").unwrap()[0].line,
        2
    );
    assert!(matches!(
        resolve("boot.cpp", text, "boot"),
        Err(ResolveError::Ambiguous { .. })
    ));
    assert_eq!(
        resolve("boot.c", "void start();\nvoid start() {}\n", "start").unwrap()[0].line,
        2
    );
    assert!(matches!(
        resolve("boot.py", "boot = lambda: 0\nboot = 1\n", "boot"),
        Err(ResolveError::Ambiguous { .. })
    ));
}

#[test]
fn rust_swift_python_and_assembly_use_real_declaration_names() {
    for (path, text, selector) in [
        (
            "boot.rs",
            "struct Kernel;\nimpl Kernel { fn boot() {} }\n",
            "Kernel::boot",
        ),
        (
            "boot.swift",
            "struct Kernel { func boot() {} }\n",
            "Kernel.boot",
        ),
        (
            "boot.py",
            "class Kernel:\n    def boot(self):\n        pass\n",
            "Kernel.boot",
        ),
        (
            "boot.S",
            "// _start is not a label\n  _start:\n  nop\n",
            "_start",
        ),
    ] {
        let result = resolve(path, text, selector).unwrap();
        let terminal = selector
            .split("::")
            .last()
            .unwrap()
            .split('.')
            .next_back()
            .unwrap();
        assert!(
            text[result[0].location.byte_offset..].starts_with(terminal),
            "{path}"
        );
    }
}

#[test]
fn comments_strings_calls_unsupported_text_and_ordinary_duplicate_labels_fail() {
    for (path, text, selector) in [
        ("boot.cpp", "// void boot() {}\nconst char *s = \"boot\";\n", "boot"),
        ("notes.txt", "fn boot() {}\n", "boot"),
        ("boot.S", "_start:\n nop\n_start:\n nop\n", "_start"),
        ("head.S", "/* BEGIN_FUNC(_start) */\n.asciz \"BEGIN_FUNC(_start)\"\n bl _start\nEND_FUNC(_start)\n", "_start"),
    ] { assert!(resolve(path, text, selector).is_err(), "{path}: {text}"); }
    for name in [
        "BEGIN_FUNC",
        "ENTRY",
        "ENTRY_NP",
        "SYM_FUNC_START",
        "SYM_CODE_START",
    ] {
        let text = format!(
            "/*\n{name}(_start)\n*/\n// {name}(_start)\n{name}(_start)\n nop\nEND_FUNC(_start)\n"
        );
        assert_eq!(resolve("head.S", &text, "_start").unwrap()[0].line, 5);
    }
}

#[test]
fn only_opposing_shared_preprocessor_branches_are_variants() {
    for path in ["boot.c", "boot.cpp", "boot.S"] {
        let declaration = if path.ends_with('S') {
            "ENTRY(boot)"
        } else {
            "void boot() {}"
        };
        let text = format!(
            "#if(HYP)\n{declaration}\n#elif OTHER\n{declaration}\n#else\n{declaration}\n#endif\n"
        );
        assert_eq!(
            resolve(path, &text, "boot")
                .unwrap()
                .iter()
                .map(|d| d.line)
                .collect::<Vec<_>>(),
            [2, 4, 6]
        );
        let buffer = SourceBuffer::from_str(path, &text);
        assert!(
            !source_text::alternatives(&[text.len() - 1, 0], &buffer, &|| Ok::<(), ResolveError>(
                ()
            ))
            .unwrap()
        );
        let duplicate =
            format!("#if HYP\n{declaration}\n{declaration}\n#else\n{declaration}\n#endif\n");
        assert!(matches!(
            resolve(path, &duplicate, "boot"),
            Err(ResolveError::Ambiguous { .. })
        ));
        let separate = format!("#if X\n{declaration}\n#endif\n#if Y\n{declaration}\n#endif\n");
        assert!(resolve(path, &separate, "boot").is_err());
    }
    let fake = "const char *s = R\"raw(\n#if FAKE\n\"\n)raw\";\nvoid boot() {}\n/*\n#else\n*/\nvoid boot() {}\n";
    assert!(matches!(
        resolve("boot.cpp", fake, "boot"),
        Err(ResolveError::Ambiguous { .. })
    ));
}

#[test]
fn conditional_body_brace_recovery_is_navigation_only_and_never_matches_calls() {
    let text = "BOOT_CODE void create_idle_thread(void)\n{\n#ifdef SMP\n for (int i = 0; i < 4; ++i) {\n#endif\n configureIdleThread();\n#ifdef SMP\n }\n#endif\n}\n";
    let result = resolve("boot.c", text, "create_idle_thread").unwrap();
    assert_eq!(result[0].line, 1);
    assert!(resolve("boot.c", text, "configureIdleThread").is_err());
    let syntax = DocumentSyntax::analyze(&SourceBuffer::from_str("boot.c", text));
    assert!(!syntax.recovered_declarations.is_empty());
    for recovered in &syntax.recovered_declarations {
        assert!(!syntax
            .calls
            .scopes
            .iter()
            .any(|scope| scope.range == recovered.range));
        assert!(!syntax
            .folds
            .iter()
            .any(|fold| fold.byte_range == recovered.range));
    }
}

#[test]
fn cpp_structure_captures_name_tokens_and_does_not_call_function_pointer_variables() {
    let text =
        "namespace N { class A { void boot(); int (*callback)(int); }; }\nvoid N::A::boot() {}\n";
    let buffer = SourceBuffer::from_str("boot.cpp", text);
    let syntax = DocumentSyntax::analyze(&buffer);
    assert!(
        syntax.parse_error.is_none() && syntax.fold_error.is_none() && syntax.calls.supported,
        "{:?} {:?} {:?}",
        syntax.parse_error,
        syntax.fold_error,
        syntax.calls.error
    );
    let mut symbols = Vec::new();
    flatten(&syntax.symbols, &mut symbols, &AtomicBool::new(false)).unwrap();
    assert!(symbols
        .iter()
        .any(|s| s.name == "A" && s.kind == crate::syntax::SymbolKind::Class));
    assert!(symbols
        .iter()
        .filter(|s| s.name == "boot")
        .all(|s| &buffer.text[s.selection_range.clone()] == "boot"));
    assert!(!symbols.iter().any(|s| s.name == "callback"));
}

#[test]
fn matching_displayed_bytes_are_authoritative_and_dot_paths_are_normalized() {
    let owner = tempfile::tempdir().unwrap();
    std::fs::write(owner.path().join("a.rs"), "fn renamed() {}\n").unwrap();
    let buffer = Arc::new(SourceBuffer::from_str("a.rs", "// café\nfn boot() {}\n"));
    let step = crate::code_paths::CodePath::parse("T\n./a.rs:boot:Begin")
        .unwrap()
        .steps
        .remove(0);
    let source = SourceRead::Checkout(owner.path().into());
    let resolved = resolve_step(&step, &source, Some(&buffer), &AtomicBool::new(false)).unwrap();
    assert!(Arc::ptr_eq(&buffer, &resolved.buffer));
    assert_eq!(resolved.destinations[0].line, 2);
    assert!(resolve_step(&step, &source, None, &AtomicBool::new(false)).is_err());
    assert_eq!(
        destinations("boot", &buffer, &resolved.syntax, &AtomicBool::new(true)).unwrap_err(),
        ResolveError::Cancelled
    );
}

#[test]
fn review_resolution_uses_pinned_after_and_ignores_a_displayed_before_buffer() {
    let before = "fn before() {}\n";
    let after = "// 🦀\nfn boot() {}\n";
    let (_owner, root, review) =
        crate::pull_request::review::tests::fixture_with_sources(before, after);
    let review = Arc::new(review);
    let open_before = Arc::new(review.read("a file.rs", ReviewRevision::Before).unwrap());
    std::fs::write(root.join("a file.rs"), "fn checkout() {}\n").unwrap();
    std::fs::write(review.head_root.join("a file.rs"), "fn materialized() {}\n").unwrap();
    let step = crate::code_paths::CodePath::parse("T\na file.rs:boot:Begin")
        .unwrap()
        .steps
        .remove(0);
    let resolved = resolve_step(
        &step,
        &SourceRead::Review(review),
        Some(&open_before),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(resolved.buffer.text, after);
    assert_eq!(
        resolved.destinations[0].location.revision,
        Some(ReviewRevision::After)
    );
    assert_eq!(resolved.destinations[0].line, 2);
    assert!(resolve_step(
        &step,
        &SourceRead::Buffer(open_before),
        None,
        &AtomicBool::new(false)
    )
    .is_err());
}

#[cfg(unix)]
#[test]
fn checkout_reads_reject_external_symlinks_missing_and_binary_files() {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    let outside = owner.path().join("outside.rs");
    std::fs::write(&outside, "fn boot() {}\n").unwrap();
    std::os::unix::fs::symlink(&outside, root.join("outside.rs")).unwrap();
    std::fs::write(root.join("binary.rs"), [0, 1, 2]).unwrap();
    for path in ["outside.rs", "missing.rs", "binary.rs"] {
        let step = crate::code_paths::CodePath::parse(&format!("T\n{path}:boot:Begin"))
            .unwrap()
            .steps
            .remove(0);
        assert!(resolve_step(
            &step,
            &SourceRead::Checkout(root.clone()),
            None,
            &AtomicBool::new(false)
        )
        .is_err());
    }
}

#[test]
fn masking_preserves_unicode_crlf_coordinates_and_cancels_without_partial_results() {
    let text = "// café 🦀\r\n  ENTRY(_start)\r\n .asciz \"_start:\"\r\n";
    assert_eq!(
        resolve("head.S", text, "_start").unwrap()[0]
            .location
            .byte_offset,
        text.find("_start").unwrap()
    );
    let checks = std::cell::Cell::new(0);
    let result = source_text::masked(&vec![b'x'; 1000], &|| {
        checks.set(checks.get() + 1);
        if checks.get() == 10 {
            Err(ResolveError::Cancelled)
        } else {
            Ok(())
        }
    });
    assert_eq!(result.unwrap_err(), ResolveError::Cancelled);
    assert_eq!(checks.get(), 10);
}
