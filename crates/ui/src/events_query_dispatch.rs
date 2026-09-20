//! Keyboard shortcuts and query dispatch / destructive-run gate.
use super::events::PendingDestructiveRun;
use super::*;

impl DbProApp {
    pub(super) fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if self.palette.mode.is_some()
            || self.connection.dialog.is_open()
            || self.overlay.delete_confirmation_id.is_some()
            || self.overlay.folder_delete_confirmation.is_some()
            || self.table_data.insert_row_open
        {
            return;
        }
        if ctx.input(|input| self.shortcut_pressed(input, "query.save_as")) {
            self.open_save_as_dialog();
            return;
        }
        if ctx.input(|input| self.shortcut_pressed(input, "query.save")) {
            self.save_query_document_at(self.query_session_state.active_document_index);
            return;
        }
        let text_input_has_focus = ctx.wants_keyboard_input();
        if !text_input_has_focus && ctx.input(|i| self.shortcut_pressed(i, "palette.commands")) {
            self.palette.open(PaletteMode::Commands);
            return;
        }
        if !text_input_has_focus
            && (ctx.input(|i| self.shortcut_pressed(i, "palette.quick_open_alt"))
                || ctx.input(|i| self.shortcut_pressed(i, "palette.quick_open")))
        {
            self.palette.open(PaletteMode::QuickOpen);
            return;
        }
        if !text_input_has_focus && ctx.input(|i| self.shortcut_pressed(i, "view.toggle_sidebar")) {
            self.workspace.sidebar_open = !self.workspace.sidebar_open;
        }
        if !text_input_has_focus && ctx.input(|i| self.shortcut_pressed(i, "connection.new")) {
            self.connection.open_new();
            return;
        }
        if !text_input_has_focus && ctx.input(|i| self.shortcut_pressed(i, "query.new")) {
            self.new_query_document();
            self.workspace.active_tab = WorkspaceTab::Query;
            return;
        }
        if !text_input_has_focus && ctx.input(|i| self.shortcut_pressed(i, "editor.find")) {
            self.query_editor.editor_search_open = true;
        }
        if ctx.input(|i| {
            self.shortcut_pressed(i, "query.run")
                || (self.query_editor.query_editor_focused
                    && !self.workspace.agent_open
                    && i.key_pressed(egui::Key::Enter)
                    && Self::primary_modifier_pressed(i))
        }) {
            self.dispatch_query();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if let Some(request_id) = self.active_query_running_request() {
                if self.query_capabilities().allows(|c| c.query.cancel) {
                    self.cancel_query(request_id);
                } else {
                    self.feedback.runtime_message = "Query cancellation is not supported for this provider".to_owned();
                }
            } else if self.query_editor.query_tools_open {
                self.query_editor.query_tools_open = false;
            } else if self.query_editor.editor_search_open {
                self.query_editor.editor_search_open = false;
            } else {
                self.set_agent_open(false, ctx);
            }
        }
    }

    fn shortcut_pressed(&self, input: &egui::InputState, command_id: &str) -> bool {
        let token = self.preferences.settings.keybindings.resolved(command_id);
        settings_model::match_shortcut_token(input, &token)
    }

    pub(super) fn cancel_query(&mut self, request_id: crate::RequestId) {
        self.dispatch_command(UiCommand::CancelQuery { request_id });
        self.feedback.runtime_message = "Cancelling query…".to_owned();
    }

    pub(super) fn dispatch_query(&mut self) {
        if self.active_query_running_request().is_some() {
            return;
        }
        let Some(connection_id) = self
            .active_query_connection_id()
            .map(String::from)
            .or_else(|| self.active_connection().map(|connection| connection.id.clone()))
        else {
            self.feedback.runtime_message = "Create or select a connection first".to_owned();
            return;
        };
        let (sql, execution_range) = self
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .map(|doc| doc.resolve_executable_range())
            .unwrap_or_else(|| {
                (
                    self.active_query_text().trim().to_owned(),
                    (0, self.active_query_text().len()),
                )
            });
        if sql.trim().is_empty() {
            self.feedback.runtime_message = "Query is empty".to_owned();
            return;
        }
        let version = self.query_session_state.active_buffer_version();
        if self.hold_destructive_run(&sql, execution_range, version, false) {
            return;
        }
        self.send_query_run(connection_id, sql, execution_range, version, false);
    }

    pub(super) fn dispatch_query_all(&mut self) {
        if self.active_query_running_request().is_some() {
            return;
        }
        let Some(connection_id) = self
            .active_query_connection_id()
            .map(String::from)
            .or_else(|| self.active_connection().map(|connection| connection.id.clone()))
        else {
            self.feedback.runtime_message = "Create or select a connection first".to_owned();
            return;
        };
        let (sql, execution_range) = self
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .map(|doc| {
                let text = doc.text().trim().to_owned();
                let leading = doc.text().len().saturating_sub(doc.text().trim_start().len());
                (text.clone(), (leading, leading + text.len()))
            })
            .unwrap_or_else(|| {
                (
                    self.active_query_text().trim().to_owned(),
                    (0, self.active_query_text().len()),
                )
            });
        if sql.is_empty() {
            self.feedback.runtime_message = "Query is empty".to_owned();
            return;
        }
        let version = self.query_session_state.active_buffer_version();
        if self.hold_destructive_run(&sql, execution_range, version, true) {
            return;
        }
        self.send_query_run(connection_id, sql, execution_range, version, true);
    }

    /// Hold a statement or script the classifier rates `Destructive` until the user
    /// confirms the exact text, and report whether the run was held.
    ///
    /// The query editor is the documented path for arbitrary SQL (including DDL), so
    /// nothing is refused here: the text is kept in `pending_destructive_run` and sent by
    /// `confirm_pending_destructive_run`. Reads, writes and plain DDL dispatch exactly as
    /// before, and the backend policy still runs on whatever is dispatched.
    pub(super) fn hold_destructive_run(
        &mut self,
        sql: &str,
        execution_range: (usize, usize),
        version: u64,
        all_statements: bool,
    ) -> bool {
        if db_pro_core::domain::safety::classify_script_safety(sql)
            != Some(db_pro_core::domain::safety::StatementSafety::Destructive)
        {
            return false;
        }
        self.query_execution.pending_destructive_run = Some(PendingDestructiveRun {
            sql: sql.to_owned(),
            execution_range,
            version,
            all_statements,
        });
        self.feedback.runtime_message =
            "Destructive statement held for confirmation — nothing was sent to the database".to_owned();
        true
    }

    /// Send the statement the user confirmed. The text and the buffer version are the ones
    /// the prompt displayed, so a confirmation can never execute something the user did
    /// not see.
    pub(super) fn confirm_pending_destructive_run(&mut self) {
        let Some(pending) = self.query_execution.pending_destructive_run.take() else {
            return;
        };
        let Some(connection_id) = self
            .active_query_connection_id()
            .map(String::from)
            .or_else(|| self.active_connection().map(|connection| connection.id.clone()))
        else {
            self.feedback.runtime_message = "Create or select a connection first".to_owned();
            return;
        };
        self.send_query_run(
            connection_id,
            pending.sql,
            pending.execution_range,
            pending.version,
            pending.all_statements,
        );
    }

    /// Drop a held destructive statement without executing it.
    pub(super) fn cancel_pending_destructive_run(&mut self) {
        if self.query_execution.pending_destructive_run.take().is_some() {
            self.feedback.runtime_message =
                "Destructive statement cancelled — nothing was sent to the database".to_owned();
        }
    }

    pub(crate) fn send_query_run(
        &mut self,
        connection_id: String,
        sql: String,
        execution_range: (usize, usize),
        version: u64,
        all_statements: bool,
    ) {
        let discovered = crate::query::discover_sql_parameters(&sql);
        if all_statements && !discovered.is_empty() {
            self.feedback.runtime_message =
                "Parameterized scripts are not supported yet — run a single statement with bindings".to_owned();
            return;
        }

        let style = if self.active_driver().eq_ignore_ascii_case("postgresql")
            || self.active_driver().eq_ignore_ascii_case("postgres")
        {
            crate::query::PlaceholderStyle::NumberedDollar
        } else {
            crate::query::PlaceholderStyle::QuestionMark
        };
        let values = self
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .map(|doc| doc.parameter_values.clone())
            .unwrap_or_default();
        let (sql, params) = if discovered.is_empty() {
            (sql, Vec::new())
        } else {
            match crate::query::prepare_bound_sql(&sql, &values, style) {
                Ok(prepared) => (prepared.sql, prepared.values),
                Err(missing) => {
                    self.feedback.runtime_message = format!("Fill parameter {missing} before running");
                    return;
                }
            }
        };

        if !self.query_editor.query_history.iter().any(|query| query == &sql) {
            self.query_editor.query_history.push(sql.clone());
            if self.query_editor.query_history.len() > 20 {
                self.query_editor.query_history.remove(0);
            }
        }
        let request_id = self.task_bridge.next_request_id();
        if let Some(doc) = self
            .query_session_state
            .documents
            .get_mut(self.query_session_state.active_document_index)
        {
            doc.execution_state = QueryExecutionState::Running(request_id);
            doc.execution_started_at = Some(Instant::now());
            doc.execution_started_wall_time = Some(chrono::Utc::now().to_rfc3339());
            doc.executing_range = Some(execution_range);
            doc.executing_sql = Some(sql.clone());
            doc.executing_version = Some(version);
            doc.last_executed_range = Some(execution_range);
            doc.execution_diagnostic = None;
            self.query_session_state
                .document_requests
                .insert(request_id, doc.id.clone());
        }
        self.feedback.runtime_message = if all_statements {
            "Sending full script to runtime…".to_owned()
        } else {
            "Sending query to runtime…".to_owned()
        };
        self.dispatch_command(if all_statements {
            UiCommand::RunQueryMulti {
                request_id,
                connection_id,
                sql,
            }
        } else {
            UiCommand::RunQuery {
                request_id,
                connection_id,
                sql,
                params,
            }
        });
    }
}
