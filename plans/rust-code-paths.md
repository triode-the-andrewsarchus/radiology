# Rust Code Paths parity

Current checkpoint: the reusable text-format and private-storage foundation is
implemented. There is no Code Paths sidebar/import UI or symbol navigation yet.
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

## Required remaining work

1. Resolve names, enclosing ::/dot paths and exact normalized declaration headers
   against actual parsed source without an index. Prefer real definitions to
   prototypes, report ordinary overload ambiguity, and preserve byte destinations
   after inserted/deleted lines. The current Rust C++ registry uses the C grammar;
   verify proper C++ declaration/scope fidelity against Swift fixtures and repair
   that foundation where needed rather than reducing qualified-name support.
2. Recognize assembly labels and the five documented entry macros while excluding
   comments/strings/calls/END_FUNC. Port opposing-preprocessor-branch classification
   and selectable conditional variants. Recover recognized C/C++ declarator heads
   before bodies with conditional braces without inventing outline/fold/call extents.
3. Bind cancellable resolution to root/review/navigation/caret/trace ownership.
   Reuse authoritative open buffers, canonical checkout containment and pinned After
   reads. Preserve exact Before/After departure history and reject late success/error.
4. Add the left-sidebar peer tab, saved-trace picker, numbered wrapping steps,
   conditional-destination menu, Copy/Delete/import controls and trace-count footer.
   Preserve step state when switching tabs; import/show reveals only the left sidebar.
   Existing Context pane preferences and Bookmarks must retain their meanings.
5. Add pasted UTF-8/text-file import with bounded reading, atomic failure/error UI,
   ordinary editor Return behavior and IME ownership. Reconcile authored command
   shortcuts/help and narrow full-title/icon tab layouts with existing reader routing.
6. Verify real app frames, native1×/2×/dark/light, persistence/relaunch, source guards,
   exact Back, full Swift language/ambiguity/conditional/assembly fixtures and the
   user's91-step seL4 source-resolution/app probe when a checkout is available.

The standalone app and shared APIs remain the target. Actual sibling adoption and
full accessibility remain deferred; ordinary keyboard behavior is required.
