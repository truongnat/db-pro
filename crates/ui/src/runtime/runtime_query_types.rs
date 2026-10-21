//! Query/result-facing UI runtime models.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiColumn {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiCell {
    Null,
    Boolean(bool),
    Number(String),
    Text(String),
    Json(String),
    Bytes(String),
}

/// Byte cap the runtime applies when mapping provider values into [`UiCell`] (#537).
///
/// Text and JSON keep a bounded prefix; bytea values keep half as many source
/// bytes because they cross the bridge hex-encoded.
pub const MAX_CELL_TEXT_BYTES: usize = 65_536;

impl UiCell {
    /// `true` when the runtime elided this value for transport.
    ///
    /// A truncated text/JSON value is a char-boundary prefix of at most
    /// [`MAX_CELL_TEXT_BYTES`] with a trailing `…`; a truncated bytea value is
    /// a hex render of that size, also ending in `…`. No untruncated runtime
    /// value can take this shape, so writes must never stage it back — doing so
    /// would replace the full value with its prefix. The check is deliberately
    /// conservative: a user-typed value that happens to be that large and ends
    /// in `…` is treated as truncated, which blocks the edit instead of risking
    /// silent data loss.
    pub fn is_truncated(&self) -> bool {
        match self {
            Self::Text(value) | Self::Json(value) => value.len() >= MAX_CELL_TEXT_BYTES && value.ends_with('…'),
            Self::Bytes(value) => value.len() > MAX_CELL_TEXT_BYTES && value.ends_with('…'),
            Self::Null | Self::Boolean(_) | Self::Number(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiQueryResult {
    pub columns: Vec<UiColumn>,
    pub rows: Vec<Vec<UiCell>>,
    pub row_count: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiQueryError {
    pub code: String,
    pub message: String,
    pub position: Option<usize>,
    pub detail: Option<String>,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiStatementOutput {
    pub statement_index: usize,
    pub result_set: Option<UiQueryResult>,
    pub affected_rows: Option<u64>,
    pub duration_ms: u64,
    pub message: Option<String>,
    pub error: Option<UiQueryError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiQueryExecutionOutput {
    pub statements: Vec<UiStatementOutput>,
    pub total_duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiQueryHistoryStatus {
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiQueryHistoryEntry {
    pub id: String,
    pub sql: String,
    pub connection_id: Option<String>,
    pub schema: Option<String>,
    pub started_at: String,
    pub duration_ms: u64,
    pub status: UiQueryHistoryStatus,
    pub row_count: Option<u64>,
    pub affected_rows: Option<u64>,
    pub error_code: Option<String>,
    pub error_summary: Option<String>,
}

#[cfg(test)]
#[path = "tests/runtime_query_types_tests.rs"]
mod tests;
