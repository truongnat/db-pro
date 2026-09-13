use crate::editor::buffer::TextBuffer;
use crate::editor::completion::CompletionState;
use crate::editor::cursor::CursorPosition;
use crate::editor::diagnostics::Diagnostic;
use crate::editor::document::SqlDocumentAnalysis;
use crate::editor::prediction::EditPrediction;
use crate::editor::selection::SelectionRange;
use crate::editor::syntax::{CachedSqlTokens, SqlDialect};
use crate::runtime::UiQueryResult;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::{Duration, Instant};

pub const SQL_PREDICTION_DEBOUNCE: Duration = Duration::from_millis(300);

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
    pub saved_query_id: Option<String>,
    pub execution_state: QueryExecutionState,
    pub analysis: SqlDocumentAnalysis,
    pub diagnostics: Vec<Diagnostic>,
    pub completion: CompletionState,
    pub prediction: Option<EditPrediction>,
    pub pending_prediction_request: Option<crate::runtime::RequestId>,
    pub prediction_debounce_deadline: Option<Instant>,
    pub cached_tokens: CachedSqlTokens,
    pub search: EditorSearchState,
    pub query_result: Option<UiQueryResult>,
    pub query_messages: Vec<String>,
    pub explain_plan: Option<String>,
    pub explain_request: Option<crate::runtime::RequestId>,
}

impl QueryDocument {
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        let content_str = content.into();
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
            saved_query_id: None,
            execution_state: QueryExecutionState::Idle,
            analysis,
            diagnostics: Vec::new(),
            completion: CompletionState::new(),
            prediction: None,
            pending_prediction_request: None,
            prediction_debounce_deadline: None,
            cached_tokens: CachedSqlTokens::new(),
            search: EditorSearchState::default(),
            query_result: None,
            query_messages: Vec::new(),
            explain_plan: None,
            explain_request: None,
        }
    }

    pub fn mark_saved(&mut self) {
        self.saved_version = self.buffer.version();
        self.dirty = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty || self.buffer.version() != self.saved_version
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

    pub fn schedule_prediction(&mut self, now: Instant) {
        self.prediction_debounce_deadline = Some(now + SQL_PREDICTION_DEBOUNCE);
        self.prediction = None;
    }

    pub fn prediction_is_due(&self, now: Instant) -> bool {
        self.prediction_debounce_deadline
            .is_some_and(|deadline| now >= deadline)
    }

    pub fn take_prediction_schedule(&mut self) -> bool {
        self.prediction_debounce_deadline.take().is_some()
    }

    pub fn invalidate_prediction(&mut self) {
        self.prediction_debounce_deadline = None;
        self.prediction = None;
        self.pending_prediction_request = None;
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
        doc.cursor.set_offset(&doc.buffer, helper.cursor_offset);
        doc.selection.anchor = helper.selection_anchor;
        doc.selection.active = helper.selection_active;
        doc.scroll = helper.scroll;
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
    fn prediction_schedule_is_due_only_after_debounce_and_can_be_consumed() {
        let mut doc = QueryDocument::new("doc-1", "Doc 1", "SELECT 1");
        let now = Instant::now();
        doc.schedule_prediction(now);
        assert!(!doc.prediction_is_due(now));
        assert!(doc.prediction_is_due(now + SQL_PREDICTION_DEBOUNCE));
        assert!(doc.take_prediction_schedule());
        assert!(!doc.prediction_is_due(now + SQL_PREDICTION_DEBOUNCE));
    }
}
