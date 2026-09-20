//! Native storage restore adapters.
//!
//! Persistence keys are an outer-shell concern. This context keeps the
//! adapter explicit about which feature state it hydrates without giving the
//! storage layer a dependency on the whole `DbProApp` composition root.

use super::*;

pub(crate) const THEME_STORAGE_VERSION: &str = "light-first-v1";

pub(crate) struct NativeStorageDependencies<'a> {
    pub(super) preferences: &'a mut PreferencesState,
    pub(super) agent: &'a mut AgentState,
    pub(super) theme: &'a mut DbProTheme,
    pub(super) workspace: &'a mut WorkspaceFeatureState,
    pub(super) query: &'a mut QueryFeatureState,
    pub(super) table: &'a mut TableEditorState,
    pub(super) connection: &'a mut ConnectionFeatureState,
    pub(super) schema_explorer: &'a mut SchemaExplorerState,
}

pub(crate) struct NativeStorageContext<'a> {
    preferences: &'a mut PreferencesState,
    agent: &'a mut AgentState,
    theme: &'a mut DbProTheme,
    workspace: &'a mut WorkspaceFeatureState,
    query: &'a mut QueryFeatureState,
    table: &'a mut TableEditorState,
    connection: &'a mut ConnectionFeatureState,
    schema_explorer: &'a mut SchemaExplorerState,
}

impl<'a> NativeStorageContext<'a> {
    pub(crate) fn new(dependencies: NativeStorageDependencies<'a>) -> Self {
        Self {
            preferences: dependencies.preferences,
            agent: dependencies.agent,
            theme: dependencies.theme,
            workspace: dependencies.workspace,
            query: dependencies.query,
            table: dependencies.table,
            connection: dependencies.connection,
            schema_explorer: dependencies.schema_explorer,
        }
    }

    pub(crate) fn restore_preferences(&mut self, storage: &dyn eframe::Storage) {
        self.preferences.dark_mode =
            if storage.get_string("dbpro.native.theme-version").as_deref() == Some(THEME_STORAGE_VERSION) {
                storage
                    .get_string("dbpro.native.dark-mode")
                    .map(|value| value == "true")
                    .unwrap_or(false)
            } else {
                false
            };
        self.preferences.reduce_motion = storage
            .get_string("dbpro.native.reduce-motion")
            .is_some_and(|value| value == "true");
        if let Some(mode) = storage
            .get_string("dbpro.native.prediction-mode")
            .and_then(|value| serde_json::from_str::<PredictionMode>(&value).ok())
        {
            self.preferences.prediction_mode = mode;
        }
        if let Some(raw) = storage.get_string(SETTINGS_STORAGE_KEY) {
            if let Some(settings) = AppSettings::from_json(&raw) {
                self.preferences.settings = settings;
                settings_view::apply_settings_state(self.preferences, self.query, self.agent, self.theme);
            }
        } else {
            settings_view::sync_settings_state(self.preferences, self.query, self.agent);
        }
        *self.theme = if self.preferences.dark_mode {
            DbProTheme::dark()
        } else {
            DbProTheme::light()
        };
    }

    pub(crate) fn restore_connection_profiles(&mut self, storage: &dyn eframe::Storage) {
        let Some(raw) = storage.get_string("dbpro.native.ssh-profiles-v1") else {
            return;
        };
        if let Ok(profiles) = serde_json::from_str(&raw) {
            self.connection.dialog.set_ssh_profiles(profiles);
        }
    }

    pub(crate) fn restore_shell_layout(&mut self, storage: &dyn eframe::Storage) {
        if let Some(width) = storage
            .get_string("dbpro.native.sidebar-width")
            .and_then(|value| value.parse::<f32>().ok())
        {
            self.workspace.set_sidebar_width(width);
        }
        if let Some(width) = storage
            .get_string("dbpro.native.agent-width")
            .and_then(|value| value.parse::<f32>().ok())
        {
            self.workspace.set_agent_width(width);
        }
        self.workspace.bottom_panel_open = storage
            .get_string("dbpro.native.output-open")
            .is_some_and(|value| value == "true");
        if let Some(height) = storage
            .get_string("dbpro.native.output-height")
            .and_then(|value| value.parse::<f32>().ok())
        {
            self.workspace.set_bottom_panel_height(height);
        }
        if let Some(height) = storage
            .get_string("dbpro.native.connections-pane-height")
            .and_then(|value| value.parse::<f32>().ok())
        {
            self.schema_explorer.connections_pane_height = height.clamp(80.0, 400.0);
        }
        if let Some(height) = storage
            .get_string("dbpro.native.schemas-pane-height")
            .and_then(|value| value.parse::<f32>().ok())
        {
            self.schema_explorer.schemas_pane_height = height.clamp(60.0, 200.0);
        }
    }

    pub(crate) fn restore_table_layout(&mut self, storage: &dyn eframe::Storage) {
        if let Some(widths) = storage.get_string("dbpro.native.grid-widths") {
            if let Ok(widths) = serde_json::from_str::<Vec<f32>>(&widths) {
                self.table.data.grid_column_widths = widths.into_iter().map(|width| width.clamp(90.0, 520.0)).collect();
            }
        }
        if let Some(layouts) = storage.get_string("dbpro.native.grid-layouts") {
            if let Ok(layouts) = serde_json::from_str(&layouts) {
                self.table.data.grid_layout_preferences = layouts;
            }
        }
        self.table.data.grid_columns_user_resized = storage
            .get_string("dbpro.native.grid-widths-customized")
            .is_some_and(|value| value == "true");
    }

    pub(crate) fn restore_query_state(&mut self, storage: &dyn eframe::Storage) {
        if let Some(documents) = storage.get_string("dbpro.native.query-documents") {
            if let Ok(documents) = serde_json::from_str::<Vec<QueryDocument>>(&documents) {
                if !documents.is_empty() {
                    self.query.session.documents = documents;
                }
            }
        }
        if let Some(history) = storage.get_string("dbpro.native.query-history-v1") {
            if let Ok(history) = serde_json::from_str(&history) {
                self.query.editor.query_history_entries = history;
            }
        }
        if let Some(pinned) = storage.get_string("dbpro.native.pinned-tables-v1") {
            if let Ok(tables) = serde_json::from_str::<Vec<String>>(&pinned) {
                self.schema_explorer.pinned_tables = tables;
            }
        }
    }

    pub(crate) fn restore_workspace_files(&mut self, storage: &dyn eframe::Storage) {
        if let Some(recent) = storage.get_string("dbpro.native.recent-tables-v1") {
            if let Ok(tables) = serde_json::from_str::<Vec<String>>(&recent) {
                self.schema_explorer.recent_tables = tables;
            }
        }
        if let Some(recent_ws) = storage.get_string("dbpro.native.workspace-recent-v1") {
            if let Ok(paths) = serde_json::from_str::<Vec<String>>(&recent_ws) {
                self.workspace.files.ide_workspace.recent_roots =
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
                let _ = if self.workspace.files.ide_workspace.roots.is_empty() {
                    self.workspace.files.ide_workspace.open_root(path)
                } else {
                    self.workspace.files.ide_workspace.add_root(path)
                };
            }
        }
        if storage.get_string("dbpro.native.workspace-trusted-v1").as_deref() == Some("true") {
            self.workspace.files.ide_workspace.set_trusted(true);
        }
    }
}
