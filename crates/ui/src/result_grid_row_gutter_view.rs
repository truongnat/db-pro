use super::*;
use egui::{Align2, Pos2, Rounding, Stroke};

pub(super) struct GridRowGutterContext {
    pub(super) theme: DbProTheme,
    pub(super) row_number: u64,
    pub(super) selected: bool,
}

pub(super) fn draw_row_gutter(ui: &mut egui::Ui, context: GridRowGutterContext) -> egui::Response {
    let (gutter_rect, response) = ui.allocate_exact_size(egui::vec2(GRID_ROW_NUMBER_WIDTH, 28.0), Sense::click());
    let gutter_fill = if context.selected {
        context.theme.accent.linear_multiply(0.18)
    } else if response.hovered() {
        context.theme.surface_hover.linear_multiply(0.5)
    } else {
        context.theme.surface_panel.linear_multiply(0.5)
    };
    ui.painter().rect_filled(gutter_rect, Rounding::ZERO, gutter_fill);
    ui.painter().hline(
        gutter_rect.x_range(),
        gutter_rect.bottom(),
        Stroke::new(1.0, context.theme.border_subtle.linear_multiply(0.4)),
    );
    ui.painter().vline(
        gutter_rect.right(),
        gutter_rect.y_range(),
        Stroke::new(1.0, context.theme.border_subtle.linear_multiply(0.4)),
    );
    ui.painter().text(
        Pos2::new(gutter_rect.right() - 8.0, gutter_rect.center().y),
        Align2::RIGHT_CENTER,
        context.row_number.to_string(),
        FontId::monospace(11.0),
        if context.selected {
            context.theme.accent
        } else {
            context.theme.text_muted
        },
    );
    response
}
