//! Entry point for the Database Navigator in Codex / DBeaver style: a unified
//! hierarchical tree where connections are the root nodes.
//!
//! Row painting lives in `explorer_tree`, table details in `explorer_details`,
//! and the Views / Functions / Triggers folders in `explorer_folders`.

use super::explorer_schema_feedback_view::{ExplorerSchemaFeedbackAction, ExplorerSchemaFeedbackContext};
use super::explorer_toolbar_view::{ExplorerToolbarAction, ExplorerToolbarContext};
use super::*;

impl DbProApp {
    /// Entry-point for Database Navigator in Codex / DBeaver style:
    /// Unified hierarchical tree where connections are root nodes.
    pub(crate) fn draw_explorer_sub_panes(&mut self, ui: &mut egui::Ui) {
        self.draw_explorer_toolbar(ui);
        ui.add_space(6.0);

        // Capture the padded sidebar width *before* ScrollArea. egui's scroll
        // content ui otherwise settles on a content-sized width (short labels),
        // so tree rows / error hints truncate mid-panel while the drag line sits
        // much farther right — unlike VS Code / DBeaver where the tree fills the
        // sidebar.
        //
        // Bound it by the clip as well: `max_rect` is not a hard cap, because
        // `set_max_width` unions with `min_rect`, so a sibling that overflowed earlier in
        // the frame inflates it for the rest of the frame. A tree wider than the column
        // paints its trailing driver badge past the clip and silently loses it. This has to
        // happen here rather than inside the row: once the ScrollArea is running, its clip
        // is narrowed by the scrollbar, so a row clamped to that would jitter narrower
        // every time the connection list crosses the fold.
        let tree_width = ui
            .max_rect()
            .width()
            .max(ui.available_width())
            .min(ui.clip_rect().width());
        // Bound height explicitly so the area scrolls with the wheel instead of
        // growing with content (which leaves only drag-to-scroll working).
        let scroll_h = ui.available_height();
        egui::ScrollArea::vertical()
            .id_salt("codex_navigator_scroll")
            .auto_shrink([false, false])
            .max_height(scroll_h)
            .show(ui, |ui| {
                ui.set_min_width(tree_width);
                ui.set_max_width(tree_width);
                ui.expand_to_include_x(ui.max_rect().left() + tree_width);
                // Claim the full width up front so the first row inherits it
                // instead of measuring against intrinsic label width.
                ui.allocate_exact_size(egui::vec2(tree_width, 0.0), egui::Sense::hover());
                if self.connection.catalog.is_empty() {
                    self.draw_dbeaver_empty_state(ui);
                } else {
                    self.draw_dbeaver_connections_tree(ui);
                }
            });
    }

    /// Filter + refresh above the tree. New-connection lives in the sidebar header
    /// so we do not duplicate the Plus control here.
    pub(crate) fn draw_explorer_toolbar(&mut self, ui: &mut egui::Ui) {
        let actions = {
            let mut context = ExplorerToolbarContext {
                theme: self.theme,
                search: &mut self.schema.explorer.explorer_search,
            };
            context.draw_toolbar(ui)
        };
        for action in actions {
            if matches!(action, ExplorerToolbarAction::RefreshSchema) {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    self.request_schema_introspection(connection_id, true);
                }
            }
        }
    }

    /// Empty state shown when no connections exist yet.
    pub(crate) fn draw_dbeaver_empty_state(&mut self, ui: &mut egui::Ui) {
        let actions = ExplorerToolbarContext {
            theme: self.theme,
            search: &mut self.schema.explorer.explorer_search,
        }
        .draw_empty_state(ui);
        for action in actions {
            if matches!(action, ExplorerToolbarAction::NewConnection) {
                self.connection.open_new();
            }
        }
    }

    pub(crate) fn draw_explorer_schema_feedback(&mut self, ui: &mut egui::Ui) {
        let actions = ExplorerSchemaFeedbackContext {
            theme: self.theme,
            error: self.schema.explorer.schema_error.as_deref(),
            loading: self.schema.explorer.schema_request.is_some(),
            reduce_motion: self.preferences.reduce_motion,
            has_active_connection: self.connection.lifecycle.active_connection_id().is_some(),
        }
        .draw(ui);
        for action in actions {
            if matches!(action, ExplorerSchemaFeedbackAction::RefreshSchema) {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    self.request_schema_introspection(connection_id, true);
                }
            }
        }
    }

    /// Clears the connected state after an explicit disconnect.
    pub(crate) fn disconnect_from_connection(&mut self, connection: &UiConnectionSummary) {
        if self.query.execution.query_in_transaction {
            self.query.execution.disconnect_txn_guard = true;
            self.feedback.runtime_message =
                "Open transaction detected — commit or rollback before disconnecting".to_owned();
            return;
        }
        self.connection.lifecycle.set_connected(false);
        self.schema.explorer.schema = UiSchemaSummary::default();
        self.schema.explorer.schema_symbol_index = SchemaSymbolIndex::default();
        self.schema.explorer.selected_table = None;
        self.schema.explorer.selected_schema_object = None;
        self.feedback.runtime_message = format!("Disconnected from {}", connection.name);
    }

    /// Helper to initiate connection logic.
    pub(crate) fn connect_to_connection(&mut self, connection: &UiConnectionSummary) {
        if self.connection.lifecycle.active_connection_id() == Some(&connection.id)
            && self.connection.lifecycle.is_connected()
        {
            return;
        }
        if !self.table.mutation.staged_changes.is_empty() {
            self.workspace.pending_navigation_action =
                Some(PendingNavigationAction::ChangeConnection(connection.id.clone()));
            self.table.editing.discard_changes_confirmation = true;
            self.feedback.runtime_message = "Apply or discard staged changes before changing connection".to_owned();
            return;
        }
        if self.query.execution.query_in_transaction {
            self.query.execution.disconnect_txn_guard = true;
            self.feedback.runtime_message =
                "Commit or rollback the open transaction before changing connection".to_owned();
            return;
        }
        self.workspace.pending_navigation_action = None;
        self.reset_agent_context();
        *self.connection.lifecycle.active_connection_id_mut() = Some(connection.id.clone());
        self.connection
            .lifecycle
            .set_pending_connection_id(Some(connection.id.clone()));
        self.connection.lifecycle.clear_connection_error(&connection.id);
        self.schema.explorer.selected_schema = None;
        self.schema.explorer.schema = UiSchemaSummary::default();
        self.schema.explorer.schema_symbol_index = SchemaSymbolIndex::default();
        self.schema.explorer.selected_table = None;
        self.schema.explorer.selected_schema_object = None;
        self.table.reset_workspace();
        self.schema.explorer.explorer_search.clear();
        let request_id = self.task_bridge.next_request_id();
        self.connection.lifecycle.set_connected(false);
        self.connection.lifecycle.set_pending_request(Some(request_id));
        self.schema.explorer.schema_request = None;
        self.schema.explorer.schema_error = None;
        self.dispatch_command(
            self.connection
                .lifecycle
                .connect_command(request_id, connection.id.clone()),
        );
        self.feedback.runtime_message = format!("Connecting to {}…", connection.name);
    }
}
