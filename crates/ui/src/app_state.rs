use super::*;

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
            app.load_saved_tasks_from_storage(storage);
            app.load_named_sessions_from_storage(storage);
            {
                let mut storage_context =
                    app_storage::NativeStorageContext::new(app_storage::NativeStorageDependencies {
                        preferences: &mut app.preferences,
                        agent: &mut app.agent,
                        theme: &mut app.theme,
                        workspace: &mut app.workspace,
                        query: &mut app.query,
                        table: &mut app.table,
                        connection: &mut app.connection,
                        schema_explorer: &mut app.schema_explorer,
                    });
                storage_context.restore_preferences(storage);
                storage_context.restore_connection_profiles(storage);
                storage_context.restore_shell_layout(storage);
                storage_context.restore_table_layout(storage);
                storage_context.restore_query_state(storage);
            }
            // Shell/layout restore after documents + pins so tab refs resolve (#222).
            app.restore_last_workspace_session_from_storage(storage);
            app_storage::NativeStorageContext::new(app_storage::NativeStorageDependencies {
                preferences: &mut app.preferences,
                agent: &mut app.agent,
                theme: &mut app.theme,
                workspace: &mut app.workspace,
                query: &mut app.query,
                table: &mut app.table,
                connection: &mut app.connection,
                schema_explorer: &mut app.schema_explorer,
            })
            .restore_workspace_files(storage);
        }
        app
    }
}

impl Default for DbProApp {
    fn default() -> Self {
        Self {
            theme: DbProTheme::default(),
            preferences: PreferencesState::default(),
            workspace: WorkspaceFeatureState::default(),
            welcome: WelcomeState::default(),
            query: QueryFeatureState {
                session: QuerySessionState {
                    documents: vec![QueryDocument::new("query-1", "Query 1", DEFAULT_QUERY)],
                    ..Default::default()
                },
                ..Default::default()
            },
            palette: PaletteState::default(),
            agent: AgentState::default(),
            task_bridge: TaskBridge::default(),
            feedback: FeedbackState {
                runtime_message: "Ready".to_owned(),
                ..Default::default()
            },
            table: TableEditorState {
                data: TableDataState::default(),
                data_query: TableDataQueryState::default(),
                editing: TableEditingState::default(),
                mutation: TableMutationState::default(),
                state: TableState {
                    table_dependency_filter: "all".to_owned(),
                    table_constraint_filter: "all".to_owned(),
                    ..Default::default()
                },
            },
            overlay: OverlayState::default(),
            connection: ConnectionFeatureState::default(),
            schema_explorer: SchemaExplorerState::default(),
            audit: AuditState::default(),
            event_trigger: EventTriggerState::default(),
            fdw: FdwState::default(),
            masking: MaskingState::default(),
            monitoring: MonitoringState::default(),
            pg_settings: PgSettingsState::default(),
            replication: ReplicationState::default(),
            routine: RoutineState::default(),
            security: SecurityState::default(),
            schema_workbench: schema_workbench::SchemaWorkbenchState::default(),
            schema_compare: SchemaCompareState::default(),
            synthetic_data: SyntheticDataState::default(),
            saved_tasks: SavedTaskState::default(),
            transfer: TransferState::default(),
            diagram: DiagramState::default(),
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

        assert!(!app.preferences.dark_mode);
        assert!(!app.theme.dark_mode);
    }

    #[test]
    fn current_theme_storage_restores_user_choice() {
        let mut storage = MemoryStorage::default();
        storage.values.insert(
            "dbpro.native.theme-version".to_owned(),
            app_storage::THEME_STORAGE_VERSION.to_owned(),
        );
        storage
            .values
            .insert("dbpro.native.dark-mode".to_owned(), "false".to_owned());

        let app = DbProApp::with_task_bridge_and_storage(TaskBridge::default(), Some(&storage));

        assert!(!app.preferences.dark_mode);
        assert!(!app.theme.dark_mode);
    }
}
