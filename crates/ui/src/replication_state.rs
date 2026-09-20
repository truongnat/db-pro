//! State owned by the logical-replication administration surface.

use super::{RequestId, UiCommand};

#[derive(Default)]
pub(super) struct ReplicationState {
    pub(super) replication_inventory: Option<db_pro_core::domain::replication::ReplicationInventory>,
    pub(super) replication_error: Option<String>,
    pub(super) replication_create_name: String,
    pub(super) replication_ddl_preview: Option<String>,
    pub(super) replication_drop_publication: Option<String>,
    pub(super) replication_drop_subscription: Option<String>,
}

impl ReplicationState {
    pub(super) fn list_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::ListReplicationInventory {
            request_id,
            connection_id,
        }
    }

    pub(super) fn create_publication_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::CreatePublicationAll {
            request_id,
            connection_id,
            name: self.replication_create_name.clone(),
            confirmed: true,
        }
    }

    pub(super) fn drop_publication_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        name: String,
    ) -> UiCommand {
        UiCommand::DropPublication {
            request_id,
            connection_id,
            name,
            confirmed: true,
        }
    }

    pub(super) fn drop_subscription_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        name: String,
    ) -> UiCommand {
        UiCommand::DropSubscription {
            request_id,
            connection_id,
            name,
            confirmed: true,
        }
    }
}
