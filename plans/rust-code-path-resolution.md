# Code Paths source resolution and native C++ structure

The Code Paths library now resolves declarations and conditional/assembly variants
against owned source snapshots. All91 authored seL4 steps resolve at the reference
commit. The app's C++ reader also uses real C++ syntax. Code Paths import/sidebar
presentation and app request/history guards remain required; this is not a completed
Code Paths UI or a completed indexing-personality feature.

## Source and selector contract

code_paths::resolver::resolve_step validates the record and normalizes accepted dot/
repeated-slash filenames to canonical repository-relative names. A matching displayed
buffer is authoritative. Other checkout reads use existing containment, bounded UTF-8
source reads and cancellation. Reviews explicitly use pinned After objects; a displayed
Before buffer cannot replace them. A Before-only buffer fails. Missing/binary/external
symlink source fails without a text-search fallback. ResolvedStep owns the exact buffer,
its already-computed DocumentSyntax and source-ordered destinations with full-source
digests, bytes and display lines, ready for later app publication reuse.

The pure destinations API requires structure from that same source snapshot. Bare
names, enclosing ::/dot suffixes and exact whitespace-normalized declaration headers
follow the Swift contract. C++ out-of-class qualifiers are recognized. Real function
bodies and recovered function heads outrank matching prototypes; closures do not
count as function-definition evidence. Ordinary ambiguity fails. Parsed declarations
are used independently of SCIP; approximate fallback outlines are refused.

Opposing branches of a shared #if/#ifdef/#ifndef group can produce alternatives.
Every pair must be opposed; unrelated groups, repeated declarations in one branch
and ordinary overloads cannot silently choose a winner. Comments/strings are masked
without moving bytes/newlines, including C++ raw strings, before branch classification.
This is not active-build preprocessing. Assembly .S/.s/.asm recognizes explicit
labels and BEGIN_FUNC/ENTRY/ENTRY_NP/SYM_FUNC_START/SYM_CODE_START, excluding calls,
comments/strings and END_FUNC. Cancellation prevents partial results.

## Shared parser repair

C++ previously used the C grammar. The reader now links tree-sitter-cpp0.23.4 and
registers a separate Cpp grammar, using the bundled C queries plus Swift's C++ overlays
for calls/scopes/folds/symbols/highlights. Only that grammar package was added; the
existing Tree-sitter runtime remains in use. The [official grammar manifest](https://raw.githubusercontent.com/tree-sitter/tree-sitter-cpp/master/Cargo.toml)
was checked before selecting the dependency; Cargo.lock pins the downloaded release.

C and C++ outline extraction now share the native query path. A single declarator-
spine walker chooses the real name token, preserving pointer-return functions and
aliases while rejecting function-pointer variables as functions. Namespace/class/
method/prototype structure reaches the existing reader, picker and fold navigation.
The obsolete C-only collector and dispatch wrapper were removed. The existing
cancellation test invokes the shared extractor directly.

For errored C/C++ trees, recognized function declarators immediately followed by a
body opener are retained in DocumentSyntax.recovered_declarations. That navigation-
only field does not create outline rows, callable scopes or fold extents. It recovers
configuration-dependent brace cases such as create_idle_thread while refusing calls,
prototypes and pointer variables. All facts come from the same accepted source parse.

## Evidence and limits

- Twelve resolver tests port Swift's language/overload/qualified/conditional/assembly/
  malformed-body fixtures and add raw-string, Unicode/CRLF, cancellation, authoritative
  buffer, pinned After and unsafe/missing/binary source cases. A Python reassignment
  case proves closures cannot win declaration ambiguity as function bodies.
- Actual app frames prove C++ namespace/class/lambda facts, exact name-byte outline
  commits and destination-body expansion. The initial test incorrectly assumed End
  selected the last source function; it now respects the existing functions-first
  ordering and navigates to the observed main row.
- Full workspace:697 app tests pass, seven existing ignored; two binary/13 font/21
  text tests and54 doctests pass. Strict Clippy/check/build/rustdoc/fmt/diff gates pass.
  Public resolver/owned-result API contracts and runnable examples inspected.
- The external probe is reproducible with cargo run --example code_path_probe -- ROOT.
  It ran against clean seL4 d419a119a61a9b42edf61fb79ad08de45248daa7:91/91 steps,
  93 destinations in22 files. map_kernel_window and activate_kernel_vspace each have
  two conditional alternatives. Independent checks match every destination digest
  and byte/line coordinate to its Git blob, plus bare-selector spelling. This proves
  source resolution, not app step-list navigation, Back or conditional-menu behavior.
- Native C++1×/2× dark/light checks use the real app and a compiler-checked C++17
  fixture. Go to Symbol opens main; Shift-selection/Copy proves the exact four-byte
  destination token, then whole-source Copy matches all360 original Unicode bytes.
  Captures inspected; actual light background checked, hashes preserved, owned child
  exits zero. No native Code Paths UI acceptance is claimed.
- Failed driver attempts remain: long private Niri socket paths needed a shorter
  runtime directory, initial captures needed source-publication waiting, footer OCR
  read18 as1s, and the theme shortcut needed a painted frame after Go to Line closed.
  Accepted runs use token-byte verification and actual palette pixels, not those
  failed or insufficient checks. Existing design/typography skills remain absent;
  no new font scale or custom UI surface was introduced by this unit.

QA: ~/.cache/radiology-qa-code-path-resolution; accepted native outputs are copied
from ~/.cache/rcpq/v5-{1.0,2.0}-after into that evidence directory.
Archive: ~/.local/state/radiology-qa/code-path-resolution-20261102-3f43ee9.
The seL4 QA checkout lives outside user repositories at ~/.cache/radiology-qa-sel4-source.

## Architecture review

Baseline before-code-path-resolution at3f43ee9. Reuse searches found native syntax,
callable scopes, canonical source reads and shared query resources. Definition-body
ends are computed once rather than repeatedly scanning scopes. Lexical helpers accept
an error-typed cancellation callback, avoiding a helper/resolver dependency cycle.
A first16-CCN declaration matcher and duplicated definition checks were simplified.

| Dimension | Before | After |
| --- | --- | --- |
| Analyzed NLOC |97,205 |98,143 (+938, including tests/fixture/probe) |
| Functions /files |4,078 /624 |4,102 /631 |
| Average /maximum CCN |3.38 /132 |3.39 /132 |
| High-complexity functions |33 |33 |
| Clone groups /cycles |126 /1 |126 /1 |

The raw comparator reports REGRESSION because pre-existing clones touch the modified
outline/app-test files and untouched Python chokepoints change ranking. Normalized
full path/fragment/token audit proves zero added/removed clone identities. No new
hotspot or cycle remains. No new worker pool or source-navigation subsystem was added.

Next: bind asynchronous resolution to app root/review/navigation/caret/trace ownership;
reuse returned source/structure; add the left-sidebar trace picker/steps/conditional
menu and import/copy/delete/shortcuts, then run app-level91-step navigation acceptance.
