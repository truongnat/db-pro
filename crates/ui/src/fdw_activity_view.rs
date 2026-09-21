use super::*;

#[path = "fdw_surface_view.rs"]
mod fdw_surface_view;

impl DbProApp {
    pub(super) fn draw_fdw_activity(&mut self, ui: &mut egui::Ui) {
        let actions = fdw_surface_view::FdwSurfaceContext {
            theme: self.theme,
            state: &mut self.management.fdw,
        }
        .draw(ui);
        self.apply_fdw_surface_actions(actions);
    }

    fn apply_fdw_surface_actions(&mut self, actions: Vec<fdw_surface_view::FdwSurfaceAction>) {
        for action in actions {
            match action {
                fdw_surface_view::FdwSurfaceAction::Refresh => self.request_fdw_inventory(),
                fdw_surface_view::FdwSurfaceAction::PreviewDropServer(name) => {
                    self.management.fdw.fdw_ddl_preview =
                        db_pro_core::domain::fdw::preview_drop_server(&name, true).ok();
                }
                fdw_surface_view::FdwSurfaceAction::RequestDropServer(name) => {
                    self.management.fdw.fdw_drop_confirm = Some(name);
                }
                fdw_surface_view::FdwSurfaceAction::PreviewCreate {
                    name,
                    wrapper,
                    host,
                    dbname,
                    port,
                } => {
                    self.management.fdw.fdw_ddl_preview =
                        db_pro_core::domain::fdw::preview_create_server(&name, &wrapper, &host, &dbname, &port).ok();
                }
                fdw_surface_view::FdwSurfaceAction::Create => self.create_fdw_server_confirmed(),
                fdw_surface_view::FdwSurfaceAction::ClosePreview => self.management.fdw.fdw_ddl_preview = None,
                fdw_surface_view::FdwSurfaceAction::ConfirmDropServer { name, cascade } => {
                    self.drop_fdw_server_confirmed(&name, cascade);
                }
                fdw_surface_view::FdwSurfaceAction::CancelDrop => self.management.fdw.fdw_drop_confirm = None,
            }
        }
    }

    fn request_fdw_inventory(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.management.fdw.fdw_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.active_driver().to_ascii_lowercase().contains("postgres") {
            self.management.fdw.fdw_error = Some("FDW administration is PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.fdw.list_command(request_id, connection_id));
    }

    fn create_fdw_server_confirmed(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.fdw.create_command(request_id, connection_id));
    }

    fn drop_fdw_server_confirmed(&mut self, name: &str, cascade: bool) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            self.management
                .fdw
                .drop_command(request_id, connection_id, name.to_owned(), cascade),
        );
    }
}
