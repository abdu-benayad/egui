# Right-to-left text in egui

egui resolves paragraph direction from the first strong character, splits text
into directional runs, shapes each run, and places the resulting glyphs in
visual order. There is no public option to force a paragraph direction.

Applications must install fonts that cover the scripts they display. The
default fonts do not cover every RTL script.

## Logical text and visual glyphs

Text, cursor, and selection indices use logical source order. `Row::glyphs`
also stays in logical order, while each `Glyph::pos.x` records its visual
position. In an RTL run, x positions can therefore decrease as logical indices
increase.

Keep the original glyph index for editing, selection, and source ranges. Code
that needs visual traversal can sort references to glyphs by `pos.x`. Do not
sort `Row::glyphs` itself and then use its indices as source indices.

The first and last entries in `Row::glyphs` are logical endpoints. They are not
necessarily the leftmost and rightmost glyphs, so visual bounds must inspect all
positioned glyphs. `PlacedRow::rect_without_leading_space` does not yet do this
and can return incorrect bounds for RTL rows.

The current implementation keeps one `Glyph` entry per Rust `char` in the
covered cases, but it is not a complete grapheme-cluster editing model.
Combining marks can be reconstructed or assigned incorrectly when a shaped
cluster contains a different number of glyphs and characters.

## Cursors and selections

`CCursor::index` and `LayoutCursor::column` represent logical character
boundaries. `Row::char_at` maps a visual x coordinate to a logical boundary,
and `Row::x_offset` maps a logical boundary to an x coordinate.

Cursor values do not include bidi caret affinity. A logical boundary at a
direction change can have two visual caret positions, but the cursor types
cannot distinguish them. Mixed-direction end-caret geometry is also incomplete.

`cursor_left_one_character` and `cursor_right_one_character` change the logical
source index. Their historical names do not promise visual left and right
movement in bidirectional text.

`CCursorRange` stores logical endpoints, so slicing and clipboard text follow
source order. Selection painting currently emits one span per row and does not
cover every discontiguous visual span of a mixed-direction selection.

## Alignment and widget layout

`LayoutJob::halign` uses physical alignment: `Align::LEFT` means the left edge
and `Align::RIGHT` means the right edge for both LTR and RTL paragraphs. It does
not provide logical start and end alignment.

Text direction and widget placement are separate. Automatic bidi resolution
controls glyph shaping and placement inside a galley. `Layout::right_to_left`
controls the physical order in which a `Ui` places widgets. Ordinary child UIs,
including those created by `Ui::scope`, inherit their parent's layout unless
they explicitly override it, so they can continue that physical RTL placement.
This does not force RTL text shaping or establish an application-wide direction
policy.

Containers that choose another layout and directional affordances such as
icons, popups, tables, scrolling, and navigation are not mirrored
automatically.

## Working with bidi rows

For source operations, iterate `Row::glyphs` in its stored order. For visual
operations, retain each logical index while making a separately sorted view:

```text
logical: enumerate Row::glyphs and retain each original index
visual:  sort those (index, glyph) pairs by glyph.pos.x
```

Treat pointer hit-testing, keyboard movement, and text mutation as separate
operations. Pointer hit-testing starts with a visual coordinate and returns a
logical boundary. Insertion, deletion, selection ranges, and clipboard text use
logical order. Visual arrow navigation needs affinity information that the
current cursor types do not contain.
