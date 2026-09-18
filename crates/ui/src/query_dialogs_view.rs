//! Query destructive-run, export, and SQL diagnostics helpers.
use super::query_view::{deduplicate_diagnostics, deduplicate_messages, format_query_document, write_file_atomically};
use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::editor::{Diagnostic, SqlDialect};
use egui::RichText;
use lucide_icons::Icon;
use std::path::PathBuf;
use std::time::{Duration, Instant};

impl DbProApp {
    pub(super) fn draw_destructive_run_dialog(&mut self, ui: &mut egui::Ui) {
        let Some(pending) = self.pending_destructive_run.clone() else {
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
        if !self.export_open {
            return;
        }
        card_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Export results");
                ui.selectable_value(&mut self.export_format, "CSV".to_owned(), "CSV");
                ui.selectable_value(&mut self.export_format, "TSV".to_owned(), "TSV");
                ui.selectable_value(&mut self.export_format, "SQL".to_owned(), "INSERT");
                ui.selectable_value(&mut self.export_format, "COPY".to_owned(), "COPY");
                input(ui, &mut self.export_path, "output path", 260.0, self.theme);
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
                    self.export_open = false;
                    self.export_overwrite_pending = false;
                }
            });
            if self.export_overwrite_pending {
                ui.add_space(SPACE_XS);
                ui.colored_label(
                    self.theme.warning,
                    format!("{} already exists. Overwrite it?", self.export_path.trim()),
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
                        self.export_overwrite_pending = false;
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
        if self.active_explain_request().is_some() {
            return;
        }
        let lookup = self.query_capabilities();
        if let Some(reason) = lookup.feature_limitation(db_pro_core::domain::capabilities::CapabilityFeature::Explain) {
            self.runtime_message = format!("Explain is unavailable: {reason}");
            return;
        }
        let Some(connection_id) = self.active_query_connection_id().map(str::to_owned) else {
            self.runtime_message = "Connect to a database before explaining a query".to_owned();
            return;
        };
        let sql = if self.selected_query.trim().is_empty() {
            self.active_query_text().trim().to_owned()
        } else {
            self.selected_query.trim().to_owned()
        };
        if sql.is_empty() {
            self.runtime_message = "Enter a query before explaining it".to_owned();
            return;
        }
        if analyze && !self.explain_analyze_confirmed {
            self.pending_explain_analyze = true;
            self.set_active_query_output_tab(OutputTab::Explain);
            self.runtime_message = "EXPLAIN ANALYZE executes the statement — confirm in the Explain pane".to_owned();
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
            let doc_index = self.active_query_document;
            if let Some(doc) = self.query_documents.get_mut(doc_index) {
                doc.explain_request = Some(request_id);
                doc.explain_plan = None;
            }
            self.pending_explain_analyze = false;
            self.explain_analyze_confirmed = false;
            self.set_active_query_output_tab(OutputTab::Explain);
            self.runtime_message = if analyze {
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
        let path_text = self.export_path.trim().to_owned();
        if path_text.is_empty() {
            self.runtime_message = "Choose an export path first".to_owned();
            return;
        }
        let path = PathBuf::from(&path_text);
        if path.exists() && !overwrite {
            self.export_overwrite_pending = true;
            return;
        }

        let delimiter = if self.export_format == "CSV" { "," } else { "\t" };
        let output = match self.export_format.as_str() {
            "SQL" => DbProApp::format_result_sql_insert(result, "exported_rows"),
            "COPY" => DbProApp::format_result_copy(result, "exported_rows"),
            _ => DbProApp::format_result_delimited(result, delimiter),
        };
        let exported_rows = result.rows.len();
        match write_file_atomically(&path, output.as_bytes()) {
            Ok(()) => {
                // Truthful row count: a capped result exports the rows it holds, not the rows the
                // query matched, and the message has to say which one it is.
                self.runtime_message = if result.row_count > exported_rows as u64 {
                    format!(
                        "Exported {exported_rows} rows to {path_text} ({} rows matched; the result holds the first {exported_rows})",
                        result.row_count
                    )
                } else {
                    format!("Exported {exported_rows} rows to {path_text}")
                };
            }
            Err(error) => {
                self.runtime_message = format!("Export failed: {error} (no file was written)");
            }
        }
        self.export_overwrite_pending = false;
        self.export_open = false;
    }

    pub(crate) fn format_active_query(&mut self) {
        let doc_index = self.active_query_document;
        self.cancel_prediction_for_document(doc_index);
        let dialect = if self.query_capabilities().allows(|caps| caps.query.numbered_parameters) {
            SqlDialect::Postgres
        } else {
            SqlDialect::SQLite
        };
        if let Some(doc) = self.query_documents.get_mut(doc_index) {
            format_query_document(doc, dialect);
        }
    }

    #[cfg(test)]
    pub(crate) fn analyze_sql_diagnostics(sql: &str, driver: &str) -> (Vec<String>, Vec<Diagnostic>) {
        Self::analyze_sql_diagnostics_with_lint(sql, driver, &SqlLintSettings::default())
    }

    pub(crate) fn analyze_sql_diagnostics_with_lint(
        sql: &str,
        driver: &str,
        lint: &SqlLintSettings,
    ) -> (Vec<String>, Vec<Diagnostic>) {
        let mut string_diagnostics = Vec::new();
        let mut structured_diagnostics = Vec::new();
        let capabilities = match CapabilityLookup::for_driver_label(driver) {
            CapabilityLookup::Supported(caps) => Some(caps),
            CapabilityLookup::NoActiveConnection | CapabilityLookup::UnsupportedDriver { .. } => None,
        };
        let parse_result = if capabilities.as_ref().is_some_and(|caps| caps.query.numbered_parameters) {
            Parser::parse_sql(&PostgreSqlDialect {}, sql)
        } else if driver.eq_ignore_ascii_case("mysql") {
            // MySQL shares the positional editor dialect; GenericDialect is the closest
            // sqlparser stand-in until a dedicated MySQL dialect is wired.
            Parser::parse_sql(&GenericDialect {}, sql)
        } else {
            Parser::parse_sql(&SQLiteDialect {}, sql)
        };
        if let Err(error) = parse_result {
            let msg = format!("SQL parser: {error}");
            string_diagnostics.push(msg.clone());
            structured_diagnostics.push(Diagnostic::error((0, sql.len().clamp(1, 4)), msg));
        }
        if sql.trim().is_empty() {
            string_diagnostics.push("Query is empty".to_owned());
            return (string_diagnostics, structured_diagnostics);
        }
        for issue in crate::editor::brackets::structural_delimiter_issues(sql) {
            let end = issue.offset + sql[issue.offset..].chars().next().map_or(1, char::len_utf8);
            let message = if let Some(expected) = issue.expected {
                format!("Mismatched delimiter {}: expected {}", issue.character, expected)
            } else {
                format!("Unmatched delimiter {}", issue.character)
            };
            string_diagnostics.push(message.clone());
            structured_diagnostics.push(Diagnostic::delimiter((issue.offset, end), message));
        }
        Self::append_sql_lint_diagnostics(sql, lint, &mut string_diagnostics, &mut structured_diagnostics);
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_string = false;
        let mut string_start_byte = 0;

        for (byte_offset, ch) in sql.char_indices() {
            if ch == '\'' {
                if !in_string {
                    in_string = true;
                    string_start_byte = byte_offset;
                } else {
                    in_string = false;
                }
                current.push(ch);
            } else if in_string {
                current.push(ch);
            } else if matches!(ch, '(' | ')') {
                tokens.push((current.to_lowercase(), byte_offset));
                current.clear();
            } else if ch.is_whitespace() || ch == ';' || ch == ',' {
                if !current.is_empty() {
                    tokens.push((current.to_lowercase(), byte_offset - current.len()));
                    current.clear();
                }
            } else {
                current.push(ch);
            }
        }
        if !current.is_empty() {
            tokens.push((current.to_lowercase(), sql.len() - current.len()));
        }
        if in_string {
            let msg = "Unclosed string literal".to_owned();
            string_diagnostics.push(msg.clone());
            structured_diagnostics.push(Diagnostic::error((string_start_byte, sql.len()), msg));
        }
        if lint.allows("lint.update-no-where")
            && tokens.first().map(|(t, _)| t.as_str()) == Some("update")
            && !tokens.iter().any(|(t, _)| t == "where")
        {
            let msg = "UPDATE without WHERE will affect every row".to_owned();
            string_diagnostics.push(msg.clone());
            structured_diagnostics.push(Diagnostic::lint((0, sql.len().min(6)), msg, "lint.update-no-where"));
        }
        // Dialect capability mismatches — distinct from lint; these are provider contract errors.
        if !capabilities.as_ref().is_some_and(|caps| caps.query.ilike) {
            if let Some((_, offset)) = tokens.iter().find(|(t, _)| t == "ilike") {
                let msg = "ILIKE is not supported for this provider; use LIKE or lower()".to_owned();
                string_diagnostics.push(msg.clone());
                structured_diagnostics.push(Diagnostic::error((*offset, offset + 5), msg));
            }
        }
        if !capabilities.as_ref().is_some_and(|caps| caps.query.glob) {
            if let Some((_, offset)) = tokens.iter().find(|(t, _)| t == "glob") {
                let msg = "GLOB is not supported for this provider; use LIKE instead".to_owned();
                string_diagnostics.push(msg.clone());
                structured_diagnostics.push(Diagnostic::error((*offset, offset + 4), msg));
            }
        }
        let lower = sql.to_lowercase();
        if lower.contains("select * from") && lower.contains("select * from select") {
            let msg = "Subquery must be enclosed in parentheses".to_owned();
            string_diagnostics.push(msg.clone());
            structured_diagnostics.push(Diagnostic::error((0, sql.len()), msg));
        }

        (
            deduplicate_messages(string_diagnostics),
            deduplicate_diagnostics(structured_diagnostics),
        )
    }
    pub(super) fn save_query_document(&mut self) {
        self.save_query_document_at(self.active_query_document);
    }

    pub(crate) fn save_query_document_at(&mut self, document_index: usize) {
        if self
            .query_documents
            .get(document_index)
            .and_then(|document| document.file_path.as_ref())
            .is_some()
            && document_index == self.active_query_document
            && self.save_active_workspace_file()
        {
            return;
        }
        let Some(connection_id) = self
            .query_documents
            .get(document_index)
            .and_then(|document| document.connection_id.clone())
            .or_else(|| self.active_connection_id.clone())
        else {
            self.runtime_message = "Create or select a connection first".to_owned();
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let document_id = self
            .query_documents
            .get(document_index)
            .map(|document| document.id.clone());
        let name = self
            .query_documents
            .get(document_index)
            .map(|document| document.title.clone())
            .unwrap_or_else(|| "Saved query".to_owned());
        let saved_query_id = self
            .query_documents
            .get(document_index)
            .and_then(|document| document.saved_query_id.clone());
        let sql = self
            .query_documents
            .get(document_index)
            .map_or_else(String::new, |document| document.text().to_owned());
        self.dispatch_command(UiCommand::SaveQuery {
            request_id,
            connection_id,
            saved_query_id,
            name,
            sql,
            folder: (!self.query_folder.trim().is_empty()).then(|| self.query_folder.trim().to_owned()),
        });
        if let Some(document_id) = document_id {
            self.query_save_requests.insert(request_id, document_id);
        }
        self.runtime_message = "Saving query…".to_owned();
    }

    pub(crate) fn open_save_as_dialog(&mut self) {
        self.save_as_name = self
            .query_documents
            .get(self.active_query_document)
            .map_or_else(|| "Saved query".to_owned(), |document| document.title.clone());
        self.save_as_open = true;
    }

    pub(super) fn draw_save_as_dialog(&mut self, ctx: &egui::Context) {
        if !self.save_as_open {
            return;
        }
        let mut save = false;
        let mut cancel = false;
        egui::Window::new("Save Query As")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut self.save_as_name);
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
            self.save_as_open = false;
        } else if save {
            let name = self.save_as_name.trim().to_owned();
            if name.is_empty() {
                self.runtime_message = "Enter a name for the saved query".to_owned();
                return;
            }
            let document_index = self.active_query_document;
            if let Some(document) = self.query_documents.get_mut(document_index) {
                document.saved_query_id = None;
                document.title = name;
            }
            self.save_as_open = false;
            self.save_query_document_at(document_index);
        }
    }

    pub(super) fn draw_dirty_close_dialog(&mut self, ctx: &egui::Context) {
        let Some(document_index) = self.pending_dirty_close else {
            return;
        };
        let Some(title) = self
            .query_documents
            .get(document_index)
            .map(|document| document.title.clone())
        else {
            self.pending_dirty_close = None;
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
            self.pending_dirty_close = None;
        } else if discard {
            self.pending_dirty_close = None;
            self.close_query_document(document_index);
        } else if save {
            self.pending_dirty_close = None;
            self.pending_close_after_save = Some(document_index);
            self.save_query_document_at(document_index);
        }
    }

    /// Editor entries: find, font size, completion, snippets and folder creation.
    /// Returns true when the menu should close.
    pub(crate) fn append_sql_lint_diagnostics(
        sql: &str,
        lint: &SqlLintSettings,
        string_diagnostics: &mut Vec<String>,
        structured_diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !lint.enabled {
            return;
        }
        let lower = sql.to_lowercase();
        // SELECT * — warn on the star token when it is a projection wildcard.
        if lint.allows("lint.select-star") {
            if let Some(star_at) = lower.find("select") {
                let after = &lower[star_at..];
                if let Some(rel) = after.find('*') {
                    let abs = star_at + rel;
                    let before_ok = after[..rel].chars().rev().find(|c| !c.is_whitespace()).is_some();
                    let from_follows = after[rel..].contains("from");
                    if before_ok && from_follows {
                        let msg = "SELECT * makes column contracts brittle; prefer an explicit column list".to_owned();
                        string_diagnostics.push(msg.clone());
                        structured_diagnostics.push(Diagnostic::lint((abs, abs + 1), msg, "lint.select-star"));
                    }
                }
            }
        }
        // = NULL / != NULL / <> NULL — always unknown in SQL; suggest IS [NOT] NULL.
        if lint.allows("lint.null-compare") {
            for (needle, suggestion) in [
                ("= null", "IS NULL"),
                ("!= null", "IS NOT NULL"),
                ("<> null", "IS NOT NULL"),
                ("=null", "IS NULL"),
                ("!=null", "IS NOT NULL"),
                ("<>null", "IS NOT NULL"),
            ] {
                if let Some(at) = lower.find(needle) {
                    let msg = format!("Comparing with NULL using {needle} is always unknown; use {suggestion}");
                    string_diagnostics.push(msg.clone());
                    structured_diagnostics.push(Diagnostic::lint_with_fix(
                        (at, at + needle.len()),
                        msg,
                        "lint.null-compare",
                        suggestion,
                    ));
                }
            }
        }
        // DELETE without WHERE.
        let trimmed = lower.trim_start();
        if lint.allows("lint.delete-no-where") && trimmed.starts_with("delete") && !lower.contains("where") {
            let msg = "DELETE without WHERE will remove every row".to_owned();
            string_diagnostics.push(msg.clone());
            structured_diagnostics.push(Diagnostic::lint((0, sql.len().min(6)), msg, "lint.delete-no-where"));
        }
        // ORDER BY n — positional ordinals are brittle across projection changes.
        if lint.allows("lint.order-by-ordinal") {
            if let Some(order_at) = lower.find("order by") {
                let after = &lower[order_at + "order by".len()..];
                let trimmed_after = after.trim_start();
                let skip = after.len() - trimmed_after.len();
                if let Some(first) = trimmed_after.chars().next() {
                    if first.is_ascii_digit() {
                        let abs = order_at + "order by".len() + skip;
                        let end = abs
                            + trimmed_after
                                .chars()
                                .take_while(|c| c.is_ascii_digit() || *c == ',' || c.is_whitespace())
                                .map(char::len_utf8)
                                .sum::<usize>();
                        let msg = "ORDER BY ordinal is brittle; prefer an explicit column or expression".to_owned();
                        string_diagnostics.push(msg.clone());
                        structured_diagnostics.push(Diagnostic::lint(
                            (abs, end.max(abs + 1)),
                            msg,
                            "lint.order-by-ordinal",
                        ));
                    }
                }
            }
        }
        // FROM a, b — classic comma join / cartesian-product pattern when JOIN is absent.
        if lint.allows("lint.comma-join") {
            if let Some(from_at) = lower.find("from") {
                let after_from = &lower[from_at + 4..];
                let has_join = after_from.contains(" join ")
                    || after_from.contains(" join\n")
                    || after_from.contains("\njoin ")
                    || after_from.starts_with("join ")
                    || after_from.contains(" join(");
                if !has_join {
                    if let Some(comma_rel) = after_from.find(',') {
                        let between = after_from[..comma_rel].trim();
                        let after_comma = after_from[comma_rel + 1..].trim_start();
                        let looks_like_table = !between.is_empty()
                            && after_comma
                                .chars()
                                .next()
                                .is_some_and(|c| c.is_alphabetic() || c == '"');
                        if looks_like_table {
                            let abs = from_at + 4 + comma_rel;
                            let msg =
                                "Comma join may produce a cartesian product; prefer explicit JOIN … ON".to_owned();
                            string_diagnostics.push(msg.clone());
                            structured_diagnostics.push(Diagnostic::lint((abs, abs + 1), msg, "lint.comma-join"));
                        }
                    }
                }
            }
        }
        // Duplicate projection aliases: `SELECT a AS x, b AS x`.
        if lint.allows("lint.duplicate-alias") {
            if let Some(select_at) = lower.find("select") {
                let after_select = &lower[select_at + "select".len()..];
                let projection = after_select.split(" from ").next().unwrap_or(after_select);
                let mut seen: Vec<(String, usize)> = Vec::new();
                let mut search_from = 0usize;
                while let Some(rel) = projection[search_from..].find(" as ") {
                    let abs_in_proj = search_from + rel + " as ".len();
                    let alias_slice = projection[abs_in_proj..].trim_start();
                    let skip = projection[abs_in_proj..].len() - alias_slice.len();
                    let alias: String = alias_slice
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '"')
                        .collect();
                    if !alias.is_empty() {
                        let alias_key = alias.trim_matches('"').to_ascii_lowercase();
                        let abs = select_at + "select".len() + abs_in_proj + skip;
                        if let Some((_, first_at)) = seen.iter().find(|(name, _)| name == &alias_key) {
                            let msg = format!("Duplicate projection alias `{alias_key}`");
                            string_diagnostics.push(msg.clone());
                            structured_diagnostics.push(Diagnostic::lint(
                                (abs, abs + alias.len()),
                                msg,
                                "lint.duplicate-alias",
                            ));
                            let _ = first_at;
                        } else {
                            seen.push((alias_key, abs));
                        }
                    }
                    search_from = abs_in_proj + alias.len().max(1);
                }
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn parse_sql_diagnostics(sql: &str, driver: &str) -> Vec<String> {
        Self::analyze_sql_diagnostics_with_lint(sql, driver, &SqlLintSettings::default()).0
    }

    pub(crate) fn refresh_diagnostics(&mut self) {
        let driver = self.active_driver().to_owned();
        let doc_index = self.active_query_document;
        let version = self
            .query_documents
            .get(doc_index)
            .map(|doc| doc.buffer.version())
            .unwrap_or(0);
        let cache_key = (doc_index, version);
        let exec_fp = self
            .query_documents
            .get(doc_index)
            .and_then(|doc| doc.execution_diagnostic.as_ref())
            .map(|d| d.range);

        if self.diagnostics_cache_key == Some(cache_key) && self.diagnostics_cache_driver == driver {
            // Cheap path: only rematch when execution diagnostic identity changes.
            if self.diagnostics_exec_fp == exec_fp {
                return;
            }
            if let Some(doc) = self.query_documents.get_mut(doc_index) {
                doc.diagnostics = deduplicate_diagnostics(
                    self.diagnostics_lint_structured
                        .iter()
                        .cloned()
                        .chain(doc.execution_diagnostic.clone())
                        .collect(),
                );
            }
            self.diagnostics_exec_fp = exec_fp;
            return;
        }

        // While typing, defer sqlparser until a short quiet window so keystrokes stay snappy.
        let now = Instant::now();
        if self.diagnostics_debounce_key != Some(cache_key) {
            self.diagnostics_debounce_key = Some(cache_key);
            self.diagnostics_debounce_at = Some(now + Duration::from_millis(180));
            return;
        }
        if let Some(deadline) = self.diagnostics_debounce_at {
            if now < deadline {
                return;
            }
        }

        let lint = self.settings.editor.lint.clone();
        if let Some(doc) = self.query_documents.get_mut(doc_index) {
            let (raw_diags, structured) = Self::analyze_sql_diagnostics_with_lint(doc.text(), &driver, &lint);
            self.diagnostics = raw_diags;
            self.diagnostics_lint_structured = structured.clone();
            doc.diagnostics =
                deduplicate_diagnostics(structured.into_iter().chain(doc.execution_diagnostic.clone()).collect());
        } else {
            self.diagnostics = Self::analyze_sql_diagnostics_with_lint(self.active_query_text(), &driver, &lint).0;
            self.diagnostics_lint_structured.clear();
        }
        self.diagnostics_cache_key = Some(cache_key);
        self.diagnostics_cache_driver = driver;
        self.diagnostics_exec_fp = exec_fp;
        self.diagnostics_debounce_at = None;
    }
}
