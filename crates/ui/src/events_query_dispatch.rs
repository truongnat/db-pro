//! Keyboard shortcuts and query dispatch / destructive-run gate.
use super::*;

impl DbProApp {
    pub(super) fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if self.shortcuts_blocked() {
            return;
        }
        if self.handle_document_shortcuts(ctx) {
            return;
        }
        let text_input_has_focus = ctx.wants_keyboard_input();
        if self.handle_unfocused_shortcuts(ctx, text_input_has_focus) {
            return;
        }
        if ctx.input(|i| {
            self.shortcut_pressed(i, "query.run")
                || (self.query.editor.query_editor_focused
                    && !self.workspace.agent_open
                    && i.key_pressed(egui::Key::Enter)
                    && Self::primary_modifier_pressed(i))
        }) {
            self.dispatch_query();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.handle_escape(ctx);
        }
    }

    fn shortcuts_blocked(&self) -> bool {
        self.palette.mode.is_some()
            || self.connection.dialog.is_open()
            || self.overlay.delete_confirmation_id.is_some()
            || self.overlay.folder_delete_confirmation.is_some()
            || self.table.editing.insert_row_open
    }

    fn handle_document_shortcuts(&mut self, ctx: &egui::Context) -> bool {
        if ctx.input(|input| self.shortcut_pressed(input, "query.save_as")) {
            self.open_save_as_dialog();
            return true;
        }
        if ctx.input(|input| self.shortcut_pressed(input, "query.save")) {
            self.save_query_document_at(self.query.session.active_document_index);
            return true;
        }
        false
    }

    fn handle_unfocused_shortcuts(&mut self, ctx: &egui::Context, text_input_has_focus: bool) -> bool {
        if text_input_has_focus {
            return false;
        }
        if ctx.input(|i| self.shortcut_pressed(i, "palette.commands")) {
            self.palette.open(PaletteMode::Commands);
            return true;
        }
        if ctx.input(|i| {
            self.shortcut_pressed(i, "palette.quick_open_alt")
                || self.shortcut_pressed(i, "palette.quick_open")
        }) {
            self.palette.open(PaletteMode::QuickOpen);
            return true;
        }
        if ctx.input(|i| self.shortcut_pressed(i, "view.toggle_sidebar")) {
            self.workspace.sidebar_open = !self.workspace.sidebar_open;
        }
        if ctx.input(|i| self.shortcut_pressed(i, "connection.new")) {
            self.connection.open_new();
            return true;
        }
        if ctx.input(|i| self.shortcut_pressed(i, "query.new")) {
            self.new_query_document();
            self.workspace.active_tab = WorkspaceTab::Query;
            return true;
        }
        if ctx.input(|i| self.shortcut_pressed(i, "editor.find")) {
            self.query.editor.editor_search_open = true;
        }
        false
    }

    fn handle_escape(&mut self, ctx: &egui::Context) {
        if let Some(request_id) = self.query.session.active_running_request() {
            if self.query_capabilities().allows(|c| c.query.cancel) {
                self.cancel_query(request_id);
            } else {
                self.feedback.runtime_message = "Query cancellation is not supported for this provider".to_owned();
            }
        } else if self.query.editor.query_tools_open {
            self.query.editor.query_tools_open = false;
        } else if self.query.editor.editor_search_open {
            self.query.editor.editor_search_open = false;
        } else {
            self.set_agent_open(false, ctx);
        }
    }

    fn shortcut_pressed(&self, input: &egui::InputState, command_id: &str) -> bool {
        let token = self.preferences.settings.keybindings.resolved(command_id);
        settings_model::match_shortcut_token(input, &token)
    }

    pub(super) fn cancel_query(&mut self, request_id: crate::RequestId) {
        if self.dispatch_command(UiCommand::CancelQuery { request_id }) {
            self.feedback.runtime_message = "Cancelling query…".to_owned();
        }
    }

    pub(super) fn dispatch_query(&mut self) -> bool {
        if self.query.session.active_running_request().is_some() {
            return false;
        }
        let Some(connection_id) = self
            .active_query_connection_id()
            .map(String::from)
            .or_else(|| self.active_connection().map(|connection| connection.id.clone()))
        else {
            self.feedback.runtime_message = "Create or select a connection first".to_owned();
            return false;
        };
        let (sql, execution_range) = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .map(|doc| doc.resolve_executable_range())
            .unwrap_or_else(|| {
                (
                    self.query.session.active_text().trim().to_owned(),
                    (0, self.query.session.active_text().len()),
                )
            });
        if sql.trim().is_empty() {
            self.feedback.runtime_message = "Query is empty".to_owned();
            return false;
        }
        let version = self.query.session.active_buffer_version();
        if self.hold_destructive_run(&sql, execution_range, version, false) {
            return false;
        }
        self.send_query_run(connection_id, sql, execution_range, version, false)
    }

    pub(super) fn dispatch_query_all(&mut self) {
        if self.query.session.active_running_request().is_some() {
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
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .map(|doc| {
                let text = doc.text().trim().to_owned();
                let leading = doc.text().len().saturating_sub(doc.text().trim_start().len());
                (text.clone(), (leading, leading + text.len()))
            })
            .unwrap_or_else(|| {
                (
                    self.query.session.active_text().trim().to_owned(),
                    (0, self.query.session.active_text().len()),
                )
            });
        if sql.is_empty() {
            self.feedback.runtime_message = "Query is empty".to_owned();
            return;
        }
        let version = self.query.session.active_buffer_version();
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
        self.query_execution_context()
            .hold_destructive_run(sql, execution_range, version, all_statements)
    }

    /// Send the statement the user confirmed. The text and the buffer version are the ones
    /// the prompt displayed, so a confirmation can never execute something the user did
    /// not see.
    pub(super) fn confirm_pending_destructive_run(&mut self) {
        let Some(pending) = self.query_execution_context().take_pending_destructive_run() else {
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
            pending.sql().to_owned(),
            pending.execution_range(),
            pending.version(),
            pending.all_statements(),
        );
    }

    /// Drop a held destructive statement without executing it.
    pub(super) fn cancel_pending_destructive_run(&mut self) {
        self.query_execution_context().cancel_pending_destructive_run();
    }

    pub(crate) fn send_query_run(
        &mut self,
        connection_id: String,
        sql: String,
        execution_range: (usize, usize),
        version: u64,
        all_statements: bool,
    ) -> bool {
        let request_id = self.next_request_id();
        let Some(command) =
            self.query_execution_context()
                .prepare_query_run(request_id, connection_id, sql, all_statements)
        else {
            return false;
        };
        let runtime_command = query_run_command(&command);
        if self.dispatch_command(runtime_command) {
            self.query_execution_context()
                .commit_dispatched(&command, execution_range, version);
            true
        } else {
            false
        }
    }

    fn query_execution_context(&mut self) -> query_execution_actions::QueryExecutionContext<'_> {
        let driver = self.active_driver().to_owned();
        query_execution_actions::QueryExecutionContext::new(
            &mut self.query.session,
            &mut self.query.editor,
            &mut self.query.execution,
            &mut self.feedback,
            driver,
        )
    }
}

fn query_run_command(prepared: &query_execution_actions::PreparedQueryRun) -> UiCommand {
    if prepared.all_statements {
        UiCommand::RunQueryMulti {
            request_id: prepared.request_id,
            connection_id: prepared.connection_id.clone(),
            sql: prepared.sql.clone(),
        }
    } else {
        UiCommand::RunQuery {
            request_id: prepared.request_id,
            connection_id: prepared.connection_id.clone(),
            sql: prepared.sql.clone(),
            params: prepared.params.clone(),
        }
    }
}
