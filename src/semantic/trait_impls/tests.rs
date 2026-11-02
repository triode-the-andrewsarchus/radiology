use super::*;
use crate::semantic::ScipIndex;

pub(crate) fn fixture() -> (tempfile::TempDir, SourceRead, ScipIndex, Vec<String>) {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path().join("source");
    std::fs::create_dir_all(root.join("src")).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("Tests/RadiologyTests/Fixtures/scip-rust-traits");
    let paths = ["src/lib.rs", "src/implementations.rs"]
        .map(str::to_owned)
        .to_vec();
    for path in &paths {
        std::fs::copy(fixture.join(path), root.join(path)).unwrap();
    }
    let raw = ScipIndex::decode(&std::fs::read(fixture.join("index.scip")).unwrap()).unwrap();
    (owner, SourceRead::Checkout(root), raw, paths)
}

pub(crate) fn imported_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let (owner, source, _, _) = fixture();
    let SourceRead::Checkout(root) = source else {
        unreachable!()
    };
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("Tests/RadiologyTests/Fixtures/scip-rust-traits/index.scip"),
        root.join("index.scip"),
    )
    .unwrap();
    (owner, root)
}

fn build(
    index: &SemanticIndex,
    source: &SourceRead,
    paths: &[String],
    cache: &Path,
) -> TraitImplementationIndex {
    TraitImplementationIndex::build(
        index,
        source,
        Some(paths),
        Some(cache),
        &HashSet::new(),
        &AtomicBool::new(false),
    )
    .unwrap()
}

fn rows<'a>(catalog: &'a TraitImplementationIndex, suffix: &str) -> &'a [TraitImplementation] {
    catalog
        .by_trait
        .iter()
        .find(|(id, _)| id.symbol.ends_with(suffix))
        .unwrap()
        .1
}

fn alias_reference(raw: &mut ScipIndex) -> &mut crate::semantic::ScipOccurrence {
    raw.documents
        .iter_mut()
        .find(|doc| doc.relative_path == "src/implementations.rs")
        .unwrap()
        .occurrences
        .iter_mut()
        .find(|occ| occ.range == crate::semantic::ScipRange::from_packed(&[1, 5, 10]))
        .unwrap()
}

#[test]
fn real_rust_trait_catalog_joins_aliases_qualified_blanket_and_negative_blocks() {
    let (owner, source, mut raw, paths) = fixture();
    assert!(raw
        .documents
        .iter()
        .flat_map(|doc| &doc.symbols)
        .all(|symbol| symbol.relationships.is_empty()));
    let index = SemanticIndex::from_scip(raw.clone());
    let catalog = build(&index, &source, &paths, &owner.path().join("cache"));
    let left = rows(&catalog, "left/Render#");
    assert_eq!(
        left.iter()
            .map(|row| row.target_type.as_str())
            .collect::<Vec<_>>(),
        ["Alpha", "Beta", "Wrapper<T>"]
    );
    assert_eq!(left[0].signature, "impl Alias for Alpha");
    assert!(left[2].signature.contains("where T: Copy"));
    assert!(left.iter().all(|row| !row.is_negative));
    assert_eq!(rows(&catalog, "right/Render#")[0].target_type, "Other");
    assert!(rows(&catalog, "left/Marker#")[0].is_negative);
    assert!(!catalog
        .by_trait
        .keys()
        .any(|id| id.symbol.ends_with("left/Empty#")));
    assert_eq!(catalog.unresolved_count, 0);
    assert!(!catalog.has_gaps());
    assert_eq!(catalog.parsed_document_count, 2);
    assert_eq!(catalog.cached_document_count, 0);
    assert!(catalog.unverified_semantics);
    for document in &mut raw.documents {
        document.occurrences.extend(document.occurrences.clone());
    }
    let duplicate = build(
        &SemanticIndex::from_scip(raw),
        &source,
        &paths,
        &owner.path().join("cache"),
    );
    assert_eq!(rows(&duplicate, "left/Render#").len(), 3);
    let row = &left[0];
    let location = row.resolve(&source, &AtomicBool::new(false)).unwrap();
    assert_eq!(location.path, row.path);
    assert_eq!(location.byte_offset, row.byte_range.start);
    let SourceRead::Checkout(root) = &source else {
        unreachable!()
    };
    let path = root.join(&row.path);
    let original = std::fs::read_to_string(&path).unwrap();
    std::fs::write(path, original.replace("Alpha", "Gamma")).unwrap();
    assert!(row
        .resolve(&source, &AtomicBool::new(false))
        .unwrap_err()
        .contains("changed"));
}

#[test]
fn trait_cache_revalidates_bytes_semantics_roles_encoding_and_corruption() {
    let (owner, source, raw, paths) = fixture();
    let cache = owner.path().join("cache");
    let index = SemanticIndex::from_scip(raw.clone());
    let cold = build(&index, &source, &paths, &cache);
    assert_eq!(cold.parsed_document_count, 2);
    let warm = build(&index, &source, &paths, &cache);
    assert_eq!(
        (warm.parsed_document_count, warm.cached_document_count),
        (0, 2)
    );
    let SourceRead::Checkout(root) = &source else {
        unreachable!()
    };
    let path = root.join("src/implementations.rs");
    let bytes = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, format!("{bytes}\n// changed bytes\n")).unwrap();
    let changed = build(&index, &source, &paths, &cache);
    assert_eq!(
        (changed.parsed_document_count, changed.cached_document_count),
        (1, 1)
    );
    assert!(changed.coverage_detail().contains("freshness is unknown"));
    let mut remapped = raw.clone();
    let right = index
        .symbols
        .keys()
        .find(|id| id.symbol.ends_with("right/Render#"))
        .unwrap()
        .symbol
        .clone();
    let occurrence = alias_reference(&mut remapped);
    occurrence.symbol = right;
    let remapped_index = SemanticIndex::from_scip(remapped.clone());
    let remapped_catalog = build(&remapped_index, &source, &paths, &cache);
    assert_eq!(
        (
            remapped_catalog.parsed_document_count,
            remapped_catalog.cached_document_count
        ),
        (1, 1)
    );
    assert_eq!(
        rows(&remapped_catalog, "right/Render#")
            .iter()
            .map(|row| row.target_type.as_str())
            .collect::<Vec<_>>(),
        ["Alpha", "Other"]
    );
    let occurrence = alias_reference(&mut remapped);
    occurrence.symbol_roles = 1;
    let roles = build(&SemanticIndex::from_scip(remapped), &source, &paths, &cache);
    assert_eq!(roles.parsed_document_count, 1);
    assert_eq!(roles.unresolved_count, 1);
    let mut encoded = raw.clone();
    encoded.text_document_encoding = 3;
    let encoding = build(&SemanticIndex::from_scip(encoded), &source, &paths, &cache);
    assert_eq!(encoding.parsed_document_count, 2);
    std::fs::write(cache.join("rust-trait-implementations.json"), "corrupt").unwrap();
    let recovered = build(&index, &source, &paths, &cache);
    assert_eq!(recovered.parsed_document_count, 2);
    let failed = owner.path().join("not-a-directory");
    std::fs::write(&failed, "file").unwrap();
    assert_eq!(
        build(&index, &source, &paths, &failed).by_trait,
        recovered.by_trait
    );
    let inner = root.join("unsafe-cache");
    let _ = build(&index, &source, &paths, &inner);
    assert!(!inner.exists());
}

#[test]
fn trait_catalog_reports_unresolved_stale_unreadable_excluded_and_unknown_coverage() {
    let (owner, source, raw, mut paths) = fixture();
    let cache = owner.path().join("cache");
    let mut index = SemanticIndex::from_scip(raw.clone());
    paths.push("src/inactive.rs".into());
    let excluded = HashSet::from(["src/implementations.rs".to_owned()]);
    let catalog = TraitImplementationIndex::build(
        &index,
        &source,
        Some(&paths),
        Some(&cache),
        &excluded,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(catalog.by_trait.is_empty());
    assert_eq!(catalog.excluded_paths, ["src/implementations.rs"]);
    assert_eq!(catalog.unindexed_file_count, Some(1));
    assert!(catalog.has_gaps());
    let unknown = TraitImplementationIndex::build(
        &index,
        &source,
        None,
        None,
        &HashSet::new(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(unknown.unindexed_file_count, None);
    assert!(unknown.has_gaps());
    assert!(unknown.coverage_detail().contains("coverage is unknown"));
    for path in &paths {
        index.source_digests.insert(path.clone(), "wrong".into());
    }
    let stale = build(&index, &source, &paths, &cache);
    assert!(stale.by_trait.is_empty());
    assert_eq!(stale.excluded_paths.len(), 2);
    let mut missing = raw;
    missing
        .documents
        .iter_mut()
        .find(|doc| doc.relative_path == "src/implementations.rs")
        .unwrap()
        .occurrences
        .retain(|occ| !occ.symbol.ends_with("left/Render#"));
    let missing = build(&SemanticIndex::from_scip(missing), &source, &paths, &cache);
    assert_eq!(missing.unresolved_count, 3);
    let SourceRead::Checkout(root) = &source else {
        unreachable!()
    };
    std::fs::remove_file(root.join("src/implementations.rs")).unwrap();
    let unreadable = build(
        &SemanticIndex::from_scip(ScipIndex::default()),
        &source,
        &paths,
        &cache,
    );
    assert_eq!(unreadable.unindexed_file_count, Some(3));
    index.source_digests.clear();
    let unreadable = build(&index, &source, &paths, &cache);
    assert_eq!(unreadable.unreadable_paths, ["src/implementations.rs"]);
    assert!(TraitImplementationIndex::build(
        &index,
        &source,
        Some(&paths),
        Some(&cache),
        &HashSet::new(),
        &AtomicBool::new(true)
    )
    .is_err());
}

#[test]
fn rust_impl_structure_excludes_bounds_arguments_and_unexpanded_macro_tokens() {
    let text="trait A<T> {}\ntrait B {}\n// impl A<B> for Pretend {}\nimpl<T: B> A<T> for Wrapper<T> where T: B {}\nimpl<T: A<B>> Wrapper<T> { fn f() {} }\nmacro_rules! generated { () => { impl A<B> for Generated {} } }\nimpl !B for Negative {}\nunsafe impl crate::A<dyn B> for Concrete {}\n";
    let buffer = SourceBuffer::from_str("impls.rs", text);
    let full = crate::syntax::DocumentSyntax::analyze(&buffer);
    let facts =
        crate::syntax::rust_impls::RustTraitImpls::analyze_cancellable(&buffer, || false).unwrap();
    assert!(!facts.has_errors);
    assert_eq!(full.rust_trait_impls.implementations, facts.implementations);
    assert_eq!(
        facts
            .implementations
            .iter()
            .map(|item| item.target_type.as_str())
            .collect::<Vec<_>>(),
        ["Wrapper<T>", "Negative", "Concrete"]
    );
    assert_eq!(
        facts
            .implementations
            .iter()
            .map(|item| &text[item.trait_name_range.clone()])
            .collect::<Vec<_>>(),
        ["A", "B", "A"]
    );
    assert!(facts.implementations[1].is_negative);
    assert!(facts.implementations[2]
        .signature
        .starts_with("unsafe impl"));
    assert!(
        crate::syntax::rust_impls::RustTraitImpls::analyze_cancellable(&buffer, || true).is_err()
    );
}

#[test]
fn trait_identity_requires_one_exact_nonlocal_nondefinition_token_in_every_encoding() {
    use crate::semantic::{ScipDocument, ScipOccurrence, ScipRange};
    let text = "/* café 🦀 */ impl Trait for Type {}\r\n";
    let start = text.find("Trait").unwrap();
    let owner = tempfile::tempdir().unwrap();
    std::fs::write(owner.path().join("a.rs"), text).unwrap();
    let source = SourceRead::Checkout(owner.path().to_owned());
    let paths = vec!["a.rs".to_owned()];
    for encoding in [1, 2, 3, 77] {
        let column = match encoding {
            2 => text[..start].encode_utf16().count(),
            3 => text[..start].chars().count(),
            _ => start,
        } as i32;
        let site = ScipOccurrence {
            symbol: "scip test fixture 1 Trait#".into(),
            range: ScipRange::from_packed(&[0, column, column + 5]),
            ..Default::default()
        };
        let document = ScipDocument {
            relative_path: "a.rs".into(),
            position_encoding: encoding,
            occurrences: vec![site.clone()],
            ..Default::default()
        };
        let catalog = build(
            &SemanticIndex::from_scip(ScipIndex {
                documents: vec![document.clone()],
                ..Default::default()
            }),
            &source,
            &paths,
            &owner.path().join("never"),
        );
        if encoding == 77 {
            assert!(catalog.has_gaps());
            assert!(catalog.by_trait.is_empty());
            continue;
        }
        assert_eq!(
            rows(&catalog, "Trait#")[0].trait_name_range,
            start..start + 5
        );
        for (symbol, definition, overlap) in [
            ("scip test fixture 1 Different#", false, false),
            ("local 0", false, false),
            ("scip test fixture 1 Trait#", true, false),
            ("scip test fixture 1 Trait#", false, true),
        ] {
            let mut changed = document.clone();
            let mut other = site.clone();
            other.symbol = symbol.into();
            other.symbol_roles = i32::from(definition);
            if overlap {
                other.range.as_mut().unwrap().start_col -= 1;
            }
            if symbol.ends_with("Different#") {
                changed.occurrences.push(other);
            } else {
                changed.occurrences = vec![other];
            }
            let rejected = build(
                &SemanticIndex::from_scip(ScipIndex {
                    documents: vec![changed],
                    ..Default::default()
                }),
                &source,
                &paths,
                &owner.path().join("never"),
            );
            assert!(rejected.by_trait.is_empty());
            assert_eq!(rejected.unresolved_count, 1);
        }
    }
}

#[cfg(unix)]
#[test]
fn trait_catalog_cannot_read_a_checkout_symlink_outside_its_root() {
    let (owner, source, raw, paths) = fixture();
    let SourceRead::Checkout(root) = &source else {
        unreachable!()
    };
    let path = root.join("src/implementations.rs");
    let outside = owner.path().join("outside.rs");
    std::fs::rename(&path, &outside).unwrap();
    std::os::unix::fs::symlink(&outside, &path).unwrap();
    let catalog = build(
        &SemanticIndex::from_scip(raw),
        &source,
        &paths,
        &owner.path().join("cache"),
    );
    assert!(catalog.by_trait.is_empty());
    assert_eq!(catalog.unreadable_paths, ["src/implementations.rs"]);
}

#[test]
fn trait_catalog_keeps_valid_blocks_but_marks_parse_errors_and_late_cancel() {
    let (owner, source, raw, paths) = fixture();
    let SourceRead::Checkout(root) = &source else {
        unreachable!()
    };
    let path = root.join("src/implementations.rs");
    let original = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, format!("{original}\nimpl Missing for {{\n")).unwrap();
    let catalog = build(
        &SemanticIndex::from_scip(raw),
        &source,
        &paths,
        &owner.path().join("cache"),
    );
    assert_eq!(rows(&catalog, "left/Render#").len(), 3);
    assert_eq!(catalog.parse_error_paths, ["src/implementations.rs"]);
    assert!(catalog.has_gaps());
    assert!(catalog
        .coverage_detail()
        .contains("Parse or coordinate errors"));
    let source = SourceBuffer::from_str("a.rs", &"impl T for S {}\n".repeat(1000));
    let checks = std::cell::Cell::new(0);
    assert!(
        crate::syntax::rust_impls::RustTraitImpls::analyze_cancellable(&source, || {
            checks.set(checks.get() + 1);
            checks.get() > 5
        })
        .is_err()
    );
    assert!(checks.get() > 5);
}
