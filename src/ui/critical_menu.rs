//! Symbol/package choices shared by pointer and keyboard context-menu actions.

use super::commands::MenuItem;
use crate::critical::menu::MarkMenu;
use crate::critical::overrides::{MarkEdit, MarkTarget};
use crate::critical::Criticality;

pub(super) fn items(menu: &MarkMenu) -> Vec<MenuItem<MarkEdit>> {
    let built_in = format!(
        "Built-In ({:?} · {})",
        menu.built_in.criticality,
        menu.built_in.reason.description()
    );
    let mut items = vec![
        MenuItem::separator(),
        MenuItem::submenu(
            format!("Mark Symbol “{}”", menu.name),
            choices(
                MarkTarget::Symbol(menu.symbol.clone()),
                menu.symbol_decision,
                &built_in,
            ),
        ),
    ];
    if let (Some(name), Some(key)) = (&menu.package_name, &menu.package_key) {
        items.push(MenuItem::submenu(
            format!("Mark Package “{name}”"),
            choices(
                MarkTarget::Package(key.clone()),
                menu.package_decision,
                "Built-In",
            ),
        ));
    }
    items
}

fn choices(
    target: MarkTarget,
    current: Option<Criticality>,
    built_in: &str,
) -> Vec<MenuItem<MarkEdit>> {
    [
        ("Critical", Some(Criticality::Critical)),
        ("Non-Critical", Some(Criticality::Ordinary)),
        (built_in, None),
    ]
    .into_iter()
    .map(|(label, decision)| {
        MenuItem::action(
            label,
            MarkEdit {
                target: target.clone(),
                decision,
            },
        )
        .checked(current == decision)
    })
    .collect()
}
