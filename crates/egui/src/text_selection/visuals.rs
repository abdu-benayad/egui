use std::sync::Arc;

use emath::{Pos2, Rangef};
use epaint::{
    Stroke,
    text::{
        CharIndex, Glyph, Row,
        cursor::{CCursor, LayoutCursor},
    },
};

use crate::{
    Galley, Painter, Rect, Ui, Visuals, text_selection::text_cursor_state::cursor_rect, vec2,
};

use super::CCursorRange;

#[derive(Clone, Debug)]
pub struct RowVertexIndices {
    pub row: usize,
    pub vertex_indices: [u32; 6],
}

/// Adds text selection rectangles to the galley.
pub fn paint_text_selection(
    galley: &mut Arc<Galley>,
    visuals: &Visuals,
    cursor_range: &CCursorRange,
    mut new_vertex_indices: Option<&mut Vec<RowVertexIndices>>,
) {
    if cursor_range.is_empty() {
        return;
    }

    // We need to modify the galley (add text selection painting to it),
    // and so we need to clone it if it is shared:
    let galley: &mut Galley = Arc::make_mut(galley);

    let background_color = visuals.selection.bg_fill;
    let text_color = visuals.selection.stroke.color;

    let [min, max] = cursor_range.sorted_cursors();
    let min = galley.layout_from_cursor(min);
    let max = galley.layout_from_cursor(max);

    for ri in min.row..=max.row {
        let placed_row = &mut galley.rows[ri];
        let ends_with_newline = placed_row.ends_with_newline;
        let row = Arc::make_mut(&mut placed_row.row);

        let from = (ri == min.row).then_some(min.column);
        let to = (ri == max.row).then_some(max.column);
        let newline_size = if ends_with_newline {
            row.height() / 2.0 // visualize that we select the newline
        } else {
            0.0
        };
        let spans = selection_spans(row, from, to, newline_size);
        let mesh = &mut row.visuals.mesh;

        if !row.glyphs.is_empty() {
            // Change color of the selected text:
            let first_glyph_index = from.map_or(0, |column| column.0);
            let last_glyph_index = to.map_or(row.glyphs.len(), |column| column.0);

            let first_vertex_index = row
                .glyphs
                .get(first_glyph_index)
                .map_or(row.visuals.glyph_vertex_range.end, |g| g.first_vertex as _);
            let last_vertex_index = row
                .glyphs
                .get(last_glyph_index)
                .map_or(row.visuals.glyph_vertex_range.end, |g| g.first_vertex as _);

            for vi in first_vertex_index..last_vertex_index {
                mesh.vertices[vi].color = text_color;
            }
        }

        // Time to insert the selection rectangles into the row mesh.
        // They should be on top (after) of any background in the galley,
        // but behind (before) any glyphs. The row visuals has this information:
        let glyph_index_start = row.visuals.glyph_index_start;

        // Start by appending the rectangles to the end of the mesh, two triangles (= 6 indices) each:
        let num_indices_before = mesh.indices.len();
        for span in &spans {
            let rect = Rect::from_x_y_ranges(*span, 0.0..=row.size.y);
            mesh.add_colored_rect(rect, background_color);
        }
        debug_assert_eq!(
            num_indices_before + 6 * spans.len(),
            mesh.indices.len(),
            "We expect exactly 6 new indices per rectangle"
        );

        // Copy out the new triangles:
        let selection_triangles = mesh.indices[num_indices_before..].to_vec();

        // Move them in front of the glyphs, and every old triangle after them:
        mesh.indices[glyph_index_start..].rotate_right(selection_triangles.len());

        row.visuals.mesh_bounds = mesh.calc_bounds();

        if let Some(new_vertex_indices) = &mut new_vertex_indices {
            for triangles in selection_triangles.chunks_exact(6) {
                new_vertex_indices.push(RowVertexIndices {
                    row: ri,
                    vertex_indices: [
                        triangles[0],
                        triangles[1],
                        triangles[2],
                        triangles[3],
                        triangles[4],
                        triangles[5],
                    ],
                });
            }
        }
    }
}

/// The horizontal spans a selection covers on `row`.
///
/// `from`/`to` are the selection's columns on this row, or `None` where it
/// began on an earlier row / continues past this one; a continuing selection
/// also covers `newline_size` past the row's end.
///
/// A left-to-right row is one span between two carets. In a row with
/// right-to-left text the selected characters need not be contiguous on
/// screen, so the spans are built from the selected glyphs themselves.
fn selection_spans(
    row: &Row,
    from: Option<CharIndex>,
    to: Option<CharIndex>,
    newline_size: f32,
) -> Vec<Rangef> {
    let past_row_end = row.size.x + newline_size;

    if !row.glyphs.iter().any(Glyph::is_rtl) {
        let left = from.map_or(0.0, |column| row.x_offset(column));
        let right = to.map_or(past_row_end, |column| row.x_offset(column));
        return vec![Rangef::new(left, right)];
    }

    let first = from.map_or(0, |column| column.0).min(row.glyphs.len());
    let last = to
        .map_or(row.glyphs.len(), |column| column.0)
        .min(row.glyphs.len());
    let mut glyph_spans: Vec<Rangef> = row.glyphs[first..last]
        .iter()
        .filter(|glyph| 0.0 < glyph.advance_width)
        .map(|glyph| Rangef::new(glyph.pos.x, glyph.max_x()))
        .collect();
    glyph_spans.sort_by(|a, b| a.min.total_cmp(&b.min));

    let mut spans: Vec<Rangef> = Vec::new();
    for span in glyph_spans {
        match spans.last_mut() {
            Some(last) if span.min <= last.max + 0.5 => last.max = last.max.max(span.max),
            _ => spans.push(span),
        }
    }
    if to.is_none() && 0.0 < newline_size {
        spans.push(Rangef::new(row.size.x, past_row_end));
    }
    spans
}

#[expect(clippy::too_many_arguments)]
pub(crate) fn paint_ime_preedit_text_visuals(
    pos: Pos2,
    ui: &Ui,
    painter: &Painter,
    galley: &Arc<Galley>,
    row_height: f32,
    preedit_range: core::ops::Range<CCursor>,
    mut relative_active_range: Option<core::ops::Range<CCursor>>,
    time_since_last_interaction: f64,
) {
    /// Instead of implementing [`PartialOrd`] and [`Ord`] for [`CCursor`] to
    /// make [`std::ops::Range::is_empty`] available, we use this helper
    /// function instead.
    ///
    /// These traits are intentionally not implemented because
    /// [`CCursor::prefer_next_row`] makes it difficult to define a clear
    /// ordering between two [`CCursor`]s.
    fn is_cursor_range_empty(range: &core::ops::Range<CCursor>) -> bool {
        range.start.index == range.end.index
    }

    if is_cursor_range_empty(&preedit_range) {
        return;
    }

    if let Some(relative_active_range) = &mut relative_active_range
        && relative_active_range.end.index > preedit_range.end.index - preedit_range.start.index
    {
        relative_active_range.end.index = preedit_range.end.index - preedit_range.start.index;
    }

    let visuals = ui.visuals();
    let active_underline_stroke = visuals.ime_composition.active_underline_stroke;
    let inactive_underline_stroke = visuals.ime_composition.inactive_underline_stroke;

    if let Some(relative_active_range) = &relative_active_range
        && !is_cursor_range_empty(relative_active_range)
    {
        if relative_active_range.start.index > CharIndex::ZERO {
            paint_underlines(
                pos,
                painter,
                galley,
                galley.layout_from_cursor(preedit_range.start),
                galley.layout_from_cursor(preedit_range.start + relative_active_range.start.index),
                inactive_underline_stroke,
            );
        }

        paint_underlines(
            pos,
            painter,
            galley,
            galley.layout_from_cursor(preedit_range.start + relative_active_range.start.index),
            galley.layout_from_cursor(preedit_range.start + relative_active_range.end.index),
            active_underline_stroke,
        );

        if !is_cursor_range_empty(
            &(relative_active_range.end..(preedit_range.end - preedit_range.start.index)),
        ) {
            paint_underlines(
                pos,
                painter,
                galley,
                galley.layout_from_cursor(preedit_range.start + relative_active_range.end.index),
                galley.layout_from_cursor(preedit_range.end),
                inactive_underline_stroke,
            );
        }
    } else {
        paint_underlines(
            pos,
            painter,
            galley,
            galley.layout_from_cursor(preedit_range.start),
            galley.layout_from_cursor(preedit_range.end),
            inactive_underline_stroke,
        );
    }

    if let Some(relative_active_range) = relative_active_range
        && is_cursor_range_empty(&relative_active_range)
    {
        let active_cursor = preedit_range.start + relative_active_range.start.index;
        let cursor_rect = cursor_rect(galley, &active_cursor, row_height);

        paint_text_cursor(
            ui,
            painter,
            cursor_rect.translate(pos.to_vec2()),
            time_since_last_interaction,
        );
    }
}

fn paint_underlines(
    pos: Pos2,
    painter: &Painter,
    galley: &Arc<Galley>,
    min: LayoutCursor,
    max: LayoutCursor,
    stroke: Stroke,
) {
    for ri in min.row..=max.row {
        let placed_row = &galley.rows[ri];
        let row = &placed_row.row;

        let from = (ri == min.row).then_some(min.column);
        let to = (ri == max.row).then_some(max.column);
        let offset_y = placed_row.pos.y + row.size.y;

        for span in selection_spans(row, from, to, 0.0) {
            painter.line_segment(
                [
                    pos + vec2(span.min, offset_y),
                    pos + vec2(span.max, offset_y),
                ],
                stroke,
            );
        }
    }
}

/// Paint one end of the selection, e.g. the primary cursor.
///
/// This will never blink.
pub fn paint_cursor_end(painter: &Painter, visuals: &Visuals, cursor_rect: Rect) {
    let stroke = visuals.text_cursor.stroke;

    let top = cursor_rect.center_top();
    let bottom = cursor_rect.center_bottom();

    painter.line_segment([top, bottom], stroke);

    if false {
        // Roof/floor:
        let extrusion = 3.0;
        let width = 1.0;
        painter.line_segment(
            [top - vec2(extrusion, 0.0), top + vec2(extrusion, 0.0)],
            (width, stroke.color),
        );
        painter.line_segment(
            [bottom - vec2(extrusion, 0.0), bottom + vec2(extrusion, 0.0)],
            (width, stroke.color),
        );
    }
}

/// Paint one end of the selection, e.g. the primary cursor, with blinking (if enabled).
pub fn paint_text_cursor(
    ui: &Ui,
    painter: &Painter,
    primary_cursor_rect: Rect,
    time_since_last_interaction: f64,
) {
    if ui.visuals().text_cursor.blink {
        let on_duration = ui.visuals().text_cursor.on_duration;
        let off_duration = ui.visuals().text_cursor.off_duration;
        let total_duration = on_duration + off_duration;

        let time_in_cycle = (time_since_last_interaction % (total_duration as f64)) as f32;

        let wake_in = if time_in_cycle < on_duration {
            // Cursor is visible
            paint_cursor_end(painter, ui.visuals(), primary_cursor_rect);
            on_duration - time_in_cycle
        } else {
            // Cursor is not visible
            total_duration - time_in_cycle
        };

        ui.request_repaint_after_secs(wake_in);
    } else {
        paint_cursor_end(painter, ui.visuals(), primary_cursor_rect);
    }
}

#[cfg(test)]
#[cfg(feature = "default_fonts")]
mod tests {
    use std::sync::Arc;

    use epaint::text::{FontDefinitions, FontId, Fonts, TextOptions};

    use super::*;
    use crate::Color32;

    fn layout(text: &str) -> Arc<Galley> {
        let mut fonts = Fonts::new(TextOptions::default(), FontDefinitions::default());
        fonts.with_pixels_per_point(1.0).layout_no_wrap(
            text.to_owned(),
            FontId::proportional(14.0),
            Color32::WHITE,
        )
    }

    /// The x range of every selection rectangle painted on `row`, from the vertices it added.
    fn painted_spans(galley: &Galley, painted: &[RowVertexIndices], row: usize) -> Vec<Rangef> {
        let mesh = &galley.rows[row].row.visuals.mesh;
        painted
            .iter()
            .filter(|p| p.row == row)
            .map(|p| {
                let xs = p.vertex_indices.map(|vi| mesh.vertices[vi as usize].pos.x);
                Rangef::new(
                    xs.iter().copied().fold(f32::INFINITY, f32::min),
                    xs.iter().copied().fold(f32::NEG_INFINITY, f32::max),
                )
            })
            .collect()
    }

    #[test]
    fn selecting_a_right_to_left_word_highlights_the_word() {
        let mut galley = layout("abc אבג def");
        let hebrew = &galley.rows[0].row.glyphs[4..7];
        let left = hebrew.iter().map(|g| g.pos.x).fold(f32::INFINITY, f32::min);
        let right = hebrew
            .iter()
            .map(Glyph::max_x)
            .fold(f32::NEG_INFINITY, f32::max);

        let mut painted = Vec::new();
        paint_text_selection(
            &mut galley,
            &Visuals::default(),
            &CCursorRange::two(CCursor::new(4), CCursor::new(7)),
            Some(&mut painted),
        );

        let spans = painted_spans(&galley, &painted, 0);
        assert_eq!(spans.len(), 1, "the three letters are contiguous on screen");
        assert_eq!(spans[0], Rangef::new(left, right));
        assert!(
            0.0 < spans[0].span(),
            "the selection has width, not two carets at one x"
        );
    }

    #[test]
    fn selecting_everything_in_two_right_to_left_rows_covers_both() {
        let mut galley = layout("אב\nגד");
        let mut painted = Vec::new();
        paint_text_selection(
            &mut galley,
            &Visuals::default(),
            &CCursorRange::two(CCursor::new(0), CCursor::new(5)),
            Some(&mut painted),
        );

        for (ri, continues) in [(0, true), (1, false)] {
            let row = &galley.rows[ri].row;
            let spans = painted_spans(&galley, &painted, ri);
            let covered = Rangef::new(
                spans.iter().map(|s| s.min).fold(f32::INFINITY, f32::min),
                spans
                    .iter()
                    .map(|s| s.max)
                    .fold(f32::NEG_INFINITY, f32::max),
            );
            assert_eq!(covered.min, 0.0, "row {ri} is selected from its left edge");
            let expected_right = if continues {
                row.size.x + row.height() / 2.0 // the newline marker
            } else {
                row.size.x
            };
            assert_eq!(
                covered.max, expected_right,
                "row {ri} is selected to its right edge"
            );
        }
    }
}
