//! Query destructive-run, export, and SQL diagnostics helpers.
use super::query_view::write_file_atomically;
use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use egui::RichText;
use lucide_icons::Icon;
use std::path::PathBuf;

impl DbProApp {
    pub(super) fn draw_destructive_run_dialog(&mut self, ui: &mut egui::Ui) {
        let Some(pending) = self.query.execution.pending_destructive_run.clone() else {
            return;
        };
        const PREVIEW_CHARS: usize = 600;
        let mut open = true;
        let mut confirmed = false;
        let mut cancelled = false;
        Dialog::new(&mut open, "Run Destructive Statement?", self.theme)
            .id_salt("destructive_run_dialog")
            .width(560.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(if pending.all_statements {
                        "The script you are about to run contains a statement that can drop or truncate data. Nothing has been sent yet."
                    } else {
                        "This statement can drop or truncate data. Nothing has been sent yet."
                    })
                    .color(self.theme.text_primary),
                );
                ui.add_space(SPACE_SM);
                let mut preview = pending.sql.clone();
                if preview.chars().count() > PREVIEW_CHARS {
                    preview = preview.chars().take(PREVIEW_CHARS).collect::<String>() + "…";
                }
                editor_frame(self.theme).show(ui, |ui| {
                    ui.label(RichText::new(preview).font(font_mono_sm()).color(self.theme.text_secondary));
                });
                ui.add_space(SPACE_SM);
                ui.colored_label(
                    self.theme.warning,
                    "It is sent to the server exactly as written; the app cannot undo it.",
                );
                ui.add_space(SPACE_MD);
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .text("Run Destructive Statement")
                        .variant(ButtonVariant::Destructive)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        confirmed = true;
                    }
                    if Button::new(self.theme)
                        .text("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        cancelled = true;
                    }
                });
            });
        if confirmed {
            self.confirm_pending_destructive_run();
        } else if cancelled || !open {
            self.cancel_pending_destructive_run();
        }
    }

    pub(super) fn draw_export_dialog(&mut self, ui: &mut egui::Ui, result: Option<&UiQueryResult>) {
        if !self.overlay.export_open {
            return;
        }
        card_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Export results");
                ui.selectable_value(&mut self.overlay.export_format, "CSV".to_owned(), "CSV");
                ui.selectable_value(&mut self.overlay.export_format, "TSV".to_owned(), "TSV");
                ui.selectable_value(&mut self.overlay.export_format, "SQL".to_owned(), "INSERT");
                ui.selectable_value(&mut self.overlay.export_format, "COPY".to_owned(), "COPY");
                input(ui, &mut self.overlay.export_path, "output path", 260.0, self.theme);
                if Button::new(self.theme)
                    .text("Export")
                    .variant(ButtonVariant::Default)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    if let Some(result) = result {
                        self.export_result(result);
                    }
                }
                if Button::new(self.theme)
                    .text("Cancel")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.overlay.export_open = false;
                    self.overlay.export_overwrite_pending = false;
                }
            });
            if self.overlay.export_overwrite_pending {
                ui.add_space(SPACE_XS);
                ui.colored_label(
                    self.theme.warning,
                    format!("{} already exists. Overwrite it?", self.overlay.export_path.trim()),
                );
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .text("Overwrite")
                        .variant(ButtonVariant::Destructive)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        if let Some(result) = result {
                            self.export_result_confirming_overwrite(result);
                        }
                    }
                    if Button::new(self.theme)
                        .text("Keep existing file")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.overlay.export_overwrite_pending = false;
                    }
                });
            }
        });
    }

    pub(crate) fn explain_query(&mut self) {
        self.dispatch_explain_query(false);
    }

    pub(crate) fn explain_query_analyze(&mut self) {
        self.dispatch_explain_query(true);
    }

    fn dispatch_explain_query(&mut self, analyze: bool) {
        if self.query.session.active_explain_request().is_some() {
            return;
        }
        let lookup = self.query_capabilities();
        if let Some(reason) = lookup.feature_limitation(db_pro_core::domain::capabilities::CapabilityFeature::Explain) {
            self.feedback.runtime_message = format!("Explain is unavailable: {reason}");
            return;
        }
        let Some(connection_id) = self.active_query_connection_id().map(str::to_owned) else {
            self.feedback.runtime_message = "Connect to a database before explaining a query".to_owned();
            return;
        };
        let sql = if self.query.session.selected_text.trim().is_empty() {
            self.query.session.active_text().trim().to_owned()
        } else {
            self.query.session.selected_text.trim().to_owned()
        };
        if sql.is_empty() {
            self.feedback.runtime_message = "Enter a query before explaining it".to_owned();
            return;
        }
        if analyze && !self.query.execution.explain_analyze_confirmed {
            self.query.execution.pending_explain_analyze = true;
            self.query.output.set_active_for_optional_document(
                self.query
                    .session
                    .active_document()
                    .map(|document| document.id.as_str()),
                OutputTab::Explain,
            );
            self.feedback.runtime_message =
                "EXPLAIN ANALYZE executes the statement — confirm in the Explain pane".to_owned();
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        if self
            .task_bridge
            .send(UiCommand::ExplainQuery {
                request_id,
                connection_id,
                sql,
                analyze,
            })
            .is_ok()
        {
            let doc_index = self.query.session.active_document_index;
            if let Some(doc) = self.query.session.documents.get_mut(doc_index) {
                doc.explain_request = Some(request_id);
                doc.explain_plan = None;
            }
            self.query.execution.pending_explain_analyze = false;
            self.query.execution.explain_analyze_confirmed = false;
            self.query.output.set_active_for_optional_document(
                self.query
                    .session
                    .active_document()
                    .map(|document| document.id.as_str()),
                OutputTab::Explain,
            );
            self.feedback.runtime_message = if analyze {
                "EXPLAIN ANALYZE running (query executes)…".to_owned()
            } else {
                "Explaining query…".to_owned()
            };
        }
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
        let request_id = self.task_bridge.next_request_id();
        let document_id = self
            .query
            .session
            .documents
            .get(document_index)
            .map(|document| document.id.clone());
        let name = self
            .query
            .session
            .documents
            .get(document_index)
            .map(|document| document.title.clone())
            .unwrap_or_else(|| "Saved query".to_owned());
        let saved_query_id = self
            .query
            .session
            .documents
            .get(document_index)
            .and_then(|document| document.saved_query_id.clone());
        let sql = self
            .query
            .session
            .documents
            .get(document_index)
            .map_or_else(String::new, |document| document.text().to_owned());
        self.dispatch_command(self.query.library.save_query_command(
            request_id,
            connection_id,
            saved_query_id,
            name,
            sql,
        ));
        if let Some(document_id) = document_id {
            self.query.session.save_requests.insert(request_id, document_id);
        }
        self.feedback.runtime_message = "Saving query…".to_owned();
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
        let mut save = false;
        let mut cancel = false;
        egui::Window::new("Save Query As")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut self.query.session.save_as_name);
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .icon(Icon::Save)
                        .text("Save")
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        save = true;
                    }
                    if Button::new(self.theme)
                        .icon(Icon::X)
                        .text("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        cancel = true;
                    }
                });
            });
        if cancel {
            self.query.session.save_as_open = false;
        } else if save {
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
        let mut save = false;
        let mut discard = false;
        let mut cancel = false;
        egui::Window::new("Unsaved query")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("Save changes to {title} before closing?"));
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .icon(Icon::Save)
                        .text("Save")
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        save = true;
                    }
                    if Button::new(self.theme)
                        .icon(Icon::Trash2)
                        .text("Don't Save")
                        .variant(ButtonVariant::Destructive)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        discard = true;
                    }
                    if Button::new(self.theme)
                        .icon(Icon::X)
                        .text("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        cancel = true;
                    }
                });
            });
        if cancel {
            self.query.session.pending_dirty_close = None;
        } else if discard {
            self.query.session.pending_dirty_close = None;
            self.close_query_document(document_index);
        } else if save {
            self.query.session.pending_dirty_close = None;
            self.query.session.pending_close_after_save = Some(document_index);
            self.save_query_document_at(document_index);
        }
    }
}
