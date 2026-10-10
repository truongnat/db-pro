use super::workspace_shell::WorkspaceLocation;
use super::*;
impl DbProApp {
    pub(super) fn draw_topbar(&mut self, ui: &mut egui::Ui) {
        let connection_name = self.active_connection_name().to_owned();
        let driver = self.active_driver().to_owned();
        let has_connection = self.connection.lifecycle.active_connection_id().is_some();
        let (connection_icon, connection_color) = self
            .active_connection()
            .map(|connection| self.connection_indicator(connection))
            .unwrap_or((Icon::Circle, self.theme.warning));
        self.record_workspace_navigation();
        let actions = shell_topbar_view::ShellTopbarContext {
            theme: self.theme,
            sidebar_open: self.workspace.sidebar_open,
            can_go_back: !self.workspace.nav_back.is_empty(),
            can_go_forward: !self.workspace.nav_forward.is_empty(),
            has_connection,
            connection_name: &connection_name,
            connection_icon,
            connection_color,
            driver: &driver,
            agent_open: self.workspace.agent_open,
            dark_mode: self.preferences.dark_mode,
        }
        .draw(ui);
        self.apply_topbar_actions(ui.ctx(), actions);
    }

    fn apply_topbar_actions(&mut self, ctx: &egui::Context, actions: Vec<shell_topbar_view::ShellTopbarAction>) {
        for action in actions {
            match action {
                shell_topbar_view::ShellTopbarAction::ToggleSidebar => {
                    self.workspace.sidebar_open = !self.workspace.sidebar_open;
                }
                shell_topbar_view::ShellTopbarAction::PreviousDocument => self.go_back(),
                shell_topbar_view::ShellTopbarAction::NextDocument => self.go_forward(),
                shell_topbar_view::ShellTopbarAction::OpenCommandPalette => {
                    self.palette.open(PaletteMode::Commands);
                }
                shell_topbar_view::ShellTopbarAction::OpenComponentGallery => {
                    self.workspace.active_tab = WorkspaceTab::ComponentGallery;
                }
                shell_topbar_view::ShellTopbarAction::ToggleAgent => {
                    self.set_agent_open(!self.workspace.agent_open, ctx);
                }
                shell_topbar_view::ShellTopbarAction::ToggleTheme => {
                    self.preferences.dark_mode = !self.preferences.dark_mode;
                }
                shell_topbar_view::ShellTopbarAction::OpenQuickOpen => {
                    self.palette.open(PaletteMode::QuickOpen);
                }
            }
        }
    }

    pub(crate) fn record_workspace_navigation(&mut self) {
        let current = WorkspaceLocation {
            tab: self.workspace.active_tab,
            query_index: self.query.session.active_document_index,
        };
        if !self.workspace.nav_ready {
            self.workspace.nav_ready = true;
            self.workspace.nav_current = current;
            return;
        }
        if self.workspace.nav_suppress {
            self.workspace.nav_suppress = false;
            self.workspace.nav_current = current;
            return;
        }
        if current == self.workspace.nav_current {
            return;
        }
        let previous = self.workspace.nav_current;
        self.workspace.nav_back.push(previous);
        if self.workspace.nav_back.len() > 64 {
            self.workspace.nav_back.remove(0);
        }
        self.workspace.nav_forward.clear();
        self.workspace.nav_current = current;
    }

    pub(crate) fn go_back(&mut self) {
        let Some(previous) = self.workspace.nav_back.pop() else {
            return;
        };
        let current = self.workspace.nav_current;
        self.workspace.nav_forward.push(current);
        self.workspace.nav_suppress = true;
        self.restore_workspace_location(previous);
    }

    pub(crate) fn go_forward(&mut self) {
        let Some(next) = self.workspace.nav_forward.pop() else {
            return;
        };
        let current = self.workspace.nav_current;
        self.workspace.nav_back.push(current);
        self.workspace.nav_suppress = true;
        self.restore_workspace_location(next);
    }

    fn restore_workspace_location(&mut self, location: WorkspaceLocation) {
        match location.tab {
            WorkspaceTab::Welcome => self.workspace.welcome_open = true,
            WorkspaceTab::Diagram => self.workspace.diagram_open = true,
            WorkspaceTab::Results => self.workspace.results_open = true,
            WorkspaceTab::Query => self.switch_query_document(location.query_index),
            _ => {}
        }
        self.workspace.active_tab = location.tab;
    }

    pub(super) fn draw_statusbar(&mut self, ui: &mut egui::Ui) {
        let (icon, color, label) = self.statusbar_state();
        let runtime_status = self.runtime_status();
        let database = self.active_connection().map(|connection| connection.database.clone());
        let schema = self.active_connection().map(|_| self.active_schema().to_owned());
        let duration_ms = self
            .query
            .session
            .active_result()
            .or(self.table.data_query.result.as_ref())
            .map(|result| result.duration_ms);
        let action = shell_statusbar_view::ShellStatusbarContext {
            theme: self.theme,
            icon,
            icon_color: color,
            label,
            runtime_status,
            connected: self.connection.lifecycle.is_connected(),
            connection_name: self.active_connection_name(),
            driver: self.active_driver(),
            database: database.as_deref(),
            schema: schema.as_deref(),
            duration_ms,
            editor_status: self.shows_editor_status(),
            cursor_line: self.query.editor.query_cursor_line,
            cursor_column: self.query.editor.query_cursor_column,
            context_label: self.statusbar_context_label(),
        }
        .draw(ui);
        if matches!(action, Some(shell_statusbar_view::ShellStatusbarAction::ToggleOutput)) {
            self.workspace.bottom_panel_open = !self.workspace.bottom_panel_open;
        }
    }

    pub(super) fn draw_output_panel(&mut self, ui: &mut egui::Ui) {
        if !self.workspace.bottom_panel_open {
            return;
        }
        let result = self
            .query
            .session
            .active_result()
            .or(self.table.data_query.result.as_ref())
            .cloned();
        let height = self.workspace.bottom_panel_height;
        let response = egui::Panel::bottom("output_panel")
            .resizable(true)
            .default_size(height)
            .size_range(OUTPUT_MIN_HEIGHT..=OUTPUT_MAX_HEIGHT)
            .frame(panel_frame(self.theme))
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                // Same tab strip and panes as the query output dock — the shell
                // panel used to render summary stubs that could not run
                // Explain/History actions, which read as a broken surface.
                let mut tabs_context = query_output_tabs_view::QueryOutputTabsContext {
                    theme: self.theme,
                    output: &mut self.query.output,
                    session: &self.query.session,
                    editor: &mut self.query.editor,
                    bottom_panel_open: &mut self.workspace.shell.bottom_panel_open,
                    dock_position: None,
                    results_open: Some(&mut self.workspace.shell.results_open),
                    active_tab: Some(&mut self.workspace.shell.active_tab),
                };
                query_output_tabs_view::draw_output_tabs(&mut tabs_context, ui, true);
                self.draw_output_pane(ui, result.as_ref());
            });
        // egui's resizable Panel emits an anonymous `Unknown` node for the
        // drag handle (id = panel id + "__resize") — name it so AT can
        // announce "resize output panel" instead of nothing.
        ui.ctx().accesskit_node_builder(egui::Id::new("output_panel").with("__resize"), |b| {
            b.set_label("Resize output panel");
        });
        self.workspace.set_bottom_panel_height(response.response.rect.height());
    }
}
