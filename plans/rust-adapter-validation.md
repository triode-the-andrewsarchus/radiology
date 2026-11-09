# Reject unknown configured indexing adapters

Default personality captures now reject adapter IDs outside the actual automatic
registry. Indexer::AUTOMATIC is the single enum-backed six-adapter list; the seL4
pipeline remains selectable only through its own personality. Unknown IDs are
sorted and deduplicated in the error, matching Swift's service-level validation.
Rust validates during Settings capture, before planning or any tool/cache work,
rather than allowing an apparently valid capture to silently select no projects.
Public signatures and valid settings/caches remain compatible.

The field-specific Root.indexers method owns both automatic-ID validation and the
existing seL4 exact-selection rule beside the other typed root readers. Capture
still owns TOML bytes/digest and immutable metadata. The raw TOML reader continues
to accept the exact Swift literal/comment/multiline-array example (a,b and c);
typed full-settings coverage now uses real adapter IDs. This preserves syntax
coverage rather than weakening it to make the semantic guard pass.

Every known automatic ID is accepted. The invalid-ID test checks multiple unknown
names, duplicate removal and scip-sel4 rejection under Default. An actual app case
opens with ghost plus a valid ID, checks visible settings failure/no plan/no tool
execution and unchanged user bytes, then repairs and reloads the same file to
produce the expected two-document index without replacing the reader buffer.

All twenty personality and eight lifecycle cases pass, plus strict workspace/all-
target Clippy, normal build, warning-free docs and the Settings doctest. Architecture
NEUTRAL: +84 measured NLOC/net one analyzed function, average/max CCN 3.43/132,
36 hotspots/one cycle and all 125 normalized clone instances unchanged (zero added/
removed). The initial capture hotspot was removed by placing the complete adapter
field policy alongside existing typed field readers; no metric-only wrapper or
new subsystem was added. Public AUTOMATIC documentation is generated successfully.

Evidence: ~/.local/state/radiology-qa/index-diagnostics-20261109-5f729cc/.
Latest full workspace gate remains 5f729cc (926 app tests/all 67 doctests). This
focused checkpoint precedes invocation diagnostics and its combined full gate;
no fresh full-run result is claimed here. Compiler/personality remains 17/35 pending
the rest of Indexing/Kotlin/PR reconciliation; inspector and final acceptance remain.
