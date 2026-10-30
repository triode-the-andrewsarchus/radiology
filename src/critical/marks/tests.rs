use super::*;
use crate::critical::Criticality;

fn edit(value: Option<Criticality>) -> MarkEdit {
    MarkEdit {
        target: MarkTarget::Symbol("rust-analyzer cargo domain 1 f().".into()),
        decision: value,
    }
}

fn settle(session: &mut MarkSession) {
    let started = Instant::now();
    while session.is_busy() {
        session.poll();
        assert!(started.elapsed() < Duration::from_secs(5));
        std::thread::sleep(Duration::from_millis(5));
    }
    session.poll();
}

#[test]
fn edits_before_load_and_immediate_switch_survive_reopening_without_losing_other_marks() {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path().join("repo");
    let other = owner.path().join("other");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(&other).unwrap();
    let store = OverrideStore::new(owner.path().join("settings"));
    let mut initial = CriticalityOverrides::default();
    initial.set_package("rust-analyzer cargo core", Some(Criticality::Ordinary));
    store
        .save(&root, &initial, store.reserve_revision())
        .unwrap();
    let mut session = MarkSession::new(Some(store.clone()));
    session.open(root.clone());
    assert!(session.edit(edit(Some(Criticality::Critical))));
    session.open(other.clone()); // Load was not polled; its durable edit must not be retired.
    settle(&mut session);
    assert!(session.overrides().is_empty());
    session.open(root.clone());
    settle(&mut session);
    assert_eq!(session.overrides().symbols.len(), 1);
    assert_eq!(session.overrides().packages, initial.packages);
    session.close();
    session.open(root.clone());
    session.edit(edit(None)); // A reset is meaningful even if the pre-load set is empty.
    settle(&mut session);
    assert!(session.overrides().symbols.is_empty());
    assert_eq!(
        store.load(&root, &AtomicBool::new(false)).unwrap().packages,
        initial.packages
    );
}

#[test]
fn rapid_choices_coalesce_and_shutdown_flush_keeps_the_last_choice() {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    let store = OverrideStore::new(owner.path().join("settings"));
    let mut session = MarkSession::new(Some(store.clone()));
    session.open(root.clone());
    settle(&mut session);
    for number in 0..100 {
        session.edit(edit(Some(if number % 2 == 0 {
            Criticality::Ordinary
        } else {
            Criticality::Critical
        })));
    }
    session.flush(Duration::from_secs(5)).unwrap();
    let loaded = store.load(&root, &AtomicBool::new(false)).unwrap();
    assert_eq!(loaded.symbols.values().next(), Some(&Criticality::Critical));
    assert!(!session.is_busy());
    let mut reopened = MarkSession::new(Some(store));
    reopened.open(root);
    settle(&mut reopened);
    assert_eq!(reopened.overrides().symbols, loaded.symbols);
}

#[test]
fn save_failure_keeps_session_choices_and_does_not_leak_to_another_root() {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path().join("repo");
    let other = owner.path().join("other");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(&other).unwrap();
    let blocked = owner.path().join("settings");
    std::fs::write(&blocked, "blocked").unwrap();
    let mut session = MarkSession::new(Some(OverrideStore::new(blocked)));
    session.open(root);
    settle(&mut session);
    session.edit(edit(Some(Criticality::Critical)));
    settle(&mut session);
    assert_eq!(session.overrides().symbols.len(), 1);
    assert!(session.error().is_some());
    session.open(other);
    settle(&mut session);
    assert!(session.overrides().is_empty());
}

#[test]
fn dropping_a_session_retains_accepted_edits_even_before_load_publication() {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    let store = OverrideStore::new(owner.path().join("settings"));
    let mut session = MarkSession::new(Some(store.clone()));
    session.open(root.clone());
    session.edit(edit(Some(Criticality::Critical)));
    let writer = session.writer.clone();
    drop(session);
    writer.flush(Duration::from_secs(5)).unwrap();
    assert_eq!(
        store
            .load(&root, &AtomicBool::new(false))
            .unwrap()
            .symbols
            .len(),
        1
    );
}
