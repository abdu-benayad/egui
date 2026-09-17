# RTL demo font fixtures

The two Noto files in this directory are vendored only for the RTL demo and its
tests. They do not change epaint's production default fonts. `include_bytes!`
supplies identical bytes to native and wasm builds, so the demo does not depend
on host-installed fonts or a runtime download.

The demo reuses `Ubuntu-Light.ttf` from `epaint_default_fonts` for Latin text,
digits, and common punctuation. It does not vendor a second Latin font.

## Manifest

| File | Version | SHA-256 | Immutable source | License |
| --- | --- | --- | --- | --- |
| `NotoSansArabic-Regular.ttf` | Noto Sans Arabic 2.013, hinted static TTF | `bdff3e5659d67e67def05b33f749683b9376ae819d65d3dd62ac4640b3aaef48` | [`NotoSansArabic-v2.013.zip`](https://github.com/notofonts/arabic/releases/download/NotoSansArabic-v2.013/NotoSansArabic-v2.013.zip), archive SHA-256 `1301aceaea84c501cf2e6dcfb3182e2328c8eae5725817fcb239672bda7154f1`, path `NotoSansArabic/hinted/ttf/NotoSansArabic-Regular.ttf` | SIL Open Font License 1.1; see `OFL-NotoSansArabic.txt` |
| `NotoSansHebrew-Regular.ttf` | Noto Sans Hebrew 3.001, hinted static TTF | `cdefaf8efd47045f6820928eba84db5bed7557539328952b5f828315485e02ee` | [`NotoSansHebrew-v3.001.zip`](https://github.com/notofonts/hebrew/releases/download/NotoSansHebrew-v3.001/NotoSansHebrew-v3.001.zip), archive SHA-256 `df0a71814b4e63644cf40fcc4529111b61266b7a2dafbe95068b29a7520cc3cb`, path `NotoSansHebrew/hinted/ttf/NotoSansHebrew-Regular.ttf` | SIL Open Font License 1.1; see `OFL-NotoSansHebrew.txt` |
| `epaint_default_fonts::UBUNTU_LIGHT` | repository-bundled Ubuntu Light | `80307b8da7649aa4ee4d484b232140e3ce1ec0ca093073d3c53c8f5a5ced7a70` | `crates/epaint_default_fonts/fonts/Ubuntu-Light.ttf` | Ubuntu Font Licence 1.0 in `epaint_default_fonts/fonts/UFL.txt` |

Copyright and reserved-font-name notices for the two vendored fonts are in the
corresponding OFL files. The fonts are unmodified and not subsetted. Retain each
font and its license file together.

## Loading and fallback

The demo installs a named `RTL demo font fallback` family in this order:

1. Noto Sans Arabic;
2. Noto Sans Hebrew;
3. the already-bundled Ubuntu Light font.

Installation uses `Context::add_font`, preserving every existing font
definition and family. A context marker makes repeated calls idempotent even
when they happen before pending fonts become active. The existing proportional
family remains unchanged, so enabling the RTL demo does not change typography
elsewhere in an embedding application.

`POSITIVE_SAMPLES` in `src/rtl_font_fixtures.rs` is the canonical text
corpus. Test-only `ttf-parser` checks the script-specific font and the complete
fallback family. Runtime parsing is unnecessary because the compiled-in assets
cannot be absent; a malformed or insufficient fixture fails the tests instead
of adding parser work to native and wasm demos.

`漢` is the intentional missing-glyph sample. None of the three fonts contains
it; it must fall through to an existing font provider or render visibly as a
missing glyph. It must never be used to approve a positive snapshot.
