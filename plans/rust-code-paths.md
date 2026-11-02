# Rust Code Paths parity

Current checkpoint: the reusable text-format and private-storage foundation is
implemented. [Source resolution](rust-code-path-resolution.md) is now implemented too,
with real91-step seL4 acceptance. [App step navigation](rust-code-path-app.md) now
passes the real91-step/93-destination source/history probe with ownership guards.
The [standalone sidebar/import/persistence workflow](rust-code-path-sidebar.md) is now
implemented, with native1×/2× dark/light/relaunch and exact authored-text copying.
The full feature remains required by the upstream parity ledger. The behavioral
reference is [code-paths.md](code-paths.md) and the current Swift CodePaths source,
not the earlier line-number-based proposal.

## Accepted format and storage

`code_paths::CodePath::parse` atomically imports a freeform first-line title and
ordered file:function:description records. Only the first two unescaped colons
split fields. In filenames/selectors, backslash-colon and double backslash decode
one literal character; unknown/trailing escapes stay literal. Descriptions consume
the rest of the physical line unchanged except surrounding whitespace. LF, CRLF,
CR, a title BOM and blank post-title lines match Swift. Original physical line
numbers remain in typed diagnostics; the2MiB limit counts original UTF-8 bytes.
No source coordinates are saved or guessed. Missing files/selectors are a later
resolution concern, not an import-time text-search fallback.

Stable trace/step UUIDs persist, while Copy/export produces import text with fresh
identities on reimport. Same-title traces and repeated destinations remain distinct.
Validity matches Swift: nonempty title/records/fields, no absolute or parent-traversing
or NUL filename, and unique step IDs. Dot components/repeated slashes are retained
by import; resolution must canonicalize them without changing authored copy text.

CodePathStore accepts an app-owned absolute directory outside source and a canonical
attached checkout root. PR integration must use that same attached root. SHA-256
root keys select independent version1 JSON archives; archive ownership is verified.
Missing loads write nothing. Valid records retain IDs/order; invalid record values
and repeated trace IDs are filtered as in Swift. Corrupt JSON, unsupported version,
foreign ownership and I/O remain typed visible errors. Save checks existing storage
before replacement, so import/deletion cannot overwrite a corrupt archive. Other
checkouts have independent files. New data serializes borrowed records without
cloning the entire trace collection. There is no new arbitrary archive-total cap
that would turn multiple valid2MiB imports into a smaller feature.

The store is a synchronous library API: the app integration must schedule file I/O
outside rendering and serialize mutations for one repository. No cross-process
transaction guarantee is claimed. App state must publish import/delete only after a
successful save and retain the accepted selection/data on failure.

## Architecture and validation

Existing canonical private-destination validation, no-follow regular-file opening,
UUID/serde/SHA-256 and atomic temporary-file mechanisms are reused. A single private
storage::atomic_write now serves this store and Critical Lens overrides, retaining
sync-before-rename and existing override save ordering. The initial13-line atomic
write clone was removed through that shared operation; no unrelated stores changed.
No dependency, worker pool, UI framework or source parser was added.

Baseline before-code-paths at d0da261: +450 analyzed NLOC,23 functions and six files;
average/max CCN3.38/132,33 high-complexity functions,126 clone groups and one dependency
cycle unchanged. Architecture result NEUTRAL. Chokepoint ties continue to alternate
among untouched Swift-side Python functions; no new source hotspot/cycle/clone.

Nine new regressions cover escaped Unicode fields/literal descriptions, unknown
escapes/newline styles, ordered repeats and fresh IDs, malformed atomic imports,
physical-line errors, exact UTF-8 size boundary, canonical aliases/scope isolation,
corrupt/foreign/unsupported storage, invalid/duplicate records, source-contained
settings and linked archive rejection. The checked-in upstream91-step seL4 trace
parses/exports/reimports with all authored destinations in order. This proves the
format; it is not a91-destination source-resolution or native UI acceptance claim.
Full workspace:684 app tests pass, seven existing ignored; two binary/13 font/21
text and52 doctests pass. Strict Clippy/check/build/rustdoc/fmt/diff checks pass.
Rendered public format/store/error APIs inspected; existing Critical Lens tests
exercise the shared writer. An initial Clippy test-only clone-to-slice warning was
corrected using a borrowed slice.

Evidence: ~/.local/state/radiology-qa/code-paths-data-20261102-d0da261.

## Feature checkpoints and remaining acceptance

1. [Source resolution](rust-code-path-resolution.md) supplies qualified declarations,
   exact headers, definition preference, conditional/assembly alternatives and C/C++
   body-head recovery. The reader uses a dedicated C++ grammar.
2. The reference seL4 corpus passes91/91 steps and93 Git-verified destinations.
   [App navigation/history](rust-code-path-app.md) also passes those destinations and
   exact Back, with queued-result and same-batch caret retirement coverage.
3. [Standalone presentation and saved collections](rust-code-path-sidebar.md) provide
   serialized background mutations, a peer tab, trace picker, numbered wrapping steps,
   variants, Copy/Delete, pasted/text-file import and import/show shortcuts.
4. Native1×/2× dark/light/relaunch checks verify pasted import, step/variant navigation,
   Back, exact Unicode text export and persistent identities. Synthetic editor Return,
   IME/Tab ownership and regular UTF-8 file byte bounds pass automated tests.
5. Desktop-portal chooser and actual IME-engine/candidate-window acceptance remain
   environment-specific. Native PR saved-trace and broader unusual-input stress QA
   remain follow-ups; the port's other indexing/help parity work is still required.

The standalone app and shared APIs remain the target. Actual sibling adoption and
full accessibility remain deferred; ordinary keyboard behavior is required.
