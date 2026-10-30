use super::*;
use crate::{
    pull_request::diff::{DiffHunk, FileLineChanges},
    source::{ReviewRevision, SourceBuffer},
};
use std::collections::HashSet;

fn defaults(
    path: &str,
    text: &str,
    revision: Option<ReviewRevision>,
    changes: Option<&FileLineChanges>,
) -> (SourceBuffer, DocumentSyntax, FoldStore) {
    let buffer = SourceBuffer::from_bytes(path, text.as_bytes().to_vec(), revision).unwrap();
    let syntax = DocumentSyntax::analyze(&buffer);
    assert!(syntax.calls.supported, "{:?}", syntax.calls.error);
    let mut store = FoldStore::new();
    store.activate(&buffer, &syntax.folds);
    store.apply_initial_defaults(&buffer, &syntax.folds, &syntax.calls, changes);
    (buffer, syntax, store)
}

#[test]
fn fresh_callable_defaults_leave_containers_open_and_remember_expanded_choices() {
    let text = "struct Demo {\n value: i32,\n}\nimpl Demo {\n fn first() {\n one();\n }\n fn second() {\n two();\n }\n}\nfn outside() {\n three();\n}\n";
    let (buffer, syntax, mut store) = defaults("a.rs", text, None, None);
    assert_eq!(store.folded_lines, HashSet::from([5, 8, 12]));
    assert!(!store.is_folded(1));
    assert!(!store.is_folded(4));
    store.unfold_all();
    store.apply_initial_defaults(&buffer, &syntax.folds, &syntax.calls, None);
    assert!(store.folded_lines.is_empty());
    let other = SourceBuffer::from_str("b.rs", "fn other() {\n four();\n}\n");
    let other_syntax = DocumentSyntax::analyze(&other);
    store.activate(&other, &other_syntax.folds);
    store.apply_initial_defaults(&other, &other_syntax.folds, &other_syntax.calls, None);
    assert!(store.is_folded(1));
    store.activate(&buffer, &syntax.folds);
    store.apply_initial_defaults(&buffer, &syntax.folds, &syntax.calls, None);
    assert!(
        store.folded_lines.is_empty(),
        "an expanded saved document is initialized, not fresh"
    );
    store.toggle(1);
    store.collapse_all_functions(&buffer, &syntax.folds, &syntax.calls);
    assert_eq!(store.folded_lines, HashSet::from([1, 5, 8, 12]));
}

#[test]
fn abstract_multiline_signatures_are_not_treated_as_callable_bodies() {
    let (_,syntax,store) = defaults("a.swift","protocol P {\n func required(\n value: Int\n ) -> Int\n}\nfunc implemented() {\n doWork()\n}\n",None,None);
    assert!(syntax.calls.scopes.iter().any(|scope| !scope.has_body()));
    assert!(!store.is_folded(2));
    assert!(!store.is_folded(1));
    assert!(store.is_folded(6));
}

#[test]
fn pr_annotations_and_body_changes_expand_only_their_owners_and_ancestors_once() {
    let text="impl Demo {\n #[changed]\n fn first(&self) {\n one();\n }\n fn second(&self) {\n two();\n }\n}\nfn sibling() {\n three();\n}\n";
    let changes = FileLineChanges {
        hunks: vec![DiffHunk {
            before_start: 7,
            before_count: 1,
            after_start: 2,
            after_count: 1,
        }],
        ..Default::default()
    };
    let (after, syntax, mut store) =
        defaults("a.rs", text, Some(ReviewRevision::After), Some(&changes));
    assert_eq!(store.folded_lines, HashSet::from([6, 10]));
    let selected = super::review_folds::expanded_headers(
        &after,
        &syntax.folds,
        &syntax.calls,
        &changes,
        ReviewRevision::After,
    );
    assert!(selected.contains(&1), "container ancestor remains visible");
    assert!(
        selected.contains(&3),
        "Rust attribute owns its following function body"
    );
    store.toggle(3);
    store.apply_initial_defaults(&after, &syntax.folds, &syntax.calls, Some(&changes));
    assert!(
        store.is_folded(3),
        "initial changes cannot override a later manual fold"
    );
    let (_, _, before) = defaults("a.rs", text, Some(ReviewRevision::Before), Some(&changes));
    assert_eq!(before.folded_lines, HashSet::from([3, 10]));
    let (_, _, checkout) = defaults("a.rs", text, None, Some(&changes));
    assert_eq!(
        checkout.folded_lines,
        HashSet::from([3, 6, 10]),
        "checkout washes do not apply PR folding policy"
    );
}

#[test]
fn python_decorator_and_rust_attributes_belong_only_to_their_declarations() {
    for (path,text,changed,opened,closed) in [
        ("a.py","@decorator\ndef first():\n    one()\n    more()\ndef second():\n    two()\n    more()\n",1,3,6),
        ("a.rs","#[one]\n// attached comment\n#[two]\nfn first() {\n one();\n}\nfn second() {\n two();\n}\n",1,4,7),
    ] {
        let changes=FileLineChanges { hunks:vec![DiffHunk { before_start:changed,before_count:1,after_start:changed,after_count:1 }], ..Default::default() };
        let (buffer,syntax,store)=defaults(path,text,Some(ReviewRevision::After),Some(&changes));
        assert!(!store.is_folded(opened),"{path}");
        assert!(store.is_folded(closed),"{path}");
        let owner=syntax.calls.scopes.iter().find(|scope| scope.body_range.as_ref().is_some_and(|body| buffer.line_at_byte(body.start)==opened)).unwrap();
        assert_eq!(buffer.line_at_byte(owner.annotated_range.start),changed);
    }
}

#[test]
fn empty_side_anchors_do_not_open_closed_neighbors_but_open_indentation_eof_is_owned() {
    let rust = "fn first() {\n one();\n}\nfn second() {\n two();\n}";
    for (anchor, expected) in [
        (0, HashSet::from([1, 4])),
        (1, HashSet::from([4])),
        (3, HashSet::from([1, 4])),
        (6, HashSet::from([1, 4])),
    ] {
        let changes = FileLineChanges {
            hunks: vec![DiffHunk {
                before_start: 1,
                before_count: 1,
                after_start: anchor,
                after_count: 0,
            }],
            ..Default::default()
        };
        let (_, _, store) = defaults("a.rs", rust, Some(ReviewRevision::After), Some(&changes));
        assert_eq!(
            store.folded_lines, expected,
            "empty side after line {anchor}"
        );
    }
    for (text, folded) in [
        ("def f():\n    one()\n    two()", false),
        ("def f():\n    one()\n    two()\n", true),
    ] {
        let changes = FileLineChanges {
            hunks: vec![DiffHunk {
                before_start: 1,
                before_count: 1,
                after_start: 3,
                after_count: 0,
            }],
            ..Default::default()
        };
        let (_, _, store) = defaults("a.py", text, Some(ReviewRevision::After), Some(&changes));
        assert_eq!(store.is_folded(2), folded);
    }
}

#[test]
fn changed_test_functions_stay_hidden_by_the_independent_test_presentation() {
    let text =
        "fn production() {}\n#[cfg(test)]\nmod tests {\n fn changed() {\n  assertion();\n }\n}\n";
    let changes = FileLineChanges {
        hunks: vec![DiffHunk {
            before_start: 5,
            before_count: 1,
            after_start: 5,
            after_count: 1,
        }],
        ..Default::default()
    };
    let (buffer, syntax, store) =
        defaults("a.rs", text, Some(ReviewRevision::After), Some(&changes));
    assert!(!store.is_folded(4));
    let hidden = text.find("fn changed").unwrap();
    let mut tests = crate::source::test_code::TestCodePresentation::default();
    tests.load(&buffer, syntax.tests, true);
    assert!(tests.hides_declaration(&(hidden..hidden + "fn changed".len())));
}

#[test]
fn multiline_signatures_stay_visible_and_same_line_containers_are_not_function_defaults() {
    let text = "fn multiline(\n value: i32,\n) -> i32\n{\n let result = value;\n result\n}\nfn sibling() {\n other();\n}\n";
    let (_, syntax, store) = defaults("a.rs", text, None, None);
    assert_eq!(store.folded_lines, HashSet::from([4, 8]));
    assert!(syntax.folds.iter().all(|fold| fold.start_line != 1));
    let changes = FileLineChanges {
        hunks: vec![DiffHunk {
            before_start: 2,
            before_count: 1,
            after_start: 2,
            after_count: 1,
        }],
        ..Default::default()
    };
    let (_, _, review) = defaults("a.rs", text, Some(ReviewRevision::After), Some(&changes));
    assert_eq!(review.folded_lines, HashSet::from([8]));
    let (_, _, container) = defaults(
        "a.rs",
        "impl Demo { fn first() {\n one();\n }\n fn second() {\n two();\n }\n}\n",
        None,
        None,
    );
    assert_eq!(container.folded_lines, HashSet::from([4]));
}
