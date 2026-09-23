//! Query-shell layout policies.
use super::*;

pub(super) fn draw_visual_builder<F>(ui: &mut egui::Ui, open: bool, draw_builder: F)
where
    F: FnOnce(&mut egui::Ui),
{
    if !open {
        return;
    }
    ui.add_space(SPACE_XS);
    egui::CollapsingHeader::new("Visual query builder")
        .default_open(true)
        .show(ui, draw_builder);
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
