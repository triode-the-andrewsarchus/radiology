# Rust trait context identity

The reusable resolver and app lifecycle/selection/presentation/navigation are now
implemented. See the [inspector contract](rust-trait-inspector.md) for current app
and native acceptance; the resolver checkpoint below remains historical evidence.

`semantic::trait_context::trait_at_byte` accepts an immutable source buffer, its
already-parsed DocumentSyntax, the accepted SemanticIndex, canonical caret byte and
cancellation flag. It does not read or parse files. The host must pair syntax with
its exact source snapshot and retire work when repository/index/source changes.

Resolution follows the Swift priority: a trait/interface occurrence (SCIP42/53)
under the caret wins; otherwise resolve the innermost enclosing parsed trait
(including requirements/default methods); otherwise resolve the innermost explicit
trait impl header. Declaration identity requires an exact definition-name range;
impl identity requires one distinct exact nonlocal nondefinition header-token ID.
Duplicate occurrences of the same identity are harmless; ambiguous identities cannot
pick an arbitrary row. Bounds/inherent impls cannot synthesize enclosing traits.
No name matching, relationship fallback or implementing-type identity is used.
Before revisions, stale source and non-Rust files have no context. Traits with zero
implementation rows can still have context. Unknown imported freshness retains the
existing index policy; this API does not manufacture provenance.

Tests reuse the real upstream compiler fixture: same-named left/right traits,
member/default bodies, aliases, qualified/blanket/negative impls, inherent methods
and zero-row Empty. Additional cases prove kind42/53 caret priority over an enclosing
trait, Unicode canonical coordinates, same-identity duplicates, ambiguity, invalid
roles/local impl identity, document-local trait definitions, stale source, Before, language gating and cancellation.
The runnable public example proves syntax alone cannot manufacture semantic context.

Architecture baseline before-trait-context at b40280d: +261 NLOC, seven functions,
two files; avg/max CCN3.38/132,33 high-complexity functions,125 clone groups and one
cycle unchanged. Result NEUTRAL. Caret-kind selection and exact-token binding are
separate cohesive operations; source/index mapping and outline breadcrumbs reuse
existing mechanisms. No new dependency, parser, worker pool or UI abstraction.

Evidence: ~/.local/state/radiology-qa/trait-context-20261102-b40280d.
The later [inspector integration](rust-trait-inspector.md) connects this resolver to
accepted source/index lifecycle, retained same-trait selection, compact middle-slot
rows, ordinary keys and source-verified navigation/history guards. TraitStructure
also accepts separately retained outline/impl facts from the same source parse.

Validation:667 app tests pass with seven existing ignored, plus two binary/13 font/
21 text tests. The final local-declaration refinement also passes all three context
regressions. All49 workspace doctests and strict Clippy/check/build/rustdoc/fmt/diff
gates pass. Rendered function contract/example inspected. No app/native UI claim.
