//! Table open, workspace folder/file, and schema-drift actions.
use super::*;

impl DbProApp {
    pub(crate) fn open_table(&mut self, table: String) {
        if self.schema_explorer.selected_table.as_deref() == Some(&table) {
            self.workspace.active_tab = WorkspaceTab::Table;
            self.record_recent_table(&table);
            return;
        }
        if !self.table_mutation.staged_changes.is_empty() {
            self.workspace.pending_navigation_action = Some(PendingNavigationAction::OpenTable(table));
            self.table_data.discard_changes_confirmation = true;
            self.runtime_message = "Apply or discard staged changes before opening another table".to_owned();
            return;
        }
        self.workspace.pending_navigation_action = None;
        self.persist_current_grid_layout();
        self.record_recent_table(&table);
        self.schema_explorer.selected_table = Some(table);
        self.restore_grid_layout_for_active_table();
        self.request_table_info();
        self.request_table_data();
        self.workspace.active_tab = WorkspaceTab::Table;
    }

    /// Push `table` to the front of the MRU recent list (#212).
    pub(crate) fn record_recent_table(&mut self, table: &str) {
        if table.is_empty() {
            return;
        }
        self.schema_explorer.recent_tables.retain(|item| item != table);
        self.schema_explorer.recent_tables.insert(0, table.to_owned());
        if self.schema_explorer.recent_tables.len() > RECENT_TABLES_MAX {
            self.schema_explorer.recent_tables.truncate(RECENT_TABLES_MAX);
        }
    }

    pub(crate) fn remove_recent_table(&mut self, table: &str) {
        self.schema_explorer.recent_tables.retain(|item| item != table);
    }

    pub(crate) fn request_open_workspace_folder(&mut self) {
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::PickWorkspaceFolder { request_id });
        self.runtime_message = "Choose a workspace folder…".to_owned();
    }

    pub(crate) fn open_workspace_folder(&mut self, path: std::path::PathBuf) {
        let result = if self.workspace_files.ide_workspace.roots.is_empty() {
            self.workspace_files.ide_workspace.open_root(path)
        } else {
            self.workspace_files.ide_workspace.add_root(path)
        };
        match result {
            Ok(()) => {
                self.workspace.activity = Activity::Files;
                self.workspace.sidebar_open = true;
                self.workspace_files.workspace_search_hits.clear();
                self.workspace_files.ide_workspace.scan_diagnostics();
                self.runtime_message = format!(
                    "Opened workspace {} · {} files · {} roots",
                    self.workspace_files.ide_workspace.root_label(),
                    self.workspace_files.ide_workspace.index().len(),
                    self.workspace_files.ide_workspace.roots.len()
                );
            }
            Err(error) => {
                self.runtime_message = format!("Failed to open workspace: {error}");
            }
        }
    }

    pub(crate) fn close_workspace_folder(&mut self) {
        self.workspace_files.ide_workspace.close();
        self.workspace_files.workspace_search_hits.clear();
        self.workspace_files.workspace_replace_previews.clear();
        self.workspace_files.workspace_context_items.clear();
        self.workspace.split_editor_secondary = None;
        self.runtime_message = "Workspace closed".to_owned();
    }

    pub(crate) fn refresh_workspace_folder(&mut self) {
        match self.workspace_files.ide_workspace.refresh() {
            Ok(()) => {
                self.workspace_files.ide_workspace.scan_diagnostics();
                self.runtime_message = format!(
                    "Workspace refreshed · {} files indexed",
                    self.workspace_files.ide_workspace.index().len()
                );
            }
            Err(error) => {
                self.runtime_message = format!("Workspace refresh failed: {error}");
            }
        }
    }

    pub(crate) fn open_workspace_sql_file(&mut self, relative_path: String) {
        let Some(absolute) = self.workspace_files.ide_workspace.absolute_for_relative(&relative_path) else {
            self.runtime_message = "Open a workspace folder first".to_owned();
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
                self.runtime_message = format!("Failed to read {}: {error}", absolute.display());
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
        doc.connection_id = self.connection_lifecycle.active_connection_id.clone();
        doc.schema = Some(self.active_schema().to_owned());
        doc.mark_saved();
        if let Some(mtime) = git_workspace::disk_mtime_secs(&absolute) {
            self.workspace_files.workspace_file_mtimes.insert(absolute_str, mtime);
        }
        self.query_session_state.documents.push(doc);
        self.query_session_state.active_document_index = self.query_session_state.documents.len() - 1;
        self.workspace.activity = Activity::Queries;
        self.workspace.active_tab = WorkspaceTab::Query;
        self.reset_query_cursor();
        self.runtime_message = format!("Opened {relative_path}");
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
                    self.workspace_files.workspace_file_mtimes.insert(path.clone(), mtime);
                }
                self.workspace_files.workspace_external_change = None;
                self.runtime_message = format!("Saved {}", std::path::Path::new(&path).display());
                true
            }
            Err(error) => {
                self.runtime_message = format!("Save failed: {error}");
                false
            }
        }
    }

    pub(crate) fn run_workspace_search(&mut self) {
        if self.workspace_files.ide_workspace.roots.is_empty() {
            self.workspace_files.workspace_search_hits.clear();
            self.runtime_message = "Open a workspace folder before searching".to_owned();
            return;
        }
        self.workspace_files.workspace_search_hits = self
            .workspace_files
            .ide_workspace
            .search(&self.workspace_files.workspace_search_query, 100);
        self.runtime_message = format!("{} matches", self.workspace_files.workspace_search_hits.len());
    }

    pub(crate) fn preview_workspace_replace(&mut self) {
        self.workspace_files.workspace_replace_previews = self.workspace_files.ide_workspace.preview_replace(
            &self.workspace_files.workspace_search_query,
            &self.workspace_files.workspace_replace_query,
        );
        self.runtime_message = format!(
            "{} files would change",
            self.workspace_files.workspace_replace_previews.len()
        );
    }

    pub(crate) fn apply_workspace_replace(&mut self) {
        match self.workspace_files.ide_workspace.apply_replace(
            &self.workspace_files.workspace_search_query,
            &self.workspace_files.workspace_replace_query,
        ) {
            Ok(count) => {
                self.preview_workspace_replace();
                self.run_workspace_search();
                self.runtime_message = format!("Replaced {count} occurrence(s)");
            }
            Err(error) => self.runtime_message = error,
        }
    }

    pub(crate) fn add_workspace_context_item(&mut self, item: String) {
        if !self
            .workspace_files
            .workspace_context_items
            .iter()
            .any(|existing| existing == &item)
        {
            self.workspace_files.workspace_context_items.push(item);
        }
    }

    pub(crate) fn clear_workspace_context_items(&mut self) {
        self.workspace_files.workspace_context_items.clear();
    }

    pub(crate) fn export_live_schema_snapshot(&mut self) {
        let mut sql = String::from("-- DB Pro schema snapshot\n");
        for table in &self.schema_explorer.schema.table_details {
            sql.push_str(&format!(
                "-- table {}.{} ({} columns)\n",
                table.schema,
                table.name,
                table.columns.len()
            ));
        }
        match self.workspace_files.ide_workspace.export_schema_snapshot(&sql) {
            Ok(path) => self.runtime_message = format!("Wrote schema snapshot {}", path.display()),
            Err(error) => self.runtime_message = error,
        }
    }

    pub(crate) fn run_workspace_task(&mut self) {
        let command = self.workspace_files.workspace_task_command.clone();
        match self.workspace_files.ide_workspace.run_task(&command) {
            Ok(result) => {
                self.runtime_message = format!("Task exit {:?} · {}ms", result.exit_code, result.duration_ms);
            }
            Err(error) => self.runtime_message = error,
        }
    }

    pub(crate) fn apply_workspace_refactor(&mut self) {
        let from = self.workspace_files.workspace_refactor_from.clone();
        let to = self.workspace_files.workspace_refactor_to.clone();
        match self.workspace_files.ide_workspace.rename_symbol_across_sql(&from, &to) {
            Ok(count) => self.runtime_message = format!("Refactored {count} occurrence(s)"),
            Err(error) => self.runtime_message = error,
        }
    }

    pub(crate) fn toggle_split_editor(&mut self) {
        if self.workspace.split_editor_secondary.is_some() {
            self.workspace.split_editor_secondary = None;
            self.runtime_message = "Split editor closed".to_owned();
            return;
        }
        if self.query_session_state.documents.len() < 2 {
            self.runtime_message = "Open a second document before splitting".to_owned();
            return;
        }
        let secondary = if self.query_session_state.active_document_index + 1 < self.query_session_state.documents.len()
        {
            self.query_session_state.active_document_index + 1
        } else {
            0
        };
        self.workspace.split_editor_secondary = Some(secondary);
        self.runtime_message = "Split editor enabled".to_owned();
    }

    pub(crate) fn refresh_schema_drift_watch(&mut self) {
        let names: Vec<String> = self
            .schema_explorer
            .schema
            .table_details
            .iter()
            .map(|table| format!("{}.{}", table.schema, table.name))
            .collect();
        let fingerprint = ide_workspace::fingerprint_schema_names(&names);
        self.workspace_files
            .ide_workspace
            .update_schema_fingerprint(fingerprint);
        if let Some(message) = self.workspace_files.ide_workspace.schema_drift_message.clone() {
            self.runtime_message = message;
        }
    }

    pub(super) fn open_palette(&mut self, mode: PaletteMode) {
        self.open_palette_with_scope(mode, SearchScope::All);
    }

    pub(super) fn open_palette_with_scope(&mut self, mode: PaletteMode, scope: SearchScope) {
        self.palette.mode = Some(mode);
        self.palette.query.clear();
        self.palette.scope = scope;
        self.palette.selected = 0;
        self.palette.focus_requested = true;
        self.palette.search_index.invalidate();
    }

    pub fn open_new_connection(&mut self) {
        self.connection_dialog
            .transition(super::connection::state::ConnectionDialogAction::OpenNew);
        self.connection_lifecycle.clear_pending_request();
    }

    /// Capture/evidence helper: open the Edit Connection dialog with a test draft so
    /// the password input + eye toggle can be documented (the affected surface for the
    /// input click-steal fix) without needing a real saved connection.
    pub fn open_edit_connection_for_capture(&mut self) {
        self.connection_dialog.editing_connection_id = Some("capture-test".to_owned());
        self.connection_dialog.draft = UiConnectionDraft {
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
        };
        self.connection_dialog.error.clear();
        self.connection_dialog.test_valid = false;
        self.connection_dialog.test_draft = None;
        self.connection_lifecycle.clear_pending_request();
        self.connection_dialog.focus_name_on_open = true;
        self.connection_dialog.open = true;
    }

    /// Capture/evidence helper: open a fresh untitled Query buffer (UI05 editor-first shots).
    pub fn open_query_workspace_for_capture(&mut self) {
        self.dark_mode = true;
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
        self.dark_mode = false;
        self.theme = DbProTheme::light();
    }

    pub(crate) fn request_close_workspace_tab(&mut self, tab: WorkspaceTab) {
        match tab {
            WorkspaceTab::Table => {
                if !self.table_mutation.staged_changes.is_empty() {
                    self.workspace.pending_navigation_action = Some(PendingNavigationAction::CloseWorkspace(tab));
                    self.table_data.discard_changes_confirmation = true;
                    self.runtime_message = "Apply or discard staged changes before closing the table".to_owned();
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
                self.database_operations.schema_workbench.apply_confirmation = false;
            }
            WorkspaceTab::SchemaCompare => {
                self.database_operations.schema_diff = None;
            }
            WorkspaceTab::ComponentGallery => {}
            WorkspaceTab::Welcome | WorkspaceTab::Query => return,
        }
        if self.workspace.active_tab == tab {
            self.activate_welcome_tab();
        }
        self.runtime_message = "Workspace closed".to_owned();
    }

    pub(crate) fn refresh_git_status(&mut self) {
        let Some(root) = self
            .workspace_files
            .ide_workspace
            .primary_path()
            .map(std::path::PathBuf::from)
        else {
            self.workspace_files.git_status = None;
            self.workspace_files.git_last_error = Some("Open a workspace folder first".into());
            return;
        };
        let status = git_workspace::probe_git_status(&root);
        self.workspace_files.git_last_error = if status.available {
            None
        } else {
            Some(status.message.clone())
        };
        self.workspace_files.git_status = Some(status);
        self.runtime_message = "Git status refreshed".into();
    }

    pub(crate) fn stage_git_path(&mut self, relative: &str) {
        let Some(root) = self.workspace_files.ide_workspace.primary_path() else {
            return;
        };
        match git_workspace::stage_path(root, relative) {
            Ok(()) => {
                self.workspace_files.git_last_error = None;
                self.refresh_git_status();
            }
            Err(err) => self.workspace_files.git_last_error = Some(err),
        }
    }

    pub(crate) fn unstage_git_path(&mut self, relative: &str) {
        let Some(root) = self.workspace_files.ide_workspace.primary_path() else {
            return;
        };
        match git_workspace::unstage_path(root, relative) {
            Ok(()) => {
                self.workspace_files.git_last_error = None;
                self.refresh_git_status();
            }
            Err(err) => self.workspace_files.git_last_error = Some(err),
        }
    }

    pub(crate) fn diff_git_path(&mut self, relative: &str) {
        let Some(root) = self.workspace_files.ide_workspace.primary_path() else {
            return;
        };
        match git_workspace::diff_against_head(root, relative) {
            Ok(diff) => {
                self.workspace_files.git_diff = Some(diff);
                self.workspace_files.git_last_error = None;
            }
            Err(err) => self.workspace_files.git_last_error = Some(err),
        }
    }

    pub(crate) fn commit_git_staged(&mut self) {
        let Some(root) = self.workspace_files.ide_workspace.primary_path() else {
            return;
        };
        match git_workspace::commit_paths(root, &self.workspace_files.git_commit_message) {
            Ok(out) => {
                self.workspace_files.git_commit_message.clear();
                self.workspace_files.git_last_error = None;
                self.runtime_message = out.lines().next().unwrap_or("Committed").to_owned();
                self.refresh_git_status();
            }
            Err(err) => self.workspace_files.git_last_error = Some(err),
        }
    }

    pub(crate) fn check_external_file_changes(&mut self) {
        let docs: Vec<(String, String, bool)> = self
            .query_session_state
            .documents
            .iter()
            .filter_map(|doc| {
                let path = doc.file_path.clone()?;
                Some((path, doc.text().to_owned(), doc.dirty))
            })
            .collect();
        for (path, buffer, dirty) in docs {
            let path_buf = std::path::PathBuf::from(&path);
            let Some(mtime) = git_workspace::disk_mtime_secs(&path_buf) else {
                continue;
            };
            let known = self.workspace_files.workspace_file_mtimes.get(&path).copied();
            if known == Some(mtime) {
                continue;
            }
            if dirty && git_workspace::disk_diverged_from_buffer(&path_buf, &buffer) {
                self.workspace_files.workspace_external_change = Some(path);
            } else if !dirty {
                self.workspace_files.workspace_file_mtimes.insert(path, mtime);
            }
        }
    }

    pub(crate) fn reload_workspace_file_from_disk(&mut self, path: &str) {
        let Ok(content) = std::fs::read_to_string(path) else {
            self.runtime_message = format!("Could not reload {path}");
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
                self.workspace_files
                    .workspace_file_mtimes
                    .insert(path.to_owned(), mtime);
            }
            self.workspace_files.workspace_external_change = None;
            self.runtime_message = format!("Reloaded {path}");
        }
    }
}
