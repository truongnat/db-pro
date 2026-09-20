use super::*;

pub(super) struct PaletteSearchContext<'a> {
    pub(super) workspace: &'a WorkspaceFeatureState,
    pub(super) schema: &'a SchemaWorkspaceState,
    pub(super) query: &'a QueryFeatureState,
    pub(super) connection: &'a ConnectionFeatureState,
    pub(super) active_schema: &'a str,
    pub(super) table_names: &'a [String],
    pub(super) column_names: &'a [String],
}

impl<'a> PaletteSearchContext<'a> {
    /// Build searchable entries for the given mode (schema objects + actions).
    pub(super) fn entries(&self, mode: PaletteMode) -> Vec<(SearchKind, PaletteItem)> {
        let mut items: Vec<(SearchKind, PaletteItem)> = match mode {
            PaletteMode::QuickOpen => palette_catalog::quick_open_items()
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
            PaletteMode::Commands => palette_catalog::command_items()
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
            items.extend(palette_catalog::agent_action_items());
        }
        if mode == PaletteMode::Commands {
            items.extend(self.connection_items());
            items.extend(palette_catalog::agent_action_items());
        }
        items
    }

    fn schema_table_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        self.table_names
            .iter()
            .take(EXPLORER_MAX_TABLES)
            .cloned()
            .map(|table| {
                (
                    SearchKind::Table,
                    PaletteItem {
                        icon: Icon::Table2,
                        title: table.clone(),
                        subtitle: format!("Open table in {}", self.active_schema),
                        shortcut: None,
                        action: PaletteAction::OpenTable(table),
                    },
                )
            })
            .collect()
    }

    fn schema_view_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        let schema = self.active_schema.to_owned();
        self.schema
            .explorer
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
        let schema = self.active_schema.to_owned();
        self.schema
            .explorer
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
        self.schema
            .explorer
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
        self.schema
            .explorer
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
        self.workspace
            .files
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
        self.connection
            .catalog
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
        self.query
            .library
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
        self.column_names
            .iter()
            .filter(|column| seen.insert((*column).clone()))
            .take(60)
            .map(|column| {
                let column = (*column).clone();
                (
                    SearchKind::Column,
                    PaletteItem {
                        icon: Icon::Columns3,
                        title: column.clone(),
                        subtitle: format!("Insert column · {}", self.active_schema),
                        shortcut: None,
                        action: PaletteAction::InsertColumn(column),
                    },
                )
            })
            .collect()
    }

    fn query_history_items(&self) -> Vec<(SearchKind, PaletteItem)> {
        self.query
            .editor
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
        query_snippets::builtin_sql_snippets()
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
}
