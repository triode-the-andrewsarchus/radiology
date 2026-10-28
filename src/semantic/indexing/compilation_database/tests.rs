use super::*;
use serde_json::{json, Value};
use std::fs;

struct Fixture {
    _owner: tempfile::TempDir,
    original: PathBuf,
    workspace: PathBuf,
    run: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let owner = tempfile::tempdir().unwrap();
        let original = owner.path().join("repository with spaces;literal");
        let workspace = owner.path().join("copy with spaces");
        let run = workspace.join(".run");
        for root in [&original, &workspace] {
            fs::create_dir_all(root.join("src")).unwrap();
            fs::create_dir_all(root.join("include")).unwrap();
            fs::write(
                root.join("src/main file.cpp"),
                "#include <header.h>\nint main() { return VALUE; }\n",
            )
            .unwrap();
            fs::write(root.join("include/header.h"), "#define VALUE 0\n").unwrap();
        }
        fs::create_dir(&run).unwrap();
        Self {
            _owner: owner,
            original,
            workspace,
            run,
        }
    }

    fn rebase(&self, commands: Value) -> Result<RebasedDatabase, String> {
        rebase(
            &serde_json::to_vec(&commands).unwrap(),
            &self.original,
            &self.workspace,
            &self.run,
            &AtomicBool::new(false),
        )
    }

    fn command(&self, arguments: Value) -> Value {
        json!([{"directory": self.original.join("build"), "file": "../src/main file.cpp", "arguments": arguments}])
    }
}

#[test]
fn rebases_absolute_and_relative_paths_preserving_duplicate_configurations_and_literals() {
    let fixture = Fixture::new();
    let original = &fixture.original;
    let macro_literal = format!("-DORIGINAL=\"{}\"", original.display());
    let arguments = json!([
        "/usr/bin/clang++",
        "-I",
        "../include",
        format!("-isystem{}", original.join("include").display()),
        "-isystem",
        "/usr/include",
        "--sysroot=/",
        "-include",
        original.join("include/header.h"),
        macro_literal,
        "-MT",
        "literal-target",
        "-c",
        original.join("src/main file.cpp"),
        "-o",
        "main.o",
        "-MF../deps.d"
    ]);
    let mut commands = fixture.command(arguments);
    commands[0]["output"] = json!("main.o");
    let duplicate = commands[0].clone();
    commands.as_array_mut().unwrap().push(duplicate);
    let rebased = fixture.rebase(commands).unwrap();
    assert_eq!(
        rebased.directories,
        BTreeSet::from([fixture.workspace.join("build")])
    );
    let commands: Value = serde_json::from_slice(&rebased.bytes).unwrap();
    assert_eq!(commands[0], commands[1]);
    let command = &commands[0];
    assert_eq!(
        command["file"],
        json!(fixture.workspace.join("src/main file.cpp"))
    );
    assert_eq!(
        command["output"],
        json!(fixture.workspace.join("build/main.o"))
    );
    let arguments = command["arguments"].as_array().unwrap();
    assert_eq!(arguments[0], "/usr/bin/clang++");
    assert_eq!(arguments[2], json!(fixture.workspace.join("include")));
    assert_eq!(
        arguments[3],
        format!("-isystem{}", fixture.workspace.join("include").display())
    );
    assert!(arguments.contains(&json!("/usr/include")));
    assert!(arguments.contains(&json!("--sysroot=/")));
    assert!(arguments.contains(&json!(macro_literal)));
    assert!(arguments.contains(&json!("literal-target")));
    assert!(arguments.contains(&json!(format!(
        "-MF{}",
        fixture.workspace.join("deps.d").display()
    ))));
}

#[test]
fn shell_quoted_commands_become_literal_argv_and_explicit_arguments_take_precedence() {
    let fixture = Fixture::new();
    let commands = json!([
        {"directory": fixture.original, "file": "src/main file.cpp", "command": "clang++ '-Iinclude' '-DNAME=hello world' '-DCMD=$(touch sentinel)' -c 'src/main file.cpp'"},
        {"directory": fixture.original, "file": "src/main file.cpp", "arguments": ["clang++", "-c", "src/main file.cpp"], "command": "invalid '"}
    ]);
    let rebased = fixture.rebase(commands).unwrap();
    let commands: Value = serde_json::from_slice(&rebased.bytes).unwrap();
    assert!(commands[0].get("command").is_none());
    let arguments = commands[0]["arguments"].as_array().unwrap();
    assert!(arguments.contains(&json!("-DNAME=hello world")));
    assert!(arguments.contains(&json!("-DCMD=$(touch sentinel)")));
    assert!(!fixture.original.join("sentinel").exists());
    assert!(!fixture.workspace.join("sentinel").exists());
    assert_eq!(
        commands[1]["arguments"][2],
        json!(fixture.workspace.join("src/main file.cpp"))
    );
}

#[test]
fn copied_nested_responses_expand_with_gnu_quoting_and_external_outputs_are_private() {
    let fixture = Fixture::new();
    fs::write(
        fixture.workspace.join("compile.rsp"),
        "-Iinclude @nested.rsp -c \"src/main file.cpp\"",
    )
    .unwrap();
    fs::write(
        fixture.workspace.join("nested.rsp"),
        "-DNAME='hello world' -DCHAR='a\\\\b' -DNUM=#123",
    )
    .unwrap();
    let commands = json!([{"directory": fixture.original, "file": "src/main file.cpp", "arguments": ["clang++", "@compile.rsp", "-o", "/external/artifact.o", "-fmodules-cache-path=/external/modules"], "output": "/external/artifact.o"}]);
    let rebased = fixture.rebase(commands).unwrap();
    let commands: Value = serde_json::from_slice(&rebased.bytes).unwrap();
    let arguments = commands[0]["arguments"].as_array().unwrap();
    assert!(arguments.contains(&json!("-DNAME=hello world")));
    assert!(arguments.contains(&json!("-DCHAR=a\\b")));
    assert!(arguments.contains(&json!("-DNUM=#123")));
    assert_eq!(arguments[arguments.len() - 2], commands[0]["output"]);
    assert!(commands[0]["output"]
        .as_str()
        .unwrap()
        .starts_with(fixture.run.to_str().unwrap()));
    assert!(!String::from_utf8(rebased.bytes)
        .unwrap()
        .contains("/external/"));
}

#[test]
fn invalid_or_escaping_commands_and_response_cycles_fail_with_entry_diagnostics() {
    let fixture = Fixture::new();
    let invalid = [
        json!([]),
        json!([{"directory": "/outside", "file": "main.cpp", "arguments": ["clang++"]}]),
        fixture.command(json!([])),
        fixture.command(json!(["clang++", "-I"])),
        fixture.command(json!(["clang++", "@absent.rsp"])),
        fixture.command(json!(["clang++", "@/outside/compile.rsp"])),
        fixture.command(json!(["clang++", "-DVALUE=\u{0000}"])),
        fixture.command(json!([
            "clang++",
            format!("-unsupported={}", fixture.original.display())
        ])),
        json!([{"directory": fixture.original, "file": "../escape.cpp", "arguments": ["clang++"]}]),
        json!([{"directory": fixture.original, "file": "src/main file.cpp", "command": "clang++ 'broken"}]),
    ];
    for command in invalid {
        assert!(fixture.rebase(command.clone()).is_err(), "{command}");
    }
    fs::write(fixture.workspace.join("cycle.rsp"), "@cycle.rsp").unwrap();
    let error = fixture.rebase(json!([{"directory": fixture.original, "file": "src/main file.cpp", "arguments": ["clang++", "@cycle.rsp"]}])).err().unwrap();
    assert!(error.contains("Compilation command 1"));
    assert!(error.contains("nesting"));
    assert!(rebase(
        b"[]",
        &fixture.original,
        &fixture.workspace,
        &fixture.run,
        &AtomicBool::new(true)
    )
    .is_err());
}

#[test]
fn response_and_database_limits_and_private_root_boundaries_are_enforced() {
    let fixture = Fixture::new();
    let command = json!([{"directory": fixture.original, "file": "src/main file.cpp", "arguments": ["clang++", "@large.rsp"]}]);
    let response = fs::File::create(fixture.workspace.join("large.rsp")).unwrap();
    response.set_len(MAX_RESPONSE_BYTES + 1).unwrap();
    assert!(fixture
        .rebase(command.clone())
        .err()
        .unwrap()
        .contains("8 MiB"));
    fs::write(fixture.workspace.join("large.rsp"), "'unterminated").unwrap();
    assert!(fixture
        .rebase(command.clone())
        .err()
        .unwrap()
        .contains("quote"));
    fs::remove_file(fixture.workspace.join("large.rsp")).unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            fixture.original.join("src/main file.cpp"),
            fixture.workspace.join("large.rsp"),
        )
        .unwrap();
        assert!(fixture.rebase(command).err().unwrap().contains("escapes"));
    }
    let oversized = vec![b' '; MAX_BYTES + 1];
    assert!(rebase(
        &oversized,
        &fixture.original,
        &fixture.workspace,
        &fixture.run,
        &AtomicBool::new(false)
    )
    .err()
    .unwrap()
    .contains("32 MiB"));
    assert!(rebase(
        b"[]",
        &fixture.original,
        &fixture.workspace,
        &fixture.workspace.join("../outside"),
        &AtomicBool::new(false)
    )
    .is_err());
}

#[test]
#[ignore = "requires Clang; run explicitly for compilation-database acceptance"]
fn real_clang_compiles_rebased_inputs_with_the_original_repository_unavailable() {
    let fixture = Fixture::new();
    fs::write(
        fixture.workspace.join("flags.rsp"),
        "-I../include -DNAME='hello world' -c '../src/main file.cpp' -o main.o",
    )
    .unwrap();
    let original_response = fixture.original.join("flags.rsp");
    let commands = fixture.command(json!([
        "clang++",
        format!("@{}", original_response.display())
    ]));
    let rebased = fixture.rebase(commands).unwrap();
    let commands: Value = serde_json::from_slice(&rebased.bytes).unwrap();
    for directory in &rebased.directories {
        fs::create_dir_all(directory).unwrap();
    }
    let unavailable = fixture.original.with_file_name("original-unavailable");
    fs::rename(&fixture.original, &unavailable).unwrap();
    let command = &commands[0];
    let arguments = command["arguments"].as_array().unwrap();
    let result = std::process::Command::new(arguments[0].as_str().unwrap())
        .args(
            arguments[1..]
                .iter()
                .map(|argument| argument.as_str().unwrap()),
        )
        .current_dir(command["directory"].as_str().unwrap())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(fixture.workspace.join("build/main.o").is_file());
    assert!(!unavailable.join("build/main.o").exists());
}

#[test]
fn command_hash_literals_and_quoted_escapes_are_preserved_without_shell_comments() {
    let fixture = Fixture::new();
    let commands = json!([{"directory": fixture.original, "file": "src/main file.cpp", "command": "clang++ -D HASH=#literal -D 'QUOTED=#value' -D \"DOUBLE=\\\"quoted\\\"\" -c 'src/main file.cpp' #literal"}]);
    let rebased = fixture.rebase(commands).unwrap();
    let commands: Value = serde_json::from_slice(&rebased.bytes).unwrap();
    let arguments = commands[0]["arguments"].as_array().unwrap();
    assert!(arguments.contains(&json!("HASH=#literal")));
    assert!(arguments.contains(&json!("QUOTED=#value")));
    assert!(arguments.contains(&json!("DOUBLE=\"quoted\"")));
    assert_eq!(arguments.last().unwrap(), "#literal");
}
