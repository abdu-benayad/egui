# Right-to-left text support matrix

This document describes the implementation at commit
[`c93e8863e958507056415e2a6dace56e23ba5f3a`](https://github.com/abdu-benayad/egui/commit/c93e8863e958507056415e2a6dace56e23ba5f3a),
inspected on 2026-09-17. It is a development baseline, not a claim that egui has
complete right-to-left (RTL) support.

The commit is the two-commit head of the `bidi-runs` branch, based on upstream
commit `7ba3dbc4b07e72dcc85697e8cb51dcc70b1c8a6f`. It reports crate version 0.36.2,
but these changes are only in [draft PR #8577](https://github.com/emilk/egui/pull/8577):
they are not merged into `emilk/egui:main` and are not part of a released egui
version. All statuses below therefore describe this exact branch unless a row
explicitly says otherwise.

## Status meanings

- **Supported**: the stated behavior passed a focused check on this commit.
- **Partial**: a useful subset passed or exists, with a named failing or missing case.
- **Unsupported**: the required API or behavior is absent, or a focused check proves it incorrect.
- **Untested**: no matching runtime or conformance evidence was produced. Compilation alone does not change this status.

## Matrix

| Area | Status | Semantics and current limit | Evidence | Follow-up |
| --- | --- | --- | --- | --- |
| Single-row bidi layout | **Partial** | First-strong paragraph direction, embedding levels, directional shaping, and UAX #9 L2 visual placement work for the tested Hebrew/Latin/digit samples. Full UAX #9 conformance and L1 were not run. | [E1](rtl-validation.md#e1-bundled-font-bidi-layout), [E2](rtl-validation.md#e2-real-font-shaping), [source](https://github.com/abdu-benayad/egui/blob/c93e8863e958507056415e2a6dace56e23ba5f3a/crates/epaint/src/text/text_layout.rs#L1698-L1711) | [paragraph direction and UAX #9](https://taskum.com/project/egui-rtl-fixes/issue/paragraph-direction-uax9), [Unicode conformance](https://taskum.com/project/egui-rtl-fixes/issue/unicode-bidi-conformance) |
| Paragraph direction policy | **Partial** | Direction is automatic from the first strong character, similar to `dir="auto"`. There is no public explicit paragraph-direction override. | [E4](rtl-validation.md#e4-source-audit) | [paragraph direction and UAX #9](https://taskum.com/project/egui-rtl-fixes/issue/paragraph-direction-uax9) |
| UAX #9 L1 and explicit controls | **Untested** | No official BidiTest/BidiCharacterTest run was performed. Paragraph-end whitespace/reset behavior and explicit controls have no focused evidence. | [V20](rtl-validation.md#v20-uax-9-conformance) | [Unicode conformance](https://taskum.com/project/egui-rtl-fixes/issue/unicode-bidi-conformance), [paragraph direction and UAX #9](https://taskum.com/project/egui-rtl-fixes/issue/paragraph-direction-uax9) |
| Arabic and Hebrew shaping | **Partial** | Harfrust shapes runs with the resolved direction. A focused probe verified contextual Arabic forms and distinct Hebrew glyphs; the pinned-font demo rendered Arabic, Persian, Urdu, and Hebrew on native and web. Detailed Persian/Urdu shaping, ligatures, and fallback transitions remain untested. | [E2](rtl-validation.md#e2-real-font-shaping), [E6](rtl-validation.md#e6-interactive-demo), [source](https://github.com/abdu-benayad/egui/blob/c93e8863e958507056415e2a6dace56e23ba5f3a/crates/epaint/src/text/text_layout.rs#L1854-L1901) | [script shaping and font fallback](https://taskum.com/project/egui-rtl-fixes/issue/script-shaping-font-fallback) |
| Bundled font coverage | **Partial** | The production default fonts still do not cover Hebrew, so applications must supply suitable fonts. `egui_demo_lib` now contains checksum-pinned Arabic, Hebrew, and Latin fixtures for portable demos and tests, including Persian/Urdu, digits, punctuation, and marks. | [E1](rtl-validation.md#e1-bundled-font-bidi-layout), [E5](rtl-validation.md#e5-reproducible-font-fixtures), [source](https://github.com/abdu-benayad/egui/blob/c93e8863e958507056415e2a6dace56e23ba5f3a/crates/epaint/src/text/text_layout.rs#L2798-L2808) | [script shaping and font fallback](https://taskum.com/project/egui-rtl-fixes/issue/script-shaping-font-fallback) |
| Logical glyph order and simple clusters | **Partial** | `Row::glyphs` remains in logical character order while `Glyph::pos.x` is visual. One-glyph/many-character and many-glyph/one-character cases pass existing tests. A real Arabic base-plus-mark probe reconstructed `بِ` as `بب`, so character identity inside some clusters is wrong. | [E1](rtl-validation.md#e1-bundled-font-bidi-layout), [E2](rtl-validation.md#e2-real-font-shaping), [source](https://github.com/abdu-benayad/egui/blob/c93e8863e958507056415e2a6dace56e23ba5f3a/crates/epaint/src/text/text_layout_types.rs#L921-L968) | [grapheme cluster model](https://taskum.com/project/egui-rtl-fixes/issue/grapheme-cluster-model), [script shaping and font fallback](https://taskum.com/project/egui-rtl-fixes/issue/script-shaping-font-fallback) |
| Pointer hit-testing and simple caret geometry | **Partial** | Pure two-character RTL caret positions round-trip. The logical end of mixed `אב 12` resolves to the row's right edge instead of the visual end of the final logical glyph. No bidi caret affinity is represented. | [E1](rtl-validation.md#e1-bundled-font-bidi-layout), [E3](rtl-validation.md#e3-negative-geometry-and-selection-probes) | [visual caret navigation](https://taskum.com/project/egui-rtl-fixes/issue/visual-caret-navigation), [grapheme cluster model](https://taskum.com/project/egui-rtl-fixes/issue/grapheme-cluster-model) |
| Keyboard movement and caret affinity | **Unsupported** | Arrow movement has no bidi affinity model and has not been made visual-order aware for ambiguous boundaries. Logical insertion storage remains the only established invariant. | [E4](rtl-validation.md#e4-source-audit) | [visual caret navigation](https://taskum.com/project/egui-rtl-fixes/issue/visual-caret-navigation) |
| Selection and decorations | **Partial** | Selection code notices reversed endpoint coordinates, but uses one span between two logical cursors. Selecting the Hebrew word in `abc אבג def` produced a zero-width rectangle. Discontiguous mixed-bidi spans, styled-section boundaries, underline, background, and strikethrough need dedicated validation. | [E3](rtl-validation.md#e3-negative-geometry-and-selection-probes), [source](https://github.com/abdu-benayad/egui/blob/c93e8863e958507056415e2a6dace56e23ba5f3a/crates/egui/src/text_selection/visuals.rs#L43-L69) | [selection and decorations](https://taskum.com/project/egui-rtl-fixes/issue/bidi-selection-decorations) |
| Physical alignment | **Partial** | `LayoutJob::halign` still uses physical `Align::LEFT`, `Center`, and `RIGHT`; there is no logical start/end text alignment. Visual reordering precedes alignment. A pure RTL row's trimmed bounds are wrong on this baseline. | [E3](rtl-validation.md#e3-negative-geometry-and-selection-probes), [E4](rtl-validation.md#e4-source-audit) | [wrapping, alignment, and truncation](https://taskum.com/project/egui-rtl-fixes/issue/bidi-wrapping-truncation), [widget layout](https://taskum.com/project/egui-rtl-fixes/issue/rtl-widget-layout) |
| Wrapping, justification, and truncation | **Untested** | The code applies line breaking before visual reordering and alignment afterward. No RTL wrap, justification, ellipsis, trailing-whitespace, or narrow-mark case passed a focused check here. | [V10](rtl-validation.md#v10-narrow-marked-wrapping), [V12](rtl-validation.md#v12-wrapping-alignment-and-truncation) | [wrapping, alignment, and truncation](https://taskum.com/project/egui-rtl-fixes/issue/bidi-wrapping-truncation), [grapheme cluster model](https://taskum.com/project/egui-rtl-fixes/issue/grapheme-cluster-model) |
| AccessKit text direction and geometry | **Unsupported** | Text runs are unconditionally exposed as left-to-right, while character positions are derived from logical glyph iteration. | [E4](rtl-validation.md#e4-source-audit), [source](https://github.com/abdu-benayad/egui/blob/c93e8863e958507056415e2a6dace56e23ba5f3a/crates/egui/src/text_selection/accesskit_text.rs#L102-L148) | [RTL accessibility](https://taskum.com/project/egui-rtl-fixes/issue/rtl-accessibility) |
| IME and platform text input | **Untested** | Generic composition support exists, but marked text, composition ranges, candidate placement, deletion, and commit behavior were not exercised with RTL text on any platform. | [V17](rtl-validation.md#v17-ime-and-platform-input) | [RTL IME and platform input](https://taskum.com/project/egui-rtl-fixes/issue/rtl-ime-platform-input) |
| Widget/container direction | **Partial** | `Layout::right_to_left` physically places widgets from right to left. Text direction does not automatically inherit into container layout, and directional icons, popups, tables, scrolling, and navigation are not mirrored by a locale/direction policy. | [E4](rtl-validation.md#e4-source-audit), [source](https://github.com/emilk/egui/blob/7ba3dbc4b07e72dcc85697e8cb51dcc70b1c8a6f/crates/egui/src/layout.rs#L156-L164) | [RTL widget layout](https://taskum.com/project/egui-rtl-fixes/issue/rtl-widget-layout) |
| Linux native | **Partial** | The interactive demo was rendered with wgpu/Vulkan on Linux and its Ctrl+A logical range was verified, while screenshots expose incomplete bidi selection painting. IME and broader interaction remain untested. | [E6](rtl-validation.md#e6-interactive-demo), [V18](rtl-validation.md#v18-native-platforms-and-renderers) | [RTL IME and platform input](https://taskum.com/project/egui-rtl-fixes/issue/rtl-ime-platform-input) |
| Windows, macOS, Android, iOS | **Untested** | No build, runtime, input, or accessibility check was performed. | [V18](rtl-validation.md#v18-native-platforms-and-renderers) | [RTL IME and platform input](https://taskum.com/project/egui-rtl-fixes/issue/rtl-ime-platform-input), [RTL accessibility](https://taskum.com/project/egui-rtl-fixes/issue/rtl-accessibility) |
| Web/wasm and browsers | **Partial** | The exact wgpu wasm build passes. Chrome 150 rendered the glow build through SwiftShader and reproduced native Ctrl+A behavior; browser WebGPU, IME, and other browsers remain untested. | [E5](rtl-validation.md#e5-reproducible-font-fixtures), [E6](rtl-validation.md#e6-interactive-demo), [V19](rtl-validation.md#v19-wasm-and-browser) | [performance and wasm size](https://taskum.com/project/egui-rtl-fixes/issue/rtl-performance-wasm) |
| glow, wgpu, and other renderers | **Partial** | Linux wgpu/Vulkan and browser glow/WebGL were captured. Headless browser WebGPU presentation failed in Chrome's swap-chain setup; remaining renderers are untested. | [E6](rtl-validation.md#e6-interactive-demo), [V18](rtl-validation.md#v18-native-platforms-and-renderers) | [foundation landing](https://taskum.com/project/egui-rtl-fixes/issue/land-pr-8577-foundation) |
| Unicode conformance | **Untested** | Neither Unicode BidiTest.txt nor BidiCharacterTest.txt was run. | [V20](rtl-validation.md#v20-uax-9-conformance) | [Unicode conformance](https://taskum.com/project/egui-rtl-fixes/issue/unicode-bidi-conformance) |
| Runtime performance | **Untested** | No bidi layout or editing benchmark was measured against the upstream-base commit. | [V21](rtl-validation.md#v21-performance-and-wasm-size) | [performance and wasm size](https://taskum.com/project/egui-rtl-fixes/issue/rtl-performance-wasm) |
| Wasm binary size | **Untested** | The added `unicode-bidi` tables were not measured in a wasm artifact. | [V21](rtl-validation.md#v21-performance-and-wasm-size) | [performance and wasm size](https://taskum.com/project/egui-rtl-fixes/issue/rtl-performance-wasm) |

## Consumer invariants at this baseline

Consumers may rely on logical text storage and on `Row::glyphs` being indexed in
logical character order for the covered simple cases. They must not sort
`Row::glyphs` by x and then treat that result as source order. Conversely, the
first and last logical glyph are not reliable visual left/right bounds; inspect
positions across the row. Mixed-bidi caret, selection, bounds, wrapping, and
truncation behavior should be treated as provisional until their follow-up
issues land and this matrix is updated.

Text direction and container layout direction are separate. Automatic bidi
resolution changes glyph placement inside a galley. `Layout::right_to_left`
changes physical widget placement and does not establish inherited language or
paragraph direction.

## Backlog coverage

Every active project issue has a corresponding matrix area:

- [foundation PR](https://taskum.com/project/egui-rtl-fixes/issue/land-pr-8577-foundation): baseline, layout, shaping, logical glyph order, and renderer validation.
- [paragraph direction and UAX #9](https://taskum.com/project/egui-rtl-fixes/issue/paragraph-direction-uax9): paragraph policy, L1, and controls.
- [visual caret navigation](https://taskum.com/project/egui-rtl-fixes/issue/visual-caret-navigation): keyboard movement, mixed-row end caret, and affinity.
- [grapheme cluster model](https://taskum.com/project/egui-rtl-fixes/issue/grapheme-cluster-model): cluster identity, marks, cursor boundaries, and wrapping.
- [selection and decorations](https://taskum.com/project/egui-rtl-fixes/issue/bidi-selection-decorations): visual selection spans and styled geometry.
- [wrapping, alignment, and truncation](https://taskum.com/project/egui-rtl-fixes/issue/bidi-wrapping-truncation): physical/logical alignment, line breaking, justification, and ellipsis.
- [RTL accessibility](https://taskum.com/project/egui-rtl-fixes/issue/rtl-accessibility): AccessKit direction, geometry, and navigation.
- [RTL IME and platform input](https://taskum.com/project/egui-rtl-fixes/issue/rtl-ime-platform-input): composition and platform input.
- [RTL widget layout](https://taskum.com/project/egui-rtl-fixes/issue/rtl-widget-layout): inherited direction and mirrored widgets.
- [script shaping and font fallback](https://taskum.com/project/egui-rtl-fixes/issue/script-shaping-font-fallback): Arabic/Persian/Urdu/Hebrew shaping, fallback, and rendering.
- [Unicode conformance](https://taskum.com/project/egui-rtl-fixes/issue/unicode-bidi-conformance): official data and regression infrastructure.
- [performance and wasm size](https://taskum.com/project/egui-rtl-fixes/issue/rtl-performance-wasm): runtime benchmarks and artifact size.
- [documentation and examples](https://taskum.com/project/egui-rtl-fixes/issue/rtl-docs-examples): this matrix, reproducible fonts, public documentation, and the interactive demo.

The exact cases, environment, commands, and observed results are recorded in
[the RTL validation checklist](rtl-validation.md).
