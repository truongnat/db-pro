use self::connection::{ConnectionCatalogState, ConnectionDialogState, ConnectionLifecycleState};
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
    GridProjectionKey, OfflineAgentProvider, TaskBridge, UiCell, UiCommand, UiConnectionDraft, UiConnectionSummary,
    UiEvent, UiFunctionSummary, UiQueryExecutionOutput, UiQueryHistoryEntry, UiQueryHistoryStatus, UiQueryResult,
    UiSavedQuerySummary, UiSchemaForeignKey, UiSchemaSummary, UiStatementOutput, UiTableDataFilter, UiTableDataSort,
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

#[path = "agent_events.rs"]
mod agent_events;
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
#[path = "capability_lookup.rs"]
mod capability_lookup;
#[path = "database_feature_states.rs"]
mod database_feature_states;
#[path = "diagram_state.rs"]
mod diagram_state;
#[path = "diagram_view.rs"]
mod diagram_view;
#[path = "event_router.rs"]
mod event_router;
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
#[path = "feedback_state.rs"]
mod feedback_state;
#[path = "files_activity_view.rs"]
mod files_activity_view;
#[path = "git_workspace.rs"]
mod git_workspace;
#[path = "ide_workspace.rs"]
mod ide_workspace;
#[path = "navigation_view.rs"]
mod navigation_view;
#[path = "operation_events.rs"]
mod operation_events;
#[path = "overlay_state.rs"]
mod overlay_state;
#[path = "palette_state.rs"]
mod palette_state;
#[path = "preferences_state.rs"]
mod preferences_state;
#[path = "query_execution_state.rs"]
mod query_execution_state;
#[path = "query_library_state.rs"]
mod query_library_state;
#[path = "saved_task_state.rs"]
mod saved_task_state;
#[path = "settings_model.rs"]
mod settings_model;
#[path = "settings_view.rs"]
mod settings_view;
pub(crate) use capability_lookup::CapabilityLookup;
use database_feature_states::{
    AuditState, EventTriggerState, FdwState, MaskingState, MonitoringState, PgSettingsState, ReplicationState,
    RoutineState, SecurityState, SyntheticDataState, TransferState,
};
pub(crate) use diagram_state::DiagramState;
pub(crate) use feedback_state::FeedbackState;
pub(crate) use overlay_state::OverlayState;
pub(crate) use palette_state::PaletteState;
pub(crate) use preferences_state::PreferencesState;
pub(crate) use query_execution_state::QueryExecutionPolicyState;
pub(crate) use query_library_state::QueryLibraryState;
pub(crate) use saved_task_state::SavedTaskState;
pub(crate) use settings_model::{
    default_keybinding_catalog, AppSettings, SettingsSection, SqlLintSettings, SETTINGS_STORAGE_KEY,
};
pub(crate) use workspace_shell::WorkspaceShellState;
#[path = "activity_bar_view.rs"]
mod activity_bar_view;
#[path = "connection_events.rs"]
mod connection_events;
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
#[path = "query_editor_state.rs"]
mod query_editor_state;
#[path = "query_output_state.rs"]
mod query_output_state;
#[path = "query_output_view.rs"]
mod query_output_view;
#[path = "query_session.rs"]
mod query_session;
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
#[path = "table_data_state.rs"]
mod table_data_state;
#[path = "table_events.rs"]
mod table_events;
#[path = "table_mutation_state.rs"]
mod table_mutation_state;
#[path = "table_state.rs"]
mod table_state;
#[path = "tasks_view.rs"]
mod tasks_view;
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
pub(crate) use query_output_state::QueryOutputState;
pub(crate) use query_state::QuerySessionState;
pub(crate) use result_grid_view::GridSelectionCache;
use schema_compare_state::SchemaCompareState;
pub(crate) use schema_explorer_state::SchemaExplorerState;
pub(crate) use table_data_state::TableDataState;
pub(crate) use table_mutation_state::TableMutationState;
pub(crate) use table_state::TableState;
pub(crate) use welcome_state::WelcomeState;
pub(crate) use workspace_files_state::WorkspaceFilesState;
pub(crate) use workspace_session_state::WorkspaceSessionState;
#[path = "schema_compare.rs"]
mod schema_compare;
#[path = "schema_compare_state.rs"]
mod schema_compare_state;
#[path = "schema_events.rs"]
mod schema_events;
#[path = "schema_explorer_state.rs"]
mod schema_explorer_state;
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
#[path = "welcome_view.rs"]
mod welcome_view;
#[path = "workspace_view.rs"]
mod workspace_view;

pub use crate::query::{QueryDocument, QueryExecutionState};
pub(crate) use app_types::*;

pub struct DbProApp {
    theme: DbProTheme,
    preferences: PreferencesState,
    workspace: WorkspaceShellState,
    welcome: WelcomeState,
    query_session_state: QuerySessionState,
    query_editor: QueryEditorState,
    palette: PaletteState,
    agent: AgentState,
    task_bridge: TaskBridge,
    feedback: FeedbackState,
    query_output_state: QueryOutputState,
    table_data: TableDataState,
    overlay: OverlayState,
    connection_catalog: ConnectionCatalogState,
    query_library: QueryLibraryState,
    schema_explorer: SchemaExplorerState,
    workspace_files: WorkspaceFilesState,
    audit: AuditState,
    event_trigger: EventTriggerState,
    fdw: FdwState,
    masking: MaskingState,
    monitoring: MonitoringState,
    pg_settings: PgSettingsState,
    replication: ReplicationState,
    routine: RoutineState,
    security: SecurityState,
    synthetic_data: SyntheticDataState,
    transfer: TransferState,
    schema_workbench: schema_workbench::SchemaWorkbenchState,
    schema_compare: SchemaCompareState,
    query_execution: QueryExecutionPolicyState,
    saved_tasks: SavedTaskState,
    workspace_sessions: WorkspaceSessionState,
    diagram: DiagramState,
    table_state: TableState,
    table_mutation: TableMutationState,
    connection_lifecycle: ConnectionLifecycleState,
    connection_dialog: ConnectionDialogState,
    /// Counter for initial render frames to ensure window is maximized on startup.
    initial_frames_count: u8,
    gallery_state: component_gallery_view::ComponentGalleryState,
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
        if let Ok(settings) = serde_json::to_string(&self.preferences.settings) {
            storage.set_string(SETTINGS_STORAGE_KEY, settings);
        }
        self.persist_saved_tasks(storage);
        self.persist_workspace_sessions(storage);
        if let Ok(raw) = serde_json::to_string(self.connection_dialog.ssh_profiles()) {
            storage.set_string("dbpro.native.ssh-profiles-v1", raw);
        }
        if let Ok(layouts) = serde_json::to_string(&self.table_data.grid_layout_preferences) {
            storage.set_string("dbpro.native.grid-layouts", layouts);
        }
        if let Ok(widths) = serde_json::to_string(&self.table_data.grid_column_widths) {
            storage.set_string("dbpro.native.grid-widths", widths);
        }
        storage.set_string(
            "dbpro.native.grid-widths-customized",
            self.table_data.grid_columns_user_resized.to_string(),
        );
        if let Ok(documents) = serde_json::to_string(&self.query_session_state.documents) {
            storage.set_string("dbpro.native.query-documents", documents);
        }
        if let Ok(history) = serde_json::to_string(&self.query_editor.query_history_entries) {
            storage.set_string("dbpro.native.query-history-v1", history);
        }
        if let Ok(pinned) = serde_json::to_string(&self.schema_explorer.pinned_tables) {
            storage.set_string("dbpro.native.pinned-tables-v1", pinned);
        }
        if let Ok(recent) = serde_json::to_string(&self.schema_explorer.recent_tables) {
            storage.set_string("dbpro.native.recent-tables-v1", recent);
        }
        if let Ok(recent_ws) = serde_json::to_string(
            &self
                .workspace_files
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
                .workspace_files
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
            self.workspace_files.ide_workspace.is_trusted().to_string(),
        );
        storage.set_string("dbpro.native.theme-version", "light-first-v1".to_owned());
        storage.set_string("dbpro.native.dark-mode", self.preferences.dark_mode.to_string());
        storage.set_string("dbpro.native.reduce-motion", self.preferences.reduce_motion.to_string());
        if let Ok(prediction_mode) = serde_json::to_string(&self.preferences.prediction_mode) {
            storage.set_string("dbpro.native.prediction-mode", prediction_mode);
        }
        storage.set_string("dbpro.native.sidebar-width", self.workspace.sidebar_width.to_string());
        storage.set_string("dbpro.native.agent-width", self.workspace.agent_width.to_string());
        storage.set_string("dbpro.native.output-open", self.workspace.bottom_panel_open.to_string());
        storage.set_string(
            "dbpro.native.output-height",
            self.workspace.bottom_panel_height.to_string(),
        );
        storage.set_string(
            "dbpro.native.connections-pane-height",
            self.schema_explorer.connections_pane_height.to_string(),
        );
        storage.set_string(
            "dbpro.native.schemas-pane-height",
            self.schema_explorer.schemas_pane_height.to_string(),
        );
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
        let runtime_events_pending = self.apply_runtime_events();
        self.tick_saved_task_scheduler();
        if runtime_events_pending
            || self.runtime_work_pending()
            || self
                .saved_tasks
                .store
                .tasks
                .iter()
                .any(|t| t.schedule.as_ref().is_some_and(|s| s.enabled))
        {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        self.theme = if self.preferences.dark_mode {
            DbProTheme::dark()
        } else {
            DbProTheme::light()
        };
        self.theme.apply(ctx);
        self.handle_shortcuts(ctx);
        self.draw_topbar(ctx);
        // Query owns its rich output dock; the shell panel is for other tabs.
        if self.workspace.active_tab != WorkspaceTab::Query {
            self.draw_output_panel(ctx);
        }
        self.draw_statusbar(ctx);
        self.draw_activity_bar(ctx);

        if self.workspace.sidebar_open {
            self.draw_sidebar(ctx);
        }

        if self.workspace.agent_open {
            self.draw_agent_panel(ctx);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame {
                // Flush to the sidebar splitter; match `SHELL_SPLIT_INSET` / sidebar
                // `pad_right` so the body lines up with the navigator across the divider.
                fill: self.theme.surface_panel,
                inner_margin: egui::Margin {
                    left: SHELL_SPLIT_INSET,
                    right: SHELL_SPLIT_INSET,
                    top: 0.0,
                    bottom: 0.0,
                },
                outer_margin: egui::Margin::ZERO,
                stroke: egui::Stroke::NONE,
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                self.draw_workspace(ui);
            });

        if self.connection_dialog.is_open() {
            self.draw_connection_dialog(ctx);
        }
        if self.overlay.delete_confirmation_id.is_some() {
            self.draw_delete_confirmation(ctx);
        }
        if self.overlay.folder_delete_confirmation.is_some() {
            self.draw_folder_delete_confirmation(ctx);
        }
        if self.table_data.insert_row_open {
            self.draw_insert_row_dialog(ctx);
        }
        if self.palette.mode.is_some() {
            self.draw_palette(ctx);
        }

        self.feedback.toasts.render_ctx(ctx, self.theme);
        if !self.feedback.toasts.is_empty() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
}

// CapabilityLookup lives in `capability_lookup.rs`.

impl DbProApp {
    // Grid layout: `grid_layout.rs`.

    pub(crate) fn show_toast_error(&mut self, message: impl Into<String>) {
        self.feedback
            .toasts
            .error(message, crate::components::overlay::ToastPosition::BottomRight);
    }

    pub(crate) fn show_toast_success(&mut self, message: impl Into<String>) {
        self.feedback
            .toasts
            .success(message, crate::components::overlay::ToastPosition::BottomRight);
    }

    // Kept as a public runtime entry point for future informational notifications.
    #[allow(dead_code)]
    pub(crate) fn show_toast_info(&mut self, message: impl Into<String>) {
        self.feedback
            .toasts
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
        self.show_toast_error(message);
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

    pub(crate) fn open_agent_prompt(&mut self, prompt: impl Into<String>, ctx: &egui::Context) {
        self.agent.input = prompt.into();
        self.set_agent_open(true, ctx);
    }

    fn request_connections_once(&mut self) {
        if self.connection_lifecycle.connections_requested() {
            return;
        }
        self.connection_lifecycle.mark_connections_requested();
        self.connection_lifecycle.set_connections_request_pending(true);
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ListConnections { request_id });
    }

    fn runtime_work_pending(&self) -> bool {
        self.connection_lifecycle.connections_request_pending()
            || self.connection_lifecycle.pending_request.is_some()
            || self.schema_explorer.schema_request.is_some()
            || self
                .query_session_state
                .documents
                .iter()
                .any(|d| d.pending_prediction_request.is_some() || d.prediction_debounce_deadline.is_some())
            || self
                .query_session_state
                .documents
                .iter()
                .any(|d| d.explain_request.is_some())
            || self
                .agent
                .sessions
                .values()
                .any(|session| session.request_id.is_some() || session.active_run_id.is_some())
            || self.table_state.table_info_request.is_some()
            || self.table_state.table_ddl_request.is_some()
            || self.table_state.table_data_request.is_some()
            || self.table_mutation.table_mutation_request.is_some()
            || self.table_mutation.staged_apply_request.is_some()
            || self.table_state.ddl_execution_request.is_some()
    }

    fn request_schema_introspection(&mut self, connection_id: String, force_refresh: bool) {
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::IntrospectSchema {
            request_id,
            connection_id,
            force_refresh,
        });
        self.schema_explorer.schema_request = Some(request_id);
        self.schema_explorer.schema_error = None;
        self.feedback.runtime_message = if force_refresh {
            "Refreshing schema…"
        } else {
            "Loading schema…"
        }
        .to_owned();
    }
}
