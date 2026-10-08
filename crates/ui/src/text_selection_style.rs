// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
//! egui 0.29 paints selection backgrounds but leaves selected glyph colors unchanged.
//! Adapt its native text meshes after label selection has finalized, without reshaping text.

use std::sync::Arc;

use egui::{epaint::TextShape, Color32, Context, Shape};

pub(crate) fn install(ctx: &Context) {
    let id = egui::Id::new("db-pro-text-selection-style");
    let installed = ctx.data_mut(|data| {
        let installed = data.get_temp::<bool>(id).unwrap_or(false);
        data.insert_temp(id, true);
        installed
    });
    if installed {
        return;
    }
    ctx.on_end_pass(
        "db-pro-text-selection-style",
        Arc::new(|ctx| {
            if ctx.memory(|memory| memory.focused().is_none())
                && !egui::text_selection::LabelSelectionState::load(ctx).has_selection()
            {
                return;
            }
            let selection = ctx.style().visuals.selection;
            let layers = ctx.memory(|memory| memory.layer_ids().collect::<Vec<_>>());
            ctx.graphics_mut(|graphics| {
                for layer in layers {
                    if let Some(list) = graphics.get_mut(layer) {
                        for index in 0..list.next_idx().0 {
                            list.mutate_shape(egui::layers::ShapeIdx(index), |shape| {
                                style_shape(&mut shape.shape, selection.bg_fill, selection.stroke.color);
                            });
                        }
                    }
                }
            });
        }),
    );
}

fn style_shape(shape: &mut Shape, background: Color32, foreground: Color32) {
    match shape {
        Shape::Text(text) => style_text(text, background, foreground),
        Shape::Vec(shapes) => {
            for shape in shapes {
                style_shape(shape, background, foreground);
            }
        }
        _ => {}
    }
}

fn selection_bounds(row: &egui::epaint::text::Row, background: Color32) -> Option<egui::Rect> {
    let mesh = &row.visuals.mesh;
    let start = mesh.vertices.len().checked_sub(4)?;
    let indices = mesh
        .indices
        .get(row.visuals.glyph_index_start..row.visuals.glyph_index_start + 6)?;
    // Native Label and TextEdit append a selection quad, then insert its triangles
    // immediately before glyphs. Ordinary rich-text backgrounds do not have this layout.
    if start < row.visuals.glyph_vertex_range.end
        || !indices
            .iter()
            .all(|&index| (start..start + 4).contains(&(index as usize)))
        || !mesh.vertices[start..].iter().all(|vertex| vertex.color == background)
    {
        return None;
    }
    Some(mesh.vertices[start..].iter().fold(egui::Rect::NOTHING, |rect, vertex| {
        rect.union(egui::Rect::from_min_max(vertex.pos, vertex.pos))
    }))
}

fn style_text(text: &mut TextShape, background: Color32, foreground: Color32) {
    if !text
        .galley
        .rows
        .iter()
        .any(|row| selection_bounds(row, background).is_some())
    {
        return;
    }
    let galley = Arc::make_mut(&mut text.galley);
    if let Some(color) = text.override_text_color.take() {
        for row in &mut galley.rows {
            for vertex in &mut row.visuals.mesh.vertices[row.visuals.glyph_vertex_range.clone()] {
                vertex.color = color;
            }
        }
    }
    for row in &mut galley.rows {
        let Some(bounds) = selection_bounds(row, background) else {
            continue;
        };
        let mut vertices = row.visuals.mesh.vertices[row.visuals.glyph_vertex_range.clone()].chunks_exact_mut(4);
        for glyph in &row.glyphs {
            if glyph.uv_rect.is_nothing() {
                continue;
            }
            let Some(quad) = vertices.next() else { break };
            let center = glyph.pos.x + glyph.advance_width * 0.5;
            if bounds.left() <= center && center < bounds.right() {
                for vertex in quad {
                    vertex.color = foreground;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_label_drag_selection_is_white_without_recoloring_unselected_text() {
        let ctx = Context::default();
        crate::DbProTheme::install_fonts(&ctx);
        let theme = crate::DbProTheme::light();
        theme.apply(&ctx);
        let mut rect = egui::Rect::NOTHING;
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rect = ui
                    .add(
                        egui::Label::new(
                            egui::RichText::new("Correct the highlighted fields, then save again.")
                                .color(theme.text_secondary),
                        )
                        .selectable(true),
                    )
                    .rect;
            });
        });
        let start = rect.left_center() + egui::vec2(2.0, 0.0);
        let end = start + egui::vec2(70.0, 0.0);
        let mut output = None;
        for events in [
            vec![
                egui::Event::PointerMoved(start),
                egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            vec![egui::Event::PointerMoved(end)],
            vec![egui::Event::PointerButton {
                pos: end,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        ] {
            output = Some(ctx.run(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new("Correct the highlighted fields, then save again.")
                                    .color(theme.text_secondary),
                            )
                            .selectable(true),
                        );
                    });
                },
            ));
        }
        let output = output.unwrap();
        let text = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                Shape::Text(text) if text.galley.text().starts_with("Correct") => Some(text),
                _ => None,
            })
            .unwrap();
        let row = &text.galley.rows[0];
        assert!(selection_bounds(row, ctx.style().visuals.selection.bg_fill).is_some());
        let vertices = &row.visuals.mesh.vertices[row.visuals.glyph_vertex_range.clone()];
        assert!(vertices.iter().any(|vertex| vertex.color == Color32::WHITE));
        assert!(vertices.iter().any(|vertex| vertex.color == theme.text_secondary));
    }

    #[test]
    fn native_text_edit_selection_recolors_only_selected_glyphs_in_both_themes() {
        for theme in [crate::DbProTheme::light(), crate::DbProTheme::dark()] {
            let ctx = Context::default();
            crate::DbProTheme::install_fonts(&ctx);
            theme.apply(&ctx);
            theme.apply(&ctx); // Installation must not multiply callbacks each frame.
            let id = egui::Id::new("selected-input");
            let mut state = egui::text_edit::TextEditState::default();
            state.cursor.set_char_range(Some(egui::text::CCursorRange::two(
                egui::text::CCursor::new(1),
                egui::text::CCursor::new(4),
            )));
            state.store(&ctx, id);
            ctx.memory_mut(|memory| memory.request_focus(id));
            let mut value = "abcdef".to_owned();
            let output = ctx.run(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut value)
                            .id(id)
                            .text_color(theme.text_secondary),
                    );
                });
            });
            let text = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    Shape::Text(text) if text.galley.text() == "abcdef" => Some(text),
                    _ => None,
                })
                .expect("native TextEdit paint output");
            let row = &text.galley.rows[0];
            assert!(selection_bounds(row, ctx.style().visuals.selection.bg_fill).is_some());
            for (index, quad) in row.visuals.mesh.vertices[row.visuals.glyph_vertex_range.clone()]
                .chunks_exact(4)
                .enumerate()
            {
                let expected = if (1..4).contains(&index) {
                    Color32::WHITE
                } else {
                    theme.text_secondary
                };
                assert!(quad.iter().all(|vertex| vertex.color == expected), "glyph {index}");
            }
            assert_eq!(value, "abcdef");
        }
    }
}
