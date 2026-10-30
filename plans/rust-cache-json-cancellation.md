# Normalized semantic-cache JSON cancellation

Implementation: `f59932d`, pushed on 2026-10-30.

## Contract

Cache loads retain the existing bounded file read and format, then decode through
a cancellation-aware buffered reader. Each input read is at most 64 KiB, including
when serde traverses a single long string. Cancellation latches: serde may attempt
another read after an I/O error, and a retired reader must keep returning an error.
Trailing or truncated JSON remains invalid. UTF-8, escaped text and provenance
round-trip unchanged. Each cache document and occurrence is checked again during
validation; restoration uses the existing cancellable semantic-map constructor.

Serialization already checked the owning flag before writes. Writes are now
bounded to 64 KiB and final flush checks the flag before and after flushing.
The existing size limit, temporary-file guard, sync and final pre-publish check
remain in force. Cancellation removes incomplete temporary output and leaves the
accepted destination untouched. A subsequently cancelled semantic-map build can
still follow a successfully published complete cache; these are separate stages.

The implementation separates read/decode from validated semantic restoration,
and inventory/provenance validation from per-document range validation. It adds
no dependency or cache-format migration. The application remains standalone.

## Verification

Three tests cover all 14 decode checkpoints in a large nested Unicode/escaped
string fixture, fresh exact-field recovery, trailing/truncated JSON, chunked writes,
size-limit errors and cancellation during final flush. A separate real-file path
within these tests observes temporary serialization output before setting the flag,
requires an error, checks the original cache bytes and absence of temporary files,
then saves and loads a fresh replacement successfully. Existing cache tests retain
cross-repository isolation, stale-source rejection, partial coverage, invalid
provenance and failed-write behavior.

Full checks pass 353 app tests (one external-Clang fixture ignored), eight shared
reader-text tests and 30 doctests, formatting, strict Clippy, build and rustdoc.
Logs: `/tmp/radiology-cache-json-{targeted,all-tests,clippy,build,doc}.log`.

Architecture baseline `before-cache-json-cancellation` compares neutral with no
new problems, clone groups or cycles. Cache load/restore measure CCN 6/11 versus
previous load 15; validation/inventory measure 9/10 versus previous validation 14.
All cache functions remain below the CCN 15 hotspot threshold.

## Native acceptance

A private 1280×900 Wayland session opens an initial one-document SCIP import.
Reload begins a 72,000,898-byte normalized JSON cache containing one very large
metadata string. The source and inspector remain displayed during loading.
Explicit Cancel reports “Index loading cancelled · previous data retained” and
keeps semantic data available. Replacing that fixture with a 1,400-byte two-document
cache and reloading eventually accepts both documents. Its deliberately mismatched
fingerprint produces the correct “Stale cached index” label; source digests still
match the fixture files. The unchanged reader remains usable. The app, Weston and
Niri exit zero; pointer-helper termination is intentional.

Artifacts: `/tmp/radiology-cache-json-native/manifest.json`, `initial-index.png`,
`cache-loading.png`, `cache-cancelled.png` and `cache-recovered-later.png`.
The manifest hashes those captures, fixture/driver/path-helper sources, targeted
and full-test logs and the tested binary. App SHA-256:
`8050583b9a53a399ad8ccd76e0f2c09444ca5407951889a46af5ac97bb93252d`.
Driver: `/tmp/radiology-cache-json-native-driver.py`; fixture encoder:
`/tmp/radiology-cache-json-native-fixture.py`. The session isolates both configuration
and cache directories. Sibling capture helpers are QA tooling only.

The native sequence proves UI cancellation, retention and recovery; deterministic
backend tests independently prove cancellation during JSON traversal. No exact
native interruption checkpoint or recovery latency is claimed. Temporary artifacts
are local evidence, not repository assets.

## Remaining limits

Cancellation is cooperative. An individual allocation, completed-string UTF-8
validation, bounded inventory/plan comparison, hash operation, filesystem sync or
cleanup may finish before the next check. Buffered reader decoding favors bounded
interruption over the former slice decoder's throughput; this is background work.
Broader source/graph/context reuse, per-project/version-aware index provenance,
provisioning and PR lifecycle work remain on the
[full parity queue](rust-parity-implementation.md).
