use super::*;
use super::{replication_state::ReplicationState, RequestId, UiCommand};

#[path = "replication_surface_view.rs"]
mod replication_surface_view;

impl DbProApp {
    pub(super) fn draw_replication_activity(&mut self, ui: &mut egui::Ui) {
        let actions = replication_surface_view::ReplicationSurfaceContext {
            theme: self.theme,
            state: &mut self.management.replication,
        }
        .draw(ui);
        self.apply_replication_surface_actions(actions);
    }

    fn apply_replication_surface_actions(&mut self, actions: Vec<replication_surface_view::ReplicationSurfaceAction>) {
        for action in actions {
            match action {
                replication_surface_view::ReplicationSurfaceAction::Refresh => self.request_replication_inventory(),
                replication_surface_view::ReplicationSurfaceAction::PreviewDropPublication(name) => {
                    self.management.replication.replication_ddl_preview =
                        db_pro_core::domain::replication::preview_drop_publication(&name).ok();
                }
                replication_surface_view::ReplicationSurfaceAction::RequestDropPublication(name) => {
                    self.management.replication.replication_drop_publication = Some(name);
                }
                replication_surface_view::ReplicationSurfaceAction::PreviewDropSubscription(name) => {
                    self.management.replication.replication_ddl_preview =
                        db_pro_core::domain::replication::preview_drop_subscription(&name).ok();
                }
                replication_surface_view::ReplicationSurfaceAction::RequestDropSubscription(name) => {
                    self.management.replication.replication_drop_subscription = Some(name);
                }
                replication_surface_view::ReplicationSurfaceAction::PreviewCreatePublication(name) => {
                    self.management.replication.replication_ddl_preview =
                        db_pro_core::domain::replication::preview_create_publication_all(&name).ok();
                }
                replication_surface_view::ReplicationSurfaceAction::CreatePublication => {
                    self.create_publication_confirmed()
                }
                replication_surface_view::ReplicationSurfaceAction::ClosePreview => {
                    self.management.replication.replication_ddl_preview = None;
                }
                replication_surface_view::ReplicationSurfaceAction::ConfirmDropPublication(name) => {
                    self.drop_publication_confirmed(&name);
                }
                replication_surface_view::ReplicationSurfaceAction::ConfirmDropSubscription(name) => {
                    self.drop_subscription_confirmed(&name);
                }
                replication_surface_view::ReplicationSurfaceAction::CancelDropPublication => {
                    self.management.replication.replication_drop_publication = None;
                }
                replication_surface_view::ReplicationSurfaceAction::CancelDropSubscription => {
                    self.management.replication.replication_drop_subscription = None;
                }
            }
        }
    }

    fn request_replication_inventory(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.management.replication.replication_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.active_driver().to_ascii_lowercase().contains("postgres") {
            self.management.replication.replication_error =
                Some("Logical replication administration is PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(list_replication_inventory_command(request_id, connection_id));
    }

    fn create_publication_confirmed(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            create_publication_command(&self.management.replication, request_id, connection_id),
        );
    }

    fn drop_publication_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(drop_publication_command(
            request_id,
            connection_id,
            name.to_owned(),
        ));
    }

    fn drop_subscription_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(drop_subscription_command(
            request_id,
            connection_id,
            name.to_owned(),
        ));
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
