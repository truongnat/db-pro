use crate::components::*;
use crate::editor::PredictionMode;
use crate::tokens::*;
use crate::{
    activity_bar_frame, agent_message_frame, badge, card_frame, compact_button, compact_button_with_icon,
    compact_icon_button, compact_icon_button_enabled, danger_button, editor_frame, empty_state, ghost_button_with_icon,
    grid_frame, icon_button, icon_text, input, input_full_width, menu_button_with_icon, panel_frame,
    primary_button_with_icon, secondary_button_with_icon, section_label, sidebar_frame, sidebar_item, tab_frame,
    toolbar_frame, AgentContext, AgentMessage, AgentProvider, AgentRole, ColumnWriteBlock, ColumnWritePolicy,
    DbProTheme, GridProjectionCache, GridProjectionKey, OfflineAgentProvider, TaskBridge, UiCell, UiCommand,
    UiConnectionDraft, UiConnectionSummary, UiDriver, UiEvent, UiFunctionSummary, UiQueryExecutionOutput,
    UiQueryFolderSummary, UiQueryHistoryEntry, UiQueryHistoryStatus, UiQueryResult, UiSavedQuerySummary,
    UiSchemaForeignKey, UiSchemaSummary, UiSslMode, UiStatementOutput, UiTableDataFilter, UiTableDataSort,
    UiTableFilterOperator, UiTableInfo, UiTableMutation, UiTableSummary, UiTriggerSummary, UiViewSummary,
};
use bigdecimal::BigDecimal;
use db_pro_core::domain::capabilities::DatabaseCapabilities;
use db_pro_core::domain::connection::DriverType;
use eframe::egui::{self, Align, Color32, FontId, Layout, RichText, Sense, TextEdit, TopBottomPanel};
use lucide_icons::Icon;
use serde::{Deserialize, Serialize};
use sqlparser::dialect::{GenericDialect, PostgreSqlDialect, SQLiteDialect};
use sqlparser::parser::Parser;
use std::collections::{BTreeSet, HashMap};
use std::time::{Duration, Instant};

use agent_workflow_state::AgentUiSession;
use change_set::{ChangeSet, MutationFailure, MutationTarget, RowIdentity, StagedChange};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PendingNavigationAction {
    OpenTable(String),
    ChangeSchema(String),
    ChangeConnection(String),
    CloseWorkspace(WorkspaceTab),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PersistedGridColumnLayout {
    column_name: String,
    width: f32,
    order: usize,
    hidden: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PersistedGridLayout {
    /// Stable layout entries. The legacy fields below are read only for
    /// migration and are intentionally not written after the next save.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    columns: Vec<PersistedGridColumnLayout>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    widths: Vec<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    order: Vec<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    hidden_columns: Vec<usize>,
}

#[path = "agent_state.rs"]
mod agent_state;
#[path = "agent_view.rs"]
mod agent_view;
#[path = "agent_workflow_state.rs"]
mod agent_workflow_state;
#[path = "app_state.rs"]
mod app_state;
#[path = "change_set.rs"]
mod change_set;
#[path = "component_gallery_view.rs"]
mod component_gallery_view;
#[path = "connection_view.rs"]
mod connection_view;

pub use component_gallery_view::ComponentGalleryState;
#[path = "diagram_view.rs"]
mod diagram_view;
#[path = "events.rs"]
mod events;
#[path = "explorer_details.rs"]
mod explorer_details;
#[path = "explorer_folders.rs"]
mod explorer_folders;
#[path = "explorer_tree.rs"]
mod explorer_tree;
#[path = "explorer_view.rs"]
mod explorer_view;
#[path = "navigation_view.rs"]
mod navigation_view;
#[path = "palette_view.rs"]
mod palette_view;
#[path = "query_view.rs"]
mod query_view;
#[path = "result_grid_view.rs"]
pub(crate) mod result_grid_view;
pub(crate) use result_grid_view::GridSelectionCache;
#[path = "schema_object_view.rs"]
mod schema_object_view;
#[path = "table_ddl_view.rs"]
mod table_ddl_view;
#[path = "table_editor_view.rs"]
mod table_editor_view;
#[path = "table_metadata_view.rs"]
mod table_metadata_view;
#[path = "table_view.rs"]
mod table_view;
#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
#[path = "workspace_view.rs"]
mod workspace_view;

pub use crate::query::{QueryDocument, QueryExecutionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Activity {
    Explorer,
    Queries,
    History,
    Transfers,
    Monitor,
    Settings,
    Diagram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaletteMode {
    QuickOpen,
    Commands,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PaletteAction {
    Welcome,
    Query,
    History,
    Settings,
    Diagram,
    Agent,
    NewQuery,
    NewConnection,
    RefreshSchema,
    ToggleExplorer,
    OpenTable(String),
    ExplainQuery,
    ExportResults,
    RunQuery,
    FormatSql,
    SwitchConnection(String),
    ComponentGallery,
}

#[derive(Debug, Clone)]
pub(crate) struct PaletteItem {
    icon: Icon,
    title: String,
    subtitle: String,
    shortcut: Option<String>,
    action: PaletteAction,
}

const AGENT_SIDEBAR_COLLAPSE_WIDTH: f32 = 1180.0;
const SIDEBAR_MIN_WIDTH: f32 = 220.0;
const SIDEBAR_MAX_WIDTH: f32 = 380.0;
const AGENT_MIN_WIDTH: f32 = 300.0;
const AGENT_MAX_WIDTH: f32 = 480.0;
const OUTPUT_MIN_HEIGHT: f32 = 120.0;
const OUTPUT_MAX_HEIGHT: f32 = 420.0;
const TABLE_PAGE_SIZE: u64 = 100;
const GRID_ROW_NUMBER_WIDTH: f32 = 48.0;
pub use crate::diagram::{ErGraph, ErSpatialIndex};

const EXPLORER_MAX_TABLES: usize = 100;

fn matches_diagram_search(table: &UiTableSummary, query: &str) -> bool {
    table.name.to_ascii_lowercase().contains(query)
        || table.schema.to_ascii_lowercase().contains(query)
        || table
            .columns
            .iter()
            .any(|column| column.name.to_ascii_lowercase().contains(query))
}

fn ddl_impact_summary(sql: &str, target: &str) -> String {
    let verb = sql.split_whitespace().next().unwrap_or_default().to_ascii_lowercase();
    match verb.as_str() {
        "drop" => format!("Drop {target}: removes the database object; recovery requires a backup."),
        "truncate" => format!("Truncate {target}: removes its rows; recovery requires a backup."),
        "alter" => format!("Alter {target}: changes its structure and may invalidate dependent queries."),
        "create" => format!("Create {target}: adds or rebuilds a database object."),
        _ => format!("This SQL changes {target}; review the preview and keep a backup before applying."),
    }
}

fn matches_explorer_table(table: &str, query: &str) -> bool {
    query.is_empty() || table.to_ascii_lowercase().contains(query)
}

fn filtered_explorer_tables(tables: &[String], query: &str) -> (usize, Vec<String>) {
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
    ComponentGallery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TableView {
    Structure,
    Data,
    Indexes,
    Relations,
    Constraints,
    Dependencies,
    Ddl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputTab {
    Results,
    Messages,
    Explain,
    History,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SchemaObjectSelection {
    View(String),
    Trigger(String),
    Function(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SchemaObjectView {
    Definition,
    Data,
}

/// First native vertical slice: visual shell, navigation, workspace tabs and
/// a functional query surface. The task bridge is backend-agnostic so the
/// same UI can run with the native runtime adapter or in an isolated preview.
pub struct DbProApp {
    theme: DbProTheme,
    dark_mode: bool,
    reduce_motion: bool,
    activity: Activity,
    welcome_open: bool,
    active_tab: WorkspaceTab,
    sidebar_open: bool,
    sidebar_width: f32,
    agent_open: bool,
    agent_width: f32,
    bottom_panel_open: bool,
    bottom_panel_height: f32,
    sidebar_open_before_agent: Option<bool>,
    pub prediction_mode: PredictionMode,
    welcome_prompt: String,
    selected_query: String,
    query_documents: Vec<QueryDocument>,
    active_query_document: usize,
    editor_search: String,
    editor_search_open: bool,
    query_editor_focused: bool,
    query_cursor_line: usize,
    query_cursor_column: usize,
    editor_font_size: f32,
    query_tools_open: bool,
    completion_open: bool,
    snippets_open: bool,
    diagnostics: Vec<String>,
    query_history: Vec<String>,
    query_history_entries: Vec<UiQueryHistoryEntry>,
    query_history_search: String,
    connection_name: String,
    connected: bool,
    palette_mode: Option<PaletteMode>,
    palette_query: String,
    palette_selected: usize,
    palette_focus_requested: bool,
    agent_pending_prompt: Option<String>,
    agent_pending_context: Option<AgentContext>,
    agent_provider_label: String,
    agent_provider_detail: String,
    agent_input: String,
    agent_messages: Vec<AgentMessage>,
    agent_sessions: HashMap<String, AgentUiSession>,
    pub(crate) agent_auto_run_read_only: bool,
    agent_settings_open: bool,
    agent_api_key_draft: String,
    agent_configure_request: Option<crate::RequestId>,
    task_bridge: TaskBridge,
    pub(crate) query_document_requests: HashMap<crate::RequestId, String>,
    query_save_requests: HashMap<crate::RequestId, String>,
    pending_dirty_close: Option<usize>,
    pending_close_after_save: Option<usize>,
    save_as_name: String,
    save_as_open: bool,
    runtime_message: String,
    toasts: crate::components::overlay::ToastManager,
    output_tab: OutputTab,
    query_output_tabs: HashMap<String, OutputTab>,
    grid_filter: String,
    grid_sort_column: Option<usize>,
    grid_sort_desc: bool,
    grid_column_widths: Vec<f32>,
    grid_column_order: Vec<usize>,
    grid_hidden_columns: BTreeSet<usize>,
    grid_layout_preferences: HashMap<String, PersistedGridLayout>,
    grid_pending_named_layout: Option<Vec<PersistedGridColumnLayout>>,
    grid_legacy_layout_pending: bool,
    grid_layout_column_names: Vec<String>,
    grid_row_identity_cache: HashMap<usize, RowIdentity>,
    grid_row_identity_cache_ready: bool,
    /// Monotonic id for the row data behind the grid. Everything that replaces the displayed result
    /// set, or edits a displayed row in place, must advance it through
    /// `invalidate_grid_projection`, or the grid keeps drawing the previous filtered/sorted
    /// projection (see `GridProjectionCache`).
    grid_projection_epoch: u64,
    grid_projection_cache: GridProjectionCache,
    grid_selection_cache: GridSelectionCache,
    grid_columns_user_resized: bool,
    selected_cell: Option<(usize, usize)>,
    selected_row: Option<usize>,
    selected_rows: BTreeSet<usize>,
    selection_anchor_row: Option<usize>,
    selection_anchor_cell: Option<(usize, usize)>,
    data_editing_cell: Option<(usize, usize)>,
    expanded_data_editor: Option<(usize, usize)>,
    data_edit_value: String,
    data_edit_error: Option<String>,
    data_delete_confirmation: bool,
    discard_changes_confirmation: bool,
    pub(crate) pending_navigation_action: Option<PendingNavigationAction>,
    insert_row_open: bool,
    insert_row_values: Vec<String>,
    insert_row_error: String,
    copy_status: String,
    export_open: bool,
    export_format: String,
    export_path: String,
    /// Set when the export dialog was asked to write over an existing file and is waiting for the
    /// user to confirm it (#244, E-1).
    export_overwrite_pending: bool,
    connections: Vec<UiConnectionSummary>,
    saved_queries: Vec<UiSavedQuerySummary>,
    query_folders: Vec<UiQueryFolderSummary>,
    schema: UiSchemaSummary,
    selected_schema: Option<String>,
    explorer_search: String,
    schema_error: Option<String>,
    schema_request: Option<crate::RequestId>,
    selected_table: Option<String>,
    selected_schema_object: Option<SchemaObjectSelection>,
    schema_object_view: SchemaObjectView,
    diagram_zoom: f32,
    diagram_pan: egui::Vec2,
    diagram_pan_origin: Option<egui::Vec2>,
    diagram_search: String,
    diagram_show_all: bool,
    diagram_neighborhood_depth: usize,
    diagram_graph: ErGraph,
    diagram_spatial_index: ErSpatialIndex,
    diagram_schema_version: u64,
    diagram_layout_worker: crate::diagram::ErLayoutWorker,
    diagram_layout_state: crate::diagram::ErLayoutState,
    diagram_latest_layout_request: u64,
    table_info: Option<UiTableInfo>,
    table_ddl: Option<String>,
    table_info_error: Option<String>,
    table_ddl_error: Option<String>,
    ddl_execute_confirmation: bool,
    /// A destructive statement the user must confirm before it reaches the database.
    pending_destructive_run: Option<events::PendingDestructiveRun>,
    ddl_execution_request: Option<crate::RequestId>,
    refresh_table_info_after_schema: bool,
    table_data_result: Option<UiQueryResult>,
    table_data_total_rows: Option<u64>,
    table_data_offset: u64,
    table_data_limit: u64,
    table_data_filter_column: String,
    table_data_filter_operator: UiTableFilterOperator,
    table_data_filter_value: String,
    table_data_filter_editing: Option<usize>,
    table_data_filters: Vec<UiTableDataFilter>,
    table_data_sorts: Vec<UiTableDataSort>,
    table_data_error: Option<String>,
    table_structure_search: String,
    table_metadata_search: String,
    table_column_detail: Option<String>,
    table_index_detail: Option<String>,
    table_dependency_filter: String,
    table_constraint_filter: String,
    table_info_request: Option<crate::RequestId>,
    table_ddl_request: Option<crate::RequestId>,
    table_data_request: Option<crate::RequestId>,
    table_row_reload_request: Option<crate::RequestId>,
    table_row_reload_identity: Option<RowIdentity>,
    table_mutation_request: Option<crate::RequestId>,
    staged_changes: ChangeSet,
    pending_changes_open: bool,
    staged_apply_request: Option<crate::RequestId>,
    staged_apply_targets: Vec<MutationTarget>,
    table_mutation_retry_after_reload: bool,
    table_mutation_retry_target: Option<MutationTarget>,
    table_mutation_error: Option<MutationFailure>,
    conflict_dialog_open: bool,
    table_view: TableView,
    query_folder: String,
    backup_output_path: String,
    restore_input_path: String,
    restore_confirmation: bool,
    active_connection_id: Option<String>,
    pending_connection_id: Option<String>,
    pending_connection_request: Option<crate::RequestId>,
    connection_errors: std::collections::HashMap<String, String>,
    failed_connection_ids: std::collections::HashSet<String>,
    connections_requested: bool,
    connections_request_pending: bool,
    connection_dialog_open: bool,
    editing_connection_id: Option<String>,
    connection_draft: UiConnectionDraft,
    connection_show_password: bool,
    connection_error: String,
    connection_test_valid: bool,
    connection_test_draft: Option<UiConnectionDraft>,
    delete_confirmation_id: Option<String>,
    folder_delete_confirmation: Option<String>,
    /// Persisted height of the Connections sub-pane inside the Explorer sidebar.
    connections_pane_height: f32,
    /// Persisted height of the Schemas sub-pane inside the Explorer sidebar.
    schemas_pane_height: f32,
    /// Counter for initial render frames to ensure window is maximized on startup.
    initial_frames_count: u8,
    pub gallery_state: component_gallery_view::ComponentGalleryState,
}

impl eframe::App for DbProApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.theme.surface_app.to_normalized_gamma_f32()
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.persist_current_grid_layout();
        if let Ok(layouts) = serde_json::to_string(&self.grid_layout_preferences) {
            storage.set_string("dbpro.native.grid-layouts", layouts);
        }
        if let Ok(widths) = serde_json::to_string(&self.grid_column_widths) {
            storage.set_string("dbpro.native.grid-widths", widths);
        }
        storage.set_string(
            "dbpro.native.grid-widths-customized",
            self.grid_columns_user_resized.to_string(),
        );
        if let Ok(documents) = serde_json::to_string(&self.query_documents) {
            storage.set_string("dbpro.native.query-documents", documents);
        }
        if let Ok(history) = serde_json::to_string(&self.query_history_entries) {
            storage.set_string("dbpro.native.query-history-v1", history);
        }
        storage.set_string("dbpro.native.theme-version", "light-first-v1".to_owned());
        storage.set_string("dbpro.native.dark-mode", self.dark_mode.to_string());
        storage.set_string("dbpro.native.reduce-motion", self.reduce_motion.to_string());
        if let Ok(prediction_mode) = serde_json::to_string(&self.prediction_mode) {
            storage.set_string("dbpro.native.prediction-mode", prediction_mode);
        }
        storage.set_string("dbpro.native.sidebar-width", self.sidebar_width.to_string());
        storage.set_string("dbpro.native.agent-width", self.agent_width.to_string());
        storage.set_string("dbpro.native.output-open", self.bottom_panel_open.to_string());
        storage.set_string("dbpro.native.output-height", self.bottom_panel_height.to_string());
        storage.set_string(
            "dbpro.native.connections-pane-height",
            self.connections_pane_height.to_string(),
        );
        storage.set_string("dbpro.native.schemas-pane-height", self.schemas_pane_height.to_string());
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Map Ctrl to Command in input events so Ctrl+A/C/V/X/Z work seamlessly on macOS
        ctx.input_mut(|i| {
            if i.modifiers.ctrl {
                i.modifiers.command = true;
            }
            for event in &mut i.events {
                if let egui::Event::Key { modifiers, .. } = event {
                    if modifiers.ctrl {
                        modifiers.command = true;
                    }
                }
            }
        });

        if self.initial_frames_count < 3 {
            self.initial_frames_count += 1;
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        }
        self.request_connections_once();
        self.apply_runtime_events();
        if self.runtime_work_pending() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        self.theme = if self.dark_mode {
            DbProTheme::dark()
        } else {
            DbProTheme::light()
        };
        self.theme.apply(ctx);
        self.handle_shortcuts(ctx);
        self.draw_topbar(ctx);
        self.draw_output_panel(ctx);
        self.draw_statusbar(ctx);
        self.draw_activity_bar(ctx);

        if self.sidebar_open {
            self.draw_sidebar(ctx);
        }

        if self.agent_open {
            self.draw_agent_panel(ctx);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(self.theme.surface_app))
            .show(ctx, |ui| {
                self.draw_workspace(ui);
            });

        if self.connection_dialog_open {
            self.draw_connection_dialog(ctx);
        }
        if self.delete_confirmation_id.is_some() {
            self.draw_delete_confirmation(ctx);
        }
        if self.folder_delete_confirmation.is_some() {
            self.draw_folder_delete_confirmation(ctx);
        }
        if self.insert_row_open {
            self.draw_insert_row_dialog(ctx);
        }
        if self.palette_mode.is_some() {
            self.draw_palette(ctx);
        }

        self.toasts.render_ctx(ctx, self.theme);
        if !self.toasts.is_empty() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
}

/// The answer to "which capabilities apply to this connection?".
///
/// Deliberately not an `Option`: a lookup that cannot answer has to say *why*, so a
/// driver the UI has no provider entry for is a named state instead of `None`. `None`
/// conflated two different situations — "no connection is active" and "this driver has
/// no capability entry" — and a consumer could not tell them apart from a genuine
/// "the provider supports nothing".
#[derive(Debug, Clone)]
pub(crate) enum CapabilityLookup {
    /// The driver is a provider this build ships; the set is authoritative.
    Supported(DatabaseCapabilities),
    /// Nothing is connected, so there is nothing to resolve.
    NoActiveConnection,
    /// The connection names a driver with no provider entry in this build.
    UnsupportedDriver { driver: String },
}

impl CapabilityLookup {
    /// Resolves a connection's driver label to the provider entry the UI dispatches on.
    ///
    /// The label is what the runtime stored on the connection summary (`PostgreSQL`,
    /// `SQLite`, `MySQL`). A label with no entry resolves to
    /// [`CapabilityLookup::UnsupportedDriver`] rather than to a default driver's set.
    pub(crate) fn for_driver_label(label: &str) -> Self {
        let driver = if label.eq_ignore_ascii_case("sqlite") {
            Some(DriverType::SQLite)
        } else if label.eq_ignore_ascii_case("postgresql") || label.eq_ignore_ascii_case("postgres") {
            Some(DriverType::Postgres)
        } else if label.eq_ignore_ascii_case("mysql") {
            Some(DriverType::Mysql)
        } else {
            None
        };
        match driver {
            Some(driver) => Self::Supported(DatabaseCapabilities::for_driver(driver)),
            None => Self::UnsupportedDriver {
                driver: label.to_owned(),
            },
        }
    }

    /// The resolved capability set, when one exists.
    pub(crate) fn resolved(&self) -> Option<&DatabaseCapabilities> {
        match self {
            Self::Supported(capabilities) => Some(capabilities),
            Self::NoActiveConnection | Self::UnsupportedDriver { .. } => None,
        }
    }

    /// Whether the resolved set satisfies `predicate`.
    ///
    /// A lookup that could not answer never satisfies a capability predicate, so an
    /// unresolved driver never enables a capability-gated action.
    pub(crate) fn allows(&self, predicate: impl FnOnce(&DatabaseCapabilities) -> bool) -> bool {
        self.resolved().is_some_and(predicate)
    }

    /// A user-facing reason this lookup has no capability set, if it has none.
    pub(crate) fn unavailable_reason(&self) -> Option<String> {
        match self {
            Self::Supported(_) => None,
            Self::NoActiveConnection => Some("no database connection is active".to_owned()),
            Self::UnsupportedDriver { driver } => Some(format!("{driver} has no provider entry in this build")),
        }
    }
}

impl DbProApp {
    fn grid_layout_scope(&self) -> Option<String> {
        Some(format!(
            "{}|{}|{}",
            self.active_connection_id.as_deref()?,
            self.active_schema(),
            self.selected_table.as_deref()?
        ))
    }

    fn persist_current_grid_layout(&mut self) {
        let Some(scope) = self.grid_layout_scope() else {
            return;
        };
        let column_names = self.grid_layout_column_names.clone();
        if column_names.len() != self.grid_column_order.len() {
            return;
        }
        let columns = self
            .grid_column_order
            .iter()
            .enumerate()
            .filter_map(|(order, &column_index)| {
                let column_name = column_names.get(column_index)?.clone();
                Some(PersistedGridColumnLayout {
                    column_name,
                    width: self.grid_column_widths.get(column_index).copied().unwrap_or(180.0),
                    order,
                    hidden: self.grid_hidden_columns.contains(&column_index),
                })
            })
            .collect();
        self.grid_layout_preferences.insert(
            scope,
            PersistedGridLayout {
                columns,
                ..PersistedGridLayout::default()
            },
        );
    }

    pub(crate) fn restore_grid_layout_for_active_table(&mut self) {
        let Some(scope) = self.grid_layout_scope() else {
            return;
        };
        let Some(layout) = self.grid_layout_preferences.get(&scope).cloned() else {
            self.grid_column_widths.clear();
            self.grid_column_order.clear();
            self.grid_hidden_columns.clear();
            self.grid_pending_named_layout = None;
            self.grid_legacy_layout_pending = false;
            self.grid_columns_user_resized = false;
            return;
        };
        self.grid_layout_column_names.clear();
        self.grid_pending_named_layout = (!layout.columns.is_empty()).then_some(layout.columns);
        self.grid_legacy_layout_pending = self.grid_pending_named_layout.is_none()
            && (!layout.widths.is_empty() || !layout.order.is_empty() || !layout.hidden_columns.is_empty());
        self.grid_column_widths = layout.widths;
        self.grid_column_order = layout.order;
        self.grid_hidden_columns = layout.hidden_columns.into_iter().collect();
        self.grid_columns_user_resized = !self.grid_column_widths.is_empty();
    }

    pub(crate) fn show_toast_error(&mut self, message: impl Into<String>) {
        self.toasts
            .error(message, crate::components::overlay::ToastPosition::BottomRight);
    }

    pub(crate) fn show_toast_success(&mut self, message: impl Into<String>) {
        self.toasts
            .success(message, crate::components::overlay::ToastPosition::BottomRight);
    }

    #[allow(dead_code)]
    pub(crate) fn show_toast_info(&mut self, message: impl Into<String>) {
        self.toasts
            .info(message, crate::components::overlay::ToastPosition::BottomRight);
    }

    pub(super) fn primary_modifier_pressed(input: &egui::InputState) -> bool {
        input.modifiers.command || input.modifiers.ctrl || input.modifiers.mac_cmd
    }

    fn primary_modifier_label() -> &'static str {
        if cfg!(target_os = "macos") {
            "⌘"
        } else {
            "Ctrl"
        }
    }

    /// Queues a command for the runtime worker, ignoring transport failures.
    ///
    /// The UI is fire-and-forget: a send only fails once the worker channel is
    /// closed (shutdown), and a frame that already drew its widgets has nothing
    /// actionable to do about it. Runtime-side problems are reported back
    /// through `UiEvent`, not through this return value.
    fn dispatch_command(&mut self, command: UiCommand) {
        // Intentionally ignored — see the method contract above.
        let _ = self.task_bridge.send(command);
    }

    fn active_connection(&self) -> Option<&UiConnectionSummary> {
        self.connections
            .iter()
            .find(|connection| Some(connection.id.as_str()) == self.active_connection_id.as_deref())
    }

    fn active_connection_name(&self) -> &str {
        self.active_connection()
            .map(|connection| connection.name.as_str())
            .unwrap_or(self.connection_name.as_str())
    }

    fn active_driver(&self) -> &str {
        self.active_connection()
            .map(|connection| connection.driver.as_str())
            .unwrap_or("PostgreSQL")
    }

    pub(crate) fn active_capabilities(&self) -> CapabilityLookup {
        match self.active_connection() {
            Some(connection) => CapabilityLookup::for_driver_label(&connection.driver),
            None => CapabilityLookup::NoActiveConnection,
        }
    }

    fn active_schema(&self) -> &str {
        self.selected_schema
            .as_deref()
            .or_else(|| self.schema.schemas.first().map(String::as_str))
            .unwrap_or_else(|| {
                if self.active_driver().eq_ignore_ascii_case("sqlite") {
                    "main"
                } else {
                    "public"
                }
            })
    }

    fn active_schema_table_names(&self) -> Vec<String> {
        if self.schema.schemas.is_empty() || self.schema.table_details.is_empty() {
            return self.schema.tables.clone();
        }
        self.schema
            .table_details
            .iter()
            .filter(|table| table.schema == self.active_schema())
            .map(|table| table.name.clone())
            .collect()
    }

    fn active_schema_column_names(&self) -> Vec<String> {
        if self.schema.schemas.is_empty() || self.schema.table_details.is_empty() {
            return self.schema.columns.clone();
        }
        self.schema
            .table_details
            .iter()
            .filter(|table| table.schema == self.active_schema())
            .flat_map(|table| table.columns.iter().map(|column| column.name.clone()))
            .collect()
    }

    fn has_runtime_error(&self) -> bool {
        self.runtime_message.contains("failed")
            || self.runtime_message.contains("Failed")
            || self.runtime_message.contains("error")
            || self.runtime_message.contains("Error")
    }

    /// The status-bar message and the colour it is rendered in.
    ///
    /// Every `runtime_message` is user-facing: a refusal such as "Connect with write
    /// access to delete rows" is the *only* feedback a blocked action produces, so the
    /// bar shows any non-empty message and reserves the danger colour for errors.
    /// Restricting the bar to strings containing "failed"/"error" hid every refusal,
    /// gate and informational message the app sets.
    fn runtime_status(&self) -> Option<(String, Color32)> {
        if self.runtime_message.trim().is_empty() {
            None
        } else if self.has_runtime_error() {
            Some((self.runtime_message.clone(), self.theme.danger))
        } else {
            Some((self.runtime_message.clone(), self.theme.text_secondary))
        }
    }

    fn statusbar_state(&self) -> (Icon, Color32, &'static str) {
        if self.connected && self.active_connection_id.is_some() {
            return (Icon::CircleCheck, self.theme.success, "Connected");
        }
        if self.runtime_message.starts_with("Connecting") {
            return (Icon::Circle, self.theme.accent, "Connecting…");
        }
        if self.has_runtime_error() {
            return (Icon::TriangleAlert, self.theme.danger, "Runtime error");
        }
        (Icon::Circle, self.theme.warning, "Not connected")
    }

    pub(super) fn shows_editor_status(&self) -> bool {
        self.active_tab == WorkspaceTab::Query
    }

    pub(super) fn statusbar_context_label(&self) -> &'static str {
        match self.active_tab {
            WorkspaceTab::Welcome => "Workspace",
            WorkspaceTab::Query => "SQL Editor",
            WorkspaceTab::Table => match self.table_view {
                TableView::Structure => "Table Structure",
                TableView::Data => "Data Editor",
                TableView::Indexes => "Table Indexes",
                TableView::Relations => "Table Relations",
                TableView::Constraints => "Table Constraints",
                TableView::Dependencies => "Table Dependencies",
                TableView::Ddl => "Table DDL",
            },
            WorkspaceTab::SchemaObject => "Schema Object",
            WorkspaceTab::Diagram => "ER Diagram",
            WorkspaceTab::ComponentGallery => "Component Gallery",
        }
    }

    fn connection_indicator(&self, connection: &UiConnectionSummary) -> (Icon, Color32) {
        let is_active = self.active_connection_id.as_deref() == Some(connection.id.as_str());
        let is_connected = is_active && self.connected;
        let is_failed = self.failed_connection_ids.contains(&connection.id);
        let icon = if is_connected {
            Icon::CircleCheck
        } else if is_failed {
            Icon::AlertCircle
        } else {
            Icon::Circle
        };
        let color = if is_connected && connection.readonly {
            self.theme.warning
        } else if is_connected {
            self.theme.success
        } else if is_failed {
            self.theme.danger
        } else if is_active {
            self.theme.accent
        } else {
            self.theme.text_muted
        };
        (icon, color)
    }

    pub(crate) fn active_query_text(&self) -> &str {
        self.query_documents
            .get(self.active_query_document)
            .map(|doc| doc.text())
            .unwrap_or("")
    }

    pub(crate) fn set_active_query_text(&mut self, text: impl Into<String>) {
        self.cancel_prediction_for_document(self.active_query_document);
        if let Some(doc) = self.query_documents.get_mut(self.active_query_document) {
            doc.set_text(text);
        }
    }

    pub(crate) fn append_to_active_query(&mut self, text: &str) {
        self.cancel_prediction_for_document(self.active_query_document);
        if let Some(doc) = self.query_documents.get_mut(self.active_query_document) {
            let mut current = doc.text().to_owned();
            if !current.trim().is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(text);
            doc.set_text(current);
        }
    }

    pub(crate) fn active_explain_plan(&self) -> Option<&str> {
        self.query_documents
            .get(self.active_query_document)
            .and_then(|d| d.explain_plan.as_deref())
    }

    pub(crate) fn active_explain_request(&self) -> Option<crate::RequestId> {
        self.query_documents
            .get(self.active_query_document)
            .and_then(|d| d.explain_request)
    }

    pub(crate) fn active_query_output_tab(&self) -> OutputTab {
        self.query_documents
            .get(self.active_query_document)
            .and_then(|doc| self.query_output_tabs.get(&doc.id).copied())
            .unwrap_or(OutputTab::Results)
    }

    pub(crate) fn set_active_query_output_tab(&mut self, tab: OutputTab) {
        self.output_tab = tab;
        if let Some(doc_id) = self
            .query_documents
            .get(self.active_query_document)
            .map(|doc| doc.id.clone())
        {
            self.query_output_tabs.insert(doc_id, tab);
        }
    }

    pub(crate) fn set_query_output_tab(&mut self, document_id: &str, tab: OutputTab) {
        self.query_output_tabs.insert(document_id.to_owned(), tab);
        if self
            .query_documents
            .get(self.active_query_document)
            .is_some_and(|doc| doc.id == document_id)
        {
            self.output_tab = tab;
        }
    }

    pub(crate) fn active_query_running_request(&self) -> Option<crate::RequestId> {
        self.query_documents
            .get(self.active_query_document)
            .and_then(|doc| match doc.execution_state {
                QueryExecutionState::Running(request_id) => Some(request_id),
                _ => None,
            })
    }

    pub(crate) fn switch_query_document(&mut self, index: usize) {
        if index >= self.query_documents.len() || index == self.active_query_document {
            return;
        }
        self.active_query_document = index;
        let doc = &self.query_documents[index];
        self.query_cursor_line = doc.cursor.line + 1;
        self.query_cursor_column = doc.cursor.col + 1;
        if !doc.selection.is_empty() {
            let (start, end) = doc.selection.normalized();
            self.selected_query = doc.buffer.slice(start, end).to_owned();
        } else {
            self.selected_query.clear();
        }
        self.runtime_message = format!("Opened {}", self.query_documents[index].title);
    }

    pub(crate) fn active_query_result(&self) -> Option<&UiQueryResult> {
        self.query_documents.get(self.active_query_document).and_then(|doc| {
            doc.query_results
                .get(doc.active_result_index)
                .or(doc.query_result.as_ref())
        })
    }

    pub(crate) fn active_query_result_count(&self) -> usize {
        self.query_documents.get(self.active_query_document).map_or(0, |doc| {
            doc.query_results
                .len()
                .max(if doc.query_result.is_some() { 1 } else { 0 })
        })
    }

    pub(crate) fn set_active_query_result(&mut self, index: usize) {
        if let Some(doc) = self.query_documents.get_mut(self.active_query_document) {
            if index < doc.query_results.len() && doc.active_result_index != index {
                doc.active_result_index = index;
                self.invalidate_grid_projection();
            }
        }
    }

    /// Advance the grid's projection epoch: the displayed rows are about to change.
    ///
    /// Called wherever the row data behind the grid is replaced or edited in place — loading query
    /// results, loading table data, reloading one row, switching the active result set. Missing a
    /// call does not corrupt data, but the grid would keep drawing the previous order and filter.
    pub(crate) fn invalidate_grid_projection(&mut self) {
        self.grid_projection_epoch = self.grid_projection_epoch.wrapping_add(1);
    }

    /// Drop the per-row identity cache and the projection built from those rows.
    ///
    /// The two are invalidated together on purpose: every site that changes row data needs both, and
    /// keeping them in one call is what makes "no site was forgotten" checkable by grep.
    pub(crate) fn invalidate_grid_row_caches(&mut self) {
        self.grid_row_identity_cache.clear();
        self.grid_row_identity_cache_ready = false;
        self.invalidate_grid_projection();
    }

    /// The projection key for the result currently being drawn.
    fn grid_projection_key(&self, result: &UiQueryResult) -> GridProjectionKey {
        GridProjectionKey {
            epoch: self.grid_projection_epoch,
            filter: self.grid_filter.clone(),
            sort_column: self.grid_sort_column,
            sort_desc: self.grid_sort_desc,
            row_count: result.row_count,
            column_count: result.columns.len(),
        }
    }

    pub(crate) fn active_query_messages(&self) -> &[String] {
        self.query_documents
            .get(self.active_query_document)
            .map(|doc| doc.query_messages.as_slice())
            .unwrap_or(&[])
    }

    pub(crate) fn active_query_connection_id(&self) -> Option<&str> {
        self.query_documents
            .get(self.active_query_document)
            .and_then(|doc| doc.connection_id.as_deref())
            .or(self.active_connection_id.as_deref())
    }

    pub(crate) fn active_query_connection(&self) -> Option<&UiConnectionSummary> {
        let conn_id = self.active_query_connection_id()?;
        self.connections.iter().find(|c| c.id == conn_id)
    }

    /// Capabilities for the connection the active query document is bound to.
    ///
    /// Resolved from the bound connection, then from the active connection. It
    /// deliberately does **not** go through `active_query_driver`, whose display fallback
    /// is the literal `"PostgreSQL"`: answering with PostgreSQL's set while no connection
    /// exists is the same silent-wrong-answer this lookup replaces with a named state.
    pub(crate) fn query_capabilities(&self) -> CapabilityLookup {
        match self.active_query_connection() {
            Some(connection) => CapabilityLookup::for_driver_label(&connection.driver),
            None => match self.active_connection() {
                Some(connection) => CapabilityLookup::for_driver_label(&connection.driver),
                None => CapabilityLookup::NoActiveConnection,
            },
        }
    }

    pub(crate) fn active_query_connection_name(&self) -> &str {
        self.active_query_connection()
            .map(|c| c.name.as_str())
            .unwrap_or(self.connection_name.as_str())
    }

    pub(crate) fn active_query_driver(&self) -> &str {
        self.active_query_connection()
            .map(|c| c.driver.as_str())
            .unwrap_or_else(|| self.active_driver())
    }

    pub(crate) fn active_query_schema(&self) -> &str {
        self.query_documents
            .get(self.active_query_document)
            .and_then(|doc| doc.schema.as_deref())
            .unwrap_or_else(|| self.active_schema())
    }

    pub(crate) fn set_document_connection(&mut self, doc_index: usize, connection_id: Option<String>) {
        self.cancel_prediction_for_document(doc_index);
        if let Some(doc) = self.query_documents.get_mut(doc_index) {
            doc.connection_id = connection_id;
            doc.completion.clear();
        }
    }

    pub(crate) fn set_document_schema(&mut self, doc_index: usize, schema: Option<String>) {
        self.cancel_prediction_for_document(doc_index);
        if let Some(doc) = self.query_documents.get_mut(doc_index) {
            doc.schema = schema;
            doc.completion.clear();
        }
    }

    pub(crate) fn cancel_prediction_for_document(&mut self, doc_index: usize) {
        let request_id = self
            .query_documents
            .get(doc_index)
            .and_then(|doc| doc.pending_prediction_request);
        if let Some(request_id) = request_id {
            self.dispatch_command(UiCommand::CancelSqlPrediction { request_id });
        }
        if let Some(doc) = self.query_documents.get_mut(doc_index) {
            if request_id.is_some() {
                doc.prediction_requests_cancelled = doc.prediction_requests_cancelled.saturating_add(1);
            }
            doc.invalidate_prediction();
        }
    }

    pub(crate) fn new_query_document(&mut self) {
        let (document_id, index) = self.next_query_document_identity();
        let mut doc = QueryDocument::new(document_id, format!("Query {index}"), String::new());
        doc.connection_id = self.active_connection_id.clone();
        doc.schema = Some(self.active_schema().to_owned());
        self.query_documents.push(doc);
        self.active_query_document = self.query_documents.len() - 1;
        self.reset_query_cursor();
        self.activity = Activity::Queries;
        self.sidebar_open = true;
        self.active_tab = WorkspaceTab::Query;
    }

    pub(crate) fn open_history_entry(&mut self, entry: &UiQueryHistoryEntry, run: bool) {
        let (document_id, document_number) = self.next_query_document_identity();
        let mut document = QueryDocument::new(document_id, format!("History {document_number}"), entry.sql.clone());
        document.connection_id = entry.connection_id.clone();
        document.schema = entry.schema.clone();
        self.query_documents.push(document);
        self.active_query_document = self.query_documents.len() - 1;
        self.activity = Activity::Queries;
        self.active_tab = WorkspaceTab::Query;
        self.reset_query_cursor();
        if run {
            self.dispatch_query();
        }
    }

    pub(crate) fn close_query_document(&mut self, index: usize) {
        if index >= self.query_documents.len() {
            return;
        }

        self.cancel_prediction_for_document(index);
        let closed_id = self.query_documents[index].id.clone();
        let closed_title = self.query_documents[index].title.clone();
        if let Some(run_id) = self
            .agent_sessions
            .get(&closed_id)
            .and_then(|session| session.active_run_id)
        {
            let request_id = self.task_bridge.next_request_id();
            let _ = self.task_bridge.send(UiCommand::CancelAgentRun { request_id, run_id });
        }
        self.agent_sessions.remove(&closed_id);
        self.query_documents.remove(index);
        self.query_output_tabs.remove(&closed_id);

        if self.query_documents.is_empty() {
            self.active_query_document = 0;
            self.reset_query_cursor();
            if self.active_tab == WorkspaceTab::Query {
                self.activate_fallback_workspace_tab();
            }
            self.runtime_message = format!("Closed {closed_title}");
            return;
        }

        if self.active_query_document > index {
            self.active_query_document -= 1;
        } else if self.active_query_document == index {
            self.active_query_document = self.active_query_document.min(self.query_documents.len() - 1);
        }
        let doc = &self.query_documents[self.active_query_document];
        self.query_cursor_line = doc.cursor.line + 1;
        self.query_cursor_column = doc.cursor.col + 1;
        if !doc.selection.is_empty() {
            let (start, end) = doc.selection.normalized();
            self.selected_query = doc.buffer.slice(start, end).to_owned();
        } else {
            self.selected_query.clear();
        }
        self.runtime_message = format!("Closed {}", self.query_documents[self.active_query_document].title);
    }

    fn next_query_document_identity(&self) -> (String, usize) {
        let mut number = self.query_documents.len().saturating_add(1);
        loop {
            let id = format!("query-{number}");
            if !self.query_documents.iter().any(|document| document.id == id) {
                return (id, number);
            }
            number = number.saturating_add(1);
        }
    }

    pub(crate) fn request_close_query_document(&mut self, index: usize) {
        if self.query_documents.get(index).is_some_and(QueryDocument::is_dirty) {
            self.pending_dirty_close = Some(index);
        } else {
            self.close_query_document(index);
        }
    }

    fn reset_query_cursor(&mut self) {
        self.query_cursor_line = 1;
        self.query_cursor_column = 1;
    }

    pub(crate) fn duplicate_query_document(&mut self, index: usize) {
        if index >= self.query_documents.len() {
            return;
        }
        let src = &self.query_documents[index];
        let title = format!("{} (Copy)", src.title);
        let content = src.text().to_owned();
        let (document_id, _) = self.next_query_document_identity();
        let mut new_doc = QueryDocument::new(document_id, title, content);
        new_doc.connection_id = src.connection_id.clone().or_else(|| self.active_connection_id.clone());
        new_doc.schema = src.schema.clone().or_else(|| Some(self.active_schema().to_owned()));
        self.query_documents.push(new_doc);
        self.active_query_document = self.query_documents.len() - 1;
        self.active_tab = WorkspaceTab::Query;
        self.runtime_message = format!("Duplicated {}", self.query_documents[index].title);
    }

    pub(crate) fn close_other_query_documents(&mut self, keep_index: usize) {
        if keep_index >= self.query_documents.len() {
            return;
        }
        for index in 0..self.query_documents.len() {
            if index != keep_index {
                self.cancel_prediction_for_document(index);
            }
        }
        let kept = self.query_documents[keep_index].clone();
        self.query_documents = vec![kept];
        self.active_query_document = 0;
        self.runtime_message = "Closed other queries".to_owned();
    }

    pub(crate) fn close_query_documents_to_right(&mut self, index: usize) {
        if index >= self.query_documents.len() {
            return;
        }
        for query_index in index + 1..self.query_documents.len() {
            self.cancel_prediction_for_document(query_index);
        }
        self.query_documents.truncate(index + 1);
        if self.active_query_document > index {
            self.active_query_document = index;
        }
        self.runtime_message = "Closed queries to the right".to_owned();
    }

    pub(crate) fn close_all_tabs(&mut self) {
        for index in 0..self.query_documents.len() {
            self.cancel_prediction_for_document(index);
        }
        self.welcome_open = true;
        self.query_documents = vec![QueryDocument::new("query-1", "Query 1", String::new())];
        self.active_query_document = 0;
        self.selected_table = None;
        self.selected_schema_object = None;
        self.active_tab = WorkspaceTab::Welcome;
        self.runtime_message = "Closed all tabs".to_owned();
    }

    pub(crate) fn close_welcome_tab(&mut self) {
        self.welcome_open = false;
        if self.active_tab == WorkspaceTab::Welcome {
            self.activate_fallback_workspace_tab();
        }
        self.runtime_message = "Closed Welcome".to_owned();
    }

    fn activate_welcome_tab(&mut self) {
        self.welcome_open = true;
        self.active_tab = WorkspaceTab::Welcome;
    }

    fn activate_fallback_workspace_tab(&mut self) {
        if !self.query_documents.is_empty() {
            self.active_tab = WorkspaceTab::Query;
        } else if self.selected_table.is_some() {
            self.active_tab = WorkspaceTab::Table;
        } else if self.selected_schema_object.is_some() {
            self.active_tab = WorkspaceTab::SchemaObject;
        } else {
            self.activate_welcome_tab();
        }
    }

    pub(crate) fn execute_pending_navigation(&mut self, action: PendingNavigationAction) {
        match action {
            PendingNavigationAction::OpenTable(table) => {
                self.open_table(table);
            }
            PendingNavigationAction::ChangeSchema(schema) => {
                self.activate_schema(&schema);
            }
            PendingNavigationAction::ChangeConnection(connection_id) => {
                if let Some(conn) = self.connections.iter().find(|c| c.id == connection_id).cloned() {
                    self.connect_to_connection(&conn);
                }
            }
            PendingNavigationAction::CloseWorkspace(tab) => {
                self.request_close_workspace_tab(tab);
            }
        }
    }

    pub(crate) fn request_close_workspace_tab(&mut self, tab: WorkspaceTab) {
        match tab {
            WorkspaceTab::Table => {
                if !self.staged_changes.is_empty() {
                    self.pending_navigation_action = Some(PendingNavigationAction::CloseWorkspace(tab));
                    self.discard_changes_confirmation = true;
                    self.runtime_message = "Apply or discard staged changes before closing the table".to_owned();
                    return;
                }
                self.pending_navigation_action = None;
                self.selected_table = None;
                self.table_info = None;
                self.table_ddl = None;
                self.table_info_error = None;
                self.table_ddl_error = None;
                self.table_data_result = None;
                self.table_data_total_rows = None;
                self.table_data_request = None;
                self.table_info_request = None;
                self.table_ddl_request = None;
                self.table_mutation_request = None;
                self.staged_changes.clear();
                self.staged_apply_request = None;
                self.staged_apply_targets.clear();
                self.table_mutation_retry_after_reload = false;
                self.table_mutation_retry_target = None;
                self.table_mutation_error = None;
                self.selected_cell = None;
                self.selected_row = None;
                self.selected_rows.clear();
                self.selection_anchor_row = None;
                self.selection_anchor_cell = None;
                self.data_editing_cell = None;
                self.data_edit_error = None;
                self.data_delete_confirmation = false;
                self.discard_changes_confirmation = false;
            }
            WorkspaceTab::SchemaObject => {
                self.selected_schema_object = None;
                self.schema_object_view = SchemaObjectView::Definition;
                self.table_data_result = None;
                self.table_data_total_rows = None;
                self.table_data_request = None;
            }
            WorkspaceTab::Diagram => {
                self.diagram_search.clear();
                self.diagram_show_all = false;
                self.diagram_pan = egui::Vec2::ZERO;
                self.diagram_pan_origin = None;
            }
            WorkspaceTab::ComponentGallery => {}
            WorkspaceTab::Welcome | WorkspaceTab::Query => return,
        }
        if self.active_tab == tab {
            self.activate_welcome_tab();
        }
        self.runtime_message = "Workspace closed".to_owned();
    }

    pub(crate) fn open_table(&mut self, table: String) {
        if self.selected_table.as_deref() == Some(&table) {
            self.active_tab = WorkspaceTab::Table;
            return;
        }
        if !self.staged_changes.is_empty() {
            self.pending_navigation_action = Some(PendingNavigationAction::OpenTable(table));
            self.discard_changes_confirmation = true;
            self.runtime_message = "Apply or discard staged changes before opening another table".to_owned();
            return;
        }
        self.pending_navigation_action = None;
        self.persist_current_grid_layout();
        self.selected_table = Some(table);
        self.restore_grid_layout_for_active_table();
        self.request_table_info();
        self.request_table_data();
        self.active_tab = WorkspaceTab::Table;
    }

    fn open_palette(&mut self, mode: PaletteMode) {
        self.palette_mode = Some(mode);
        self.palette_query.clear();
        self.palette_selected = 0;
        self.palette_focus_requested = true;
    }

    fn open_new_connection(&mut self) {
        self.editing_connection_id = None;
        self.connection_draft = UiConnectionDraft::default();
        self.connection_error.clear();
        self.connection_test_valid = false;
        self.connection_test_draft = None;
        self.pending_connection_request = None;
        self.connection_dialog_open = true;
    }

    pub(crate) fn set_agent_open(&mut self, open: bool, ctx: &egui::Context) {
        if open == self.agent_open {
            return;
        }
        self.agent_open = open;
        if open {
            self.sidebar_open_before_agent = Some(self.sidebar_open);
            if ctx.screen_rect().width() < AGENT_SIDEBAR_COLLAPSE_WIDTH {
                self.sidebar_open = false;
            }
        } else if let Some(sidebar_open) = self.sidebar_open_before_agent.take() {
            self.sidebar_open = sidebar_open;
        }
    }

    pub(crate) fn open_agent_prompt(&mut self, prompt: impl Into<String>, ctx: &egui::Context) {
        self.agent_input = prompt.into();
        self.set_agent_open(true, ctx);
    }

    fn request_connections_once(&mut self) {
        if self.connections_requested {
            return;
        }
        self.connections_requested = true;
        self.connections_request_pending = true;
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ListConnections { request_id });
    }

    fn runtime_work_pending(&self) -> bool {
        self.connections_request_pending
            || self.pending_connection_request.is_some()
            || self.schema_request.is_some()
            || self
                .query_documents
                .iter()
                .any(|d| d.pending_prediction_request.is_some() || d.prediction_debounce_deadline.is_some())
            || self.query_documents.iter().any(|d| d.explain_request.is_some())
            || self
                .agent_sessions
                .values()
                .any(|session| session.request_id.is_some() || session.active_run_id.is_some())
            || self.table_info_request.is_some()
            || self.table_ddl_request.is_some()
            || self.table_data_request.is_some()
            || self.table_mutation_request.is_some()
            || self.staged_apply_request.is_some()
            || self.ddl_execution_request.is_some()
    }

    fn request_schema_introspection(&mut self, connection_id: String, force_refresh: bool) {
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::IntrospectSchema {
            request_id,
            connection_id,
            force_refresh,
        });
        self.schema_request = Some(request_id);
        self.schema_error = None;
        self.runtime_message = if force_refresh {
            "Refreshing schema…"
        } else {
            "Loading schema…"
        }
        .to_owned();
    }
}
