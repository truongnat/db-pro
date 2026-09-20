use self::connection::ConnectionFeatureState;
pub(crate) use self::connection::{ConnectionCatalogState, ConnectionDialogState, ConnectionLifecycleState};
use crate::components::*;
use crate::editor::PredictionMode;
use crate::query::SchemaSymbolIndex;
use crate::tokens::*;
use crate::{
    agent_message_frame, badge, card_frame, compact_button, compact_button_with_icon, compact_icon_button,
    danger_button, editor_frame, empty_state, ghost_button_with_icon, grid_frame, icon_button, icon_text, input,
    input_full_width, menu_button_with_icon, panel_frame, primary_button, primary_button_with_icon, secondary_button,
    secondary_button_with_icon, section_label, sidebar_frame, sidebar_item, tab_frame, toolbar_frame, AgentContext,
    AgentMessage, AgentProvider, AgentRole, ColumnWriteBlock, ColumnWritePolicy, DbProTheme, GridProjectionCache,
    GridProjectionKey, OfflineAgentProvider, RequestId, TaskBridge, UiCell, UiCommand, UiConnectionDraft,
    UiConnectionSummary, UiDriver, UiEvent, UiFunctionSummary, UiQueryExecutionOutput, UiQueryFolderSummary,
    UiQueryHistoryEntry, UiQueryHistoryStatus, UiQueryResult, UiSavedQuerySummary, UiSchemaForeignKey, UiSchemaSummary,
    UiStatementOutput, UiTableDataFilter, UiTableDataSort, UiTableFilterOperator, UiTableInfo, UiTableMutation,
    UiTableSummary, UiTriggerSummary, UiViewSummary,
};
use eframe::egui::{self, Align, FontId, Layout, RichText, Sense, TextEdit, TopBottomPanel};
use lucide_icons::Icon;
use std::collections::{BTreeSet, HashMap};
use std::time::{Duration, Instant};

use agent_workflow_state::AgentUiSession;
use change_set::{ChangeSet, MutationFailure, MutationTarget, RowIdentity, StagedChange};

#[path = "agent_context.rs"]
mod agent_context;
#[path = "agent_events.rs"]
mod agent_events;
#[path = "agent_state.rs"]
mod agent_state;
#[path = "agent_thread_view.rs"]
mod agent_thread_view;
#[path = "agent_view.rs"]
mod agent_view;
#[path = "agent_workflow_state.rs"]
mod agent_workflow_state;
#[path = "app_state.rs"]
mod app_state;
#[path = "app_types.rs"]
mod app_types;
#[path = "audit_state.rs"]
mod audit_state;
#[path = "cell_inspector.rs"]
mod cell_inspector;
#[path = "change_set.rs"]
mod change_set;
#[path = "component_gallery_agent.rs"]
mod component_gallery_agent;
#[path = "component_gallery_feedback.rs"]
mod component_gallery_feedback;
#[path = "component_gallery_inputs.rs"]
mod component_gallery_inputs;
#[path = "component_gallery_overlays.rs"]
mod component_gallery_overlays;
#[path = "component_gallery_surfaces.rs"]
mod component_gallery_surfaces;
#[path = "component_gallery_view.rs"]
mod component_gallery_view;
#[path = "connection/mod.rs"]
pub mod connection;

pub use component_gallery_view::ComponentGalleryState;
#[path = "audit_activity_view.rs"]
mod audit_activity_view;
#[path = "capability_lookup.rs"]
mod capability_lookup;
#[path = "database_management_state.rs"]
mod database_management_state;
#[path = "ddl_events.rs"]
mod ddl_events;
#[path = "diagram_canvas_view.rs"]
mod diagram_canvas_view;
#[path = "diagram_design_actions.rs"]
mod diagram_design_actions;
#[path = "diagram_design_panel_view.rs"]
mod diagram_design_panel_view;
#[path = "diagram_state.rs"]
mod diagram_state;
#[path = "diagram_view.rs"]
mod diagram_view;
#[path = "event_router.rs"]
mod event_router;
#[path = "event_trigger_activity_view.rs"]
mod event_trigger_activity_view;
#[path = "event_trigger_state.rs"]
mod event_trigger_state;
#[path = "events.rs"]
mod events;
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
#[path = "fdw_activity_view.rs"]
mod fdw_activity_view;
#[path = "fdw_state.rs"]
mod fdw_state;
#[path = "feedback_state.rs"]
mod feedback_state;
#[path = "file_picker_events.rs"]
mod file_picker_events;
#[path = "files_activity_tabs.rs"]
mod files_activity_tabs;
#[path = "files_activity_view.rs"]
mod files_activity_view;
#[path = "git_workspace.rs"]
mod git_workspace;
#[path = "ide_workspace.rs"]
mod ide_workspace;
#[path = "ide_workspace_scan.rs"]
mod ide_workspace_scan;
#[path = "ide_workspace_types.rs"]
mod ide_workspace_types;
#[path = "maintenance_activity_view.rs"]
mod maintenance_activity_view;
#[path = "management_events.rs"]
mod management_events;
#[path = "masking.rs"]
mod masking;
#[path = "masking_state.rs"]
mod masking_state;
#[path = "monitoring_activity_view.rs"]
mod monitoring_activity_view;
#[path = "monitoring_state.rs"]
mod monitoring_state;
#[path = "navigation_view.rs"]
mod navigation_view;
#[path = "operation_events.rs"]
mod operation_events;
#[path = "overlay_state.rs"]
mod overlay_state;
#[path = "palette_actions.rs"]
mod palette_actions;
#[path = "palette_catalog.rs"]
mod palette_catalog;
#[path = "palette_state.rs"]
mod palette_state;
#[path = "pg_settings_activity_view.rs"]
mod pg_settings_activity_view;
#[path = "pg_settings_state.rs"]
mod pg_settings_state;
#[path = "preferences_state.rs"]
mod preferences_state;
#[path = "query_execution_events.rs"]
mod query_execution_events;
#[path = "query_execution_state.rs"]
mod query_execution_state;
#[path = "query_failure_events.rs"]
mod query_failure_events;
#[path = "query_history_events.rs"]
mod query_history_events;
#[path = "query_library_events.rs"]
mod query_library_events;
#[path = "query_library_state.rs"]
mod query_library_state;
#[path = "query_multi_result_events.rs"]
mod query_multi_result_events;
#[path = "query_prediction_events.rs"]
mod query_prediction_events;
#[path = "query_queue_events.rs"]
mod query_queue_events;
#[path = "query_result_events.rs"]
mod query_result_events;
#[path = "query_save_events.rs"]
mod query_save_events;
#[path = "replication_activity_view.rs"]
mod replication_activity_view;
#[path = "replication_state.rs"]
mod replication_state;
#[path = "routine_state.rs"]
mod routine_state;
#[path = "saved_task_state.rs"]
mod saved_task_state;
#[path = "security_state.rs"]
mod security_state;
#[path = "settings_model.rs"]
mod settings_model;
#[path = "settings_view.rs"]
mod settings_view;
#[path = "synthetic_data_state.rs"]
mod synthetic_data_state;
#[path = "transfer_harness_view.rs"]
mod transfer_harness_view;
#[path = "transfer_state.rs"]
mod transfer_state;
use audit_state::AuditState;
pub(crate) use capability_lookup::CapabilityLookup;
use database_management_state::DatabaseManagementState;
pub(crate) use diagram_state::DiagramState;
use event_trigger_state::EventTriggerState;
use fdw_state::FdwState;
pub(crate) use feedback_state::FeedbackState;
use masking_state::MaskingState;
use monitoring_state::MonitoringState;
pub(crate) use overlay_state::OverlayState;
pub(crate) use palette_state::PaletteState;
use pg_settings_state::PgSettingsState;
pub(crate) use preferences_state::PreferencesState;
pub(crate) use query_execution_state::QueryExecutionPolicyState;
pub(crate) use query_library_state::QueryLibraryState;
use replication_state::ReplicationState;
use routine_state::RoutineState;
pub(crate) use saved_task_state::SavedTaskState;
use security_state::SecurityState;
pub(crate) use settings_model::{
    default_keybinding_catalog, AppSettings, SettingsSection, SqlLintSettings, SETTINGS_STORAGE_KEY,
};
use synthetic_data_state::SyntheticDataState;
use transfer_state::TransferState;
pub(crate) use workspace_shell::WorkspaceShellState;
#[path = "activity_bar_view.rs"]
mod activity_bar_view;
#[path = "connection_events.rs"]
mod connection_events;
#[path = "connection_status.rs"]
mod connection_status;
#[path = "palette_view.rs"]
mod palette_view;
#[path = "search_service.rs"]
mod search_service;
pub(crate) use search_service::{SearchFingerprintParts, SearchIndex, SearchService};
#[path = "problems_view.rs"]
mod problems_view;
#[path = "query_actions_view.rs"]
mod query_actions_view;
#[path = "query_diagnostics_view.rs"]
mod query_diagnostics_view;
#[path = "query_dialogs_view.rs"]
mod query_dialogs_view;
#[path = "query_documents.rs"]
mod query_documents;
#[path = "query_editor_panel.rs"]
mod query_editor_panel;
#[path = "query_editor_state.rs"]
mod query_editor_state;
#[path = "query_editor_support.rs"]
mod query_editor_support;
#[path = "query_feature_state.rs"]
mod query_feature_state;
#[path = "query_folder_delete_dialog.rs"]
mod query_folder_delete_dialog;
#[path = "query_output_actions_view.rs"]
mod query_output_actions_view;
#[path = "query_output_panes_view.rs"]
mod query_output_panes_view;
#[path = "query_output_state.rs"]
mod query_output_state;
#[path = "query_output_tabs_view.rs"]
mod query_output_tabs_view;
#[path = "query_output_view.rs"]
mod query_output_view;
#[path = "query_results_pane_view.rs"]
mod query_results_pane_view;
#[path = "query_search_view.rs"]
mod query_search_view;
#[path = "query_session.rs"]
mod query_session;
#[path = "query_snippets.rs"]
mod query_snippets;
#[path = "query_state.rs"]
mod query_state;
#[path = "query_view.rs"]
mod query_view;
#[path = "result_grid_cell.rs"]
mod result_grid_cell;
#[path = "result_grid_clipboard.rs"]
mod result_grid_clipboard;
#[path = "result_grid_edit.rs"]
mod result_grid_edit;
#[path = "result_grid_export.rs"]
mod result_grid_export;
#[path = "result_grid_header.rs"]
mod result_grid_header;
#[path = "result_grid_selection.rs"]
mod result_grid_selection;
#[path = "result_grid_view.rs"]
pub(crate) mod result_grid_view;
#[path = "runtime_event_handlers.rs"]
mod runtime_event_handlers;
#[path = "sidebar_activities_view.rs"]
mod sidebar_activities_view;
#[path = "sidebar_view.rs"]
mod sidebar_view;
#[path = "synthetic_data.rs"]
mod synthetic_data;
#[path = "table_data_query_state.rs"]
mod table_data_query_state;
#[path = "table_data_state.rs"]
mod table_data_state;
#[path = "table_data_view.rs"]
mod table_data_view;
#[path = "table_editing_state.rs"]
mod table_editing_state;
#[path = "table_editor_state.rs"]
mod table_editor_state;
#[path = "table_events.rs"]
mod table_events;
#[path = "table_mutation_actions.rs"]
mod table_mutation_actions;
#[path = "table_mutation_dialogs_view.rs"]
mod table_mutation_dialogs_view;
#[path = "table_mutation_state.rs"]
mod table_mutation_state;
#[path = "table_state.rs"]
mod table_state;
#[path = "tasks_view.rs"]
mod tasks_view;
#[path = "visual_query_builder_state.rs"]
mod visual_query_builder_state;
#[path = "visual_query_builder_view.rs"]
mod visual_query_builder_view;
#[path = "welcome_state.rs"]
mod welcome_state;
#[path = "workspace_actions.rs"]
mod workspace_actions;
#[path = "workspace_files_state.rs"]
mod workspace_files_state;
#[path = "workspace_session.rs"]
mod workspace_session;
#[path = "workspace_session_state.rs"]
mod workspace_session_state;
#[path = "workspace_shell.rs"]
mod workspace_shell;
pub(crate) use agent_state::AgentState;
pub(crate) use query_editor_state::QueryEditorState;
pub(crate) use query_feature_state::QueryFeatureState;
pub(crate) use query_output_state::QueryOutputState;
pub(crate) use query_state::QuerySessionState;
pub(crate) use result_grid_view::GridSelectionCache;
use schema_compare_state::SchemaCompareState;
pub(crate) use schema_explorer_state::SchemaExplorerState;
use schema_workspace_state::SchemaWorkspaceState;
pub(crate) use table_data_query_state::TableDataQueryState;
pub(crate) use table_data_state::TableDataState;
pub(crate) use table_editing_state::TableEditingState;
pub(crate) use table_editor_state::TableEditorState;
pub(crate) use table_mutation_actions::StagedApplyFailure;
pub(crate) use table_mutation_state::TableMutationState;
pub(crate) use table_state::TableState;
pub(crate) use welcome_state::WelcomeState;
pub(crate) use workspace_feature_state::WorkspaceFeatureState;
pub(crate) use workspace_files_state::WorkspaceFilesState;
pub(crate) use workspace_session_state::WorkspaceSessionState;
#[path = "schema_compare.rs"]
mod schema_compare;
#[path = "schema_compare_state.rs"]
mod schema_compare_state;
#[path = "schema_compare_view.rs"]
mod schema_compare_view;
#[path = "schema_events.rs"]
mod schema_events;
#[path = "schema_explorer_state.rs"]
mod schema_explorer_state;
#[path = "schema_object_view.rs"]
mod schema_object_view;
#[path = "schema_workbench.rs"]
mod schema_workbench;
#[path = "schema_workbench_actions.rs"]
mod schema_workbench_actions;
#[path = "schema_workbench_form.rs"]
mod schema_workbench_form;
#[path = "schema_workspace_state.rs"]
mod schema_workspace_state;
#[path = "security_activity_view.rs"]
mod security_activity_view;
#[path = "security_rls.rs"]
mod security_rls;
#[path = "shell_chrome_view.rs"]
mod shell_chrome_view;
#[path = "table_ddl_view.rs"]
mod table_ddl_view;
#[path = "table_editor_context.rs"]
mod table_editor_context;
#[path = "table_editor_values.rs"]
mod table_editor_values;
#[path = "table_editor_view.rs"]
mod table_editor_view;
#[path = "table_insert_row_view.rs"]
mod table_insert_row_view;
#[path = "table_metadata_view.rs"]
mod table_metadata_view;
#[path = "table_relations_view.rs"]
mod table_relations_view;
#[path = "table_structure_view.rs"]
mod table_structure_view;
#[path = "table_view.rs"]
mod table_view;
#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
#[path = "transfer_activity_view.rs"]
mod transfer_activity_view;
#[path = "welcome_view.rs"]
mod welcome_view;
#[path = "workspace_feature_state.rs"]
mod workspace_feature_state;
#[path = "workspace_tab_primitives.rs"]
mod workspace_tab_primitives;
#[path = "workspace_tabs_view.rs"]
mod workspace_tabs_view;
#[path = "workspace_view.rs"]
mod workspace_view;

pub use crate::query::{QueryDocument, QueryExecutionState};
pub(crate) use app_types::*;
#[path = "app_lifecycle.rs"]
mod app_lifecycle;
#[path = "app_storage.rs"]
mod app_storage;

pub struct DbProApp {
    theme: DbProTheme,
    preferences: PreferencesState,
    workspace: WorkspaceFeatureState,
    welcome: WelcomeState,
    query: QueryFeatureState,
    palette: PaletteState,
    agent: AgentState,
    task_bridge: TaskBridge,
    feedback: FeedbackState,
    table: TableEditorState,
    overlay: OverlayState,
    connection: ConnectionFeatureState,
    schema: SchemaWorkspaceState,
    management: DatabaseManagementState,
    saved_tasks: SavedTaskState,
    /// Counter for initial render frames to ensure window is maximized on startup.
    initial_frames_count: u8,
    gallery_state: component_gallery_view::ComponentGalleryState,
}

// CapabilityLookup lives in `capability_lookup.rs`.

impl DbProApp {
    /// Apply a driver choice from the connection dialog.
    pub fn select_connection_driver(&mut self, driver: UiDriver) {
        connection::select_connection_driver(self.connection.dialog.draft_mut(), driver);
    }

    /// Open connection edit dialog from a saved summary.
    pub fn open_edit_connection(&mut self, connection: &UiConnectionSummary) {
        connection::open_edit_connection(&mut self.connection.dialog, &mut self.connection.lifecycle, connection);
    }

    /// Open a duplicate connection draft from a saved summary.
    pub fn open_duplicate_connection(&mut self, connection: &UiConnectionSummary) {
        connection::open_duplicate_connection(&mut self.connection.dialog, &mut self.connection.lifecycle, connection);
    }

    /// Save the active draft's SSH parameters as a reusable profile.
    pub fn save_draft_as_ssh_profile(&mut self) {
        match connection::save_draft_as_ssh_profile(&mut self.connection.dialog) {
            Ok(id) => {
                self.connection.dialog.draft_mut().ssh_profile_id = id;
                self.feedback
                    .set_runtime_message("SSH profile saved — reusable by other connections");
            }
            Err(err) => self.feedback.set_runtime_message(err),
        }
    }

    /// Apply an existing SSH profile to the active draft.
    pub fn apply_ssh_profile(&mut self, profile_id: &str) {
        connection::apply_ssh_profile(&mut self.connection.dialog, profile_id);
    }

    /// Apply the selected cloud preset to the active draft.
    pub fn apply_cloud_preset(&mut self) {
        if let Err(err) = connection::apply_cloud_preset(&mut self.connection.dialog) {
            self.connection.dialog.set_error(err);
        }
    }

    /// Dispatch connection test or save command to the runtime worker.
    pub fn dispatch_connection_command(&mut self, save: bool) {
        connection::view::ConnectionDialogView {
            dialog: &mut self.connection.dialog,
            lifecycle: &mut self.connection.lifecycle,
            task_bridge: &mut self.task_bridge,
            feedback: &mut self.feedback,
            theme: self.theme,
        }
        .dispatch_connection_command(save);
    }

    /// Refresh the diagnostic report for the active draft.
    pub fn refresh_connection_diagnostics(&mut self, auth_ok: bool, auth_message: &str) {
        connection::refresh_connection_diagnostics(&mut self.connection.dialog, auth_ok, auth_message);
    }

    pub(super) fn handle_connection_request_failure(&mut self, request_id: RequestId, message: &str) -> bool {
        connection_events::handle_connection_request_failure(
            &mut self.connection.lifecycle,
            &mut self.connection.dialog,
            &mut self.schema.explorer,
            &mut self.feedback,
            request_id,
            message,
        )
    }

    pub(super) fn on_connections_loaded(&mut self, connections: Vec<UiConnectionSummary>) {
        let active = connection_events::on_connections_loaded(
            &mut self.connection.lifecycle,
            &mut self.connection.catalog,
            &mut self.feedback,
            connections,
        );
        if let Some(active) = active {
            self.connect_to_connection(&active);
        }
    }

    pub(super) fn on_connected(&mut self, request_id: RequestId, connection_id: String) {
        let Some(connection_id) = connection_events::on_connected(
            &mut self.connection.lifecycle,
            &mut self.feedback,
            request_id,
            connection_id,
        ) else {
            return;
        };
        self.request_schema_introspection(connection_id.clone(), false);
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            self.query
                .library
                .list_queries_command(request_id, connection_id.clone()),
        );
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.query.library.list_folders_command(request_id, connection_id));
    }

    // Connection read models and shell status are implemented as explicit pure helpers
    // in `connection_status.rs`; these root methods preserve the app's internal API while
    // keeping that feature module independent from the composition root.
    pub(super) fn active_connection(&self) -> Option<&UiConnectionSummary> {
        connection_status::active_connection(&self.connection.catalog, &self.connection.lifecycle)
    }

    pub(super) fn active_connection_name(&self) -> &str {
        connection_status::active_connection_name(&self.connection.catalog, &self.connection.lifecycle)
    }

    pub(super) fn active_driver(&self) -> &str {
        connection_status::active_driver(&self.connection.catalog, &self.connection.lifecycle)
    }

    pub(crate) fn active_capabilities(&self) -> CapabilityLookup {
        connection_status::active_capabilities(&self.connection.catalog, &self.connection.lifecycle)
    }

    pub(super) fn active_schema(&self) -> &str {
        connection_status::active_schema(
            &self.schema.explorer,
            &self.connection.catalog,
            &self.connection.lifecycle,
        )
    }

    pub(super) fn active_schema_table_names(&self) -> Vec<String> {
        connection_status::active_schema_table_names(
            &self.schema.explorer,
            &self.connection.catalog,
            &self.connection.lifecycle,
        )
    }

    pub(super) fn schema_table_names(&self, schema: &str) -> Vec<String> {
        connection_status::schema_table_names(&self.schema.explorer, schema)
    }

    pub(super) fn schema_table_count(&self, schema: &str) -> usize {
        connection_status::schema_table_count(&self.schema.explorer, schema)
    }

    pub(super) fn schema_matching_table_count(&self, schema: &str, query: &str) -> usize {
        connection_status::schema_matching_table_count(&self.schema.explorer, schema, query)
    }

    pub(super) fn active_schema_column_names(&self) -> Vec<String> {
        connection_status::active_schema_column_names(
            &self.schema.explorer,
            &self.connection.catalog,
            &self.connection.lifecycle,
        )
    }

    pub(super) fn has_runtime_error(&self) -> bool {
        connection_status::has_runtime_error(&self.feedback)
    }

    pub(super) fn runtime_status(&self) -> Option<(String, egui::Color32)> {
        connection_status::runtime_status(&self.feedback, self.theme)
    }

    pub(super) fn statusbar_state(&self) -> (Icon, egui::Color32, &'static str) {
        connection_status::statusbar_state(&self.connection.lifecycle, &self.feedback, self.theme)
    }

    pub(super) fn shows_editor_status(&self) -> bool {
        connection_status::shows_editor_status()
    }

    pub(super) fn statusbar_context_label(&self) -> &'static str {
        connection_status::statusbar_context_label(&self.workspace, &self.table.state)
    }

    pub(super) fn connection_indicator(&self, connection: &UiConnectionSummary) -> (Icon, egui::Color32) {
        connection_status::connection_indicator(&self.connection.lifecycle, connection, self.theme)
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

    /// Platform-aware shortcut parts for kbd chips, e.g. `["Ctrl", "N"]` / `["⌘", "N"]`.
    fn shortcut_parts(keys: &[&str]) -> Vec<String> {
        let mut parts = Vec::with_capacity(keys.len() + 1);
        parts.push(Self::primary_modifier_label().to_owned());
        for key in keys {
            let mapped = match *key {
                "Shift" if cfg!(target_os = "macos") => "⇧",
                other => other,
            };
            parts.push(mapped.to_owned());
        }
        parts
    }

    /// Human-readable shortcut for tooltips: `Ctrl+N` on Linux/Windows, `⌘N` on macOS.
    fn format_shortcut(keys: &[&str]) -> String {
        let parts = Self::shortcut_parts(keys);
        if cfg!(target_os = "macos") {
            parts.concat()
        } else {
            parts.join("+")
        }
    }

    /// Queues a command and exposes a closed runtime boundary to the user.
    pub(crate) fn dispatch_command(&mut self, command: UiCommand) -> bool {
        if self.send_command_best_effort(command) {
            return true;
        }
        let message = "Runtime worker unavailable";
        self.feedback.runtime_message = message.to_owned();
        self.feedback.show_error_toast(message);
        false
    }

    /// Sends a cancellation/background command without borrowing the whole app.
    ///
    /// These calls are intentionally best-effort because their authoritative
    /// guards are request IDs and document versions; a closed worker cannot
    /// execute the cancellation, but it also cannot mutate UI state anymore.
    pub(crate) fn send_command_best_effort(&self, command: UiCommand) -> bool {
        self.task_bridge.send_best_effort(command)
    }

    // Connection/status: `connection_status.rs`.

    // Query session helpers: `query_session.rs`.

    // Schema snapshot / query result / txn: `query_session.rs`.
    // Close workspace tab: `workspace_actions.rs`.

    pub(crate) fn set_agent_open(&mut self, open: bool, ctx: &egui::Context) {
        if open == self.workspace.agent_open {
            return;
        }
        self.workspace.agent_open = open;
        if open {
            self.workspace.sidebar_open_before_agent = Some(self.workspace.sidebar_open);
            if ctx.screen_rect().width() < AGENT_SIDEBAR_COLLAPSE_WIDTH {
                self.workspace.sidebar_open = false;
            }
        } else if let Some(sidebar_open) = self.workspace.sidebar_open_before_agent.take() {
            self.workspace.sidebar_open = sidebar_open;
        }
    }

    fn workspace_session_context(&mut self) -> workspace_session::WorkspaceSessionContext<'_> {
        workspace_session::WorkspaceSessionContext {
            workspace: &mut self.workspace,
            connection: &mut self.connection,
            schema_explorer: &mut self.schema.explorer,
            query_session_state: &mut self.query.session,
            feedback: &mut self.feedback,
            preferences: &self.preferences,
        }
    }

    pub(crate) fn save_named_workspace_session(&mut self) {
        self.workspace_session_context().save_named();
    }

    pub(crate) fn restore_named_workspace_session(&mut self, id: &str) {
        self.workspace_session_context().restore_named(id);
    }

    pub(crate) fn duplicate_named_workspace_session(&mut self, id: &str) {
        self.workspace_session_context().duplicate_named(id);
    }

    pub(crate) fn persist_workspace_sessions(&mut self, storage: &mut dyn eframe::Storage) {
        self.workspace_session_context().persist(storage);
    }

    pub(crate) fn load_named_sessions_from_storage(&mut self, storage: &dyn eframe::Storage) {
        self.workspace_session_context().load_named_sessions(storage);
    }

    pub(crate) fn restore_last_workspace_session_from_storage(&mut self, storage: &dyn eframe::Storage) {
        self.workspace_session_context().restore_last(storage);
    }

    pub(crate) fn open_agent_prompt(&mut self, prompt: impl Into<String>, ctx: &egui::Context) {
        self.agent.input = prompt.into();
        self.set_agent_open(true, ctx);
    }

    fn request_connections_once(&mut self) {
        if self.connection.lifecycle.connections_requested() {
            return;
        }
        self.connection.lifecycle.mark_connections_requested();
        self.connection.lifecycle.set_connections_request_pending(true);
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ListConnections { request_id });
    }

    fn runtime_work_pending(&self) -> bool {
        self.connection.lifecycle.connections_request_pending()
            || self.connection.lifecycle.pending_request().is_some()
            || self.schema.explorer.schema_request.is_some()
            || self
                .query
                .session
                .documents
                .iter()
                .any(|d| d.pending_prediction_request.is_some() || d.prediction_debounce_deadline.is_some())
            || self.query.session.documents.iter().any(|d| d.explain_request.is_some())
            || self
                .agent
                .sessions
                .values()
                .any(|session| session.request_id.is_some() || session.active_run_id.is_some())
            || self.table.state.table_info_request.is_some()
            || self.table.state.table_ddl_request.is_some()
            || self.table.data_query.request.is_some()
            || self.table.mutation.table_mutation_request.is_some()
            || self.table.mutation.staged_apply_request.is_some()
            || self.table.state.ddl_execution_request.is_some()
    }

    fn request_schema_introspection(&mut self, connection_id: String, force_refresh: bool) {
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::IntrospectSchema {
            request_id,
            connection_id,
            force_refresh,
        });
        self.schema.explorer.schema_request = Some(request_id);
        self.schema.explorer.schema_error = None;
        self.feedback.runtime_message = if force_refresh {
            "Refreshing schema…"
        } else {
            "Loading schema…"
        }
        .to_owned();
    }
}
