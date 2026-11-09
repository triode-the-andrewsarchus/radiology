use super::*;
use std::{fs, sync::atomic::AtomicBool};

fn parse(personality: Personality, text: &str) -> Result<Settings, String> {
    Settings::parse(personality, "settings.toml".into(), text.as_bytes())
}

fn default_text() -> String {
    String::from_utf8(Personality::Default.template().to_vec()).unwrap()
}

#[test]
fn shared_templates_capture_typed_settings_all_profiles_and_pipeline_provenance() {
    let automatic = parse(Personality::Default, &default_text()).unwrap();
    assert_eq!(automatic.personality(), Personality::Default);
    assert_eq!(automatic.name(), "Default");
    assert_eq!(automatic.clang_jobs(), 4);
    assert_eq!(automatic.project_limit(), 32);
    assert!(automatic.indexers().is_empty());
    assert_eq!(automatic.bootstrap_python()[0], "python3.11");
    assert_eq!(
        automatic.digest(),
        format!("{:x}", Sha256::digest(automatic.bytes()))
    );
    let sel4 = Settings::parse(
        Personality::Sel4,
        "sel4.toml".into(),
        Personality::Sel4.template(),
    )
    .unwrap();
    assert_eq!(sel4.name(), Personality::Sel4.name());
    assert_eq!(sel4.clang_jobs(), 1);
    assert_eq!(sel4.indexers(), ["scip-sel4"]);
    assert_ne!(sel4.digest(), format!("{:x}", Sha256::digest(sel4.bytes())));
    let recipe = toml_edit::Document::parse(std::str::from_utf8(sel4.bytes()).unwrap()).unwrap();
    assert_eq!(
        recipe["sel4"]["profiles"]
            .as_array_of_tables()
            .unwrap()
            .len(),
        67
    );
    assert_eq!(pipeline_resources().len(), 5);
    assert!(pipeline_resources()
        .iter()
        .all(|(name, bytes)| name.ends_with(".py") && !bytes.is_empty()));
}

#[test]
fn typed_roots_accept_comments_literal_strings_and_multiline_string_arrays() {
    let raw = "name = 'literal # value' # comment\nindexers = [\n \"a,b\", # comment\n 'c',\n]\n[recipe]\nignored = true";
    let document = toml_edit::Document::parse(raw).unwrap();
    let root = parser::Root::new(document.as_table(), raw).unwrap();
    assert_eq!(root.string("name").unwrap(), "literal # value");
    assert_eq!(root.strings("indexers").unwrap(), ["a,b", "c"]);
    let text = default_text()
        .replace("name = \"Default\"", "name = 'literal # value' # comment")
        .replace(
            "indexers = []",
            "indexers = [\n \"scip-go\", # comment\n 'scip-clang',\n]",
        )
        .replace("project_limit = 32", "project_limit = +256")
        .replace("clang_jobs = 4", "clang_jobs = 64");
    let settings = parse(Personality::Default, &text).unwrap();
    assert_eq!(settings.name(), "literal # value");
    assert_eq!(settings.indexers(), ["scip-go", "scip-clang"]);
    assert_eq!(settings.project_limit(), 256);
    assert_eq!(settings.clang_jobs(), 64);
    let text = format!("{text}\nbootstrap_python = ['python3', '/path with spaces/python']\n[recipe]\nignored = true\n");
    assert_eq!(
        parse(Personality::Default, &text)
            .unwrap()
            .bootstrap_python(),
        ["python3", "/path with spaces/python"]
    );
}

#[test]
fn invalid_root_syntax_identity_types_and_limits_never_become_defaults() {
    let template = default_text();
    for (from, to) in [
        ("schema_version = 1", "schema_version = 9"),
        ("schema_version = 1", "schema_version = 0x1"),
        ("id = \"default\"", "id = \"sel4\""),
        ("engine = \"automatic\"", "engine = \"sel4\""),
        ("name = \"Default\"", "name = true"),
        ("name = \"Default\"", "name = \"\"\"Default\"\"\""),
        ("name = \"Default\"", "name = '''one\ntwo'''"),
        ("name = \"Default\"", "name = { value = 'Default' }"),
        ("name = \"Default\"", "\"name\" = \"Default\""),
        ("project_limit = 32", "project_limit = 0"),
        ("project_limit = 32", "project_limit = 257"),
        ("project_limit = 32", "project_limit = 1_2"),
        ("clang_jobs = 4", "clang_jobs = 65"),
        ("clang_jobs = 4", "clang_jobs = -1"),
        ("clang_jobs = 4", "clang_jobs = 1.0"),
        ("indexers = []", "indexers = [1]"),
        ("indexers = []", "indexers = [\"\"\"a\"\"\"]"),
        ("indexers = []", "indexers = ['unterminated'"),
        ("indexers = []", ""),
    ] {
        assert!(
            parse(Personality::Default, &template.replace(from, to)).is_err(),
            "{to}"
        );
    }
    for suffix in [
        "id = 'again'",
        "unknown = 1",
        "nested.root = 1",
        "bootstrap_python = false",
    ] {
        assert!(
            parse(Personality::Default, &format!("{template}\n{suffix}")).is_err(),
            "{suffix}"
        );
    }
    let sel4 = std::str::from_utf8(Personality::Sel4.template()).unwrap();
    assert!(parse(Personality::Sel4, &sel4.replace("[\"scip-sel4\"]", "[]")).is_err());
    assert!(Settings::parse(Personality::Default, "bad.toml".into(), &[0xff]).is_err());
    assert!(Settings::parse(
        Personality::Default,
        "big.toml".into(),
        &vec![b' '; MAX_BYTES as usize + 1]
    )
    .is_err());
}

#[test]
fn loads_preserve_user_edits_invalid_files_and_captured_run_bytes() {
    let owner = tempfile::tempdir().unwrap();
    let store = PersonalityStore::new(owner.path().join("personalities"));
    let cancel = AtomicBool::new(false);
    assert!(!store.path(Personality::Default).exists());
    let original = store.load(Personality::Default, &[], &cancel).unwrap();
    let edited = default_text().replace("clang_jobs = 4", "clang_jobs = 2");
    fs::write(original.path(), &edited).unwrap();
    let changed = store.load(Personality::Default, &[], &cancel).unwrap();
    assert_eq!(changed.clang_jobs(), 2);
    assert_ne!(changed.digest(), original.digest());
    assert_eq!(original.bytes(), Personality::Default.template());
    assert_eq!(changed.bytes(), edited.as_bytes());
    fs::write(original.path(), "invalid settings").unwrap();
    let error = store.load(Personality::Default, &[], &cancel).unwrap_err();
    assert!(error.contains("default.toml"));
    assert!(store.path(Personality::Default).exists());
    assert_eq!(
        fs::read_to_string(original.path()).unwrap(),
        "invalid settings"
    );
}

#[test]
fn concurrent_first_use_and_nonclobber_seed_keep_a_complete_user_file() {
    let owner = tempfile::tempdir().unwrap();
    let store = PersonalityStore::new(owner.path().join("settings"));
    std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    store
                        .load(Personality::Sel4, &[], &AtomicBool::new(false))
                        .unwrap()
                })
            })
            .collect();
        for job in jobs {
            assert_eq!(job.join().unwrap().bytes(), Personality::Sel4.template());
        }
    });
    let path = store.path(Personality::Default);
    crate::storage::atomic_write(&path, b"user edits arriving before a seed").unwrap();
    crate::storage::create_if_missing(&path, Personality::Default.template()).unwrap();
    assert_eq!(
        fs::read(path).unwrap(),
        b"user edits arriving before a seed"
    );
    assert_eq!(
        fs::read_dir(owner.path().join("settings")).unwrap().count(),
        2
    );
}

#[test]
fn protected_roots_cancellation_and_nonregular_files_fail_without_source_writes() {
    let owner = tempfile::tempdir().unwrap();
    let source = owner.path().join("source");
    fs::create_dir(&source).unwrap();
    let store = PersonalityStore::new(source.join("settings"));
    let error = store
        .load(
            Personality::Default,
            std::slice::from_ref(&source),
            &AtomicBool::new(false),
        )
        .unwrap_err();
    assert!(error.contains("outside viewed"));
    assert!(!source.join("settings").exists());
    let store = PersonalityStore::new(owner.path().join("settings"));
    assert!(store
        .load(Personality::Default, &[], &AtomicBool::new(true))
        .unwrap_err()
        .contains("cancelled"));
    assert!(!store.path(Personality::Default).exists());
    fs::create_dir(owner.path().join("settings")).unwrap();
    fs::write(source.join("victim"), "untouched").unwrap();
    std::os::unix::fs::symlink(source.join("victim"), store.path(Personality::Default)).unwrap();
    assert!(store
        .load(Personality::Default, &[], &AtomicBool::new(false))
        .is_err());
    assert_eq!(
        fs::read_to_string(source.join("victim")).unwrap(),
        "untouched"
    );
    fs::remove_file(store.path(Personality::Default)).unwrap();
    fs::create_dir(store.path(Personality::Default)).unwrap();
    assert!(store
        .load(Personality::Default, &[], &AtomicBool::new(false))
        .is_err());
}

#[test]
fn oversized_regular_settings_are_rejected_before_parsing_and_never_replaced() {
    let owner = tempfile::tempdir().unwrap();
    let store = PersonalityStore::new(owner.path().to_owned());
    let path = store.path(Personality::Sel4);
    fs::File::create(&path)
        .unwrap()
        .set_len(MAX_BYTES + 1)
        .unwrap();
    assert!(store
        .load(Personality::Sel4, &[], &AtomicBool::new(false))
        .unwrap_err()
        .contains("2 MiB"));
    assert_eq!(fs::metadata(path).unwrap().len(), MAX_BYTES + 1);
}

#[test]
fn automatic_allowlists_reject_unknown_and_personality_only_ids_without_losing_parser_literals() {
    for id in super::super::Indexer::AUTOMATIC {
        let settings = parse(
            Personality::Default,
            &default_text().replace("indexers = []", &format!("indexers = ['{}']", id.id())),
        )
        .unwrap();
        assert_eq!(settings.indexers(), [id.id()]);
    }
    let text = default_text().replace(
        "indexers = []",
        "indexers = ['z-missing', 'scip-clang', 'scip-sel4', 'a-missing', 'z-missing']",
    );
    assert_eq!(
        parse(Personality::Default, &text).unwrap_err(),
        "Unknown personality indexers: a-missing, scip-sel4, z-missing."
    );
}
