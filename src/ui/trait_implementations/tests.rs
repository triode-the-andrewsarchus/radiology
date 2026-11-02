use super::*;
use crate::semantic::SemanticIndex;
use crate::ui::test_support::{painted_frame, text_shapes};
use std::sync::{atomic::AtomicBool, Arc};

fn catalog() -> (
    tempfile::TempDir,
    SemanticIndex,
    crate::semantic::index::SemanticSymbolID,
) {
    let (owner, source, raw, paths) = crate::semantic::trait_impls::tests::fixture();
    let mut index = SemanticIndex::from_scip(raw);
    let catalog = crate::semantic::trait_impls::TraitImplementationIndex::build(
        &index,
        &source,
        Some(&paths),
        None,
        &Default::default(),
        &AtomicBool::new(false),
    )
    .unwrap();
    let target = catalog
        .by_trait
        .keys()
        .find(|id| id.symbol.ends_with("left/Render#"))
        .unwrap()
        .clone();
    index.trait_implementations = Arc::new(catalog);
    (owner, index, target)
}

#[test]
fn compact_trait_rows_fit_narrow_panes_and_virtualize_large_lists() {
    for scale in [1.0, 2.0] {
        for width in [220.0, 280.0, 420.0] {
            for theme in [AppTheme::dark(), AppTheme::light()] {
                let (_owner, mut index, target) = catalog();
                let catalog = Arc::make_mut(&mut index.trait_implementations);
                let template = catalog.by_trait[&target][0].clone();
                catalog.by_trait.insert(
                    target.clone(),
                    (0..1000)
                        .map(|position| {
                            let mut row = template.clone();
                            row.target_type =
                                format!("Implementation{position:04}<LongGenericType>");
                            row.line = position + 1;
                            row.is_negative = position % 2 == 0;
                            row
                        })
                        .collect(),
                );
                let mut context = TraitImplementationContext::default();
                context.update(None, Some(Arc::new(index)), Some(target));
                context.select(Some(999));
                let mut list = TraitImplementationList::default();
                let ctx = egui::Context::default();
                ctx.set_pixels_per_point(scale);
                ctx.set_visuals(if theme.is_dark {
                    egui::Visuals::dark()
                } else {
                    egui::Visuals::light()
                });
                let size = egui::vec2(width, 220.0);
                painted_frame(&ctx, size, vec![], |ui| {
                    list.show(ui, &theme, &context);
                });
                list.reveal_selection();
                let output = painted_frame(&ctx, size, vec![], |ui| {
                    list.show(ui, &theme, &context);
                });
                let rows: Vec<_> = text_shapes(&output)
                    .filter(|shape| shape.galley.job.text.starts_with("Implementation"))
                    .collect();
                assert!(
                    rows.len() < 20 && !rows.is_empty(),
                    "rendered {} rows",
                    rows.len()
                );
                assert!(
                    rows.iter()
                        .any(|shape| shape.galley.job.text.contains("0999")),
                    "selection not revealed"
                );
                for row in rows {
                    assert!(row.galley.size().y <= 24.0);
                    assert!(
                        row.pos.x + row.galley.size().x <= width - 3.0,
                        "row exceeded pane bounds"
                    );
                }
                assert!(text_shapes(&output)
                    .any(|shape| shape.galley.job.text == "Trait Implementations"));
            }
        }
    }
}
