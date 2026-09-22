//! Query destructive-run, export, and SQL diagnostics helpers.
use super::query_view::write_file_atomically;
use super::*;
use std::path::PathBuf;

impl DbProApp {
    pub(super) fn draw_destructive_run_dialog(&mut self, ui: &mut egui::Ui) {
        let Some(pending) = self.query.execution.pending_destructive_run().cloned() else {
            return;
        };
        match (query_dialog_surface_view::DestructiveDialogContext {
            theme: self.theme,
            pending: &pending,
        })
        .draw(ui)
        {
            query_dialog_surface_view::DestructiveDialogAction::Confirm => self.confirm_pending_destructive_run(),
            query_dialog_surface_view::DestructiveDialogAction::Cancel => self.cancel_pending_destructive_run(),
        }
    }

    pub(super) fn draw_export_dialog(&mut self, ui: &mut egui::Ui, result: Option<&UiQueryResult>) {
        if !self.overlay.export_open {
            return;
        }
        let action = query_dialog_surface_view::ExportDialogContext {
            theme: self.theme,
            overlay: &mut self.overlay,
        }
        .draw(ui);
        match action {
            Some(query_dialog_surface_view::ExportDialogAction::Export) => {
                if let Some(result) = result {
                    self.export_result(result);
                }
            }
            Some(query_dialog_surface_view::ExportDialogAction::Cancel) => {
                self.overlay.export_open = false;
                self.overlay.export_overwrite_pending = false;
            }
            Some(query_dialog_surface_view::ExportDialogAction::Overwrite) => {
                if let Some(result) = result {
                    self.export_result_confirming_overwrite(result);
                }
            }
            Some(query_dialog_surface_view::ExportDialogAction::KeepExisting) => {
                self.overlay.export_overwrite_pending = false;
            }
            None => {}
        }
    }

    pub(crate) fn explain_query(&mut self) {
        self.dispatch_explain_query(false);
    }

    pub(crate) fn explain_query_analyze(&mut self) {
        self.dispatch_explain_query(true);
    }

    fn dispatch_explain_query(&mut self, analyze: bool) {
        let capabilities = self.query_capabilities();
        let connection_id = self.active_query_connection_id().map(str::to_owned);
        let Some(request) = self
            .query_explain_context()
            .prepare(connection_id, &capabilities, analyze)
        else {
            return;
        };
        let request_id = self.next_request_id();
        let command = UiCommand::ExplainQuery {
            request_id,
            connection_id: request.connection_id.clone(),
            sql: request.sql.clone(),
            analyze: request.analyze,
        };
        if self.dispatch_command(command) {
            self.query_explain_context().commit_dispatched(request_id, &request);
        }
    }

    fn query_explain_context(&mut self) -> query_explain_actions::QueryExplainContext<'_> {
        query_explain_actions::QueryExplainContext::new(
            &mut self.query.session,
            &mut self.query.execution,
            &mut self.query.output,
            &mut self.feedback,
        )
    }

    pub(crate) fn export_result(&mut self, result: &UiQueryResult) {
        self.export_result_to_disk(result, false);
    }

    /// Write the export after the user confirmed overwriting an existing file (#244, E-1).
    pub(crate) fn export_result_confirming_overwrite(&mut self, result: &UiQueryResult) {
        self.export_result_to_disk(result, true);
    }

    /// Export the visible result to `export_path`.
    ///
    /// #244 (E-1): the file is published atomically and an existing path is never replaced without
    /// asking, so a failed export cannot leave a truncated file that looks complete and a typo in the
    /// path cannot destroy an unrelated file.
    pub(super) fn export_result_to_disk(&mut self, result: &UiQueryResult, overwrite: bool) {
        let path_text = self.overlay.export_path.trim().to_owned();
        if path_text.is_empty() {
            self.feedback.runtime_message = "Choose an export path first".to_owned();
            return;
        }
        let path = PathBuf::from(&path_text);
        if path.exists() && !overwrite {
            self.overlay.export_overwrite_pending = true;
            return;
        }

        let delimiter = if self.overlay.export_format == "CSV" { "," } else { "\t" };
        let output = match self.overlay.export_format.as_str() {
            "SQL" => result_grid_export::format_result_sql_insert(result, "exported_rows"),
            "COPY" => result_grid_export::format_result_copy(result, "exported_rows"),
            "JSON" => result_grid_export::format_result_json(result),
            "MD" | "Markdown" => result_grid_export::format_result_markdown(result),
            _ => result_grid_export::format_result_delimited(result, delimiter),
        };
        let exported_rows = result.rows.len();
        match write_file_atomically(&path, output.as_bytes()) {
            Ok(()) => {
                // Truthful row count: a capped result exports the rows it holds, not the rows the
                // query matched, and the message has to say which one it is.
                self.feedback.runtime_message = if result.row_count > exported_rows as u64 {
                    format!(
                        "Exported {exported_rows} rows to {path_text} ({} rows matched; the result holds the first {exported_rows})",
                        result.row_count
                    )
                } else {
                    format!("Exported {exported_rows} rows to {path_text}")
                };
            }
            Err(error) => {
                self.feedback.runtime_message = format!("Export failed: {error} (no file was written)");
            }
        }
        self.overlay.export_overwrite_pending = false;
        self.overlay.export_open = false;
    }

    pub(super) fn save_query_document(&mut self) {
        self.save_query_document_at(self.query.session.active_document_index);
    }

    pub(crate) fn save_query_document_at(&mut self, document_index: usize) {
        if self
            .query
            .session
            .documents
            .get(document_index)
            .and_then(|document| document.file_path.as_ref())
            .is_some()
            && document_index == self.query.session.active_document_index
            && self.save_active_workspace_file()
        {
            return;
        }
        let Some(connection_id) = self
            .query
            .session
            .documents
            .get(document_index)
            .and_then(|document| document.connection_id.clone())
            .or_else(|| self.connection.lifecycle.active_connection_id().map(str::to_owned))
        else {
            self.feedback.runtime_message = "Create or select a connection first".to_owned();
            return;
        };
        let request_id = self.next_request_id();
        let Some(prepared) = self
            .query_save_context()
            .prepare_save(request_id, document_index, Some(connection_id))
        else {
            return;
        };
        let command = query_save_commands::save_query_command(&prepared);
        if self.dispatch_command(command) {
            self.query_save_context().commit_dispatched(request_id, document_index);
        }
    }

    fn query_save_context(&mut self) -> query_save_actions::QuerySaveContext<'_> {
        query_save_actions::QuerySaveContext::new(&mut self.query.session, &self.query.library, &mut self.feedback)
    }

    pub(crate) fn open_save_as_dialog(&mut self) {
        self.query.session.save_as_name = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .map_or_else(|| "Saved query".to_owned(), |document| document.title.clone());
        self.query.session.save_as_open = true;
    }

    pub(super) fn draw_save_as_dialog(&mut self, ctx: &egui::Context) {
        if !self.query.session.save_as_open {
            return;
        }
        let action = query_save_dialog_surface_view::SaveAsDialogContext {
            theme: self.theme,
            name: &mut self.query.session.save_as_name,
        }
        .draw(ctx);
        match action {
            Some(query_save_dialog_surface_view::SaveAsDialogAction::Cancel) => {
                self.query.session.save_as_open = false;
            }
            Some(query_save_dialog_surface_view::SaveAsDialogAction::Save) => {
                let name = self.query.session.save_as_name.trim().to_owned();
                if name.is_empty() {
                    self.feedback.runtime_message = "Enter a name for the saved query".to_owned();
                    return;
                }
                let document_index = self.query.session.active_document_index;
                if let Some(document) = self.query.session.documents.get_mut(document_index) {
                    document.saved_query_id = None;
                    document.title = name;
                }
                self.query.session.save_as_open = false;
                self.save_query_document_at(document_index);
            }
            None => {}
        }
    }

    pub(super) fn draw_dirty_close_dialog(&mut self, ctx: &egui::Context) {
        let Some(document_index) = self.query.session.pending_dirty_close else {
            return;
        };
        let Some(title) = self
            .query
            .session
            .documents
            .get(document_index)
            .map(|document| document.title.clone())
        else {
            self.query.session.pending_dirty_close = None;
            return;
        };
        let action = query_save_dialog_surface_view::DirtyCloseDialogContext {
            theme: self.theme,
            title: &title,
        }
        .draw(ctx);
        match action {
            Some(query_save_dialog_surface_view::DirtyCloseDialogAction::Cancel) => {
                self.query.session.pending_dirty_close = None;
            }
            Some(query_save_dialog_surface_view::DirtyCloseDialogAction::Discard) => {
                self.query.session.pending_dirty_close = None;
                self.close_query_document(document_index);
            }
            Some(query_save_dialog_surface_view::DirtyCloseDialogAction::Save) => {
                self.query.session.pending_dirty_close = None;
                self.query.session.pending_close_after_save = Some(document_index);
                self.save_query_document_at(document_index);
            }
            None => {}
        }
    }
}
