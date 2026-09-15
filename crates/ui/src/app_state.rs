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
            if let Some(width) = storage
                .get_string("dbpro.native.sidebar-width")
                .and_then(|value| value.parse::<f32>().ok())
            {
                app.sidebar_width = width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
            }
            if let Some(width) = storage
                .get_string("dbpro.native.agent-width")
                .and_then(|value| value.parse::<f32>().ok())
            {
                app.agent_width = width.clamp(AGENT_MIN_WIDTH, AGENT_MAX_WIDTH);
            }
            app.bottom_panel_open = storage
                .get_string("dbpro.native.output-open")
                .is_some_and(|value| value == "true");
            if let Some(height) = storage
                .get_string("dbpro.native.output-height")
                .and_then(|value| value.parse::<f32>().ok())
            {
                app.bottom_panel_height = height.clamp(OUTPUT_MIN_HEIGHT, OUTPUT_MAX_HEIGHT);
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
                    app.grid_column_widths = widths.into_iter().map(|width| width.clamp(90.0, 520.0)).collect();
                }
            }
            if let Some(layouts) = storage.get_string("dbpro.native.grid-layouts") {
                if let Ok(layouts) = serde_json::from_str(&layouts) {
                    app.grid_layout_preferences = layouts;
                }
            }
            app.grid_columns_user_resized = storage
                .get_string("dbpro.native.grid-widths-customized")
                .is_some_and(|value| value == "true");
            if let Some(documents) = storage.get_string("dbpro.native.query-documents") {
                if let Ok(documents) = serde_json::from_str::<Vec<QueryDocument>>(&documents) {
                    if !documents.is_empty() {
                        app.query_documents = documents;
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
            activity: Activity::Explorer,
            welcome_open: true,
            active_tab: WorkspaceTab::ComponentGallery,
            sidebar_open: true,
            sidebar_width: 260.0,
            agent_open: false,
            agent_width: 360.0,
            bottom_panel_open: false,
            bottom_panel_height: 180.0,
            sidebar_open_before_agent: None,
            prediction_mode: PredictionMode::default(),
            welcome_prompt: String::new(),
            selected_query: String::new(),
            query_documents: vec![QueryDocument::new("query-1", "Query 1", DEFAULT_QUERY)],
            active_query_document: 0,
            editor_search: String::new(),
            editor_search_open: false,
            query_editor_focused: false,
            query_cursor_line: 1,
            query_cursor_column: 1,
            editor_font_size: 14.0,
            query_tools_open: false,
            completion_open: false,
            snippets_open: false,
            diagnostics: Vec::new(),
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
            palette_selected: 0,
            palette_focus_requested: false,
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
            agent_configure_request: None,
            task_bridge: TaskBridge::default(),
            query_document_requests: HashMap::new(),
            query_save_requests: HashMap::new(),
            pending_dirty_close: None,
            pending_close_after_save: None,
            save_as_name: String::new(),
            save_as_open: false,
            runtime_message: "Ready".to_owned(),
            toasts: crate::components::overlay::ToastManager::default(),
            output_tab: OutputTab::Results,
            query_output_tabs: HashMap::new(),
            grid_filter: String::new(),
            grid_sort_column: None,
            grid_sort_desc: false,
            grid_column_widths: Vec::new(),
            grid_column_order: Vec::new(),
            grid_hidden_columns: std::collections::BTreeSet::new(),
            grid_layout_preferences: std::collections::HashMap::new(),
            grid_pending_named_layout: None,
            grid_legacy_layout_pending: false,
            grid_layout_column_names: Vec::new(),
            grid_row_identity_cache: std::collections::HashMap::new(),
            grid_row_identity_cache_ready: false,
            grid_projection_epoch: 0,
            grid_projection_cache: crate::GridProjectionCache::default(),
            grid_selection_cache: GridSelectionCache::default(),
            grid_columns_user_resized: false,
            selected_cell: None,
            selected_row: None,
            selected_rows: std::collections::BTreeSet::new(),
            selection_anchor_row: None,
            selection_anchor_cell: None,
            data_editing_cell: None,
            expanded_data_editor: None,
            data_edit_value: String::new(),
            data_edit_error: None,
            data_delete_confirmation: false,
            discard_changes_confirmation: false,
            pending_navigation_action: None,
            insert_row_open: false,
            insert_row_values: Vec::new(),
            insert_row_error: String::new(),
            copy_status: String::new(),
            export_open: false,
            export_format: "CSV".to_owned(),
            export_path: String::new(),
            export_overwrite_pending: false,
            connections: Vec::new(),
            saved_queries: Vec::new(),
            query_folders: Vec::new(),
            schema: UiSchemaSummary::default(),
            selected_schema: None,
            explorer_search: String::new(),
            schema_error: None,
            schema_request: None,
            selected_table: None,
            pinned_tables: Vec::new(),
            recent_tables: Vec::new(),
            ide_workspace: ide_workspace::IdeWorkspaceState::default(),
            workspace_search_query: String::new(),
            workspace_replace_query: String::new(),
            workspace_search_hits: Vec::new(),
            workspace_replace_previews: Vec::new(),
            workspace_task_command: String::new(),
            workspace_refactor_from: String::new(),
            workspace_refactor_to: String::new(),
            workspace_context_items: Vec::new(),
            split_editor_secondary: None,
            files_panel_tab: FilesPanelTab::Tree,
            selected_schema_object: None,
            schema_object_view: SchemaObjectView::Definition,
            diagram_zoom: 1.0,
            diagram_pan: egui::Vec2::ZERO,
            diagram_pan_origin: None,
            diagram_search: String::new(),
            diagram_show_all: false,
            diagram_neighborhood_depth: 1,
            diagram_graph: ErGraph::default(),
            diagram_spatial_index: ErSpatialIndex::default(),
            // Start from 1 so saturating_add never wraps to 0 and collides with
            // a stale worker result from the initial state.
            diagram_schema_version: 1,
            diagram_layout_worker: crate::diagram::ErLayoutWorker::default(),
            diagram_layout_state: crate::diagram::ErLayoutState::Idle,
            diagram_latest_layout_request: 0,
            table_info: None,
            table_ddl: None,
            table_info_error: None,
            table_ddl_error: None,
            ddl_execute_confirmation: false,
            ddl_execution_request: None,
            pending_destructive_run: None,
            refresh_table_info_after_schema: false,
            table_data_result: None,
            table_data_total_rows: None,
            table_data_offset: 0,
            table_data_limit: TABLE_PAGE_SIZE,
            table_data_filter_column: String::new(),
            table_data_filter_operator: crate::UiTableFilterOperator::default(),
            table_data_filter_value: String::new(),
            table_data_filter_editing: None,
            table_data_filters: Vec::new(),
            table_data_sorts: Vec::new(),
            table_data_error: None,
            table_structure_search: String::new(),
            table_metadata_search: String::new(),
            table_column_detail: None,
            table_index_detail: None,
            table_dependency_filter: "all".to_owned(),
            table_constraint_filter: "all".to_owned(),
            table_info_request: None,
            table_ddl_request: None,
            table_data_request: None,
            table_row_reload_request: None,
            table_row_reload_identity: None,
            table_mutation_request: None,
            staged_changes: ChangeSet::new(),
            pending_changes_open: false,
            staged_apply_request: None,
            staged_apply_targets: Vec::new(),
            table_mutation_retry_after_reload: false,
            table_mutation_retry_target: None,
            table_mutation_error: None,
            conflict_dialog_open: false,
            table_view: TableView::Structure,
            query_folder: String::new(),
            backup_output_path: String::new(),
            restore_input_path: String::new(),
            restore_confirmation: false,
            active_connection_id: None,
            pending_connection_id: None,
            pending_connection_request: None,
            connection_errors: std::collections::HashMap::new(),
            failed_connection_ids: std::collections::HashSet::new(),
            connections_requested: false,
            connections_request_pending: false,
            connection_dialog_open: false,
            editing_connection_id: None,
            connection_draft: UiConnectionDraft::default(),
            connection_show_password: false,
            connection_error: String::new(),
            connection_test_valid: false,
            connection_test_draft: None,
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
