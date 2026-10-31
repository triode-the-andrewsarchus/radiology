# Reader appearance and host alignment

## Problem and ownership

The reader's custom source colors came from AppTheme, while normal egui widgets
used unmodified Light/Dark defaults. Its sidebar and raised-surface roles were
unused by the main layout. Muted interface text had about 3.26:1 light and 3.36:1
dark contrast on the original source surface. Theme switching reinstalled those
unrelated widget defaults.

AppTheme now resolves its own widget visuals. Window creation and theme commands
use that same mapping. Calling `visuals()` has no context side effects; embedding
hosts can opt in with `Context::set_visuals` and customize existing public theme
colors. The mapping does not set fonts, type sizes, spacing, zoom or window policy.
`from_session` remains independent of a graphics context. Standalone launch and
resizing remain owned by Radiology; no workbench or sibling runtime is imported.

## Roles

The following neutral values match little-screen-clip's current
`little-clip-core::palette::BaseColors`. Border and accent/link values use its
fixed chrome border and readable link roles. These are small local reader
constants; this does not migrate the sibling or establish a shared palette crate.
The existing AppTheme mechanism is reused rather than adding a parallel theme API.

| Reader role | Dark | Light |
| --- | --- | --- |
| Source canvas/gutter | #1f1f1f | #eeeeee |
| Panels, popup windows | #2c2c2c | #e3e3e3 |
| Raised controls | #383838 | #ffffff |
| Primary text | #f2f2f2 | #262626 |
| Supporting text | #d5d5d5 | #484848 |
| Muted interface text | #b5b5b5 | #595959 |
| Border | #8c8c8c | #707070 |
| Accent/link/focus | #42b9f3 | #215c84 |

Buttons, fields, popups and selected controls derive from these roles. Selection
and hovered/open/active controls use a 12% sRGB accent mix over the panel, resolved
opaque before painting. This keeps custom-painted outline/bookmark text readable
on the same fill. Selected widget text and focused field borders use the accent.
Warning/error foregrounds are separately readable in both appearances. Existing
source diff, occurrence, destination and function-shade mechanisms retain their
ownership; no source geometry, byte mapping or selection logic changes here.
Syntax token colors and deliberately receded Critical Lens text are outside the
interface contrast claim below.

The requested SwiftUI/macOS/typography skills remain absent from configured roots;
before/after review uses `design-system.md`, the original parity review and the
sibling's current BaseColors/widget mapping. Font sizes are unchanged in this
increment. Broader typography/spacing and full accessibility acceptance remain open.

## Validation — 2026-10-30

Two theme tests cover interface contrast across all built-in canvas/panel/raised/
gutter/current-line/selection surfaces, normal/hovered/active/open widget states,
links/warnings/errors, and border/caret visibility. Text roles are at least 4.5:1
on the tested surfaces; border/caret contrast is at least 3:1 on base/panel/raised
surfaces. Custom host colors drive the mapping, and applying visuals preserves
text styles and spacing. Muted text now has 5.46:1 on the light panel and 5.72:1
on the dark raised control surface. This verifies the named color pairs, not
complete application accessibility or every syntax token.

An app integration test toggles both appearances, checks actual reader visual
publication and persisted session choice, and preserves customized host metrics.
Workspace gates pass 436 app tests (seven ignored), four font tests, eight shared
text tests and 33 doctests. Strict Clippy, build and formatting pass. Warning-denied
rustdoc and its rendered AppTheme/visuals contract/example pass inspection.

Architecture baseline `before-reader-appearance`: max CCN stays 100, hotspot count
25, duplication 0.72%, 92 clone groups, no dependency cycles. The comparator flags
four app-test clone fragments because their file is touched. Direct comparison
confirms all four fragments exactly match the baseline (only positions shift);
none intersects the new theme test. No new clone/hotspot/cycle is introduced.

## Native acceptance and retained evidence

Accepted private Weston/Niri sessions use 1280×900 at 1× and 1024×768 logical at
2× (2048×1536 pixels). Both exercise source, selected outline/file rows, exact
emoji-line copy, 1/1 local find in dark/light, file menus and focused Go to Line
fields in both appearances. The narrow find bar wraps its count and Close control
without overlap. All fixture bytes remain unchanged, and app/Weston/Niri exit zero
at both scales. Clipboard bytes are read before screenshots, which replace Niri's
clipboard with PNG content. The sibling capture helper is QA-only.

Durable local evidence is retained under:
`/home/vega/.local/state/radiology-qa/appearance-20261030-cd82b05`.
Its `1.0` and `2.0` subdirectories contain before/acceptance JSON, binary and fixture
hashes, clipboard bytes, native captures, logs and exit statuses. Top-level logs,
rendered rustdoc, the QA driver and SHA256SUMS are included. Working outputs are
`/tmp/radiology-appearance-native-{1.0,2.0}`; the earlier `-before-focus` trial is
excluded from accepted evidence. The final native run uses accent focus borders.

The native pass also proves a remaining keyboard gap: the file context menu stays
open after standalone Escape and a 400 ms wait at either scale. Outside click
closes it. See `file-menu-after-escape-light.png`. Earlier combined Escape/Go to
Line also left the menu behind the dialog. The follow-up [context-menu dismissal](rust-context-menu-dismissal.md) increment
now repairs this gap with shared pre-frame Escape ownership and native/component
regression evidence. Broader menu traversal/focus audit remains open.

Broader project/corpus/recovery, all widths/scales, complete menu/focus states,
accessibility, sibling component adoption and the final Swift requirement audit
remain required for the full goal.
