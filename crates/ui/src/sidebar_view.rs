//! Primary left sidebar shell: exact-width panel, padded content, and resize grip.
use super::*;

#[path = "sidebar_surface_view.rs"]
mod sidebar_surface_view;

/// Activity content stays in the root adapter; shell geometry lives in the surface module.
///
/// Sidebar content is clipped to the shell surface.
/// The surface preserves the original clipping and resize behavior.
impl DbProApp {
    pub(super) fn draw_sidebar(&mut self, ctx: &egui::Context) {
        let active_name = if self.connection.lifecycle.active_connection_id().is_some() {
            self.active_connection_name().to_owned()
        } else {
            "DB Pro".to_owned()
        };
        let command_palette_shortcut = Self::format_shortcut(&["Shift", "P"]);
        let new_connection_shortcut = Self::format_shortcut(&["N"]);
        let new_query_shortcuts = Self::shortcut_parts(&["T"]);
        let context = sidebar_surface_view::SidebarSurfaceContext {
            theme: self.theme,
            sidebar_width: self.workspace.sidebar_width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH),
            active_name: &active_name,
            command_palette_shortcut: &command_palette_shortcut,
            new_connection_shortcut: &new_connection_shortcut,
            new_query_shortcuts: &new_query_shortcuts,
        };
        for action in sidebar_surface_view::draw(&context, ctx, |ui| self.draw_sidebar_activity_content(ui)) {
            match action {
                sidebar_surface_view::SidebarSurfaceAction::Chrome(action) => match action {
                    sidebar_chrome_view::SidebarChromeAction::OpenCommandPalette => {
                        self.palette.open(PaletteMode::Commands)
                    }
                    sidebar_chrome_view::SidebarChromeAction::NewConnection => self.connection.open_new(),
                    sidebar_chrome_view::SidebarChromeAction::NewQuery => {
                        self.new_query_document();
                        self.workspace.active_tab = WorkspaceTab::Query;
                    }
                },
                sidebar_surface_view::SidebarSurfaceAction::Resize(width) => {
                    self.workspace.set_sidebar_width(width);
                }
            }
        }
    }

    fn draw_sidebar_activity_content(&mut self, ui: &mut egui::Ui) {
        if self.workspace.activity == Activity::Explorer {
            self.draw_explorer_sub_panes(ui);
            return;
        }
        let scroll_h = ui.available_height();
        egui::ScrollArea::vertical()
            .id_salt("sidebar_scroll")
            .auto_shrink([false, false])
            .max_height(scroll_h)
            .show(ui, |ui| {
                ui.add_space(4.0);
                self.draw_sidebar_activity_body(ui);
            });
    }

    fn draw_sidebar_activity_body(&mut self, ui: &mut egui::Ui) {
        match self.workspace.activity {
            Activity::Queries => self.draw_queries(ui),
            Activity::Files => self.draw_files_activity(ui),
            Activity::Data => self.draw_data_activity(ui),
            Activity::History => self.draw_history(ui),
            Activity::Problems => self.draw_problems(ui),
            Activity::Transfers => self.draw_transfers_activity(ui),
            Activity::Monitor => self.draw_monitor_activity(ui),
            Activity::Security => self.draw_security_activity(ui),
            Activity::Settings => self.draw_settings(ui),
            Activity::Diagram => {
                if navigation_view::draw_diagram_sidebar(ui, self.theme, &self.schema.explorer.schema) {
                    self.workspace.activity = Activity::Explorer;
                    self.workspace.sidebar_open = true;
                }
            }
            Activity::Schema => self.draw_schema_workbench_sidebar(ui),
            Activity::Compare => {
                let action = {
                    let connection_name = self.active_connection_name().to_owned();
                    let driver = self.active_driver().to_owned();
                    let mut context = schema_compare_view::SchemaCompareViewContext {
                        theme: self.theme,
                        compare: &mut self.schema.compare,
                        schema: &self.schema.explorer.schema,
                        connection_name: &connection_name,
                        driver: &driver,
                        feedback: &mut self.feedback,
                    };
                    schema_compare_view::draw_schema_compare_sidebar(&mut context, ui)
                };
                if let Some(action) = action {
                    self.apply_schema_compare_action(action);
                }
            }
            Activity::Tasks => self.draw_tasks_activity(ui),
            Activity::Explorer => unreachable!(),
        }
    }

    fn draw_security_activity(&mut self, ui: &mut egui::Ui) {
        let connected =
            self.connection.lifecycle.is_connected() && self.connection.lifecycle.active_connection_id().is_some();
        let driver = self.active_driver().to_owned();
        let mut command_dispatcher = command_dispatch::RuntimeCommandDispatcher::new(&mut self.task_bridge);
        security_activity_view::SecurityActivityContext {
            theme: self.theme,
            state: &mut self.management.security,
            table_state: &mut self.table.state,
            connected,
            is_postgres: driver.eq_ignore_ascii_case("postgresql") || driver.eq_ignore_ascii_case("postgres"),
            connection_id: self.connection.lifecycle.active_connection_id(),
            command_dispatcher: &mut command_dispatcher,
            feedback: &mut self.feedback,
        }
        .draw(ui);
    }
}
