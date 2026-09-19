use super::*;

const THEME_STORAGE_VERSION: &str = "light-first-v1";

/// Query seeded into the editor and the first query tab at startup.
const DEFAULT_QUERY: &str = "select\n  id, name, status\nfrom customers\nlimit 100;";

impl DbProApp {
    pub fn with_task_bridge(task_bridge: TaskBridge) -> Self {
        Self::with_task_bridge_and_storage(task_bridge, None)
    }

    pub fn with_task_bridge_and_storage(task_bridge: TaskBridge, storage: Option<&dyn eframe::Storage>) -> Self {
        let mut app = Self {
            task_bridge,
            ..Self::default()
        };
        if let Some(storage) = storage {
            app.dark_mode =
                if storage.get_string("dbpro.native.theme-version").as_deref() == Some(THEME_STORAGE_VERSION) {
                    storage
                        .get_string("dbpro.native.dark-mode")
                        .map(|value| value == "true")
                        .unwrap_or(false)
                } else {
                    false
                };
            app.reduce_motion = storage
                .get_string("dbpro.native.reduce-motion")
                .is_some_and(|value| value == "true");
            if let Some(mode) = storage
                .get_string("dbpro.native.prediction-mode")
                .and_then(|value| serde_json::from_str::<PredictionMode>(&value).ok())
            {
                app.prediction_mode = mode;
            }
            // Typed settings blob wins when present (#205).
            if let Some(raw) = storage.get_string(SETTINGS_STORAGE_KEY) {
                if let Some(settings) = AppSettings::from_json(&raw) {
                    app.settings = settings;
                    app.apply_settings_to_runtime();
                }
            } else {
                app.sync_settings_from_runtime();
            }
            app.load_saved_tasks_from_storage(storage);
            app.load_named_sessions_from_storage(storage);
            if let Some(raw) = storage.get_string("dbpro.native.ssh-profiles-v1") {
                if let Ok(profiles) = serde_json::from_str(&raw) {
                    app.connection_dialog.ssh_profiles = profiles;
                }
            }
            if let Some(width) = storage
                .get_string("dbpro.native.sidebar-width")
                .and_then(|value| value.parse::<f32>().ok())
            {
                app.workspace.set_sidebar_width(width);
            }
            if let Some(width) = storage
                .get_string("dbpro.native.agent-width")
                .and_then(|value| value.parse::<f32>().ok())
            {
                app.workspace.set_agent_width(width);
            }
            app.workspace.bottom_panel_open = storage
                .get_string("dbpro.native.output-open")
                .is_some_and(|value| value == "true");
            if let Some(height) = storage
                .get_string("dbpro.native.output-height")
                .and_then(|value| value.parse::<f32>().ok())
            {
                app.workspace.set_bottom_panel_height(height);
            }
            if let Some(height) = storage
                .get_string("dbpro.native.connections-pane-height")
                .and_then(|value| value.parse::<f32>().ok())
            {
                app.connections_pane_height = height.clamp(80.0, 400.0);
            }
            if let Some(height) = storage
                .get_string("dbpro.native.schemas-pane-height")
                .and_then(|value| value.parse::<f32>().ok())
            {
                app.schemas_pane_height = height.clamp(60.0, 200.0);
            }
            app.theme = if app.dark_mode {
                DbProTheme::dark()
            } else {
                DbProTheme::light()
            };
            if let Some(widths) = storage.get_string("dbpro.native.grid-widths") {
                if let Ok(widths) = serde_json::from_str::<Vec<f32>>(&widths) {
                    app.table_data.grid_column_widths =
                        widths.into_iter().map(|width| width.clamp(90.0, 520.0)).collect();
                }
            }
            if let Some(layouts) = storage.get_string("dbpro.native.grid-layouts") {
                if let Ok(layouts) = serde_json::from_str(&layouts) {
                    app.table_data.grid_layout_preferences = layouts;
                }
            }
            app.table_data.grid_columns_user_resized = storage
                .get_string("dbpro.native.grid-widths-customized")
                .is_some_and(|value| value == "true");
            if let Some(documents) = storage.get_string("dbpro.native.query-documents") {
                if let Ok(documents) = serde_json::from_str::<Vec<QueryDocument>>(&documents) {
                    if !documents.is_empty() {
                        app.query_session_state.documents = documents;
                    }
                }
            }
            if let Some(history) = storage.get_string("dbpro.native.query-history-v1") {
                if let Ok(history) = serde_json::from_str(&history) {
                    app.query_history_entries = history;
                }
            }
            if let Some(pinned) = storage.get_string("dbpro.native.pinned-tables-v1") {
                if let Ok(tables) = serde_json::from_str::<Vec<String>>(&pinned) {
                    app.pinned_tables = tables;
                }
            }
            // Shell/layout restore after documents + pins so tab refs resolve (#222).
            app.restore_last_workspace_session_from_storage(storage);
            if let Some(recent) = storage.get_string("dbpro.native.recent-tables-v1") {
                if let Ok(tables) = serde_json::from_str::<Vec<String>>(&recent) {
                    app.recent_tables = tables;
                }
            }
            if let Some(recent_ws) = storage.get_string("dbpro.native.workspace-recent-v1") {
                if let Ok(paths) = serde_json::from_str::<Vec<String>>(&recent_ws) {
                    app.ide_workspace.recent_roots = paths.into_iter().map(std::path::PathBuf::from).collect();
                }
            }
            let roots = storage
                .get_string("dbpro.native.workspace-roots-v1")
                .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
                .or_else(|| {
                    storage
                        .get_string("dbpro.native.workspace-root-v1")
                        .filter(|root| !root.is_empty())
                        .map(|root| vec![root])
                })
                .unwrap_or_default();
            for root in roots {
                let path = std::path::PathBuf::from(root);
                if path.is_dir() {
                    let _ = if app.ide_workspace.roots.is_empty() {
                        app.ide_workspace.open_root(path)
                    } else {
                        app.ide_workspace.add_root(path)
                    };
                }
            }
            if storage.get_string("dbpro.native.workspace-trusted-v1").as_deref() == Some("true") {
                app.ide_workspace.set_trusted(true);
            }
        }
        app
    }
}

impl Default for DbProApp {
    fn default() -> Self {
        let offline_provider = OfflineAgentProvider;
        let offline_info = offline_provider.info();
        Self {
            theme: DbProTheme::default(),
            dark_mode: false,
            reduce_motion: false,
            settings: AppSettings::default(),
            settings_section: SettingsSection::General,
            keybindings_filter: String::new(),
            keybinding_edit_id: None,
            keybinding_edit_draft: String::new(),
            workspace: WorkspaceShellState::default(),
            prediction_mode: PredictionMode::default(),
            welcome_prompt: String::new(),
            query_session_state: QuerySessionState {
                documents: vec![QueryDocument::new("query-1", "Query 1", DEFAULT_QUERY)],
                ..Default::default()
            },
            editor_search: String::new(),
            editor_search_open: false,
            query_editor_focused: false,
            query_focus_editor_on_open: true,
            query_cursor_line: 1,
            query_cursor_column: 1,
            editor_font_size: 14.0,
            query_tools_open: false,
            query_context_picker_open: false,
            query_params_panel_open: false,
            query_output_dock_maximized: false,
            query_editor_rect: egui::Rect::NOTHING,
            completion_open: false,
            snippets_open: false,
            visual_query_builder_open: false,
            visual_query_model: crate::query::visual_builder::VisualQueryModel::default(),
            visual_query_sql_preview: String::new(),
            visual_query_error: None,
            visual_query_add_table: String::new(),
            visual_query_join_table: String::new(),
            visual_query_join_left: String::new(),
            visual_query_join_right: String::new(),
            visual_query_col_ref: String::new(),
            visual_query_col_alias: String::new(),
            visual_query_col_agg: String::new(),
            visual_query_where_left: String::new(),
            visual_query_where_op: "=".into(),
            visual_query_where_value: String::new(),
            visual_query_order: String::new(),
            visual_query_order_desc: false,
            visual_query_limit: "100".into(),
            visual_query_offset: String::new(),
            diagnostics: Vec::new(),
            diagnostics_cache_key: None,
            diagnostics_cache_driver: String::new(),
            diagnostics_lint_structured: Vec::new(),
            diagnostics_debounce_key: None,
            diagnostics_debounce_at: None,
            diagnostics_exec_fp: None,
            param_count_cache_key: None,
            param_count_cache: 0,
            problems_severity_filter: ProblemsSeverityFilter::All,
            problems_source_filter: ProblemsSourceFilter::All,
            problems_selected: None,
            query_history: Vec::new(),
            query_history_entries: Vec::new(),
            query_history_search: String::new(),
            connection_name: "Local PostgreSQL".to_owned(),
            connected: false,
            palette_mode: None,
            palette_query: String::new(),
            palette_scope: SearchScope::All,
            palette_selected: 0,
            palette_focus_requested: false,
            search_index: SearchIndex::default(),
            agent_pending_prompt: None,
            agent_pending_context: None,
            agent_provider_label: offline_info.label.to_owned(),
            agent_provider_detail: offline_info.detail.to_owned(),
            agent_input: String::new(),
            agent_messages: Vec::new(),
            agent_sessions: HashMap::new(),
            agent_auto_run_read_only: false,
            agent_settings_open: false,
            agent_api_key_draft: String::new(),
            agent_api_key_show_password: false,
            agent_configure_request: None,
            task_bridge: TaskBridge::default(),
            runtime_message: "Ready".to_owned(),
            toasts: crate::components::overlay::ToastManager::default(),
            query_output_state: QueryOutputState::default(),
            table_data: TableDataState::default(),
            copy_status: String::new(),
            export_open: false,
            export_format: "CSV".to_owned(),
            export_path: String::new(),
            export_overwrite_pending: false,
            connection_catalog: ConnectionCatalogState::default(),
            saved_queries: Vec::new(),
            query_folders: Vec::new(),
            schema: UiSchemaSummary::default(),
            schema_symbol_index: SchemaSymbolIndex::default(),
            selected_schema: None,
            explorer_search: String::new(),
            explorer_nav_cache: None,
            schema_error: None,
            schema_request: None,
            selected_table: None,
            pinned_tables: Vec::new(),
            recent_tables: Vec::new(),
            ide_workspace: ide_workspace::IdeWorkspaceState::default(),
            git_status: None,
            git_diff: None,
            git_commit_message: String::new(),
            git_last_error: None,
            workspace_file_mtimes: std::collections::HashMap::new(),
            workspace_external_change: None,
            workspace_search_query: String::new(),
            workspace_replace_query: String::new(),
            workspace_search_hits: Vec::new(),
            workspace_replace_previews: Vec::new(),
            workspace_task_command: String::new(),
            workspace_refactor_from: String::new(),
            workspace_refactor_to: String::new(),
            workspace_context_items: Vec::new(),
            selected_schema_object: None,
            schema_object_view: SchemaObjectView::Definition,
            routine_source_draft: String::new(),
            routine_param_values: Vec::new(),
            routine_param_nulls: Vec::new(),
            routine_ddl_preview: None,
            routine_drop_confirm: false,
            transfer_jobs: Vec::new(),
            synthetic_table: String::new(),
            synthetic_row_count: "10".into(),
            synthetic_seed: "42".into(),
            synthetic_null_pct: "0".into(),
            synthetic_preview: None,
            synthetic_error: None,
            synthetic_production_confirm: false,
            masking_columns_csv: "email,phone".into(),
            masking_rule: db_pro_core::domain::masking::MaskRule::PartialReveal,
            masking_keyed: true,
            masking_preview: None,
            masking_error: None,
            monitoring_snapshot: None,
            monitoring_error: None,
            monitoring_poll: true,
            monitoring_last_poll: None,
            monitoring_terminate_confirm: None,
            monitoring_filter_active_only: true,
            monitoring_maintenance_confirm: None,
            monitoring_stat_sort: db_pro_core::domain::monitoring::StatStatementSort::TotalTime,
            monitoring_reset_stats_confirm: false,
            monitoring_workload_prev: None,
            monitoring_workload_filter: String::new(),
            audit_page: None,
            audit_error: None,
            audit_filter_text: String::new(),
            audit_filter_database: String::new(),
            audit_filter_username: String::new(),
            audit_filter_severity: String::new(),
            audit_bookmarks: std::collections::HashSet::new(),
            audit_selected: std::collections::HashSet::new(),
            audit_export_preview: None,
            pg_settings: None,
            pg_settings_filter: String::new(),
            pg_settings_edit_name: String::new(),
            pg_settings_edit_value: String::new(),
            pg_settings_preview: None,
            pg_settings_error: None,
            fdw_inventory: None,
            fdw_error: None,
            fdw_create_name: String::new(),
            fdw_create_wrapper: "postgres_fdw".into(),
            fdw_create_host: String::new(),
            fdw_create_dbname: String::new(),
            fdw_create_port: "5432".into(),
            fdw_ddl_preview: None,
            fdw_drop_confirm: None,
            replication_inventory: None,
            replication_error: None,
            replication_create_name: String::new(),
            replication_ddl_preview: None,
            replication_drop_publication: None,
            replication_drop_subscription: None,
            event_trigger_inventory: None,
            event_trigger_error: None,
            event_trigger_create_name: String::new(),
            event_trigger_create_event: "ddl_command_end".into(),
            event_trigger_create_function: String::new(),
            event_trigger_create_tags: String::new(),
            event_trigger_ddl_preview: None,
            event_trigger_drop_confirm: None,
            security_users: Vec::new(),
            security_selected_role: None,
            security_privileges: Vec::new(),
            security_memberships: Vec::new(),
            security_new_role: String::new(),
            security_new_role_login: true,
            security_membership_role: String::new(),
            security_password: String::new(),
            security_grant_kind: db_pro_core::domain::user::PrivilegeObjectKind::Table,
            security_grant_schema: String::new(),
            security_grant_object: String::new(),
            security_grant_privilege: "SELECT".into(),
            security_rls_schema: "public".into(),
            security_rls_table: String::new(),
            security_rls_state: None,
            security_rls_policy_name: String::new(),
            security_rls_command: "SELECT".into(),
            security_rls_roles: String::new(),
            security_rls_using: String::new(),
            security_rls_with_check: String::new(),
            security_rls_preview_sql: String::new(),
            security_rls_confirm_apply: false,
            security_drop_confirm: None,
            security_error: None,
            schema_workbench: schema_workbench::SchemaWorkbenchState::default(),
            schema_snapshot: None,
            schema_diff: None,
            migration_plan: None,
            migration_preview_sql: String::new(),
            migration_confirm_destructive: false,
            pending_explain_analyze: false,
            explain_analyze_confirmed: false,
            explain_show_raw_json: false,
            saved_task_store: db_pro_core::domain::saved_task::SavedTaskStore::new(),
            saved_task_draft: None,
            saved_tasks_dirty: false,
            saved_task_confirm_destructive: false,
            pending_destructive_task_id: None,
            named_session_store: workspace_session::NamedSessionStore::new(),
            session_name_draft: String::new(),
            selected_named_session_id: None,
            last_session_restore_notes: Vec::new(),
            migration_fingerprint_at_preview: String::new(),
            data_diff_target_id: String::new(),
            data_diff_schema: "public".into(),
            data_diff_table: String::new(),
            data_diff_keys: "id".into(),
            data_diff_result: None,
            data_diff_filter: "all".into(),
            query_auto_commit: true,
            query_in_transaction: false,
            query_txn_pending: 0,
            disconnect_txn_guard: false,
            query_txn_bar_open: false,
            diagram_zoom: 1.0,
            diagram_pan: egui::Vec2::ZERO,
            diagram_pan_origin: None,
            diagram_search: String::new(),
            diagram_show_all: false,
            er_design: crate::diagram::design_mode::DesignModeState::default(),
            er_design_new_table: String::new(),
            er_design_new_schema: "public".into(),
            er_design_col_name: String::new(),
            er_design_col_type: "text".into(),
            er_design_fk_name: String::new(),
            er_design_fk_from: String::new(),
            er_design_fk_to: String::new(),
            diagram_neighborhood_depth: 1,
            diagram_graph: ErGraph::default(),
            diagram_spatial_index: ErSpatialIndex::default(),
            // Start from 1 so saturating_add never wraps to 0 and collides with
            // a stale worker result from the initial state.
            diagram_schema_version: 1,
            diagram_layout_worker: crate::diagram::ErLayoutWorker::default(),
            diagram_layout_state: crate::diagram::ErLayoutState::Idle,
            diagram_latest_layout_request: 0,
            pending_destructive_run: None,
            table_state: TableState {
                table_dependency_filter: "all".to_owned(),
                table_constraint_filter: "all".to_owned(),
                ..Default::default()
            },
            table_mutation: TableMutationState::default(),
            query_folder: String::new(),
            backup_output_path: String::new(),
            restore_input_path: String::new(),
            restore_confirmation: false,
            connection_lifecycle: ConnectionLifecycleState::default(),
            connection_dialog: ConnectionDialogState::default(),
            delete_confirmation_id: None,
            folder_delete_confirmation: None,
            connections_pane_height: 160.0,
            schemas_pane_height: 90.0,
            initial_frames_count: 0,
            gallery_state: ComponentGalleryState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct MemoryStorage {
        values: HashMap<String, String>,
    }

    impl eframe::Storage for MemoryStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.values.get(key).cloned()
        }

        fn set_string(&mut self, key: &str, value: String) {
            self.values.insert(key.to_owned(), value);
        }

        fn flush(&mut self) {}
    }

    #[test]
    fn stale_theme_storage_resets_to_light_first_default() {
        let mut storage = MemoryStorage::default();
        storage
            .values
            .insert("dbpro.native.theme-version".to_owned(), "dark-first-v3".to_owned());
        storage
            .values
            .insert("dbpro.native.dark-mode".to_owned(), "true".to_owned());

        let app = DbProApp::with_task_bridge_and_storage(TaskBridge::default(), Some(&storage));

        assert!(!app.dark_mode);
        assert!(!app.theme.dark_mode);
    }

    #[test]
    fn current_theme_storage_restores_user_choice() {
        let mut storage = MemoryStorage::default();
        storage.values.insert(
            "dbpro.native.theme-version".to_owned(),
            THEME_STORAGE_VERSION.to_owned(),
        );
        storage
            .values
            .insert("dbpro.native.dark-mode".to_owned(), "false".to_owned());

        let app = DbProApp::with_task_bridge_and_storage(TaskBridge::default(), Some(&storage));

        assert!(!app.dark_mode);
        assert!(!app.theme.dark_mode);
    }
}
