# Semantic caret identity corrections

Reference: the caret, shorthand-field, occurrence-selection and malformed-data
assertions in `SemanticIndexTests`, inspected against Rust `ae030f0`. This is a
focused correction during the semantic audit, not completion of that suite or
the Context/navigation suites.

## Confirmed mismatches and behavior

`index-red.log` records three failing regressions. The same checked-in Rust-analyzer
fixture used by Swift selected local 0 for Circle { radius } instead of the
repository field. A synthetic equal-range case failed too, and an escaping source
path was accepted by the public index constructor.

`SemanticIndex::occurrence_at_byte` now considers nonempty-symbol, single-line
occurrences. Narrowest original-byte range wins; equal ranges prefer a definition
destination other than a definition the caret already occupies, then a global
identity over a document-local identity. This matches Swift's navigation/global
preference and prevents a whole-file module occurrence from becoming a caret or
highlight target on punctuation. The legacy encoded-coordinate occurrence method
remains a general covering-range lookup; no product path uses it for caret selection.

The constructor now ignores documents with escaping/noncanonical repository paths
before populating symbols, definitions, references, package ownership or search
entries. It reuses the canonical source-path check and rejects explicit empty/dot
components, matching the Swift constructor's path contract. The importer retains
its existing visible rejection of invalid index files; this change also protects
direct constructor callers without inventing index locations.

The existing app occurrence/pin fallback policy is preserved. Swift's AppModel
intentionally uses a function-local syntactic spelling group when SCIP does not
cover all local spellings, including cfg omissions and shadowed tokens. Semantic
caret/Context identity still resolves the repository field with its three indexed
occurrences; the broader decoration may include the same-spelled parameter. An
early draft incorrectly made semantic identity unconditional for decorations;
the existing cfg-pin test caught that loss of behavior. That app implementation
change was reverted. `app-red.log` belongs to that discarded unconditional-
semantic decoration draft; `app-final.log` records the corrected Swift-aligned
app regression. No pin, color or lexical fallback redesign is included.

## Verification scope

| Contract | Evidence |
| --- | --- |
| `shorthandFieldPrefersTheRepositorySymbolOverTheLocal` | The real fixture resolves the exact Circle#radius identity, its line-two definition and two reference sites. The new both-scale app test navigates the actual caret to shorthand and requires that same Context identity, reference count two, file-occurrence count three and destination. This complete semantic-index group is mapped. |
| Narrowest/tied caret preference | Matrix covers global/local with neither, either or both definitions, in both occurrence orders. A narrower local range still wins over a wider navigable global range. Real fixture Circle references resolve while the adjacent closing bracket has no caret target. Existing under-caret tests retain their destination/definition/import guards. |
| Decorations and file replacement | App regression preserves Swift's explicitly syntactic two-token shorthand/parameter decoration and pin group, clears whitespace selection and retires transient ranges on file changes. The bracket after Circle cannot fabricate a whole-file module highlight. The existing cfg-omission pin test passes unchanged. Original fixture bytes remain unchanged. |
| Invalid paths and derived data | Pure matrix covers parent traversal, absolute/empty paths, inner parents, dot aliases and repeated separators. Dropped documents contribute no symbols, definitions, references, search entries or repository packages. Existing strict loader error paths remain in place. |
| Reference fixture ownership | Two old reference tests depended on a local/global-overlapping declaration being selected as local. Queries now explicitly select the unambiguous local call site. The caller-only source invalidation test expands its actual reference row, whose enclosing caller has the intended c.rs edge, instead of hardcoding the overlapping declaration row. All ten reference-context tests pass with their ownership/invalidation assertions retained. |

The other `SemanticIndexTests` groups retain pending assertion-level reconciliation.
In particular, inspection found that Rust's raw SCIP decoder currently retains the
metadata text encoding but drops tool name/version/arguments and project root.
Swift retains those fields; add that to the next metadata/decoder checkpoint.
Do not mark the eleven-group suite complete based on the corrections here.
Context/reference model ownership, call-tree and integrated navigation audits also
remain open. Existing general encoding tests intentionally reject split scalars/
surrogates, rather than Swift's clamping conversion; retain that stricter indexed-
coordinate policy and document it in the eventual full comparison.

## Validation and architecture

Archive: `~/.local/state/radiology-qa/semantic-identity-20261109-ae030f0/`.
The three final index regressions, ten reference-context checks, both-scale app
regression and unchanged cfg-pin regression pass. Strict all-target Clippy,
normal build and warning-free API docs pass. The final locked offline workspace retry passes 862 app tests (11 existing ignored),
two binary/13 font/22 text tests and all 65 doctests. Compilation uses -j 1 and tests
run serially after the earlier unconfirmed parallel-link crash; normal half-core
Cargo configuration remains 16 jobs. The initial whole-app run caught the two ambiguous fixture queries
and draft cfg-decoration behavior loss; `workspace-initial-failure.log` preserves
that evidence. Failed local retries were resolved before starting the final run.

The LSP tool's update check succeeded, but its server/outline endpoint returned
HTTP 500; source inspection and executable regressions supply the evidence here.
No LSP result is claimed.

Baseline `before-semantic-index-contracts-20261109` to
`verified-semantic-identity-20261109`: +208 measured NLOC, four test functions/two
focused test files, no new production function or dependency. Average/max CCN
3.42/132, 36 hotspots and dependency cycles are unchanged. Complete normalized clone
multisets contain zero additions and one removed old reference-fixture clone
(123 to 122 instances). Reference fixtures now
select the intended real row; no metric-only extraction was used. Existing lookup,
definition, source-path, Context and pin mechanisms are reused. Raw comparator
warnings identify an old touched app-test clone and equal-score untouched Python
chokepoint rankings; the complete clone comparison confirms no new duplicate.

Overall app parity, remaining behavior suites, final native/standalone acceptance,
known parallel timing limits and Android remain open. Full accessibility and actual
sibling integration remain deferred; ordinary keyboard support remains required.
