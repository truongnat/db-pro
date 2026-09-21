//! Inline result-cell editor presentation and typed keyboard actions.
use super::super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CellEditorAction {
    Commit,
    Cancel,
}

pub(super) struct CellEditorContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) is_boolean: bool,
    pub(super) value: &'a mut String,
    pub(super) error: &'a mut Option<String>,
}

pub(super) fn draw(
    context: &mut CellEditorContext<'_>,
    ui: &mut egui::Ui,
    cell_rect: egui::Rect,
) -> Option<CellEditorAction> {
    let response = draw_input(context, ui, cell_rect);
    response.request_focus();
    if response.changed() {
        *context.error = None;
    }
    keyboard_action(ui)
}

fn draw_input(context: &mut CellEditorContext<'_>, ui: &mut egui::Ui, cell_rect: egui::Rect) -> egui::Response {
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(cell_rect.shrink(1.0)), |ui| {
        if context.is_boolean {
            let mut checked = context.value.eq_ignore_ascii_case("true");
            let response = ui.checkbox(&mut checked, "");
            if response.changed() {
                *context.value = checked.to_string();
                *context.error = None;
            }
            response
        } else {
            ui.add_sized(
                ui.available_size(),
                egui::TextEdit::singleline(context.value)
                    .margin(egui::Margin::symmetric(6.0, 2.0))
                    .text_color(context.theme.text_primary),
            )
        }
    })
    .inner
}

fn keyboard_action(ui: &egui::Ui) -> Option<CellEditorAction> {
    if ui.input(|input| input.key_pressed(egui::Key::Enter)) {
        Some(CellEditorAction::Commit)
    } else if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        Some(CellEditorAction::Cancel)
    } else {
        None
    }
}
