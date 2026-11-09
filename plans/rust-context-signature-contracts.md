# Nested declaration signature presentation

Swift reference: ContextModelTests.nestedMultilineSignaturesStopAtTheBodyAndReadFlush.
Rust starting point: `e86dbd3`. The signature group and the exact escaped-descriptor/
package-label group now have mappings: ten of eighteen are mapped, eight remain.

## Reproduced differences

Two checks fail against the previous renderer (`red.log`). The exact Swift fixture
paints eight spaces before &self and four before the closing parenthesis; Swift
removes the containing impl's four-space indent. The existing oversized-signature
check also fails when tightened from the generic 4096-byte fit to the 600-byte
signature budget. Neither failure requires an index or a new parse mechanism.

The summary now derives display text from its existing raw header. It reuses
semantic Context's scalar-safe 600-byte/eight-line limiter, then removes the
source declaration's own space/tab prefix from matching continuation lines.
CRLF handling comes from the shared line normalization. Trailing display whitespace
is removed. Inline declarations with non-whitespace before them do not acquire
an invented indentation prefix. Invalid source ranges safely yield no prefix.
A prefix longer than the already bounded display cannot match a continuation,
so it is skipped without scanning an arbitrarily long source line on the UI thread.

The parser's DocumentSymbol.detail and SourceBuffer remain exact. The existing
Copy Signature action still copies the full raw trimmed header; selecting display
text naturally selects the normalized presentation. This preserves the independent
host/source-copy contract while matching Swift's compact inspector signature.
No syntax tree, navigation coordinate, new state/job, public API or dependency.
The limiter's visibility becomes crate-internal so both existing render paths use
one byte/line policy. Font choice, weight, size and region geometry stay unchanged;
the required design skills are unavailable as recorded in the preceding audit.

## Assertion coverage

The new test parses Swift's impl/inline/open/Unicode-comment source and locates
open from its body. Actual keyboard disclosure paints exactly:

```text
fn open(
    &self,
    name: &str, // ✓ ünïcode
) -> Result<(), E>
```

It runs at both scales/themes and with LF/CRLF source, requires unchanged raw
header/source after drawing, and verifies that the byte just after the method's
closing brace belongs to the enclosing implementation. The body and attribute
cannot appear in the displayed header. Existing copy/menu/keyboard/link tests
now impose the signature budget while still checking >4096-byte raw copy text,
eight visible rows and the original exact navigation destinations. Shared semantic
checks continue to cover scalar-safe non-ASCII truncation and the eight-line cap.

## Descriptor and package-label mapping

ContextModelTests.escapedImplDescriptorsAndURLVersionsReadCleanly is now covered
by the existing semantic Context table with Swift's exact symbols. The escaped
slice/impl#[`[T]`]first descriptor yields first and package core without its URL
version. serde_json retains numeric version 1.0.151 and displays Value. local 4
has no package label and retains the source-token name. All cases require zero
invented definitions/destinations. This is a synthetic SCIP table through the real
index-to-Context preparation path; it does not claim new compiler-generated output.
The existing header/descriptor parser already satisfies the cases, so only the
assertions change. Doubled-space package and ordinary-version cases stay in place.

## Gates and structural result

Archive: `~/.local/state/radiology-qa/context-signature-20261109-e86dbd3/`.
All five declaration-summary checks and all five semantic Context checks pass.
Strict all-target Clippy, ordinary build and warning-free docs also pass.
The complete locked/offline workspace run passes 887 app tests (eleven existing
opt-in ignores), two binary/13 font/22 text tests and all 66 doctests. Compilation
uses -j 1 and serial tests; ordinary half-core jobs=16 is unchanged. The descriptor
table was extended afterward, with final focused semantic checks and strict
all-target Clippy passing. Production code matches the full-run snapshot.

Measured source delta: +67 NLOC/two functions/no files. The private formatter and
one regression account for the additions; the existing limiter gains only internal
visibility. Average/max CCN remain 3.42/132, high-complexity sites stay at 36 and
dependency cycles are unchanged. Comparator classification is NEUTRAL with 125
clone instances before and after. Equal-ranked Python chokepoint exchanges concern
unmodified files. Complete content/multiplicity comparison is archived separately.
The extra markdown clone compared with the preceding checkpoint's pre-doc report
is existing repeated historical next-step prose in PROGRESS, not signature code.

Still unaudited in ContextModel: stale publication; repository/reset retirement;
new index ownership; same-index changed bytes; explicit references/Follow Caret;
scalar-window reference excerpts; file coverage;
and expansion identity. Reference/call-tree/palette and the other indexed suites,
compiler/platform validation and final native/standalone acceptance also remain.
The documented Android license gate and parallel timing/linking limitations are
unchanged; full accessibility and sibling-app adoption are deferred.
