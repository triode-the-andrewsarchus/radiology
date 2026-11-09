use super::focus::{frame, settle_frame};
use super::review_refresh::open_review;
use super::*;

fn press(app: &mut RadiologyApp, ctx: &Context, label: &str) {
    let output = frame(app, ctx, vec![]);
    let position = crate::ui::test_support::text_center(&output, label);
    frame(app, ctx, crate::ui::test_support::click(position));
    settle_frame(app, ctx);
}

#[test]
fn choose_dismiss_exit_and_root_switch_retire_only_the_failed_review_intent() {
    use crate::ui::app::reviews::recovery::RecoveryAction;
    let (_owner, root, review) = crate::pull_request::review::tests::fixture();
    for scale in [1.0, 2.0] {
        for transition in ["choose", "dismiss", "exit", "root"] {
            let (_settings, mut app) = open_review(&root, review.clone(), ReviewRevision::After);
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            let buffer = app.open_buffer.clone().unwrap();
            app.pending_pr = Some(8);
            app.review_job.spawn(|_| Err("offline".into()));
            settle(&mut app);
            assert!(app.review_error().unwrap().contains("offline"));
            let output = frame(&mut app, &ctx, vec![]);
            for label in ["Try Again", "Choose Number…"] {
                crate::ui::test_support::assert_visible_text(&output, label);
            }
            assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
            assert_eq!(app.active_pr.as_ref().unwrap().number, 7);
            assert!(app.error.is_none(), "review failure has a separate owner");
            let other = tempfile::tempdir().unwrap();
            match transition {
                "choose" => {
                    press(&mut app, &ctx, "Choose Number…");
                    assert!(
                        matches!(&app.modal, ActiveModal::PullRequest(prompt) if prompt.pr_str == "8")
                    );
                    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
                }
                "dismiss" => {
                    app.error = Some("independent reader error".into());
                    assert!(app.review_error().unwrap().contains("offline"));
                    app.error = None;
                    press(&mut app, &ctx, "Dismiss");
                    assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
                }
                "exit" => app.execute_command(&ctx, ToolbarAction::ExitPRReview),
                _ => app.open_repository(other.path()),
            }
            settle(&mut app);
            assert!(app.review_error().is_none());
            app.recover_review(RecoveryAction::Retry); // A stale action cannot restart work.
            assert!(app.pending_pr.is_none());
            assert!(!app.review_job.is_running());
            if transition == "root" {
                assert_eq!(app.repository_root.as_deref(), Some(other.path()));
            }
            if matches!(transition, "exit" | "root") {
                assert!(app.review.is_none());
            }
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn real_cli_recovery_retries_exact_failed_targets_and_source_preparation_failures() {
    const MARKER: &str = "RADIOLOGY_REVIEW_RECOVERY_CHILD";
    if let Some(directory) = std::env::var_os(MARKER) {
        run_cli_recovery(Path::new(&directory));
        return;
    }
    let owner = tempfile::tempdir().unwrap();
    let tools = owner.path().join("bin");
    std::fs::create_dir(&tools).unwrap();
    let gh = crate::repository::test_support::fake_git(
        &tools,
        r#"
case "$*" in
  'pr list '*) printf '[]'; exit 0 ;;
esac
printf '%s\n' "$*" >> "$RADIOLOGY_REVIEW_RECOVERY_CHILD/requests"
if [ -f "$RADIOLOGY_REVIEW_RECOVERY_CHILD/fail" ]; then
  cat "$RADIOLOGY_REVIEW_RECOVERY_CHILD/fail" >&2; exit 1
fi
cat "$RADIOLOGY_REVIEW_RECOVERY_CHILD/summary.json"
"#,
    );
    std::fs::rename(gh, tools.join("gh")).unwrap();
    assert_isolated_test(
        "ui::app::tests::review_recovery::real_cli_recovery_retries_exact_failed_targets_and_source_preparation_failures",
        &[
            (MARKER, owner.path().as_os_str().to_owned()),
            ("PATH", format!("{}:/usr/bin:/bin", tools.display()).into()),
            ("XDG_CACHE_HOME", owner.path().join("cache").into_os_string()),
            ("XDG_CONFIG_HOME", owner.path().join("config").into_os_string()),
        ],
    );
}

#[cfg(target_os = "linux")]
fn run_cli_recovery(directory: &Path) {
    let (owner, root, review) = crate::pull_request::review::tests::fixture();
    let respond = |summary: &PullRequestSummary, error: Option<&str>| {
        std::fs::write(
            directory.join("summary.json"),
            serde_json::to_vec(summary).unwrap(),
        )
        .unwrap();
        if let Some(error) = error {
            std::fs::write(directory.join("fail"), error).unwrap();
        } else if directory.join("fail").exists() {
            std::fs::remove_file(directory.join("fail")).unwrap();
        }
    };
    let assert_requests = |number: usize, count: usize| {
        let requests = std::fs::read_to_string(directory.join("requests")).unwrap();
        assert_eq!(requests.lines().count(), count, "{requests}");
        assert!(
            requests
                .lines()
                .all(|line| line.starts_with(&format!("pr view {number} "))),
            "{requests}"
        );
    };
    let status_frame =
        |app: &mut RadiologyApp, ctx: &Context, width: f32, events: Vec<egui::Event>| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 800.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| app.show_review_status(ctx),
            )
        };
    for scale in [1.0, 2.0] {
        for requested in [None, Some(7), Some(8)] {
            let (_settings, mut app) = if requested.is_none() {
                let (settings, mut app) = reader();
                app.open_repository(&root);
                settle(&mut app);
                app.open_file("a file.rs");
                settle(&mut app);
                (settings, app)
            } else {
                open_review(&root, review.clone(), ReviewRevision::After)
            };
            let number = requested.unwrap_or(8);
            let mut summary = review.summary.clone();
            summary.number = number;
            respond(&summary, Some("offline fixture"));
            std::fs::write(directory.join("requests"), "").unwrap();
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            let buffer = app.open_buffer.clone().unwrap();
            let history = app.history.current().cloned();
            app.open_pull_request(number);
            settle(&mut app);
            assert!(app.review_error().unwrap().contains("offline fixture"));
            assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
            assert_eq!(app.history.current(), history.as_ref());
            respond(&summary, None);
            press(&mut app, &ctx, "Try Again");
            settle(&mut app);
            assert!(app.review_error().is_none());
            assert!(app.error.is_none(), "{:?}", app.error);
            assert_eq!(app.active_pr.as_ref().unwrap().number, number);
            assert_eq!(app.review.as_ref().unwrap().head, review.head);
            if requested != Some(7) {
                assert!(app.open_buffer.is_none() && app.history.current().is_none());
                app.open_file("a file.rs");
                settle(&mut app);
            }
            assert_eq!(
                app.open_buffer.as_ref().unwrap().text,
                "one\nchanged\nthree\n"
            );
            if requested == Some(7) {
                assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
            }
            assert_requests(number, 2);
        }
        // Snapshot preparation succeeds, but its binary source restoration fails.
        let (_settings, mut app) = open_review(&root, review.clone(), ReviewRevision::After);
        let unreadable = super::review_refresh::advance(&root, owner.path(), &review, |root| {
            std::fs::write(root.join("a file.rs"), [0, 1]).unwrap();
        });
        respond(&unreadable.summary, None);
        std::fs::write(directory.join("requests"), "").unwrap();
        let buffer = app.open_buffer.clone().unwrap();
        let tree = review.head_root.clone();
        app.open_pull_request(7);
        settle(&mut app);
        assert!(app
            .review_error()
            .unwrap()
            .contains("Unable to refresh review"));
        assert!(Arc::ptr_eq(app.open_buffer.as_ref().unwrap(), &buffer));
        assert!(tree.exists());
        let fresh = super::review_refresh::advance(&root, owner.path(), &unreadable, |root| {
            std::fs::write(root.join("a file.rs"), "one\nrecovered café\nthree\n").unwrap();
        });
        respond(&fresh.summary, None);
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        press(&mut app, &ctx, "Try Again");
        settle(&mut app);
        assert!(app.review_error().is_none());
        assert_eq!(app.review.as_ref().unwrap().head, fresh.head);
        assert_eq!(
            app.open_buffer.as_ref().unwrap().text,
            "one\nrecovered café\nthree\n"
        );
        assert!(!app.history.can_go_back());
        assert_requests(7, 2);

        let (_settings, mut app) = open_review(&root, review.clone(), ReviewRevision::After);
        let mut summary = review.summary.clone();
        summary.number = 8;
        respond(
            &summary,
            Some(&format!(
                "offline fixture {}\nsecond line",
                "café ".repeat(900)
            )),
        );
        std::fs::write(directory.join("requests"), "").unwrap();
        app.open_pull_request(8);
        settle(&mut app);
        assert!(app.review_error().unwrap().contains("second line"));
        for width in [320.0, 480.0, 900.0] {
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            status_frame(&mut app, &ctx, width, vec![]);
            let output = status_frame(&mut app, &ctx, width, vec![]);
            for label in ["Try Again", "Choose Number…", "Details…", "Dismiss"] {
                crate::ui::test_support::assert_visible_text(&output, label);
            }
        }
        let ctx = Context::default();
        ctx.set_pixels_per_point(scale);
        status_frame(&mut app, &ctx, 320.0, vec![]);
        status_frame(&mut app, &ctx, 320.0, vec![]);
        for _ in 0..4 {
            status_frame(
                &mut app,
                &ctx,
                320.0,
                vec![super::focus::key(egui::Key::Tab, egui::Modifiers::NONE)],
            );
        }
        status_frame(
            &mut app,
            &ctx,
            320.0,
            vec![super::focus::key(egui::Key::Enter, egui::Modifiers::NONE)],
        );
        assert!(matches!(&app.modal, ActiveModal::PullRequest(prompt) if prompt.pr_str == "8"));
        assert!(app.review_error().is_none());
        assert_requests(8, 1);
        respond(&summary, None);
    }
    println!("Review recovery: isolated CLI target counts, source recovery, narrow geometry and keyboard choice pass at 1×/2×");
}
