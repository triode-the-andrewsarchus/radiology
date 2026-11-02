use super::*;

#[test]
fn ordered_unicode_escaped_fields_and_literal_descriptions_round_trip() {
    let trace = CodePath::parse("\u{feff}Boot: café 🚀\r\n\r\nsrc/a\\:b\\\\c.cc:Kernel\\:\\:boot:Starts: CPU setup \\: \\\\ literal\r\nsrc/next.swift:run(label\\:):Continue\r\n").unwrap();
    assert_eq!(trace.title, "Boot: café 🚀");
    assert_eq!(trace.steps[0].path, "src/a:b\\c.cc");
    assert_eq!(trace.steps[0].function, "Kernel::boot");
    assert_eq!(
        trace.steps[0].description,
        "Starts: CPU setup \\: \\\\ literal"
    );
    assert_eq!(trace.steps[1].function, "run(label:)");
    let restored = CodePath::parse(&trace.text()).unwrap();
    assert_eq!(restored.title, trace.title);
    for (before, after) in trace.steps.iter().zip(&restored.steps) {
        assert_eq!(
            (&before.path, &before.function, &before.description),
            (&after.path, &after.function, &after.description)
        );
        assert_ne!(before.id, after.id);
    }
    assert_ne!(trace.id, restored.id);
}

#[test]
fn repeated_steps_unknown_escapes_and_all_newline_styles_keep_authored_order() {
    for newline in ["\n", "\r\n", "\r"] {
        let text = [
            "  Title  ",
            " a.c : fn\\q : first ",
            "",
            "a.c:fn\\q:second",
            "",
        ]
        .join(newline);
        let trace = CodePath::parse(&text).unwrap();
        assert_eq!(trace.title, "Title");
        assert_eq!(trace.steps.len(), 2);
        assert_eq!(trace.steps[0].function, "fn\\q");
        assert_eq!(trace.steps[0].path, "a.c");
        assert_ne!(trace.steps[0].id, trace.steps[1].id);
        assert_eq!(trace.steps[0].description, "first");
        assert_eq!(trace.steps[1].description, "second");
    }
    assert!(CodePath::parse("T\n./dir//a.c:f:x").is_ok());
}

#[test]
fn malformed_inputs_fail_atomically_with_physical_input_line_diagnostics() {
    for text in [
        "",
        "\na.c:f:hello",
        "T",
        "T\na.c:f",
        "T\n/etc/passwd:f:hello",
        "T\n../outside:f:hello",
        "T\na/../b:f:hello",
        "T\na.c::hello",
        "T\na.c:f: ",
        "T\na\0.c:f:x",
    ] {
        assert!(CodePath::parse(text).is_err(), "accepted {text:?}");
    }
    let error = CodePath::parse("Title\r\n\r\na.c:f:okay\r\nnot a step").unwrap_err();
    assert_eq!(error, ImportError::Fields { line: 4 });
    assert!(error.to_string().starts_with("Line 4:"));
    assert_eq!(
        CodePath::parse("T\n\na.c::invalid").unwrap_err(),
        ImportError::InvalidStep { line: 3 }
    );
    let mut trace = CodePath::parse("T\na.c:f:one\na.c:f:two").unwrap();
    trace.steps[1].id = trace.steps[0].id;
    assert!(!trace.is_valid());
}

#[test]
fn import_limit_counts_utf8_bytes_and_accepts_the_exact_boundary() {
    let text = format!("T\na:f:{}", "é".repeat((MAX_IMPORT_BYTES - 6) / 2));
    assert_eq!(text.len(), MAX_IMPORT_BYTES);
    assert!(CodePath::parse(&text).is_ok());
    assert_eq!(
        CodePath::parse(&(text + "x")).unwrap_err(),
        ImportError::TooLarge
    );
}

#[test]
fn upstream_ninety_one_step_sel4_trace_round_trips_without_line_coordinates() {
    let trace = CodePath::parse(include_str!(
        "../../Tests/RadiologyTests/Fixtures/code-paths/sel4-arm-boot.txt"
    ))
    .unwrap();
    assert_eq!(trace.steps.len(), 91);
    assert_eq!(trace.steps[0].path, "src/arch/arm/64/head.S");
    assert_eq!(trace.steps[0].function, "_start");
    assert_eq!(trace.steps[1].path, "src/arch/arm/32/head.S");
    assert_eq!(trace.steps[1].function, "_start");
    let copy = CodePath::parse(&trace.text()).unwrap();
    assert_eq!(copy.text(), trace.text());
    assert!(trace.is_valid() && copy.is_valid());
}
