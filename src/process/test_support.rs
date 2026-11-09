//! Linux descendant checks shared by process and installer interruption tests.

use std::{
    path::Path,
    time::{Duration, Instant},
};

// The runner reaps its direct child. Descendants are killed as a group but may
// remain zombies until their new parent reaps them; a live descendant must fail.
pub(crate) fn assert_child_stopped(root: &Path) {
    let pid: u32 = std::fs::read_to_string(root.join("child.pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"));
        if stat.is_err() || stat.unwrap().rsplit_once(") ").unwrap().1.starts_with('Z') {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "descendant {pid} survived group termination"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
