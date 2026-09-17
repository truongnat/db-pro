//! Shared UI enums and small value types for the native shell.
use crate::UiTableSummary;
use lucide_icons::Icon;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PendingNavigationAction {
    OpenTable(String),
    ChangeSchema(String),
    ChangeConnection(String),
    CloseWorkspace(WorkspaceTab),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PersistedGridColumnLayout {
    pub(crate) column_name: String,
    pub(crate) width: f32,
    pub(crate) order: usize,
    pub(crate) hidden: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct PersistedGridLayout {
    /// Stable layout entries. The legacy fields below are read only for
    /// migration and are intentionally not written after the next save.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) columns: Vec<PersistedGridColumnLayout>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) widths: Vec<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) order: Vec<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) hidden_columns: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Activity {
    Explorer,
    Files,
    Queries,
    Data,
    History,
    Transfers,
    Monitor,
    Security,
    Settings,
    Diagram,
    Schema,
    Compare,
    Problems,
    Tasks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum FilesPanelTab {
    #[default]
    Tree,
    Search,
    Migrations,
    Tasks,
    Graph,
    Git,
}

/// Cap for MRU recent-table entries persisted for Data Activity (#212).
pub(crate) const RECENT_TABLES_MAX: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ProblemsSeverityFilter {
    #[default]
    All,
    Errors,
    Warnings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ProblemsSourceFilter {
    #[default]
    All,
    Parser,
    Lint,
    Delimiter,
    Database,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProblemEntry {
    pub(crate) document_index: usize,
    pub(crate) document_id: String,
    pub(crate) document_title: String,
    pub(crate) diagnostic_index: usize,
    pub(crate) severity: crate::editor::DiagnosticSeverity,
    pub(crate) source: crate::editor::DiagnosticSource,
    pub(crate) message: String,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) range: (usize, usize),
    pub(crate) has_fix: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaletteMode {
    QuickOpen,
    Commands,
}

/// Scope chips for global search / command palette (#201).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SearchScope {
    #[default]
    All,
    Connections,
    Schema,
    Queries,
    Commands,
    Agent,
}

impl SearchScope {
    pub(crate) fn all() -> &'static [SearchScope] {
        &[
            SearchScope::All,
            SearchScope::Connections,
            SearchScope::Schema,
            SearchScope::Queries,
            SearchScope::Commands,
            SearchScope::Agent,
        ]
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            SearchScope::All => "All",
            SearchScope::Connections => "Connections",
            SearchScope::Schema => "Schema",
            SearchScope::Queries => "Queries",
            SearchScope::Commands => "Commands",
            SearchScope::Agent => "Agent",
        }
    }
}

/// Kind tags used by SearchService ranking (#201).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchKind {
    Connection,
    Table,
    View,
    Column,
    Function,
    SavedQuery,
    History,
    Command,
    Agent,
    WorkspaceFile,
    Navigation,
}

impl SearchKind {
    pub(crate) fn matches_scope(self, scope: SearchScope) -> bool {
        match scope {
            SearchScope::All => true,
            SearchScope::Connections => matches!(self, SearchKind::Connection),
            SearchScope::Schema => matches!(
                self,
                SearchKind::Table | SearchKind::View | SearchKind::Column | SearchKind::Function
            ),
            SearchScope::Queries => matches!(self, SearchKind::SavedQuery | SearchKind::History),
            SearchScope::Commands => matches!(self, SearchKind::Command | SearchKind::Navigation),
            SearchScope::Agent => matches!(self, SearchKind::Agent),
        }
    }

    /// Small boost so tables outrank history noise on equal match class.
    pub(crate) fn rank_boost(self) -> u32 {
        match self {
            SearchKind::Command | SearchKind::Navigation => 80,
            SearchKind::Table | SearchKind::View | SearchKind::Function => 70,
            SearchKind::Column | SearchKind::Connection => 60,
            SearchKind::SavedQuery | SearchKind::Agent => 50,
            SearchKind::WorkspaceFile => 40,
            SearchKind::History => 30,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PaletteAction {
    Welcome,
    Query,
    History,
    Data,
    Files,
    Settings,
    Diagram,
    Agent,
    Problems,
    Diagnostics,
    SchemaWorkbench,
    SchemaCompare,
    Transfers,
    Monitor,
    NewQuery,
    NewConnection,
    RefreshSchema,
    ToggleExplorer,
    OpenTable(String),
    OpenView(String),
    OpenFunction { name: String, identity_arguments: String },
    OpenSavedQuery(String),
    OpenHistoryEntry(usize),
    OpenWorkspaceFile(String),
    OpenWorkspaceFolder,
    CloseWorkspaceFolder,
    InsertColumn(String),
    InsertSnippet(usize),
    ExplainQuery,
    ExportResults,
    RunQuery,
    FormatSql,
    SwitchConnection(String),
    TogglePinTable(String),
    ComponentGallery,
}

#[derive(Debug, Clone)]
pub(crate) struct PaletteItem {
    pub(crate) icon: Icon,
    pub(crate) title: String,
    pub(crate) subtitle: String,
    pub(crate) shortcut: Option<String>,
    pub(crate) action: PaletteAction,
}

pub(crate) const AGENT_SIDEBAR_COLLAPSE_WIDTH: f32 = 1180.0;
pub(crate) const SIDEBAR_MIN_WIDTH: f32 = 220.0;
pub(crate) const SIDEBAR_MAX_WIDTH: f32 = 380.0;
pub(crate) const AGENT_MIN_WIDTH: f32 = 300.0;
pub(crate) const AGENT_MAX_WIDTH: f32 = 480.0;
pub(crate) const OUTPUT_MIN_HEIGHT: f32 = 120.0;
pub(crate) const OUTPUT_MAX_HEIGHT: f32 = 420.0;
pub(crate) const TABLE_PAGE_SIZE: u64 = 100;
pub(crate) const GRID_ROW_NUMBER_WIDTH: f32 = 48.0;
pub use crate::diagram::{ErGraph, ErSpatialIndex};

pub(crate) const EXPLORER_MAX_TABLES: usize = 100;
/// Row height used by the Codex navigator for viewport culling.
pub(crate) const EXPLORER_ROW_HEIGHT: f32 = 26.0;

/// Schemas the explorer should show. Hides PostgreSQL catalog / session-temp
/// namespaces (`pg_temp_*`, `pg_toast_temp_*`) even if a provider still returns them.
pub(crate) fn is_user_visible_schema(name: &str) -> bool {
    !matches!(name, "pg_catalog" | "information_schema" | "pg_toast")
        && !name.starts_with("pg_temp")
        && !name.starts_with("pg_toast_temp")
}

/// Cached explorer table listing for one `(connection, schema, search)` key.
/// Rebuilt only when the key changes — avoids reallocating 1k+ name strings every frame.
#[derive(Debug, Clone, Default)]
pub(crate) struct ExplorerNavCache {
    pub connection_id: String,
    pub schema: String,
    pub search: String,
    pub total_count: usize,
    pub matching_count: usize,
    pub visible: Vec<String>,
}

pub(crate) fn matches_diagram_search(table: &UiTableSummary, query: &str) -> bool {
    table.name.to_ascii_lowercase().contains(query)
        || table.schema.to_ascii_lowercase().contains(query)
        || table
            .columns
            .iter()
            .any(|column| column.name.to_ascii_lowercase().contains(query))
}

pub(crate) fn ddl_impact_summary(sql: &str, target: &str) -> String {
    let verb = sql.split_whitespace().next().unwrap_or_default().to_ascii_lowercase();
    match verb.as_str() {
        "drop" => format!("Drop {target}: removes the database object; recovery requires a backup."),
        "truncate" => format!("Truncate {target}: removes its rows; recovery requires a backup."),
        "alter" => format!("Alter {target}: changes its structure and may invalidate dependent queries."),
        "create" => format!("Create {target}: adds or rebuilds a database object."),
        _ => format!("This SQL changes {target}; review the preview and keep a backup before applying."),
    }
}

pub(crate) fn matches_explorer_table(table: &str, query: &str) -> bool {
    query.is_empty() || table.to_ascii_lowercase().contains(query)
}

pub(crate) fn filtered_explorer_tables(tables: &[String], query: &str) -> (usize, Vec<String>) {
    let matching_count = tables
        .iter()
        .filter(|table| matches_explorer_table(table, query))
        .count();
    let visible_tables = tables
        .iter()
        .filter(|table| matches_explorer_table(table, query))
        .take(EXPLORER_MAX_TABLES)
        .cloned()
        .collect();
    (matching_count, visible_tables)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkspaceTab {
    Welcome,
    Query,
    Table,
    SchemaObject,
    Diagram,
    SchemaWorkbench,
    SchemaCompare,
    ComponentGallery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TableView {
    Structure,
    Data,
    Indexes,
    Relations,
    Constraints,
    Dependencies,
    Ddl,
    Profile,
}

#[derive(Debug, Clone)]
pub(crate) struct ColumnProfile {
    pub name: String,
    pub data_type: String,
    pub null_count: usize,
    pub null_rate: f64,
    pub distinct_count: usize,
    pub distinct_rate: f64,
    pub min: Option<String>,
    pub max: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputTab {
    Results,
    Chart,
    Messages,
    Explain,
    History,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SchemaObjectSelection {
    View(String),
    Trigger(String),
    Function { name: String, identity_arguments: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SchemaObjectView {
    Definition,
    Data,
}
