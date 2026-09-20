use super::*;

impl DbProApp {
    pub(crate) fn execute_palette_action(&mut self, action: PaletteAction, _ctx: &egui::Context) {
        self.palette.mode = None;
        match action {
            PaletteAction::Welcome => self.activate_welcome_tab(),
            PaletteAction::Query => self.workspace.active_tab = WorkspaceTab::Query,
            PaletteAction::History => self.open_palette_activity(Activity::History),
            PaletteAction::Data => self.open_palette_activity(Activity::Data),
            PaletteAction::Files => self.open_palette_activity(Activity::Files),
            PaletteAction::Diagram => self.workspace.active_tab = WorkspaceTab::Diagram,
            PaletteAction::SchemaWorkbench => self.open_schema_workbench(),
            PaletteAction::SchemaCompare => {
                self.workspace.activity = Activity::Compare;
                self.workspace.active_tab = WorkspaceTab::SchemaCompare;
                self.workspace.sidebar_open = true;
            }
            PaletteAction::Transfers => self.open_palette_activity(Activity::Transfers),
            PaletteAction::Monitor => self.open_palette_activity(Activity::Monitor),
            PaletteAction::Settings => self.open_palette_activity(Activity::Settings),
            PaletteAction::Agent => self.workspace.agent_open = true,
            PaletteAction::Problems => self.open_palette_activity(Activity::Problems),
            PaletteAction::Diagnostics => {
                self.open_palette_activity(Activity::Settings);
                self.feedback.runtime_message = "Opened Settings → Diagnostics".to_owned();
            }
            PaletteAction::NewQuery => {
                self.workspace.active_tab = WorkspaceTab::Query;
                self.new_query_document();
                self.feedback.runtime_message = "New query ready".to_owned();
            }
            PaletteAction::NewConnection => self.connection.open_new(),
            PaletteAction::RefreshSchema => self.refresh_schema_palette(),
            PaletteAction::ToggleExplorer => self.workspace.sidebar_open = !self.workspace.sidebar_open,
            PaletteAction::OpenTable(table) => self.open_table_from_palette(table),
            PaletteAction::OpenView(name) => {
                let schema = self.active_schema().to_owned();
                self.open_schema_object(SchemaObjectSelection::View(name.clone()), &schema, &name, "view");
            }
            PaletteAction::OpenFunction {
                name,
                identity_arguments,
            } => {
                let schema = self.active_schema().to_owned();
                self.open_schema_object(
                    SchemaObjectSelection::Function {
                        name: name.clone(),
                        identity_arguments,
                    },
                    &schema,
                    &name,
                    "function",
                );
            }
            PaletteAction::OpenWorkspaceFile(path) => self.open_workspace_sql_file(path),
            PaletteAction::OpenWorkspaceFolder => self.request_open_workspace_folder(),
            PaletteAction::CloseWorkspaceFolder => self
                .workspace
                .files
                .close(&mut self.workspace.shell, &mut self.feedback),
            PaletteAction::OpenSavedQuery(query_id) => self.open_saved_query_from_palette(query_id),
            PaletteAction::OpenHistoryEntry(index) => {
                if let Some(entry) = self.query.editor.query_history_entries.get(index).cloned() {
                    self.open_history_entry(&entry, false);
                } else {
                    self.feedback.runtime_message = "History entry is no longer available".to_owned();
                }
            }
            PaletteAction::InsertColumn(column) => {
                self.workspace.active_tab = WorkspaceTab::Query;
                self.append_to_active_query(&column);
                self.feedback.runtime_message = format!("Inserted column {column}");
            }
            PaletteAction::InsertSnippet(index) => {
                if let Some((_, snippet)) = Self::builtin_sql_snippets().get(index) {
                    self.insert_snippet(snippet);
                }
            }
            PaletteAction::ExplainQuery => self.explain_query(),
            PaletteAction::ExportResults => self.export_results_from_palette(),
            PaletteAction::RunQuery => {
                self.workspace.active_tab = WorkspaceTab::Query;
                self.dispatch_query();
            }
            PaletteAction::FormatSql => {
                self.workspace.active_tab = WorkspaceTab::Query;
                self.format_active_query();
                self.feedback.runtime_message = "SQL formatted".to_owned();
            }
            PaletteAction::SwitchConnection(connection_id) => self.switch_connection_from_palette(connection_id),
            PaletteAction::TogglePinTable(table) => self.toggle_pinned_table(table),
            PaletteAction::ComponentGallery => self.workspace.active_tab = WorkspaceTab::ComponentGallery,
        }
    }

    fn open_palette_activity(&mut self, activity: Activity) {
        self.workspace.activity = activity;
        self.workspace.sidebar_open = true;
    }
}
