use super::*;
use egui::{Align2, Rounding, Stroke};

pub(super) struct GridRowGutterContext {
    pub(super) theme: DbProTheme,
    pub(super) row_number: u64,
    pub(super) selected: bool,
}

pub(super) fn draw_row_gutter(ui: &mut egui::Ui, context: GridRowGutterContext) -> egui::Response {
    let (gutter_rect, response) = ui.allocate_exact_size(egui::vec2(GRID_ROW_NUMBER_WIDTH, 28.0), Sense::click());
    let gutter_fill = if context.selected {
        context.theme.accent_soft
    } else if response.hovered() {
        context.theme.surface_hover.linear_multiply(0.5)
    } else {
        context.theme.surface_panel
    };
    ui.painter().rect_filled(gutter_rect, Rounding::ZERO, gutter_fill);
    let border = Stroke::new(1.0, context.theme.border_subtle);
    ui.painter().hline(gutter_rect.x_range(), gutter_rect.bottom(), border);
    ui.painter().vline(gutter_rect.right(), gutter_rect.y_range(), border);
    ui.painter().text(
        gutter_rect.center(),
        Align2::CENTER_CENTER,
        context.row_number.to_string(),
        FontId::monospace(11.0),
        if context.selected {
            context.theme.accent
        } else {
            context.theme.text_tertiary
        },
    );
    response
}
