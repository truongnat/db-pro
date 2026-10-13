//! State owned by the data-transfer surface.

#[derive(Default)]
pub(super) struct TransferState {
    pub(super) transfer_jobs: Vec<db_pro_core::domain::transfer::TransferJob>,
    pub(super) source_connection_id: String,
    pub(super) target_connection_id: String,
    pub(super) copy_tables: String,
    pub(super) copy_format: String,
    pub(super) copy_conflict: String,
    pub(super) copy_columns: String,
    pub(super) last_copy_body: String,
}
