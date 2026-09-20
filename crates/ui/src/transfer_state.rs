//! State owned by the data-transfer surface.

#[derive(Default)]
pub(super) struct TransferState {
    pub(super) transfer_jobs: Vec<db_pro_core::domain::transfer::TransferJob>,
}
