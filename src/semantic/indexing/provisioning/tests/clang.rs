use super::*;

impl Fixture {
    pub(crate) fn clang(&self) {
        for name in ["mkdir", "mktemp", "rm", "chmod", "mv"] {
            std::os::unix::fs::symlink(format!("/usr/bin/{name}"), self.bin.join(name)).unwrap();
        }
        self.program(
            "clang-download",
            r#"
if [ "${1:-}" = --version ]; then printf 'scip-clang 0.4.0\n'; exit; fi
while [ "$#" -gt 0 ]; do case "$1" in --index-output-path) shift; output=$1;; esac; shift; done
/bin/printf '\022\010\012\006main.c' > "$output"
"#,
        );
        self.program(
            "curl",
            r#"
base=$(/usr/bin/dirname "$0")
printf '%s\n' "$@" > "$base/curl-args"
if [ -f "$base/download-fails" ]; then echo 'controlled download failure' >&2; exit 22; fi
if [ -f "$base/pause" ]; then
    printf '%s' "$$" > "$base/child.pid"
    printf '%s' "$PPID" > "$base/paused.pid"
    exec /bin/sleep 30
fi
for destination do :; done
/bin/cp "$base/clang-download" "$destination"
"#,
        );
        self.program(
            "sha256sum",
            &format!("printf '%s  -\\n' '{}'", plan::CLANG_DIGEST),
        );
    }
}

#[test]
fn clang_plan_pins_linux_asset_digest_and_resolved_prerequisites_without_writes() {
    let fixture = Fixture::new();
    fixture.clang();
    let plan = Indexer::Clang.provisioning(&fixture.tools).unwrap();
    assert_eq!(plan.command.arguments[2], "0.4.0");
    assert_eq!(plan.command.arguments[3], plan::CLANG_DIGEST);
    assert_eq!(plan.command.arguments[4], "scip-clang-x86_64-linux");
    assert_eq!(plan.command.arguments[5], fixture.bin.join("curl"));
    assert_eq!(plan.command.arguments[6], fixture.bin.join("sha256sum"));
    assert_eq!(plan.command.resources.len(), 1);
    assert_eq!(
        Indexer::Sel4
            .provisioning(&fixture.tools)
            .unwrap()
            .command
            .arguments,
        plan.command.arguments
    );
    assert!(!fixture.tools.managed.as_ref().unwrap().exists());
    fs::remove_file(fixture.bin.join("sha256sum")).unwrap();
    assert!(Indexer::Clang
        .provisioning(&fixture.tools)
        .unwrap_err()
        .contains("Missing sha256sum"));
}

#[test]
fn shared_clang_script_checks_real_sha256_with_newline_and_backslash_paths() {
    use sha2::{Digest, Sha256};
    let mut fixture = Fixture::new();
    fixture.clang();
    fixture.tools.managed = Some(fixture._owner.path().join("tools\\literal\nnext line"));
    fs::remove_file(fixture.bin.join("sha256sum")).unwrap();
    std::os::unix::fs::symlink("/usr/bin/sha256sum", fixture.bin.join("sha256sum")).unwrap();
    let digest = format!(
        "{:x}",
        Sha256::digest(fs::read(fixture.bin.join("clang-download")).unwrap())
    );
    let root = fixture.tools.managed.as_ref().unwrap();
    fs::create_dir_all(root).unwrap();
    let lease = writer(
        root,
        &AtomicBool::new(false),
        Instant::now() + Duration::from_secs(5),
    )
    .unwrap();
    for valid in [true, false] {
        let mut plan = Indexer::Clang.provisioning(&fixture.tools).unwrap();
        if valid {
            plan.command.arguments[3] = digest.clone().into();
        }
        let result = execute(
            plan,
            &lease,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(5),
            &Mutex::new(String::new()),
        );
        if valid {
            result.unwrap();
        } else {
            assert!(result.unwrap_err().contains("checksum mismatch"));
        }
        assert_eq!(
            fs::read(root.join("bin/scip-clang")).unwrap(),
            fs::read(fixture.bin.join("clang-download")).unwrap()
        );
        assert_eq!(fs::read_dir(root.join("bin")).unwrap().count(), 1);
    }
}

#[test]
fn clang_staged_install_replaces_only_its_binary_and_preserves_it_on_failures() {
    let fixture = Fixture::new();
    fixture.clang();
    let root = fixture.tools.managed.as_ref().unwrap();
    fs::create_dir_all(root.join("bin")).unwrap();
    fs::write(root.join("bin/unrelated"), "keep this tool").unwrap();
    let installed = fixture.install(Indexer::Clang).unwrap();
    assert!(
        matches!(installed.availability, ToolAvailability::Available { executable, version }
        if executable == root.join("bin/scip-clang") && version == "scip-clang 0.4.0")
    );
    let previous = fs::read(root.join("bin/scip-clang")).unwrap();
    assert_eq!(
        fixture.install(Indexer::Sel4).unwrap().indexer,
        Indexer::Sel4
    );
    let arguments = fs::read_to_string(fixture.bin.join("curl-args")).unwrap();
    assert!(arguments.contains("https://github.com/sourcegraph/scip-clang/releases/download/v0.4.0/scip-clang-x86_64-linux"));
    assert!(arguments.contains(".tool-install-") && arguments.contains(".scip-clang."));
    for failure in ["download", "checksum", "version"] {
        match failure {
            "download" => fs::write(fixture.bin.join("download-fails"), "").unwrap(),
            "checksum" => {
                fs::remove_file(fixture.bin.join("download-fails")).unwrap();
                fixture.program("sha256sum", "printf 'wrong-digest  -\\n'");
            }
            _ => {
                fixture.program(
                    "sha256sum",
                    &format!("printf '%s  -\\n' '{}'", plan::CLANG_DIGEST),
                );
                fixture.program(
                    "clang-download",
                    "echo 'controlled version failure' >&2; exit 7",
                );
            }
        }
        assert!(fixture.install(Indexer::Clang).is_err(), "{failure}");
        assert_eq!(fs::read(root.join("bin/scip-clang")).unwrap(), previous);
    }
    assert_eq!(
        fs::read_to_string(root.join("bin/unrelated")).unwrap(),
        "keep this tool"
    );
    assert_eq!(
        fs::read_to_string(fixture.source.join("marker.txt")).unwrap(),
        "untouched checkout"
    );
    assert!(fs::read_dir(root).unwrap().all(|e| !e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".tool-")));
}

#[test]
fn clang_download_cancellation_and_timeout_preserve_the_accepted_executable() {
    let fixture = Fixture::new();
    fixture.clang();
    interrupted_installation(&fixture, Indexer::Clang);
}
