//! Entry point for the Database Navigator in Codex / DBeaver style: a unified
//! hierarchical tree where connections are the root nodes.
//!
//! Row painting lives in `explorer_tree`, table details in `explorer_details`,
//! and the Views / Functions / Triggers folders in `explorer_folders`.

use super::*;
use egui::FontFamily;
use lucide_icons::Icon;

/// Actions selectable from a connection row's context menu.
#[derive(Default)]
pub(crate) struct ConnectionRowActions {
    pub(crate) connect: bool,
    pub(crate) disconnect: bool,
    pub(crate) reconnect: bool,
    pub(crate) refresh: bool,
    pub(crate) new_script: bool,
    pub(crate) er_diagram: bool,
    pub(crate) ask_agent: bool,
    pub(crate) create_table: bool,
    pub(crate) copy_name: bool,
    pub(crate) copy_conn_string: bool,
    pub(crate) edit: bool,
    pub(crate) duplicate: bool,
    pub(crate) delete: bool,
}

/// Collects the connection context-menu choices in full DBeaver style.
pub(crate) fn connection_context_menu(
    ui: &mut egui::Ui,
    response: &egui::Response,
    is_connected: bool,
    theme: DbProTheme,
) -> ConnectionRowActions {
    let mut actions = ConnectionRowActions::default();
    let modifier = DbProApp::primary_modifier_label();
    context_action_menu(ui, response, theme, |ui, close_menu| {
        // ── 1. Connection Lifecycle ──
        if is_connected {
            if ctx_menu_item(ui, Some(Icon::Unplug), "Disconnect", None, theme.text_primary, theme).clicked() {
                actions.disconnect = true;
                *close_menu = true;
            }
            if ctx_menu_item(ui, Some(Icon::RefreshCw), "Reconnect", None, theme.text_primary, theme).clicked() {
                actions.reconnect = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::RotateCcw),
                "Refresh Schema",
                Some("F5"),
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                actions.refresh = true;
                *close_menu = true;
            }
        } else if ctx_menu_item(ui, Some(Icon::Plug), "Connect", None, theme.text_primary, theme).clicked() {
            actions.connect = true;
            *close_menu = true;
        }

        ui.separator();

        // ── 2. SQL & Diagrams ──
        let new_script_sc = format!("{modifier}T");
        if ctx_menu_item(
            ui,
            Some(Icon::FileCode2),
            "SQL Editor / New Script",
            Some(&new_script_sc),
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.new_script = true;
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::Workflow),
            "View ER Diagram",
            None,
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.er_diagram = true;
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::Bot),
            "Ask AI Agent about Database",
            None,
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.ask_agent = true;
            *close_menu = true;
        }

        ui.separator();

        // ── 3. Database Tools & Copy ──
        if ctx_menu_item(
            ui,
            Some(Icon::Plus),
            "Create Table (DDL)…",
            None,
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.create_table = true;
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::Copy),
            "Copy Connection Name",
            None,
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.copy_name = true;
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::Link),
            "Copy Connection String",
            None,
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.copy_conn_string = true;
            *close_menu = true;
        }

        ui.separator();

        // ── 4. Configuration & Management ──
        if ctx_menu_item(
            ui,
            Some(Icon::Pencil),
            "Edit Connection…",
            Some("F4"),
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.edit = true;
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::CopyPlus),
            "Duplicate Connection",
            None,
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.duplicate = true;
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::Trash2),
            "Delete Connection",
            Some("Del"),
            theme.danger,
            theme,
        )
        .clicked()
        {
            actions.delete = true;
            *close_menu = true;
        }
    });
    actions
}

impl DbProApp {
    /// Entry-point for Database Navigator in Codex / DBeaver style:
    /// Unified hierarchical tree where connections are root nodes.
    pub(crate) fn draw_explorer_sub_panes(&mut self, ui: &mut egui::Ui) {
        self.draw_explorer_toolbar(ui);
        ui.add_space(6.0);

        // Capture the padded sidebar width *before* ScrollArea; the scroll
        // content ui can otherwise stick to a narrower previous-frame size and
        // tree rows stop stretching when the panel is resized.
        let tree_width = ui.available_width();
        egui::ScrollArea::vertical()
            .id_salt("codex_navigator_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(tree_width);
                ui.set_max_width(tree_width);
                if self.connections.is_empty() {
                    self.draw_dbeaver_empty_state(ui);
                } else {
                    self.draw_dbeaver_connections_tree(ui);
                }
            });
    }

    /// Search bar plus new-connection / refresh actions above the tree.
    pub(crate) fn draw_explorer_toolbar(&mut self, ui: &mut egui::Ui) {
        let row_width = ui.available_width();
        ui.horizontal(|ui| {
            ui.set_min_width(row_width);
            let actions_width = 56.0;
            let search_width = (row_width - actions_width).max(80.0);
            SearchInput::new(&mut self.explorer_search, "Filter objects…", self.theme)
                .width(search_width)
                .show(ui);

            if compact_icon_button(ui, Icon::Plus, self.theme)
                .on_hover_text("New connection")
                .clicked()
            {
                self.open_new_connection();
            }

            let mut refresh_schema = false;
            let refresh_btn =
                compact_icon_button(ui, Icon::RotateCcw, self.theme).on_hover_text("Refresh active schema");
            refresh_btn.context_menu(|ui| {
                if ctx_menu_item(
                    ui,
                    Some(Icon::RotateCcw),
                    "Refresh Schema",
                    Some("F5"),
                    self.theme.text_primary,
                    self.theme,
                )
                .clicked()
                {
                    refresh_schema = true;
                    ui.close_menu();
                }
            });
            if refresh_btn.clicked() || refresh_schema {
                if let Some(connection_id) = self.active_connection_id.clone() {
                    self.request_schema_introspection(connection_id, true);
                }
            }
        });
    }

    /// Empty state shown when no connections exist yet.
    pub(crate) fn draw_dbeaver_empty_state(&mut self, ui: &mut egui::Ui) {
        ui.add_space(36.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(char::from(Icon::Database).to_string())
                    .family(FontFamily::Name("lucide".into()))
                    .size(28.0)
                    .color(self.theme.text_muted),
            );
            ui.add_space(8.0);
            ui.label(RichText::new("No connections").strong().color(self.theme.text_primary));
            ui.add_space(3.0);
            ui.label(
                RichText::new("Create a database connection to begin.")
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.add_space(12.0);
            if compact_button_with_icon(ui, Icon::Plus, "New connection", self.theme).clicked() {
                self.open_new_connection();
            }
        });
    }

    pub(crate) fn draw_explorer_schema_feedback(&mut self, ui: &mut egui::Ui) {
        let schema_error = self.schema_error.clone();
        if let Some(error) = schema_error.as_deref() {
            grid_frame(self.theme).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(icon_text(Icon::TriangleAlert, "Schema load failed", self.theme.danger));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if let Some(connection_id) = self.active_connection_id.clone() {
                            if secondary_button_with_icon(ui, Icon::RotateCcw, "Refresh schema", self.theme).clicked() {
                                self.request_schema_introspection(connection_id, true);
                            }
                        }
                    });
                });
                ui.add_space(6.0);
                egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                    ui.label(
                        RichText::new(error)
                            .small()
                            .monospace()
                            .color(self.theme.text_secondary),
                    );
                });
            });
            ui.add_space(8.0);
        } else if self.schema_request.is_some() {
            grid_frame(self.theme).show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.reduce_motion {
                        ui.label(icon_text(Icon::LoaderCircle, "Loading schema…", self.theme.accent));
                    } else {
                        ui.spinner();
                        ui.label(RichText::new("Loading schema…").color(self.theme.accent));
                    }
                    ui.label(
                        RichText::new("Large databases may take a moment.")
                            .small()
                            .color(self.theme.text_muted),
                    );
                });
            });
            ui.add_space(8.0);
        }
    }

    /// Clears the connected state after an explicit disconnect.
    pub(crate) fn disconnect_from_connection(&mut self, connection: &UiConnectionSummary) {
        if self.query_in_transaction {
            self.disconnect_txn_guard = true;
            self.runtime_message = "Open transaction detected — commit or rollback before disconnecting".to_owned();
            return;
        }
        self.connected = false;
        self.schema = UiSchemaSummary::default();
        self.selected_table = None;
        self.selected_schema_object = None;
        self.runtime_message = format!("Disconnected from {}", connection.name);
    }

    /// Helper to initiate connection logic.
    pub(crate) fn connect_to_connection(&mut self, connection: &UiConnectionSummary) {
        if self.active_connection_id.as_deref() == Some(&connection.id) && self.connected {
            return;
        }
        if !self.staged_changes.is_empty() {
            self.pending_navigation_action = Some(PendingNavigationAction::ChangeConnection(connection.id.clone()));
            self.discard_changes_confirmation = true;
            self.runtime_message = "Apply or discard staged changes before changing connection".to_owned();
            return;
        }
        if self.query_in_transaction {
            self.disconnect_txn_guard = true;
            self.runtime_message = "Commit or rollback the open transaction before changing connection".to_owned();
            return;
        }
        self.pending_navigation_action = None;
        self.reset_agent_context();
        self.active_connection_id = Some(connection.id.clone());
        self.pending_connection_id = Some(connection.id.clone());
        self.connection_errors.remove(&connection.id);
        self.failed_connection_ids.remove(&connection.id);
        self.selected_schema = None;
        self.schema = UiSchemaSummary::default();
        self.selected_table = None;
        self.selected_schema_object = None;
        self.reset_table_workspace_state();
        self.explorer_search.clear();
        let request_id = self.task_bridge.next_request_id();
        self.connected = false;
        self.pending_connection_request = Some(request_id);
        self.schema_request = None;
        self.schema_error = None;
        self.dispatch_command(UiCommand::Connect {
            request_id,
            connection_id: connection.id.clone(),
        });
        self.runtime_message = format!("Connecting to {}…", connection.name);
    }
}
