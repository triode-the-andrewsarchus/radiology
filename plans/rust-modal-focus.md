# Modal and breadcrumb keyboard focus

## Contract

Navigation committed from Go To Line, Go To Symbol, Go To File or a source
breadcrumb returns keyboard ownership to the stable source canvas. No extra
source click is needed before Home/End, selection or Copy. Cancel returns to the
previous live control, including the Find query; a missing owner is released.
Closing keys belong to the sheet for the whole frame. They cannot also close
Find or move source selection underneath it.

Modal keyboard/text events use the existing command event extractor before
framework focus handling, then reach the existing modal UI inside its frame.
The host removes those events before drawing the underlying controls. A single
modal dispatcher replaces repeated close/action branches. The native eframe
adapter and deterministic GUI tests call the same application rendering method.
There is no second renderer, dependency or alternate navigation implementation.

The stable canvas is also registered while the first source loads. Focus is
requested at the user's navigation action, never at asynchronous publication.
Later deliberate Find focus therefore survives a source result. Repository reset
retires both staged modal input and the previous owner. Closing explicitly
requests a repaint: a native 2× trial exposed the closing window remaining painted
until another event without it.

## Verification (2026-10-31)

Five real-app frame regressions cover:

- Go To Line → source selection/copy with no click, including a closing batch
  containing Return and Down that must not move beyond the chosen line.
- Cancel → live Find query, continued Unicode typing, and Cancel → source keys.
- Pointer breadcrumb commit → exact source line copy.
- First-file selection through Go To File, focus during loading and subsequent
  publication without stealing a deliberately focused Find query.
- Repository reset between raw input capture and rendering, with no retired key
  reaching a later modal.

With source focus requests disabled, all four initial focus regressions fail;
restoring the repair passes them. This is a causal negative control, not a claim
that an unmodified baseline supports the new real-app test adapter.
Workspace passes 472 application tests (seven ignored), two binary tests, four
font tests, eleven reader-text tests and 34 doctests. Strict Clippy, build, fmt
and diff checks pass. The final repaint change also passes the focused suite.

Architecture comparison labels the renamed rendering body a new hotspot and
flags four clones in the touched pre-existing app test file. The actual event
loop complexity falls from 47 to 36; native update forwards to it with CCN 1,
and modal dispatch is CCN 10. The full clone multiset audit has 92 before/after,
zero additions/removals; cycles and maximum CCN are unchanged. The comparison's
raw REGRESSION label is retained in the evidence archive, not treated as a clean
classification. No production complexity or duplicate mechanism is added by
splitting the modal lifecycle out of that body.

Final private native 1× and 2× runs use the same Unicode Rust fixture and verify
three fresh exact text offers per scale:

- Go To Line 5 → Home/Shift-End/Copy: `            let value = "café 🦀";`.
- Find `println` → Go To Line → Escape → type `target` → select/copy the query:
  `printlntarget`. Escape has neither closed Find nor sent the typing to source.
- Click the `target` breadcrumb → Home/Shift-End/Copy:
  `        pub fn target(&self) {`.

Every copy follows a PNG clipboard replacement and requires the expected SHA-256,
so a stale text offer cannot pass. Final dark captures at both scales show the
modal gone without a further user event; light captures show the destination
selection. Fixtures are unchanged and application/compositor exits are all zero.
QA helpers borrow the sibling's capture scripts only; the shipped app has no
sibling checkout/runtime dependency.

Durable archive: `~/.local/state/radiology-qa/modal-focus-20261031-4266178`.
It contains native inputs/output, captures, exact copies, fixture/binary hashes,
negative/positive tests, architecture reports, source patch and SHA256SUMS.

## Still open

[Toolbar menu traversal/focus](rust-toolbar-keyboard.md) is now implemented.
Broader rapid transitions among reader/search/
inspector owners, other long-label surfaces, large-file/Back/scroll/wrap
acceptance, broader grammar/index and PR/cache fixtures, and the final
requirement-by-requirement Swift parity audit remain required. The current
increment does not certify them. Full assistive accessibility and actual sibling
adoption remain deferred by the user.
