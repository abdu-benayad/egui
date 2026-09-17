use std::sync::Arc;

use emath::Pos2;
use epaint::{
    Stroke,
    text::{
        CharIndex,
        cursor::{CCursor, LayoutCursor},
    },
};

use crate::{
    Galley, Painter, Rect, Ui, Visuals, pos2, text_selection::text_cursor_state::cursor_rect, vec2,
};

use super::CCursorRange;

#[derive(Clone, Debug)]
pub struct RowVertexIndices {
    pub row: usize,
    pub vertex_indices: [u32; 6],
}

/// Adds text selection rectangles to the galley.
///
/// The input range uses logical character indices. On the current bidi
/// baseline this function emits one span per row and does not yet cover all
/// discontiguous visual spans of a mixed-direction selection.
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
        let row = Arc::make_mut(&mut placed_row.row);

        let left = if ri == min.row {
            row.x_offset(min.column)
        } else {
            0.0
        };
        let right = if ri == max.row {
            row.x_offset(max.column)
        } else {
            let newline_size = if placed_row.ends_with_newline {
                row.height() / 2.0 // visualize that we select the newline
            } else {
                0.0
            };
            row.size.x + newline_size
        };
        // In right-to-left text the cursor at the smaller char index is the one further right:
        let (left, right) = (left.min(right), left.max(right));

        let rect = Rect::from_min_max(pos2(left, 0.0), pos2(right, row.size.y));
        let mesh = &mut row.visuals.mesh;

        if !row.glyphs.is_empty() {
            // Change color of the selected text:
            let first_glyph_index = if ri == min.row { min.column.0 } else { 0 };
            let last_glyph_index = if ri == max.row {
                max.column.0
            } else {
                row.glyphs.len()
            };

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

        // Time to insert the selection rectangle into the row mesh.
        // It should be on top (after) of any background in the galley,
        // but behind (before) any glyphs. The row visuals has this information:
        let glyph_index_start = row.visuals.glyph_index_start;

        // Start by appending the selection rectangle to end of the mesh, as two triangles (= 6 indices):
        let num_indices_before = mesh.indices.len();
        mesh.add_colored_rect(rect, background_color);
        assert_eq!(
            num_indices_before + 6,
            mesh.indices.len(),
            "We expect exactly 6 new indices"
        );

        // Copy out the new triangles:
        let selection_triangles = [
            mesh.indices[num_indices_before],
            mesh.indices[num_indices_before + 1],
            mesh.indices[num_indices_before + 2],
            mesh.indices[num_indices_before + 3],
            mesh.indices[num_indices_before + 4],
            mesh.indices[num_indices_before + 5],
        ];

        // Move every old triangle forwards by 6 indices to make room for the new triangle:
        for i in (glyph_index_start..num_indices_before).rev() {
            mesh.indices.swap(i, i + 6);
        }
        // Put the new triangle in place:
        mesh.indices[glyph_index_start..glyph_index_start + 6]
            .clone_from_slice(&selection_triangles);

        row.visuals.mesh_bounds = mesh.calc_bounds();

        if let Some(new_vertex_indices) = &mut new_vertex_indices {
            new_vertex_indices.push(RowVertexIndices {
                row: ri,
                vertex_indices: selection_triangles,
            });
        }
    }
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

#[cfg(test)]
mod tests {
    use epaint::{Color32, FontId, text::Fonts};

    use super::*;

    /// Executable evidence for the RTL selection limitation documented in
    /// `docs/rtl-validation.md`. This should become a positive regression test
    /// when selection painting accounts for visual bidi runs.
    #[test]
    fn selecting_a_right_to_left_word_highlights_the_word() {
        let mut fonts = Fonts::new(Default::default(), Default::default());
        let mut galley = fonts.with_pixels_per_point(1.0).layout_no_wrap(
            "abc אבג def".to_owned(),
            FontId::default(),
            Color32::WHITE,
        );
        let row = &galley.rows[0].row;
        let expected_left = row.glyphs[4..7]
            .iter()
            .map(|glyph| glyph.pos.x)
            .reduce(f32::min)
            .unwrap();
        let expected_right = row.glyphs[4..7]
            .iter()
            .map(|glyph| glyph.max_x())
            .reduce(f32::max)
            .unwrap();

        let mut selection_vertices = Vec::new();
        paint_text_selection(
            &mut galley,
            &Visuals::default(),
            &CCursorRange::two(CCursor::new(4), CCursor::new(7)),
            Some(&mut selection_vertices),
        );

        let indices = selection_vertices[0].vertex_indices;
        let mesh = &galley.rows[0].row.visuals.mesh;
        let actual_left = indices
            .iter()
            .map(|index| mesh.vertices[*index as usize].pos.x)
            .reduce(f32::min)
            .unwrap();
        let actual_right = indices
            .iter()
            .map(|index| mesh.vertices[*index as usize].pos.x)
            .reduce(f32::max)
            .unwrap();

        eprintln!(
            "RTL word selection: actual=[{actual_left}, {actual_right}], expected=[{expected_left}, {expected_right}]"
        );
        assert!(
            actual_left != expected_left || actual_right != expected_right,
            "the documented limitation was fixed; update the support matrix and turn this into a positive regression test"
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

        let left = if ri == min.row {
            row.x_offset(min.column)
        } else {
            0.0
        };
        let right = if ri == max.row {
            row.x_offset(max.column)
        } else {
            row.size.x
        };
        let (left, right) = (left.min(right), left.max(right));

        let offset_y = placed_row.pos.y + row.size.y;

        painter.line_segment(
            [pos + vec2(left, offset_y), pos + vec2(right, offset_y)],
            stroke,
        );
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
