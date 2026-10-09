//! Result-grid viewport composition and virtualized row presentation.
use super::super::*;
use super::{GridRows, GridSelectionLookup};
use egui::Vec2;

pub(super) struct ResultGridBodyContext<'a> {
    pub(super) result: &'a UiQueryResult,
    pub(super) indexes: &'a [usize],
    pub(super) widths: &'a [f32],
    pub(super) order: &'a [usize],
    pub(super) editable: bool,
    pub(super) row_offset: u64,
    pub(super) selection_lookup: &'a GridSelectionLookup,
    pub(super) theme: DbProTheme,
}

pub(super) struct ResultGridRowInput<'a> {
    pub(super) result: &'a UiQueryResult,
    pub(super) rows: &'a GridRows<'a>,
    pub(super) position: usize,
}

pub(super) trait ResultGridBodyRenderer {
    fn draw_header(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        indexes: &[usize],
        widths: &[f32],
        order: &[usize],
    );

    fn draw_row(&mut self, ui: &mut egui::Ui, input: ResultGridRowInput<'_>);
}

/// Paint the leftover viewport below the last row so the grid surface reaches
/// the pane bottom: editor fill, the pinned-width gutter strip, and row-height
/// separator lines continuing the grid cadence.
fn draw_empty_fill(
    ui: &mut egui::Ui,
    rows_top: f32,
    viewport_height: f32,
    row_count: usize,
    row_height: f32,
    theme: DbProTheme,
) {
    let filler_top = ui.cursor().top();
    let filler_bottom = rows_top + viewport_height;
    if filler_bottom - filler_top < 0.5 {
        return;
    }
    let left = ui.max_rect().left();
    let right = ui.max_rect().right();
    let rect = egui::Rect::from_min_max(egui::pos2(left, filler_top), egui::pos2(right, filler_bottom));
    let painter = ui.painter();
    let border = egui::Stroke::new(1.0, theme.border_subtle);
    painter.rect_filled(rect, egui::CornerRadius::ZERO, theme.surface_editor);
    let gutter = egui::Rect::from_min_max(rect.min, egui::pos2(left + GRID_ROW_NUMBER_WIDTH, rect.bottom()));
    painter.rect_filled(gutter, egui::CornerRadius::ZERO, theme.surface_panel);
    painter.vline(gutter.right(), gutter.y_range(), border);
    let mut y = rows_top + (row_count as f32 + 1.0) * row_height;
    while y <= filler_bottom {
        painter.hline(left..=right, y, border);
        y += row_height;
    }
}

pub(super) fn draw_body(
    ui: &mut egui::Ui,
    context: ResultGridBodyContext<'_>,
    renderer: &mut dyn ResultGridBodyRenderer,
) {
    let grid_height = ui.available_height().max(180.0);
    let grid_width = ui.available_width().max(0.0);

    ui.allocate_ui_with_layout(
        egui::vec2(grid_width, grid_height),
        Layout::top_down(Align::Min),
        |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                let content_width = GRID_ROW_NUMBER_WIDTH + context.widths.iter().sum::<f32>();
                // Rows/header paint a trailing filler up to the grid edge, so the
                // scroll content must claim the viewport width even when the
                // columns are narrower.
                ui.set_min_width(content_width.max(grid_width));
                renderer.draw_header(ui, context.result, context.indexes, context.widths, context.order);

                let rows = GridRows {
                    indexes: context.indexes,
                    widths: context.widths,
                    order: context.order,
                    editable: context.editable,
                    row_offset: context.row_offset,
                    selection_lookup: context.selection_lookup,
                };
                let row_height = 28.0;
                let viewport_height = (grid_height - 28.0).max(140.0);
                let rows_top = ui.cursor().top();
                egui::ScrollArea::vertical()
                    .max_height(viewport_height)
                    .show_rows(ui, row_height, context.indexes.len(), |ui, range| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        for position in range {
                            renderer.draw_row(
                                ui,
                                ResultGridRowInput {
                                    result: context.result,
                                    rows: &rows,
                                    position,
                                },
                            );
                        }
                    });
                draw_empty_fill(ui, rows_top, viewport_height, context.indexes.len(), row_height, context.theme);
            });
        },
    );
}
