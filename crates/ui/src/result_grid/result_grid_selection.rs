//! Result-grid selection and keyboard navigation helpers.

use super::result_grid_view::GridSelectionLookup;
use super::*;

pub(super) struct GridNavigationContext<'a> {
    pub(super) data: &'a mut TableDataState,
    pub(super) editing: &'a mut TableEditingState,
    pub(super) feedback: &'a mut FeedbackState,
}

/// Arrow / Tab / Home / End navigation over the visible (filtered, sorted)
/// indexes and column order.
pub(super) fn handle_grid_navigation(
    ui: &mut egui::Ui,
    indexes: &[usize],
    order: &[usize],
    editable: bool,
    selection_lookup: &GridSelectionLookup,
    context: &mut GridNavigationContext<'_>,
) {
    if order.is_empty() || indexes.is_empty() {
        return;
    }

    let is_tab = ui.input(|input| input.key_pressed(egui::Key::Tab));
    let is_shift_tab = is_tab && ui.input(|input| input.modifiers.shift);

    // Committing an active edit remains a table-editor orchestration concern;
    // the caller performs it before entering this pure navigation transition.
    if is_tab && editable && context.editing.data_editing_cell.is_some() {
        return;
    }

    let navigation_key = if is_tab {
        Some(egui::Key::Tab)
    } else {
        ui.input(|input| {
            [
                egui::Key::ArrowUp,
                egui::Key::ArrowDown,
                egui::Key::ArrowLeft,
                egui::Key::ArrowRight,
                egui::Key::Home,
                egui::Key::End,
            ]
            .into_iter()
            .find(|key| input.key_pressed(*key))
        })
    };
    let Some(key) = navigation_key else {
        return;
    };

    if let Some(selection) = context
        .data
        .navigation_target(indexes, order, selection_lookup, key, is_shift_tab)
    {
        context.data.select_cell_range(
            indexes,
            &selection_lookup.row_positions,
            selection,
            ui.input(|input| input.modifiers.shift),
        );
        context.editing.data_editing_cell = None;
        context.editing.data_edit_value.clear();
        context.feedback.copy_status.clear();
    }
}
