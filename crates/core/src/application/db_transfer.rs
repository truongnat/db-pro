//! Database-to-database streaming transfer (#209).
//!
//! Conversion plans are previewable before any write. Adapters stream in bounded
//! batches and report committed vs uncommitted work according to
//! [`TransferTransactionPolicy`].

use crate::domain::transfer::{TransferConflictPolicy, TransferError, TransferRow, TransferTransactionPolicy};

use super::transfer_service::{TransferSource, TransferTarget};

#[path = "db_transfer_plan.rs"]
mod db_transfer_plan;

pub use db_transfer_plan::{
    assert_endpoint_capabilities, build_conversion_plan, project_row, ColumnConversion, ConversionKind, ConversionPlan,
    DbColumnSpec,
};

#[cfg(test)]
use crate::domain::connection::DriverType;
#[cfg(test)]
use crate::domain::transfer::TransferMapping;

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

#[cfg(test)]
#[path = "tests/db_transfer_tests.rs"]
mod tests;
