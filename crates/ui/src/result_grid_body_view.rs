//! Result-grid viewport composition and virtualized row presentation.
use super::super::*;
use super::super::result_grid_virtual_adapter::{
    adapt_virtual_grid_window, VirtualGridAdapterInput, RESULT_GRID_ROW_HEIGHT,
    ResultGridVirtualRuntime,
};
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

pub(super) fn draw_body(
    ui: &mut egui::Ui,
    context: ResultGridBodyContext<'_>,
    virtual_runtime: &mut ResultGridVirtualRuntime,
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
                ui.set_min_width(content_width);
                renderer.draw_header(ui, context.result, context.indexes, context.widths, context.order);

                let rows = GridRows {
                    indexes: context.indexes,
                    widths: context.widths,
                    order: context.order,
                    editable: context.editable,
                    row_offset: context.row_offset,
                    selection_lookup: context.selection_lookup,
                };
                let row_height = RESULT_GRID_ROW_HEIGHT;
                let viewport_top = ui.cursor().min.y;
                egui::ScrollArea::vertical()
                    .max_height((grid_height - 34.0).max(140.0))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        let scroll_offset_y = (viewport_top - ui.max_rect().top()).max(0.0);
                        let window = match adapt_virtual_grid_window(
                            virtual_runtime,
                            VirtualGridAdapterInput {
                                row_count: rows.indexes.len(),
                                viewport_height: (grid_height - 34.0).max(140.0),
                                scroll_offset_y,
                            },
                        ) {
                            Ok(window) => window,
                            Err(error) => {
                                tracing::error!(?error, "could not prepare result-grid viewport");
                                return;
                            }
                        };
                        let visible_end = window.positions.end;
                        ui.add_space(window.positions.start as f32 * row_height);
                        for position in window.positions {
                            renderer.draw_row(
                                ui,
                                ResultGridRowInput {
                                    result: context.result,
                                    rows: &rows,
                                    position,
                                },
                            );
                        }
                        ui.add_space(
                            context.indexes.len().saturating_sub(visible_end) as f32 * row_height,
                        );
                    });
            });
        },
    );
}
