//! In-cell editor for the result grid.
use super::*;
use egui::Rect;

impl DbProApp {
    pub(super) fn draw_grid_cell_editor(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
        cell_rect: Rect,
    ) {
        let is_boolean = result
            .columns
            .get(column_index)
            .is_some_and(|column| column.data_type.to_ascii_lowercase().contains("bool"));
        let is_expanded = self.expanded_data_editor == Some((row_index, column_index));
        if !is_expanded {
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(cell_rect.shrink(1.0)), |ui| {
                let response = if is_boolean {
                    let mut checked = self.data_edit_value.eq_ignore_ascii_case("true");
                    let response = ui.checkbox(&mut checked, "");
                    if response.changed() {
                        self.data_edit_value = checked.to_string();
                        self.data_edit_error = None;
                    }
                    response
                } else {
                    ui.add_sized(
                        ui.available_size(),
                        TextEdit::singleline(&mut self.data_edit_value)
                            .margin(egui::Margin::symmetric(6.0, 2.0))
                            .text_color(self.theme.text_primary),
                    )
                };
                response.request_focus();
                if response.changed() {
                    self.data_edit_error = None;
                }
            });
            let commit = ui.input(|input| input.key_pressed(egui::Key::Enter));
            if commit {
                self.submit_data_cell_edit(result, row_index, column_index);
            } else if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                self.data_editing_cell = None;
                self.expanded_data_editor = None;
                self.data_edit_value.clear();
                self.data_edit_error = None;
            }
            return;
        }

        let ctx = ui.ctx().clone();
        let mut open = true;
        let mut commit = false;
        let mut cancel = false;
        egui::Window::new("Expanded cell editor")
            .id(egui::Id::new(("expanded-table-cell-editor", row_index, column_index)))
            .open(&mut open)
            .resizable(true)
            .default_width(520.0)
            .show(&ctx, |ui| {
                ui.label(
                    RichText::new("Edit value · Enter applies, Escape cancels")
                        .small()
                        .color(self.theme.text_muted),
                );
                let response = ui.add(
                    TextEdit::multiline(&mut self.data_edit_value)
                        .desired_width(ui.available_width())
                        .desired_rows(14),
                );
                if response.changed() {
                    self.data_edit_error = None;
                }
                if let Some(error) = self.data_edit_error.as_deref() {
                    ui.label(RichText::new(error).small().color(self.theme.danger));
                }
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        commit = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        if commit {
            self.submit_data_cell_edit(result, row_index, column_index);
        } else if cancel || !open || ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.data_editing_cell = None;
            self.expanded_data_editor = None;
            self.data_edit_value.clear();
            self.data_edit_error = None;
        }
    }

    pub(super) fn commit_active_data_edit(&mut self, result: &UiQueryResult) -> bool {
        if let Some((row_index, column_index)) = self.data_editing_cell {
            self.submit_data_cell_edit(result, row_index, column_index)
        } else {
            true
        }
    }
}
