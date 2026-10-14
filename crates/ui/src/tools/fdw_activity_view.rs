use super::command_dispatch::RuntimeCommandDispatcher;
use super::fdw_state::FdwState;
use super::*;

#[path = "fdw_surface_view.rs"]
mod fdw_surface_view;

pub(super) struct FdwActivityContext<'a, 'bridge> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut FdwState,
    pub(super) connection_id: Option<&'a str>,
    pub(super) driver: &'a str,
    pub(super) command_dispatcher: &'a mut RuntimeCommandDispatcher<'bridge>,
    pub(super) feedback: &'a mut FeedbackState,
}

impl FdwActivityContext<'_, '_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) {
        let actions = fdw_surface_view::FdwSurfaceContext {
            theme: self.theme,
            state: self.state,
        }
        .draw(ui);
        self.apply_actions(actions);
    }

    fn apply_actions(&mut self, actions: Vec<fdw_surface_view::FdwSurfaceAction>) {
        for action in actions {
            match action {
                fdw_surface_view::FdwSurfaceAction::Refresh => self.request_fdw_inventory(),
                fdw_surface_view::FdwSurfaceAction::PreviewDropServer(name) => {
                    self.state.fdw_ddl_preview =
                        db_pro_core::domain::fdw::preview_drop_server(&name, true).ok();
                }
                fdw_surface_view::FdwSurfaceAction::RequestDropServer(name) => {
                    self.state.fdw_drop_confirm = Some(name);
                }
                fdw_surface_view::FdwSurfaceAction::PreviewCreate {
                    name,
                    wrapper,
                    host,
                    dbname,
                    port,
                } => {
                    self.state.fdw_ddl_preview = db_pro_core::domain::fdw::preview_create_server(
                        &name, &wrapper, &host, &dbname, &port,
                    )
                    .ok();
                }
                fdw_surface_view::FdwSurfaceAction::Create => self.create_fdw_server_confirmed(),
                fdw_surface_view::FdwSurfaceAction::ClosePreview => self.state.fdw_ddl_preview = None,
                fdw_surface_view::FdwSurfaceAction::ConfirmDropServer { name, cascade } => {
                    self.drop_fdw_server_confirmed(&name, cascade);
                }
                fdw_surface_view::FdwSurfaceAction::CancelDrop => self.state.fdw_drop_confirm = None,
            }
        }
    }

    fn request_fdw_inventory(&mut self) {
        let Some(connection_id) = self.connection_id else {
            self.state.fdw_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.driver.to_ascii_lowercase().contains("postgres") {
            self.state.fdw_error = Some("FDW administration is PostgreSQL-only".into());
            return;
        }
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(list_fdw_inventory_command(request_id, connection_id.to_owned()));
    }

    fn create_fdw_server_confirmed(&mut self) {
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(create_fdw_server_command(
            self.state,
            request_id,
            connection_id.to_owned(),
        ));
    }

    fn drop_fdw_server_confirmed(&mut self, name: &str, cascade: bool) {
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(drop_fdw_server_command(
            request_id,
            connection_id.to_owned(),
            name.to_owned(),
            cascade,
        ));
    }

    fn dispatch(&mut self, command: UiCommand) -> bool {
        self.command_dispatcher.dispatch(command, self.feedback)
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
