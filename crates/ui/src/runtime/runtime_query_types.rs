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
