use super::*;

impl DbProApp {
    pub(crate) fn open_duplicate_row(&mut self, result: &UiQueryResult, row_index: usize) {
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
        self.table.editing.insert_row_error.clear();
        self.table.editing.insert_row_open = true;
    }

    pub(crate) fn open_insert_row(&mut self) {
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to insert rows".to_owned();
            return;
        }
        let Some(info) = self.table.state.table_info.clone() else {
            self.feedback.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        self.table.editing.insert_row_values = vec![String::new(); info.columns.len()];
        self.table.editing.insert_row_error.clear();
        self.table.editing.insert_row_open = true;
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
        if self.table.editing.insert_row_values.len() != info.columns.len() {
            self.table.editing.insert_row_values = vec![String::new(); info.columns.len()];
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
                self.table.editing.insert_row_open = false;
                self.table.editing.insert_row_error.clear();
            }
            None => {}
        }
    }
}
