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
    grid_frame(context.theme).show(ui, |ui| {
        ui.set_min_width(data_width.max(0.0));
        draw_result_grid(ui, result);
    });
}
