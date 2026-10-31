# Real indexer acceptance

## Shared lifecycle coverage — 2026-10-30

Four opt-in tests in `src/semantic/indexing/service/tests/real_tools.rs` pass
against actual tools: rust-analyzer 1.98.1, scip-typescript 0.4.0, scip-go 0.2.7,
and scip-python 0.6.6. Go, TypeScript and Python use the production pinned
installer and verify the resulting managed executable. Rust uses an explicitly
selected real component from a private QA toolchain; this does not exercise
installation into the user's active rustup toolchain.

One shared assertion path checks production generation, complete expected document
coverage, cross-file definition/reference navigation, exact encoding-aware byte
ranges, version-qualified Update reuse, asynchronous cache-only reload with no
tools configured, source digest validation, unchanged viewed files/inventory, and
cleanup of private generation workspaces. The caller fixtures contain emoji before
the call for Go, Python and TypeScript; TypeScript also uses CRLF. Rust exercises a
feature-gated module requiring the nested-workspace feature/cfg parity behavior.
Fixtures have paths containing spaces and a literal semicolon.

Go reports two documents using UTF-8 columns. Python reports two documents using
UTF-16 columns. Both have zero missing sources and no partial/stale/cache-warning
state. Python retains the real tool's nonfatal warning that pyproject.toml lacks a
`[tool.pyright]` section. All viewed input bytes and the discovered inventory stay
unchanged; no Go module artifacts or Python environment/cache directories appear
in the viewed repository. Temporary installed tools are removed with each fixture.

Run explicitly, with network access and language prerequisites available:

```sh
GOMAXPROCS=16 \
RADIOLOGY_REAL_RUST_ANALYZER=/path/to/private/toolchain/bin/rust-analyzer \
cargo test --lib semantic::indexing::service::tests::real_tools -- --ignored --nocapture
```

The four-case run passes after sharing the installer polling and lifecycle checks.
Environment: Go 1.27.1-X:nodwarf5, Python 3.14.8/pip 26.2.2, Node 26.12.1/npm
12.2.1. Logs: `/tmp/radiology-real-go.log`, `/tmp/radiology-real-python.log`, and
`/tmp/radiology-real-four-indexers.log`. Architecture comparison against
`before-real-go-acceptance` is neutral: no new hotspots, cycles or clone groups;
duplication remains 0.57% and 90 clone groups.

## Limits and remaining acceptance

These are small real-tool projects, not evidence for dependency-heavy or broad
mixed-language corpora. They exercise backend production paths, not new native UI
interactions. Kotlin/Java and actual scip-clang acceptance remain open, as do the
broader recovery, retention and final visual/accessibility/scaling/font/sibling
reuse audits. See [implementation queue](rust-parity-implementation.md),
[provisioning](rust-tool-provisioning.md) and
[Rust feature acceptance](rust-feature-configuration.md).
