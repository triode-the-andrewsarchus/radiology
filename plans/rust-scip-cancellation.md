# SCIP decoding and semantic-map cancellation

Implementation: `e2a7f6f`, pushed on 2026-10-30.

## Contract

Import and generation jobs pass their cancellation predicate through the protobuf
reader and all nested readers. This covers documents, occurrences, relationships,
signatures, packed coordinates and unknown-field traversal. Strings copy in
64-KiB chunks with stop checks; complete UTF-8 validation preserves scalars that
cross chunk boundaries. Invalid lengths and overflowing varints return errors
without arithmetic overflow. Public synchronous decoding remains available.

Cancelled decoding returns a typed error, never a partial ScipIndex. Normalized
SemanticIndex construction checks documents, symbols, occurrences, fallback IDs,
derived definitions/references and search preparation before returning one complete
index. Import, generation and normalized-cache restoration use this construction
path. Existing document-local symbol scoping, external metadata precedence and
source encodings remain intact. Fallback symbols no longer require a cloned ID list.
The UI publishes accepted semantic data only after successful completion; cancelled
or failed replacement retains the previously accepted index.

Cancellation is cooperative. Individual UTF-8 validation, allocation, hashing,
cloning and sorting operations may finish before their following checkpoint.
Normalized-cache JSON serialization/deserialization still lacks internal stop
checks. Generation can save a complete decoded cache before semantic-map assembly;
cancellation during subsequent assembly therefore does not imply that the prior
persisted cache is unchanged. Partial semantic maps are never published.

## Verification

Full checks: 350 app tests pass (one external-Clang fixture ignored), eight shared
reader-text tests and 30 doctests; formatting, strict Clippy, build and rustdoc pass.
New deterministic tests stop at every checkpoint of nested decoding and all 146
map-construction checkpoints, then require a fresh successful result. Large packed
arrays and UTF-8 copies stop within their nested work; malformed length/varint
fixtures return errors. The reader retains Send + Sync compatibility.

Targeted runtime logs:
`/tmp/radiology-scip-decode-runtime.log` and
`/tmp/radiology-scip-map-runtime.log`.

Architecture baseline `before-scip-cancellation` compares neutral with no new clone
or dependency-cycle growth. Cohesive map-construction stages measure CCN 13/12/9
against the previous constructor's 21. The renamed root decoder retains its
baseline CCN 11 and cognitive complexity 34; its report flag is existing complexity.

## Native acceptance

A private 1280×900 Wayland session runs the standalone debug app against a temporary
fixture. Initial import accepts one document. Reload starts a 58,500,052-byte SCIP
file with 1,500,000 reference occurrences; the loading capture retains the existing
source and inspector. Explicit Cancel changes the footer to “Index import cancelled”
and “Previous semantic data retained · freshness unknown”, with semantic data still
available. A fresh 100-byte reload accepts two documents and keeps the reader usable.
The app, Weston and Niri exit zero. Pointer helper termination is intentional.

Artifacts under `/tmp/radiology-scip-cancel-native/`:
`initial-index.png`, `large-loading.png`, `cancelled-index.png`,
`recovered-index.png` and `manifest.json`. The manifest hashes captures, driver,
fixture encoder, targeted runtime logs and the tested binary. App SHA-256:
`82dd1aeedf31031e63b4e85439bd28422525bdf4be39b47896f17d122d0c37c9`.
Driver: `/tmp/radiology-scip-cancel-driver.py`; fixture encoder:
`/tmp/radiology-scip-native-fixture.py`. Capture helpers from little-screen-clip are
QA tooling only; the standalone app acquires no sibling runtime dependency.

This verifies UI retention and recovery; deterministic backend tests prove nested
interruption separately. It does not establish which decoding checkpoint the
native Cancel reached. Temporary artifacts are local evidence, not repository assets.

Full parity remains open. The current queue is maintained in
[rust-parity-implementation.md](rust-parity-implementation.md).
