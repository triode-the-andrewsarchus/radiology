use super::*;
use crate::critical::worker::tests::fixture;

fn ready(hover: &mut CriticalHover, byte: usize, now: f64) {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while matches!(
        hover.presentation().map(|shown| &shown.phase),
        Some(HoverPhase::Loading)
    ) {
        hover.update(Some(byte), false, false, now);
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn dwell_tracks_the_exact_occurrence_and_background_completion_keeps_popup_identity() {
    let (_dir, inputs) = fixture();
    let byte = inputs.buffer.text.find("helper").unwrap();
    let inputs = Arc::new(inputs);
    let mut hover = CriticalHover::default();
    hover.configure(Some(inputs.clone()));
    hover.update(Some(byte), false, false, 0.0);
    assert!(hover.presentation().is_none());
    assert_eq!(hover.next_wake(0.0), Some(Duration::from_millis(350)));
    // Pointer movement inside the same occurrence does not restart the dwell.
    hover.update(Some(byte + 1), false, false, 0.3);
    hover.update(Some(byte + 1), false, false, 0.351);
    let shown = hover.presentation().unwrap();
    let serial = shown.serial;
    assert!(matches!(shown.phase, HoverPhase::Loading));
    assert_eq!(shown.target.range, byte..byte + 6);
    ready(&mut hover, byte + 1, 0.4);
    let shown = hover.presentation().unwrap();
    assert_eq!(shown.serial, serial);
    assert!(
        matches!(&shown.phase, HoverPhase::Ready(frontier) if frontier.annotation().label() == "1")
    );
    hover.configure(Some(inputs));
    assert_eq!(hover.presentation().unwrap().serial, serial);
    assert!(hover.next_wake(0.4).is_none());
}

#[test]
fn popup_entry_and_anchor_return_cancel_grace_and_interruptions_suppress_idle_reopening() {
    let (_dir, inputs) = fixture();
    let byte = inputs.buffer.text.find("helper").unwrap();
    let mut hover = CriticalHover::default();
    hover.configure(Some(Arc::new(inputs)));
    hover.update(Some(byte), false, false, 0.0);
    hover.update(Some(byte), false, false, 0.4);
    ready(&mut hover, byte, 0.4);
    hover.update(None, false, false, 0.5);
    hover.update(None, true, false, 1.1);
    hover.update(None, true, false, 2.0);
    assert!(hover.presentation().is_some());
    hover.update(None, false, false, 2.1);
    hover.update(Some(byte), false, false, 2.7);
    assert!(hover.presentation().is_some());
    hover.update(Some(byte), false, true, 3.0);
    hover.update(Some(byte), false, false, 4.0);
    assert!(hover.presentation().is_none());
    assert!(hover.next_wake(4.0).is_none());
    hover.update(None, false, false, 4.1);
    hover.update(Some(byte), false, false, 4.2);
    hover.update(Some(byte), false, false, 4.6);
    hover.update(None, false, false, 4.7);
    hover.update(None, false, false, 5.401);
    assert!(hover.presentation().is_none());
}

#[test]
fn leaving_before_dwell_and_reconfiguration_retire_pending_and_loaded_presentations() {
    let (_dir, inputs) = fixture();
    let byte = inputs.buffer.text.find("helper").unwrap();
    let inputs = Arc::new(inputs);
    let mut hover = CriticalHover::default();
    hover.configure(Some(inputs.clone()));
    hover.update(Some(byte), false, false, 0.0);
    hover.update(None, false, false, 0.2);
    hover.update(None, false, false, 0.5);
    assert!(hover.presentation().is_none());
    hover.update(Some(byte), false, false, 0.6);
    hover.update(Some(byte), false, false, 1.0);
    hover.configure(Some(Arc::new((*inputs).clone())));
    assert!(hover.presentation().is_none());
    hover.update(None, false, false, 2.0);
    assert!(hover.presentation().is_none());
    hover.configure(None);
    hover.update(Some(byte), false, false, 3.0);
    hover.update(Some(byte), false, false, 4.0);
    assert!(hover.presentation().is_none());
}

#[test]
fn external_critical_is_honest_while_plumbing_locals_stale_and_before_do_not_open() {
    let (_dir, inputs) = fixture();
    let byte = inputs.buffer.text.find("helper").unwrap();
    for (symbol, expected) in [
        ("rust-analyzer cargo std 0 fs/open().", true),
        ("rust-analyzer cargo core 0 clone().", false),
        ("rust-analyzer cargo thirdparty 0 unknown().", false),
        ("local 0", false),
    ] {
        let mut next = inputs.clone();
        Arc::make_mut(&mut next.index)
            .documents
            .get_mut("a.rs")
            .unwrap()
            .occurrences[1]
            .symbol = symbol.into();
        let mut hover = CriticalHover::default();
        hover.configure(Some(Arc::new(next)));
        hover.update(Some(byte), false, false, 0.0);
        hover.update(Some(byte), false, false, 0.4);
        assert_eq!(hover.presentation().is_some(), expected, "{symbol}");
        if expected {
            assert!(hover.presentation().unwrap().target.definition.is_none());
            assert!(
                matches!(&hover.presentation().unwrap().phase,HoverPhase::Ready(frontier) if frontier.coverage == CallCoverage::NoBody(NoBodyReason::External))
            );
        }
    }
    let mut stale = inputs.clone();
    Arc::make_mut(&mut stale.index)
        .source_digests
        .insert("a.rs".into(), "older".into());
    let mut before = inputs;
    Arc::make_mut(&mut before.buffer).revision = Some(ReviewRevision::Before);
    for next in [stale, before] {
        let mut hover = CriticalHover::default();
        hover.configure(Some(Arc::new(next)));
        hover.update(Some(byte), false, false, 0.0);
        hover.update(Some(byte), false, false, 0.4);
        assert!(hover.presentation().is_none());
    }
}
