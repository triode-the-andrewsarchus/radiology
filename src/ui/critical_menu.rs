//! Symbol/package radio groups for the read-only source context menu.

use crate::critical::menu::MarkMenu;
use crate::critical::overrides::{MarkEdit, MarkTarget};
use crate::critical::Criticality;

pub(super) fn show(ui: &mut egui::Ui, menu: &MarkMenu) -> Option<MarkEdit> {
    let mut edit = None;
    ui.separator();
    ui.menu_button(format!("Mark Symbol “{}”", menu.name), |ui| {
        let built_in = format!(
            "Built-In ({:?} · {})",
            menu.built_in.criticality,
            menu.built_in.reason.description()
        );
        edit = choices(
            ui,
            MarkTarget::Symbol(menu.symbol.clone()),
            menu.symbol_decision,
            &built_in,
        );
    });
    if let (Some(name), Some(key)) = (&menu.package_name, &menu.package_key) {
        ui.menu_button(format!("Mark Package “{name}”"), |ui| {
            edit = choices(
                ui,
                MarkTarget::Package(key.clone()),
                menu.package_decision,
                "Built-In",
            )
            .or(edit.take());
        });
    }
    edit
}

fn choices(
    ui: &mut egui::Ui,
    target: MarkTarget,
    current: Option<Criticality>,
    built_in: &str,
) -> Option<MarkEdit> {
    for (label, decision) in [
        ("Critical", Some(Criticality::Critical)),
        ("Non-Critical", Some(Criticality::Ordinary)),
        (built_in, None),
    ] {
        if ui.selectable_label(current == decision, label).clicked() {
            ui.close_menu();
            return Some(MarkEdit { target, decision });
        }
    }
    None
}
