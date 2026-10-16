//! State owned by the logical-replication administration surface.

/// `CREATE PUBLICATION … FOR ALL TABLES` awaiting the user's SQL-preview
/// confirmation — the payload is frozen so the dialog dispatches what it showed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ReplicationCreateApply {
    pub(super) name: String,
    pub(super) sql: String,
}

#[derive(Default)]
pub(super) struct ReplicationState {
    pub(super) replication_inventory: Option<db_pro_core::domain::replication::ReplicationInventory>,
    pub(super) replication_error: Option<String>,
    pub(super) replication_create_name: String,
    pub(super) replication_create_confirm: Option<ReplicationCreateApply>,
    pub(super) replication_ddl_preview: Option<String>,
    pub(super) replication_drop_publication: Option<String>,
    pub(super) replication_drop_subscription: Option<String>,
}
