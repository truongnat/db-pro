use super::*;
impl DbProApp {
    pub(super) fn draw_topbar(&mut self, ctx: &egui::Context) {
        let connection_name = self.active_connection_name().to_owned();
        let driver = self.active_driver().to_owned();
        let has_connection = self.connection.lifecycle.active_connection_id().is_some();
        let (connection_icon, connection_color) = self
            .active_connection()
            .map(|connection| self.connection_indicator(connection))
            .unwrap_or((Icon::Circle, self.theme.warning));
        let actions = shell_topbar_view::ShellTopbarContext {
            theme: self.theme,
            sidebar_open: self.workspace.sidebar_open,
            active_document_index: self.query.session.active_document_index,
            document_count: self.query.session.documents.len(),
            has_connection,
            connection_name: &connection_name,
            connection_icon,
            connection_color,
            driver: &driver,
            agent_open: self.workspace.agent_open,
            dark_mode: self.preferences.dark_mode,
        }
        .draw(ctx);
        self.apply_topbar_actions(ctx, actions);
    }

    fn apply_topbar_actions(&mut self, ctx: &egui::Context, actions: Vec<shell_topbar_view::ShellTopbarAction>) {
        for action in actions {
            match action {
                shell_topbar_view::ShellTopbarAction::ToggleSidebar => {
                    self.workspace.sidebar_open = !self.workspace.sidebar_open;
                }
                shell_topbar_view::ShellTopbarAction::PreviousDocument => {
                    self.switch_query_document(self.query.session.active_document_index.saturating_sub(1));
                }
                shell_topbar_view::ShellTopbarAction::NextDocument => {
                    self.switch_query_document(self.query.session.active_document_index + 1);
                }
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

    pub(super) fn draw_statusbar(&mut self, ctx: &egui::Context) {
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
        .draw(ctx);
        if matches!(action, Some(shell_statusbar_view::ShellStatusbarAction::ToggleOutput)) {
            self.workspace.bottom_panel_open = !self.workspace.bottom_panel_open;
        }
    }

    pub(super) fn draw_output_panel(&mut self, ctx: &egui::Context) {
        let active_result = self
            .query
            .session
            .active_result()
            .or(self.table.data_query.result.as_ref());
        shell_output_panel_view::ShellOutputPanelContext {
            theme: self.theme,
            workspace: &mut self.workspace,
            output: &mut self.query.output,
            session: &self.query.session,
            editor: &self.query.editor,
            active_result,
        }
        .draw(ctx);
    }
}
