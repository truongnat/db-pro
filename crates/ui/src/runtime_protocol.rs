//! Typed UI command/event protocol.

use super::*;

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
    MonitoringStatStatements {
        request_id: RequestId,
        connection_id: String,
        sort: db_pro_core::domain::monitoring::StatStatementSort,
        limit: usize,
    },
    MonitoringResetStatStatements {
        request_id: RequestId,
        connection_id: String,
        confirmed: bool,
    },
    AuditEventsLoad {
        request_id: RequestId,
        connection_id: String,
        filter: db_pro_core::domain::audit::AuditFilter,
        limit: Option<usize>,
    },
    ListPgSettings {
        request_id: RequestId,
        connection_id: String,
    },
    SetPgSettingSession {
        request_id: RequestId,
        connection_id: String,
        name: String,
        value: String,
    },
    ResetPgSettingSession {
        request_id: RequestId,
        connection_id: String,
        name: String,
    },
    ListFdwInventory {
        request_id: RequestId,
        connection_id: String,
    },
    CreateFdwServer {
        request_id: RequestId,
        connection_id: String,
        name: String,
        fdw: String,
        host: String,
        dbname: String,
        port: String,
        confirmed: bool,
    },
    DropFdwServer {
        request_id: RequestId,
        connection_id: String,
        name: String,
        cascade: bool,
        confirmed: bool,
    },
    ListReplicationInventory {
        request_id: RequestId,
        connection_id: String,
    },
    CreatePublicationAll {
        request_id: RequestId,
        connection_id: String,
        name: String,
        confirmed: bool,
    },
    DropPublication {
        request_id: RequestId,
        connection_id: String,
        name: String,
        confirmed: bool,
    },
    DropSubscription {
        request_id: RequestId,
        connection_id: String,
        name: String,
        confirmed: bool,
    },
    ListEventTriggers {
        request_id: RequestId,
        connection_id: String,
    },
    CreateEventTrigger {
        request_id: RequestId,
        connection_id: String,
        name: String,
        event: String,
        function_ref: String,
        tags_csv: String,
        confirmed: bool,
    },
    DropEventTrigger {
        request_id: RequestId,
        connection_id: String,
        name: String,
        confirmed: bool,
    },
    AlterEventTrigger {
        request_id: RequestId,
        connection_id: String,
        name: String,
        mode: String,
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
    /// Delete the persisted AI provider key and deactivate the provider.
    ForgetAgentApiKey {
        request_id: RequestId,
    },
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
    MonitoringWorkloadLoaded {
        request_id: RequestId,
        workload: db_pro_core::domain::monitoring::StatStatementsSnapshot,
    },
    AuditPageLoaded {
        request_id: RequestId,
        page: db_pro_core::domain::audit::AuditPage,
    },
    PgSettingsLoaded {
        request_id: RequestId,
        snapshot: db_pro_core::domain::pg_settings::PgSettingsSnapshot,
    },
    PgSettingActionCompleted {
        request_id: RequestId,
        action: String,
        name: String,
    },
    FdwInventoryLoaded {
        request_id: RequestId,
        inventory: db_pro_core::domain::fdw::FdwInventory,
    },
    FdwActionCompleted {
        request_id: RequestId,
        action: String,
        name: String,
    },
    ReplicationInventoryLoaded {
        request_id: RequestId,
        inventory: db_pro_core::domain::replication::ReplicationInventory,
    },
    ReplicationActionCompleted {
        request_id: RequestId,
        action: String,
        name: String,
    },
    EventTriggerInventoryLoaded {
        request_id: RequestId,
        inventory: db_pro_core::domain::event_trigger::EventTriggerInventory,
    },
    EventTriggerActionCompleted {
        request_id: RequestId,
        action: String,
        name: String,
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
    AgentProviderReady {
        provider: String,
        detail: String,
    },
    AgentFailed {
        request_id: RequestId,
        message: String,
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
    AgentForgotten {
        request_id: RequestId,
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
