# RTL validation checklist

This is the evidence record for [the RTL support matrix](rtl-support.md). A case
is **passed** only when its stated expected result was observed on the recorded
commit and environment. Missing runtime access is recorded as **untested**;
successful compilation is not interaction or rendering evidence.

## Revisions and delivery status

- Foundation behavior under review: `c93e8863e958507056415e2a6dace56e23ba5f3a`
- Executable probes and additive fixture installer: `7682d862c7e1e126265b348c9d114be3c14dcfc3`
- Interactive demo and captured screenshots: `5c35307d` (the additive installer
  change does not alter the fixture bytes or named demo family)
- Delivery stack: [PR #1](https://github.com/abdu-benayad/egui/pull/1), open and
  targeting `bidi-runs`; it is not merged upstream
- Foundation PR: draft [PR #8577](https://github.com/emilk/egui/pull/8577),
  also not merged upstream
- Four later local RTL fixes through `7d8d0870` are outside this baseline. The
  defects recorded here are expected baseline behavior, not regressions in those fixes.

The documentation describes foundation behavior at `c93e8863`. Commands for
the retained probes and fixture installer must be run at `7682d862` or a
descendant containing it. The final documentation-only revision may therefore
be newer than the code revision that produced the evidence.

## Environment

- Date: 2026-09-17
- Foundation branch: `bidi-runs`
- Upstream base: `7ba3dbc4b07e72dcc85697e8cb51dcc70b1c8a6f` (two commits behind the tested head)
- Source crate version: egui/epaint 0.36.2; no release contains the tested commits
- OS: Linux Lite 7.6 / Ubuntu 24.04.4 LTS, Linux 6.8.0-134-generic, x86_64
- Toolchain: `rustc 1.95.0 (59807616e 2026-04-14)`, `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- Browser: none
- Renderer: none; all executed cases were headless layout/mesh assertions
- Default features: `epaint/default_fonts` and `egui/default_fonts`
- Bidi/shaping dependencies: `unicode-bidi 0.3.18` with `hardcoded-data,std`; `harfrust 0.12.0`
- Bundled default font used by focused layout tests: Ubuntu Light,
  SHA-256 `80307b8da7649aa4ee4d484b232140e3ce1ec0ca093073d3c53c8f5a5ced7a70`.
  It has no Hebrew coverage in these tests, which therefore render `.notdef`.
- Probe Arabic font: Noto Naskh Arabic Regular, font version 131334,
  SHA-256 `8c67eaa2f0086872628fc3f7f760386f83c8be5c5e9a1ba3a38891237b2ac7cb`.
- Probe Hebrew font: Noto Sans Hebrew Regular, font version 196608,
  SHA-256 `436900d5ad77d33e4234247f3076eaecd25b92b9bea0519514f684140e3566a7`.

The Noto files above came from `/usr/share/fonts/truetype/noto/` and are not
repository fixtures. Their E2 results are valid for this machine and checksum
only. E5 records the separate portable fixtures used by demos and future
snapshots.

<a id="e1-bundled-font-bidi-layout"></a>
## E1: bundled-font bidi layout

Metadata: the full environment above; browser and renderer are not applicable;
Ubuntu Light is the font; default features were enabled.

Commands:

```sh
cargo test -p epaint bidi_ -- --nocapture
cargo test -p epaint rtl_ -- --nocapture
cargo test -p epaint grapheme_cluster_ -- --nocapture
cargo test -p epaint a_letter_the_font_composes_from_several_glyphs_is_one_glyph -- --nocapture
```

Expected: logical row text and character count survive simple bidi samples;
Hebrew plus digits and Latin plus Hebrew receive the expected visual x order;
pure RTL caret positions round-trip; the existing simple cluster invariants pass.

Observed: 1/1 bidi test, 2/2 RTL tests, 2/2 grapheme-cluster tests, and 1/1
composed-letter test passed. These are six distinct tests. The Hebrew samples
use `.notdef`, so the result supports bidi ordering and indexing, not Hebrew
font rendering. The executable evidence is in
[`text_layout.rs`](https://github.com/abdu-benayad/egui/blob/c93e8863e958507056415e2a6dace56e23ba5f3a/crates/epaint/src/text/text_layout.rs#L2693-L2871).

<a id="e2-real-font-shaping"></a>
## E2: real-font shaping

Metadata: the full environment above; browser and renderer are not applicable;
the two Noto files and checksums recorded above were loaded into otherwise empty
`FontDefinitions`; default features were enabled.

Method: an internal test probe used the existing `layout_simple` helper. It
loaded each font with `FontData::from_owned`, compared isolated Arabic `ب` with
joined `بب`, inspected logical characters, visual x positions, bidi levels, and
atlas rectangles, then laid out `ב 12` and `بِ`. The temporary probe was removed
after the results below were recorded; it changed no implementation behavior.

Expected: `بب` uses contextual forms and is placed RTL; `אב 12` keeps digits
LTR within an RTL paragraph; `بِ` reconstructs the original base and kasra.

Observed: contextual Arabic atlas rectangles differed from the isolated form;
both Arabic letters had odd bidi levels and the second logical letter sat to the
left. Hebrew glyphs were distinct and visual x positions were
`1=0`, `2=9`, space=`17`, bet=`21`, alef=`29`. The mark case failed:
`Row::text()` returned `"بب"` for source `"بِ"`; emitted glyphs were
`[('ب', x=15, advance=0, RTL), ('ب', x=4, advance=10.808001, RTL)]`.
The relevant implementation is
[`layout_shaped_run`](https://github.com/abdu-benayad/egui/blob/c93e8863e958507056415e2a6dace56e23ba5f3a/crates/epaint/src/text/text_layout.rs#L242-L295).

<a id="e3-negative-geometry-and-selection-probes"></a>
## E3: negative geometry and selection probes

Metadata: the full environment above; browser and renderer are not applicable;
Ubuntu Light and default features were used. The probes are checked in at
`e10ba24d`. They calculate the correct visual geometry, assert that the
foundation still differs, and print both values. This makes the known defects
reproducible while causing a future fix to require an evidence update.

Commands:

```sh
cargo test -p epaint rtl_support_probe_end_caret_mixed_row -- --nocapture
cargo test -p epaint rtl_support_probe_row_visual_bounds -- --nocapture
cargo test -p egui selecting_a_right_to_left_word_highlights_the_word -- --nocapture
```

Expected:

- the end cursor for `אב 12` equals the visual end of the last logical glyph;
- a pure RTL row's `rect_without_leading_space()` starts at the leftmost glyph;
- selecting logical range 4..7 in `abc אבג def` paints the Hebrew word's visual span.

Observed: all three commands ran one test and passed by reproducing the known
baseline mismatch. The mixed-row
end cursor was x=`32.96875` instead of x=`15.896`; the trimmed pure-RTL bounds
were `[[7, 0] - [14, 16]]` instead of `[[0, 0] - [14, 16]]`; the Hebrew-word
selection was `[46, 46]` instead of `[25, 46]`. The retained probes are in
[`text_layout.rs`](https://github.com/abdu-benayad/egui/blob/e10ba24df38c716cd8a40dbefc365d95bb58c18d/crates/epaint/src/text/text_layout.rs)
and
[`visuals.rs`](https://github.com/abdu-benayad/egui/blob/e10ba24df38c716cd8a40dbefc365d95bb58c18d/crates/egui/src/text_selection/visuals.rs).

<a id="e4-source-audit"></a>
## E4: source audit

Metadata: commit, date, OS, and toolchain are the environment above. Browser,
renderer, and font are not applicable to this source-only check. No feature gate
changes the audited fields or APIs.

Method and expected result: inspect the exact commit for an explicit paragraph
direction policy, logical start/end alignment, bidi caret affinity and visual
keyboard navigation, AccessKit text direction, and inherited widget mirroring.

Observed:

- paragraph direction is passed as `None` to `unicode_bidi::BidiInfo::new`, so it is automatic first-strong only;
- text alignment uses physical `Align` values;
- `Glyph` stores an embedding level, but cursor types expose no bidi affinity;
- AccessKit text runs are hard-coded to `TextDirection::LeftToRight`;
- `Layout::right_to_left` provides explicit physical widget placement, without an inherited text/container direction policy.

Pinned source: [`bidi_levels`](https://github.com/abdu-benayad/egui/blob/c93e8863e958507056415e2a6dace56e23ba5f3a/crates/epaint/src/text/text_layout.rs#L1698-L1711),
[`Glyph::bidi_level`](https://github.com/abdu-benayad/egui/blob/c93e8863e958507056415e2a6dace56e23ba5f3a/crates/epaint/src/text/text_layout_types.rs#L921-L960),
[`AccessKit` direction](https://github.com/abdu-benayad/egui/blob/c93e8863e958507056415e2a6dace56e23ba5f3a/crates/egui/src/text_selection/accesskit_text.rs#L102-L148),
and [`Layout::right_to_left`](https://github.com/emilk/egui/blob/7ba3dbc4b07e72dcc85697e8cb51dcc70b1c8a6f/crates/egui/src/layout.rs#L156-L164).

<a id="e5-reproducible-font-fixtures"></a>
## E5: reproducible font fixtures

Metadata: the date, OS, toolchain, and source baseline are the environment
above. The repository embeds unmodified static Noto Sans Arabic 2.013, Noto
Sans Hebrew 3.001, and Noto Sans 2.015 files in `egui_demo_lib`; the fixture
manifest records release archive URLs, paths, versions, font and archive
SHA-256 values, and the included SIL OFL 1.1 files.

Commands:

```sh
cargo test -p egui_demo_lib rtl_font_fixtures -- --nocapture
cargo package -p egui_demo_lib --allow-dirty --list
cargo check -p egui_demo_app --lib --target wasm32-unknown-unknown \
  --no-default-features --features web_app,wgpu
```

Expected: integrity and license hashes match; the named fonts cover the
Arabic, Persian, Urdu, Hebrew, Latin, digit, punctuation, and combining-mark
corpus; `漢` is absent from the fixture fallback chain; missing and corrupt
bytes return explicit errors; additive installation preserves a host's custom
fonts and families and is idempotent; the crate package contains every font,
license, manifest, and loader; the same `include_bytes!` loader compiles for wasm.

Observed: all 4 `rtl_font_fixtures` tests passed, including the preconfigured
host-family preservation test. The first coverage probe
correctly failed because the Arabic-specific font lacks Latin letters; adding
the separately pinned Noto Sans 2.015 fixture removed that false assumption.
The package listing contained all three TTFs, all three license files, the
manifest, and `rtl_font_fixtures.rs`. The exact wasm check passed. No browser or
pixel-rendering claim is derived from compilation.

<a id="e6-interactive-demo"></a>
## E6: interactive native and browser demo

Metadata: source baseline and fixture versions are recorded above; the demo
changes at `5c35307d` were run on 2026-09-17. Native was Linux x86_64 with wgpu/Vulkan on an
NVIDIA GeForce GTX 1050 Ti, NVIDIA driver 535.309.01. Browser was headless
Google Chrome 150.0.7871.128 on Linux using glow/WebGL 2 through ANGLE
SwiftShader. The wasm SHA-256 for that browser run was
`ffdedd27a62e17f56b2ddaca61b64928b5d8f518cfa9c6e30484a5e787736999`.

Commands and actions:

```sh
cargo test -p egui_demo_lib rtl_demo -- --nocapture
cargo run -p egui_demo_app
CARGO_TARGET_DIR=target bash scripts/build_demo_web.sh
bash scripts/start_server.sh
# Additional browser-rendered build after the exact wgpu build:
CARGO_TARGET_DIR=target bash scripts/build_demo_web.sh --glow
```

Open **RTL text**, inspect the display and layout samples, scroll to TextEdit,
then press Ctrl+A in the four-line editor. The same steps were sent to Chrome
through the DevTools input protocol. The exact wgpu web build passed after
`build_demo_web.sh` was corrected to keep a relative `CARGO_TARGET_DIR`
anchored at the workspace root. Headless Chrome could not present its WebGPU
swap-chain image, so the recorded browser interaction used the additional glow
build; this does not validate browser WebGPU rendering.

Expected: the pinned scripts render without positive-sample replacement boxes;
mixed Hebrew/European digits preserve internal direction; Ctrl+A selects every
logical character and paints every row completely; unsupported U+1AB0 remains
an explicit missing-glyph entry; physical alignment and widget order match the
selected controls.

Observed: all 3 `rtl_demo` tests passed. The layout probe recorded
`q\u{1AB0}\u{301}` as row text `qqq`, confirming the known character-identity
failure rather than accepting it. The event test confirmed the Ctrl+A logical
range covers the entire four-line string. Native and browser screenshots show
Arabic, Persian, Urdu, Hebrew, Latin, digits, punctuation, marks, physical
right alignment, truncation, and right-to-left widget placement. Both Ctrl+A
screenshots show incomplete mixed-bidi highlight geometry even though the
logical range is complete. Native and browser font providers supplied `漢`;
the deterministic provider-free snapshot shows the expected replacement box.

Artifacts:

- native display: [`rtl-demo-native-wgpu.png`](rtl-demo-native-wgpu.png), SHA-256 `6abc1331570a9beca4d8b0f9ba15f16b6afbd1d54c55ad7992ba620983c01ffb`;
- native Ctrl+A: [`rtl-demo-native-wgpu-select-all.png`](rtl-demo-native-wgpu-select-all.png), SHA-256 `30ac34c4275f3bdf28e8e848c51667225561336b72e65c05f235423f1f763249`;
- browser display: [`rtl-demo-web-glow.png`](rtl-demo-web-glow.png), SHA-256 `1ac8882797bc85c19dcc01851c879408551ddf4c4b319041ee3e26694b55fde2`;
- browser Ctrl+A: [`rtl-demo-web-glow-select-all.png`](rtl-demo-web-glow-select-all.png), SHA-256 `687be662c608c0229ca72f68cab14fe6014bd0fe9b7a86bd984381f479bdc94f`;
- provider-free snapshot: `crates/egui_demo_lib/tests/snapshots/rtl_demo/static.png`.

No runtime or wasm-size head-to-head measurement was performed. Browser WebGPU
rendering also remains unvalidated; this section records a successful wgpu
build and a separate Chrome glow/SwiftShader rendering session.

## Per-case checklist

| ID | Sample or operation | Exact steps | Expected result | Observed result | Status / method |
| --- | --- | --- | --- | --- | --- |
| V01 | `אב 12` | Run the layout test and inspect the pinned-font demo on native and browser. | Hebrew reads RTL; `12` reads LTR. | Expected x ordering passed with real Hebrew glyphs on both rendered platforms. | **Passed**, E1/E2/E6 |
| V02 | `ab אב` | Run `cargo test -p epaint rtl_rows_are_placed_in_visual_order -- --nocapture`. | Latin remains LTR; Hebrew word is visually RTL. | Passed with `.notdef` Hebrew glyphs. | **Passed for ordering**, E1 |
| V03 | `אב` pointer/caret | Run `cargo test -p epaint rtl_cursor_sits_on_the_right_side_of_its_glyph -- --nocapture`. | All three boundaries round-trip. | 1/1 passed. | **Passed**, E1 |
| V04 | End caret in `אב 12` | Run the first E3 regression probe. | End caret is at x=`15.896`, after logical `2`. | x=`32.96875`. | **Failed**, E3 |
| V05 | Select `אבג` in `abc אבג def` | Run the E3 range probe; the demo includes the same mixed line and exact drag instructions. | Highlight x span `[25, 46]`. | Probe produced zero-width `[46, 46]`; demo retains the failing case. | **Failed**, E3/E6 |
| V06 | Multiline RTL select-all | Focus the four-line native/browser demo editor and press Ctrl+A; compare logical range and pixels. | Every logical character is selected and every visual row is fully covered. | Automated logical range passed; native and Chrome pixels show incomplete row coverage. | **Failed for painting**, E6 |
| V07 | Arabic `ب` and `بب` | Run E2 with pinned Noto Naskh Arabic. | Joined text uses contextual forms and visual RTL order. | Passed. | **Passed for this font/sample**, E2 |
| V08 | Arabic mark `بِ` | Run E2 and compare `Row::text()` with source. | Exact source reconstruction and stable cluster geometry. | Returned `بب`; mark identity lost. | **Failed**, E2 |
| V09 | Dropped unsupported combining mark | Run `rtl_demo_unsupported_combining_mark_is_recorded` for `q\u{1AB0}\u{301}` and inspect the demo. | Text preserves all characters; unsupported mark is an explicit zero-width missing glyph. | Provider-free row text was `qqq`; source identity was lost. | **Failed**, E6 |
| <a id="v10-narrow-marked-wrapping"></a>V10 | Marked text under narrow wrapping | Resize the demo across `بِ بِ بِ — שָׁלוֹם שָׁלוֹם — q\u{1AB0}\u{301}` and inspect clusters. | Break only between grapheme clusters; each row retains its base and mark. | Display case and fixed-width snapshot exist; the Arabic/U+1AB0 identity failures prevent a pass. | **Failed for identity; wrapping partial**, E2/E6 |
| V11 | Spaced RTL selection | Raise the spacing slider, drag across the `אבג` TextEdit, and inspect the highlight. | One continuous visual span including spacing. | Interactive case is present; no durable pointer-drag capture was recorded. | **Untested interaction**, E6 |
| <a id="v12-wrapping-alignment-and-truncation"></a>V12 | RTL wrapping, physical alignment, justification, truncation | Use the demo's left/center/right controls, narrow marked line, styled bidi spans, and one-row ellipsis. | No cluster split or dropped run; physical alignment is exact and documented. | Native/browser and snapshot cover physical alignment, styling and truncation; cluster loss is reproduced; justification was not claimed. | **Partial**, E6 |
| V13 | Explicit LTR/RTL paragraph override | Search public layout API and try to force direction independent of content. | Stable public override. | No API exists. | **Unsupported**, E4 |
| V14 | Visual arrow navigation and ambiguous boundary | Move left/right through mixed Hebrew, Latin, and digits in `TextEdit`. | Movement follows visual order and preserves affinity. | No affinity model; interaction not run. | **Unsupported**, E4 |
| V15 | Inherited widget mirroring | Set application/container direction and inspect controls, icons, popups, and tables. | Descendants inherit direction and directional affordances mirror. | No inherited policy exists. | **Unsupported**, E4 |
| V16 | AccessKit RTL run | Build accessibility nodes for RTL and inspect direction/positions. | RTL direction and correct visual geometry. | Source always sets LTR. | **Unsupported**, E4 |
| <a id="v17-ime-and-platform-input"></a>V17 | RTL IME composition | On each platform, compose marked RTL text under narrow wrapping; inspect candidate location, selection, commit, and cancel. | Composition range and candidate geometry follow the visual caret without text corruption. | No platform run. | **Untested** |
| <a id="v18-native-platforms-and-renderers"></a>V18 | Native app and renderers | Run the demo on Linux, Windows, macOS, Android, and iOS with each supported renderer; capture matching-baseline screenshots. | Shaping, order, caret, selection, and decoration pixels match the checklist. | Linux wgpu/Vulkan was captured; other native platforms/renderers remain untested. | **Partial**, E6 |
| <a id="v19-wasm-and-browser"></a>V19 | wasm/browser | Build the web demo and execute the checklist in supported browsers. | Same text, caret, selection, IME, and font bytes as native. | Exact wgpu build passed; Chrome glow/SwiftShader rendered and Ctrl+A matched native logical and failing visual behavior. Browser WebGPU and other browsers remain untested. | **Partial**, E5/E6 |
| <a id="v20-uax-9-conformance"></a>V20 | UAX #9 conformance | Run pinned Unicode BidiTest.txt and BidiCharacterTest.txt through the layout resolver. | All declared-supported classes and paragraph levels pass; exclusions are enumerated. | No conformance runner or result. | **Untested** |
| <a id="v21-performance-and-wasm-size"></a>V21 | Performance and wasm size | Benchmark representative LTR, pure RTL, and mixed paragraphs against `7ba3dbc4`; compare stripped wasm artifacts with identical features. | Regression budgets and exact byte delta are recorded. | No benchmark or artifact measurement. | **Untested** |
| V22 | Rust documentation examples | Run `cargo test -p epaint -p egui --doc` at the exact commit. | All executable rustdoc examples compile and pass; pass count recorded. | 178 passed, 0 failed, 2 ignored (egui 172/0/1; epaint 6/0/1). | **Passed** |

## API documentation delivery

The canonical guide is [`docs/rtl.md`](rtl.md). The versioned matrix and this
report remain at [`docs/rtl-support.md`](rtl-support.md) and
`docs/rtl-validation.md`. Public rustdoc was updated in these exact sources:

- `crates/epaint/src/text/text_layout_types.rs`: automatic direction, physical alignment, logical glyph order, visual positions, bounds, hit-testing, and two executable examples;
- `crates/epaint/src/text/cursor.rs`: logical cursor semantics and the absence of bidi affinity;
- `crates/egui/src/layout.rs`: separation between text direction and physical widget placement;
- `crates/egui/src/text_selection/cursor_range.rs`: logical selection ranges and current mixed-bidi limits;
- `crates/egui/src/text_selection/visuals.rs`: current one-span-per-row painting limit.

`RUSTDOCFLAGS='-D warnings' cargo doc -p epaint -p egui --no-deps` passed and
generated `/home/abdu/.cargo-target/doc/epaint/index.html` and
`/home/abdu/.cargo-target/doc/egui/index.html`. The doc-test command in V22
passed both new examples against the recorded source baseline.

## Document integrity verification

The repository has no link-checker command. A local path/anchor scan resolves
the relative guide, matrix, report, and every evidence anchor. All 27 unique
external targets returned HTTP 200, including pinned sources, the baseline
commit, draft PR, and Taskum records. A slug check found all 13 active
`egui-rtl-fixes` backlog issues in the support matrix. `git diff --check` passed.

## Future validation record

For every new run, append the commit, date, OS, browser, renderer, exact font
version and checksum, feature flags, command or manual steps, expected result,
observed result, and artifact link. Do not promote a matrix cell from untested
based only on compilation, or from partial based on a different commit or font.
