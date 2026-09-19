use super::*;
use crate::components::{kbd_badge, Dialog};

impl DbProApp {
    /// Build searchable entries for the given mode (schema objects + actions).
    fn palette_entries(&self, mode: PaletteMode) -> Vec<(SearchKind, PaletteItem)> {
        let mut items: Vec<(SearchKind, PaletteItem)> = match mode {
            PaletteMode::QuickOpen => Self::quick_open_items()
                .into_iter()
                .map(|item| {
                    let kind = if matches!(item.action, PaletteAction::Agent) {
                        SearchKind::Agent
                    } else {
                        SearchKind::Navigation
                    };
                    (kind, item)
                })
                .collect(),
            PaletteMode::Commands => Self::command_items()
                .into_iter()
                .map(|item| {
                    let kind = match item.action {
                        PaletteAction::Agent => SearchKind::Agent,
                        _ => SearchKind::Command,
                    };
                    (kind, item)
                })
                .collect(),
        };
        if mode == PaletteMode::QuickOpen {
            items.extend(self.recent_table_items());
            items.extend(self.workspace_file_items());
            items.extend(self.schema_table_items());
            items.extend(self.schema_view_items());
            items.extend(self.schema_function_items());
            items.extend(self.pinned_table_items());
            items.extend(self.saved_query_items());
            items.extend(self.query_history_items());
            items.extend(self.schema_column_items());
            items.extend(self.snippet_items());
            items.extend(self.agent_action_items());
        }
        if mode == PaletteMode::Commands {
            items.extend(self.connection_items());
            items.extend(self.agent_action_items());
        }
        items
    }

    fn search_fingerprint(&self) -> String {
        let workspace_files = self
            .workspace_files
            .ide_workspace
            .index()
            .into_iter()
            .filter(|e| e.is_sql)
            .count();
        SearchService::build_fingerprint(SearchFingerprintParts {
            connection_id: self.connection_lifecycle.active_connection_id.as_deref(),
            schema: self.active_schema(),
            tables: self.schema_explorer.schema.tables.len(),
            views: self.schema_explorer.schema.views.len(),
            functions: self.schema_explorer.schema.functions.len(),
            columns: self.active_schema_column_names().len(),
            saved_queries: self.query_library.saved_queries.len(),
            history: self.query_editor.query_history_entries.len(),
            connections: self.connection_catalog.connections.len(),
            workspace_files,
        })
    }

    fn ensure_search_index(&mut self, mode: PaletteMode) {
        let fingerprint = format!("{}|{:?}", self.search_fingerprint(), mode);
        if self.palette.search_index.fingerprint() == fingerprint && !self.palette.search_index.is_empty() {
            return;
        }
        let entries = self.palette_entries(mode);
        self.palette.search_index.replace(fingerprint, entries);
    }

    fn quick_open_items() -> Vec<PaletteItem> {
        vec![
            PaletteItem {
                icon: Icon::House,
                title: "Welcome".to_owned(),
                subtitle: "Database workspace home".to_owned(),
                shortcut: None,
                action: PaletteAction::Welcome,
            },
            PaletteItem {
                icon: Icon::FileCode2,
                title: "Query".to_owned(),
                subtitle: "Open the SQL editor".to_owned(),
                shortcut: Some(format!("{}K", Self::primary_modifier_label())),
                action: PaletteAction::Query,
            },
            PaletteItem {
                icon: Icon::History,
                title: "Query history".to_owned(),
                subtitle: "Browse saved and recent queries".to_owned(),
                shortcut: None,
                action: PaletteAction::History,
            },
            PaletteItem {
                icon: Icon::Table2,
                title: "Data".to_owned(),
                subtitle: "Recent and pinned tables".to_owned(),
                shortcut: None,
                action: PaletteAction::Data,
            },
            PaletteItem {
                icon: Icon::FolderOpen,
                title: "Files".to_owned(),
                subtitle: "Workspace folder and project SQL files".to_owned(),
                shortcut: None,
                action: PaletteAction::Files,
            },
            PaletteItem {
                icon: Icon::ArrowRightLeft,
                title: "ER diagram".to_owned(),
                subtitle: "Explore tables and relationships".to_owned(),
                shortcut: None,
                action: PaletteAction::Diagram,
            },
            PaletteItem {
                icon: Icon::Boxes,
                title: "Schema workbench".to_owned(),
                subtitle: "Create and alter schema objects".to_owned(),
                shortcut: None,
                action: PaletteAction::SchemaWorkbench,
            },
            PaletteItem {
                icon: Icon::GitCompare,
                title: "Schema compare".to_owned(),
                subtitle: "Diff two schema snapshots".to_owned(),
                shortcut: None,
                action: PaletteAction::SchemaCompare,
            },
            PaletteItem {
                icon: Icon::Upload,
                title: "Transfers".to_owned(),
                subtitle: "Import, export, and background copy jobs".to_owned(),
                shortcut: None,
                action: PaletteAction::Transfers,
            },
            PaletteItem {
                icon: Icon::Gauge,
                title: "Monitor".to_owned(),
                subtitle: "Connection health and recent statements".to_owned(),
                shortcut: None,
                action: PaletteAction::Monitor,
            },
            PaletteItem {
                icon: Icon::Settings2,
                title: "Settings".to_owned(),
                subtitle: "Connections, backups and restore".to_owned(),
                shortcut: None,
                action: PaletteAction::Settings,
            },
            PaletteItem {
                icon: Icon::Bot,
                title: "Agent".to_owned(),
                subtitle: "Open the database copilot".to_owned(),
                shortcut: None,
                action: PaletteAction::Agent,
            },
            PaletteItem {
                icon: Icon::TriangleAlert,
                title: "Problems".to_owned(),
                subtitle: "Open SQL diagnostics across documents".to_owned(),
                shortcut: None,
                action: PaletteAction::Problems,
            },
            PaletteItem {
                icon: Icon::Activity,
                title: "Diagnostics".to_owned(),
                subtitle: "App version, drivers, and redacted support summary".to_owned(),
                shortcut: None,
                action: PaletteAction::Diagnostics,
            },
        ]
    }

    fn command_items() -> Vec<PaletteItem> {
        vec![
            PaletteItem {
                icon: Icon::Plus,
                title: "New query".to_owned(),
                subtitle: "Create a fresh SQL document".to_owned(),
                shortcut: None,
                action: PaletteAction::NewQuery,
            },
            PaletteItem {
                icon: Icon::Play,
                title: "Run query".to_owned(),
                subtitle: "Execute the current SQL or selection".to_owned(),
                shortcut: Some(format!("{}↵", Self::primary_modifier_label())),
                action: PaletteAction::RunQuery,
            },
            PaletteItem {
                icon: Icon::WandSparkles,
                title: "Format SQL".to_owned(),
                subtitle: "Format the active SQL document".to_owned(),
                shortcut: None,
                action: PaletteAction::FormatSql,
            },
            PaletteItem {
                icon: Icon::Database,
                title: "New connection".to_owned(),
                subtitle: "Add a PostgreSQL or SQLite connection".to_owned(),
                shortcut: None,
                action: PaletteAction::NewConnection,
            },
            PaletteItem {
                icon: Icon::RotateCcw,
                title: "Refresh schema".to_owned(),
                subtitle: "Reload tables, views and relationships".to_owned(),
                shortcut: None,
                action: PaletteAction::RefreshSchema,
            },
            PaletteItem {
                icon: Icon::Pin,
                title: "Pin / unpin selected table".to_owned(),
                subtitle: "Toggle the active table in pinned Quick Open entries".to_owned(),
                shortcut: None,
                action: PaletteAction::TogglePinTable(String::new()),
            },
            PaletteItem {
                icon: Icon::FolderOpen,
                title: "Open Folder…".to_owned(),
                subtitle: "Open a local workspace folder (#261)".to_owned(),
                shortcut: None,
                action: PaletteAction::OpenWorkspaceFolder,
            },
            PaletteItem {
                icon: Icon::Folder,
                title: "Close Workspace".to_owned(),
                subtitle: "Close the active workspace folder".to_owned(),
                shortcut: None,
                action: PaletteAction::CloseWorkspaceFolder,
            },
            PaletteItem {
                icon: Icon::PanelLeft,
                title: "Toggle explorer".to_owned(),
                subtitle: "Show or hide the connection sidebar".to_owned(),
                shortcut: Some(format!("{}B", Self::primary_modifier_label())),
                action: PaletteAction::ToggleExplorer,
            },
            PaletteItem {
                icon: Icon::Bot,
                title: "Open Agent".to_owned(),
                subtitle: "Ask Agent about the active schema".to_owned(),
                shortcut: None,
                action: PaletteAction::Agent,
            },
            PaletteItem {
                icon: Icon::ArrowRightLeft,
                title: "Open ER diagram".to_owned(),
                subtitle: "Show the active schema map".to_owned(),
                shortcut: None,
                action: PaletteAction::Diagram,
            },
            PaletteItem {
                icon: Icon::ChartNoAxesCombined,
                title: "Explain query".to_owned(),
                subtitle: "Inspect a read-only query plan".to_owned(),
                shortcut: None,
                action: PaletteAction::ExplainQuery,
            },
            PaletteItem {
                icon: Icon::Download,
                title: "Export results".to_owned(),
                subtitle: "Open export options for the current result".to_owned(),
                shortcut: None,
                action: PaletteAction::ExportResults,
            },
            PaletteItem {
                icon: Icon::Palette,
                title: "Open Component Gallery".to_owned(),
                subtitle: "Preview DB Pro common UI design system".to_owned(),
                shortcut: None,
                action: PaletteAction::ComponentGallery,
            },
        ]
    }

    fn schema_table_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        self.active_schema_table_names()
            .iter()
            .take(EXPLORER_MAX_TABLES)
            .cloned()
            .map(|table| {
                (
                    SearchKind::Table,
                    PaletteItem {
                        icon: Icon::Table2,
                        title: table.clone(),
                        subtitle: format!("Open table in {}", self.active_schema()),
                        shortcut: None,
                        action: PaletteAction::OpenTable(table),
                    },
                )
            })
            .collect()
    }

    fn schema_view_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        let schema = self.active_schema().to_owned();
        self.schema_explorer
            .schema
            .views
            .iter()
            .filter(|view| view.schema == schema)
            .take(EXPLORER_MAX_TABLES)
            .map(|view| {
                (
                    SearchKind::View,
                    PaletteItem {
                        icon: Icon::Eye,
                        title: view.name.clone(),
                        subtitle: format!("Open view in {schema}"),
                        shortcut: None,
                        action: PaletteAction::OpenView(view.name.clone()),
                    },
                )
            })
            .collect()
    }

    fn schema_function_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        let schema = self.active_schema().to_owned();
        self.schema_explorer
            .schema
            .functions
            .iter()
            .filter(|function| function.schema == schema)
            .take(EXPLORER_MAX_TABLES)
            .map(|function| {
                (
                    SearchKind::Function,
                    PaletteItem {
                        icon: Icon::Code2,
                        title: function.name.clone(),
                        subtitle: if function.identity_arguments.is_empty() {
                            format!("Open function in {schema}")
                        } else {
                            format!("Open function {}({})", schema, function.identity_arguments)
                        },
                        shortcut: None,
                        action: PaletteAction::OpenFunction {
                            name: function.name.clone(),
                            identity_arguments: function.identity_arguments.clone(),
                        },
                    },
                )
            })
            .collect()
    }

    fn pinned_table_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        self.schema_explorer
            .pinned_tables
            .iter()
            .cloned()
            .map(|table| {
                (
                    SearchKind::Table,
                    PaletteItem {
                        icon: Icon::Pin,
                        title: table.clone(),
                        subtitle: "Pinned table · open".to_owned(),
                        shortcut: None,
                        action: PaletteAction::OpenTable(table),
                    },
                )
            })
            .collect()
    }

    fn recent_table_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        self.schema_explorer
            .recent_tables
            .iter()
            .cloned()
            .map(|table| {
                (
                    SearchKind::Table,
                    PaletteItem {
                        icon: Icon::History,
                        title: table.clone(),
                        subtitle: "Recent table · open".to_owned(),
                        shortcut: None,
                        action: PaletteAction::OpenTable(table),
                    },
                )
            })
            .collect()
    }

    fn workspace_file_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        self.workspace_files
            .ide_workspace
            .index()
            .into_iter()
            .filter(|entry| entry.is_sql)
            .take(80)
            .map(|entry| {
                (
                    SearchKind::WorkspaceFile,
                    PaletteItem {
                        icon: Icon::FileCode2,
                        title: entry.relative_path.clone(),
                        subtitle: format!("Workspace SQL · {}", entry.root_id),
                        shortcut: None,
                        action: PaletteAction::OpenWorkspaceFile(format!("{}::{}", entry.root_id, entry.relative_path)),
                    },
                )
            })
            .collect()
    }

    fn connection_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        self.connection_catalog
            .connections
            .iter()
            .cloned()
            .map(|connection| {
                (
                    SearchKind::Connection,
                    PaletteItem {
                        icon: Icon::Database,
                        title: format!("Switch to {}", connection.name),
                        subtitle: format!("{} · {}", connection.driver, connection.database),
                        shortcut: None,
                        action: PaletteAction::SwitchConnection(connection.id),
                    },
                )
            })
            .collect()
    }

    fn saved_query_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        self.query_library
            .saved_queries
            .iter()
            .take(40)
            .map(|query| {
                (
                    SearchKind::SavedQuery,
                    PaletteItem {
                        icon: Icon::Bookmark,
                        title: query.name.clone(),
                        subtitle: query
                            .folder
                            .clone()
                            .map(|folder| format!("Saved query · {folder}"))
                            .unwrap_or_else(|| "Saved query".to_owned()),
                        shortcut: None,
                        action: PaletteAction::OpenSavedQuery(query.id.clone()),
                    },
                )
            })
            .collect()
    }

    fn schema_column_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        let mut seen = std::collections::BTreeSet::new();
        self.active_schema_column_names()
            .into_iter()
            .filter(|column| seen.insert(column.clone()))
            .take(60)
            .map(|column| {
                (
                    SearchKind::Column,
                    PaletteItem {
                        icon: Icon::Columns3,
                        title: column.clone(),
                        subtitle: format!("Insert column · {}", self.active_schema()),
                        shortcut: None,
                        action: PaletteAction::InsertColumn(column),
                    },
                )
            })
            .collect()
    }

    fn query_history_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        self.query_editor
            .query_history_entries
            .iter()
            .take(30)
            .enumerate()
            .map(|(index, entry)| {
                let preview = entry.sql.chars().take(72).collect::<String>();
                let status = match entry.status {
                    UiQueryHistoryStatus::Success => "success",
                    UiQueryHistoryStatus::Failed => "failed",
                    UiQueryHistoryStatus::Cancelled => "cancelled",
                };
                (
                    SearchKind::History,
                    PaletteItem {
                        icon: Icon::History,
                        title: if preview.len() < entry.sql.len() {
                            format!("{preview}…")
                        } else {
                            preview
                        },
                        subtitle: format!(
                            "History · {} · {status}",
                            entry.connection_id.clone().unwrap_or_else(|| "any".to_owned())
                        ),
                        shortcut: None,
                        action: PaletteAction::OpenHistoryEntry(index),
                    },
                )
            })
            .collect()
    }

    fn snippet_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        DbProApp::builtin_sql_snippets()
            .iter()
            .enumerate()
            .map(|(index, (label, _))| {
                (
                    SearchKind::Command,
                    PaletteItem {
                        icon: Icon::FileCode2,
                        title: (*label).to_owned(),
                        subtitle: "Insert SQL snippet at cursor".to_owned(),
                        shortcut: None,
                        action: PaletteAction::InsertSnippet(index),
                    },
                )
            })
            .collect()
    }

    fn agent_action_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        vec![
            (
                SearchKind::Agent,
                PaletteItem {
                    icon: Icon::Bot,
                    title: "Ask Agent".to_owned(),
                    subtitle: "Open the database copilot".to_owned(),
                    shortcut: None,
                    action: PaletteAction::Agent,
                },
            ),
            (
                SearchKind::Agent,
                PaletteItem {
                    icon: Icon::Sparkles,
                    title: "Explain current query".to_owned(),
                    subtitle: "Agent action · explain plan".to_owned(),
                    shortcut: None,
                    action: PaletteAction::ExplainQuery,
                },
            ),
        ]
    }

    pub(crate) fn filtered_palette_items(&self, mode: PaletteMode) -> Vec<PaletteItem> {
        let entries = if self.palette.search_index.fingerprint().contains(&format!("{mode:?}"))
            && !self.palette.search_index.is_empty()
        {
            self.palette.search_index.entries().to_vec()
        } else {
            self.palette_entries(mode)
        };
        SearchService::filter_rank(&entries, &self.palette.query, self.palette.scope, 120)
    }

    /// Rebuild + rank for mutable callers (palette draw path).
    pub(crate) fn filtered_palette_items_fresh(&mut self, mode: PaletteMode) -> Vec<PaletteItem> {
        self.ensure_search_index(mode);
        self.filtered_palette_items(mode)
    }

    pub(crate) fn execute_palette_action(&mut self, action: PaletteAction, _ctx: &egui::Context) {
        self.palette.mode = None;
        match action {
            PaletteAction::Welcome => self.activate_welcome_tab(),
            PaletteAction::Query => {
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            PaletteAction::History => {
                self.workspace.activity = Activity::History;
                self.workspace.sidebar_open = true;
            }
            PaletteAction::Data => {
                self.workspace.activity = Activity::Data;
                self.workspace.sidebar_open = true;
            }
            PaletteAction::Files => {
                self.workspace.activity = Activity::Files;
                self.workspace.sidebar_open = true;
            }
            PaletteAction::Diagram => self.workspace.active_tab = WorkspaceTab::Diagram,
            PaletteAction::SchemaWorkbench => self.open_schema_workbench(),
            PaletteAction::SchemaCompare => {
                self.workspace.activity = Activity::Compare;
                self.workspace.active_tab = WorkspaceTab::SchemaCompare;
                self.workspace.sidebar_open = true;
            }
            PaletteAction::Transfers => {
                self.workspace.activity = Activity::Transfers;
                self.workspace.sidebar_open = true;
            }
            PaletteAction::Monitor => {
                self.workspace.activity = Activity::Monitor;
                self.workspace.sidebar_open = true;
            }
            PaletteAction::Settings => {
                self.workspace.activity = Activity::Settings;
                self.workspace.sidebar_open = true;
            }
            PaletteAction::Agent => {
                self.workspace.agent_open = true;
            }
            PaletteAction::Problems => {
                self.workspace.activity = Activity::Problems;
                self.workspace.sidebar_open = true;
            }
            PaletteAction::Diagnostics => {
                self.workspace.activity = Activity::Settings;
                self.workspace.sidebar_open = true;
                self.feedback.runtime_message = "Opened Settings → Diagnostics".to_owned();
            }
            PaletteAction::NewQuery => {
                self.workspace.active_tab = WorkspaceTab::Query;
                self.new_query_document();
                self.feedback.runtime_message = "New query ready".to_owned();
            }
            PaletteAction::NewConnection => {
                self.open_new_connection();
            }
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
            PaletteAction::CloseWorkspaceFolder => self.close_workspace_folder(),
            PaletteAction::OpenSavedQuery(query_id) => self.open_saved_query_from_palette(query_id),
            PaletteAction::OpenHistoryEntry(index) => {
                if let Some(entry) = self.query_editor.query_history_entries.get(index).cloned() {
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
            PaletteAction::SwitchConnection(connection_id) => {
                self.switch_connection_from_palette(connection_id);
            }
            PaletteAction::TogglePinTable(table) => {
                self.toggle_pinned_table(table);
            }
            PaletteAction::ComponentGallery => {
                self.workspace.active_tab = WorkspaceTab::ComponentGallery;
            }
        }
    }

    fn refresh_schema_palette(&mut self) {
        if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
            self.table_state.refresh_table_info_after_schema = self.schema_explorer.selected_table.is_some();
            self.request_schema_introspection(connection_id, true);
        } else {
            self.feedback.runtime_message = "Connect to a database before refreshing schema".to_owned();
        }
    }

    pub(crate) fn open_table_from_palette(&mut self, table: String) {
        if self.schema_explorer.selected_table.as_deref() != Some(table.as_str())
            && !self.table_mutation.staged_changes.is_empty()
        {
            self.feedback.runtime_message = "Apply or discard staged changes before opening another table".to_owned();
            return;
        }
        self.persist_current_grid_layout();
        self.record_recent_table(&table);
        self.schema_explorer.selected_table = Some(table.clone());
        self.restore_grid_layout_for_active_table();
        self.schema_explorer.selected_schema_object = None;
        self.table_state.table_view = TableView::Structure;
        self.table_state.table_info = None;
        self.table_state.table_ddl = None;
        self.table_state.table_data_result = None;
        self.request_table_info();
        self.workspace.active_tab = WorkspaceTab::Table;
        self.feedback.runtime_message = format!("Opening table {table}");
    }

    fn open_saved_query_from_palette(&mut self, query_id: String) {
        let Some(query) = self
            .query_library
            .saved_queries
            .iter()
            .find(|item| item.id == query_id)
            .cloned()
        else {
            self.feedback.runtime_message = "Saved query is no longer available".to_owned();
            return;
        };
        self.new_query_document();
        if let Some(doc) = self
            .query_session_state
            .documents
            .get_mut(self.query_session_state.active_document_index)
        {
            doc.set_text(query.sql.clone());
            doc.title = query.name.clone();
            doc.saved_query_id = Some(query.id.clone());
            doc.mark_saved();
        }
        self.workspace.active_tab = WorkspaceTab::Query;
        self.feedback.runtime_message = format!("Opened saved query {}", query.name);
    }

    pub(crate) fn toggle_pinned_table(&mut self, table: String) {
        let target = if table.is_empty() {
            self.schema_explorer.selected_table.clone()
        } else {
            Some(table)
        };
        let Some(table) = target else {
            self.feedback.runtime_message = "Select a table before pinning".to_owned();
            return;
        };
        if let Some(index) = self
            .schema_explorer
            .pinned_tables
            .iter()
            .position(|item| item == &table)
        {
            self.schema_explorer.pinned_tables.remove(index);
            self.feedback.runtime_message = format!("Unpinned table {table}");
        } else {
            self.schema_explorer.pinned_tables.push(table.clone());
            self.feedback.runtime_message = format!("Pinned table {table}");
        }
    }

    fn export_results_from_palette(&mut self) {
        if self.active_query_result().is_some() {
            self.query_output_state.active_tab = OutputTab::Results;
            self.overlay.export_open = true;
            self.workspace.active_tab = WorkspaceTab::Query;
        } else {
            self.feedback.runtime_message = "Run a query before exporting results".to_owned();
        }
    }

    fn switch_connection_from_palette(&mut self, connection_id: String) {
        if let Some(connection) = self
            .connection_catalog
            .connections
            .iter()
            .find(|item| item.id == connection_id)
            .cloned()
        {
            self.connection_lifecycle.active_connection_id = Some(connection.id.clone());
            self.connection_lifecycle.connected = false;
            let request_id = self.task_bridge.next_request_id();
            self.connection_lifecycle.pending_request = Some(request_id);
            self.dispatch_command(UiCommand::Connect {
                request_id,
                connection_id: connection.id,
            });
            self.feedback.runtime_message = format!("Connecting to {}…", connection.name);
        }
    }

    pub(super) fn draw_palette(&mut self, ctx: &egui::Context) {
        let Some(mode) = self.palette.mode else {
            return;
        };
        let items = self.filtered_palette_items_fresh(mode);
        self.clamp_palette_selection(&items);
        let mut activate = false;
        let mut open = true;
        let title = if mode == PaletteMode::QuickOpen {
            "Quick Open"
        } else if self.palette.scope == SearchScope::Connections {
            "Switch Connection"
        } else {
            "Command Palette"
        };
        let description = if mode == PaletteMode::QuickOpen {
            "Switch workspaces, tabs, or open editors"
        } else if self.palette.scope == SearchScope::Connections {
            "Pick a saved connection to open"
        } else {
            "Search commands, actions, and database tools"
        };

        Dialog::new(&mut open, title, self.theme)
            .description(description)
            .width(580.0)
            .id_salt("palette_dialog")
            .show_ctx(ctx, |ui| {
                let response = ui.add(
                    TextEdit::singleline(&mut self.palette.query)
                        .hint_text(RichText::new("Type a command or search…").color(self.theme.text_muted))
                        .desired_width(ui.available_width())
                        .margin(egui::Margin::symmetric(12.0, 8.0))
                        .font(egui::FontId::proportional(13.5))
                        .text_color(self.theme.text_primary),
                );
                if self.palette.focus_requested {
                    response.request_focus();
                    self.palette.focus_requested = false;
                }

                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    for scope in SearchScope::all() {
                        let selected = self.palette.scope == *scope;
                        if ui.selectable_label(selected, scope.label()).clicked() {
                            self.palette.scope = *scope;
                            self.palette.selected = 0;
                        }
                    }
                });

                if ui.input(|input| input.key_pressed(egui::Key::ArrowDown)) && !items.is_empty() {
                    self.palette.selected = (self.palette.selected + 1) % items.len();
                }
                if ui.input(|input| input.key_pressed(egui::Key::ArrowUp)) && !items.is_empty() {
                    self.palette.selected = if self.palette.selected == 0 {
                        items.len() - 1
                    } else {
                        self.palette.selected - 1
                    };
                }
                if ui.input(|input| input.key_pressed(egui::Key::Enter)) && !items.is_empty() {
                    activate = true;
                }

                ui.add_space(8.0);
                egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                    if items.is_empty() {
                        ui.add_space(16.0);
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("No matching commands found").color(self.theme.text_muted));
                        });
                        ui.add_space(16.0);
                    }
                    for (index, item) in items.iter().enumerate() {
                        let selected = index == self.palette.selected;
                        let item_fill = if selected {
                            self.theme.surface_hover
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        let (rect, item_resp) =
                            ui.allocate_exact_size(egui::vec2(ui.available_width(), 44.0), egui::Sense::click());
                        if item_resp.hovered() {
                            self.palette.selected = index;
                        }
                        if item_resp.clicked() {
                            self.palette.selected = index;
                            activate = true;
                        }

                        if selected || item_resp.hovered() {
                            ui.painter().rect_filled(rect, egui::Rounding::same(6.0), item_fill);
                            if selected {
                                ui.painter().rect_stroke(
                                    rect,
                                    egui::Rounding::same(6.0),
                                    egui::Stroke::new(1.0, self.theme.border_subtle),
                                );
                            }
                        }

                        // Icon
                        let icon_char = char::from(item.icon).to_string();
                        ui.painter().text(
                            egui::pos2(rect.left() + 12.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            icon_char,
                            egui::FontId::new(15.0, egui::FontFamily::Name("lucide".into())),
                            if selected {
                                self.theme.text_primary
                            } else {
                                self.theme.text_secondary
                            },
                        );

                        // Title and Subtitle
                        let text_x = rect.left() + 38.0;
                        ui.painter().text(
                            egui::pos2(text_x, rect.center().y - 8.0),
                            egui::Align2::LEFT_CENTER,
                            &item.title,
                            crate::DbProTheme::ui_medium_font(13.0),
                            self.theme.text_primary,
                        );
                        ui.painter().text(
                            egui::pos2(text_x, rect.center().y + 8.0),
                            egui::Align2::LEFT_CENTER,
                            &item.subtitle,
                            egui::FontId::proportional(11.5),
                            self.theme.text_muted,
                        );

                        if let Some(shortcut) = &item.shortcut {
                            ui.allocate_new_ui(
                                egui::UiBuilder::new().max_rect(egui::Rect::from_min_max(
                                    egui::pos2(rect.right() - 80.0, rect.top()),
                                    rect.right_bottom(),
                                )),
                                |ui| {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(8.0);
                                        kbd_badge(ui, shortcut, self.theme);
                                    });
                                },
                            );
                        }
                    }
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("↑↓ Navigate · ↵ Select · Esc Close")
                            .size(11.0)
                            .color(self.theme.text_muted),
                    );
                });
            });

        if !open {
            self.palette.mode = None;
        }

        if activate {
            if let Some(item) = items.get(self.palette.selected) {
                self.execute_palette_action(item.action.clone(), ctx);
            }
        }
    }

    fn clamp_palette_selection(&mut self, items: &[PaletteItem]) {
        if items.is_empty() {
            self.palette.selected = 0;
        } else {
            self.palette.selected = self.palette.selected.min(items.len() - 1);
        }
    }
}
