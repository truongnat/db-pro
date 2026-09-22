//! State owned by the logical-replication administration surface.

#[derive(Default)]
pub(super) struct ReplicationState {
    pub(super) replication_inventory: Option<db_pro_core::domain::replication::ReplicationInventory>,
    pub(super) replication_error: Option<String>,
    pub(super) replication_create_name: String,
    pub(super) replication_ddl_preview: Option<String>,
    pub(super) replication_drop_publication: Option<String>,
    pub(super) replication_drop_subscription: Option<String>,
}
