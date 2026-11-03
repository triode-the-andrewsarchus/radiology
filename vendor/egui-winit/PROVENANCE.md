# Pinned egui-winit IME lifecycle repair

Source: crates.io `egui-winit`0.27.2, egui revision
`014327e36535deeca0839b6aca2646191f2bd2fb`:
https://github.com/emilk/egui/tree/014327e36535deeca0839b6aca2646191f2bd2fb/crates/egui-winit

`UPSTREAM.json` records the original archive SHA-256 (matching the former Cargo.lock
checksum), all original file hashes and licenses fetched from that exact revision.
The upstream MIT and Apache2.0 licenses are retained. `Cargo.toml.orig` is unchanged.
The root patches the existing dependency to this directory; builds require no
network retrieval or untracked registry modification for the repair.

`ime.patch` contains the entire local source/manifest delta. It extracts the existing
IME translation into one private function, enabling two deterministic boundary
regressions without constructing a native window. Other native event handling,
clipboard, window settings, versions and dependencies are unchanged. The normalized
manifest's license paths are adjusted to the retained local copies.

Winit documents an empty Preedit as a clear, independently of its optional cursor.
The old backend ignored `Preedit(_, None)` and Disabled. Actual Wayland/Anthy
cancellation consequently left marked text and composition ownership active.
The repair translates a clear or Disabled while composing into an empty
CompositionUpdate followed by an empty CompositionEnd. Both events are necessary:
egui0.27's empty End alone retains the marked selection. Duplicate clears are
ignored. Nonempty preedit is retained even with a hidden cursor. Commits keep their
existing standalone End behavior; winit's preceding clear releases the old mark
before inserting the final text exactly once. A following preedit starts afresh.

Run the backend regressions from the root:

```sh
cargo test -p egui-winit --lib --locked
```

The package remains outside the workspace, like the pinned Kotlin scanner, so this
explicit gate supplements `cargo test --workspace --locked`. Native acceptance uses
an actual Fcitx/Anthy engine and checks exact draft/query bytes, candidate navigation,
clear, commit/Return ownership and resize at1×/2×. See
[the native IME contract](../../plans/rust-native-ime.md) for evidence and limitations.
The separate app output correction anchors native candidates to the caret; this
patch addresses input lifecycle only.

To update: verify a new archive/revision and licenses, replace unchanged inputs,
review/reapply the recorded patch and repeat both backend and real-engine checks.
Retire this patch when an upgraded integration passes those same checks. No upstream
submission or broad egui upgrade is part of this change.
