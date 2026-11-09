//! Repository discovery, filtering and source-loading parity contracts.
use super::{focus::frame, *};

#[cfg(unix)]
#[test]
fn external_checkout_links_cannot_replace_the_reader_through_file_or_bookmark_navigation() {
    use std::os::unix::fs::symlink;
    for scale in [1.0, 2.0] {
        for bookmark in [false, true] {
            let (owner, mut app, id) = reader_with_bookmark();
            let root = app.repository_root.clone().unwrap();
            let path = root.join("a.rs");
            let original = std::fs::read(&path).unwrap();
            let outside = owner.path().join("outside.rs");
            std::fs::write(&outside, &original).unwrap();
            app.open_file("b.rs");
            settle(&mut app);
            let ctx = Context::default();
            ctx.set_pixels_per_point(scale);
            frame(&mut app, &ctx, vec![]);
            let accepted = app.open_buffer.clone().unwrap();
            let caret = (app.caret_line, app.caret_col);
            let history = format!("{:?}", app.history);
            std::fs::remove_file(&path).unwrap();
            symlink(&outside, &path).unwrap();
            if bookmark {
                app.execute_command(&ctx, ToolbarAction::OpenBookmark(0));
            } else {
                app.open_file("a.rs");
            }
            settle(&mut app);
            assert!(Arc::ptr_eq(&accepted, app.open_buffer.as_ref().unwrap()));
            assert_eq!((app.caret_line, app.caret_col), caret);
            assert_eq!(format!("{:?}", app.history), history);
            assert_eq!(app.bookmarks.bookmarks[0].id, id);
            let error = if bookmark {
                assert!(app.error.is_none());
                &app.bookmark_list.navigation_error.as_ref().unwrap().1
            } else {
                app.error.as_ref().unwrap()
            };
            assert!(error.contains("leaves the repository"));
            assert_eq!(std::fs::read(&outside).unwrap(), original);
        }
    }
}
