use super::*;
use super::{fdw_state::FdwState, RequestId, UiCommand};

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
        let request_id = self.next_request_id();
        self.dispatch_command(list_fdw_inventory_command(request_id, connection_id));
    }

    fn create_fdw_server_confirmed(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.next_request_id();
        self.dispatch_command(create_fdw_server_command(
            &self.management.fdw,
            request_id,
            connection_id,
        ));
    }

    fn drop_fdw_server_confirmed(&mut self, name: &str, cascade: bool) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.next_request_id();
        self.dispatch_command(
            drop_fdw_server_command(request_id, connection_id, name.to_owned(), cascade),
        );
    }
}

pub(super) fn list_fdw_inventory_command(request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::ListFdwInventory {
        request_id,
        connection_id,
    }
}

fn create_fdw_server_command(state: &FdwState, request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::CreateFdwServer {
        request_id,
        connection_id,
        name: state.fdw_create_name.clone(),
        fdw: state.fdw_create_wrapper.clone(),
        host: state.fdw_create_host.clone(),
        dbname: state.fdw_create_dbname.clone(),
        port: state.fdw_create_port.clone(),
        confirmed: true,
    }
}

fn drop_fdw_server_command(
    request_id: RequestId,
    connection_id: String,
    name: String,
    cascade: bool,
) -> UiCommand {
    UiCommand::DropFdwServer {
        request_id,
        connection_id,
        name,
        cascade,
        confirmed: true,
    }
}

#[cfg(test)]
mod tests {
    use super::{create_fdw_server_command, FdwState, RequestId, UiCommand};

    #[test]
    fn create_command_copies_form_state_and_confirms_ddl() {
        let state = FdwState {
            fdw_create_name: "analytics".to_owned(),
            fdw_create_wrapper: "postgres_fdw".to_owned(),
            fdw_create_host: "db.internal".to_owned(),
            fdw_create_dbname: "analytics".to_owned(),
            fdw_create_port: "5432".to_owned(),
            ..FdwState::default()
        };

        assert!(matches!(
            create_fdw_server_command(&state, RequestId(5), "source".to_owned()),
            UiCommand::CreateFdwServer {
                request_id: RequestId(5),
                connection_id,
                name,
                fdw,
                host,
                dbname,
                port,
                confirmed: true,
            } if connection_id == "source"
                && name == "analytics"
                && fdw == "postgres_fdw"
                && host == "db.internal"
                && dbname == "analytics"
                && port == "5432"
        ));
    }
}
