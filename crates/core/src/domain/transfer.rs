//! Streaming transfer job model (#193 / Phase C01).

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    /// Completed with some rejected/error rows but not aborted.
    Partial,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferSourceKind {
    Query { sql: String },
    Table { schema: String, table: String },
    Synthetic { rows: u64 },
    File { path: String, format: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferTargetKind {
    Table {
        schema: String,
        table: String,
    },
    File {
        path: String,
        format: String,
    },
    /// Harness sink that counts rows without materializing them.
    Counting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TransferMapping {
    /// Source column index → target column name (empty = drop).
    pub columns: Vec<(usize, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TransferProgress {
    pub rows_read: u64,
    pub rows_written: u64,
    pub bytes_written: u64,
    pub error_rows: u64,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferResult {
    pub status: TransferStatus,
    pub progress: TransferProgress,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferJob {
    pub id: String,
    pub label: String,
    pub source: TransferSourceKind,
    pub target: TransferTargetKind,
    pub mapping: TransferMapping,
    pub batch_size: usize,
    pub status: TransferStatus,
    pub progress: TransferProgress,
    pub error: Option<String>,
}

impl TransferJob {
    pub fn new_synthetic(id: impl Into<String>, rows: u64, batch_size: usize) -> Self {
        Self {
            id: id.into(),
            label: format!("Synthetic transfer ({rows} rows)"),
            source: TransferSourceKind::Synthetic { rows },
            target: TransferTargetKind::Counting,
            mapping: TransferMapping::default(),
            batch_size: batch_size.max(1),
            status: TransferStatus::Pending,
            progress: TransferProgress::default(),
            error: None,
        }
    }
}

/// Shared cancellation flag for a running transfer.
#[derive(Debug, Clone, Default)]
pub struct TransferCancellation {
    flag: Arc<AtomicBool>,
}

impl TransferCancellation {
    pub fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferRow {
    pub cells: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferError {
    Cancelled,
    Source(String),
    Target(String),
    Mapping(String),
}

impl std::fmt::Display for TransferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => write!(f, "transfer cancelled"),
            Self::Source(msg) => write!(f, "source: {msg}"),
            Self::Target(msg) => write!(f, "target: {msg}"),
            Self::Mapping(msg) => write!(f, "mapping: {msg}"),
        }
    }
}

impl std::error::Error for TransferError {}
