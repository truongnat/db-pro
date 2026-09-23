use super::command_dispatch::RuntimeCommandDispatcher;
use super::replication_state::ReplicationState;
use super::*;

#[path = "replication_surface_view.rs"]
mod replication_surface_view;

pub(super) struct ReplicationActivityContext<'a, 'bridge> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut ReplicationState,
    pub(super) connection_id: Option<&'a str>,
    pub(super) driver: &'a str,
    pub(super) command_dispatcher: &'a mut RuntimeCommandDispatcher<'bridge>,
    pub(super) feedback: &'a mut FeedbackState,
}

impl ReplicationActivityContext<'_, '_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) {
        let actions = replication_surface_view::ReplicationSurfaceContext {
            theme: self.theme,
            state: self.state,
        }
        .draw(ui);
        self.apply_actions(actions);
    }

    fn apply_actions(&mut self, actions: Vec<replication_surface_view::ReplicationSurfaceAction>) {
        for action in actions {
            match action {
                replication_surface_view::ReplicationSurfaceAction::Refresh => self.request_replication_inventory(),
                replication_surface_view::ReplicationSurfaceAction::PreviewDropPublication(name) => {
                    self.state.replication_ddl_preview =
                        db_pro_core::domain::replication::preview_drop_publication(&name).ok();
                }
                replication_surface_view::ReplicationSurfaceAction::RequestDropPublication(name) => {
                    self.state.replication_drop_publication = Some(name);
                }
                replication_surface_view::ReplicationSurfaceAction::PreviewDropSubscription(name) => {
                    self.state.replication_ddl_preview =
                        db_pro_core::domain::replication::preview_drop_subscription(&name).ok();
                }
                replication_surface_view::ReplicationSurfaceAction::RequestDropSubscription(name) => {
                    self.state.replication_drop_subscription = Some(name);
                }
                replication_surface_view::ReplicationSurfaceAction::PreviewCreatePublication(name) => {
                    self.state.replication_ddl_preview =
                        db_pro_core::domain::replication::preview_create_publication_all(&name).ok();
                }
                replication_surface_view::ReplicationSurfaceAction::CreatePublication => {
                    self.create_publication_confirmed();
                }
                replication_surface_view::ReplicationSurfaceAction::ClosePreview => {
                    self.state.replication_ddl_preview = None;
                }
                replication_surface_view::ReplicationSurfaceAction::ConfirmDropPublication(name) => {
                    self.drop_publication_confirmed(&name);
                }
                replication_surface_view::ReplicationSurfaceAction::ConfirmDropSubscription(name) => {
                    self.drop_subscription_confirmed(&name);
                }
                replication_surface_view::ReplicationSurfaceAction::CancelDropPublication => {
                    self.state.replication_drop_publication = None;
                }
                replication_surface_view::ReplicationSurfaceAction::CancelDropSubscription => {
                    self.state.replication_drop_subscription = None;
                }
            }
        }
    }

    fn request_replication_inventory(&mut self) {
        let Some(connection_id) = self.connection_id else {
            self.state.replication_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.driver.to_ascii_lowercase().contains("postgres") {
            self.state.replication_error = Some("Logical replication administration is PostgreSQL-only".into());
            return;
        }
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(list_replication_inventory_command(request_id, connection_id.to_owned()));
    }

    fn create_publication_confirmed(&mut self) {
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(create_publication_command(
            self.state,
            request_id,
            connection_id.to_owned(),
        ));
    }

    fn drop_publication_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(drop_publication_command(
            request_id,
            connection_id.to_owned(),
            name.to_owned(),
        ));
    }

    fn drop_subscription_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(drop_subscription_command(
            request_id,
            connection_id.to_owned(),
            name.to_owned(),
        ));
    }

    fn dispatch(&mut self, command: UiCommand) -> bool {
        self.command_dispatcher.dispatch(command, self.feedback)
    }
}

pub(super) fn list_replication_inventory_command(request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::ListReplicationInventory {
        request_id,
        connection_id,
    }
}

fn create_publication_command(
    state: &ReplicationState,
    request_id: RequestId,
    connection_id: String,
) -> UiCommand {
    UiCommand::CreatePublicationAll {
        request_id,
        connection_id,
        name: state.replication_create_name.clone(),
        confirmed: true,
    }
}

fn drop_publication_command(request_id: RequestId, connection_id: String, name: String) -> UiCommand {
    UiCommand::DropPublication {
        request_id,
        connection_id,
        name,
        confirmed: true,
    }
}

fn drop_subscription_command(request_id: RequestId, connection_id: String, name: String) -> UiCommand {
    UiCommand::DropSubscription {
        request_id,
        connection_id,
        name,
        confirmed: true,
    }
}
