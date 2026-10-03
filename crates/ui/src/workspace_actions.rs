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
                name: "idx_users_email".to_owned(),
                columns: vec!["email".to_owned()],
                unique: true,
                primary: false,
                method: "btree".to_owned(),
                definition: "CREATE UNIQUE INDEX idx_users_email ON users(email)".to_owned(),
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
        self.table.data.selected_cell = Some((0, 0));
        self.table.data.selected_rows.insert(0);
        // Cell editing is gated on a connected, writable connection — the
        // fixture provides one so typed-editor captures can render.
        self.connection.catalog.replace(vec![crate::UiConnectionSummary {
            id: "capture-conn".to_owned(),
            name: "Sample E-Commerce (PostgreSQL)".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
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

    /// Capture helper: open the Diagram / ER canvas.
    pub fn open_diagram_workspace_for_capture(&mut self) {
        self.preferences.dark_mode = true;
        self.theme = DbProTheme::dark();
        self.workspace.activity = Activity::Diagram;
        self.workspace.active_tab = WorkspaceTab::Diagram;
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

    pub(crate) fn request_close_workspace_tab(&mut self, tab: WorkspaceTab) {
        match tab {
            WorkspaceTab::Table => {
                if !self.table.mutation.staged_changes.is_empty() {
                    self.workspace.pending_navigation_action = Some(PendingNavigationAction::CloseWorkspace(tab));
                    self.table.editing.discard_changes_confirmation = true;
                    self.feedback.runtime_message =
                        "Apply or discard staged changes before closing the table".to_owned();
                    return;
                }
                self.workspace.pending_navigation_action = None;
                self.schema.explorer.selected_table = None;
                self.table.reset_workspace();
            }
            WorkspaceTab::SchemaObject => {
                self.schema.explorer.selected_schema_object = None;
                self.schema.explorer.schema_object_view = SchemaObjectView::Definition;
                self.table.data_query.result = None;
                self.table.data_query.total_rows = None;
                self.table.data_query.request = None;
            }
            WorkspaceTab::Diagram => {
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
