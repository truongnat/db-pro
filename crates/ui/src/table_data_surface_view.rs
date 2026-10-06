//! Table-data shell layout.
use super::*;

pub(super) struct TableDataSurfaceContext {
    pub(super) theme: DbProTheme,
}

pub(super) fn draw_grid<F>(
    context: &TableDataSurfaceContext,
    ui: &mut egui::Ui,
    result: &UiQueryResult,
    draw_result_grid: F,
) where
    F: FnOnce(&mut egui::Ui, &UiQueryResult),
{
    ui.add_space(SPACE_XS);
    let data_width = ui.max_rect().width();
    const FOOTER_HEIGHT: f32 = 40.0;
    let available_height = (ui.max_rect().bottom() - ui.cursor().min.y - FOOTER_HEIGHT).max(0.0);
    let grid_height = available_height;
    ui.allocate_ui_with_layout(
        egui::vec2(data_width.max(0.0), grid_height),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            grid_frame(context.theme).show(ui, |ui| {
                ui.set_min_width(data_width.max(0.0));
                ui.set_min_height((grid_height - 2.0 * SPACE_XS).max(0.0));
                draw_result_grid(ui, result);
            });
        },
    );
}
