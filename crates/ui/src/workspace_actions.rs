// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
//! Table open, workspace folder/file, and schema-drift actions.
use super::*;

impl DbProApp {
    pub(crate) fn apply_schema_compare_action(&mut self, action: schema_compare_view::SchemaCompareAction) {
        match action {
            schema_compare_view::SchemaCompareAction::OpenWorkspace => {
                self.workspace.activity = Activity::Compare;
                self.workspace.active_tab = WorkspaceTab::SchemaCompare;
                self.workspace.sidebar_open = true;
            }
            schema_compare_view::SchemaCompareAction::RequestDataDiff => self.request_data_diff_keyed(),
            schema_compare_view::SchemaCompareAction::ApplyMigration => self.apply_migration_preview(),
        }
    }

    pub(crate) fn request_data_diff_keyed(&mut self) {
        let Some(source_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.feedback.runtime_message = "Connect a source database first".into();
            return;
        };
        let request_id = self.next_request_id();
        match self.schema.compare.prepare_data_diff_request() {
            Ok(request) => {
                let command = UiCommand::DiffTableDataKeyed {
                    request_id,
                    source_id,
                    target_id: request.target_id,
                    schema: request.schema,
                    table: request.table,
                    key_columns: request.key_columns,
                    sample_limit: request.sample_limit,
                };
                if self.dispatch_command(command) {
                    self.feedback.runtime_message = "Running key-aware data compare…".into();
                }
            }
            Err(error) => self.feedback.runtime_message = error,
        }
    }

    pub(crate) fn apply_diagram_action(&mut self, action: diagram_view::DiagramAction) {
        match action {
            diagram_view::DiagramAction::OpenTable(table) => self.open_table(table),
            diagram_view::DiagramAction::ExecuteQuery(sql) => {
                self.set_active_query_text(sql);
                self.workspace.active_tab = WorkspaceTab::Query;
                self.dispatch_query();
                self.feedback.runtime_message = "Design Mode mutation plan applied via query runtime".into();
            }
        }
    }

    pub(crate) fn open_table(&mut self, table: String) {
        if self.schema.explorer.selected_table.as_deref() == Some(&table) {
            self.workspace.active_tab = WorkspaceTab::Table;
            self.schema.explorer.record_recent_table(&table);
            return;
        }
        if !self.table.mutation.staged_changes.is_empty() {
            self.workspace.pending_navigation_action = Some(PendingNavigationAction::OpenTable(table));
            self.table.editing.discard_changes_confirmation = true;
            self.feedback.runtime_message = "Apply or discard staged changes before opening another table".to_owned();
            return;
        }
        self.workspace.pending_navigation_action = None;
        let scope = TableDataState::layout_scope(
            self.connection.lifecycle.active_connection_id(),
            self.active_schema(),
            self.schema.explorer.selected_table.as_deref(),
        );
        self.table.data.persist_layout(scope);
        self.schema.explorer.record_recent_table(&table);
        self.schema.explorer.selected_table = Some(table);
        let scope = TableDataState::layout_scope(
            self.connection.lifecycle.active_connection_id(),
            self.active_schema(),
            self.schema.explorer.selected_table.as_deref(),
        );
        self.table.data.restore_layout(scope);
        self.request_table_info();
        self.request_table_data();
        self.workspace.active_tab = WorkspaceTab::Table;
    }

    pub(crate) fn request_open_workspace_folder(&mut self) {
        let request_id = self.next_request_id();
        if self.dispatch_command(UiCommand::PickWorkspaceFolder { request_id }) {
            self.feedback.runtime_message = "Choose a workspace folder…".to_owned();
        }
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
            .query
            .session
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
        self.query.session.documents.push(doc);
        self.query.session.active_document_index = self.query.session.documents.len() - 1;
        self.workspace.activity = Activity::Queries;
        self.workspace.active_tab = WorkspaceTab::Query;
        self.reset_query_cursor();
        self.feedback.runtime_message = format!("Opened {relative_path}");
    }

    pub(crate) fn save_active_workspace_file(&mut self) -> bool {
        let Some(doc) = self
            .query
            .session
            .documents
            .get_mut(self.query.session.active_document_index)
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
                self.workspace.files.dismiss_external_change();
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
        if self.query.session.documents.len() < 2 {
            self.feedback.runtime_message = "Open a second document before splitting".to_owned();
            return;
        }
        let secondary = if self.query.session.active_document_index + 1 < self.query.session.documents.len() {
            self.query.session.active_document_index + 1
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

    /// Capture the static Welcome introduction in either theme.
    pub fn open_welcome_workspace_for_capture(&mut self, light: bool) {
        self.activate_welcome_tab();
        self.preferences.dark_mode = !light;
        self.theme = if light { DbProTheme::light() } else { DbProTheme::dark() };
        self.connection.catalog.replace(vec![crate::UiConnectionSummary {
            id: "sample-ecommerce-conn".to_owned(),
            name: "Sample E-Commerce (SQLite)".to_owned(),
            host: "/tmp/db_pro_sample.db".to_owned(),
            port: 0,
            database: "main".to_owned(),
            username: String::new(),
            driver: "SQLite".to_owned(),
            ssl_mode: crate::UiSslMode::Disable,
            readonly: false,
            tags: vec!["sample".to_owned(), "sqlite".to_owned()],
            group: Some("Local Samples".to_owned()),
            favorite: true,
            environment: "Development".to_owned(),
        }]);
    }

    /// Capture/evidence helper: open a fresh untitled Query buffer (UI05 editor-first shots).
    pub fn open_query_workspace_for_capture(&mut self) {
        self.preferences.dark_mode = true;
        self.theme = DbProTheme::dark();
        self.new_query_document();
        if let Some(doc) = self
            .query
            .session
            .documents
            .get_mut(self.query.session.active_document_index)
        {
            doc.set_text("SELECT u.id, u.email\nFROM users u\nWHERE u.active = true;\n");
            doc.dirty = false;
        }
        self.workspace.bottom_panel_open = false;
        self.query.editor.query_output_dock_maximized = false;
        self.query.editor.query_params_panel_open = false;
        self.query.editor.visual_builder.open = false;
        self.query.editor.editor_search_open = false;
        self.query.editor.snippets_open = false;
        self.query.execution.query_txn_bar_open = false;
    }

    /// Capture helper: same as query workspace but force light theme.
    pub fn open_query_workspace_for_capture_light(&mut self) {
        self.open_query_workspace_for_capture();
        self.preferences.dark_mode = false;
        self.theme = DbProTheme::light();
    }

    /// Capture helper: open the Query workspace with the output dock populated
    /// with a deterministic result grid so evidence shots document the results
    /// toolbar, export entry points, and the docked layout.
    ///
    /// `DB_PRO_CAPTURE_EXPORT` additionally opens the export dialog, and
    /// `DB_PRO_CAPTURE_RESULTS_TAB` promotes the result into the workspace tab.
    /// `DB_PRO_CAPTURE_RESULTS_RIGHT` docks the output panel on the right.
    pub fn open_results_dock_for_capture(&mut self, light: bool) {
        if light {
            self.open_query_workspace_for_capture_light();
        } else {
            self.open_query_workspace_for_capture();
        }
        if let Some(doc) = self
            .query
            .session
            .documents
            .get_mut(self.query.session.active_document_index)
        {
            doc.query_results = vec![capture_results_fixture()];
            doc.active_result_index = 0;
        }
        self.workspace.bottom_panel_open = true;
        self.workspace.bottom_panel_height = 280.0;
        if std::env::var_os("DB_PRO_CAPTURE_RESULTS_RIGHT").is_some() {
            self.workspace.shell.output_dock_position = OutputDockPosition::Right;
        }
        if std::env::var_os("DB_PRO_CAPTURE_EXPORT").is_some() {
            self.overlay.export_open = true;
        }
        if std::env::var_os("DB_PRO_CAPTURE_RESULTS_TAB").is_some() {
            self.open_results_tab();
        }
    }

    /// Capture helper: open the Table workspace with data grid populated.
    pub fn open_table_workspace_for_capture(&mut self) {
        self.preferences.dark_mode = true;
        self.theme = DbProTheme::dark();
        self.workspace.activity = Activity::Explorer;
        self.workspace.active_tab = WorkspaceTab::Table;
        self.schema.explorer.selected_table = Some("customers".to_owned());
        self.table.state.table_view = TableView::Data;
        self.table.state.table_info = Some(crate::UiTableInfo {
            schema: "main".to_owned(),
            name: "customers".to_owned(),
            row_count: Some(3),
            columns: vec![
                crate::UiTableColumn {
                    name: "id".to_owned(),
                    data_type: "uuid".to_owned(),
                    nullable: false,
                    is_primary_key: true,
                    is_unique: true,
                    is_identity: false,
                    is_generated: false,
                    ordinal: 1,
                    default: None,
                    collation: None,
                    enum_labels: Vec::new(),
                },
                crate::UiTableColumn {
                    name: "email".to_owned(),
                    data_type: "text".to_owned(),
                    nullable: false,
                    is_primary_key: false,
                    is_unique: true,
                    is_identity: false,
                    is_generated: false,
                    ordinal: 2,
                    default: None,
                    collation: None,
                    enum_labels: Vec::new(),
                },
                crate::UiTableColumn {
                    name: "active".to_owned(),
                    data_type: "boolean".to_owned(),
                    nullable: false,
                    is_primary_key: false,
                    is_unique: false,
                    is_identity: false,
                    is_generated: false,
                    ordinal: 3,
                    default: Some("true".to_owned()),
                    collation: None,
                    enum_labels: Vec::new(),
                },
                crate::UiTableColumn {
                    name: "status".to_owned(),
                    data_type: "order_status".to_owned(),
                    nullable: true,
                    is_primary_key: false,
                    is_unique: false,
                    is_identity: false,
                    is_generated: false,
                    ordinal: 4,
                    default: None,
                    collation: None,
                    enum_labels: vec!["pending".to_owned(), "shipped".to_owned(), "delivered".to_owned()],
                },
                crate::UiTableColumn {
                    name: "created_at".to_owned(),
                    data_type: "timestamp without time zone".to_owned(),
                    nullable: false,
                    is_primary_key: false,
                    is_unique: false,
                    is_identity: false,
                    is_generated: false,
                    ordinal: 5,
                    default: None,
                    collation: None,
                    enum_labels: Vec::new(),
                },
            ],
            primary_key: Some(vec!["id".to_owned()]),
            indexes: vec![crate::UiTableIndex {
                name: "idx_customers_email".to_owned(),
                columns: vec!["email".to_owned()],
                unique: true,
                primary: false,
                method: "btree".to_owned(),
                definition: "CREATE UNIQUE INDEX idx_customers_email ON customers(email)".to_owned(),
                predicate: None,
                include_columns: Vec::new(),
            }],
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        });
        self.table.data_query.result = Some(crate::UiQueryResult {
            columns: vec![
                crate::UiColumn {
                    name: "id".to_owned(),
                    data_type: "uuid".to_owned(),
                    nullable: false,
                },
                crate::UiColumn {
                    name: "email".to_owned(),
                    data_type: "text".to_owned(),
                    nullable: false,
                },
                crate::UiColumn {
                    name: "active".to_owned(),
                    data_type: "boolean".to_owned(),
                    nullable: false,
                },
                crate::UiColumn {
                    name: "status".to_owned(),
                    data_type: "order_status".to_owned(),
                    nullable: true,
                },
                crate::UiColumn {
                    name: "created_at".to_owned(),
                    data_type: "timestamp without time zone".to_owned(),
                    nullable: false,
                },
            ],
            rows: vec![
                vec![
                    crate::UiCell::Text("a1b2c3d4-e5f6-7890-1234-56789abcdef0".to_owned()),
                    crate::UiCell::Text("alice@example.com".to_owned()),
                    crate::UiCell::Boolean(true),
                    crate::UiCell::Text("pending".to_owned()),
                    crate::UiCell::Text("2026-09-30 14:22:11".to_owned()),
                ],
                vec![
                    crate::UiCell::Text("b2c3d4e5-f6a7-8901-2345-6789abcdef01".to_owned()),
                    crate::UiCell::Text("bob@example.com".to_owned()),
                    crate::UiCell::Boolean(true),
                    crate::UiCell::Text("shipped".to_owned()),
                    crate::UiCell::Text("2026-09-29 09:05:44".to_owned()),
                ],
                vec![
                    crate::UiCell::Text("c3d4e5f6-a7b8-9012-3456-789abcdef012".to_owned()),
                    crate::UiCell::Text("charlie@example.com".to_owned()),
                    crate::UiCell::Boolean(false),
                    crate::UiCell::Null,
                    crate::UiCell::Text("2026-09-28 18:47:02".to_owned()),
                ],
            ],
            row_count: 3,
            duration_ms: 1,
        });
        // a selected cell + row so captures document the selection styling
        self.table.data.select_single_cell((0, 0));
        // Cell editing is gated on a connected, writable connection — the
        // fixture provides one so typed-editor captures can render.
        self.connection.catalog.replace(vec![crate::UiConnectionSummary {
            id: "capture-conn".to_owned(),
            name: "Sample E-Commerce (PostgreSQL)".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            // cc-scan:allow DUPLICATE_BLOCK — coincidental boilerplate, not a real clone
            database: "app".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: crate::UiSslMode::Require,
            readonly: false,
            tags: Vec::new(),
            group: None,
            favorite: false,
            environment: "Development".to_owned(),
        }]);
        self.connection.lifecycle.set_connected(true);
        self.connection
            .lifecycle
            .set_active_connection_id(Some("capture-conn".to_owned()));
        self.apply_capture_cell_edit_env();
    }

    /// `DB_PRO_CAPTURE_CELL_EDIT=<row>,<col>` opens that cell's inline editor
    /// so captures can document each typed editor. Read once; zero cost unset.
    pub(super) fn apply_capture_cell_edit_env(&mut self) {
        static TARGET: std::sync::OnceLock<Option<(usize, usize)>> = std::sync::OnceLock::new();
        let Some((row_index, column_index)) = *TARGET.get_or_init(|| {
            std::env::var("DB_PRO_CAPTURE_CELL_EDIT").ok().and_then(|raw| {
                let (row, column) = raw.split_once(',')?;
                Some((row.trim().parse().ok()?, column.trim().parse().ok()?))
            })
        }) else {
            return;
        };
        let Some(cell) = self
            .table
            .data_query
            .result
            .as_ref()
            .and_then(|result| result.rows.get(row_index))
            .and_then(|row| row.get(column_index))
        else {
            return;
        };
        self.table.editing.data_editing_cell = Some((row_index, column_index));
        self.table.editing.data_edit_value = crate::app::cell_inspector::cell_raw_text(cell);
    }

    /// Capture helper: same as table workspace but force light theme.
    pub fn open_table_workspace_for_capture_light(&mut self) {
        self.open_table_workspace_for_capture();
        self.preferences.dark_mode = false;
        self.theme = DbProTheme::light();
    }

    /// Capture the loaded record or a table placeholder without a provider connection.
    pub fn prepare_table_record_for_capture(&mut self, state: &str) {
        self.table.data.select_single_row(0);
        self.table.editing.record_view_open = true;
        match state {
            "changed" => {
                if let Some(result) = self.table.data_query.result.take() {
                    self.table.editing.data_editing_cell = Some((0, 1));
                    self.table.editing.data_edit_value = "updated@example.com".to_owned();
                    self.submit_data_cell_edit(&result, 0, 1);
                    self.table.data_query.result = Some(result);
                }
                self.table.editing.record_view_open = false;
            }
            "loading" => {
                self.table.data_query.result = None;
                self.table.data_query.error = None;
            }
            "error" => {
                self.table.data_query.result = None;
                self.table.data_query.error = Some("The database connection was interrupted.".to_owned());
            }
            "empty" => {
                if let Some(result) = &mut self.table.data_query.result {
                    result.rows.clear();
                    result.row_count = 0;
                }
                self.table.data_query.total_rows = Some(0);
                self.table.data.clear_selection();
                self.table.editing.record_view_open = false;
            }
            _ => {}
        }
    }

    /// Capture helper: open the deterministic table fixture directly on Profile.
    pub fn open_table_profile_for_capture(&mut self, light: bool) {
        if light {
            self.open_table_workspace_for_capture_light();
        } else {
            self.open_table_workspace_for_capture();
        }
        if std::env::var_os("DB_PRO_CAPTURE_PROFILE_NUMERIC").is_some() {
            if let Some(result) = self.table.data_query.result.as_mut() {
                result.columns[0].data_type = "BIGINT".to_owned();
                for row in &mut result.rows { row[0] = crate::UiCell::Number("1000000000000000000".to_owned()); }
            }
        }
        self.table.state.table_view = TableView::Profile;
    }

    /// Capture helper: open the deterministic table fixture directly on Structure.
    pub fn open_table_structure_for_capture(&mut self, light: bool) {
        if light {
            self.open_table_workspace_for_capture_light();
        } else {
            self.open_table_workspace_for_capture();
        }
        self.table.state.table_view = TableView::Structure;
    }

    /// Capture helper: open the deterministic table fixture directly on Foreign Keys.
    pub fn open_table_foreign_keys_for_capture(&mut self, light: bool) {
        if light { self.open_table_workspace_for_capture_light(); }
        else { self.open_table_workspace_for_capture(); }
        if let Some(info) = self.table.state.table_info.as_mut() {
            info.foreign_keys = vec![crate::UiTableForeignKey {
                name: "customers_owner_fk".to_owned(), from_columns: vec!["id".to_owned()],
                to_schema: "main".to_owned(), to_table: "owners".to_owned(), to_columns: vec!["id".to_owned()],
                on_update: "NO ACTION".to_owned(), on_delete: "NO ACTION".to_owned(),
                match_option: "SIMPLE".to_owned(), deferrable: false, initially_deferred: false,
            }];
        }
        self.table.state.table_view = TableView::Relations;
    }

    /// Capture helper: open the deterministic table fixture directly on Constraints.
    pub fn open_table_constraints_for_capture(&mut self, light: bool) {
        if light {
            self.open_table_workspace_for_capture_light();
        } else {
            self.open_table_workspace_for_capture();
        }
        self.table.state.table_view = TableView::Constraints;
    }

    /// Capture helper: show outgoing dependencies and a matching/total count.
    pub fn open_table_dependencies_for_capture(&mut self, light: bool) {
        if light {
            self.open_table_workspace_for_capture_light();
        } else {
            self.open_table_workspace_for_capture();
        }
        if let Some(info) = self.table.state.table_info.as_mut() {
            info.dependencies = vec![
                crate::UiTableDependency {
                    name: "products".to_owned(),
                    schema: "main".to_owned(),
                    kind: crate::UiDependencyKind::Table,
                    direction: crate::UiDependencyDirection::DependsOn,
                    details: "Foreign key order_items_fk_0 (product_id) → main.products(id)".to_owned(),
                },
                crate::UiTableDependency {
                    name: "orders".to_owned(),
                    schema: "main".to_owned(),
                    kind: crate::UiDependencyKind::Table,
                    direction: crate::UiDependencyDirection::DependsOn,
                    details: "Foreign key order_items_fk_1 (order_id) → main.orders(id)".to_owned(),
                },
                crate::UiTableDependency {
                    name: "order_items".to_owned(),
                    schema: "main".to_owned(),
                    kind: crate::UiDependencyKind::Table,
                    direction: crate::UiDependencyDirection::DependedBy,
                    details: "Referenced by foreign key order_items_customer_fk (main.order_items) → (id)".to_owned(),
                },
            ];
        }
        self.table.state.table_dependency_filter = "depends_on".to_owned();
        self.table.state.table_view = TableView::Dependencies;
    }

    /// Capture helper: open a loaded DDL preview on the DDL tab.
    pub fn open_table_ddl_for_capture(&mut self, light: bool) {
        if light {
            self.open_table_workspace_for_capture_light();
        } else {
            self.open_table_workspace_for_capture();
        }
        self.table.state.table_ddl = Some(
            r#"CREATE TABLE "customers" (
    "id" INTEGER,
    "first_name" TEXT NOT NULL,
    "last_name" TEXT NOT NULL,
    "email" TEXT NOT NULL,
    "created_at" DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id"),
    UNIQUE ("email")
);"#.to_owned(),
        );
        self.table.state.table_ddl_error = None;
        self.table.state.table_view = TableView::Ddl;
    }

    /// Capture helper: open the deterministic table fixture directly on Indexes.
    pub fn open_table_indexes_for_capture(&mut self, light: bool) {
        if light {
            self.open_table_workspace_for_capture_light();
        } else {
            self.open_table_workspace_for_capture();
        }
        self.table.state.table_view = TableView::Indexes;
    }

    /// Capture helper: show table-column completion in the inline condition field.
    pub fn open_table_condition_completion_for_capture(&mut self, light: bool) {
        if light {
            self.open_table_workspace_for_capture_light();
        } else {
            self.open_table_workspace_for_capture();
        }
        self.table.data_query.sql_condition_draft = "em".to_owned();
    }

    /// Capture/evidence helper: open the History activity with deterministic
    /// execution records so the unified history surface can be documented.
    pub fn open_history_activity_for_capture(&mut self) {
        self.preferences.dark_mode = true;
        self.theme = DbProTheme::dark();
        self.workspace.activity = Activity::History;
        self.workspace.active_tab = WorkspaceTab::Query;
        self.connection.catalog.replace(vec![crate::UiConnectionSummary {
            id: "capture-conn".to_owned(),
            name: "local-pg".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app_db".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: crate::UiSslMode::Require,
            readonly: false,
            tags: Vec::new(),
            group: None,
            favorite: false,
            environment: "Development".to_owned(),
        }]);
        self.connection.lifecycle.set_connected(true);
        self.connection
            .lifecycle
            .set_active_connection_id(Some("capture-conn".to_owned()));
        let now = chrono::Utc::now();
        let yesterday = now - chrono::Duration::days(1);
        // History entries are stored oldest → newest; the view reverses for display.
        self.query.editor.query_history_entries = vec![
            crate::UiQueryHistoryEntry {
                id: "exec-capture-0003".to_owned(),
                sql: "SELECT count(*) FROM events;".to_owned(),
                connection_id: Some("capture-conn".to_owned()),
                schema: Some("public".to_owned()),
                started_at: yesterday.to_rfc3339(),
                duration_ms: 1204,
                status: crate::UiQueryHistoryStatus::Cancelled,
                row_count: None,
                affected_rows: None,
                error_code: None,
                error_summary: None,
            },
            crate::UiQueryHistoryEntry {
                id: "exec-capture-0002".to_owned(),
                sql: "SELECT * FROM user_profiles;".to_owned(),
                connection_id: Some("capture-conn".to_owned()),
                schema: Some("audit".to_owned()),
                started_at: (now - chrono::Duration::minutes(3)).to_rfc3339(),
                duration_ms: 2,
                status: crate::UiQueryHistoryStatus::Failed,
                row_count: None,
                affected_rows: None,
                error_code: Some("42P01".to_owned()),
                error_summary: Some("relation public.user_profiles does not exist".to_owned()),
            },
            crate::UiQueryHistoryEntry {
                id: "exec-capture-0001".to_owned(),
                sql: "SELECT id, email FROM customers ORDER BY email LIMIT 100;".to_owned(),
                connection_id: Some("capture-conn".to_owned()),
                schema: Some("public".to_owned()),
                started_at: now.to_rfc3339(),
                duration_ms: 12,
                status: crate::UiQueryHistoryStatus::Success,
                row_count: Some(100),
                affected_rows: None,
                error_code: None,
                error_summary: None,
            },
        ];
        self.query.editor.history_selected_id = Some("exec-capture-0002".to_owned());
    }

    /// Capture/evidence helper: Explorer with a non-default object filter so the
    /// active-filter indicator and hidden kinds can be documented.
    pub fn open_explorer_filter_for_capture(&mut self) {
        self.preferences.dark_mode = true;
        self.theme = DbProTheme::dark();
        self.workspace.activity = Activity::Explorer;
        self.schema.explorer.explorer_filter.views = false;
        self.schema.explorer.explorer_filter.mode = ExplorerMatchMode::Prefix;
    }

    /// Capture/evidence helper: open the Compare workspace with a real
    /// snapshot→diff→plan chain recorded against a different session target,
    /// so the spec-09 safety lock is exercised end-to-end.
    pub fn open_schema_compare_for_capture(&mut self) {
        self.preferences.dark_mode = true;
        self.theme = DbProTheme::dark();
        self.workspace.activity = Activity::Compare;
        self.workspace.active_tab = WorkspaceTab::SchemaCompare;
        self.connection.catalog.replace(vec![crate::UiConnectionSummary {
            id: "capture-conn".to_owned(),
            name: "local-pg".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app_db".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: crate::UiSslMode::Require,
            readonly: false,
            tags: Vec::new(),
            group: None,
            favorite: false,
            environment: "Development".to_owned(),
        }]);
        self.connection.lifecycle.set_connected(true);
        self.connection
            .lifecycle
            .set_active_connection_id(Some("capture-conn".to_owned()));
        let column = |name: &str, data_type: &str| crate::UiSchemaColumn {
            name: name.to_owned(),
            data_type: data_type.to_owned(),
            nullable: true,
            is_primary_key: false,
        };
        let table = |name: &str, columns: Vec<crate::UiSchemaColumn>| crate::UiTableSummary {
            schema: "public".to_owned(),
            name: name.to_owned(),
            row_count: Some(42),
            columns,
            foreign_keys: Vec::new(),
        };
        let mut summary = crate::UiSchemaSummary {
            schemas: vec!["public".to_owned()],
            tables: vec!["public.orders".to_owned(), "public.users".to_owned()],
            table_details: vec![
                table(
                    "orders",
                    vec![column("id", "integer"), column("total", "numeric")],
                ),
                table("users", vec![column("id", "integer"), column("email", "text")]),
            ],
            ..Default::default()
        };
        // Snapshot predates the `users` table and the orders.total type change.
        let mut snapshot_summary = summary.clone();
        snapshot_summary.tables.retain(|t| t != "public.users");
        snapshot_summary.table_details.retain(|t| t.name != "users");
        for detail in &mut snapshot_summary.table_details {
            if detail.name == "orders" {
                for col in &mut detail.columns {
                    if col.name == "total" {
                        col.data_type = "integer".to_owned();
                    }
                }
            }
        }
        self.schema.explorer.schema = std::mem::take(&mut summary);
        self.schema
            .compare
            .take_snapshot(&snapshot_summary, "local-pg", &mut self.feedback);
        let live = self.schema.explorer.schema.clone();
        self.schema
            .compare
            .diff_against_snapshot(&live, &mut self.feedback);
        self.schema.compare.plan_migration(
            "PostgreSQL",
            "prod-pg · public",
            &mut self.feedback,
        );
    }

    /// Capture helper: open the Diagram / ER canvas on a deterministic schema
    /// fixture so evidence shots render real nodes, edges, and the minimap.
    pub fn open_diagram_workspace_for_capture(&mut self) {
        let light = std::env::var_os("DB_PRO_CAPTURE_DIAGRAM_LIGHT").is_some();
        self.preferences.dark_mode = !light;
        self.theme = if light { DbProTheme::light() } else { DbProTheme::dark() };
        self.workspace.activity = Activity::Diagram;
        self.open_diagram_tab();
        self.connection.catalog.replace(vec![crate::UiConnectionSummary {
            id: "capture-conn".to_owned(),
            name: "Sample E-Commerce (SQLite)".to_owned(),
            host: "/tmp/db_pro_sample.db".to_owned(),
            port: 0,
            database: "main".to_owned(),
            username: String::new(),
            driver: "SQLite".to_owned(),
            ssl_mode: crate::UiSslMode::Disable,
            readonly: false,
            tags: Vec::new(),
            group: None,
            favorite: false,
            environment: "Development".to_owned(),
        }]);
        self.connection.lifecycle.set_connected(true);
        self.connection
            .lifecycle
            .set_active_connection_id(Some("capture-conn".to_owned()));
        self.schema.explorer.schema = diagram_capture_schema();
        // `DB_PRO_CAPTURE_DIAGRAM_ZOOM` re-centers the world at a fixed zoom so
        // evidence shots can document a specific level of detail.
        self.schema.diagram.capture_zoom_override = std::env::var("DB_PRO_CAPTURE_DIAGRAM_ZOOM")
            .ok()
            .and_then(|raw| raw.parse::<f32>().ok());
    }

    /// Capture helper: open the Settings panel.
    pub fn open_settings_workspace_for_capture(&mut self) {
        let light = std::env::var_os("DB_PRO_CAPTURE_SETTINGS_LIGHT").is_some();
        self.preferences.dark_mode = !light;
        self.theme = if light { DbProTheme::light() } else { DbProTheme::dark() };
        self.workspace.activity = Activity::Settings;
        self.workspace.sidebar_open = false;
    }

    /// Capture helper: open the Agent sidebar panel.
    pub fn open_agent_workspace_for_capture(&mut self) {
        self.preferences.dark_mode = true;
        self.theme = DbProTheme::dark();
        self.workspace.agent_open = true;
    }

    /// Capture helper: open the native Component Gallery in its canonical dark theme.
    pub fn open_component_gallery_for_capture(&mut self) {
        let light = std::env::var_os("DB_PRO_CAPTURE_GALLERY_LIGHT").is_some();
        self.preferences.dark_mode = !light;
        self.theme = if light { DbProTheme::light() } else { DbProTheme::dark() };
        self.gallery_state = ComponentGalleryState::default();
        self.gallery_state.category = match std::env::var("DB_PRO_CAPTURE_GALLERY_SECTION").as_deref() {
            Ok("buttons") => component_gallery_view::GalleryCategory::Buttons,
            Ok("badges") => component_gallery_view::GalleryCategory::Badges,
            Ok("form") | Ok("form-error") | Ok("disclosure") | Ok("calendar") => {
                component_gallery_view::GalleryCategory::Inputs
            }
            Ok("cards") => component_gallery_view::GalleryCategory::Cards,
            Ok("selection") => component_gallery_view::GalleryCategory::Selection,
            Ok("layout") => component_gallery_view::GalleryCategory::Layout,
            Ok("alerts") => component_gallery_view::GalleryCategory::Alerts,
            Ok("feedback") => component_gallery_view::GalleryCategory::Feedback,
            Ok("overlays") => component_gallery_view::GalleryCategory::Overlays,
            Ok("navigation") => component_gallery_view::GalleryCategory::Navigation,
            Ok("tables") => component_gallery_view::GalleryCategory::Tables,
            Ok("devtools") => component_gallery_view::GalleryCategory::DevTools,
            Ok("database-shell") => component_gallery_view::GalleryCategory::DatabaseShell,
            Ok("agent") => component_gallery_view::GalleryCategory::AgentUi,
            Ok("rendering") => component_gallery_view::GalleryCategory::Rendering,
            _ => component_gallery_view::GalleryCategory::Badges,
        };
        if std::env::var("DB_PRO_CAPTURE_GALLERY_SECTION").as_deref() == Ok("form-error") {
            self.gallery_state.form_name.clear();
            self.gallery_state.form_host.clear();
            self.gallery_state.form_error = Some("Correct the highlighted fields, then save again.".to_owned());
            self.gallery_state.form_state.submitted = true;
            self.gallery_state.form_state.validate_field("form_name", "");
            self.gallery_state.form_state.validate_field("form_host", "");
        }
        self.workspace.active_tab = WorkspaceTab::ComponentGallery;
    }

    /// Capture helper: open Quick Open with the Schema filter selected.
    pub fn open_quick_open_for_capture(&mut self, light: bool, empty: bool) {
        self.preferences.dark_mode = !light;
        self.theme = if light { DbProTheme::light() } else { DbProTheme::dark() };
        self.palette
            .open_with_scope(PaletteMode::QuickOpen, SearchScope::Schema);
        if empty {
            self.palette.query = "__no_matching_quick_open_item__".to_owned();
        }
    }

    pub(crate) fn open_diagram_tab(&mut self) {
        self.workspace.diagram_open = true;
        self.workspace.active_tab = WorkspaceTab::Diagram;
    }

    /// Promote the query output to a full workspace tab so the result grid can be
    /// reviewed without the dock's narrow split taking screen space.
    pub(crate) fn open_results_tab(&mut self) {
        self.workspace.results_open = true;
        self.workspace.active_tab = WorkspaceTab::Results;
    }

    pub(crate) fn request_close_workspace_tab(&mut self, tab: WorkspaceTab) {
        match tab {
            WorkspaceTab::Table => {
                if !self.request_close_table_tab(tab) {
                    return;
                }
            }
            WorkspaceTab::SchemaObject => {
                self.schema.explorer.selected_schema_object = None;
                self.schema.explorer.schema_object_view = SchemaObjectView::Definition;
                self.table.data_query.result = None;
                self.table.data_query.total_rows = None;
                self.table.data_query.request = None;
            }
            WorkspaceTab::Results => {
                self.workspace.results_open = false;
            }
            WorkspaceTab::Diagram => {
                self.workspace.diagram_open = false;
                self.schema.diagram.search.clear();
                self.schema.diagram.show_all = false;
                self.schema.diagram.pan = egui::Vec2::ZERO;
                self.schema.diagram.pan_origin = None;
            }
            WorkspaceTab::SchemaWorkbench => {
                self.schema.workbench.apply_confirmation = false;
            }
            WorkspaceTab::SchemaCompare => {
                self.schema.compare.schema_diff = None;
            }
            WorkspaceTab::ComponentGallery => {}
            WorkspaceTab::Welcome | WorkspaceTab::Query => return,
        }
        if self.workspace.active_tab == tab {
            self.activate_welcome_tab();
        }
        self.feedback.runtime_message = "Workspace closed".to_owned();
    }

    /// Returns false when the close is blocked by staged table changes — the
    /// discard confirmation must be resolved before the tab can go away.
    fn request_close_table_tab(&mut self, tab: WorkspaceTab) -> bool {
        if !self.table.mutation.staged_changes.is_empty() {
            self.workspace.pending_navigation_action = Some(PendingNavigationAction::CloseWorkspace(tab));
            self.table.editing.discard_changes_confirmation = true;
            self.feedback.runtime_message =
                "Apply or discard staged changes before closing the table".to_owned();
            return false;
        }
        self.workspace.pending_navigation_action = None;
        self.schema.explorer.selected_table = None;
        self.table.reset_workspace();
        true
    }

    pub(crate) fn reload_workspace_file_from_disk(&mut self, path: &str) {
        let Ok(content) = std::fs::read_to_string(path) else {
            self.feedback.runtime_message = format!("Could not reload {path}");
            return;
        };
        if let Some(doc) = self
            .query
            .session
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
            self.workspace.files.dismiss_external_change();
            self.feedback.runtime_message = format!("Reloaded {path}");
        }
    }
}

/// Deterministic result fixture for `DB_PRO_CAPTURE_RESULTS` evidence runs —
/// a small grid that exercises the results toolbar, export entry points, and
/// the docked/right-docked layout without needing a live database.
fn capture_results_fixture() -> crate::UiQueryResult {
    crate::UiQueryResult {
        columns: ["id", "first_name", "last_name", "email"]
            .iter()
            .map(|name| crate::UiColumn {
                name: (*name).to_owned(),
                data_type: if *name == "id" { "integer" } else { "text" }.to_owned(),
                nullable: false,
            })
            .collect(),
        rows: [
            ("1", "Alice", "Johnson", "alice@example.com"),
            ("2", "Bob", "Smith", "bob@example.com"),
            ("3", "Charlie", "Brown", "charlie@example.com"),
            ("4045", "Alex", "Morgan", "user_1278@example.com"),
            ("7361", "Alex", "Morgan", "user_3816@example.com"),
            ("8034", "Alex", "Morgan", "user_6548@example.com"),
        ]
        .iter()
        .map(|row| {
            [
                crate::UiCell::Number(row.0.to_owned()),
                crate::UiCell::Text(row.1.to_owned()),
                crate::UiCell::Text(row.2.to_owned()),
                crate::UiCell::Text(row.3.to_owned()),
            ]
            .to_vec()
        })
        .collect(),
        row_count: 6,
        duration_ms: 1,
    }
}

/// Deterministic schema fixture for `DB_PRO_CAPTURE_DIAGRAM` evidence runs —
/// a small e-commerce schema with enough relationships to exercise edges,
/// the minimap, and LOD switches without needing a live database.
fn diagram_capture_schema() -> crate::UiSchemaSummary {
    let col = |name: &str, data_type: &str, nullable: bool, pk: bool| crate::UiSchemaColumn {
        name: name.to_owned(),
        data_type: data_type.to_owned(),
        nullable,
        is_primary_key: pk,
    };
    let fk = |name: &str, from: &[&str], to_table: &str, to: &[&str]| crate::UiSchemaForeignKey {
        name: name.to_owned(),
        from_columns: from.iter().map(|c| c.to_string()).collect(),
        to_schema: "main".to_owned(),
        to_table: to_table.to_owned(),
        to_columns: to.iter().map(|c| c.to_string()).collect(),
    };
    let table = |name: &str,
                 columns: Vec<crate::UiSchemaColumn>,
                 foreign_keys: Vec<crate::UiSchemaForeignKey>,
                 rows: u64| crate::UiTableSummary {
        schema: "main".to_owned(),
        name: name.to_owned(),
        row_count: Some(rows),
        columns,
        foreign_keys,
    };

    let details = vec![
        table(
            "customers",
            vec![
                col("id", "INTEGER", false, true),
                col("full_name", "TEXT", false, false),
                col("email", "TEXT", false, false),
                col("created_at", "DATETIME", false, false),
            ],
            vec![],
            1420,
        ),
        table(
            "addresses",
            vec![
                col("id", "INTEGER", false, true),
                col("customer_id", "INTEGER", false, false),
                col("street", "TEXT", false, false),
                col("city", "TEXT", false, false),
                col("country_code", "TEXT", false, false),
            ],
            vec![fk("addresses_customer_fk", &["customer_id"], "customers", &["id"])],
            2310,
        ),
        table(
            "orders",
            vec![
                col("id", "INTEGER", false, true),
                col("customer_id", "INTEGER", false, false),
                col("status", "TEXT", false, false),
                col("ordered_at", "DATETIME", false, false),
                col("total", "REAL", false, false),
            ],
            vec![fk("orders_customer_fk", &["customer_id"], "customers", &["id"])],
            5844,
        ),
        table(
            "order_items",
            vec![
                col("order_id", "INTEGER", false, true),
                col("product_id", "INTEGER", false, true),
                col("quantity", "INTEGER", false, false),
                col("unit_price", "REAL", false, false),
            ],
            vec![
                fk("order_items_order_fk", &["order_id"], "orders", &["id"]),
                fk("order_items_product_fk", &["product_id"], "products", &["id"]),
            ],
            18_233,
        ),
        table(
            "products",
            vec![
                col("id", "INTEGER", false, true),
                col("category_id", "INTEGER", false, false),
                col("name", "TEXT", false, false),
                col("price", "REAL", false, false),
                col("stock", "INTEGER", false, false),
            ],
            vec![fk("products_category_fk", &["category_id"], "categories", &["id"])],
            931,
        ),
        table(
            "categories",
            vec![
                col("id", "INTEGER", false, true),
                col("name", "TEXT", false, false),
                col("parent_id", "INTEGER", true, false),
            ],
            vec![fk("categories_parent_fk", &["parent_id"], "categories", &["id"])],
            48,
        ),
        table(
            "payments",
            vec![
                col("id", "INTEGER", false, true),
                col("order_id", "INTEGER", false, false),
                col("method", "TEXT", false, false),
                col("amount", "REAL", false, false),
                col("paid_at", "DATETIME", true, false),
            ],
            vec![fk("payments_order_fk", &["order_id"], "orders", &["id"])],
            5712,
        ),
        table(
            "shipments",
            vec![
                col("id", "INTEGER", false, true),
                col("order_id", "INTEGER", false, false),
                col("address_id", "INTEGER", false, false),
                col("carrier", "TEXT", true, false),
                col("shipped_at", "DATETIME", true, false),
            ],
            vec![
                fk("shipments_order_fk", &["order_id"], "orders", &["id"]),
                fk("shipments_address_fk", &["address_id"], "addresses", &["id"]),
            ],
            5401,
        ),
        table(
            "reviews",
            vec![
                col("id", "INTEGER", false, true),
                col("product_id", "INTEGER", false, false),
                col("customer_id", "INTEGER", false, false),
                col("rating", "INTEGER", false, false),
                col("comment", "TEXT", true, false),
            ],
            vec![
                fk("reviews_product_fk", &["product_id"], "products", &["id"]),
                fk("reviews_customer_fk", &["customer_id"], "customers", &["id"]),
            ],
            3904,
        ),
        table(
            "inventory_logs",
            vec![
                col("id", "INTEGER", false, true),
                col("product_id", "INTEGER", false, false),
                col("delta", "INTEGER", false, false),
                col("logged_at", "DATETIME", false, false),
            ],
            vec![fk("inventory_logs_product_fk", &["product_id"], "products", &["id"])],
            22_870,
        ),
    ];

    crate::UiSchemaSummary {
        schemas: vec!["main".to_owned()],
        tables: details.iter().map(|t| format!("{}.{}", t.schema, t.name)).collect(),
        table_details: details,
        ..Default::default()
    }
}
