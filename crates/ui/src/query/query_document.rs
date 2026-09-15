use crate::editor::buffer::TextBuffer;
use crate::editor::completion::CompletionState;
use crate::editor::cursor::CursorPosition;
use crate::editor::diagnostics::Diagnostic;
use crate::editor::document::SqlDocumentAnalysis;
use crate::editor::prediction::EditPrediction;
use crate::editor::selection::SelectionRange;
use crate::editor::syntax::{CachedSqlTokens, SqlDialect};
use crate::runtime::{UiQueryExecutionOutput, UiQueryResult};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub const SQL_PREDICTION_DEBOUNCE: Duration = Duration::from_millis(300);
const SQL_PREDICTION_CACHE_TTL: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct CachedPrediction {
    pub fingerprint: u64,
    pub expires_at: Instant,
    pub prediction: EditPrediction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueryExecutionState {
    #[default]
    Idle,
    Running(crate::runtime::RequestId),
    Completed {
        affected_rows: u64,
        duration_ms: u64,
    },
    Failed,
}

#[derive(Debug, Clone, Default)]
pub struct EditorSearchState {
    pub query: String,
    pub is_open: bool,
    pub matches: Vec<(usize, usize)>,
    pub active_match_index: usize,
}

impl EditorSearchState {
    pub fn update_matches(&mut self, text: &str) {
        self.matches.clear();
        if self.query.is_empty() {
            self.active_match_index = 0;
            return;
        }
        let q = self.query.to_lowercase();
        let t = text.to_lowercase();
        for (idx, _) in t.match_indices(&q) {
            self.matches.push((idx, idx + q.len()));
        }
        if self.active_match_index >= self.matches.len() {
            self.active_match_index = 0;
        }
    }

    pub fn next_match(&mut self) -> Option<(usize, usize)> {
        if self.matches.is_empty() {
            return None;
        }
        self.active_match_index = (self.active_match_index + 1) % self.matches.len();
        self.matches.get(self.active_match_index).copied()
    }

    pub fn prev_match(&mut self) -> Option<(usize, usize)> {
        if self.matches.is_empty() {
            return None;
        }
        if self.active_match_index == 0 {
            self.active_match_index = self.matches.len().saturating_sub(1);
        } else {
            self.active_match_index -= 1;
        }
        self.matches.get(self.active_match_index).copied()
    }

    pub fn current_match(&self) -> Option<(usize, usize)> {
        self.matches.get(self.active_match_index).copied()
    }
}

#[derive(Debug, Clone)]
pub struct QueryDocument {
    pub id: String,
    pub title: String,
    pub buffer: TextBuffer,
    pub cursor: CursorPosition,
    pub selection: SelectionRange,
    pub scroll: f32,
    pub connection_id: Option<String>,
    pub schema: Option<String>,
    pub dirty: bool,
    pub saved_version: u64,
    pub saved_snapshot: String,
    pub saved_query_id: Option<String>,
    /// Absolute path when this document is backed by a workspace `.sql` file (#262).
    pub file_path: Option<String>,
    pub execution_state: QueryExecutionState,
    pub analysis: SqlDocumentAnalysis,
    pub diagnostics: Vec<Diagnostic>,
    pub completion: CompletionState,
    pub prediction: Option<EditPrediction>,
    pub pending_prediction_request: Option<crate::runtime::RequestId>,
    pub prediction_debounce_deadline: Option<Instant>,
    pub prediction_manual: bool,
    pub prediction_reveal: bool,
    pub prediction_scheduled_at: Option<Instant>,
    pub prediction_context_fingerprint: Option<u64>,
    pub prediction_last_request_fingerprint: Option<u64>,
    pub prediction_last_request_at: Option<Instant>,
    pub prediction_cache: Option<CachedPrediction>,
    pub prediction_request_started_at: Option<Instant>,
    pub prediction_requests_sent: u64,
    pub prediction_requests_deduped: u64,
    pub prediction_requests_cancelled: u64,
    pub prediction_stale_responses_dropped: u64,
    pub prediction_accepted: u64,
    pub prediction_partially_accepted: u64,
    pub prediction_rejected: u64,
    pub prediction_last_latency_ms: Option<u64>,
    pub prediction_last_debounce_ms: Option<u64>,
    pub cached_tokens: CachedSqlTokens,
    pub search: EditorSearchState,
    pub query_result: Option<UiQueryResult>,
    pub query_results: Vec<UiQueryResult>,
    pub active_result_index: usize,
    pub execution_output: Option<UiQueryExecutionOutput>,
    pub query_messages: Vec<String>,
    pub explain_plan: Option<String>,
    pub explain_request: Option<crate::runtime::RequestId>,
    pub chart_config: crate::ChartConfig,
    /// In-memory parameter drafts for the current SQL template (#225). Secrets are
    /// never persisted with the document snapshot.
    pub parameter_values: HashMap<String, String>,
    pub parameter_secrets: std::collections::BTreeSet<String>,
    pub executing_range: Option<(usize, usize)>,
    pub executing_sql: Option<String>,
    pub executing_version: Option<u64>,
    pub last_executed_range: Option<(usize, usize)>,
    pub execution_diagnostic: Option<Diagnostic>,
    pub execution_started_at: Option<Instant>,
    /// Wall-clock start time for local query history. Kept separate from the
    /// monotonic timer used for duration measurement and is never persisted.
    pub execution_started_wall_time: Option<String>,
}

impl QueryDocument {
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        let content_str = content.into();
        let saved_snapshot = content_str.clone();
        let buffer = TextBuffer::from_string(content_str);
        let analysis = SqlDocumentAnalysis::analyze(&buffer, SqlDialect::Postgres);
        Self {
            id: id.into(),
            title: title.into(),
            buffer,
            cursor: CursorPosition::default(),
            selection: SelectionRange::default(),
            scroll: 0.0,
            connection_id: None,
            schema: None,
            dirty: false,
            saved_version: 0,
            saved_snapshot,
            saved_query_id: None,
            file_path: None,
            execution_state: QueryExecutionState::Idle,
            analysis,
            diagnostics: Vec::new(),
            completion: CompletionState::new(),
            prediction: None,
            pending_prediction_request: None,
            prediction_debounce_deadline: None,
            prediction_manual: false,
            prediction_reveal: false,
            prediction_scheduled_at: None,
            prediction_context_fingerprint: None,
            prediction_last_request_fingerprint: None,
            prediction_last_request_at: None,
            prediction_cache: None,
            prediction_request_started_at: None,
            prediction_requests_sent: 0,
            prediction_requests_deduped: 0,
            prediction_requests_cancelled: 0,
            prediction_stale_responses_dropped: 0,
            prediction_accepted: 0,
            prediction_partially_accepted: 0,
            prediction_rejected: 0,
            prediction_last_latency_ms: None,
            prediction_last_debounce_ms: None,
            cached_tokens: CachedSqlTokens::new(),
            search: EditorSearchState::default(),
            query_result: None,
            query_results: Vec::new(),
            active_result_index: 0,
            execution_output: None,
            query_messages: Vec::new(),
            explain_plan: None,
            explain_request: None,
            chart_config: crate::ChartConfig::new(),
            parameter_values: HashMap::new(),
            parameter_secrets: std::collections::BTreeSet::new(),
            executing_range: None,
            executing_sql: None,
            executing_version: None,
            last_executed_range: None,
            execution_diagnostic: None,
            execution_started_at: None,
            execution_started_wall_time: None,
        }
    }

    pub fn mark_saved(&mut self) {
        self.saved_version = self.buffer.version();
        self.saved_snapshot = self.buffer.text().to_owned();
        self.dirty = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.buffer.text() != self.saved_snapshot
    }

    pub fn text(&self) -> &str {
        self.buffer.text()
    }

    pub fn content(&self) -> &str {
        self.buffer.text()
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.buffer.set_text(text);
        self.cursor.set_offset(&self.buffer, 0);
        self.selection.collapse_to_active();
        self.reanalyze(SqlDialect::Postgres);
        self.search.update_matches(self.buffer.text());
        self.dirty = true;
    }

    pub fn reanalyze(&mut self, dialect: SqlDialect) {
        if self.analysis.version != self.buffer.version() {
            self.analysis = SqlDocumentAnalysis::analyze(&self.buffer, dialect);
            self.search.update_matches(self.buffer.text());
        }
    }

    /// Resolves the executable SQL: selected text if non-empty, otherwise the current statement at cursor, otherwise full buffer.
    pub fn resolve_executable_sql(&self) -> (String, (usize, usize)) {
        if !self.selection.is_empty() {
            let (start, end) = self.selection.normalized();
            let sql = self.buffer.slice(start, end).trim().to_owned();
            return (sql, (start, end));
        }

        if let Some(stmt) = self.analysis.current_statement_at(self.cursor.offset) {
            return (stmt.text.clone(), stmt.range);
        }

        let full = self.buffer.text().trim().to_owned();
        let len = self.buffer.len_bytes();
        (full, (0, len))
    }

    /// Returns the executable SQL and the exact document range after trimming
    /// editor-only whitespace from the selected/current statement.
    pub fn resolve_executable_range(&self) -> (String, (usize, usize)) {
        let (sql, (start, end)) = self.resolve_executable_sql();
        let raw = self.buffer.slice(start, end);
        let leading = raw.len().saturating_sub(raw.trim_start().len());
        let trimmed_start = start + leading;
        let sql_len = sql.len();
        (sql, (trimmed_start, trimmed_start + sql_len))
    }

    pub fn schedule_prediction(&mut self, now: Instant) {
        self.schedule_prediction_with_mode(now, false);
    }

    pub fn schedule_prediction_with_mode(&mut self, now: Instant, manual: bool) {
        self.prediction_debounce_deadline = Some(if manual { now } else { now + SQL_PREDICTION_DEBOUNCE });
        self.prediction_manual = manual;
        self.prediction_reveal = manual;
        self.prediction_scheduled_at = Some(now);
        self.prediction = None;
    }

    pub fn prediction_is_due(&self, now: Instant) -> bool {
        self.prediction_debounce_deadline
            .is_some_and(|deadline| now >= deadline)
    }

    pub fn take_prediction_schedule(&mut self) -> bool {
        self.prediction_debounce_deadline.take().is_some()
    }

    pub fn take_prediction_manual(&mut self) -> bool {
        std::mem::replace(&mut self.prediction_manual, false)
    }

    pub fn take_cached_prediction(&mut self, fingerprint: u64, now: Instant) -> Option<EditPrediction> {
        let cached = self.prediction_cache.as_ref()?;
        if cached.fingerprint != fingerprint || cached.expires_at < now {
            self.prediction_cache = None;
            return None;
        }
        Some(cached.prediction.clone())
    }

    pub fn cache_prediction(&mut self, fingerprint: u64, prediction: EditPrediction, now: Instant) {
        self.prediction_cache = Some(CachedPrediction {
            fingerprint,
            expires_at: now + SQL_PREDICTION_CACHE_TTL,
            prediction,
        });
    }

    pub fn should_dedupe_prediction(&mut self, fingerprint: u64, now: Instant) -> bool {
        let Some(last_fingerprint) = self.prediction_last_request_fingerprint else {
            return false;
        };
        let Some(last_request_at) = self.prediction_last_request_at else {
            return false;
        };
        let elapsed = now.saturating_duration_since(last_request_at);
        if last_fingerprint == fingerprint && elapsed < SQL_PREDICTION_CACHE_TTL {
            return true;
        }
        if elapsed >= SQL_PREDICTION_CACHE_TTL {
            self.prediction_last_request_fingerprint = None;
            self.prediction_last_request_at = None;
        }
        false
    }

    pub fn invalidate_prediction(&mut self) {
        self.prediction_debounce_deadline = None;
        self.prediction = None;
        self.pending_prediction_request = None;
        self.prediction_manual = false;
        self.prediction_reveal = false;
        self.prediction_scheduled_at = None;
        self.prediction_context_fingerprint = None;
        self.prediction_request_started_at = None;
    }

    pub fn take_execution_timing(&mut self, fallback_duration_ms: u64) -> (String, u64) {
        let started_at = self.execution_started_at.take();
        let started_at_wall_time = self
            .execution_started_wall_time
            .take()
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
        let duration_ms = started_at.map_or(fallback_duration_ms, |started_at| {
            started_at.elapsed().as_millis() as u64
        });
        (started_at_wall_time, duration_ms)
    }
}

// Custom serialization for storage persistence
impl Serialize for QueryDocument {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct SerializedQueryDocument<'a> {
            id: &'a str,
            title: &'a str,
            content: &'a str,
            connection_id: Option<&'a str>,
            schema: Option<&'a str>,
            cursor_offset: usize,
            selection_anchor: usize,
            selection_active: usize,
            scroll: f32,
            saved_query_id: Option<&'a str>,
            saved_snapshot: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            file_path: Option<&'a str>,
        }
        let helper = SerializedQueryDocument {
            id: &self.id,
            title: &self.title,
            content: self.buffer.text(),
            connection_id: self.connection_id.as_deref(),
            schema: self.schema.as_deref(),
            cursor_offset: self.cursor.offset,
            selection_anchor: self.selection.anchor,
            selection_active: self.selection.active,
            scroll: self.scroll,
            saved_query_id: self.saved_query_id.as_deref(),
            saved_snapshot: &self.saved_snapshot,
            file_path: self.file_path.as_deref(),
        };
        helper.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for QueryDocument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DeserializedQueryDocument {
            #[serde(default)]
            id: String,
            title: String,
            content: String,
            #[serde(default)]
            connection_id: Option<String>,
            #[serde(default)]
            schema: Option<String>,
            #[serde(default)]
            cursor_offset: usize,
            #[serde(default)]
            selection_anchor: usize,
            #[serde(default)]
            selection_active: usize,
            #[serde(default)]
            scroll: f32,
            #[serde(default)]
            saved_query_id: Option<String>,
            #[serde(default)]
            saved_snapshot: Option<String>,
            #[serde(default)]
            file_path: Option<String>,
        }
        let helper = DeserializedQueryDocument::deserialize(deserializer)?;
        let id = if helper.id.is_empty() {
            format!("query-{}", helper.title.to_lowercase().replace(' ', "-"))
        } else {
            helper.id
        };
        let mut doc = QueryDocument::new(id, helper.title, helper.content);
        doc.connection_id = helper.connection_id;
        doc.schema = helper.schema;
        doc.saved_query_id = helper.saved_query_id;
        doc.file_path = helper.file_path;
        doc.cursor.set_offset(&doc.buffer, helper.cursor_offset);
        doc.selection.anchor = helper.selection_anchor;
        doc.selection.active = helper.selection_active;
        doc.scroll = helper.scroll;
        doc.saved_snapshot = helper.saved_snapshot.unwrap_or_else(|| doc.buffer.text().to_owned());
        doc.saved_version = doc.buffer.version();
        doc.dirty = false;
        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_document_resolve_executable_sql() {
        let mut doc = QueryDocument::new("doc-1", "Doc 1", "SELECT 1;\nSELECT 2;\nSELECT 3;");

        // 1. Cursor on first statement
        doc.cursor.set_offset(&doc.buffer, 3);
        let (sql, _) = doc.resolve_executable_sql();
        assert_eq!(sql, "SELECT 1;");

        // 2. Cursor on second statement
        doc.cursor.set_offset(&doc.buffer, 12);
        let (sql, _) = doc.resolve_executable_sql();
        assert_eq!(sql, "SELECT 2;");

        // 3. Selection active
        doc.selection.anchor = 0;
        doc.selection.active = 19;
        let (sql, _) = doc.resolve_executable_sql();
        assert_eq!(sql, "SELECT 1;\nSELECT 2;");
    }

    #[test]
    fn test_query_document_search_matches() {
        let mut doc = QueryDocument::new("doc-1", "Doc 1", "SELECT id, name FROM users WHERE id = 1;");
        doc.search.query = "id".to_owned();
        let text = doc.text().to_owned();
        doc.search.update_matches(&text);
        assert_eq!(doc.search.matches.len(), 2);
        assert_eq!(doc.search.matches[0], (7, 9));
        assert_eq!(doc.search.matches[1], (33, 35));

        let next = doc.search.next_match();
        assert_eq!(next, Some((33, 35)));
        let prev = doc.search.prev_match();
        assert_eq!(prev, Some((7, 9)));
    }

    #[test]
    fn test_query_document_serde_roundtrip() {
        let mut doc = QueryDocument::new("q-1", "My Query", "SELECT * FROM users;");
        doc.connection_id = Some("conn-123".to_owned());
        doc.schema = Some("public".to_owned());
        doc.cursor.set_offset(&doc.buffer, 14);
        doc.selection = crate::editor::SelectionRange::new(7, 14);
        doc.scroll = 42.5;

        let serialized = serde_json::to_string(&doc).unwrap();
        let deserialized: QueryDocument = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, "q-1");
        assert_eq!(deserialized.title, "My Query");
        assert_eq!(deserialized.text(), "SELECT * FROM users;");
        assert_eq!(deserialized.connection_id.as_deref(), Some("conn-123"));
        assert_eq!(deserialized.schema.as_deref(), Some("public"));
        assert_eq!(deserialized.cursor.offset, 14);
        assert_eq!(deserialized.selection.anchor, 7);
        assert_eq!(deserialized.selection.active, 14);
        assert_eq!(deserialized.scroll, 42.5);
        assert!(!deserialized.is_dirty());
    }

    #[test]
    fn undoing_to_saved_snapshot_clears_dirty_state() {
        let mut doc = QueryDocument::new("q-1", "Query", "SELECT 1;");
        doc.mark_saved();

        doc.set_text("SELECT 2;");
        assert!(doc.is_dirty());

        doc.set_text("SELECT 1;");
        assert!(!doc.is_dirty());
    }

    #[test]
    fn prediction_schedule_is_due_only_after_debounce_and_can_be_consumed() {
        let mut doc = QueryDocument::new("doc-1", "Doc 1", "SELECT 1");
        let now = Instant::now();
        doc.schedule_prediction(now);
        assert!(!doc.prediction_is_due(now));
        assert!(doc.prediction_is_due(now + SQL_PREDICTION_DEBOUNCE));
        assert!(doc.take_prediction_schedule());
        assert!(!doc.prediction_is_due(now + SQL_PREDICTION_DEBOUNCE));
    }

    #[test]
    fn cached_prediction_is_reused_only_for_the_same_fingerprint() {
        let mut doc = QueryDocument::new("doc-1", "Doc 1", "SELECT ");
        let prediction = EditPrediction::new(7, "* FROM users", None);
        let now = Instant::now();
        doc.cache_prediction(41, prediction.clone(), now);
        assert_eq!(doc.take_cached_prediction(41, now).as_ref(), Some(&prediction));
        assert!(doc.take_cached_prediction(42, now).is_none());
    }

    #[test]
    fn manual_prediction_schedule_bypasses_debounce() {
        let mut doc = QueryDocument::new("doc-1", "Doc 1", "SELECT ");
        let now = Instant::now();
        doc.schedule_prediction_with_mode(now, true);
        assert!(doc.prediction_is_due(now));
        assert!(doc.take_prediction_manual());
    }

    #[test]
    fn prediction_request_is_deduped_within_cache_window() {
        let mut doc = QueryDocument::new("doc-1", "Doc 1", "SELECT ");
        let now = Instant::now();
        doc.prediction_last_request_fingerprint = Some(7);
        doc.prediction_last_request_at = Some(now);
        assert!(doc.should_dedupe_prediction(7, now));
        assert!(!doc.should_dedupe_prediction(8, now));
    }
}
