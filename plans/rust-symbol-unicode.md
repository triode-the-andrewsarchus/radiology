# Workspace symbol Unicode matching

## Contract — 2026-11-02

The [local Swift ranker](../Sources/Radiology/Semantic/SymbolSearch.swift) searches
lowercased names using exact, prefix, component/initial and subsequence tiers.
Computed search keys now use NFC canonical equivalence. Prefixes and subsequences
compare complete extended grapheme clusters; neither cuts a skin-tone, keycap,
flag, variation-selector or ZWJ character apart. This follows Swift's
[prefix implementation](https://github.com/swiftlang/swift/blob/2b0c27dbdd76ed8b7b5b82bc52980f13bcb8a85d/stdlib/public/core/StringLegacy.swift#L124).

Ranking retains its bounded candidate inventory and existing grapheme-count
length criterion. Case-sensitive name and path tie breaks now use NFC keys,
matching Swift's [string comparison](https://github.com/swiftlang/swift/blob/2b0c27dbdd76ed8b7b5b82bc52980f13bcb8a85d/stdlib/public/core/StringComparison.swift#L99).
Canonical ties retain stable input order. Empty queries still return the original
inventory prefix. Response names, IDs, paths, lines and definition counts come
from the original entries, never from normalized keys.

Initials and component splits also operate on whole characters. One component
walker preserves both Swift strategies: dropping capital delimiters, or retaining
camel-case humps. Character number boundaries use the first scalar's Numeric_Type,
including CJK numerals and keycaps; multiscalar uppercase classification checks
canonical upper/lower conversions. These mirror the relevant
[Swift character properties](https://github.com/swiftlang/swift/blob/2b0c27dbdd76ed8b7b5b82bc52980f13bcb8a85d/stdlib/public/core/CharacterProperties.swift).
The existing locked ICU4X properties 2.4.0 package is now a direct dependency;
normalization and segmentation reuse existing dependencies. No package versions
or worker pools change, and no native ICU runtime is introduced.

Query trimming uses Unicode Zs and TAB, following
[Foundation whitespaces](https://developer.apple.com/documentation/foundation/characterset/whitespaces),
so newlines and zero-width spaces remain query characters. The picker uses the
same emptiness predicate for No Symbols versus No Matches. Matching does not strip
accents, perform compatibility normalization or apply full caseless folding.
File navigation's deliberately accent-insensitive fuzzy search keeps its own rules.

## Workspace verification (`94c8b74`)

Four new backend tests cover 24 tier/miss cases, 10 initials cases, canonical name/
path order and stable ties, and 12 trimming queries. Cases include bidirectional
NFC/NFD accents, Hangul Jamo, CJK numerals, keycaps, flags, skin tones, ZWJ families,
variation selectors, compatibility controls and multiscalar case properties.
Matched metadata must remain byte exact. The existing 4,096-entry bounded/full-sort
and cancellation matrix now compares NFC ranking keys. The existing picker scale
matrix also checks that a newline query shows No Matches rather than No Symbols.
An early full-width control was corrected to avoid an unrelated later fuzzy match.

Fresh workspace: 600 application tests passed, seven ignored; two binary, four
font, 13 text and 37 doctests passed. Strict Clippy, all-target check, binary build,
rustdoc with warnings denied, formatting and diff checks pass. A copied standalone
binary passes resource checks in a fresh directory. The Swift runtime is not
installed here; the contract comes from the repository Swift source and pinned
standard-library sources, not a claimed local Swift execution.

Native v1 control at 1× uses the previous accepted binary (2eb85f4 source) and
reproduces only one canonical spelling per query and acceptance of partial family/
keycap characters. Final v1 repaired runs at 1×/2× accept both spellings, select the
canonical first path, reject partial characters, accept whole families/keycaps and
support light-theme pointer Go. Unicode queries are pasted through the private
Wayland clipboard. Five exact line copies per run verify source destinations,
including original NFD bytes in CRLF files. Captures are inspected; all fixture/
binary hashes remain unchanged and all child exits are zero.

Architecture against `before-symbol-unicode-matching`: default REGRESSION flags
old clone locations in the edited picker. Full normalized audit proves all 109
clone groups unchanged. +213 NLOC/11 functions, no new hotspot or cycle; new core
functions have CCN at most 11. The two component strategies share one walker.
Evidence: `~/.local/state/radiology-qa/symbol-unicode-20261102-2eb85f4`.

## Document fallback integration — 2026-11-02

The Rust-only document picker now shares the workspace query-key construction,
normalization and complete-character prefix primitive. Its substring scan starts
only at grapheme boundaries, preserving the previous document substring policy
and DFS declaration order without adding fuzzy ranking. Empty queries still
include every declaration. Query keys are computed once per worker request; raw
names and source byte destinations remain unchanged. There is no new dependency,
worker pool, public API or search subsystem.

A new real picker-frame matrix covers 13 queries: bidirectional canonical accents,
case/space normalization, interior substrings, partial/whole emoji and keycaps,
newline/no-match state, non-fuzzy misses, empty queries, original display names
and byte offsets. Return opens the first canonical match. Existing 1,000-row,
cancellation, index/source replacement and keyboard/pointer tests remain intact.
Fresh workspace: 601 application tests passed, seven ignored; two binary, four
font, 13 text and 37 doctests passed. All strict gates and copied standalone
resource verification pass.

Native document v1 old control at 1× reproduces one-spelling-only matching and
acceptance of a partial accent. Repaired runs at 1×/2× use actual unindexed Rust
outlines in CRLF source, find both spellings, reject the partial accent, and verify
Return, Escape, interior substring navigation, Back and light pointer Go. Six exact
line copies per run preserve NFD/NFC source bytes and caret destinations. Captures
are inspected; binary/fixture hashes stay unchanged and all child exits are zero.

Architecture against `before-document-symbol-unicode`: default REGRESSION flags
old clone locations; normalized audit keeps all 109 groups unchanged. +77 NLOC/
three functions (two shared helpers and one UI matrix), no new hotspot or cycle.
Evidence: `~/.local/state/radiology-qa/document-unicode-20261102-94c8b74`.

## Remaining scope

Workspace and document symbol matching now share the tested canonical-character
contract. The native keycap capture exposes a source shaping gap: the digit and
enclosing keycap glyph are drawn separately. The bundled emoji font contains
U+20E3 with nonzero advance, so this is a cluster-shaping issue, not evidence of a
missing fallback font. Matching/copy acceptance does not establish complete emoji
rendering. Broader corpus/UI and whole-application Swift parity remain open. Full
accessibility and actual sibling integration remain deferred.
