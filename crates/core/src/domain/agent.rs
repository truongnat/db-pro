use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::safety::{classify_script_safety, StatementSafety};

pub const MAX_AGENT_TABLES: usize = 12;
pub const MAX_AGENT_COLUMNS_PER_TABLE: usize = 24;
pub const MAX_AGENT_RELATIONS: usize = 12;
pub const MAX_AGENT_SAMPLE_ROWS: usize = 20;
pub const MAX_AGENT_RESULT_COLUMNS: usize = 50;
pub const MAX_AGENT_CELL_CHARS: usize = 256;
pub const MAX_AGENT_CONTEXT_CHARS: usize = 12_000;
pub const MAX_AGENT_EXPLAIN_CHARS: usize = 4_000;
pub const MAX_AGENT_TOOL_OUTPUT_CHARS: usize = 8_000;
pub const MAX_AGENT_HISTORY_MESSAGES: usize = 10;
pub const MAX_AGENT_HISTORY_CHARS: usize = 16_000;
pub const MAX_AGENT_TOOL_STEPS: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentSessionId(Uuid);

impl AgentSessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AgentSessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AgentSessionId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentRunId(Uuid);

impl AgentRunId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AgentRunId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentMode {
    Ask,
    Edit,
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentSessionState {
    Idle,
    Running,
    AwaitingConfirmation,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentMessageRole {
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentMessage {
    pub role: AgentMessageRole,
    pub content: String,
    pub document_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRun {
    pub id: AgentRunId,
    pub document_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDocumentSnapshot {
    pub document_id: String,
    pub document_version: u64,
    pub sql: String,
    pub cursor_offset: usize,
    pub selection: Option<(usize, usize)>,
    pub current_statement: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: AgentSessionId,
    pub document_id: String,
    pub connection_id: Option<String>,
    pub schema: Option<String>,
    pub messages: Vec<AgentMessage>,
    pub state: AgentSessionState,
    pub active_run: Option<AgentRun>,
}

impl AgentSession {
    pub fn new(document_id: impl Into<String>, connection_id: Option<String>, schema: Option<String>) -> Self {
        Self {
            id: AgentSessionId::new(),
            document_id: document_id.into(),
            connection_id,
            schema,
            messages: Vec::new(),
            state: AgentSessionState::Idle,
            active_run: None,
        }
    }

    pub fn start_run(&mut self, document_version: u64) -> Result<AgentRunId, AgentSessionError> {
        if self.active_run.is_some() {
            return Err(AgentSessionError::RunAlreadyActive);
        }
        let run_id = AgentRunId::new();
        self.active_run = Some(AgentRun {
            id: run_id,
            document_version,
        });
        self.state = AgentSessionState::Running;
        Ok(run_id)
    }

    pub fn finish_run(&mut self, run_id: AgentRunId, state: AgentSessionState) -> Result<(), AgentSessionError> {
        if self.active_run.as_ref().map(|run| run.id) != Some(run_id) {
            return Err(AgentSessionError::RunMismatch);
        }
        self.active_run = None;
        self.state = state;
        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AgentSessionError {
    #[error("an agent run is already active for this session")]
    RunAlreadyActive,
    #[error("agent run does not belong to this session")]
    RunMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSqlPatch {
    pub document_id: String,
    pub expected_version: u64,
    pub range: (usize, usize),
    pub replacement: String,
}

impl AgentSqlPatch {
    pub fn apply_to(&self, document_id: &str, version: u64, text: &str) -> Result<String, AgentPatchError> {
        if self.document_id != document_id {
            return Err(AgentPatchError::DocumentMismatch);
        }
        if self.expected_version != version {
            return Err(AgentPatchError::StaleVersion {
                expected: self.expected_version,
                actual: version,
            });
        }
        let (start, end) = self.range;
        if start > end || end > text.len() || !text.is_char_boundary(start) || !text.is_char_boundary(end) {
            return Err(AgentPatchError::InvalidRange);
        }
        let mut patched = String::with_capacity(text.len() - (end - start) + self.replacement.len());
        patched.push_str(&text[..start]);
        patched.push_str(&self.replacement);
        patched.push_str(&text[end..]);
        Ok(patched)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentPatchError {
    #[error("agent patch targets another document")]
    DocumentMismatch,
    #[error("agent patch is stale: expected version {expected}, current version {actual}")]
    StaleVersion { expected: u64, actual: u64 },
    #[error("agent patch range is not a valid UTF-8 range")]
    InvalidRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentAction {
    ExplainSql { sql: String },
    GenerateSql { request: String },
    ReplaceSelection { patch: AgentSqlPatch },
    ReplaceStatement { patch: AgentSqlPatch },
    InsertAtCursor { patch: AgentSqlPatch },
    RunQuery { sql: String },
    RunSelection { sql: String, range: (usize, usize) },
    InspectResult,
    InspectSchema { table: Option<AgentObjectRef> },
    OpenTable { table: AgentObjectRef },
    ApplyPatch { patch: AgentSqlPatch },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentObjectRef {
    pub schema: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentTool {
    InspectSchema,
    InspectTable,
    InspectColumns,
    InspectForeignKeys,
    GetCurrentQuery,
    PatchQuery,
    RunQuery,
    InspectQueryResult,
    ExplainQuery,
}

pub fn allows_stale_document_version(tool: AgentTool) -> bool {
    matches!(
        tool,
        AgentTool::GetCurrentQuery
            | AgentTool::InspectSchema
            | AgentTool::InspectTable
            | AgentTool::InspectColumns
            | AgentTool::InspectForeignKeys
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentToolCall {
    pub call_id: String,
    pub tool: AgentTool,
    pub input: AgentToolInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentToolRequest {
    pub session_id: AgentSessionId,
    pub run_id: AgentRunId,
    pub document_id: String,
    pub document_version: u64,
    pub tool: AgentTool,
    pub input: AgentToolInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentToolInput {
    None,
    Schema {
        schema: Option<String>,
    },
    Table {
        table: AgentObjectRef,
    },
    Patch {
        patch: AgentSqlPatch,
    },
    Query {
        sql: String,
    },
    ResultSample {
        max_rows: usize,
        statement_index: Option<usize>,
    },
}

impl AgentToolInput {
    pub fn fingerprint(&self) -> String {
        match self {
            AgentToolInput::None => "none".to_owned(),
            AgentToolInput::Schema { schema } => format!("schema:{}", schema.as_deref().unwrap_or("")),
            AgentToolInput::Table { table } => {
                format!("table:{}.{}", table.schema.as_deref().unwrap_or(""), table.name)
            }
            AgentToolInput::Patch { patch } => format!(
                "patch:{}:{}:{}-{}",
                patch.document_id, patch.expected_version, patch.range.0, patch.range.1
            ),
            AgentToolInput::Query { sql } => format!("query:{}", sql.trim()),
            AgentToolInput::ResultSample {
                max_rows,
                statement_index,
            } => format!("sample:{}:{}", max_rows, statement_index.unwrap_or(0)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentToolOutput {
    Schema {
        tables: Vec<AgentObjectRef>,
        views: Vec<AgentObjectRef>,
    },
    Table {
        table: super::agent_context::AgentTableContext,
        primary_key: Vec<String>,
        unique_columns: Vec<String>,
        indexes: Vec<String>,
        foreign_keys: Vec<super::agent_context::AgentForeignKeyContext>,
    },
    Columns {
        table: AgentObjectRef,
        columns: Vec<super::agent_context::AgentColumnContext>,
    },
    ForeignKeys {
        relations: Vec<super::agent_context::AgentForeignKeyContext>,
    },
    CurrentQuery {
        document_id: String,
        document_version: u64,
        sql: String,
        cursor_offset: usize,
        selection: Option<(usize, usize)>,
        current_statement: Option<String>,
    },
    PatchPreview {
        patch: AgentSqlPatch,
        original: String,
        proposed: String,
    },
    PatchApplied {
        document_id: String,
        new_version: u64,
        range: (usize, usize),
    },
    QueryResult {
        statement_index: Option<usize>,
        result_count: usize,
        summary: super::agent_context::AgentResultSummary,
    },
    Explain {
        plan: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentToolResult {
    pub tool: AgentTool,
    pub output: AgentToolOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentSqlSafety {
    ReadOnly,
    Mutating,
    Destructive,
    Unknown,
}

impl AgentSqlSafety {
    /// Classify a SQL script by its most dangerous statement.
    ///
    /// A multi-statement batch is never read-only because it starts with
    /// `SELECT`: `SELECT 1; DROP TABLE t;` is `Destructive` and therefore never
    /// auto-runs on the read-only path (#147, #129).
    pub fn classify(sql: &str) -> Self {
        match classify_script_safety(sql) {
            Some(StatementSafety::Read) => Self::ReadOnly,
            Some(StatementSafety::Write | StatementSafety::Ddl) => Self::Mutating,
            Some(StatementSafety::Destructive) => Self::Destructive,
            None => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentExecutionDecision {
    Allowed,
    RequiresConfirmation,
    Rejected,
}

pub fn execution_decision(
    mode: AgentMode,
    safety: AgentSqlSafety,
    allow_read_only_auto_run: bool,
) -> AgentExecutionDecision {
    if !matches!(mode, AgentMode::Agent) {
        return AgentExecutionDecision::Rejected;
    }
    match safety {
        AgentSqlSafety::ReadOnly if allow_read_only_auto_run => AgentExecutionDecision::Allowed,
        AgentSqlSafety::ReadOnly | AgentSqlSafety::Mutating | AgentSqlSafety::Destructive | AgentSqlSafety::Unknown => {
            AgentExecutionDecision::RequiresConfirmation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_rejects_wrong_document_and_version() {
        let patch = AgentSqlPatch {
            document_id: "doc-a".to_owned(),
            expected_version: 4,
            range: (0, 6),
            replacement: "SELECT".to_owned(),
        };
        assert_eq!(
            patch.apply_to("doc-b", 4, "select 1"),
            Err(AgentPatchError::DocumentMismatch)
        );
        assert_eq!(
            patch.apply_to("doc-a", 5, "select 1"),
            Err(AgentPatchError::StaleVersion { expected: 4, actual: 5 })
        );
    }

    #[test]
    fn patch_is_utf8_boundary_safe() {
        let patch = AgentSqlPatch {
            document_id: "doc".to_owned(),
            expected_version: 1,
            range: (1, 2),
            replacement: "x".to_owned(),
        };
        assert_eq!(patch.apply_to("doc", 1, "é"), Err(AgentPatchError::InvalidRange));
    }

    #[test]
    fn safety_decision_never_auto_runs_mutations() {
        assert_eq!(
            execution_decision(AgentMode::Agent, AgentSqlSafety::ReadOnly, true),
            AgentExecutionDecision::Allowed
        );
        assert_eq!(
            execution_decision(AgentMode::Agent, AgentSqlSafety::Mutating, true),
            AgentExecutionDecision::RequiresConfirmation
        );
        assert_eq!(
            execution_decision(AgentMode::Agent, AgentSqlSafety::Destructive, true),
            AgentExecutionDecision::RequiresConfirmation
        );
        assert_eq!(
            execution_decision(AgentMode::Edit, AgentSqlSafety::ReadOnly, true),
            AgentExecutionDecision::Rejected
        );
    }

    #[test]
    fn incomplete_sql_is_confirmation_gated() {
        assert_eq!(AgentSqlSafety::classify("SELECT"), AgentSqlSafety::ReadOnly);
        assert_eq!(AgentSqlSafety::classify(""), AgentSqlSafety::Unknown);
        assert_eq!(
            execution_decision(AgentMode::Agent, AgentSqlSafety::Unknown, true),
            AgentExecutionDecision::RequiresConfirmation
        );
    }

    #[test]
    fn batch_is_classified_by_its_most_dangerous_statement() {
        assert_eq!(
            AgentSqlSafety::classify("SELECT 1; DROP TABLE t;"),
            AgentSqlSafety::Destructive
        );
        assert_eq!(
            AgentSqlSafety::classify("SELECT 1; DELETE FROM t WHERE id = 1"),
            AgentSqlSafety::Mutating
        );
        assert_eq!(
            AgentSqlSafety::classify("SELECT 1; UPDATE t SET a = 1"),
            AgentSqlSafety::Mutating
        );
        assert_eq!(
            AgentSqlSafety::classify("SELECT 1; CREATE TABLE t (id int)"),
            AgentSqlSafety::Mutating
        );
        assert_eq!(AgentSqlSafety::classify("SELECT 1; SELECT 2"), AgentSqlSafety::ReadOnly);
        assert_eq!(AgentSqlSafety::classify("DROP TABLE t"), AgentSqlSafety::Destructive);
    }

    #[test]
    fn destructive_batch_is_never_auto_runnable() {
        assert_eq!(
            execution_decision(
                AgentMode::Agent,
                AgentSqlSafety::classify("SELECT 1; DROP TABLE t;"),
                true
            ),
            AgentExecutionDecision::RequiresConfirmation
        );
        assert_eq!(
            execution_decision(AgentMode::Agent, AgentSqlSafety::classify("SELECT 1; SELECT 2"), true),
            AgentExecutionDecision::Allowed
        );
    }
}
