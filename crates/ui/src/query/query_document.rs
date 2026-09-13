use crate::editor::buffer::TextBuffer;
use crate::editor::completion::CompletionState;
use crate::editor::cursor::CursorPosition;
use crate::editor::diagnostics::Diagnostic;
use crate::editor::document::SqlDocumentAnalysis;
use crate::editor::prediction::EditPrediction;
use crate::editor::selection::SelectionRange;
use crate::editor::syntax::SqlDialect;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    pub saved_query_id: Option<String>,
    pub execution_state: QueryExecutionState,
    pub analysis: SqlDocumentAnalysis,
    pub diagnostics: Vec<Diagnostic>,
    pub completion: CompletionState,
    pub prediction: Option<EditPrediction>,
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
            saved_query_id: None,
            execution_state: QueryExecutionState::Idle,
            analysis,
            diagnostics: Vec::new(),
            completion: CompletionState::new(),
            prediction: None,
        }
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
        self.dirty = true;
    }

    pub fn reanalyze(&mut self, dialect: SqlDialect) {
        if self.analysis.version != self.buffer.version() {
            self.analysis = SqlDocumentAnalysis::analyze(&self.buffer, dialect);
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
        }
        let helper = SerializedQueryDocument {
            id: &self.id,
            title: &self.title,
            content: self.buffer.text(),
            connection_id: self.connection_id.as_deref(),
            schema: self.schema.as_deref(),
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
    fn test_query_document_serde_roundtrip() {
        let doc = QueryDocument::new("q-1", "My Query", "SELECT * FROM users;");
        let serialized = serde_json::to_string(&doc).unwrap();
        let deserialized: QueryDocument = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, "q-1");
        assert_eq!(deserialized.title, "My Query");
        assert_eq!(deserialized.text(), "SELECT * FROM users;");
    }
}
