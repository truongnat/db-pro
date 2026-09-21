//! Entry point for the Database Navigator in Codex / DBeaver style: a unified
//! hierarchical tree where connections are the root nodes.
//!
//! Row painting lives in `explorer_tree`, table details in `explorer_details`,
//! and the Views / Functions / Triggers folders in `explorer_folders`.

use super::explorer_surface_view::{ExplorerSurfaceAction, ExplorerSurfaceContext};
use super::*;

impl DbProApp {
    /// Entry-point for Database Navigator in Codex / DBeaver style:
    /// Unified hierarchical tree where connections are root nodes.
    pub(crate) fn draw_explorer_sub_panes(&mut self, ui: &mut egui::Ui) {
        let active_schema = self.active_schema().to_owned();
        let functions_enabled = self.active_capabilities().allows(|c| c.schema.functions);
        let actions = ExplorerSurfaceContext {
            theme: self.theme,
            catalog: &self.connection.catalog,
            lifecycle: &self.connection.lifecycle,
            explorer: &mut self.schema.explorer,
            active_schema,
            table_info: self.table.state.table_info.clone(),
            reduce_motion: self.preferences.reduce_motion,
            functions_enabled,
            modifier: Self::primary_modifier_label(),
        }
        .draw(ui);
        for action in actions {
            self.apply_explorer_surface_action(action, ui);
        }
    }

    fn apply_explorer_surface_action(&mut self, action: ExplorerSurfaceAction, ui: &mut egui::Ui) {
        match action {
            ExplorerSurfaceAction::NewConnection => self.connection.open_new(),
            ExplorerSurfaceAction::RefreshSchema => {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    self.request_schema_introspection(connection_id, true);
                }
            }
            ExplorerSurfaceAction::Connection {
                connection,
                action,
                is_connected,
            } => self.apply_connection_row_action(action, &connection, is_connected, ui),
            ExplorerSurfaceAction::Schema { connection_id, action } => {
                self.apply_schema_tree_action(action, &connection_id, ui)
            }
        }
    }

    /// Clears the connected state after an explicit disconnect.
    pub(crate) fn disconnect_from_connection(&mut self, connection: &UiConnectionSummary) {
        explorer_navigation::ExplorerConnectionContext::new(
            &mut self.connection.lifecycle,
            &mut self.schema.explorer,
            &mut self.table,
            &mut self.workspace,
            &mut self.agent,
            &mut self.query.execution,
            &mut self.feedback,
        )
        .disconnect(connection);
    }

    /// Helper to initiate connection logic.
    pub(crate) fn connect_to_connection(&mut self, connection: &UiConnectionSummary) {
        if self.connection.lifecycle.active_connection_id() == Some(connection.id.as_str())
            && self.connection.lifecycle.is_connected()
        {
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        let command = explorer_navigation::ExplorerConnectionContext::new(
            &mut self.connection.lifecycle,
            &mut self.schema.explorer,
            &mut self.table,
            &mut self.workspace,
            &mut self.agent,
            &mut self.query.execution,
            &mut self.feedback,
        )
        .connect(connection, request_id);
        if let Some(command) = command {
            if self.dispatch_command(command) {
                explorer_navigation::ExplorerConnectionContext::new(
                    &mut self.connection.lifecycle,
                    &mut self.schema.explorer,
                    &mut self.table,
                    &mut self.workspace,
                    &mut self.agent,
                    &mut self.query.execution,
                    &mut self.feedback,
                )
                .commit_connect(connection, request_id);
            }
        }
    }
}
