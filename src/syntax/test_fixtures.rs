//! Shared checked-in Swift corpus for syntax and actual reader acceptance.

pub(crate) const SWIFT_SOURCE_FIXTURES: &[(&str, &str, &str)] = &[
    (
        "sample.swift",
        include_str!("../../Tests/RadiologyTests/Fixtures/sample.swift"),
        "render",
    ),
    (
        "sample.rs",
        include_str!("../../Tests/RadiologyTests/Fixtures/sample.rs"),
        "render",
    ),
    (
        "sample.go",
        include_str!("../../Tests/RadiologyTests/Fixtures/sample.go"),
        "Render",
    ),
    (
        "sample.py",
        include_str!("../../Tests/RadiologyTests/Fixtures/sample.py"),
        "render",
    ),
    (
        "sample.js",
        include_str!("../../Tests/RadiologyTests/Fixtures/sample.js"),
        "render",
    ),
    (
        "sample.jsx",
        include_str!("../../Tests/RadiologyTests/Fixtures/sample.jsx"),
        "ScanList",
    ),
    (
        "sample.ts",
        include_str!("../../Tests/RadiologyTests/Fixtures/sample.ts"),
        "render",
    ),
    (
        "sample.tsx",
        include_str!("../../Tests/RadiologyTests/Fixtures/sample.tsx"),
        "ScanList",
    ),
    (
        "sample.kt",
        include_str!("../../Tests/RadiologyTests/Fixtures/sample.kt"),
        "render",
    ),
    (
        "sample.kts",
        include_str!("../../Tests/RadiologyTests/Fixtures/sample.kts"),
        "collect",
    ),
    (
        "sample.c",
        include_str!("../../Tests/RadiologyTests/Fixtures/sample.c"),
        "titles",
    ),
    (
        "sample.cpp",
        include_str!("../../Tests/RadiologyTests/Fixtures/sample.cpp"),
        "load",
    ),
];

// Exact SyntaxWorkerTests recovery corpus, shared with actual app warning checks.
pub(crate) const MALFORMED_SOURCE_FIXTURES: &[(&str, &str)] = &[
    ("Broken.swift", "struct Broken { func run( { return 1 }"),
    ("broken.rs", "fn main( { let x = ; }"),
    ("broken.go", "package main\nfunc main( { x := }"),
    ("broken.py", "def broken(:\n    return ["),
    ("broken.js", "function broken( { const x = ; }"),
    ("broken.jsx", "const A = () => <div className=;"),
    ("broken.ts", "interface Broken { name: ; }\nfunction f( {"),
    ("broken.tsx", "const A: FC = () => <div className=;"),
    ("broken.kt", "fun broken( { val x = }"),
    ("broken.kts", "val x = ; class {"),
];
