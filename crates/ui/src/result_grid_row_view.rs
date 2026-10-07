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

/// Capture-harness hook: `DB_PRO_DEBUG_GRID_POINTER=x,y` fakes a pointer
/// position for the row-hover hit test, so screenshots can document the
/// row-hover state. `OnceLock` makes the env read once per process; `None`
/// is a single branch per row.
fn debug_pointer_pos() -> Option<egui::Pos2> {
    static POS: std::sync::OnceLock<Option<egui::Pos2>> = std::sync::OnceLock::new();
    *POS.get_or_init(|| {
        let raw = std::env::var("DB_PRO_DEBUG_GRID_POINTER").ok()?;
        let (x, y) = raw.split_once(',')?;
        Some(egui::Pos2::new(x.trim().parse().ok()?, y.trim().parse().ok()?))
    })
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
        // Row-level hover — the demo spec paints `hover:bg-…` on the whole row,
        // not per cell, so one hit-test on the row rect feeds every cell.
        let row_rect = egui::Rect::from_min_size(
            ui.available_rect_before_wrap().min,
            egui::vec2(ui.available_width(), 28.0),
        );
        let hover_pos = ui
            .ctx()
            .input(|input| input.pointer.hover_pos())
            .or_else(debug_pointer_pos);
        let row_hovered = hover_pos.is_some_and(|pos| row_rect.contains(pos));
        let origin_x = ui.cursor().left();
        let clip = ui.clip_rect();
        let window = crate::grid_column_window(
            context.rows.order,
            context.rows.widths,
            GRID_ROW_NUMBER_WIDTH,
            (clip.left() - origin_x).max(0.0),
            (clip.right() - origin_x).max(0.0),
        );
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

        if window.leading > 0.0 {
            ui.add_space(window.leading);
        }
        for &column_index in &context.rows.order[window.start..window.end] {
            let cell = context.row.get(column_index).unwrap_or(&UiCell::Null);
            let width = context.rows.widths.get(column_index).copied().unwrap_or(180.0);
            renderer.draw_cell(
                ui,
                GridCell {
                    visible_indexes: context.rows.indexes,
                    selection_lookup: context.rows.selection_lookup,
                    row_index: context.row_index,
                    column_index,
                    row_hovered,
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

        if window.trailing > 0.0 {
            let (skipped, _) = ui.allocate_exact_size(egui::vec2(window.trailing, 28.0), egui::Sense::hover());
            let fill = if context.row_selected {
                context.theme.soft_tint(context.theme.accent)
            } else if row_hovered {
                context.theme.surface_hover
            } else {
                context.theme.surface_editor
            };
            ui.painter().rect_filled(skipped, egui::Rounding::ZERO, fill);
            ui.painter().hline(
                skipped.x_range(),
                skipped.bottom(),
                egui::Stroke::new(1.0, context.theme.border_subtle),
            );
        }

        // Trailing filler — the row's bottom border continues to the grid edge.
        let rest = ui.available_width();
        if rest > 0.5 {
            let (fill_rect, _) = ui.allocate_exact_size(egui::vec2(rest, 28.0), egui::Sense::hover());
            let fill = if context.row_selected {
                context.theme.soft_tint(context.theme.accent)
            } else if row_hovered {
                context.theme.surface_hover
            } else {
                context.theme.surface_editor
            };
            ui.painter().rect_filled(fill_rect, egui::Rounding::ZERO, fill);
            ui.painter().hline(
                fill_rect.x_range(),
                fill_rect.bottom(),
                egui::Stroke::new(1.0, context.theme.border_subtle),
            );
        }
    });
}
