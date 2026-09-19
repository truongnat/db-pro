//! Table open, workspace folder/file, and schema-drift actions.
use super::*;

impl DbProApp {
    pub(crate) fn open_table(&mut self, table: String) {
        if self.schema_explorer.selected_table.as_deref() == Some(&table) {
            self.workspace.active_tab = WorkspaceTab::Table;
            self.schema_explorer.record_recent_table(&table);
            return;
        }
        if !self.table_mutation.staged_changes.is_empty() {
            self.workspace.pending_navigation_action = Some(PendingNavigationAction::OpenTable(table));
            self.table_data.discard_changes_confirmation = true;
            self.feedback.runtime_message = "Apply or discard staged changes before opening another table".to_owned();
            return;
        }
        self.workspace.pending_navigation_action = None;
        self.persist_current_grid_layout();
        self.schema_explorer.record_recent_table(&table);
        self.schema_explorer.selected_table = Some(table);
        self.restore_grid_layout_for_active_table();
        self.request_table_info();
        self.request_table_data();
        self.workspace.active_tab = WorkspaceTab::Table;
    }

    pub(crate) fn request_open_workspace_folder(&mut self) {
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::PickWorkspaceFolder { request_id });
        self.feedback.runtime_message = "Choose a workspace folder…".to_owned();
    }

    pub(crate) fn open_workspace_folder(&mut self, path: std::path::PathBuf) {
        let result = if self.workspace.files.ide_workspace.roots.is_empty() {
            self.workspace.files.ide_workspace.open_root(path)
        } else {
            self.workspace.files.ide_workspace.add_root(path)
        };
        match result {
            Ok(()) => {
                self.workspace.activity = Activity::Files;
                self.workspace.sidebar_open = true;
                self.workspace.files.workspace_search_hits.clear();
                self.workspace.files.ide_workspace.scan_diagnostics();
                self.feedback.runtime_message = format!(
                    "Opened workspace {} · {} files · {} roots",
                    self.workspace.files.ide_workspace.root_label(),
                    self.workspace.files.ide_workspace.index().len(),
                    self.workspace.files.ide_workspace.roots.len()
                );
            }
            Err(error) => {
                self.feedback.runtime_message = format!("Failed to open workspace: {error}");
            }
        }
    }

    pub(crate) fn open_workspace_sql_file(&mut self, relative_path: String) {
        let Some(absolute) = self.workspace.files.ide_workspace.absolute_for_relative(&relative_path) else {
            self.feedback.runtime_message = "Open a workspace folder first".to_owned();
            return;
        };
        let absolute_str = absolute.to_string_lossy().into_owned();
        if let Some(index) = self
            .query_session_state
            .documents
            .iter()
            .position(|doc| doc.file_path.as_deref() == Some(absolute_str.as_str()))
        {
            self.switch_query_document(index);
            self.workspace.active_tab = WorkspaceTab::Query;
            return;
        }
        let content = match std::fs::read_to_string(&absolute) {
            Ok(text) => text,
            Err(error) => {
                self.feedback.runtime_message = format!("Failed to read {}: {error}", absolute.display());
                return;
            }
        };
        let title = absolute
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| relative_path.clone());
        let id = format!("file-{absolute_str}");
        let mut doc = QueryDocument::new(id, title, content);
        doc.file_path = Some(absolute_str.clone());
        doc.connection_id = self.connection.lifecycle.active_connection_id().map(str::to_owned);
        doc.schema = Some(self.active_schema().to_owned());
        doc.mark_saved();
        if let Some(mtime) = git_workspace::disk_mtime_secs(&absolute) {
            self.workspace.files.workspace_file_mtimes.insert(absolute_str, mtime);
        }
        self.query_session_state.documents.push(doc);
        self.query_session_state.active_document_index = self.query_session_state.documents.len() - 1;
        self.workspace.activity = Activity::Queries;
        self.workspace.active_tab = WorkspaceTab::Query;
        self.reset_query_cursor();
        self.feedback.runtime_message = format!("Opened {relative_path}");
    }

    pub(crate) fn save_active_workspace_file(&mut self) -> bool {
        let Some(doc) = self
            .query_session_state
            .documents
            .get_mut(self.query_session_state.active_document_index)
        else {
            return false;
        };
        let Some(path) = doc.file_path.clone() else {
            return false;
        };
        let contents = doc.text().as_bytes().to_vec();
        match query_view::write_file_atomically(std::path::Path::new(&path), &contents) {
            Ok(()) => {
                doc.mark_saved();
                if let Some(mtime) = git_workspace::disk_mtime_secs(std::path::Path::new(&path)) {
                    self.workspace.files.workspace_file_mtimes.insert(path.clone(), mtime);
                }
                self.workspace.files.workspace_external_change = None;
                self.feedback.runtime_message = format!("Saved {}", std::path::Path::new(&path).display());
                true
            }
            Err(error) => {
                self.feedback.runtime_message = format!("Save failed: {error}");
                false
            }
        }
    }

    pub(crate) fn toggle_split_editor(&mut self) {
        if self.workspace.split_editor_secondary.is_some() {
            self.workspace.split_editor_secondary = None;
            self.feedback.runtime_message = "Split editor closed".to_owned();
            return;
        }
        if self.query_session_state.documents.len() < 2 {
            self.feedback.runtime_message = "Open a second document before splitting".to_owned();
            return;
        }
        let secondary = if self.query_session_state.active_document_index + 1 < self.query_session_state.documents.len()
        {
            self.query_session_state.active_document_index + 1
        } else {
            0
        };
        self.workspace.split_editor_secondary = Some(secondary);
        self.feedback.runtime_message = "Split editor enabled".to_owned();
    }

    /// Capture/evidence helper: keep the initial connection request pending so
    /// the Welcome surface can be documented in its loading state.
    pub fn prepare_loading_for_capture(&mut self) {
        self.connection.lifecycle.mark_connections_requested();
        self.connection.lifecycle.set_connections_request_pending(true);
        self.feedback.runtime_message = "Loading connections…".to_owned();
    }

    /// Capture/evidence helper: open the connection editor with a deterministic
    /// validation error, without requiring a live database.
    pub fn open_connection_error_for_capture(&mut self) {
        self.connection.open_new();
        self.connection
            .dialog
            .set_error("Connection test failed: authentication rejected by the server.");
        self.feedback.runtime_message = "Connection test failed".to_owned();
    }

    /// Capture/evidence helper: open the new-connection dialog in its normal state.
    pub fn open_new_connection_for_capture(&mut self) {
        self.connection.open_new();
    }

    /// Capture/evidence helper: open the Edit Connection dialog with a test draft so
    /// the password input + eye toggle can be documented (the affected surface for the
    /// input click-steal fix) without needing a real saved connection.
    pub fn open_edit_connection_for_capture(&mut self) {
        self.connection
            .dialog
            .set_editing_connection_id(Some("capture-test".to_owned()));
        self.connection.dialog.set_draft(UiConnectionDraft {
            name: "Test Connection".to_owned(),
            host: "localhost".to_owned(),
            port: "5432".to_owned(),
            database: "testdb".to_owned(),
            username: "testuser".to_owned(),
            password: "testpassword123".to_owned(),
            driver: crate::UiDriver::Postgres,
            ssl_mode: crate::UiSslMode::Disable,
            readonly: false,
            group: String::new(),
            tags: String::new(),
            favorite: false,
            environment: "Development".to_owned(),
            ssh_tunnel_enabled: false,
            ssh_host: String::new(),
            ssh_port: "22".to_owned(),
            ssh_user: String::new(),
            ssh_private_key: String::new(),
            ssh_profile_id: String::new(),
            ssl_root_cert_path: String::new(),
            ssl_client_cert_path: String::new(),
            ssl_client_key_path: String::new(),
            cloud_preset: String::new(),
            auth_kind: "password".to_owned(),
            cloud_snippet: String::new(),
            cloud_guidance: String::new(),
        });
        self.connection.dialog.clear_error();
        self.connection.dialog.clear_test();
        self.connection.lifecycle.clear_pending_request();
        self.connection.dialog.set_focus_name_on_open(true);
        self.connection.dialog.set_open(true);
    }

    /// Capture/evidence helper: open a fresh untitled Query buffer (UI05 editor-first shots).
    pub fn open_query_workspace_for_capture(&mut self) {
        self.preferences.dark_mode = true;
        self.theme = DbProTheme::dark();
        self.new_query_document();
        if let Some(doc) = self
            .query_session_state
            .documents
            .get_mut(self.query_session_state.active_document_index)
        {
            doc.set_text("SELECT u.id, u.email\nFROM users u\nWHERE u.active = true;\n");
            doc.dirty = false;
        }
        self.workspace.bottom_panel_open = false;
        self.query_editor.query_output_dock_maximized = false;
        self.query_editor.query_params_panel_open = false;
        self.query_editor.visual_query_builder_open = false;
        self.query_editor.editor_search_open = false;
        self.query_editor.snippets_open = false;
        self.query_execution.query_txn_bar_open = false;
    }

    /// Capture helper: same as query workspace but force light theme.
    pub fn open_query_workspace_for_capture_light(&mut self) {
        self.open_query_workspace_for_capture();
        self.preferences.dark_mode = false;
        self.theme = DbProTheme::light();
    }

    pub(crate) fn request_close_workspace_tab(&mut self, tab: WorkspaceTab) {
        match tab {
            WorkspaceTab::Table => {
                if !self.table_mutation.staged_changes.is_empty() {
                    self.workspace.pending_navigation_action = Some(PendingNavigationAction::CloseWorkspace(tab));
                    self.table_data.discard_changes_confirmation = true;
                    self.feedback.runtime_message =
                        "Apply or discard staged changes before closing the table".to_owned();
                    return;
                }
                self.workspace.pending_navigation_action = None;
                self.schema_explorer.selected_table = None;
                self.table_state.table_info = None;
                self.table_state.table_ddl = None;
                self.table_state.table_info_error = None;
                self.table_state.table_ddl_error = None;
                self.table_state.table_data_result = None;
                self.table_state.table_data_total_rows = None;
                self.table_state.table_data_request = None;
                self.table_state.table_info_request = None;
                self.table_state.table_ddl_request = None;
                self.table_mutation.table_mutation_request = None;
                self.table_mutation.staged_changes.clear();
                self.table_mutation.staged_apply_request = None;
                self.table_mutation.staged_apply_targets.clear();
                self.table_mutation.table_mutation_retry_after_reload = false;
                self.table_mutation.table_mutation_retry_target = None;
                self.table_mutation.table_mutation_error = None;
                self.table_data.selected_cell = None;
                self.table_data.selected_row = None;
                self.table_data.selected_rows.clear();
                self.table_data.selection_anchor_row = None;
                self.table_data.selection_anchor_cell = None;
                self.table_data.data_editing_cell = None;
                self.table_data.data_edit_error = None;
                self.table_data.data_delete_confirmation = false;
                self.table_data.discard_changes_confirmation = false;
            }
            WorkspaceTab::SchemaObject => {
                self.schema_explorer.selected_schema_object = None;
                self.schema_explorer.schema_object_view = SchemaObjectView::Definition;
                self.table_state.table_data_result = None;
                self.table_state.table_data_total_rows = None;
                self.table_state.table_data_request = None;
            }
            WorkspaceTab::Diagram => {
                self.diagram.search.clear();
                self.diagram.show_all = false;
                self.diagram.pan = egui::Vec2::ZERO;
                self.diagram.pan_origin = None;
            }
            WorkspaceTab::SchemaWorkbench => {
                self.schema_workbench.apply_confirmation = false;
            }
            WorkspaceTab::SchemaCompare => {
                self.schema_compare.schema_diff = None;
            }
            WorkspaceTab::ComponentGallery => {}
            WorkspaceTab::Welcome | WorkspaceTab::Query => return,
        }
        if self.workspace.active_tab == tab {
            self.activate_welcome_tab();
        }
        self.feedback.runtime_message = "Workspace closed".to_owned();
    }

    pub(crate) fn reload_workspace_file_from_disk(&mut self, path: &str) {
        let Ok(content) = std::fs::read_to_string(path) else {
            self.feedback.runtime_message = format!("Could not reload {path}");
            return;
        };
        if let Some(doc) = self
            .query_session_state
            .documents
            .iter_mut()
            .find(|doc| doc.file_path.as_deref() == Some(path))
        {
            doc.buffer.set_text(content);
            doc.mark_saved();
            if let Some(mtime) = git_workspace::disk_mtime_secs(std::path::Path::new(path)) {
                self.workspace
                    .files
                    .workspace_file_mtimes
                    .insert(path.to_owned(), mtime);
            }
            self.workspace.files.workspace_external_change = None;
            self.feedback.runtime_message = format!("Reloaded {path}");
        }
    }
}
