# Accepted navigation and source selection

## Swift contract and confirmed defect

Swift AppModel.navigate marks explicit destinations as selecting text and changes
navigationRevision. SourceViewer applies a zero-length NSRange at the resolved
UTF-16 destination. Passive caret updates and preview/reflow do not use that path.

Rust previously inferred an explicit navigation from a pending scroll target and
cleared selection only if the new caret matched neither endpoint. Go to Line,
history or bookmark destinations at an old anchor/focus consequently retained the
old selection. A real-app regression failed before the repair at destination zero.

## Transaction boundary

SourceViewport.accept_navigation retires old source input, selection, hover,
preview, pending reveal and landing feedback. RadiologyApp calls it only from
apply_destination after the navigation has successfully resolved, before installing
the new caret and reveal. Immediate same-buffer and asynchronous source navigation
share this path. Narrow preview/landing cancellation keeps its existing policy.

The scroll-target heuristic is removed: reflow and deferred reveal are not proof
of a new navigation. Input after acceptance, including same-frame Select All,
remains valid. Failed/canceled source requests keep the displayed selection,
caret and history; Find separately installs its accepted match selection. Recolor,
preview and wrapping retain selection. No source clone, dependency, worker or
Cargo setting is added. Embedding hosts can call the public viewport method at
their own successful-navigation boundary.

## Verification and evidence

Three real-app regression matrices cover forward/reverse selections, both old
endpoints and interior/newline destinations, 1×/1.5×/2× with wrapping on/off,
canonical Unicode/CRLF copy, fresh Shift selection and immediate Select All,
Back/Forward, canceled/failed source requests, Find, preview and recolor.
Two existing isolated viewport tests now explicitly simulate accepted navigation.

Workspace: 614 app tests pass, seven ignored; two binary, 13 font, 19 text and
40 doctests pass. Strict Clippy/check/rustdoc/build, formatting/diff checks and a
copied standalone executable resource check pass. Architecture is NEUTRAL:
+144 NLOC/four functions including tests, no new hotspot/cycle/clone group;
109 clone groups retained and health -0.01.

Native old controls at 1×/2× reproduce the retained selection for forward and
reverse source selections followed by Go to Line at the old anchor/focus. Final
v4 acceptance at both scales checks seven fresh exact copies plus four empty-copy
checks each: accepted navigation leaves a verified text sentinel unchanged, fresh
Shift selection copies the new byte, and Select All survives appearance and wrap.
Captures show the old/new selection difference and retained new selections.
Fixture/binary hashes stay unchanged and all final child processes exit zero.

Early v1-v3 after-runs exposed a harness issue: niri screenshots replace the
clipboard with image/png. V4 checks Copy before taking a screenshot and verifies
its sentinel before every negative check. Those attempts remain in the archive.
Archive: ~/.local/state/radiology-qa/navigation-selection-20261102-523885d.

## Remaining audit

This closes the explicit-navigation selection defect. It does not close every
inherited text binding, selection-collapse rule or input-owner transition. General
Unicode/UI text, typography/spacing and broader Swift feature/corpus acceptance
remain in the master parity audit. Full accessibility and actual sibling adoption
remain deferred by user scope.

## Architecture review

| Dimension | Baseline | Current | Delta |
|---|---:|---:|---:|
| Average CCN | 3.33 | 3.33 | 0 |
| Maximum CCN | 132 | 132 | 0 |
| Clone groups | 109 | 109 | 0 |
| Dependency cycles | 0 | 0 | 0 |
| Functions including tests | 3,638 | 3,642 | +4 |

The new method reuses cancel_deferred_input and cancel_destination, combining
existing retirement with explicit selection/preview/reveal cleanup. Those helpers
alone have narrower ownership: previews call cancel_destination and must retain
selection. Production logic replaces a 13-line heuristic with one explicit
transaction call and a bounded cleanup method; most added lines are regression
matrices. No new hotspot or worsened existing hotspot is reported. Accept.
