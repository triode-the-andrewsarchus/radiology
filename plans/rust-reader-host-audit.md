# Independent build and reader-host audit — 2026-11-05

## Reuse contract

Radiology remains an independent app. `reader-text` owns source-byte geometry,
selection and search primitives; `reader-fonts` supplies portable assets and optional
egui/shaping bridges. Neither requires the Radiology app. The full `radiology`
library also exposes SourceView and its borrowed host-owned state. Actual adoption
by little-screen-clip remains deferred.

A separate consumer crate exposed a concrete gap: SourceViewport was public, but
`request_focus`, `top_byte` and `restore_top` were crate-private (three E0624 errors).
Those existing methods are now public and documented; their implementations and
state ownership are unchanged. Hosts can return keyboard focus, remember an exact
visible-row byte and restore it independently of caret/history, using the same
renderer and document identity checks as the app.

The focus contract has one active source-canvas owner per egui context. Before
restoring an anchor, hosts reveal any enclosing folds/hidden test regions they own.
Top-byte queries require the exact displayed source identity. Explicit navigation
uses `accept_navigation` before installing its destination; ordinary reading-anchor
restoration preserves caret/selection in the current document.

A full reader host handles SourceViewAction, including the new `FollowDefinition`
destination from the Return checkpoint. Destination ranges retain the index's
encoding: use `SemanticIndex::column_encoding(path)` and validate against the
loaded destination buffer before publication. Context, modal input, source loading
and history remain host responsibilities. The [ordered modal input contract](rust-picker-preframe-focus.md)
also applies when embedding the app's pickers.

### Consumer manifests

Cargo applies patches from the consumer's workspace root. A full reader host forwards
the existing pinned grammar/backend patches (or intentionally provides equivalent
fixes); a dependency's patch table does not propagate automatically:

```toml
[dependencies]
radiology = { path = "../radiology" }
egui = "0.27"
reader-text = { path = "../radiology/crates/reader-text" }
reader-fonts = { path = "../radiology/crates/reader-fonts", features = ["egui", "shaping"] }

[patch.crates-io]
tree-sitter-kotlin-ng = { path = "../radiology/vendor/tree-sitter-kotlin-ng" }
egui-winit = { path = "../radiology/vendor/egui-winit" }
```

An asset/text-only host depends on the two shared crates and leaves reader-fonts'
default features disabled. It needs no app, egui, Wayland or desktop-portal crate.
The sibling workspace was inspected read-only; it already uses the egui0.27 backend
patch convention. No sibling file or dependency was modified by this audit.

## Authoritative checks

Temporary execution: `~/.cache/radiology-qa-standalone-audit`.
Durable evidence: `~/.local/state/radiology-qa/reader-host-20261105-5de533f`.
The archive includes both complete external consumer manifests/sources/lockfiles,
Linux metadata, red/green logs, resource/host outputs and architecture reports.

1. **Clean locked workspace build:** a fresh target directory compiles the workspace
   offline with verbose compiler logs. Its initial stage builds the current repository;
   the isolated git-archive export then compiles its own project crates and executable,
   reusing the newly compiled third-party dependency cache. The public-method patch is
   copied into that export. `.cargo/config.toml` supplies16 jobs for32 available CPUs;
   separate consumer commands explicitly use16 too. No sibling source/build output is
   used. This is a Linux debug build, not a distributable package/install claim.
2. **External full host:** an independent package consumes the exported library and
   shared text/fonts. With the repaired public API it runs at1×/2×: programmatic
   source focus, exact line60 viewport restoration without moving line1's caret,
   Unicode Select All/Copy, changed-document anchor rejection and focus return pass.
   Its public SourceViewContent is constructed independently. It also compiles the
   FollowDefinition handler, checks shared original-byte fragments/selection and
   parses Kotlin through the exported native queries/grammar. The Return handler is
   a compile contract here; its actual semantic navigation is accepted separately.
3. **External asset-only host:** with reader-fonts default features disabled, the
   independent binary reads bundled font bytes and verifies exact CRLF/Unicode text
   selection. Linux Cargo metadata contains no egui/eframe/Radiology/Wayland/portal
   dependency. The earlier unrestricted offline metadata query requested an uncached
   Windows clipboard package; target-filtered Linux metadata is used for this Linux
   audit, without downloading that unused platform dependency.
4. **Source-unavailable runtime:** the entire exported source tree is renamed away.
   A copied standalone executable runs `--verify-resources` from an otherwise empty
   run directory; both external hosts run too. All three exit zero, proving the
   exercised queries/fonts/source reader require no runtime source checkout/assets.
   The source tree is restored afterward. Linux dependency metadata confines local
   packages to the private export/consumers and confirms the pinned vendor patches.
5. **Root verification:** full locked workspace777 app tests/11 opt-in ignored,
   two binary,13 font,22 text and60 doctests; final strict Clippy, rustdoc generation,
   format and diff checks pass. Public-method docs are generated and checked.

## Occurrence pin assertion reconciliation

All four OccurrencePinningTests reference contracts now have explicit evidence:

| Swift assertion | Rust evidence |
| --- | --- |
| Multiple pins retain colors after independent removal | `pin_groups_have_no_eviction_limit_and_removal_does_not_recolor_survivors`; expanded app lexical matrix retains its first group exactly when a second is pinned. |
| Per-file memory survives revisits and rejects changed bytes | `document_restoration_requires_identical_content_and_revision_and_clears_ephemeral_state`; app `occurrence_pins_restore_on_return_and_invalidate_only_after_successful_changed_source`. |
| Semantic arrival removes the equivalent lexical pin | Expanded `syntactic_pins_include_cfg_omissions_but_prefer_complete_semantic_identity` now leaves a lexical pin active through semantic arrival and verifies that the semantic toggle removes it. |
| Android logging fixture has four tokens, excluding string/other function; transient misses preserve pins and unpin is independent | Expanded `lexical_pins_use_whole_identifiers_and_the_enclosing_function` replays the exact shadowed-message/C-string fixture and original value/values case; app transient-state regression covers drawing-only ownership. |

These are actual Rust app-model/unit checks and source reconciliation; no new native
pin-click run or SDK compilation is claimed in this checkpoint. The expanded
existing tests pass without a pinning production change.

Architecture baseline `before-reader-host-api`: +42 NLOC, no new function/file/dependency,
with production changes limited to visibility/docs on existing methods. The pin
matrix adds one necessary test branch. Average/max CCN3.39/132,36 hotspots, one cycle,
eight structural findings and the entire125-instance/1335-line clone Counter are
unchanged. Raw diff-aware reports flag old self-clones and untouched Swift Python
rank noise; zero actual clone additions/removals.

## Remaining goal

This closes the build/resource/reuse gate for the audited state and reconciles the
four pin assertions. Re-run relevant gates after further runtime/API/resource
changes. Other Swift assertions in the [suite index](rust-behavior-suite-index.md)
still require reconciliation. SDK-backed Android success remains pending the user's
Google SDK-license authorization. Actual sibling adoption and full accessibility
remain outside the current scope.
