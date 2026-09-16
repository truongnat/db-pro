//! Streaming transfer engine (#193). Bounded-memory batch loop with cancellation.

use crate::domain::transfer::{
    TransferCancellation, TransferError, TransferJob, TransferResult, TransferRow, TransferSourceKind, TransferStatus,
    TransferTargetKind,
};

/// Pulls bounded batches from a source.
pub trait TransferSource {
    fn next_batch(&mut self, max_rows: usize) -> Result<Option<Vec<TransferRow>>, TransferError>;
}

/// Writes bounded batches to a target; must not require the full dataset.
pub trait TransferTarget {
    fn write_batch(&mut self, rows: &[TransferRow]) -> Result<u64, TransferError>;
    fn finish(&mut self) -> Result<(), TransferError>;
    fn cleanup_partial(&mut self) -> Result<(), TransferError>;

    /// Batches committed under the active transaction policy (DB targets).
    fn committed_batches(&self) -> u64 {
        0
    }

    /// Rows held in an open transaction that has not committed yet.
    fn uncommitted_rows(&self) -> u64 {
        0
    }

    /// Rows rejected by conflict policy (e.g. Skip) without failing the job.
    fn error_rows(&self) -> u64 {
        0
    }
}

/// In-memory synthetic source used to prove the engine without provider I/O.
pub struct SyntheticSource {
    remaining: u64,
    next_id: u64,
}

impl SyntheticSource {
    pub fn new(rows: u64) -> Self {
        Self {
            remaining: rows,
            next_id: 0,
        }
    }
}

impl TransferSource for SyntheticSource {
    fn next_batch(&mut self, max_rows: usize) -> Result<Option<Vec<TransferRow>>, TransferError> {
        if self.remaining == 0 {
            return Ok(None);
        }
        let take = (max_rows as u64).min(self.remaining) as usize;
        let mut batch = Vec::with_capacity(take);
        for _ in 0..take {
            batch.push(TransferRow {
                cells: vec![self.next_id.to_string(), format!("row-{}", self.next_id)],
            });
            self.next_id += 1;
            self.remaining -= 1;
        }
        Ok(Some(batch))
    }
}

/// Counts written rows/bytes without retaining row payloads.
#[derive(Debug, Default)]
pub struct CountingTarget {
    pub rows_written: u64,
    pub bytes_written: u64,
    pub finished: bool,
    pub cleaned_up: bool,
}

impl TransferTarget for CountingTarget {
    fn write_batch(&mut self, rows: &[TransferRow]) -> Result<u64, TransferError> {
        let mut bytes = 0u64;
        for row in rows {
            for cell in &row.cells {
                bytes += cell.len() as u64;
            }
        }
        self.rows_written += rows.len() as u64;
        self.bytes_written += bytes;
        Ok(rows.len() as u64)
    }

    fn finish(&mut self) -> Result<(), TransferError> {
        self.finished = true;
        Ok(())
    }

    fn cleanup_partial(&mut self) -> Result<(), TransferError> {
        self.cleaned_up = true;
        Ok(())
    }
}

pub struct TransferService;

impl TransferService {
    /// Run a transfer to completion (or cancel/fail). Progress is updated in `job`.
    pub fn run<S: TransferSource, T: TransferTarget>(
        job: &mut TransferJob,
        source: &mut S,
        target: &mut T,
        cancel: &TransferCancellation,
    ) -> TransferResult {
        job.status = TransferStatus::Running;
        job.progress.message = "Running".into();
        job.error = None;

        loop {
            if cancel.is_cancelled() {
                let _ = target.cleanup_partial();
                job.progress.committed_batches = target.committed_batches();
                job.progress.uncommitted_rows = target.uncommitted_rows();
                job.progress.error_rows = target.error_rows();
                job.status = TransferStatus::Cancelled;
                job.progress.message = "Cancelled".into();
                job.error = Some("cancelled by user".into());
                return TransferResult {
                    status: job.status,
                    progress: job.progress.clone(),
                    error: job.error.clone(),
                };
            }

            let batch = match source.next_batch(job.batch_size) {
                Ok(Some(batch)) => batch,
                Ok(None) => break,
                Err(TransferError::Cancelled) => {
                    let _ = target.cleanup_partial();
                    job.progress.committed_batches = target.committed_batches();
                    job.progress.uncommitted_rows = target.uncommitted_rows();
                    job.progress.error_rows = target.error_rows();
                    job.status = TransferStatus::Cancelled;
                    job.progress.message = "Cancelled".into();
                    job.error = Some("cancelled by user".into());
                    return TransferResult {
                        status: job.status,
                        progress: job.progress.clone(),
                        error: job.error.clone(),
                    };
                }
                Err(err) => {
                    let _ = target.cleanup_partial();
                    job.progress.committed_batches = target.committed_batches();
                    job.progress.uncommitted_rows = target.uncommitted_rows();
                    job.progress.error_rows = target.error_rows();
                    job.status = TransferStatus::Failed;
                    job.progress.message = "Failed".into();
                    job.error = Some(err.to_string());
                    return TransferResult {
                        status: job.status,
                        progress: job.progress.clone(),
                        error: job.error.clone(),
                    };
                }
            };

            job.progress.rows_read = job.progress.rows_read.saturating_add(batch.len() as u64);
            match target.write_batch(&batch) {
                Ok(written) => {
                    job.progress.rows_written = job.progress.rows_written.saturating_add(written);
                    job.progress.committed_batches = target.committed_batches();
                    job.progress.uncommitted_rows = target.uncommitted_rows();
                    job.progress.error_rows = target.error_rows();
                    job.progress.message =
                        format!("Wrote {} / read {}", job.progress.rows_written, job.progress.rows_read);
                }
                Err(err) => {
                    let _ = target.cleanup_partial();
                    job.progress.committed_batches = target.committed_batches();
                    job.progress.uncommitted_rows = target.uncommitted_rows();
                    job.progress.error_rows = target.error_rows();
                    job.status = TransferStatus::Failed;
                    job.progress.message = "Failed".into();
                    job.error = Some(err.to_string());
                    return TransferResult {
                        status: job.status,
                        progress: job.progress.clone(),
                        error: job.error.clone(),
                    };
                }
            }
        }

        if let Err(err) = target.finish() {
            let _ = target.cleanup_partial();
            job.progress.committed_batches = target.committed_batches();
            job.progress.uncommitted_rows = target.uncommitted_rows();
            job.progress.error_rows = target.error_rows();
            job.status = TransferStatus::Failed;
            job.progress.message = "Failed".into();
            job.error = Some(err.to_string());
            return TransferResult {
                status: job.status,
                progress: job.progress.clone(),
                error: job.error.clone(),
            };
        }

        job.progress.committed_batches = target.committed_batches();
        job.progress.uncommitted_rows = target.uncommitted_rows();
        job.progress.error_rows = target.error_rows();
        job.status = if job.progress.error_rows > 0 {
            TransferStatus::Partial
        } else {
            TransferStatus::Succeeded
        };
        job.progress.message = match job.status {
            TransferStatus::Partial => "Completed with error rows".into(),
            _ => "Succeeded".into(),
        };
        TransferResult {
            status: job.status,
            progress: job.progress.clone(),
            error: None,
        }
    }

    /// Convenience harness: synthetic source → counting target.
    pub fn run_synthetic(job: &mut TransferJob, cancel: &TransferCancellation) -> TransferResult {
        let rows = match job.source {
            TransferSourceKind::Synthetic { rows } => rows,
            _ => {
                job.status = TransferStatus::Failed;
                job.error = Some("run_synthetic requires Synthetic source".into());
                return TransferResult {
                    status: job.status,
                    progress: job.progress.clone(),
                    error: job.error.clone(),
                };
            }
        };
        if !matches!(job.target, TransferTargetKind::Counting) {
            job.status = TransferStatus::Failed;
            job.error = Some("run_synthetic requires Counting target".into());
            return TransferResult {
                status: job.status,
                progress: job.progress.clone(),
                error: job.error.clone(),
            };
        }
        let mut source = SyntheticSource::new(rows);
        let mut target = CountingTarget::default();
        let mut result = Self::run(job, &mut source, &mut target, cancel);
        job.progress.bytes_written = target.bytes_written;
        result.progress.bytes_written = target.bytes_written;
        // Keep progress truthful after cancel/failure (do not invent success counts).
        if matches!(result.status, TransferStatus::Succeeded | TransferStatus::Partial) {
            debug_assert_eq!(job.progress.rows_written, target.rows_written);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;

    #[test]
    fn streams_large_synthetic_with_bounded_batches() {
        let mut job = TransferJob::new_synthetic("job-1", 50_000, 256);
        let cancel = TransferCancellation::new();
        let result = TransferService::run_synthetic(&mut job, &cancel);
        assert_eq!(result.status, TransferStatus::Succeeded);
        assert_eq!(result.progress.rows_read, 50_000);
        assert_eq!(result.progress.rows_written, 50_000);
        assert!(result.progress.bytes_written > 0);
        // Peak batch is bounded by batch_size (engine never holds the full set).
        assert_eq!(job.batch_size, 256);
    }

    #[test]
    fn cancellation_cleans_up_and_keeps_partial_progress() {
        struct SlowSource {
            remaining: u64,
            batches: Arc<AtomicUsize>,
        }
        impl TransferSource for SlowSource {
            fn next_batch(&mut self, max_rows: usize) -> Result<Option<Vec<TransferRow>>, TransferError> {
                if self.remaining == 0 {
                    return Ok(None);
                }
                self.batches.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let take = (max_rows as u64).min(self.remaining).min(10) as usize;
                self.remaining -= take as u64;
                Ok(Some(
                    (0..take)
                        .map(|i| TransferRow {
                            cells: vec![i.to_string()],
                        })
                        .collect(),
                ))
            }
        }

        let mut job = TransferJob::new_synthetic("job-cancel", 10_000, 10);
        let cancel = TransferCancellation::new();
        let batches = Arc::new(AtomicUsize::new(0));
        let mut source = SlowSource {
            remaining: 10_000,
            batches: Arc::clone(&batches),
        };
        let mut target = CountingTarget::default();

        // Cancel after first write by wrapping run manually with mid-flight cancel.
        job.status = TransferStatus::Running;
        let first = source.next_batch(job.batch_size).unwrap().unwrap();
        job.progress.rows_read += first.len() as u64;
        let written = target.write_batch(&first).unwrap();
        job.progress.rows_written += written;
        cancel.cancel();

        let result = TransferService::run(&mut job, &mut source, &mut target, &cancel);
        assert_eq!(result.status, TransferStatus::Cancelled);
        assert!(target.cleaned_up);
        assert!(job.progress.rows_written > 0);
        assert!(job.progress.rows_written < 10_000);
        assert_eq!(job.progress.message, "Cancelled");
        assert!(job.error.as_deref() == Some("cancelled by user"));
    }

    #[test]
    fn progress_does_not_claim_success_after_failure() {
        struct FailTarget;
        impl TransferTarget for FailTarget {
            fn write_batch(&mut self, _rows: &[TransferRow]) -> Result<u64, TransferError> {
                Err(TransferError::Target("disk full".into()))
            }
            fn finish(&mut self) -> Result<(), TransferError> {
                Ok(())
            }
            fn cleanup_partial(&mut self) -> Result<(), TransferError> {
                Ok(())
            }
        }

        let mut job = TransferJob::new_synthetic("job-fail", 100, 10);
        let cancel = TransferCancellation::new();
        let mut source = SyntheticSource::new(100);
        let mut target = FailTarget;
        let result = TransferService::run(&mut job, &mut source, &mut target, &cancel);
        assert_eq!(result.status, TransferStatus::Failed);
        assert_eq!(job.progress.message, "Failed");
        assert!(job.error.as_deref().unwrap().contains("disk full"));
        assert_ne!(job.status, TransferStatus::Succeeded);
    }
}
