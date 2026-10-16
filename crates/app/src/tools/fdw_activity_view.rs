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
                    match db_pro_core::domain::fdw::preview_drop_server(&name, true) {
                        Ok(sql) => {
                            self.state.fdw_error = None;
                            self.state.fdw_ddl_preview = Some(sql);
                        }
                        Err(err) => self.state.fdw_error = Some(err),
                    }
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
                    match db_pro_core::domain::fdw::preview_create_server(
                        &name, &wrapper, &host, &dbname, &port,
                    ) {
                        Ok(sql) => {
                            self.state.fdw_error = None;
                            self.state.fdw_ddl_preview = Some(sql);
                        }
                        Err(err) => self.state.fdw_error = Some(err),
                    }
                }
                fdw_surface_view::FdwSurfaceAction::Create => self.request_fdw_create(),
                fdw_surface_view::FdwSurfaceAction::ConfirmCreate => self.create_fdw_server_confirmed(),
                fdw_surface_view::FdwSurfaceAction::CancelCreate => {
                    self.state.fdw_create_confirm = None
                }
                fdw_surface_view::FdwSurfaceAction::ClosePreview => self.state.fdw_ddl_preview = None,
                fdw_surface_view::FdwSurfaceAction::ConfirmDropServer { name, cascade } => {
                    self.drop_fdw_server_confirmed(&name, cascade);
                }
                fdw_surface_view::FdwSurfaceAction::CancelDrop => self.state.fdw_drop_confirm = None,
            }
        }
    }

    /// Build the CREATE SERVER preview and hold it behind the confirm dialog.
    fn request_fdw_create(&mut self) {
        match db_pro_core::domain::fdw::preview_create_server(
            &self.state.fdw_create_name,
            &self.state.fdw_create_wrapper,
            &self.state.fdw_create_host,
            &self.state.fdw_create_dbname,
            &self.state.fdw_create_port,
        ) {
            Ok(sql) => {
                self.state.fdw_error = None;
                self.state.fdw_create_confirm = Some(fdw_state::FdwCreateApply {
                    name: self.state.fdw_create_name.clone(),
                    fdw: self.state.fdw_create_wrapper.clone(),
                    host: self.state.fdw_create_host.clone(),
                    dbname: self.state.fdw_create_dbname.clone(),
                    port: self.state.fdw_create_port.clone(),
                    sql,
                });
            }
            Err(err) => self.state.fdw_error = Some(err),
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

    /// Dispatch the payload the confirm dialog displayed.
    fn create_fdw_server_confirmed(&mut self) {
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let Some(pending) = self.state.fdw_create_confirm.take() else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        if self.dispatch(create_fdw_server_command(&pending, request_id, connection_id.to_owned())) {
            self.state.fdw_ddl_preview = None;
        }
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

fn create_fdw_server_command(
    pending: &fdw_state::FdwCreateApply,
    request_id: RequestId,
    connection_id: String,
) -> UiCommand {
    UiCommand::CreateFdwServer {
        request_id,
        connection_id,
        name: pending.name.clone(),
        fdw: pending.fdw.clone(),
        host: pending.host.clone(),
        dbname: pending.dbname.clone(),
        port: pending.port.clone(),
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
#[path = "tests/fdw_activity_view_tests.rs"]
mod tests;

