# Changed callable markers in review outlines

Swift's reference is `AppModel+PullRequests.changedCallableIDs` and
`UI/OutlineRow.swift`: a six-point tint circle identifies callables whose complete
source declaration overlaps changed bytes in the displayed PR revision. Other
declarations keep their existing presentation.

## Implemented behavior and ownership

- After uses added lines; Before uses removed lines. Navigation-only anchors for
  empty hunk sides do not become changed source. Changed lines convert to full,
  half-open byte spans, including CRLF, and clip at EOF. An empty trailing source
  line produces no overlapping bytes.
- Any overlapping callable is marked, including nested callables and enclosing
  functions. An end exactly at the next declaration's start does not mark that
  declaration. Non-callable declarations, unchanged callables, binary diffs and
  checkout views do not receive PR markers.
- Markers occupy six points before the existing line count, with reserved name
  space. The row tooltip explains that the declaration contains PR changes;
  widget metadata includes "Changed in this pull request". Broad desktop
  accessibility acceptance remains separate.
- Marker flags use the same captured FileLineChanges and exact displayed source
  as source publication. Review reads use pinned Git objects; rename aliases
  already share the per-file diff. The host passes review data only during an
  active snapshot, and OutlineList also requires a revision-bearing buffer.
- Outline metadata is already keyed by path/revision and SHA-256 source identity.
  Marker flags recompute when that document or the diff changes, preserving
  selection/disclosure on caret motion and same-document diff updates. Removing
  the diff or leaving review clears markers. No separate overlay worker, new
  dependency or duplicated review/source model was introduced.
- `SourceBuffer::line_span_byte_range` owns the reusable coordinate conversion.
  It preserves original bytes and rejects invalid or empty spans.

The same native review check exposed a line-map error at zero-length hunk sides:
Git places an empty side after its anchor. The unchanged anchor itself must retain
its preceding position; only following lines receive the insertion/deletion
delta. `FileLineChanges::map_line` now preserves that boundary while accumulating
prior hunk offsets. Revision round trips retain the untouched Unicode comment at
line 1 rather than jumping to line 5 in Before.

## Automated evidence and architecture

Commits: `925e8c3` (full-line bytes), `3c84b2c` (markers), `752a666` (anchor mapping).
At `752a666`, full workspace tests pass 288 app tests, 5 reader-text tests and
30 doctests; one external-Clang-dependent test remains intentionally ignored.
Formatting, Clippy with warnings denied, binary build and rustdoc pass.

New checks cover CRLF/Unicode/full-line EOF boundaries; Before versus After
callables; half-open declaration boundaries; nested overlaps; binary, missing,
checkout and empty-EOF changes; same-document diff replacement/selection;
insertion/deletion/BOF/multiple-hunk mappings; and a real pinned Git review whose
painted marker centers are matched to the correct callable labels. The attached
checkout contains a different dirty callable; leaving review paints no PR dots.
The application's line-1 round-trip assertion failed before the anchor fix.

Architecture baseline `before-outline-changes` is at `f67a531`. Product/test NLOC
increases by 290 with no new files, hotspot or dependency cycle. Average CCN stays
3.34, maximum 100, duplication 0.69%, clone groups 91. The comparison labels its
touched-file report REGRESSION for four pre-existing test fragments; no new
product duplicate mechanism was found. Reusing source/diff ownership and cached
outline entries keeps the feature within existing components.

## Native evidence

The final private Wayland fixture and controller are
`/tmp/radiology-outline-changes-final` and
`/tmp/radiology-outline-changes-final-driver.py`. PID 3978451 runs the binary at
`752a666`, SHA-256
`aa2d4633b439d978814980536b68ee2aa968c429a61f30579446dc459240ab18`.
The fake gh command supplies metadata only; Git commits, pinned reads, hunks and
source parsing are real. The attached main checkout has a dirty checkout-only
function that differs from both reviewed commits. The test/helper reuse from
little-screen-clip adds no product runtime dependency.

- `after.png`: edited and added_only have dots; untouched and PlainAfter do not.
- `before.png`: removed_only and edited have dots; untouched and PlainBefore do
  not. Source status remains Line 1 after Ctrl+Alt+B.
- `marker-tooltip.png`: the first changed callable explains PR membership.
- `after-return.png`: the marker set returns to After and caret remains Line 1.
- `light.png`: the same markers/line counts are readable in light appearance.
- `checkout.png`: exiting PR review shows the real dirty checkout_only callable
  with no PR dot and Checkout bookmark scope.

Capture/build hashes and local Git identities/status are retained in
`build-evidence.json` and `git-evidence.json` in that directory. App and private compositor exits are zero; pointer-helper termination is
intentional. Earlier `/tmp/radiology-outline-changes-native` captures precede
the anchor-map fix and
are not final mapping acceptance.

## Remaining scope

Transient destination feedback, enclosing-function shading, finer fold/import
presentation, broad language/long-name/scaling/accessibility/native restart and
side-by-side Swift audits remain open. So do the larger request/revisit caches,
background repository work, index/tool lifecycle, PR refresh/cancellation/cache
provenance and shared palette/chrome integration. This feature does not close the
full parity objective.

The requested swiftui-pro, macos-design and typography-designer skills remain
absent from configured roots. Existing Swift behavior, theme colors and Rust
Body/Small styles were used; no new type scale was introduced.
