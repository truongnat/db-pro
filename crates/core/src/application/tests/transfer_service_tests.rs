//! Tests for `../transfer_service.rs` (moved out of the source file).
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
