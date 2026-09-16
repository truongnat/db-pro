use crate::components::*;
use crate::editor::PredictionMode;
use crate::tokens::*;
use crate::{
    agent_message_frame, badge, card_frame, compact_button, compact_button_with_icon, compact_icon_button,
    compact_icon_button_enabled, danger_button, editor_frame, empty_state, ghost_button_with_icon, grid_frame,
    icon_button, icon_text, input, input_full_width, menu_button_with_icon, panel_frame, primary_button,
    primary_button_with_icon, secondary_button, secondary_button_with_icon, section_label, sidebar_frame, sidebar_item,
    tab_frame, toolbar_frame, AgentContext, AgentMessage, AgentProvider, AgentRole, ColumnWriteBlock,
    ColumnWritePolicy, DbProTheme, GridProjectionCache, GridProjectionKey, OfflineAgentProvider, TaskBridge, UiCell,
    UiCommand, UiConnectionDraft, UiConnectionSummary, UiDriver, UiEvent, UiFunctionSummary, UiQueryExecutionOutput,
    UiQueryFolderSummary, UiQueryHistoryEntry, UiQueryHistoryStatus, UiQueryResult, UiSavedQuerySummary,
    UiSchemaForeignKey, UiSchemaSummary, UiSslMode, UiStatementOutput, UiTableDataFilter, UiTableDataSort,
    UiTableFilterOperator, UiTableInfo, UiTableMutation, UiTableSummary, UiTriggerSummary, UiViewSummary,
};
use bigdecimal::BigDecimal;
use eframe::egui::{self, Align, FontId, Layout, RichText, Sense, TextEdit, TopBottomPanel};
use lucide_icons::Icon;
use sqlparser::dialect::{GenericDialect, PostgreSqlDialect, SQLiteDialect};
use sqlparser::parser::Parser;
use std::collections::{BTreeSet, HashMap};
use std::time::{Duration, Instant};

use agent_workflow_state::AgentUiSession;
use change_set::{ChangeSet, MutationFailure, MutationTarget, RowIdentity, StagedChange};

#[path = "agent_state.rs"]
mod agent_state;
#[path = "agent_view.rs"]
mod agent_view;
#[path = "agent_workflow_state.rs"]
mod agent_workflow_state;
#[path = "app_state.rs"]
mod app_state;
#[path = "app_types.rs"]
mod app_types;
#[path = "change_set.rs"]
mod change_set;
#[path = "component_gallery_view.rs"]
mod component_gallery_view;
#[path = "connection_view.rs"]
mod connection_view;

pub use component_gallery_view::ComponentGalleryState;
#[path = "capability_lookup.rs"]
mod capability_lookup;
#[path = "diagram_view.rs"]
mod diagram_view;
#[path = "events.rs"]
mod events;
#[path = "events_query.rs"]
mod events_query;
#[path = "events_query_dispatch.rs"]
mod events_query_dispatch;
#[path = "explorer_connections.rs"]
mod explorer_connections;
#[path = "explorer_details.rs"]
mod explorer_details;
#[path = "explorer_folders.rs"]
mod explorer_folders;
#[path = "explorer_tree.rs"]
mod explorer_tree;
#[path = "explorer_view.rs"]
mod explorer_view;
#[path = "files_activity_view.rs"]
mod files_activity_view;
#[path = "ide_workspace.rs"]
mod ide_workspace;
#[path = "navigation_view.rs"]
mod navigation_view;
#[path = "settings_model.rs"]
mod settings_model;
#[path = "settings_view.rs"]
mod settings_view;
pub(crate) use capability_lookup::CapabilityLookup;
pub(crate) use settings_model::{default_keybinding_catalog, AppSettings, SettingsSection, SETTINGS_STORAGE_KEY};
#[path = "activity_bar_view.rs"]
mod activity_bar_view;
#[path = "connection_status.rs"]
mod connection_status;
#[path = "grid_layout.rs"]
mod grid_layout;
#[path = "palette_view.rs"]
mod palette_view;
#[path = "search_service.rs"]
mod search_service;
pub(crate) use search_service::{SearchFingerprintParts, SearchIndex, SearchService};
#[path = "problems_view.rs"]
mod problems_view;
#[path = "query_dialogs_view.rs"]
mod query_dialogs_view;
#[path = "query_documents.rs"]
mod query_documents;
#[path = "query_editor_panel.rs"]
mod query_editor_panel;
#[path = "query_output_view.rs"]
mod query_output_view;
#[path = "query_session.rs"]
mod query_session;
#[path = "query_view.rs"]
mod query_view;
#[path = "result_grid_cell.rs"]
mod result_grid_cell;
#[path = "result_grid_clipboard.rs"]
mod result_grid_clipboard;
#[path = "result_grid_edit.rs"]
mod result_grid_edit;
#[path = "result_grid_header.rs"]
mod result_grid_header;
#[path = "result_grid_selection.rs"]
mod result_grid_selection;
#[path = "result_grid_view.rs"]
pub(crate) mod result_grid_view;
#[path = "sidebar_activities_view.rs"]
mod sidebar_activities_view;
#[path = "sidebar_view.rs"]
mod sidebar_view;
#[path = "workspace_actions.rs"]
mod workspace_actions;
pub(crate) use result_grid_view::GridSelectionCache;
#[path = "schema_compare.rs"]
mod schema_compare;
#[path = "schema_object_view.rs"]
mod schema_object_view;
#[path = "schema_workbench.rs"]
mod schema_workbench;
#[path = "schema_workbench_form.rs"]
mod schema_workbench_form;
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
pub(crate) use app_types::*;

pub struct DbProApp {
    theme: DbProTheme,
    dark_mode: bool,
    reduce_motion: bool,
    settings: AppSettings,
    settings_section: SettingsSection,
    keybindings_filter: String,
    keybinding_edit_id: Option<String>,
    keybinding_edit_draft: String,
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
    problems_severity_filter: ProblemsSeverityFilter,
    problems_source_filter: ProblemsSourceFilter,
    problems_selected: Option<(String, usize)>,
    query_history: Vec<String>,
    query_history_entries: Vec<UiQueryHistoryEntry>,
    query_history_search: String,
    connection_name: String,
    connected: bool,
    palette_mode: Option<PaletteMode>,
    palette_query: String,
    palette_scope: SearchScope,
    palette_selected: usize,
    palette_focus_requested: bool,
    search_index: SearchIndex,
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
    /// Table names pinned for quick reopen (#202 / #212). Persisted locally.
    pinned_tables: Vec<String>,
    /// Most-recently-opened tables for Data Activity (#212). Persisted locally.
    recent_tables: Vec<String>,
    /// Local IDE workspace folder / file tree (#261–#264).
    ide_workspace: ide_workspace::IdeWorkspaceState,
    /// Find-in-Files / replace drafts for the Files activity (#267).
    workspace_search_query: String,
    workspace_replace_query: String,
    workspace_search_hits: Vec<ide_workspace::SearchHit>,
    workspace_replace_previews: Vec<ide_workspace::ReplacePreview>,
    workspace_task_command: String,
    workspace_refactor_from: String,
    workspace_refactor_to: String,
    workspace_context_items: Vec<String>,
    split_editor_secondary: Option<usize>,
    files_panel_tab: FilesPanelTab,
    selected_schema_object: Option<SchemaObjectSelection>,
    schema_object_view: SchemaObjectView,
    /// Editable CREATE body for the selected routine (#192).
    routine_source_draft: String,
    /// Values for IN/INOUT parameters in the execute form.
    routine_param_values: Vec<String>,
    routine_param_nulls: Vec<bool>,
    routine_ddl_preview: Option<String>,
    routine_drop_confirm: bool,
    /// Recent transfer jobs for Transfers activity (#193).
    transfer_jobs: Vec<db_pro_core::domain::transfer::TransferJob>,
    /// Latest monitoring snapshot for Monitor activity (#196).
    monitoring_snapshot: Option<db_pro_core::domain::monitoring::MonitoringSnapshot>,
    monitoring_error: Option<String>,
    monitoring_poll: bool,
    monitoring_last_poll: Option<std::time::Instant>,
    monitoring_terminate_confirm: Option<i64>,
    monitoring_filter_active_only: bool,
    monitoring_maintenance_confirm: Option<db_pro_core::domain::monitoring::MaintenanceAction>,
    security_users: Vec<db_pro_core::domain::user::DatabaseUser>,
    security_selected_role: Option<String>,
    security_privileges: Vec<db_pro_core::domain::user::Privilege>,
    security_memberships: Vec<db_pro_core::domain::user::RoleMembership>,
    security_new_role: String,
    security_new_role_login: bool,
    security_membership_role: String,
    security_password: String,
    security_grant_kind: db_pro_core::domain::user::PrivilegeObjectKind,
    security_grant_schema: String,
    security_grant_object: String,
    security_grant_privilege: String,
    security_rls_schema: String,
    security_rls_table: String,
    security_rls_state: Option<db_pro_core::domain::rls::TableRlsState>,
    security_rls_policy_name: String,
    security_rls_command: String,
    security_rls_roles: String,
    security_rls_using: String,
    security_rls_with_check: String,
    security_rls_preview_sql: String,
    security_rls_confirm_apply: bool,
    security_drop_confirm: Option<String>,
    security_error: Option<String>,
    schema_workbench: schema_workbench::SchemaWorkbenchState,
    schema_snapshot: Option<schema_compare::UiSchemaSnapshot>,
    schema_diff: Option<schema_compare::UiSchemaDiffResult>,
    query_auto_commit: bool,
    query_in_transaction: bool,
    query_txn_pending: usize,
    disconnect_txn_guard: bool,
    /// Transaction bar is opt-in so the SQL surface stays file-editor quiet by default.
    query_txn_bar_open: bool,
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
        // Match panel chrome so any sub-pixel seam between SidePanel and
        // CentralPanel cannot flash as a white strip (surface_app).
        self.theme.surface_panel.to_normalized_gamma_f32()
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.persist_current_grid_layout();
        self.sync_settings_from_runtime();
        if let Ok(settings) = serde_json::to_string(&self.settings) {
            storage.set_string(SETTINGS_STORAGE_KEY, settings);
        }
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
        if let Ok(pinned) = serde_json::to_string(&self.pinned_tables) {
            storage.set_string("dbpro.native.pinned-tables-v1", pinned);
        }
        if let Ok(recent) = serde_json::to_string(&self.recent_tables) {
            storage.set_string("dbpro.native.recent-tables-v1", recent);
        }
        if let Ok(recent_ws) = serde_json::to_string(
            &self
                .ide_workspace
                .recent_roots
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
        ) {
            storage.set_string("dbpro.native.workspace-recent-v1", recent_ws);
        }
        if let Ok(roots) = serde_json::to_string(
            &self
                .ide_workspace
                .roots
                .iter()
                .map(|root| root.path.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
        ) {
            storage.set_string("dbpro.native.workspace-roots-v1", roots);
        }
        storage.set_string(
            "dbpro.native.workspace-trusted-v1",
            self.ide_workspace.is_trusted().to_string(),
        );
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
            .frame(egui::Frame {
                // Match sidebar fill; inset content so the workspace isn't flush
                // against the sidebar divider or the window/agent edge.
                fill: self.theme.surface_panel,
                inner_margin: egui::Margin::symmetric(SPACE_MD, 0.0),
                outer_margin: egui::Margin::ZERO,
                stroke: egui::Stroke::NONE,
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
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

// CapabilityLookup lives in `capability_lookup.rs`.

impl DbProApp {
    // Grid layout: `grid_layout.rs`.

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

    // Connection/status: `connection_status.rs`.

    // Query session helpers: `query_session.rs`.

    // Schema snapshot / query result / txn: `query_session.rs`.
    // Close workspace tab: `workspace_actions.rs`.

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
