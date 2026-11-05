# Unicode file search — 2026-11-05

## Reconciled behavior

Swift `QuickFileSearch.normalize` uses Foundation case-insensitive and
diacritic-insensitive folding with `en_US_POSIX`. Rust previously decomposed accents
and lowercased scalars. A new red regression proves that query `strasse` returns no
match for `src/Straße.rs`; lowercase conversion cannot supply Unicode full case
folding, including expansions and final-sigma equivalence.

The existing Rust normalizer now uses ICU CaseMapper's locale-independent full
folding, followed by the existing NFD/combining-mark filtering. Candidate text uses
the same normalizer for non-ASCII scalars, while ASCII retains its allocation-free
per-character path. Existing source-character boundaries still delimit camel-case
and path words; expansions append to their originating boundary. Ranking, original
path identity, worker cancellation, result limits, modal ownership and navigation
continue through the existing search and picker mechanisms.

The lockfile adds only `icu_casemap` and its compiled data at2.4.0, reusing the
existing ICU2.4 collections, properties and provider dependencies. There is no
handwritten folding table, new worker/cache, new public API or reader-text/font
coupling. Stable Rust's installed full-folding API remains unstable, so it is not
used. The library operation is documented in the
[ICU CaseMapper reference](https://docs.rs/icu_casemap/2.4.0/icu_casemap/struct.CaseMapperBorrowed.html#method.fold_string).
No claim is made that Linux executes Foundation or AppKit.

## Verification

Durable evidence:
`~/.local/state/radiology-qa/unicode-file-search-20261105-1652635`.
Temporary execution:
`~/.cache/radiology-qa-file-search-unicode`.

- Red/green search tests cover sharp-S expansion in query and candidate, final
  sigma, ffi ligature, long-s, full-path matching and decomposed accents. Existing
  tier ranking, cancellation and truthful capped-total tests pass.
- Actual-app raw Ctrl+K/Text/Return tests at1×/2× wait for async ranking/publication,
  verify original selected paths, source focus, exact UTF-8 Copy and unchanged files
  for five Unicode filename cases.
- Native private Wayland1×/2× checks open the same five paths through the real picker
  and keyboard, with exact clipboard hashes for every file. Queries/reader endpoints
  are captured. Real Git and controlled PR metadata isolate the fixture; attached
  file hashes, HEAD/refs/status and executable hashes remain unchanged; app and
  compositors exit zero. Some early opening captures precede asynchronous publication;
  final endpoint captures follow successful exact Copy.
- Locked workspace passes771 app tests/11 opt-in ignored, two binary,13 font,22 text
  and59 doctests. Strict all-target workspace Clippy, locked build, embedded resource
  smoke, format and diff checks pass. Cargo remains limited to16 of32 logical CPUs.
- Architecture baseline `before-file-search-unicode-fold`: +69 NLOC/two test
  functions, average/max CCN unchanged3.39/132, no new hotspot/cycle/structural finding.
  Complete normalized clone Counter:125 before/after, zero additions/removals,
 1335 duplicate lines. Raw diff-aware warnings refer to an unchanged old focus-test
  clone and nondeterministic ranking of untouched Swift Python chokepoints.

This closes a concrete Unicode quick-file mismatch. Remaining Swift fixture
reconciliation, SDK-backed Android success and the final standalone/library reuse
audit remain open. No UI layout/type-scale change was required.
