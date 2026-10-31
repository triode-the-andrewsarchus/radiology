# reader-fonts

Portable font assets for independent readers and future little-screen-clip hosts.
There is no runtime dependency on another checkout, workbench, daemon or renderer.
By default the crate exports only static font bytes and has no normal dependencies.
Enable `egui` to call `append_fallbacks` on host-owned font definitions.

The optional registration adds CJK and monochrome emoji after existing faces in
all families, including custom host families. It preserves Latin choices and type
sizes, handles initially empty definitions and is idempotent. The host applies the
result with `Context::set_fonts` once during setup. This package does not set theme,
spacing, display scale or text roles.

## Asset provenance and licensing

The following files are copied byte for byte from little-screen-clip's existing
`crates/little-clip-render/assets/fonts`. No fonts were downloaded, regenerated or
subsetted for this extraction. Both original SIL OFL 1.1 notices ship alongside
the fonts; the Rust source uses this package's MIT OR Apache-2.0 license.

| Face | SHA-256 | Notice |
| --- | --- | --- |
| Noto Sans CJK JP Regular | `86f7ba3101617c899dd2ea44c51f0d58e58892709949c8619f5800ada8d9b601` | `assets/NotoSansCJK-OFL.txt` |
| Noto Emoji Regular | `8f685aeeef29fd7ab230e1b9aafda3bbc0e3c35c19553038e95d12e468338f07` | `assets/NotoEmoji-OFL.txt` |

The original host provenance records the CJK JP face extracted from Arch's
noto-fonts-cjk 20240730 collection. The emoji face is its static Regular instance
of Noto Emoji v3.002, pinned to Google Fonts commit
`8b0a1d0f5983c89bc2b93f1b5fb55f9e252744b5`. The two assets add about 16.6 MiB to
an executable that embeds both. They cover representative CJK and modern emoji,
not every Unicode character. Egui renders these as monochrome glyphs; this does
not add complex shaping or multi-codepoint emoji composition.

Radiology consumes this package now. The sibling still consumes its original
render-crate assets and workbench setup; migrating it to this package remains a
future integration step. These applications do not yet consume one common crate.
