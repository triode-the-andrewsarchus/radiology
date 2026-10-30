use super::*;
use crate::repository::test_support::fake_git;
use std::time::Duration;

#[test]
fn checkout_hunks_reject_moving_head_changed_bytes_and_interrupted_patches() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for (action, timeout, error) in [
        (
            "printf 'changed\\n' > source.rs",
            Duration::from_secs(5),
            None,
        ),
        (
            "touch head-changed",
            Duration::from_secs(5),
            Some("HEAD changed"),
        ),
        (
            "exec sleep 30",
            Duration::from_millis(100),
            Some("timed out"),
        ),
        (
            "head -c 17000000 /dev/zero",
            Duration::from_secs(5),
            Some("exceeds 16 MiB"),
        ),
    ] {
        let _ = std::fs::remove_file(root.join("head-changed"));
        std::fs::write(root.join("source.rs"), "old\n").unwrap();
        let buffer = SourceBuffer::read(&root.join("source.rs"), "source.rs", None).unwrap();
        let script = fake_git(
            root,
            &format!(
                r#"
case "$5:$6" in
  rev-parse:--is-inside-work-tree) printf 'true\n' ;;
  rev-parse:--verify) if [ -f head-changed ]; then printf 'after\n'; else printf 'before\n'; fi ;;
  cat-file:-e) exit 0 ;;
  diff:*) {action}; printf '@@ -1 +1,2 @@\n' ;;
esac
"#
            ),
        );
        let cancel = AtomicBool::new(false);
        let git = GitRead {
            root,
            executable: &script,
            cancel: &cancel,
            deadline: Instant::now() + timeout,
        };
        let result = git.checkout_source_lines(&buffer);
        if let Some(expected) = error {
            let error = result.err().unwrap();
            assert!(error.contains(expected), "{error}");
        } else {
            assert!(
                result.unwrap().is_none(),
                "new disk hunks cannot annotate the captured source"
            );
        }
    }
}
