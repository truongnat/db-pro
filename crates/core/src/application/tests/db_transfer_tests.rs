//! Tests for `../db_transfer.rs` (moved out of the source file).
use super::*;
use crate::application::transfer_service::TransferService;
use crate::domain::transfer::{DbTableEndpoint, DbTransferOptions, TransferCancellation, TransferJob, TransferStatus};

fn cols(pairs: &[(&str, &str)]) -> Vec<DbColumnSpec> {
    pairs
        .iter()
        .map(|(n, t)| DbColumnSpec {
            name: (*n).into(),
            type_name: (*t).into(),
        })
        .collect()
}

#[test]
fn same_provider_integer_copy_is_identity() {
    let plan = build_conversion_plan(
        DriverType::Postgres,
        DriverType::Postgres,
        &cols(&[("id", "integer"), ("name", "text")]),
        &cols(&[("id", "integer"), ("name", "text")]),
        &TransferMapping::default(),
    )
    .unwrap();
    assert!(!plan.blocked);
    assert!(plan.columns.iter().all(|c| c.kind == ConversionKind::Identity));
}

#[test]
fn pg_to_sqlite_uuid_uses_text_bridge_and_is_previewable() {
    let plan = build_conversion_plan(
        DriverType::Postgres,
        DriverType::SQLite,
        &cols(&[("id", "uuid"), ("payload", "jsonb")]),
        &cols(&[("id", "text"), ("payload", "text")]),
        &TransferMapping::default(),
    )
    .unwrap();
    assert!(!plan.blocked);
    assert!(plan.columns.iter().all(|c| c.kind == ConversionKind::TextBridge));
    assert!(!plan.warnings.is_empty());
    plan.ensure_runnable().unwrap();
}

#[test]
fn unsupported_pg_array_blocks_plan() {
    let plan = build_conversion_plan(
        DriverType::Postgres,
        DriverType::SQLite,
        &cols(&[("tags", "text[]")]),
        &cols(&[("tags", "text")]),
        &TransferMapping::default(),
    )
    .unwrap();
    assert!(plan.blocked);
    assert!(plan.ensure_runnable().is_err());
}

#[test]
fn mysql_cross_provider_is_refused() {
    let plan = build_conversion_plan(
        DriverType::Mysql,
        DriverType::Postgres,
        &cols(&[("id", "int")]),
        &cols(&[("id", "integer")]),
        &TransferMapping::default(),
    )
    .unwrap();
    assert!(plan.blocked);
}

#[test]
fn capability_gates_are_independent() {
    assert!(assert_endpoint_capabilities(true, true, false, false).is_ok());
    assert!(assert_endpoint_capabilities(false, true, false, false).is_err());
    assert!(assert_endpoint_capabilities(true, false, false, false).is_err());
    assert!(assert_endpoint_capabilities(true, true, true, false).is_err());
    assert!(assert_endpoint_capabilities(true, true, true, true).is_ok());
}

#[test]
fn same_provider_table_copy_streams_with_bounded_batches() {
    let mut job = TransferJob::new_db_table_copy(
        "db-1",
        DbTableEndpoint {
            connection_id: "src".into(),
            schema: "public".into(),
            table: "orders".into(),
        },
        DbTableEndpoint {
            connection_id: "dst".into(),
            schema: "public".into(),
            table: "orders_copy".into(),
        },
        64,
        DbTransferOptions::default(),
    );
    let mut source = GeneratorTableSource::new(5_000);
    let mut target = MemoryTableTarget::new(TransferConflictPolicy::Abort, TransferTransactionPolicy::PerBatch);
    let cancel = TransferCancellation::new();
    let result = TransferService::run(&mut job, &mut source, &mut target, &cancel);
    assert_eq!(result.status, TransferStatus::Succeeded);
    assert_eq!(result.progress.rows_written, 5_000);
    assert_eq!(target.committed.len(), 5_000);
    assert_eq!(job.batch_size, 64);
    assert!(result.progress.committed_batches > 0);
}

#[test]
fn conflict_skip_counts_error_rows_and_continues() {
    let mut target = MemoryTableTarget::new(TransferConflictPolicy::Skip, TransferTransactionPolicy::PerBatch);
    target.seed([TransferRow {
        cells: vec!["0".into(), "existing".into()],
    }]);
    let mut source = GeneratorTableSource::new(3);
    let mut job = TransferJob::new_db_table_copy(
        "db-skip",
        DbTableEndpoint {
            connection_id: "s".into(),
            schema: "main".into(),
            table: "t".into(),
        },
        DbTableEndpoint {
            connection_id: "d".into(),
            schema: "main".into(),
            table: "t".into(),
        },
        10,
        DbTransferOptions {
            conflict: TransferConflictPolicy::Skip,
            transaction: TransferTransactionPolicy::PerBatch,
            create_target_if_missing: false,
        },
    );
    let cancel = TransferCancellation::new();
    let result = TransferService::run(&mut job, &mut source, &mut target, &cancel);
    assert!(matches!(
        result.status,
        TransferStatus::Succeeded | TransferStatus::Partial
    ));
    assert_eq!(target.error_rows, 1);
    assert_eq!(target.committed.len(), 3);
}

#[test]
fn all_or_nothing_cancel_reports_zero_committed() {
    let mut source = GeneratorTableSource::new(1_000);
    let mut target = MemoryTableTarget::new(TransferConflictPolicy::Abort, TransferTransactionPolicy::AllOrNothing);
    let mut job = TransferJob::new_db_table_copy(
        "db-aon",
        DbTableEndpoint {
            connection_id: "s".into(),
            schema: "public".into(),
            table: "t".into(),
        },
        DbTableEndpoint {
            connection_id: "d".into(),
            schema: "public".into(),
            table: "t".into(),
        },
        50,
        DbTransferOptions {
            conflict: TransferConflictPolicy::Abort,
            transaction: TransferTransactionPolicy::AllOrNothing,
            create_target_if_missing: false,
        },
    );
    let cancel = TransferCancellation::new();
    // First batch into open txn, then cancel.
    job.status = TransferStatus::Running;
    let first = source.next_batch(job.batch_size).unwrap().unwrap();
    job.progress.rows_read += first.len() as u64;
    let written = target.write_batch(&first).unwrap();
    job.progress.rows_written += written;
    assert_eq!(target.committed.len(), 0);
    assert!(target.uncommitted_rows() > 0);
    cancel.cancel();
    let result = TransferService::run(&mut job, &mut source, &mut target, &cancel);
    assert_eq!(result.status, TransferStatus::Cancelled);
    assert_eq!(target.committed.len(), 0);
    assert_eq!(target.uncommitted_rows(), 0);
    assert_eq!(result.progress.committed_batches, 0);
}

#[test]
fn per_batch_cancel_keeps_already_committed_batches() {
    let mut source = GeneratorTableSource::new(1_000);
    let mut target = MemoryTableTarget::new(TransferConflictPolicy::Abort, TransferTransactionPolicy::PerBatch);
    let mut job = TransferJob::new_db_table_copy(
        "db-pb",
        DbTableEndpoint {
            connection_id: "s".into(),
            schema: "public".into(),
            table: "t".into(),
        },
        DbTableEndpoint {
            connection_id: "d".into(),
            schema: "public".into(),
            table: "t".into(),
        },
        50,
        DbTransferOptions::default(),
    );
    let cancel = TransferCancellation::new();
    job.status = TransferStatus::Running;
    let first = source.next_batch(job.batch_size).unwrap().unwrap();
    let written = target.write_batch(&first).unwrap();
    job.progress.rows_read += first.len() as u64;
    job.progress.rows_written += written;
    let committed_before = target.committed.len();
    assert!(committed_before > 0);
    cancel.cancel();
    let result = TransferService::run(&mut job, &mut source, &mut target, &cancel);
    assert_eq!(result.status, TransferStatus::Cancelled);
    assert_eq!(target.committed.len(), committed_before);
    assert!(result.progress.committed_batches >= 1);
}
