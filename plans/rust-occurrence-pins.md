# Rust occurrence pin parity

## Session store (2026-10-28)

The store supports unlimited groups. New groups advance a palette slot; removal
never recolors survivors or evicts older pins. Colors may repeat. The active
source's groups are retained by position key (path plus checkout/Before/After)
and exact SHA-256 content identity. Returning to identical bytes restores them;
changed content discards the old pins. Ephemeral highlights never cross documents.
Repository/review reset clears the session; this is in-memory state, matching the
Swift pin store, not disk persistence.

Ranges are sorted/deduplicated, and equivalent ranges reconcile lexical groups
that later acquire semantic identity. Group identity and display spelling are
separate. Clear Current preserves other documents and color progression; Clear
resets the entire repository session.

Backend regression tests cover twelve groups without eviction, stable colors,
restoration/revision/content isolation, changed-then-reverted bytes, ephemeral
reset and lexical/semantic range reconciliation. Reader activation and compact
remove/clear controls follow in the next commit. AST-aware fallback occurrence
scope and exact PR-refresh restoration remain separate open requirements.
