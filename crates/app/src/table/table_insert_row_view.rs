use super::*;

impl DbProApp {
    pub(crate) fn open_duplicate_row(&mut self, result: &UiQueryResult, row_index: usize) {
        if let Some(notice) = table_editor_context::surface_read_only_notice(self.edit_surface()) {
            self.feedback.runtime_message = notice.to_owned();
            return;
        }
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to insert rows".to_owned();
            return;
        }
        let Some(info) = self.table.state.table_info.clone() else {
            self.feedback.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        let Some(row) = result.rows.get(row_index) else {
            return;
        };
        self.table.editing.insert_row_values = table_editor_values::duplicate_row_values(&info, row);
        self.table.editing.insert_row_columns = info.columns.iter().map(|c| c.name.clone()).collect();
        self.table.editing.insert_row_error.clear();
        self.table.editing.insert_row_open = true;
    }

    pub(crate) fn open_insert_row(&mut self) {
        if let Some(notice) = table_editor_context::surface_read_only_notice(self.edit_surface()) {
            self.feedback.runtime_message = notice.to_owned();
            return;
        }
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to insert rows".to_owned();
            return;
        }
        let Some(info) = self.table.state.table_info.clone() else {
            self.feedback.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        self.table.editing.insert_row_values = vec![String::new(); info.columns.len()];
        self.table.editing.insert_row_columns = info.columns.iter().map(|c| c.name.clone()).collect();
        self.table.editing.insert_row_error.clear();
    }

    pub(crate) fn submit_insert_row(&mut self) {
        let Some(table) = self.schema.explorer.selected_table.clone() else {
            self.table.editing.insert_row_error = "Select a table before inserting a row".to_owned();
            return;
        };
        let Some(info) = self.table.state.table_info.clone() else {
            self.table.editing.insert_row_error = "Table structure is still loading".to_owned();
            return;
        };
        let (columns, values) =
            match table_editor_values::parse_insert_row_values(&info, &self.table.editing.insert_row_values) {
                Ok(parsed) => parsed,
                Err(error) => {
                    self.table.editing.insert_row_error = error;
                    return;
                }
            };
        self.table.mutation.staged_changes.ensure_target(&table);
        self.table.mutation.staged_changes.stage_insert(columns, values);
        self.table.editing.insert_row_open = false;
        self.table.editing.insert_row_error.clear();
        self.feedback.runtime_message = format!("Row staged for {}", table);
    }

    pub(super) fn draw_insert_row_dialog(&mut self, ctx: &egui::Context) {
        let Some(info) = self.table.state.table_info.clone() else {
            self.table.editing.insert_row_open = false;
            return;
        };
        let column_names: Vec<&str> = info.columns.iter().map(|c| c.name.as_str()).collect();
        if self.table.editing.insert_row_columns != column_names {
            // Schema refreshed under the open dialog: keep entered values by
            // column name; new columns start empty, dropped ones are lost.
            let old = std::mem::take(&mut self.table.editing.insert_row_values);
            let old_names = std::mem::take(&mut self.table.editing.insert_row_columns);
            self.table.editing.insert_row_values = info
                .columns
                .iter()
                .map(|col| {
                    old_names
                        .iter()
                        .position(|name| name == &col.name)
                        .and_then(|idx| old.get(idx).cloned())
                        .unwrap_or_default()
                })
                .collect();
            self.table.editing.insert_row_columns = info.columns.iter().map(|c| c.name.clone()).collect();
            self.table.editing.insert_row_error =
                "Table structure changed; values were kept by column name".to_owned();
        }
        let error = self.table.editing.insert_row_error.clone();
        let action = {
            let mut context = table_insert_row_surface_view::InsertRowDialogContext {
                theme: self.theme,
                info: &info,
                values: &mut self.table.editing.insert_row_values,
                error: &error,
            };
            table_insert_row_surface_view::draw_dialog(&mut context, ctx)
        };
        match action {
            Some(table_insert_row_surface_view::InsertRowDialogAction::Submit) => self.submit_insert_row(),
            Some(table_insert_row_surface_view::InsertRowDialogAction::Cancel) => {
                // Partially-filled values are expensive to retype — confirm
                // before clearing the form.
                if self
                    .table
                    .editing
                    .insert_row_values
                    .iter()
                    .any(|value| !value.trim().is_empty())
                {
                    self.table.editing.insert_row_discard_confirmation = true;
                } else {
                    self.table.editing.insert_row_open = false;
                    self.table.editing.insert_row_error.clear();
                }
            }
            None => {}
        }

        if self.table.editing.insert_row_discard_confirmation {
            let mut confirm_open = true;
            let confirm_action = crate::components::alert::AlertDialog::new(
                "Discard row values?",
                "The insert form has filled values. Closing clears them.",
                self.theme,
            )
            .confirm_label("Discard")
            .cancel_label("Keep Editing")
            .destructive(true)
            .id_salt("insert_row_discard")
            .show(ctx, &mut confirm_open);
            match confirm_action {
                Some(crate::components::alert::AlertDialogAction::Confirm) => {
                    self.table.editing.insert_row_discard_confirmation = false;
                    self.table.editing.insert_row_open = false;
                    self.table.editing.insert_row_values.iter_mut().for_each(String::clear);
                    self.table.editing.insert_row_error.clear();
                }
                Some(crate::components::alert::AlertDialogAction::Cancel) => {
                    self.table.editing.insert_row_discard_confirmation = false;
                }
                None => {}
            }
            if !confirm_open {
                self.table.editing.insert_row_discard_confirmation = false;
            }
        }
    }
}
