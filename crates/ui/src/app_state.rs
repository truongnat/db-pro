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
                    app.query_editor.query_history_entries = history;
                }
            }
            if let Some(pinned) = storage.get_string("dbpro.native.pinned-tables-v1") {
                if let Ok(tables) = serde_json::from_str::<Vec<String>>(&pinned) {
                    app.schema_explorer.pinned_tables = tables;
                }
            }
            // Shell/layout restore after documents + pins so tab refs resolve (#222).
            app.restore_last_workspace_session_from_storage(storage);
            if let Some(recent) = storage.get_string("dbpro.native.recent-tables-v1") {
                if let Ok(tables) = serde_json::from_str::<Vec<String>>(&recent) {
                    app.schema_explorer.recent_tables = tables;
                }
            }
            if let Some(recent_ws) = storage.get_string("dbpro.native.workspace-recent-v1") {
                if let Ok(paths) = serde_json::from_str::<Vec<String>>(&recent_ws) {
                    app.workspace_files.ide_workspace.recent_roots =
                        paths.into_iter().map(std::path::PathBuf::from).collect();
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
                    let _ = if app.workspace_files.ide_workspace.roots.is_empty() {
                        app.workspace_files.ide_workspace.open_root(path)
                    } else {
                        app.workspace_files.ide_workspace.add_root(path)
                    };
                }
            }
            if storage.get_string("dbpro.native.workspace-trusted-v1").as_deref() == Some("true") {
                app.workspace_files.ide_workspace.set_trusted(true);
            }
        }
        app
    }
}

impl Default for DbProApp {
    fn default() -> Self {
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
            query_editor: QueryEditorState::default(),
            connection_name: "Local PostgreSQL".to_owned(),
            connected: false,
            palette: PaletteState::default(),
            agent: AgentState::default(),
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
            schema_explorer: SchemaExplorerState::default(),
            workspace_files: WorkspaceFilesState::default(),
            database_operations: DatabaseOperationsState::default(),
            query_execution: QueryExecutionPolicyState::default(),
            saved_task_store: db_pro_core::domain::saved_task::SavedTaskStore::new(),
            saved_task_draft: None,
            saved_tasks_dirty: false,
            saved_task_confirm_destructive: false,
            pending_destructive_task_id: None,
            named_session_store: workspace_session::NamedSessionStore::new(),
            session_name_draft: String::new(),
            selected_named_session_id: None,
            last_session_restore_notes: Vec::new(),
            diagram: DiagramState::default(),
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
