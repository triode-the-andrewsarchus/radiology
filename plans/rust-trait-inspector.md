# Rust trait implementation inspector

The Rust app now connects the exact trait resolver/catalog to a compact, source-verified
inspector list. This ports the current dense Swift contract in
Sources/Radiology/UI/TraitImplementationsPane.swift and TraitImplementationRow.swift;
the older grouped/disclosure UI is superseded. The overall parity effort remains
active: Code Paths, indexing personalities/seL4, C/C++ fallback, help and broader
compiler/IME/corpus acceptance remain separate required work.

## Behavior and presentation

The existing middle Context slot shows **Trait Implementations** and a block count
when the Rust caret resolves to a trait, enclosing trait/member/default body, or
explicit trait impl. References and Bookmarks retain their existing regions. Pane
preferences use the established Panes and View menus; disabling implementations
restores Definition/Outline when enabled. Hidden regions retain semantic state and
selection while retiring keyboard ownership. There is no added permanent split,
trait-name subheading, file grouping, disclosure/preview or routine coverage footer.

Rows are flat, source-sorted and 22 logical points high at the normal type scale.
Primary implementing-type text uses the existing monospace style and tail elision;
secondary basename:line metadata uses the existing Small style and measured middle
elision, bounded to 96 points. Explicit negative impls have a secondary negative label.
Full path/signature are available on hover. The header tooltip retains trait identity,
positive/negative counts and coverage detail; a warning glyph appears only for gaps.
Waiting, empty and navigation-failure states are explicit. Fixed-height virtualization
keeps a 1,000-row list bounded; keyboard reveal scrolls only as needed.

A primary row click selects and activates. Up/Down (also Home/End) select without
moving source; Return activates and Escape clears. The context menu uses the shared
reader menu and offers Go to Trait Implementation. Ordinary keys are scoped to the
visible list and yield to fields, modifiers and menus through established dispatch.
Full accessibility remains outside current scope, as requested.

## Ownership, reuse and source verification

TraitImplementationContext owns an immutable index Arc, repository identity, trait
identity, selection and last navigation error. Moving source/caret within the same
trait retains rows/selection; trait/root/index changes clear them. Before has no
trait context. Accepted source publication retains impl facts from the same parse.
TraitStructure borrows those facts and the existing outline rather than cloning a
whole syntax snapshot or reparsing on the UI thread. The resolver performs only
in-memory exact semantic mapping; catalog enrichment remains on indexing workers.

Activation reuses the existing SourceLoader/LatestRead request and atomic navigation
transaction. The additive SourceTarget::ByteRange requires a full-source SHA-256
match and strict nonempty UTF-8 range validation. It forces a fresh read even for the
open file. Checkout trait reads and byte-range activation share canonical containment
validation; an external symlink with identical bytes still cannot become a destination.
Other existing source target policies remain compatible. Review requests own their
snapshot lease and read pinned After Git objects, independently of checkout and
materialized tree bytes. Pending activation also captures repository, index Arc and
full selected-row identity. Row/index/root/context changes retire publication.
Failures preserve source, caret, selection and history; Back restores the exact
last departure rather than the caret from when a background read started.

Reusable APIs remain additive: the semantic context, borrowed structure, byte-range
target and TraitImplementationList can be hosted independently. The list exposes its
keyboard owner, key mapping and reveal request. ContextInspector::show_with_traits
is the sole renderer; legacy show/show_with_panes forward to it. No sibling runtime
or crate dependency, parser, worker pool, persistence format or indexer was added.

## Validation and evidence

- Full workspace:675 app tests pass, seven existing ignored; two binary/13 font/21
  text tests and50 doctests pass. Strict Clippy/check/build/rustdoc/fmt/diff gates pass.
  Final same-open-file edit and context-menu key extensions pass all six targeted
  app trait regressions.
- Actual app frames prove pointer and Return activation, arrow-only selection, exact
  Back, same-trait retention, index replacement clearing, all16 pane combinations,
  hidden owner retirement, negative/zero-row contexts, Before clearing, stale-byte
  failure/retry and pending row/index retirement. The final failure case edits the
  already displayed destination file to prove activation cannot reuse old bytes.
- Real Git review test proves After navigation and Before clearing despite changed
  checkout/materialized bytes. An identical-content external checkout symlink is
  rejected before source publication/history changes. Byte-range unit tests reject
  empty, oversized, invalid UTF-8 and same-length stale destinations.
- A 1,000-row production-list test covers220/280/420 point widths,1×/2×, dark/light,
  bounded painting, compact heights and revealing the final selected row.
- Native previous/current1×/2× checks prove the old binary lacks the pane and the
  current binary supports row activation, selection/Return/Back, stale rejection,
  negative/empty/same-named trait contexts, preference hide/restore and light theme.
  Both scales copy the exact539-byte Unicode fixture. Accepted binaries/fixtures
  retain their hashes and all owned native children exit zero. Captures inspected.
- Earlier native driver failures remain as evidence: duplicate Alpha labels in
  references/tooltips needed row-header/left-edge targeting; decorated focused Panes
  needed retained measured geometry; a 2× copy needed a painted source frame after
  closing Go to Line. These were driver corrections, not accepted failed runs.
  The app context-menu test also selects its first item with Home before Return;
  Return on an unhighlighted mouse-opened menu correctly has no chosen action.
- Rendered public API contracts/examples were inspected. Requested SwiftUI/design/
  typography skills are absent from the installed catalog/search; existing app type
  styles and current Swift source provide the UI contract. No design-skill run claimed.

QA working directory: ~/.cache/radiology-qa-trait-inspector.
Permanent evidence: ~/.local/state/radiology-qa/trait-inspector-20261102-ed9cba3.
Accepted native records: native-v1-1.0-before, native-v2-2.0-before,
native-v4-1.0-after and native-v4-2.0-after.

## Architecture review

Baseline before-trait-inspector at ed9cba3. Reuse searches identified the source loader,
LatestRead, source range/digest mapping, shared pane/menu/list-key/reveal helpers and
existing compiler/Git fixtures. Fixture setup and pane-mask driving now reuse existing
test mechanisms. No new runtime dependency or worker subsystem was introduced.

| Dimension | Baseline | Current |
| --- | --- | --- |
| Analyzed NLOC |95,792 |96,755 (+963, including tests) |
| Functions /files |4,020 /613 |4,055 /618 |
| Average /maximum CCN |3.38 /132 |3.38 /132 |
| High-complexity functions |33 |33 |
| Clone groups /cycles |125 /1 |126 /1 |

The raw comparator reports REGRESSION for one new50-token clone: eight lines of
parameter declarations shared by the compatibility forwarding APIs. It contains no
rendering body or duplicated algorithm. Retaining those APIs is deliberate host
compatibility; inventing wrappers/renaming parameters to evade detection would not
improve the architecture. All other normalized clone identities are unchanged; the
flagged focus-test fragment predates this change. Chokepoint rankings also alternate
among untouched Swift-side Python functions. There is no new hotspot or cycle.
This is an accepted, documented signature duplication rather than a claim that the
raw tool returned NEUTRAL.
