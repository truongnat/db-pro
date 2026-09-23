//! Pure result-grid cell surface and value rendering.

use super::*;
use egui::{Align2, FontId, Pos2, Rect, Rounding, Stroke};

pub(super) struct GridCellSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) row_selected: bool,
    pub(super) cell_selected: bool,
    pub(super) row_dirty: bool,
    pub(super) display_position: usize,
    pub(super) validation_error: bool,
    pub(super) conflict_error: bool,
    pub(super) cell_mutation_error: bool,
    pub(super) row_mutation_error: bool,
    pub(super) edit_error: Option<&'a str>,
    pub(super) mutation_error: Option<&'a str>,
}

pub(super) fn draw_surface(
    context: &GridCellSurfaceContext<'_>,
    ui: &mut egui::Ui,
    cell_rect: Rect,
    cell_response: &egui::Response,
) {
    let fill = if context.validation_error || (context.cell_mutation_error && !context.conflict_error) {
        context.theme.danger.linear_multiply(0.14)
    } else if context.conflict_error {
        context.theme.warning.linear_multiply(0.16)
    } else if context.row_mutation_error {
        context.theme.danger.linear_multiply(0.08)
    } else if context.row_selected {
        if context.cell_selected {
            context.theme.accent.linear_multiply(0.20)
        } else {
            context.theme.accent.linear_multiply(0.08)
        }
    } else if context.cell_selected {
        context.theme.accent.linear_multiply(0.14)
    } else if context.row_dirty {
        context.theme.warning.linear_multiply(0.12)
    } else if cell_response.hovered() {
        context.theme.surface_hover.linear_multiply(0.35)
    } else if context.display_position % 2 == 1 {
        context.theme.surface_panel.linear_multiply(0.25)
    } else {
        context.theme.surface_elevated
    };
    ui.painter().rect_filled(cell_rect, Rounding::ZERO, fill);
    let border = Stroke::new(1.0, context.theme.border_subtle.linear_multiply(0.35));
    ui.painter().hline(cell_rect.x_range(), cell_rect.bottom(), border);
    ui.painter().vline(cell_rect.right(), cell_rect.y_range(), border);
    if context.cell_selected {
        ui.painter()
            .rect_stroke(cell_rect, Rounding::ZERO, Stroke::new(1.5, context.theme.accent));
    }
    if context.validation_error {
        ui.painter().rect_stroke(
            cell_rect.shrink(1.0),
            Rounding::ZERO,
            Stroke::new(1.5, context.theme.danger),
        );
        if let Some(error) = context.edit_error {
            cell_response.clone().on_hover_text(error);
        }
    }
    if context.cell_mutation_error || context.row_mutation_error {
        ui.painter().rect_stroke(
            cell_rect.shrink(1.0),
            Rounding::ZERO,
            Stroke::new(
                1.5,
                if context.conflict_error {
                    context.theme.warning
                } else {
                    context.theme.danger
                },
            ),
        );
        if let Some(error) = context.mutation_error {
            cell_response.clone().on_hover_text(error);
        }
    }
}

pub(super) fn draw_value(
    theme: DbProTheme,
    ui: &mut egui::Ui,
    cell_rect: Rect,
    cell_response: &egui::Response,
    display_cell: &UiCell,
) {
    let text_rect = cell_rect.shrink2(egui::vec2(8.0, 3.0));
    let is_null = matches!(display_cell, UiCell::Null);
    let raw_value = crate::result_grid::cell_text_as_str(display_cell);
    let font = match display_cell {
        UiCell::Null => FontId::proportional(11.5),
        UiCell::Number(_) | UiCell::Json(_) | UiCell::Bytes(_) => FontId::monospace(11.5),
        _ => FontId::proportional(12.0),
    };
    let text_color = match display_cell {
        UiCell::Null => theme.text_muted.linear_multiply(0.55),
        UiCell::Boolean(value) => {
            if *value {
                theme.success
            } else {
                theme.danger
            }
        }
        UiCell::Number(_) | UiCell::Text(_) => theme.text_primary,
        UiCell::Json(_) | UiCell::Bytes(_) => theme.text_secondary,
    };
    let painter = ui.painter().with_clip_rect(text_rect);
    if is_null {
        let galley = painter.layout_no_wrap(
            "NULL".to_owned(),
            FontId::new(11.0, egui::FontFamily::Proportional),
            text_color,
        );
        painter.galley(
            Pos2::new(text_rect.left(), text_rect.center().y - galley.size().y * 0.5),
            galley,
            text_color,
        );
    } else {
        painter.text(
            Pos2::new(text_rect.left(), text_rect.center().y),
            Align2::LEFT_CENTER,
            raw_value,
            font,
            text_color,
        );
    }
    if cell_response.hovered() && raw_value.len() > 36 {
        cell_response.clone().on_hover_text(raw_value);
    }
}
