//! A source-bound landing cue whose clock starts when the reader paints it.

use crate::source::{ReviewRevision, SourceBuffer};
use std::time::Duration;

const HOLD: f64 = 0.9;
const FADE: f64 = 0.21;

#[derive(Default)]
pub(super) struct DestinationFeedback {
    target: Option<Target>,
}

struct Target {
    document: (String, Option<ReviewRevision>, String),
    byte: usize,
    started: Option<f64>,
}

#[derive(Clone, Copy)]
pub(super) struct FeedbackFrame {
    pub byte: usize,
    pub alpha: f32,
    pub next_frame: Duration,
}

impl DestinationFeedback {
    pub fn request(&mut self, buffer: &SourceBuffer, byte: usize) {
        self.target = Some(Target {
            document: (
                buffer.path.clone(),
                buffer.revision,
                buffer.content_digest().to_owned(),
            ),
            byte: buffer.clamp_to_char_boundary(byte.min(buffer.byte_len())),
            started: None,
        });
    }

    pub fn cancel(&mut self) {
        self.target = None;
    }

    pub fn frame(
        &mut self,
        buffer: &SourceBuffer,
        now: f64,
        animate: bool,
    ) -> Option<FeedbackFrame> {
        let target = self.target.as_mut()?;
        if target.document.0 != buffer.path
            || target.document.1 != buffer.revision
            || target.document.2 != buffer.content_digest()
            || !now.is_finite()
        {
            self.cancel();
            return None;
        }
        let elapsed = (now - *target.started.get_or_insert(now)).max(0.0);
        let remaining = HOLD + if animate { FADE } else { 0.0 } - elapsed;
        if remaining <= 0.0 {
            self.cancel();
            return None;
        }
        let strength = ((HOLD + FADE - elapsed) / FADE).clamp(0.0, 1.0);
        Some(FeedbackFrame {
            byte: target.byte,
            alpha: (0.18 * strength) as f32,
            next_frame: Duration::from_secs_f64(if elapsed < HOLD {
                HOLD - elapsed
            } else {
                remaining.min(1.0 / 60.0)
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::ReviewRevision;

    #[test]
    fn hold_fade_restart_and_animation_disabled_use_the_first_paint_clock() {
        let source = SourceBuffer::from_str("a.rs", "café 😀\n");
        let mut feedback = DestinationFeedback::default();
        feedback.request(&source, 7); // middle of the emoji is clamped to its start
        let first = feedback.frame(&source, 1000.0, true).unwrap();
        assert_eq!(first.byte, 6);
        assert_eq!(first.alpha, 0.18);
        assert_eq!(first.next_frame, Duration::from_millis(900));
        assert_eq!(feedback.frame(&source, 1000.8, true).unwrap().alpha, 0.18);
        let fading = feedback.frame(&source, 1001.005, true).unwrap();
        assert!((fading.alpha - 0.09).abs() < 0.0001);
        assert!(fading.next_frame <= Duration::from_secs_f64(1.0 / 60.0));
        feedback.request(&source, source.byte_len());
        assert_eq!(feedback.frame(&source, 1002.0, true).unwrap().alpha, 0.18);
        assert!(feedback.frame(&source, 1003.2, true).is_none());
        feedback.request(&source, 0);
        feedback.frame(&source, 2000.0, false);
        assert_eq!(feedback.frame(&source, 2000.8, false).unwrap().alpha, 0.18);
        assert!(feedback.frame(&source, 2001.0, false).is_none());
    }

    #[test]
    fn mismatched_path_bytes_revision_and_cancel_retire_the_cue() {
        let source = SourceBuffer::from_str("a.rs", "fn start() {}\n");
        let mut feedback = DestinationFeedback::default();
        for other in [
            SourceBuffer::from_str("b.rs", &source.text),
            SourceBuffer::from_str("a.rs", "fn changed() {}\n"),
            SourceBuffer::from_bytes("a.rs", source.data.clone(), Some(ReviewRevision::Before))
                .unwrap(),
        ] {
            feedback.request(&source, 3);
            assert!(feedback.frame(&other, 1.0, true).is_none());
            assert!(feedback.frame(&source, 1.0, true).is_none());
        }
        let before =
            SourceBuffer::from_bytes("a.rs", source.data.clone(), Some(ReviewRevision::Before))
                .unwrap();
        let colon_path = SourceBuffer::from_str("before:a.rs", &source.text);
        feedback.request(&before, 3);
        assert!(feedback.frame(&colon_path, 1.0, true).is_none());
        feedback.request(&source, 3);
        feedback.cancel();
        assert!(feedback.frame(&source, 1.0, true).is_none());
    }
}
