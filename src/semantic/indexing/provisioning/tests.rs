use super::*;
use std::{fs, os::unix::fs::PermissionsExt};

pub(crate) struct Fixture {
    pub _owner: tempfile::TempDir,
    pub source: PathBuf,
    pub bin: PathBuf,
    pub tools: ToolDirectory,
}
impl Fixture {
    pub fn new() -> Self {
        let owner = tempfile::tempdir().unwrap();
        let source = owner.path().join("viewed repository");
        let bin = owner.path().join("prerequisites");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&bin).unwrap();
        fs::write(source.join("marker.txt"), "untouched checkout").unwrap();
        let tools = ToolDirectory {
            managed: Some(owner.path().join("tools with spaces;$literal'quote")),
            home: None,
            search_path: vec![bin.clone()],
        };
        Self {
            _owner: owner,
            source,
            bin,
            tools,
        }
    }
    pub fn program(&self, name: &str, script: &str) {
        let path = self.bin.join(name);
        fs::write(&path, format!("#!/bin/sh\nset -eu\n{script}\n")).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    pub fn npm(&self) {
        self.program("node", "exit 0");
        self.program("python3", "exit 0");
        self.program("npm", r#"
script_dir=$(/usr/bin/dirname "$0")
printf '%s\n' "$@" > "$script_dir/npm-args"
if [ -f "$script_dir/pause" ]; then printf '%s' "$$" > "$script_dir/paused.pid"; exec /bin/sleep 30; fi
prefix=''
while [ "$#" -gt 0 ]; do case "$1" in --prefix) shift; prefix=$1;; esac; shift; done
/bin/mkdir -p "$prefix/node_modules/.bin"
if [ -f "$script_dir/fail" ]; then printf 'controlled installation failure' >&2; exit 19; fi
/bin/cat > "$prefix/node_modules/.bin/scip-typescript" <<'TOOL'
#!/bin/sh
if [ "${1:-}" = --version ]; then printf 'scip-typescript 0.4.0\n'; exit; fi
while [ "$#" -gt 0 ]; do case "$1" in --output) shift; output=$1;; esac; shift; done
/bin/cat "$(/usr/bin/dirname "$0")/fixture.scip" > "$output"
TOOL
/bin/chmod 755 "$prefix/node_modules/.bin/scip-typescript"
if [ -f "$script_dir/fixture.scip" ]; then /bin/cp "$script_dir/fixture.scip" "$prefix/node_modules/.bin/fixture.scip"; fi
"#);
    }
    pub fn request(&self, indexer: Indexer) -> ProvisioningRequest {
        ProvisioningRequest {
            indexer,
            tools: self.tools.clone(),
            protected_roots: vec![self.source.clone()],
            timeout: Duration::from_secs(5),
        }
    }
    fn install(&self, indexer: Indexer) -> Result<ProvisionedTool, String> {
        install(
            self.request(indexer),
            &AtomicBool::new(false),
            &Mutex::new(String::new()),
        )
    }
}

#[test]
fn swift_pinned_plans_require_prerequisites_and_use_literal_private_paths() {
    let fixture = Fixture::new();
    for name in [
        "rustup", "go", "node", "npm", "python3", "java", "curl", "tar", "unzip",
    ] {
        fixture.program(name, "exit 0");
    }
    let ts = Indexer::TypeScript.provisioning(&fixture.tools).unwrap();
    assert_eq!(
        ts.command.arguments.last().unwrap(),
        "@sourcegraph/scip-typescript@0.4.0"
    );
    assert_eq!(
        ts.command.arguments[4],
        fixture
            .tools
            .managed
            .as_ref()
            .unwrap()
            .join("npm/scip-typescript")
    );
    assert_eq!(
        Indexer::Python
            .provisioning(&fixture.tools)
            .unwrap()
            .command
            .arguments
            .last()
            .unwrap(),
        "@sourcegraph/scip-python@0.6.6"
    );
    assert_eq!(
        Indexer::Go
            .provisioning(&fixture.tools)
            .unwrap()
            .command
            .arguments[1],
        "github.com/scip-code/scip-go/cmd/scip-go@v0.2.7"
    );
    assert_eq!(
        Indexer::Rust
            .provisioning(&fixture.tools)
            .unwrap()
            .command
            .arguments,
        ["component", "add", "rust-analyzer"].map(std::ffi::OsString::from)
    );
    let kotlin = Indexer::Kotlin.provisioning(&fixture.tools).unwrap();
    assert_eq!(kotlin.command.arguments[2], plan::KOTLIN_COMMIT);
    assert_eq!(kotlin.command.arguments[3], plan::KOTLIN_VERSION);
    assert_eq!(kotlin.command.arguments.last().unwrap(), "9.7.1");
    assert_eq!(kotlin.command.resources.len(), 1);
    assert!(Indexer::Clang
        .provisioning(&fixture.tools)
        .unwrap_err()
        .contains("manually"));
    fs::remove_file(fixture.bin.join("node")).unwrap();
    assert!(Indexer::TypeScript
        .provisioning(&fixture.tools)
        .unwrap_err()
        .contains("Missing node"));
    assert!(
        !fixture.tools.managed.as_ref().unwrap().exists(),
        "planning never writes tools"
    );
}

#[test]
fn staged_npm_install_verifies_and_publishes_without_viewed_tree_writes() {
    let fixture = Fixture::new();
    fixture.npm();
    let installed = fixture.install(Indexer::TypeScript).unwrap();
    let root = fixture.tools.managed.as_ref().unwrap();
    assert!(
        matches!(installed.availability, ToolAvailability::Available { executable, version } if executable.starts_with(root) && version == "scip-typescript 0.4.0")
    );
    let arguments = fs::read_to_string(fixture.bin.join("npm-args")).unwrap();
    assert!(arguments.contains("@sourcegraph/scip-typescript@0.4.0"));
    assert!(arguments.contains(".tool-install-"));
    assert_eq!(
        fs::read_to_string(fixture.source.join("marker.txt")).unwrap(),
        "untouched checkout"
    );
    assert_eq!(fs::read_dir(&fixture.source).unwrap().count(), 1);
    assert!(fs::read_dir(root).unwrap().all(|entry| !entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".tool-install-")));
    let previous =
        fs::read(root.join("npm/scip-typescript/node_modules/.bin/scip-typescript")).unwrap();
    fs::write(fixture.bin.join("fail"), "").unwrap();
    assert!(fixture
        .install(Indexer::TypeScript)
        .err()
        .unwrap()
        .contains("controlled installation failure"));
    assert_eq!(
        fs::read(root.join("npm/scip-typescript/node_modules/.bin/scip-typescript")).unwrap(),
        previous
    );
}

#[test]
fn successful_installer_without_a_managed_binary_cannot_verify_a_path_fallback() {
    let fixture = Fixture::new();
    fixture.npm();
    fixture.program("npm", "exit 0");
    fixture.program("scip-typescript", "printf 'unrelated PATH version' ");
    assert!(fixture
        .install(Indexer::TypeScript)
        .err()
        .unwrap()
        .contains("managed executable"));
}

#[test]
fn viewed_root_overlap_is_rejected_before_installation_or_private_writes() {
    let fixture = Fixture::new();
    fixture.npm();
    let mut request = fixture.request(Indexer::TypeScript);
    request.tools.managed = Some(fixture.source.join("tools"));
    assert!(
        install(request, &AtomicBool::new(false), &Mutex::new(String::new()))
            .err()
            .unwrap()
            .contains("outside viewed")
    );
    assert!(!fixture.source.join("tools").exists());
    assert!(!fixture.bin.join("npm-args").exists());
}

#[test]
fn live_installer_cancel_and_timeout_reap_children_release_lease_and_keep_prior_tool() {
    let fixture = Fixture::new();
    fixture.npm();
    fixture.install(Indexer::TypeScript).unwrap();
    let root = fixture.tools.managed.as_ref().unwrap();
    let executable = root.join("npm/scip-typescript/node_modules/.bin/scip-typescript");
    let previous = fs::read(&executable).unwrap();
    fs::write(fixture.bin.join("pause"), "").unwrap();
    for timeout in [false, true] {
        let marker = fixture.bin.join("paused.pid");
        let _ = fs::remove_file(&marker);
        let cancel = AtomicBool::new(false);
        let control = crate::pull_request::control::ReviewControl::new(&cancel);
        let mut request = fixture.request(Indexer::TypeScript);
        if timeout {
            request.timeout = Duration::from_millis(500);
        }
        crate::pull_request::test_support::interrupted(&control, &marker, timeout, || {
            install(request, &cancel, &Mutex::new(String::new()))
        });
        assert_eq!(fs::read(&executable).unwrap(), previous);
        // A fresh request must recover after descriptor retirement, including
        // concurrent fork/exec; an instantaneous nonblocking probe is too strict.
        fs::remove_file(fixture.bin.join("pause")).unwrap();
        fixture.install(Indexer::TypeScript).unwrap();
        fs::write(fixture.bin.join("pause"), "").unwrap();
        assert!(fs::read_dir(root).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".tool-install-")));
    }
}

#[test]
fn publication_verification_failure_restores_the_previous_installation() {
    let fixture = Fixture::new();
    let root = fixture.tools.managed.as_ref().unwrap();
    fs::create_dir_all(root).unwrap();
    let source = root.join("candidate");
    let destination = root.join("current");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&destination).unwrap();
    fs::write(source.join("tool"), "candidate bytes").unwrap();
    fs::write(destination.join("tool"), "accepted bytes").unwrap();
    let result = publish(&source, &destination, root, || {
        Err("post-publication probe failed".into())
    });
    assert!(result.err().unwrap().contains("probe failed"));
    assert_eq!(
        fs::read_to_string(destination.join("tool")).unwrap(),
        "accepted bytes"
    );
    assert!(!source.exists());
}

#[test]
fn writer_wait_uses_the_installation_budget_without_starting_a_package_manager() {
    let fixture = Fixture::new();
    fixture.npm();
    let root = fixture.tools.managed.as_ref().unwrap();
    fs::create_dir_all(root).unwrap();
    let held = process::lease::open_lock(&root.join("install.lock"), true).unwrap();
    assert!(process::lease::try_lock(&held, false).unwrap());
    let mut request = fixture.request(Indexer::TypeScript);
    request.timeout = Duration::from_millis(50);
    assert!(
        install(request, &AtomicBool::new(false), &Mutex::new(String::new()))
            .err()
            .unwrap()
            .contains("timed out")
    );
    assert!(!fixture.bin.join("npm-args").exists());
}

#[test]
fn go_and_rustup_commands_execute_and_verify_the_selected_executables() {
    let fixture = Fixture::new();
    fixture.program(
        "go",
        r#"
script_dir=$(/usr/bin/dirname "$0")
printf '%s\n' "$@" > "$script_dir/go-args"
/bin/mkdir -p "$GOBIN"
printf '#!/bin/sh\nprintf "scip-go v0.2.7\\n"\n' > "$GOBIN/scip-go"
/bin/chmod 755 "$GOBIN/scip-go"
"#,
    );
    let go = fixture.install(Indexer::Go).unwrap();
    assert!(
        matches!(go.availability, ToolAvailability::Available { version, .. } if version == "scip-go v0.2.7")
    );
    assert_eq!(
        fs::read_to_string(fixture.bin.join("go-args")).unwrap(),
        "install\ngithub.com/scip-code/scip-go/cmd/scip-go@v0.2.7\n"
    );
    fixture.program(
        "rustup",
        r#"
script_dir=$(/usr/bin/dirname "$0")
printf '%s\n' "$@" > "$script_dir/rustup-args"
/bin/mkdir -p bin
printf '#!/bin/sh\nprintf "rust-analyzer fixture\\n"\n' > bin/rust-analyzer
/bin/chmod 755 bin/rust-analyzer
"#,
    );
    let rust = fixture.install(Indexer::Rust).unwrap();
    assert!(
        matches!(rust.availability, ToolAvailability::Available { version, .. } if version == "rust-analyzer fixture")
    );
    assert_eq!(
        fs::read_to_string(fixture.bin.join("rustup-args")).unwrap(),
        "component\nadd\nrust-analyzer\n"
    );
    assert_eq!(
        fs::read_to_string(fixture.source.join("marker.txt")).unwrap(),
        "untouched checkout"
    );
}
