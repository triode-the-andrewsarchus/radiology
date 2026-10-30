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

## Integration still required

- Bind preparation to replaceable background work for an exact source/index/
  occurrence identity, preserving expansion inside a token and retiring stale
  publication on repository/revision/source/index changes.
- Present local, Here, package and unavailable-definition states, bounded index
  facts and source context through the existing inspector and navigation paths.
- Link parsed Rust impl/Swift extension type names to their exact indexed
  reference identities, retaining definition precedence for ordinary declarations.
- Verify real native interaction, disabled/unavailable states and exact navigation.

swiftui-pro, macos-design and typography-designer remain absent from configured
skill/plugin locations. Swift source and established inspector sizes are the UI
reference. Standalone operation and sibling reuse boundaries remain unchanged.
