# Persistent inspector pane preferences

Implements the pane controls from `inspector-pane-controls.md` for the existing
Rust References, Definition/Outline and Bookmarks renderers. Trait Implementations
has a persisted preference and tested middle-slot policy; its semantic catalog and
results renderer remain the next feature in the upstream parity ledger. No trait
results or coverage are fabricated by this preference work.

## One settings model and command path

`session::inspector::{InspectorPane, InspectorPanes}` is independent of egui and
source/index workers. All four choices default on, including absent fields in old
or partially populated session JSON. The same session store persists choices
app-wide across repository changes and restarts. `visible(has_trait)` selects at
most three regions and gives an available trait pane precedence over Definition.
Without a trait renderer/context, existing regions follow their enabled choices.

The persistent Context header has a keyboard-operable Panes menu. View > Context
Panes uses the same checked item builder and ReaderCommand actions. No repository
or source is needed to configure panes. Show All restores defaults; enabling a
pane reopens a closed inspector. Disabling panes never closes the whole inspector.
Whole-inspector visibility remains a separate toolbar/keyboard action.

All-disabled and contextual-only empty layouts explain their state and retain the
Panes control plus Show All recovery. Existing panes reclaim the remaining height;
a single region fills the inspector below the header, while two/three retain the
existing resizable panel behavior and independent scrolling. No disabled pane
leaves a blank reserved slot. Definition retains its symbol name when References
is hidden, rather than deduplicating against an invisible heading.

## Data and input ownership

Preference commands change presentation and persist settings. They retain source
bytes, caret/viewport/history, bookmarks, explicit reference target, selection,
caller disclosures and semantic preparation. Hiding a focused pane retires its key
ownership and pending focus requests; the reader receives focus as needed. Bookmark
drag state is canceled without deleting selection or stored data.

Pane UI identities remain stable when a pane moves between split and central
regions. The existing `owned_ui` helper now accepts explicit IDs; existing picker
and definition callers derive the same parent-local owners as before. Pane owners
use stable region IDs, and the region wrapper keeps overflow within its allocated
bounds. This avoids both scroll resets and a large reference subtree pushing its
neighbor out of the inspector. Full existing picker/definition tests remain gates.

`ContextInspector::show_with_panes` accepts host-owned preferences and returns
commands to persist. `ToolbarContent` groups immutable toolbar inputs for the new
preference-aware entry point. Existing `show` APIs remain available with default
preferences. There is one canonical toolbar renderer and one pane settings model.

## Verification

- All 16 preference combinations and both trait-context states validate ordering,
  shared-slot behavior, old/partial JSON defaults and settings round trips.
- Actual app matrices at 1×/2× retain source Arc/caret/history, pinned references,
  selected caller/disclosures and bookmark identity across all choices. A single
  bookmark pane fills the available height. A 120-bookmark fixture preserves the
  first visible row through split → single → split layouts after manual scrolling.
- Real app pointer/keyboard actions use both Panes and View > Context Panes,
  recover all-disabled state, reopen a closed inspector without source, preserve
  nondefault settings across repository changes and retire hidden-pane keys.
- A direct renderer regression proves a hidden reference summary cannot suppress
  the exact under-caret definition name in dark/light at both tested scales.
- Final workspace: 654 app tests pass (seven existing ignored), plus two binary,
  13 font and 21 text tests. Strict Clippy/check/build/rustdoc, formatting, diff
  checks and 46 doctests pass. Rendered preference/toolbar/inspector APIs checked.
  Native previous/current 1×/2× acceptance passes and uses
  private profiles, keyboard menu selection, saved JSON, actual process restart,
  all-disabled recovery, View reopening, exact CRLF Copy and light appearance.
  Earlier OCR failures on the focused button border are retained as harness evidence;
  the final driver reuses the measured header-button position. Final binary/fixture
  hashes stay unchanged, exact source Copy is 148 bytes and all children exit zero.
  Captures were inspected.

## Architecture review

Baseline `before-inspector-pane-preferences` at `27a18f5`. Reuse lookup identified
session persistence, checked menus, existing panels and widget ownership. Header,
empty recovery and bookmarks rendering are cohesive helpers. The original oversized
toolbar argument list is grouped for new callers; the compatibility API forwards.
No additional worker, parser, dependency, preference cache or parallel navigation
state was added.

The default comparison flags toolbar `show_with_panes` at CCN23 as a new hotspot,
but the baseline `Toolbar::show` was already CCN23: the canonical rendering body
retains that complexity under the new entry point; the old name is now a shim.
This is not counted as a complexity improvement. Overall high-complexity count,
maximum, clone groups and dependency cycles remain unchanged. Normalized fragments
and paths distinguish shifted old test/picker clone reports from new duplication.
Final measurements are archived with the validation evidence.

| Dimension | Baseline | Current |
| --- | --- | --- |
| Analyzed NLOC | 93,718 | 94,390 (+672, including tests and public API docs) |
| Functions / files | 3,956 / 604 | 3,977 / 607 |
| Average / maximum CCN | 3.38 / 132 | 3.38 / 132 |
| High-complexity functions | 33 | 33 |
| Clone groups / cycles | 125 / 1 | 125 / 1 |

All 125 normalized clone fragments/paths/tokens remain identical. The original
CCN23 toolbar renderer remains CCN23; shifting its entry point is neither a new
behavioral hotspot nor a simplification. The new pane composer is below the
hotspot threshold, and fixed region bounds retain the existing overflow policy.

The requested SwiftUI/macOS/typography skills are absent from installed paths.
Checked-in Swift behavior and existing Rust small/header/menu type roles provide
the reference. No new font scale or custom interaction framework was introduced.
Full accessibility and actual little-screen-clip adoption remain deferred.

Evidence: ~/.local/state/radiology-qa/inspector-panes-20261102-27a18f5.
Next: actual Rust trait implementation indexing/context/navigation/UI, then Code
Paths, personalities/seL4 and remaining broad feature/UI/corpus acceptance.
