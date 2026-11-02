# Modal pointer scope and surface lifecycle

## Reference and confirmed defects

Swift RootView presents file, symbol, line and PR prompts with sheet(item:), which
makes the parent reader unavailable to ordinary pointer interaction while the
sheet is active. Rust rendered egui Windows and scoped keyboard commands, but the
parent toolbar and scroll areas still received pointer input outside the dialog.

A real-app regression failed before the repair: clicking the rendered Sidebar
button behind Go to Line hid the sidebar. Native old controls at 1×/2× also hide
it and scroll the source from the beginning to later rows while the dialog stays
open. The accepted source caret/selection is not permission to operate the parent
through a modal prompt.

## Shared implementation

The app's existing ModalFocus owner now prepares a transparent, hover-only egui
Area behind the active dialog and above the parent, including existing floating
utility windows. It adds no visible dimming, font, palette or text-scale change.
All four dialogs raise their actual existing layer during rendering. Outside
presses retain the previously focused dialog control instead of blurring the query.
The first measurement pass discards pointer/wheel/touch gestures whose visible
controls do not exist yet; the existing keyboard replay remains intact. Old drags
are stopped at opening and dismissal.

Dismissal retires the rest of that frame's input, pointer state and scroll deltas
before the background renders. It restores the prior owner through the existing
navigation/cancel paths. The blocking Area becomes noninteractive and empty when
inactive. egui 0.27 snapshots an Area's old movement rectangle before its contents;
the helper re-registers that same hover-only response ID with an empty rectangle
when retiring it. This prevents a one-frame invisible surface from swallowing the
next fresh click. Review this pinned-framework detail if egui is upgraded, just
like the existing source-scrollbar response-ID contract.

Native v3 exposed a second lifecycle defect: after opening other dialog kinds,
Go to Line's reused window could sort below the blocker and stop receiving Cancel
clicks. egui's move_to_top uses a stable end-frame sort. ModalFocus now lets the
dialog rise alone for one owned settling pass, then raises the pair together over
other windows. Repaints advance these passes without a timer or a source task.
The existing registration state and one per-owner readiness flag are sufficient;
Default shares initialization at both entry points. Surface preparation belongs
to that controller, while the Area helper only registers its geometry.

There is no per-panel disable list, new owner registry, worker, dependency, Cargo
change, public API or sibling checkout dependency. Background jobs can continue
publishing normal progress; this change scopes input rather than freezing work.

## Verification

Two real-app regressions cover all four dialog types. The first checks 1×/2×,
opening-frame pointer suppression, outside Sidebar clicks, source wheel input,
query focus, Escape plus a background click in one batch, exact Unicode/CRLF
selection/caret retention and the immediately following fresh Sidebar click.
The second uses a real egui utility Window and verifies its button works before
and after each modal but never behind one, while the modal's own Cancel remains
clickable. It runs line → file → symbol → PR → line in one Context to cover reused
window ordering. Its observed utility control is explicitly outside each dialog.

Workspace: 621 app tests pass, seven ignored; two binary, 13 font, 19 text and
40 doctests pass. Strict Clippy/check/rustdoc/build/fmt/diff and a final copied
standalone executable resource check pass. Cargo retains 16 build jobs; doctests
use four test threads to avoid the previously observed /tmp linker peak.

Native old v2 controls pass at 1×/2×, proving Sidebar mutation and changed source
crops under the line prompt, with two fresh exact source copies each. Final v4
new runs pass seven fresh exact clipboard checks at each scale, unchanged source
crops for outside wheel input under all four dialog kinds, blocked Sidebar clicks,
query focus, pointer Cancel, the retained departure selection, light appearance,
reopening Go to Line after the other kinds and fresh Sidebar hide/restore after
dismissal. Captures inspected; fixture/binary hashes stay unchanged and all final
app/compositor processes exit zero.

Early v1/v2 after-runs retained as harness evidence: the first crop missed old
source text after Sidebar disappeared, and the custom OCR data directory lacked
the named TSV config. Observed footer crops and explicit tessedit_create_tsv fix
the measurement. V3 is product-defect evidence for the reused-window ordering bug;
it is not counted as acceptance. V4 repeats the full new matrix after the fix.
Archive: ~/.local/state/radiology-qa/modal-pointer-20261102-dd3ce99.

## Architecture review and remaining scope

Comparison against before-modal-pointer-scope is NEUTRAL: +198 NLOC/four functions
including the two tests; no new clone group, cycle or hotspot, 109 groups retained
and health -0.01. Production CCN: surface preparation 5, modal input capture 5,
shared dispatch 14 and Area geometry 2. Common Default initialization removes the
initial duplicate construction rather than adding another initialization path.

This closes the confirmed parent click/scroll and reused-dialog surface defects.
Broader pointer/gesture/platform transitions, inherited text bindings, Unicode/UI
text, type/spacing alignment and final Swift feature/language/indexer/PR corpus
acceptance remain open. Full accessibility and actual little-screen-clip adoption
remain deferred by user scope.
