use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Receiver, Sender};

/// Stable identity for an async UI operation. Real backend tasks will reuse
/// this identity for cancellation and stale-result protection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDriver {
    Postgres,
    Sqlite,
    Mysql,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiSslMode {
    Disable,
    Require,
    VerifyCa,
    VerifyFull,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiConnectionDraft {
    pub name: String,
    pub host: String,
    pub port: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub driver: UiDriver,
    pub ssl_mode: UiSslMode,
    pub readonly: bool,
    pub group: String,
    pub tags: String,
    pub favorite: bool,
    pub environment: String,
    pub ssh_tunnel_enabled: bool,
    pub ssh_host: String,
    pub ssh_port: String,
    pub ssh_user: String,
    pub ssh_private_key: String,
    pub ssh_profile_id: String,
    pub ssl_root_cert_path: String,
    pub ssl_client_cert_path: String,
    pub ssl_client_key_path: String,
}

/// Developer-convenience identity for a new connection draft.
///
/// Debug builds pre-fill the developer's local PostgreSQL fixture so `cargo run`
/// starts from a usable form. The preset must never reach a release build: it
/// names a private developer database, and a shipped app must not pre-fill the
/// New Connection dialog with it.
#[cfg(debug_assertions)]
fn default_draft_identity() -> (String, String, String, String, String) {
    (
        "Xe Lạc Hồng (PostgreSQL)".to_owned(),
        "localhost".to_owned(),
        "fullstack_starter".to_owned(),
        "postgres".to_owned(),
        "postgres".to_owned(),
    )
}

/// Release builds start from a blank identity — see `default_draft_identity`.
#[cfg(not(debug_assertions))]
fn default_draft_identity() -> (String, String, String, String, String) {
    (
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    )
}

impl Default for UiConnectionDraft {
    fn default() -> Self {
        let (name, host, database, username, password) = default_draft_identity();
        Self {
            name,
            host,
            port: "5432".to_owned(),
            database,
            username,
            password,
            driver: UiDriver::Postgres,
            // #144: new PostgreSQL connections default to TLS Require. Do not change
            // the Rust `SslMode::default()` / serde fallback — persisted records that
            // omit the field must keep decoding as Disable.
            ssl_mode: UiSslMode::Require,
            readonly: false,
            group: String::new(),
            tags: String::new(),
            favorite: false,
            environment: "Development".to_owned(),
            ssh_tunnel_enabled: false,
            ssh_host: String::new(),
            ssh_port: "22".to_owned(),
            ssh_user: String::new(),
            ssh_private_key: String::new(),
            ssh_profile_id: String::new(),
            ssl_root_cert_path: String::new(),
            ssl_client_cert_path: String::new(),
            ssl_client_key_path: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UiSchemaSummary {
    pub schemas: Vec<String>,
    pub tables: Vec<String>,
    pub columns: Vec<String>,
    pub table_details: Vec<UiTableSummary>,
    pub views: Vec<UiViewSummary>,
    pub triggers: Vec<UiTriggerSummary>,
    pub functions: Vec<UiFunctionSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableSummary {
    pub schema: String,
    pub name: String,
    pub row_count: Option<u64>,
    pub columns: Vec<UiSchemaColumn>,
    pub foreign_keys: Vec<UiSchemaForeignKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiSchemaColumn {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiSchemaForeignKey {
    pub name: String,
    pub from_columns: Vec<String>,
    pub to_schema: String,
    pub to_table: String,
    pub to_columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiViewSummary {
    pub schema: String,
    pub name: String,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTriggerSummary {
    pub schema: String,
    pub name: String,
    pub table_name: String,
    pub timing: String,
    pub event: String,
    pub definition: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRoutineParameter {
    pub name: String,
    pub data_type: String,
    pub mode: String,
    pub has_default: bool,
    pub default_expr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiFunctionSummary {
    pub schema: String,
    pub name: String,
    pub routine_type: String,
    pub data_type: String,
    pub definition: String,
    pub identity_arguments: String,
    pub language: String,
    pub volatility: String,
    pub security_definer: bool,
    pub parameters: Vec<UiRoutineParameter>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableInfo {
    pub schema: String,
    pub name: String,
    pub row_count: Option<u64>,
    pub columns: Vec<UiTableColumn>,
    pub primary_key: Option<Vec<String>>,
    pub indexes: Vec<UiTableIndex>,
    pub foreign_keys: Vec<UiTableForeignKey>,
    pub check_constraints: Vec<UiCheckConstraint>,
    pub dependencies: Vec<UiTableDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiCheckConstraint {
    pub name: String,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiDependencyDirection {
    DependsOn,
    DependedBy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiDependencyKind {
    Table,
    View,
    ForeignKey,
    Trigger,
    Function,
    Sequence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableDependency {
    pub name: String,
    pub schema: String,
    pub kind: UiDependencyKind,
    pub direction: UiDependencyDirection,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum UiTableFilterOperator {
    Equals,
    NotEquals,
    #[default]
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableDataFilter {
    pub column: String,
    pub data_type: String,
    pub operator: UiTableFilterOperator,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableDataSort {
    pub column: String,
    pub descending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiTableMutation {
    Update {
        columns: Vec<String>,
        data_types: Vec<String>,
        values: Vec<UiCell>,
        pk_columns: Vec<String>,
        pk_values: Vec<UiCell>,
    },
    Delete {
        pk_columns: Vec<String>,
        pk_values: Vec<UiCell>,
    },
    Insert {
        columns: Vec<String>,
        values: Vec<UiCell>,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiTableColumn {
    pub name: String,
    pub data_type: String,
    pub ordinal: usize,
    pub nullable: bool,
    pub default: Option<String>,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub is_identity: bool,
    pub is_generated: bool,
    pub collation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableIndex {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub method: String,
    pub primary: bool,
    pub include_columns: Vec<String>,
    pub predicate: Option<String>,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableForeignKey {
    pub name: String,
    pub from_columns: Vec<String>,
    pub to_schema: String,
    pub to_table: String,
    pub to_columns: Vec<String>,
    pub on_update: String,
    pub on_delete: String,
    pub match_option: String,
    pub deferrable: bool,
    pub initially_deferred: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiQueryFolderSummary {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiSavedQuerySummary {
    pub id: String,
    pub name: String,
    pub sql: String,
    pub folder: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiConnectionSummary {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub driver: String,
    /// The TLS mode stored with the connection. Present so the edit/duplicate
    /// prefill can restore it instead of falling back to a default.
    pub ssl_mode: UiSslMode,
    pub readonly: bool,
    pub tags: Vec<String>,
    pub group: Option<String>,
    pub favorite: bool,
    pub environment: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiCommand {
    OpenQuery,
    ListConnections {
        request_id: RequestId,
    },
    IntrospectSchema {
        request_id: RequestId,
        connection_id: String,
        force_refresh: bool,
    },
    LoadTableInfo {
        request_id: RequestId,
        connection_id: String,
        schema: String,
        table: String,
    },
    LoadTableDdl {
        request_id: RequestId,
        connection_id: String,
        schema: String,
        table: String,
    },
    ExecuteDdl {
        request_id: RequestId,
        connection_id: String,
        sql: String,
    },
    LoadTableData {
        request_id: RequestId,
        connection_id: String,
        schema: String,
        table: String,
        limit: u64,
        offset: u64,
        filters: Vec<UiTableDataFilter>,
        sorts: Vec<UiTableDataSort>,
    },
    ApplyTableChanges {
        request_id: RequestId,
        connection_id: String,
        schema: String,
        table: String,
        changes: Vec<UiTableMutation>,
    },
    ListSavedQueries {
        request_id: RequestId,
        connection_id: String,
    },
    ListQueryFolders {
        request_id: RequestId,
        connection_id: String,
    },
    SaveQuery {
        request_id: RequestId,
        connection_id: String,
        saved_query_id: Option<String>,
        name: String,
        sql: String,
        folder: Option<String>,
    },
    CreateQueryFolder {
        request_id: RequestId,
        connection_id: String,
        name: String,
    },
    RenameSavedQuery {
        request_id: RequestId,
        id: String,
        name: String,
    },
    DeleteSavedQuery {
        request_id: RequestId,
        id: String,
    },
    DeleteQueryFolder {
        request_id: RequestId,
        id: String,
    },
    UpdateTableRow {
        request_id: RequestId,
        connection_id: String,
        schema: String,
        table: String,
        column: String,
        data_type: String,
        value: UiCell,
        pk_columns: Vec<String>,
        pk_values: Vec<UiCell>,
    },
    DeleteTableRow {
        request_id: RequestId,
        connection_id: String,
        schema: String,
        table: String,
        pk_columns: Vec<String>,
        pk_values: Vec<UiCell>,
    },
    InsertTableRow {
        request_id: RequestId,
        connection_id: String,
        schema: String,
        table: String,
        columns: Vec<String>,
        values: Vec<UiCell>,
    },
    RunAgent {
        request_id: RequestId,
        prompt: String,
        context: crate::AgentContext,
    },
    ExecuteAgentTool {
        request_id: RequestId,
        request: db_pro_core::domain::agent::AgentToolRequest,
        context: db_pro_core::domain::agent_workflow::AgentExecutionContext,
    },
    StartAgentRun {
        request_id: RequestId,
        prompt: String,
        session: db_pro_core::domain::agent::AgentSession,
        document: db_pro_core::domain::agent::AgentDocumentSnapshot,
        mode: db_pro_core::domain::agent::AgentMode,
        allow_read_only_auto_run: bool,
        context: db_pro_core::domain::agent_context::AgentContext,
    },
    ContinueAgentRun {
        request_id: RequestId,
        run_id: db_pro_core::domain::agent::AgentRunId,
        approved: bool,
        current_document: Option<db_pro_core::domain::agent::AgentDocumentSnapshot>,
        applied_patch: Option<db_pro_core::domain::agent::AgentToolOutput>,
    },
    CancelAgentRun {
        request_id: RequestId,
        run_id: db_pro_core::domain::agent::AgentRunId,
    },
    Connect {
        request_id: RequestId,
        connection_id: String,
    },
    CreateConnection {
        request_id: RequestId,
        draft: UiConnectionDraft,
    },
    UpdateConnection {
        request_id: RequestId,
        connection_id: String,
        draft: UiConnectionDraft,
    },
    TestConnection {
        request_id: RequestId,
        draft: UiConnectionDraft,
    },
    PickSqliteFile {
        request_id: RequestId,
    },
    PickSshPrivateKey {
        request_id: RequestId,
    },
    PickBackupFile {
        request_id: RequestId,
    },
    PickRestoreFile {
        request_id: RequestId,
    },
    PickWorkspaceFolder {
        request_id: RequestId,
    },
    DeleteConnection {
        request_id: RequestId,
        connection_id: String,
    },
    RunQuery {
        request_id: RequestId,
        connection_id: String,
        sql: String,
        /// Bound parameter values in placeholder order (#225). Mapped to `QueryParam::Text`.
        params: Vec<String>,
    },
    RunQueryMulti {
        request_id: RequestId,
        connection_id: String,
        sql: String,
    },
    ExplainQuery {
        request_id: RequestId,
        connection_id: String,
        sql: String,
        analyze: bool,
    },
    Backup {
        request_id: RequestId,
        connection_id: String,
        output_path: String,
        custom_format: bool,
    },
    Restore {
        request_id: RequestId,
        connection_id: String,
        input_path: String,
        custom_format: bool,
    },
    MonitoringSnapshot {
        request_id: RequestId,
        connection_id: String,
    },
    MonitoringCancelBackend {
        request_id: RequestId,
        connection_id: String,
        backend_id: i64,
    },
    MonitoringTerminateBackend {
        request_id: RequestId,
        connection_id: String,
        backend_id: i64,
    },
    MonitoringMaintenance {
        request_id: RequestId,
        connection_id: String,
        schema: Option<String>,
        table: Option<String>,
        action: db_pro_core::domain::monitoring::MaintenanceAction,
        confirmed: bool,
    },
    ListUsers {
        request_id: RequestId,
        connection_id: String,
    },
    CreateRole {
        request_id: RequestId,
        connection_id: String,
        name: String,
        login: bool,
    },
    DropRole {
        request_id: RequestId,
        connection_id: String,
        name: String,
    },
    ListPrivileges {
        request_id: RequestId,
        connection_id: String,
        role_name: String,
    },
    ListTableRls {
        request_id: RequestId,
        connection_id: String,
        schema: String,
        table: String,
    },
    AlterRole {
        request_id: RequestId,
        connection_id: String,
        name: String,
        attributes: db_pro_core::domain::user::RoleAttributes,
    },
    UpdateRolePassword {
        request_id: RequestId,
        connection_id: String,
        name: String,
        password: String,
    },
    ListMemberships {
        request_id: RequestId,
        connection_id: String,
        member: String,
    },
    GrantMembership {
        request_id: RequestId,
        connection_id: String,
        role: String,
        member: String,
    },
    RevokeMembership {
        request_id: RequestId,
        connection_id: String,
        role: String,
        member: String,
    },
    GrantPrivilege {
        request_id: RequestId,
        connection_id: String,
        role_name: String,
        object_kind: db_pro_core::domain::user::PrivilegeObjectKind,
        schema: String,
        object_name: String,
        privilege: String,
    },
    RevokePrivilege {
        request_id: RequestId,
        connection_id: String,
        role_name: String,
        object_kind: db_pro_core::domain::user::PrivilegeObjectKind,
        schema: String,
        object_name: String,
        privilege: String,
    },
    DiffTableDataKeyed {
        request_id: RequestId,
        source_id: String,
        target_id: String,
        schema: String,
        table: String,
        key_columns: Vec<String>,
        sample_limit: Option<u64>,
    },
    CancelQuery {
        request_id: RequestId,
    },
    RequestSqlPrediction {
        request_id: RequestId,
        document_id: String,
        document_version: u64,
        anchor: usize,
        replacement_range: (usize, usize),
        context: crate::editor::prediction::AiSqlContext,
    },
    CancelSqlPrediction {
        request_id: RequestId,
    },
    /// Store a new AI provider API key and reconfigure the agent at runtime.
    /// The native adapter is responsible for persisting the key securely
    /// (OS keyring) before forwarding it to the worker.
    SaveAgentApiKey {
        request_id: RequestId,
        api_key: String,
    },
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum UiEvent {
    ConnectionsLoaded {
        request_id: RequestId,
        connections: Vec<UiConnectionSummary>,
    },
    SavedQueriesLoaded {
        request_id: RequestId,
        queries: Vec<UiSavedQuerySummary>,
    },
    SchemaLoaded {
        request_id: RequestId,
        schema: UiSchemaSummary,
    },
    TableInfoLoaded {
        request_id: RequestId,
        table_info: UiTableInfo,
    },
    TableDdlLoaded {
        request_id: RequestId,
        sql: String,
    },
    DdlCompleted {
        request_id: RequestId,
        affected_rows: u64,
    },
    TableDataLoaded {
        request_id: RequestId,
        result: UiQueryResult,
        total_rows: u64,
    },
    QueryFoldersLoaded {
        request_id: RequestId,
        folders: Vec<UiQueryFolderSummary>,
    },
    FilePicked {
        request_id: RequestId,
        kind: String,
        path: Option<String>,
    },
    OperationProgress {
        request_id: RequestId,
        operation: String,
        status: String,
    },
    BackupCompleted {
        request_id: RequestId,
        output_path: String,
        size_bytes: u64,
    },
    MonitoringSnapshotLoaded {
        request_id: RequestId,
        snapshot: db_pro_core::domain::monitoring::MonitoringSnapshot,
    },
    MonitoringActionCompleted {
        request_id: RequestId,
        action: String,
        backend_id: i64,
        succeeded: bool,
    },
    UsersLoaded {
        request_id: RequestId,
        users: Vec<db_pro_core::domain::user::DatabaseUser>,
    },
    PrivilegesLoaded {
        request_id: RequestId,
        role_name: String,
        privileges: Vec<db_pro_core::domain::user::Privilege>,
    },
    MembershipsLoaded {
        request_id: RequestId,
        member: String,
        memberships: Vec<db_pro_core::domain::user::RoleMembership>,
    },
    TableRlsLoaded {
        request_id: RequestId,
        state: db_pro_core::domain::rls::TableRlsState,
    },
    DataDiffLoaded {
        request_id: RequestId,
        diff: db_pro_core::domain::cross_connection::DataDiff,
    },
    OperationCompleted {
        request_id: RequestId,
        operation: String,
    },
    TableChangesFailed {
        request_id: RequestId,
        code: String,
        message: String,
        statement_index: usize,
        rolled_back: bool,
    },
    Connected {
        request_id: RequestId,
        connection_id: String,
    },
    QueryQueued {
        request_id: RequestId,
    },
    QueryCompleted {
        request_id: RequestId,
        result: UiQueryResult,
    },
    QueryMultiCompleted {
        request_id: RequestId,
        output: UiQueryExecutionOutput,
    },
    QuerySaved {
        request_id: RequestId,
        query: UiSavedQuerySummary,
    },
    ExplainCompleted {
        request_id: RequestId,
        plan: String,
    },
    QueryCancelled {
        request_id: RequestId,
    },
    AgentCompleted {
        request_id: RequestId,
        provider: String,
        message: crate::AgentMessage,
    },
    AgentProviderReady {
        provider: String,
        detail: String,
    },
    AgentFailed {
        request_id: RequestId,
        message: String,
    },
    AgentToolCompleted {
        request_id: RequestId,
        session_id: db_pro_core::domain::agent::AgentSessionId,
        run_id: db_pro_core::domain::agent::AgentRunId,
        document_id: String,
        result: db_pro_core::domain::agent::AgentToolResult,
    },
    AgentToolFailed {
        request_id: RequestId,
        session_id: db_pro_core::domain::agent::AgentSessionId,
        run_id: db_pro_core::domain::agent::AgentRunId,
        document_id: String,
        error: db_pro_core::domain::agent_workflow::AgentToolError,
    },
    AgentWorkflow {
        request_id: RequestId,
        event: db_pro_core::domain::agent_workflow::AgentWorkflowEvent,
    },
    AgentConfigured {
        request_id: RequestId,
        provider: String,
        detail: String,
    },
    QueryFailed {
        request_id: RequestId,
        message: String,
    },
    QueryFailedDetailed {
        request_id: RequestId,
        code: String,
        message: String,
        position: Option<usize>,
    },
    SqlPredictionReady {
        request_id: RequestId,
        document_id: String,
        document_version: u64,
        anchor: usize,
        replacement_range: (usize, usize),
        prediction: String,
    },
    SqlPredictionFailed {
        request_id: RequestId,
        document_id: String,
        document_version: u64,
        anchor: usize,
        replacement_range: (usize, usize),
        message: String,
    },
}

/// Small typed boundary between the immediate-mode UI and asynchronous work.
/// The receiver is drained by the UI thread once per frame; the native binary
/// adapts these standard channels to the tokio runtime worker.
pub struct TaskBridge {
    command_tx: Sender<UiCommand>,
    event_rx: Receiver<UiEvent>,
    next_request_id: u64,
}

impl Default for TaskBridge {
    fn default() -> Self {
        let (bridge, _command_rx, _event_tx) = Self::with_channels();
        bridge
    }
}

impl TaskBridge {
    /// Build the UI-side bridge and expose its endpoints to a runtime adapter.
    /// The adapter is responsible for translating these messages to its async
    /// channel implementation.
    pub fn with_channels() -> (Self, Receiver<UiCommand>, Sender<UiEvent>) {
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        (
            Self {
                command_tx,
                event_rx,
                next_request_id: 1,
            },
            command_rx,
            event_tx,
        )
    }

    pub fn new(command_tx: Sender<UiCommand>, event_rx: Receiver<UiEvent>) -> Self {
        Self {
            command_tx,
            event_rx,
            next_request_id: 1,
        }
    }

    pub fn next_request_id(&mut self) -> RequestId {
        let id = RequestId(self.next_request_id);
        self.next_request_id = self.next_request_id.saturating_add(1);
        id
    }

    pub fn send(&self, command: UiCommand) -> Result<(), Box<mpsc::SendError<UiCommand>>> {
        self.command_tx.send(command).map_err(Box::new)
    }

    pub fn drain_events(&self) -> impl Iterator<Item = UiEvent> + '_ {
        std::iter::from_fn(|| self.event_rx.try_recv().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression guard for the S-1 release-hygiene finding: the developer
    /// connection preset is developer-only. `cargo test --release` exercises the
    /// release half of this assertion.
    #[test]
    fn default_draft_identity_follows_build_profile() {
        let draft = UiConnectionDraft::default();

        if cfg!(debug_assertions) {
            assert_eq!(draft.name, "Xe Lạc Hồng (PostgreSQL)");
            assert_eq!(draft.host, "localhost");
            assert_eq!(draft.database, "fullstack_starter");
            assert_eq!(draft.username, "postgres");
            assert_eq!(draft.password, "postgres");
        } else {
            assert!(
                draft.name.is_empty(),
                "release builds must not pre-fill a connection name"
            );
            assert!(
                draft.host.is_empty(),
                "release builds must not pre-fill a connection host"
            );
            assert!(
                draft.database.is_empty(),
                "release builds must not pre-fill a database name"
            );
            assert!(draft.username.is_empty(), "release builds must not pre-fill a username");
            assert!(draft.password.is_empty(), "release builds must not pre-fill a password");
        }
    }

    /// The neutral parts of the draft are the same in every build profile.
    #[test]
    fn default_draft_neutral_fields_are_profile_independent() {
        let draft = UiConnectionDraft::default();

        assert_eq!(draft.port, "5432");
        assert_eq!(draft.driver, UiDriver::Postgres);
        assert_eq!(draft.ssl_mode, UiSslMode::Require);
        assert!(!draft.readonly);
        assert!(!draft.ssh_tunnel_enabled);
        assert_eq!(draft.ssh_port, "22");
        assert!(draft.ssh_host.is_empty());
        assert!(draft.ssh_user.is_empty());
        assert!(draft.ssh_private_key.is_empty());
    }

    #[test]
    fn request_ids_are_monotonic() {
        let mut bridge = TaskBridge::default();
        assert_eq!(bridge.next_request_id(), RequestId(1));
        assert_eq!(bridge.next_request_id(), RequestId(2));
    }

    #[test]
    fn bridge_delivers_typed_commands() {
        let (command_tx, command_rx) = mpsc::channel();
        let (_event_tx, event_rx) = mpsc::channel();
        let bridge = TaskBridge::new(command_tx, event_rx);
        let command = UiCommand::OpenQuery;
        bridge.send(command.clone()).expect("receiver is alive");
        assert_eq!(command_rx.recv().expect("command expected"), command);
    }
}
