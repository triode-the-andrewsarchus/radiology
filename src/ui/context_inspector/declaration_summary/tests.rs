use super::*;
use crate::{
    source::location::ReviewRevision,
    syntax::outline::SymbolExtractor,
    ui::test_support::{click, menu_frame, secondary_click, text_center},
};
use egui::{Event, FullOutput, Key, Modifiers};

struct Summary {
    ctx: egui::Context,
    state: DeclarationSummary,
    buffer: SourceBuffer,
    symbols: Vec<DocumentSymbol>,
    action: Option<InspectorAction>,
}

impl Summary {
    fn new() -> Self {
        let name = format!("λ{}_TAIL", "identifier".repeat(550));
        let text = format!(
            "// exact source\nfn {name}(\n{}\n) {{}}\nfn peer() {{}}\n",
            (0..30)
                .map(|i| format!("    argument{i}: &str,"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let buffer = SourceBuffer::from_str(format!("nested/{}.rs", "filename-".repeat(30)), &text);
        let symbols = SymbolExtractor::extract(&buffer);
        assert_eq!(symbols.len(), 2);
        Self {
            ctx: Default::default(),
            state: Default::default(),
            buffer,
            symbols,
            action: None,
        }
    }

    fn frame(
        &mut self,
        width: f32,
        index: Option<usize>,
        pinned: Option<&str>,
        events: Vec<Event>,
    ) -> FullOutput {
        menu_frame(&self.ctx, egui::vec2(width, 450.0), events, |ui| {
            ui.style_mut().interaction.selectable_labels = false;
            self.action = self.state.show(
                ui,
                &AppTheme::dark(),
                index.map(|i| &self.symbols[i]),
                Some(&self.buffer),
                pinned,
            );
        })
    }

    fn expand(&mut self) -> FullOutput {
        self.frame(230.0, Some(0), None, vec![]);
        // The disclosure is the first focusable control, just as in the native pane.
        self.frame(
            230.0,
            Some(0),
            None,
            vec![crate::ui::commands::test_key(Key::Tab)],
        );
        let output = self.frame(
            230.0,
            Some(0),
            None,
            vec![crate::ui::commands::test_key(Key::Space)],
        );
        assert!(self.state.expanded, "keyboard disclosure did not open");
        output
    }
}

#[test]
fn declaration_signature_is_bounded_but_copy_and_links_keep_exact_source() {
    let mut fixture = Summary::new();
    let closed = fixture.frame(230.0, Some(0), None, vec![]);
    assert!(!fixture.state.expanded);
    assert!(!closed.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.starts_with("fn "))));
    let expanded = fixture.expand();
    let signature = expanded
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text.starts_with("fn ") => Some(text),
            _ => None,
        })
        .expect("expanded signature");
    assert!(signature.galley.rows.len() <= 8);
    assert!(signature.galley.job.text.len() <= 4099);
    assert!(signature.galley.size().x <= 214.01);
    assert!(fixture.symbols[0].detail.as_ref().unwrap().len() > 4096);
    let position = signature.pos + egui::vec2(20.0, 5.0);
    fixture.frame(230.0, Some(0), None, secondary_click(position));
    fixture.frame(230.0, Some(0), None, vec![]);
    let mut copy = crate::ui::commands::test_key(Key::C);
    if let Event::Key { modifiers, .. } = &mut copy {
        *modifiers = Modifiers::CTRL;
    }
    let output = fixture.frame(230.0, Some(0), None, vec![copy]);
    assert_eq!(
        output.platform_output.copied_text,
        fixture.symbols[0].detail.as_ref().unwrap().trim()
    );
    assert!(!crate::ui::context_menu::is_open(&fixture.ctx));
    let output = fixture.frame(230.0, Some(0), None, vec![]);
    let labels: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text)
                if text.galley.job.text.ends_with("_TAIL")
                    || text.galley.job.text.ends_with(".rs:2") =>
            {
                Some(text.galley.job.text.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        labels.len(),
        2,
        "identifier suffix and location must remain legible"
    );
    for label in labels {
        fixture.frame(230.0, Some(0), None, click(text_center(&output, &label)));
        assert_eq!(
            fixture.action,
            Some(InspectorAction::JumpToSymbol(
                fixture.symbols[0].selection_range.start
            ))
        );
    }
}

#[test]
fn disclosure_follows_exact_declaration_and_retires_on_source_or_scope_change() {
    let mut fixture = Summary::new();
    fixture.expand();
    fixture.frame(230.0, Some(0), None, vec![]);
    assert!(fixture.state.expanded);
    fixture.frame(230.0, Some(1), None, vec![]);
    assert!(!fixture.state.expanded);
    fixture.frame(230.0, Some(0), None, vec![]);
    assert!(
        !fixture.state.expanded,
        "returning to an old declaration must stay collapsed"
    );
    fixture
        .ctx
        .memory_mut(|memory| memory.surrender_focus(memory.focused().unwrap()));
    fixture.expand();
    fixture.buffer = SourceBuffer::from_str(
        &fixture.buffer.path,
        &format!("{}// modified\n", fixture.buffer.text),
    );
    fixture.frame(230.0, Some(0), None, vec![]);
    assert!(
        !fixture.state.expanded,
        "equal positions in different source bytes"
    );
    fixture.state.expanded = true;
    fixture.buffer.revision = Some(ReviewRevision::After);
    fixture.frame(230.0, Some(0), None, vec![]);
    assert!(!fixture.state.expanded, "equal bytes in another revision");
    fixture.state.expanded = true;
    fixture.buffer.path = "another.rs".into();
    fixture.frame(230.0, Some(0), None, vec![]);
    assert!(!fixture.state.expanded, "equal bytes at another path");
    for (index, pinned) in [(Some(0), Some("Pinned symbol")), (None, None)] {
        fixture.state.expanded = true;
        fixture.frame(230.0, index, pinned, vec![]);
        assert!(!fixture.state.expanded && fixture.state.owner.is_none());
        fixture.frame(230.0, Some(0), None, vec![]);
        assert!(!fixture.state.expanded);
    }
}
