# Inspector metadata and implementation context

## Query contract

The source for this package is Swift UnderCaretContext, DefinitionDetailsView,
DefinitionLocationView and DeclarationContext. The Rust query reports one exact,
single-line non-import occurrence in current source, using scoped identities and
source/encoding validation. Before, stale, multiline and absent occurrences yield
no metadata. Preparation accepts cancellation and belongs on a background worker.

Facts include SCIP kind names (unknown numeric values remain explicit), recorded
signature bounded to eight lines/600 UTF-8 bytes, first prose documentation
paragraph excluding fenced code and bounded to 280 Unicode scalars, indexed
reference/definition counts, validated current-file occurrence count, recorded
enclosing name and package/version when no repository definition is indexed.
Counts are index facts, distinct from loaded source coverage. Missing definitions
do not prove that code is external; presentation must explain missing index
coverage as well as package facts. Local definition lines come directly from the
captured displayed bytes, without another file read.

Definitions are source-ordered/deduplicated; the chosen destination excludes the
current occurrence and invalid path/coordinate envelopes. The destination is an
index fact until existing navigation validates destination bytes and provenance.
On-definition state can therefore say Here while retaining other indexed
locations. Source content must never be reconstructed from documentation.

Five backend tests cover global Here/other-definition behavior, exact UTF-16 and
emoji/local identities, local source-line emphasis, package-only symbols,
import/Before/stale/multiline/cancellation rejection, documentation/signature
bounds and known/unknown kinds. Shared UTF-16 test occurrence construction now
serves metadata, references and Critical Lens tests. Full gates pass 267 app
tests (one external-Clang ignored), 5 reader-text tests, 30 doctests, fmt, Clippy
and rustdoc. The initial CCN17 preparation hotspot was removed by separating
occurrence counting and destination-envelope validation. Architecture has no new
product hotspot/cycle and lower duplication; the compact SCIP kind-name table
adds necessary data, not a second parser.

## Background and reader integration

Metadata preparation runs on the existing LatestRead worker pool. Captured
source/index inputs and an exact scoped occurrence determine replacement; moving
inside the same token preserves disclosure. Repository/revision/source/index
changes retire publication channels. Metadata loading/error state is explicit.
Prepared nonlocal destinations feed the existing exact-source preview worker;
locals use their primed source line and never start a definition fetch. Definition
activation uses the same prepared destination, including another definition when
the current one is Here. Tests cover replacement races and absent indexes.

The source canvas may mutate its caret and then return a higher-priority action,
including PinAtByte instead of CaretMoved. Native testing exposed stale metadata
on that path. The host now observes the actual caret before/after every source
interaction, updates current history and refreshes semantic context regardless of
the winning action. A regression covers pinning and unchanged-caret disclosure.

Rust impl and Swift extension have an Implementation outline kind. Declaration
reference context prefers a definition at the parsed name, but these constructs
may bind an exact indexed type reference. Ordinary declarations still require a
definition. Native grammar tests cover Rust/Swift kind/linking, precedence and
stale source; no line-based owner guessing was introduced.

The inspector shows compact name/location states and expands recorded kind,
indexed counts, signature, prose and exact source. Locals report file occurrences
and enclosing names. No repository destination shows package facts or an honest
unavailable state, without claiming missing coverage proves external code. Package
labels preserve escaped names and omit placeholders/source URLs in version fields,
matching Swift. Definition locations distinguish Here from navigable destinations.
The UI uses established inspector type sizes and existing explicit navigation.

## Native acceptance — 2026-10-30

Core acceptance at 96b86fd ran as PID 3819672 in a private 1280×900 Wayland
session. The fixture includes UTF-16 positions after an emoji, a Unicode local,
fenced signature documentation, a package-only symbol and a real impl/type pair.

- café shows a.rs:2, Local variable, two file occurrences, enclosing demo, recorded
  signature/prose and the exact highlighted local definition line.
- save shows Function, 517 indexed references and one definition, the first prose
  paragraph excluding fenced code, and the real three-line definition preview.
  Return opens b.rs:1 with Here, and Back restores the original use after the emoji.
- system_open shows package std 0, recorded signature/prose, two indexed references
  and zero definitions. Return leaves source unchanged (before/after captures have
  identical hashes); missing destination is not rendered as a link.
- The impl header shows Implementation and binds Thing's exact reference context.
  F12 opens the c.rs:1 type definition; Back restores the c.rs:2 implementation site.

Radiology and both private compositors exit zero. Evidence, screenshot hashes,
binary SHA-256 and code checkpoints are in
/tmp/radiology-inspector-metadata-native/build-evidence.json. Only accepted-*.png
captures belong to acceptance; earlier captures exposed the pinning refresh gap.
The final package-URL display adjustment at cc61027 is covered by parameterized
unit cases; the record distinguishes its build from native core acceptance.
The sibling project provides a private test harness only, without a runtime
or build dependency in Radiology.

Full gates pass 273 application tests (one external-Clang test ignored), five
reader-text tests, 30 doctests, fmt, Clippy with warnings denied, build and rustdoc.
Architecture comparison to before-inspector-metadata introduces no product
hotspot/cycle and keeps maximum CCN 100; average CCN rises 0.01. Duplication is
approximately unchanged with one additional short clone group. The new clone is
painted-text extraction in headless tests, not a second product mechanism; other
reports concern existing fixture/registration idioms in modified files. Shared
encoded occurrence construction removed a real fixture duplicate across three
clients. Valknut health falls about 1.30 points with the added context model and
SCIP kind-name data; no new structural hotspot is reported. The query's initial
CCN17 hotspot was removed rather than suppressing measurement.

## Remaining inspector acceptance

- Broader language, relationship, malformed/partial-index and long-name corpus.
- Pinned-review metadata/native acceptance, light/dark, scaling and side-by-side
  Swift visual checks.
- Broader outline/bookmark presentation and complete definition/inspector focus,
  enabled-state and accessibility acceptance.
- Revisit cache reuse, background repository context and finer native-stage
  cancellation within the larger lifecycle work.

swiftui-pro, macos-design and typography-designer remain absent from configured
skill/plugin locations. Swift source and established inspector sizes are the UI
reference. Standalone operation and sibling reuse boundaries remain unchanged.
