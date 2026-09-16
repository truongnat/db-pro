//! Database-to-database streaming transfer (#209).
//!
//! Conversion plans are previewable before any write. Adapters stream in bounded
//! batches and report committed vs uncommitted work according to
//! [`TransferTransactionPolicy`].

use crate::domain::connection::DriverType;
use crate::domain::transfer::{
    TransferConflictPolicy, TransferError, TransferMapping, TransferRow, TransferTransactionPolicy,
};

use super::transfer_service::{TransferSource, TransferTarget};

/// Column identity used when building a conversion plan (no secrets).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbColumnSpec {
    pub name: String,
    /// Provider type name as reported by introspection (e.g. `integer`, `text`, `uuid`).
    pub type_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionKind {
    Identity,
    /// Safe widening / dialect-equivalent cast.
    SafeCast,
    /// Values go through text; may be lossy for binary/exotic types.
    TextBridge,
    /// Explicitly refused — job must not start without remapping.
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnConversion {
    pub source_index: usize,
    pub source_name: String,
    pub source_type: String,
    pub target_name: String,
    pub target_type: String,
    pub kind: ConversionKind,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionPlan {
    pub source_driver: DriverType,
    pub target_driver: DriverType,
    pub columns: Vec<ColumnConversion>,
    pub warnings: Vec<String>,
    pub blocked: bool,
}

impl ConversionPlan {
    pub fn ensure_runnable(&self) -> Result<(), TransferError> {
        if self.blocked {
            let detail = self
                .columns
                .iter()
                .filter(|c| c.kind == ConversionKind::Unsupported)
                .map(|c| {
                    format!(
                        "{}:{} → {}:{} ({})",
                        c.source_name,
                        c.source_type,
                        c.target_name,
                        c.target_type,
                        c.warning.as_deref().unwrap_or("unsupported")
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            return Err(TransferError::Conversion(format!(
                "transfer blocked by unsupported conversions: {detail}"
            )));
        }
        Ok(())
    }
}

/// Normalize introspection type names for comparison.
fn normalize_type(raw: &str) -> String {
    raw.trim().to_ascii_lowercase().replace("character varying", "varchar")
}

fn classify_pair(
    source_driver: DriverType,
    target_driver: DriverType,
    source_ty: &str,
    target_ty: &str,
) -> (ConversionKind, Option<String>) {
    let src = normalize_type(source_ty);
    let dst = normalize_type(target_ty);

    if source_driver == target_driver {
        if src == dst || type_aliases(&src, &dst) {
            return (ConversionKind::Identity, None);
        }
        if same_provider_safe_cast(&src, &dst) {
            return (ConversionKind::SafeCast, Some(format!("cast {src} → {dst}")));
        }
        return (
            ConversionKind::Unsupported,
            Some(format!("same-provider type mismatch {src} → {dst}")),
        );
    }

    // Explicitly supported cross-provider combinations only.
    match (source_driver, target_driver) {
        (DriverType::Postgres, DriverType::SQLite) | (DriverType::SQLite, DriverType::Postgres) => {
            pg_sqlite_conversion(&src, &dst)
        }
        (DriverType::Mysql, _) | (_, DriverType::Mysql) => (
            ConversionKind::Unsupported,
            Some("MySQL database-to-database transfer is not enabled yet".into()),
        ),
        _ => (
            ConversionKind::Unsupported,
            Some(format!(
                "no conversion matrix for {source_driver:?} → {target_driver:?}"
            )),
        ),
    }
}

fn type_aliases(a: &str, b: &str) -> bool {
    matches!(
        (a, b),
        ("int", "integer")
            | ("integer", "int")
            | ("int4", "integer")
            | ("integer", "int4")
            | ("int8", "bigint")
            | ("bigint", "int8")
            | ("bool", "boolean")
            | ("boolean", "bool")
            | ("varchar", "text")
            | ("text", "varchar")
            | ("character varying", "text")
    )
}

fn same_provider_safe_cast(src: &str, dst: &str) -> bool {
    matches!(
        (src, dst),
        ("smallint", "integer")
            | ("integer", "bigint")
            | ("real", "double precision")
            | ("float", "double")
            | ("varchar", "text")
            | ("char", "text")
            | ("text", "varchar")
    )
}

fn pg_sqlite_conversion(src: &str, dst: &str) -> (ConversionKind, Option<String>) {
    // SQLite affinity classes vs common PG types — explicit matrix.
    let ok_identity = matches!(
        (src, dst),
        ("integer", "integer")
            | ("integer", "int")
            | ("int", "integer")
            | ("bigint", "integer")
            | ("text", "text")
            | ("varchar", "text")
            | ("text", "varchar")
            | ("real", "real")
            | ("double precision", "real")
            | ("boolean", "integer")
            | ("bool", "integer")
            | ("integer", "boolean")
            | ("blob", "bytea")
            | ("bytea", "blob")
            | ("numeric", "text")
            | ("text", "numeric")
            | ("uuid", "text")
            | ("text", "uuid")
            | ("json", "text")
            | ("jsonb", "text")
            | ("text", "json")
            | ("text", "jsonb")
            | ("timestamp", "text")
            | ("timestamptz", "text")
            | ("text", "timestamp")
            | ("date", "text")
            | ("text", "date")
    );
    if ok_identity || type_aliases(src, dst) {
        let kind = if src == dst || type_aliases(src, dst) {
            ConversionKind::Identity
        } else if matches!(
            (src, dst),
            ("uuid", "text")
                | ("json", "text")
                | ("jsonb", "text")
                | ("numeric", "text")
                | ("timestamp", "text")
                | ("timestamptz", "text")
                | ("date", "text")
                | ("text", "uuid")
                | ("text", "json")
                | ("text", "jsonb")
                | ("text", "numeric")
                | ("text", "timestamp")
                | ("text", "date")
                | ("boolean", "integer")
                | ("bool", "integer")
                | ("integer", "boolean")
        ) {
            ConversionKind::TextBridge
        } else {
            ConversionKind::SafeCast
        };
        let warning = match kind {
            ConversionKind::TextBridge => Some(format!(
                "PG↔SQLite bridge {src} → {dst} via text/affinity; verify values before production use"
            )),
            ConversionKind::SafeCast => Some(format!("PG↔SQLite cast {src} → {dst}")),
            _ => None,
        };
        return (kind, warning);
    }

    if matches!(src, "array" | "tsvector" | "xml" | "inet" | "cidr" | "macaddr") || src.contains("[]") {
        return (
            ConversionKind::Unsupported,
            Some(format!("PostgreSQL type `{src}` has no SQLite mapping")),
        );
    }

    (
        ConversionKind::Unsupported,
        Some(format!("no explicit PG↔SQLite mapping for {src} → {dst}")),
    )
}

/// Build a previewable conversion plan. Does not write.
pub fn build_conversion_plan(
    source_driver: DriverType,
    target_driver: DriverType,
    source_columns: &[DbColumnSpec],
    target_columns: &[DbColumnSpec],
    mapping: &TransferMapping,
) -> Result<ConversionPlan, TransferError> {
    let pairs: Vec<(usize, String)> = if mapping.columns.is_empty() {
        // Auto-match by name (case-insensitive), then by position.
        let mut auto = Vec::new();
        for (idx, src) in source_columns.iter().enumerate() {
            if let Some(tgt) = target_columns.iter().find(|t| t.name.eq_ignore_ascii_case(&src.name)) {
                auto.push((idx, tgt.name.clone()));
            } else if let Some(tgt) = target_columns.get(idx) {
                auto.push((idx, tgt.name.clone()));
            }
        }
        auto
    } else {
        mapping.columns.clone()
    };

    if pairs.is_empty() {
        return Err(TransferError::Mapping(
            "no column mappings resolved for database transfer".into(),
        ));
    }

    let mut columns = Vec::with_capacity(pairs.len());
    let mut warnings = Vec::new();
    let mut blocked = false;

    for (source_index, target_name) in pairs {
        if target_name.is_empty() {
            continue;
        }
        let source = source_columns
            .get(source_index)
            .ok_or_else(|| TransferError::Mapping(format!("source column index {source_index} out of range")))?;
        let target = target_columns
            .iter()
            .find(|c| c.name == target_name)
            .ok_or_else(|| TransferError::Mapping(format!("target column `{target_name}` not found")))?;

        let (kind, warning) = classify_pair(source_driver, target_driver, &source.type_name, &target.type_name);
        if kind == ConversionKind::Unsupported {
            blocked = true;
        }
        if let Some(ref w) = warning {
            warnings.push(format!("{} → {}: {w}", source.name, target.name));
        }
        columns.push(ColumnConversion {
            source_index,
            source_name: source.name.clone(),
            source_type: source.type_name.clone(),
            target_name: target.name.clone(),
            target_type: target.type_name.clone(),
            kind,
            warning,
        });
    }

    Ok(ConversionPlan {
        source_driver,
        target_driver,
        columns,
        warnings,
        blocked,
    })
}

/// Capability gate: both endpoints must allow table data mutation/read independently.
pub fn assert_endpoint_capabilities(
    source_allows_read: bool,
    target_allows_write: bool,
    create_target_requested: bool,
    target_allows_create_table: bool,
) -> Result<(), TransferError> {
    if !source_allows_read {
        return Err(TransferError::Source(
            "source connection does not allow reading table data for transfer".into(),
        ));
    }
    if !target_allows_write {
        return Err(TransferError::Target(
            "target connection does not allow writing table data for transfer".into(),
        ));
    }
    if create_target_requested && !target_allows_create_table {
        return Err(TransferError::Target(
            "create-target-table requested but target provider/capability forbids DDL".into(),
        ));
    }
    Ok(())
}

/// Streaming source over an in-memory row generator (bounded batches; never holds the full set).
pub struct GeneratorTableSource {
    remaining: u64,
    next_id: u64,
    /// Optional primary-key column values already present on the target (for conflict tests).
    cell_factory: fn(u64) -> Vec<String>,
}

impl GeneratorTableSource {
    pub fn new(rows: u64) -> Self {
        Self {
            remaining: rows,
            next_id: 0,
            cell_factory: |id| vec![id.to_string(), format!("row-{id}")],
        }
    }

    pub fn with_factory(rows: u64, cell_factory: fn(u64) -> Vec<String>) -> Self {
        Self {
            remaining: rows,
            next_id: 0,
            cell_factory,
        }
    }
}

impl TransferSource for GeneratorTableSource {
    fn next_batch(&mut self, max_rows: usize) -> Result<Option<Vec<TransferRow>>, TransferError> {
        if self.remaining == 0 {
            return Ok(None);
        }
        let take = (max_rows as u64).min(self.remaining) as usize;
        let mut batch = Vec::with_capacity(take);
        for _ in 0..take {
            batch.push(TransferRow {
                cells: (self.cell_factory)(self.next_id),
            });
            self.next_id += 1;
            self.remaining -= 1;
        }
        Ok(Some(batch))
    }
}

/// In-memory table target that models conflict + transaction policies without a live DB.
pub struct MemoryTableTarget {
    pub primary_key_index: usize,
    pub conflict: TransferConflictPolicy,
    pub transaction: TransferTransactionPolicy,
    /// Committed rows keyed by primary key cell.
    pub committed: std::collections::BTreeMap<String, TransferRow>,
    pending: Vec<TransferRow>,
    pub committed_batches: u64,
    pub error_rows: u64,
    pub bytes_written: u64,
    open_transaction: bool,
}

impl MemoryTableTarget {
    pub fn new(conflict: TransferConflictPolicy, transaction: TransferTransactionPolicy) -> Self {
        Self {
            primary_key_index: 0,
            conflict,
            transaction,
            committed: std::collections::BTreeMap::new(),
            pending: Vec::new(),
            committed_batches: 0,
            error_rows: 0,
            bytes_written: 0,
            open_transaction: false,
        }
    }

    pub fn seed(&mut self, rows: impl IntoIterator<Item = TransferRow>) {
        for row in rows {
            let key = row.cells.get(self.primary_key_index).cloned().unwrap_or_default();
            self.committed.insert(key, row);
        }
    }

    fn apply_row(&mut self, row: TransferRow) -> Result<bool, TransferError> {
        let key = row.cells.get(self.primary_key_index).cloned().unwrap_or_default();
        let exists = self.committed.contains_key(&key)
            || self
                .pending
                .iter()
                .any(|r| r.cells.get(self.primary_key_index).map(String::as_str) == Some(key.as_str()));

        if exists {
            match self.conflict {
                TransferConflictPolicy::Abort => {
                    return Err(TransferError::Target(format!(
                        "conflict on primary key `{key}` (policy=abort)"
                    )));
                }
                TransferConflictPolicy::Skip => {
                    self.error_rows = self.error_rows.saturating_add(1);
                    return Ok(false);
                }
                TransferConflictPolicy::Replace => {
                    self.pending
                        .retain(|r| r.cells.get(self.primary_key_index).map(String::as_str) != Some(key.as_str()));
                }
            }
        }

        for cell in &row.cells {
            self.bytes_written = self.bytes_written.saturating_add(cell.len() as u64);
        }
        self.pending.push(row);
        Ok(true)
    }

    fn commit_pending(&mut self) {
        for row in self.pending.drain(..) {
            let key = row.cells.get(self.primary_key_index).cloned().unwrap_or_default();
            self.committed.insert(key, row);
        }
        self.committed_batches = self.committed_batches.saturating_add(1);
        self.open_transaction = false;
    }

    pub fn uncommitted_rows(&self) -> u64 {
        self.pending.len() as u64
    }
}

impl TransferTarget for MemoryTableTarget {
    fn write_batch(&mut self, rows: &[TransferRow]) -> Result<u64, TransferError> {
        if matches!(
            self.transaction,
            TransferTransactionPolicy::AllOrNothing | TransferTransactionPolicy::PerBatch
        ) {
            self.open_transaction = true;
        }

        let mut written = 0u64;
        for row in rows {
            if self.apply_row(row.clone())? {
                written = written.saturating_add(1);
            }
        }

        match self.transaction {
            TransferTransactionPolicy::PerBatch | TransferTransactionPolicy::None => {
                self.commit_pending();
            }
            TransferTransactionPolicy::AllOrNothing => {
                // Keep pending until finish().
            }
        }
        Ok(written)
    }

    fn finish(&mut self) -> Result<(), TransferError> {
        if matches!(self.transaction, TransferTransactionPolicy::AllOrNothing) && !self.pending.is_empty() {
            self.commit_pending();
        }
        self.open_transaction = false;
        Ok(())
    }

    fn cleanup_partial(&mut self) -> Result<(), TransferError> {
        match self.transaction {
            TransferTransactionPolicy::AllOrNothing => {
                self.pending.clear();
                self.open_transaction = false;
            }
            TransferTransactionPolicy::PerBatch | TransferTransactionPolicy::None => {
                // Already-committed batches stay; only drop any open pending.
                self.pending.clear();
                self.open_transaction = false;
            }
        }
        Ok(())
    }

    fn committed_batches(&self) -> u64 {
        self.committed_batches
    }

    fn uncommitted_rows(&self) -> u64 {
        self.pending.len() as u64
    }

    fn error_rows(&self) -> u64 {
        self.error_rows
    }
}

/// Apply mapping to a source row → target-ordered cells for insert.
pub fn project_row(row: &TransferRow, plan: &ConversionPlan) -> Result<TransferRow, TransferError> {
    let mut cells = Vec::with_capacity(plan.columns.len());
    for column in &plan.columns {
        let value = row.cells.get(column.source_index).ok_or_else(|| {
            TransferError::Mapping(format!(
                "row missing source index {} for column {}",
                column.source_index, column.source_name
            ))
        })?;
        cells.push(value.clone());
    }
    Ok(TransferRow { cells })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::transfer_service::TransferService;
    use crate::domain::transfer::{
        DbTableEndpoint, DbTransferOptions, TransferCancellation, TransferJob, TransferStatus,
    };

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
}
