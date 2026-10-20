//! State owned by the data-transfer surface.

#[derive(Default)]
pub(super) struct TransferState {
    pub(super) transfer_jobs: Vec<db_pro_core::domain::transfer::TransferJob>,
    /// Demo harness files created in temp_dir; cleared with the job list
    /// (#422) so exports don't linger on disk.
    pub(super) harness_temp_files: Vec<std::path::PathBuf>,
    /// Transfers running on worker threads (#421); outcomes land via
    /// `drain_pending` from the per-frame tick so draw never blocks on I/O.
    pub(super) pending_transfers: Vec<std::sync::mpsc::Receiver<super::transfer_harness_view::TransferHarnessOutcome>>,
    pub(super) source_connection_id: String,
    pub(super) target_connection_id: String,
    pub(super) copy_tables: String,
    pub(super) copy_format: String,
    pub(super) copy_conflict: String,
    pub(super) copy_columns: String,
    pub(super) last_copy_body: String,
    /// One-shot presentation focus used only by deterministic capture fixtures.
    pub(super) capture_focus_once: bool,
}

impl TransferState {
    /// Collect finished transfer workers; receivers that are still running are
    /// kept for the next frame.
    pub(super) fn drain_pending(&mut self) -> Vec<super::transfer_harness_view::TransferHarnessOutcome> {
        let mut done = Vec::new();
        self.pending_transfers.retain_mut(|receiver| match receiver.try_recv() {
            Ok(outcome) => {
                done.push(outcome);
                false
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => true,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => false,
        });
        done
    }
}
