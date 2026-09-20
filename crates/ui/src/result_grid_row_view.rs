use super::result_grid_view::{GridCell, GridRows};
use super::*;

pub(super) struct GridRowSurfaceContext<'a> {
    pub(super) row: &'a [UiCell],
    pub(super) rows: &'a GridRows<'a>,
    pub(super) row_index: usize,
    pub(super) display_position: usize,
    pub(super) row_selected: bool,
    pub(super) row_dirty: bool,
    pub(super) row_mutation_error: bool,
    pub(super) cell_mutation_errors: &'a [bool],
    pub(super) theme: DbProTheme,
}

pub(super) trait GridRowSurfaceRenderer {
    fn on_gutter_click(&mut self, ui: &mut egui::Ui, rows: &GridRows<'_>, position: usize) -> bool;
    fn draw_cell(&mut self, ui: &mut egui::Ui, cell: GridCell<'_>);
}

pub(super) fn draw_row(
    ui: &mut egui::Ui,
    context: GridRowSurfaceContext<'_>,
    renderer: &mut impl GridRowSurfaceRenderer,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
        let row_number = crate::displayed_row_number(context.rows.row_offset, context.row_index);
        let gutter_response = result_grid_row_gutter_view::draw_row_gutter(
            ui,
            result_grid_row_gutter_view::GridRowGutterContext {
                theme: context.theme,
                row_number,
                selected: context.row_selected,
            },
        );
        if gutter_response.clicked() && !renderer.on_gutter_click(ui, context.rows, context.display_position) {
            return;
        }

        for &column_index in context.rows.order {
            let cell = context.row.get(column_index).unwrap_or(&UiCell::Null);
            let width = context.rows.widths.get(column_index).copied().unwrap_or(180.0);
            renderer.draw_cell(
                ui,
                GridCell {
                    visible_indexes: context.rows.indexes,
                    selection_lookup: context.rows.selection_lookup,
                    row_index: context.row_index,
                    column_index,
                    display_position: context.display_position,
                    row_selected: context.row_selected,
                    row_dirty: context.row_dirty,
                    row_mutation_error: context.row_mutation_error,
                    cell_mutation_error: context.cell_mutation_errors.get(column_index).copied().unwrap_or(false),
                    editable: context.rows.editable,
                    width,
                    cell,
                },
            );
        }
    });
}
