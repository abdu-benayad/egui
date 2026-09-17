# Right-to-left text in egui

This guide describes the development implementation at commit
`c93e8863e958507056415e2a6dace56e23ba5f3a`. The changes are in draft
[PR #8577](https://github.com/emilk/egui/pull/8577), not in an egui release.
The docs, demo, fixtures, and retained evidence probes are delivered in open
[stacked PR #1](https://github.com/abdu-benayad/egui/pull/1); the executable
evidence revision is `e10ba24df38c716cd8a40dbefc365d95bb58c18d`.
See the [versioned support matrix](rtl-support.md) for status by subsystem and
the [validation checklist](rtl-validation.md) for exact evidence and failures.

## What the baseline does

Epaint resolves paragraph embedding levels with `unicode-bidi`, shapes each
directional run with Harfrust, and places glyphs in visual order. Paragraph
direction is automatic: the first strong character determines the base
direction. There is no public API to force LTR or RTL independently of content.

`LayoutJob::halign` remains physical alignment. `Align::LEFT` means the left
edge and `Align::RIGHT` means the right edge for both LTR and RTL paragraphs.
Logical start/end alignment is not implemented.

Applications must install fonts that cover their scripts. The bundled default
fonts used by the focused tests do not cover Hebrew. The validation report
records one local Noto Naskh Arabic and Noto Sans Hebrew probe. The demo crate
also ships pinned Noto Sans Arabic, Noto Sans Hebrew, and Noto Sans fixtures
through `egui_demo_lib::rtl_font_fixtures`; these do not change epaint's
production defaults.

## Logical text and visual glyphs

Public text, cursor, and selection indices stay in logical source order. For the
covered simple cases, `Row::text()` reconstructs that logical order and
`Row::glyphs` is indexed the same way. In a bidi row, `Glyph::pos.x` records
visual placement, so x positions can decrease as logical indices increase.

Code that needs visual traversal may sort references by x, but must preserve the
original logical index for editing, selection, and source ranges. Code that
needs visual bounds must inspect every positioned glyph or use a verified mesh
bound. The first and last entries in `Row::glyphs` are logical endpoints, not
necessarily the leftmost and rightmost glyphs.

Shaping preserves one `Glyph` slot per Rust `char` for the simple tested cluster
cases. This is not yet a complete grapheme-cluster editing model. In particular,
the recorded real-font probe shows that an Arabic base-plus-mark cluster can
reconstruct the mark as another copy of the base on this commit.

## Cursors, editing, and selection

`CCursor::index` and `LayoutCursor::column` are logical character boundaries.
`Row::char_at` maps a visual x coordinate to a logical boundary, and
`Row::x_offset` performs the inverse for simple cases. A pure two-letter RTL
row passes the current round-trip test.

The cursor types do not carry bidi affinity. `cursor_left_one_character` and
`cursor_right_one_character` change the logical source index; their historical
names do not promise visual left/right movement. Mixed-direction end-caret
geometry is known to be incorrect on this baseline.

`CCursorRange` also stores logical endpoints. Copying or slicing a selection
therefore follows source order. Selection painting is incomplete: selecting the
Hebrew word in `abc אבג def` produced a zero-width highlight in the recorded
probe. Decorations and formatted-section geometry need further validation.

## Text direction and widget layout

Text direction and container direction are separate:

- automatic bidi resolution controls glyph shaping and placement inside a galley;
- `Layout::right_to_left` controls the physical order in which a `Ui` places widgets.

Neither setting is inherited as an application-wide direction policy. The
baseline does not automatically mirror directional icons, popups, tables,
scrolling, navigation, or descendant layouts.

## Accessibility, IME, and platforms

AccessKit text runs are currently exposed as left-to-right even for RTL text.
RTL accessibility geometry and navigation are unsupported at this baseline.

Generic IME composition support exists, but RTL marked text, candidate-window
placement, deletion, and commit behavior have not been validated. No native
IME run is included in this baseline.

## Interactive demo

Open **RTL text** in `egui_demo_app` to inspect the pinned Arabic, Hebrew,
Persian/Urdu, and Latin fixtures; mixed numbers and punctuation; styled and
wrapped text; physical alignment; truncation; TextEdit selection; spaced RTL
text; and physical `Layout::right_to_left` widget placement. The page labels
unsupported direction, affinity, mirroring, accessibility, and IME behavior
instead of simulating APIs that do not exist.

The [validation report](rtl-validation.md#e6-interactive-demo) records native
wgpu and Chrome glow screenshots plus the exact interaction results. Those
runs reproduce incomplete mixed-bidi selection painting and dropped combining
mark identity, so the demo is evidence of both working and failing behavior.

## Migration notes

If code previously assumed that `Row::glyphs` was x-sorted, keep its logical
iteration for source operations and build a separate visual view:

```text
logical: enumerate Row::glyphs and retain each original index
visual:  sort those (index, glyph) pairs by glyph.pos.x
```

Do not use `glyphs.first()` and `glyphs.last()` as visual bounds. This is already
wrong for pure RTL rows and mixed-direction rows. Likewise, do not infer cluster
identity from glyph width: continuation characters and combining marks may have
zero advance, while a composed cluster may be carried by one atlas entry.

Treat pointer hit-testing, keyboard movement, and text mutation as separate
operations. Pointer hit-testing starts from a visual coordinate and returns a
logical boundary. Text insertion, deletion, selection ranges, and clipboard
text operate on logical order. Visual arrow navigation and affinity are future
features, not semantics that can be inferred from the current cursor names.

## Follow-up work

- [Explicit paragraph direction and complete UAX #9 behavior](https://taskum.com/project/egui-rtl-fixes/issue/paragraph-direction-uax9)
- [Visual caret navigation and bidi affinity](https://taskum.com/project/egui-rtl-fixes/issue/visual-caret-navigation)
- [Inherited RTL widget layout and mirroring](https://taskum.com/project/egui-rtl-fixes/issue/rtl-widget-layout)
- [Cluster-aware editing and layout](https://taskum.com/project/egui-rtl-fixes/issue/grapheme-cluster-model)
- [Selection and decoration geometry](https://taskum.com/project/egui-rtl-fixes/issue/bidi-selection-decorations)
- [Accessibility](https://taskum.com/project/egui-rtl-fixes/issue/rtl-accessibility)
- [IME and platform input](https://taskum.com/project/egui-rtl-fixes/issue/rtl-ime-platform-input)
