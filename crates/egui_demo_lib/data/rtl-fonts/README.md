# RTL demo font fixtures

These files are vendored only for `egui_demo_lib` demos and tests. They do not
change epaint's production default fonts. `include_bytes!` supplies identical
bytes to native and wasm builds, so the demo never depends on host-installed
fonts or a runtime download.

## Manifest

| File | Version | SHA-256 | Immutable source | License |
| --- | --- | --- | --- | --- |
| `NotoSansArabic-Regular.ttf` | Noto Sans Arabic 2.013, hinted static TTF | `bdff3e5659d67e67def05b33f749683b9376ae819d65d3dd62ac4640b3aaef48` | [`NotoSansArabic-v2.013.zip`](https://github.com/notofonts/arabic/releases/download/NotoSansArabic-v2.013/NotoSansArabic-v2.013.zip), archive SHA-256 `1301aceaea84c501cf2e6dcfb3182e2328c8eae5725817fcb239672bda7154f1`, path `NotoSansArabic/hinted/ttf/NotoSansArabic-Regular.ttf` | SIL Open Font License 1.1; see `OFL-NotoSansArabic.txt` |
| `NotoSansHebrew-Regular.ttf` | Noto Sans Hebrew 3.001, hinted static TTF | `cdefaf8efd47045f6820928eba84db5bed7557539328952b5f828315485e02ee` | [`NotoSansHebrew-v3.001.zip`](https://github.com/notofonts/hebrew/releases/download/NotoSansHebrew-v3.001/NotoSansHebrew-v3.001.zip), archive SHA-256 `df0a71814b4e63644cf40fcc4529111b61266b7a2dafbe95068b29a7520cc3cb`, path `NotoSansHebrew/hinted/ttf/NotoSansHebrew-Regular.ttf` | SIL Open Font License 1.1; see `OFL-NotoSansHebrew.txt` |
| `NotoSans-Regular.ttf` | Noto Sans 2.015, hinted static TTF | `478c558ea716033cd60c03438f628dfa75694dcf6b5f6d505a2f05fd2b4f3823` | [`NotoSans-v2.015.zip`](https://github.com/notofonts/latin-greek-cyrillic/releases/download/NotoSans-v2.015/NotoSans-v2.015.zip), archive SHA-256 `0c34df072a3fa7efbb7cbf34950e1f971a4447cffe365d3a359e2d4089b958f5`, path `NotoSans/hinted/ttf/NotoSans-Regular.ttf` | SIL Open Font License 1.1; see `OFL-NotoSans.txt` |

Copyright and reserved-font-name notices are in the corresponding OFL files.
The fonts are unmodified and not subsetted. Redistribution is permitted under
the included OFL 1.1 terms; retain the font and license files together.

## Loading and fallback

Use `egui_demo_lib::rtl_font_fixtures::install(ctx)` once before showing the RTL
demo. It validates the embedded files and all positive sample cmap entries,
then installs the named `RTL demo font fallback` family in this order:

1. Noto Sans Arabic RTL fixture;
2. Noto Sans Hebrew RTL fixture;
3. Noto Sans RTL fixture for Latin text and common symbols;

The existing proportional family is left unchanged, so opening the RTL demo
does not change typography elsewhere in the demo application.

Together the fixtures cover Latin, European and Persian digits, punctuation,
Arabic, Persian, Urdu, Hebrew, and combining marks. `POSITIVE_SAMPLES` in
`src/rtl_font_fixtures.rs` is the canonical text corpus. Tests check both the
script-specific font and the complete fallback chain automatically.

`漢` is the intentional negative sample. Neither fixture contains it; it must
fall through to an existing font provider or render visibly as a missing glyph.
It must never be used to approve a positive screenshot. Empty, corrupt, or
insufficient fixture bytes return `RtlFontFixtureError` instead of silently
installing a font set whose snapshots could be replacement boxes.
