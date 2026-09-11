use super::*;

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
            app.dark_mode = storage
                .get_string("dbpro.native.dark-mode")
                .is_some_and(|value| value == "true");
            app.reduce_motion = storage
                .get_string("dbpro.native.reduce-motion")
                .is_some_and(|value| value == "true");
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
            if let Some(documents) = storage.get_string("dbpro.native.query-documents") {
                if let Ok(documents) = serde_json::from_str::<Vec<QueryDocument>>(&documents) {
                    if !documents.is_empty() {
                        app.query_documents = documents;
                        app.query_text = app.query_documents[0].content.clone();
                    }
                }
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
            active_tab: WorkspaceTab::Welcome,
            sidebar_open: true,
            agent_open: false,
            sidebar_open_before_agent: None,
            query_text: "select\n  id, name, status\nfrom customers\nlimit 100;".to_owned(),
            welcome_prompt: String::new(),
            selected_query: String::new(),
            query_documents: vec![QueryDocument {
                title: "Query 1".to_owned(),
                content: "select\n  id, name, status\nfrom customers\nlimit 100;".to_owned(),
            }],
            active_query_document: 0,
            editor_search: String::new(),
            editor_search_open: false,
            editor_font_size: 14.0,
            completion_open: false,
            snippets_open: false,
            diagnostics: Vec::new(),
            query_history: Vec::new(),
            connection_name: "Local PostgreSQL".to_owned(),
            connected: false,
            palette_mode: None,
            palette_query: String::new(),
            palette_selected: 0,
            palette_focus_requested: false,
            agent_provider: Box::new(offline_provider),
            agent_request: None,
            agent_pending_prompt: None,
            agent_pending_context: None,
            agent_provider_label: offline_info.label.to_owned(),
            agent_provider_detail: offline_info.detail.to_owned(),
            agent_input: String::new(),
            agent_messages: Vec::new(),
            task_bridge: TaskBridge::default(),
            next_query_request: None,
            runtime_message: "Ready".to_owned(),
            query_result: None,
            grid_filter: String::new(),
            grid_sort_column: None,
            grid_sort_desc: false,
            grid_column_widths: Vec::new(),
            grid_resize_start: None,
            selected_cell: None,
            selected_row: None,
            data_editing_cell: None,
            data_edit_value: String::new(),
            data_delete_confirmation: false,
            insert_row_open: false,
            insert_row_values: Vec::new(),
            insert_row_error: String::new(),
            copy_status: String::new(),
            export_open: false,
            export_format: "CSV".to_owned(),
            export_path: String::new(),
            mutation_table: String::new(),
            mutation_column: String::new(),
            mutation_value: String::new(),
            mutation_pk_column: String::new(),
            mutation_pk_value: String::new(),
            mutation_delete_confirmation: false,
            connections: Vec::new(),
            saved_queries: Vec::new(),
            query_folders: Vec::new(),
            schema: UiSchemaSummary {
                tables: Vec::new(),
                columns: Vec::new(),
                table_details: Vec::new(),
                views: Vec::new(),
                triggers: Vec::new(),
                functions: Vec::new(),
            },
            explorer_search: String::new(),
            schema_error: None,
            schema_request: None,
            selected_table: None,
            selected_schema_object: None,
            schema_object_view: SchemaObjectView::Definition,
            diagram_zoom: 1.0,
            diagram_pan: egui::Vec2::ZERO,
            diagram_pan_origin: None,
            diagram_search: String::new(),
            diagram_show_all: false,
            table_info: None,
            table_ddl: None,
            table_info_error: None,
            table_ddl_error: None,
            ddl_execute_confirmation: false,
            ddl_execution_request: None,
            refresh_table_info_after_schema: false,
            table_data_result: None,
            table_data_total_rows: None,
            table_data_offset: 0,
            table_data_filter_column: String::new(),
            table_data_filter_value: String::new(),
            table_data_sort_column: None,
            table_data_sort_desc: false,
            table_data_error: None,
            table_info_request: None,
            table_ddl_request: None,
            table_data_request: None,
            table_mutation_request: None,
            table_view: TableView::Structure,
            query_folder: String::new(),
            backup_output_path: String::new(),
            restore_input_path: String::new(),
            restore_confirmation: false,
            active_connection_id: None,
            pending_connection_request: None,
            connections_requested: false,
            connection_dialog_open: false,
            editing_connection_id: None,
            connection_draft: UiConnectionDraft::default(),
            connection_error: String::new(),
            connection_test_valid: false,
            connection_test_draft: None,
            delete_confirmation_id: None,
            folder_delete_confirmation: None,
        }
    }
}
