use super::*;
use crate::semantic::indexing::IndexPlan;

struct Fixture {
    owner: tempfile::TempDir,
    paths: Vec<String>,
}
impl Fixture {
    fn new(files: &[(&str, &str)]) -> Self {
        let mut fixture = Self {
            owner: tempfile::tempdir().unwrap(),
            paths: Vec::new(),
        };
        for (path, text) in files {
            fixture.add(path, text);
        }
        fixture
    }
    fn add(&mut self, path: &str, text: &str) {
        let file = self.owner.path().join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, text).unwrap();
        self.paths.push(path.into());
    }
    fn entries(&self) -> Vec<serde_json::Value> {
        let inventory = InputInventory::from_paths(self.paths.clone()).unwrap();
        serde_json::from_slice(
            &database(
                self.owner.path(),
                &inventory,
                Path::new("/usr/bin/clang"),
                &AtomicBool::new(false),
            )
            .unwrap(),
        )
        .unwrap()
    }
}

fn includes(entry: &serde_json::Value) -> Vec<&str> {
    entry["arguments"]
        .as_array()
        .unwrap()
        .windows(2)
        .filter(|pair| pair[0] == "-I")
        .map(|pair| pair[1].as_str().unwrap())
        .collect()
}

#[test]
fn defaults_plan_is_versioned_and_real_databases_always_take_priority() {
    let inventory = [
        "src/main.C",
        "src/types.h",
        "inc/templates.tcc",
        "macros.h++",
    ];
    let fallback = IndexPlan::inspect(inventory);
    assert_eq!(fallback.selected.len(), 1);
    assert!(fallback.selected[0].is_best_effort());
    assert_eq!(fallback.selected[0].source_files, 4);
    for database in ["build/compile_commands.json", "compile_commands.json"] {
        let plan = IndexPlan::inspect(inventory.into_iter().chain([database]));
        assert_eq!(plan.selected[0].evidence, [database]);
        assert!(!plan.selected[0].is_best_effort());
        assert_ne!(plan.selected[0], fallback.selected[0]);
    }
    let plan = IndexPlan::inspect([
        "build/compile_commands.json",
        "compile_commands.json",
        "main.cpp",
    ]);
    assert_eq!(plan.selected[0].evidence, ["compile_commands.json"]);
    assert!(IndexPlan::inspect(["README.md", "main.m"])
        .selected
        .is_empty());
}

#[test]
fn language_defaults_preserve_uppercase_c_and_literal_dash_prefixed_names() {
    let fixture = Fixture::new(&[
        ("-load.c", "void a(void) {}"),
        ("upper.C", "class A {};"),
        ("units/a.c++", "void f() {}"),
        ("include/api.h", "class B {};"),
        ("lib/extra.tpp", "template<class T> struct S {};"),
    ]);
    let entries = fixture.entries();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]["file"], "-load.c");
    assert_eq!(entries[0]["arguments"][2], "c");
    assert_eq!(entries[0]["arguments"][3], "-std=gnu11");
    assert_eq!(
        entries[0]["arguments"].as_array().unwrap().last().unwrap(),
        "./-load.c"
    );
    let cpp = entries
        .iter()
        .find(|entry| entry["file"] == "upper.C")
        .unwrap();
    assert_eq!(cpp["arguments"][2], "c++");
    assert_eq!(cpp["arguments"][3], "-std=gnu++20");
    assert_eq!(fixture.entries(), entries);
    for path in [
        "upper.C",
        "api.h",
        "api.HPP",
        "inline.ipp",
        "template.tpp",
        "template.tcc",
    ] {
        assert_eq!(
            crate::syntax::LanguageRegistry::for_path(path).grammar,
            Some(crate::syntax::SupportedGrammar::Cpp)
        );
        assert!(Indexer::Clang.covers_path(path));
    }
}

#[test]
fn header_only_inputs_get_real_cpp_declarations_without_a_translation_unit_cap() {
    let mut fixture = Fixture::new(&[
        ("api.h", "template<class T> T id(T value) { return value; }"),
        ("detail/api.tcc", "void extra();"),
    ]);
    let entries = fixture.entries();
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().all(|entry| entry["arguments"][2] == "c++"));
    for i in 0..2050 {
        fixture.add(&format!("src/unit{i}.c"), "void start(void) {}");
    }
    let entries = fixture.entries();
    assert_eq!(entries.len(), 2050);
    assert!(entries
        .iter()
        .all(|entry| entry["file"].as_str().unwrap().ends_with(".c")));
}

#[test]
fn literal_includes_follow_nested_headers_and_cycles_with_nearby_preference() {
    let fixture = Fixture::new(&[
        (
            "program/src/main.c",
            "#include \"driver/api.h\"\n#include <absent.h>\nvoid main(void) {}",
        ),
        ("program/private/driver/api.h", "#include <platform.h>\n"),
        ("z/driver/api.h", "#include <remote.h>\n"),
        ("platform/include/platform.h", "#include <driver/api.h>\n"),
        ("remote/inc/remote.h", "void remote(void);"),
    ]);
    let entries = fixture.entries();
    let paths = includes(&entries[0]);
    let root = fixture.owner.path();
    let expected = [
        "program/src",
        "program",
        "",
        "program/private",
        "platform/include",
        "z",
        "remote/inc",
    ]
    .map(|path| root.join(path));
    assert_eq!(
        paths,
        expected
            .iter()
            .map(|path| path.to_str().unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(paths.iter().collect::<HashSet<_>>().len(), paths.len());
}

#[test]
fn ambiguous_header_search_is_deterministic_bounded_and_cancellable() {
    let mut fixture = Fixture::new(&[("src/main.c", "#include <api.h>\n")]);
    for i in 0..100 {
        fixture.add(&format!("headers/{i:03}/api.h"), "void api(void);");
    }
    let entries = fixture.entries();
    let paths = includes(&entries[0]);
    let inferred: Vec<_> = paths
        .iter()
        .filter(|path| path.contains("/headers/"))
        .collect();
    assert_eq!(inferred.len(), 8);
    assert!(inferred[0].ends_with("headers/000"));
    assert!(inferred[7].ends_with("headers/007"));
    let inventory = InputInventory::from_paths(fixture.paths).unwrap();
    assert!(database(
        fixture.owner.path(),
        &inventory,
        Path::new("/usr/bin/clang"),
        &AtomicBool::new(true)
    )
    .unwrap_err()
    .contains("cancelled"));
    assert!(database(
        fixture.owner.path(),
        &inventory,
        Path::new("clang"),
        &AtomicBool::new(false)
    )
    .is_err());
}

#[test]
fn malformed_inventory_is_rejected_and_unreadable_headers_do_not_fabricate_directories() {
    let fixture = Fixture::new(&[
        ("main.c", "#include <broken.h>\n"),
        ("include/broken.h", ""),
    ]);
    std::fs::remove_file(fixture.owner.path().join("include/broken.h")).unwrap();
    let entries = fixture.entries();
    assert_eq!(entries.len(), 1);
    // Only known inventory locations are inferred; the compiler reports the absent file.
    assert!(includes(&entries[0])
        .iter()
        .all(|path| Path::new(path).starts_with(fixture.owner.path())));
    let inventory = InputInventory {
        paths: vec!["../outside.c".into()],
        omitted_directories: vec![],
    };
    assert!(database(
        fixture.owner.path(),
        &inventory,
        Path::new("/usr/bin/clang"),
        &AtomicBool::new(false)
    )
    .is_err());
}

#[test]
fn inferred_paths_and_recursive_header_work_are_bounded_without_dropping_source_units() {
    let mut fixture = Fixture::new(&[("src/main.c", "#include <api0.h>\n")]);
    for i in 0..200 {
        fixture.add(
            &format!("private/{i}/api{i}.h"),
            &format!("#include <api{}.h>\n", i + 1),
        );
    }
    let entries = fixture.entries();
    assert_eq!(entries.len(), 1);
    let paths = includes(&entries[0]);
    assert_eq!(paths.len(), MAX_DIRECTORIES);
    assert!(paths[2].ends_with("private/0"));
}

#[test]
fn swift_default_fixture_infers_layout_headers_and_literal_option_names() {
    let mut fixture = Fixture::new(&[
        (
            "src/main.cpp",
            "#include <layout/api.hpp>\nint run() { return api(); }\n",
        ),
        ("src/main.c", "int main(void) { return 0; }\n"),
        ("src/case.C", "class Reader {};\n"),
        ("support/include/layout/api.hpp", "int api();\n"),
    ]);
    let entries = fixture.entries();
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry["file"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["src/case.C", "src/main.c", "src/main.cpp"]
    );
    let cpp = entries
        .iter()
        .find(|entry| entry["file"] == "src/main.cpp")
        .unwrap();
    assert!(includes(cpp).contains(
        &fixture
            .owner
            .path()
            .join("support/include")
            .to_str()
            .unwrap()
    ));
    assert_eq!(cpp["arguments"][3], "-std=gnu++20");
    assert_eq!(entries[0]["arguments"][3], "-std=gnu++20");
    assert_eq!(entries[1]["arguments"][3], "-std=gnu11");
    let plan = IndexPlan::inspect(
        fixture
            .paths
            .iter()
            .map(String::as_str)
            .chain(["compile_commands.json"]),
    );
    assert_eq!(plan.selected[0].evidence, ["compile_commands.json"]);
    fixture.paths = vec!["support/include/layout/api.hpp".into()];
    assert_eq!(
        fixture.entries()[0]["file"],
        "support/include/layout/api.hpp"
    );
    fixture.add("-fplugin=helper.cpp", "int helper() { return 1; }\n");
    let entries = fixture.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0]["arguments"].as_array().unwrap().last().unwrap(),
        "./-fplugin=helper.cpp"
    );
}
