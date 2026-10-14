//! Query-shell layout policies.
use super::*;

pub(super) fn draw_visual_builder<F>(ui: &mut egui::Ui, theme: DbProTheme, open: bool, draw_builder: F)
where
    F: FnOnce(&mut egui::Ui),
{
    if !open {
        return;
    }
    ui.add_space(SPACE_XS);
    // The disclosure's own expanded state is separate from `open` (which gates the
    // whole section); persist it across frames so collapsing the header sticks.
    let id = egui::Id::new("query_visual_builder_disclosure");
    let mut expanded = ui.data(|data| data.get_temp::<bool>(id)).unwrap_or(true);
    let _ = Collapsible::new(&mut expanded, theme)
        .id(id)
        .title("Visual query builder")
        .show(ui, draw_builder);
    ui.data_mut(|data| data.insert_temp(id, expanded));
    ui.add_space(SPACE_XS);
}

pub(super) fn draw_editor_stack<F>(ui: &mut egui::Ui, editor_height: f32, draw_editor: F)
where
    F: FnOnce(&mut egui::Ui),
{
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), editor_height),
        Layout::top_down(Align::Min),
        draw_editor,
    );
}
