use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use db_pro_core::application::sql_builder::{SortClause, TableFilter};
use db_pro_core::application::TableDataMutation;
use db_pro_core::domain::agent::{AgentDocumentSnapshot, AgentMode, AgentRunId, AgentSession, AgentToolOutput};
use db_pro_core::domain::agent_context::AgentContext as StructuredAgentContext;
use db_pro_core::domain::agent_workflow::{AgentExecutionContext, AgentWorkflow};
use db_pro_core::domain::backup::{BackupOptions, RestoreOptions};
use db_pro_core::domain::query::{CellValue, QueryResult};
use tokio::sync::{mpsc, oneshot};

use crate::{
    AgentContext, AgentRunOrchestrator, AgentToolExecutor, AgentWorkflowEvent, CodexProvider, CodexProviderError,
    ConnectionSummary, DbProRuntime, QueryApi, SqlPredictionContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeRequestId(pub u64);

#[derive(Debug)]
pub enum RuntimeCommand {
    ListConnections {
        request_id: RuntimeRequestId,
    },
    IntrospectSchema {
        request_id: RuntimeRequestId,
        connection_id: String,
        force_refresh: bool,
    },
    LoadTableInfo {
        request_id: RuntimeRequestId,
        connection_id: String,
        schema: String,
        table: String,
    },
    LoadTableDdl {
        request_id: RuntimeRequestId,
        connection_id: String,
        schema: String,
        table: String,
    },
    ExecuteDdl {
        request_id: RuntimeRequestId,
        connection_id: String,
        sql: String,
    },
    LoadTableData {
        request_id: RuntimeRequestId,
        connection_id: String,
        schema: String,
        table: String,
        limit: u64,
        offset: u64,
        filters: Vec<TableFilter>,
        sorts: Vec<SortClause>,
    },
    ListSavedQueries {
        request_id: RuntimeRequestId,
        connection_id: String,
    },
    ListQueryFolders {
        request_id: RuntimeRequestId,
        connection_id: String,
    },
    SaveQuery {
        request_id: RuntimeRequestId,
        connection_id: String,
        saved_query_id: Option<String>,
        name: String,
        sql: String,
        folder: Option<String>,
    },
    CreateQueryFolder {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
    },
    RenameSavedQuery {
        request_id: RuntimeRequestId,
        id: String,
        name: String,
    },
    DeleteSavedQuery {
        request_id: RuntimeRequestId,
        id: String,
    },
    DeleteQueryFolder {
        request_id: RuntimeRequestId,
        id: String,
    },
    UpdateTableRow {
        request_id: RuntimeRequestId,
        connection_id: String,
        schema: String,
        table: String,
        column: String,
        value: CellValue,
        pk_columns: Vec<String>,
        pk_values: Vec<CellValue>,
    },
    DeleteTableRow {
        request_id: RuntimeRequestId,
        connection_id: String,
        schema: String,
        table: String,
        pk_columns: Vec<String>,
        pk_values: Vec<CellValue>,
    },
    InsertTableRow {
        request_id: RuntimeRequestId,
        connection_id: String,
        schema: String,
        table: String,
        columns: Vec<String>,
        values: Vec<CellValue>,
    },
    ApplyTableChanges {
        request_id: RuntimeRequestId,
        connection_id: String,
        schema: String,
        table: String,
        changes: Vec<TableDataMutation>,
    },
    RunAgent {
        request_id: RuntimeRequestId,
        prompt: String,
        context: AgentContext,
    },
    ExecuteAgentTool {
        request_id: RuntimeRequestId,
        request: db_pro_core::domain::agent::AgentToolRequest,
        context: db_pro_core::domain::agent_workflow::AgentExecutionContext,
    },
    StartAgentWorkflow {
        request_id: RuntimeRequestId,
        prompt: String,
        session: AgentSession,
        document: AgentDocumentSnapshot,
        mode: AgentMode,
        allow_read_only_auto_run: bool,
        context: StructuredAgentContext,
    },
    ContinueAgentWorkflow {
        request_id: RuntimeRequestId,
        run_id: AgentRunId,
        approved: bool,
        current_document: Option<AgentDocumentSnapshot>,
        applied_patch: Option<AgentToolOutput>,
    },
    CancelAgentWorkflow {
        request_id: RuntimeRequestId,
        run_id: AgentRunId,
    },
    CreateConnection {
        request_id: RuntimeRequestId,
        config: db_pro_core::domain::connection::ConnectionConfig,
        password: String,
    },
    UpdateConnection {
        request_id: RuntimeRequestId,
        connection_id: String,
        config: db_pro_core::domain::connection::ConnectionConfig,
        password: Option<String>,
    },
    DeleteConnection {
        request_id: RuntimeRequestId,
        connection_id: String,
    },
    TestConnection {
        request_id: RuntimeRequestId,
        config: db_pro_core::domain::connection::ConnectionConfig,
        password: String,
    },
    Connect {
        request_id: RuntimeRequestId,
        connection_id: String,
    },
    ExecuteQuery {
        request_id: RuntimeRequestId,
        connection_id: String,
        sql: String,
        /// Bound parameter values in placeholder order (#225).
        params: Vec<String>,
    },
    ExecuteQueryMulti {
        request_id: RuntimeRequestId,
        connection_id: String,
        sql: String,
    },
    ExplainQuery {
        request_id: RuntimeRequestId,
        connection_id: String,
        sql: String,
        analyze: bool,
    },
    Backup {
        request_id: RuntimeRequestId,
        options: BackupOptions,
    },
    Restore {
        request_id: RuntimeRequestId,
        options: RestoreOptions,
    },
    MonitoringSnapshot {
        request_id: RuntimeRequestId,
        connection_id: String,
    },
    MonitoringCancelBackend {
        request_id: RuntimeRequestId,
        connection_id: String,
        backend_id: i64,
    },
    MonitoringTerminateBackend {
        request_id: RuntimeRequestId,
        connection_id: String,
        backend_id: i64,
    },
    MonitoringMaintenance {
        request_id: RuntimeRequestId,
        connection_id: String,
        schema: Option<String>,
        table: Option<String>,
        action: db_pro_core::domain::monitoring::MaintenanceAction,
        confirmed: bool,
    },
    MonitoringStatStatements {
        request_id: RuntimeRequestId,
        connection_id: String,
        sort: db_pro_core::domain::monitoring::StatStatementSort,
        limit: usize,
    },
    MonitoringResetStatStatements {
        request_id: RuntimeRequestId,
        connection_id: String,
        confirmed: bool,
    },
    AuditEventsLoad {
        request_id: RuntimeRequestId,
        connection_id: String,
        filter: db_pro_core::domain::audit::AuditFilter,
        limit: Option<usize>,
    },
    ListPgSettings {
        request_id: RuntimeRequestId,
        connection_id: String,
    },
    SetPgSettingSession {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
        value: String,
    },
    ResetPgSettingSession {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
    },
    ListFdwInventory {
        request_id: RuntimeRequestId,
        connection_id: String,
    },
    CreateFdwServer {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
        fdw: String,
        host: String,
        dbname: String,
        port: String,
        confirmed: bool,
    },
    DropFdwServer {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
        cascade: bool,
        confirmed: bool,
    },
    ListReplicationInventory {
        request_id: RuntimeRequestId,
        connection_id: String,
    },
    CreatePublicationAll {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
        confirmed: bool,
    },
    DropPublication {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
        confirmed: bool,
    },
    DropSubscription {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
        confirmed: bool,
    },
    ListEventTriggers {
        request_id: RuntimeRequestId,
        connection_id: String,
    },
    CreateEventTrigger {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
        event: String,
        function_ref: String,
        tags_csv: String,
        confirmed: bool,
    },
    DropEventTrigger {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
        confirmed: bool,
    },
    AlterEventTrigger {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
        mode: String,
        confirmed: bool,
    },
    ListUsers {
        request_id: RuntimeRequestId,
        connection_id: String,
    },
    CreateRole {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
        login: bool,
    },
    DropRole {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
    },
    ListPrivileges {
        request_id: RuntimeRequestId,
        connection_id: String,
        role_name: String,
    },
    ListTableRls {
        request_id: RuntimeRequestId,
        connection_id: String,
        schema: String,
        table: String,
    },
    AlterRole {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
        attributes: db_pro_core::domain::user::RoleAttributes,
    },
    UpdateRolePassword {
        request_id: RuntimeRequestId,
        connection_id: String,
        name: String,
        password: String,
    },
    ListMemberships {
        request_id: RuntimeRequestId,
        connection_id: String,
        member: String,
    },
    GrantMembership {
        request_id: RuntimeRequestId,
        connection_id: String,
        role: String,
        member: String,
    },
    RevokeMembership {
        request_id: RuntimeRequestId,
        connection_id: String,
        role: String,
        member: String,
    },
    GrantPrivilege {
        request_id: RuntimeRequestId,
        connection_id: String,
        role_name: String,
        object_kind: db_pro_core::domain::user::PrivilegeObjectKind,
        schema: String,
        object_name: String,
        privilege: String,
    },
    RevokePrivilege {
        request_id: RuntimeRequestId,
        connection_id: String,
        role_name: String,
        object_kind: db_pro_core::domain::user::PrivilegeObjectKind,
        schema: String,
        object_name: String,
        privilege: String,
    },
    DiffTableDataKeyed {
        request_id: RuntimeRequestId,
        source_id: String,
        target_id: String,
        schema: String,
        table: String,
        key_columns: Vec<String>,
        sample_limit: Option<u64>,
    },
    CancelQuery {
        request_id: RuntimeRequestId,
    },
    CancelOperation {
        request_id: RuntimeRequestId,
    },
    /// Replace the AI provider key at runtime without restarting the app.
    ConfigureAgent {
        request_id: RuntimeRequestId,
        api_key: String,
    },
    RequestSqlPrediction {
        request_id: RuntimeRequestId,
        document_id: String,
        document_version: u64,
        anchor: usize,
        replacement_range: (usize, usize),
        context: SqlPredictionContext,
    },
    CancelSqlPrediction {
        request_id: RuntimeRequestId,
    },
}

#[derive(Debug)]
pub enum RuntimeEvent {
    ConnectionsLoaded {
        request_id: RuntimeRequestId,
        connections: Vec<ConnectionSummary>,
    },
    SchemaLoaded {
        request_id: RuntimeRequestId,
        schema: crate::SchemaSummary,
    },
    TableInfoLoaded {
        request_id: RuntimeRequestId,
        table_info: db_pro_core::domain::schema::TableInfo,
    },
    TableDdlLoaded {
        request_id: RuntimeRequestId,
        sql: String,
    },
    DdlCompleted {
        request_id: RuntimeRequestId,
        affected_rows: u64,
    },
    TableDataLoaded {
        request_id: RuntimeRequestId,
        result: QueryResult,
        total_rows: u64,
    },
    SavedQueriesLoaded {
        request_id: RuntimeRequestId,
        queries: Vec<crate::SavedQuerySummary>,
    },
    QueryFoldersLoaded {
        request_id: RuntimeRequestId,
        folders: Vec<crate::QueryFolderSummary>,
    },
    OperationProgress {
        request_id: RuntimeRequestId,
        operation: &'static str,
        status: &'static str,
    },
    BackupCompleted {
        request_id: RuntimeRequestId,
        output_path: String,
        size_bytes: u64,
    },
    MonitoringSnapshotLoaded {
        request_id: RuntimeRequestId,
        snapshot: db_pro_core::domain::monitoring::MonitoringSnapshot,
    },
    MonitoringWorkloadLoaded {
        request_id: RuntimeRequestId,
        workload: db_pro_core::domain::monitoring::StatStatementsSnapshot,
    },
    AuditPageLoaded {
        request_id: RuntimeRequestId,
        page: db_pro_core::domain::audit::AuditPage,
    },
    PgSettingsLoaded {
        request_id: RuntimeRequestId,
        snapshot: db_pro_core::domain::pg_settings::PgSettingsSnapshot,
    },
    PgSettingActionCompleted {
        request_id: RuntimeRequestId,
        action: &'static str,
        name: String,
    },
    FdwInventoryLoaded {
        request_id: RuntimeRequestId,
        inventory: db_pro_core::domain::fdw::FdwInventory,
    },
    FdwActionCompleted {
        request_id: RuntimeRequestId,
        action: &'static str,
        name: String,
    },
    ReplicationInventoryLoaded {
        request_id: RuntimeRequestId,
        inventory: db_pro_core::domain::replication::ReplicationInventory,
    },
    ReplicationActionCompleted {
        request_id: RuntimeRequestId,
        action: &'static str,
        name: String,
    },
    EventTriggerInventoryLoaded {
        request_id: RuntimeRequestId,
        inventory: db_pro_core::domain::event_trigger::EventTriggerInventory,
    },
    EventTriggerActionCompleted {
        request_id: RuntimeRequestId,
        action: &'static str,
        name: String,
    },
    MonitoringActionCompleted {
        request_id: RuntimeRequestId,
        action: &'static str,
        backend_id: i64,
        succeeded: bool,
    },
    UsersLoaded {
        request_id: RuntimeRequestId,
        users: Vec<db_pro_core::domain::user::DatabaseUser>,
    },
    PrivilegesLoaded {
        request_id: RuntimeRequestId,
        role_name: String,
        privileges: Vec<db_pro_core::domain::user::Privilege>,
    },
    MembershipsLoaded {
        request_id: RuntimeRequestId,
        member: String,
        memberships: Vec<db_pro_core::domain::user::RoleMembership>,
    },
    TableRlsLoaded {
        request_id: RuntimeRequestId,
        state: db_pro_core::domain::rls::TableRlsState,
    },
    DataDiffLoaded {
        request_id: RuntimeRequestId,
        diff: db_pro_core::domain::cross_connection::DataDiff,
    },
    OperationCompleted {
        request_id: RuntimeRequestId,
        operation: &'static str,
    },
    TableChangesFailed {
        request_id: RuntimeRequestId,
        code: String,
        message: String,
        statement_index: usize,
        rolled_back: bool,
    },
    Connected {
        request_id: RuntimeRequestId,
        connection_id: String,
    },
    QueryCompleted {
        request_id: RuntimeRequestId,
        result: QueryResult,
    },
    QueryMultiCompleted {
        request_id: RuntimeRequestId,
        output: db_pro_core::application::MultiQueryResult,
    },
    QuerySaved {
        request_id: RuntimeRequestId,
        query: crate::SavedQuerySummary,
    },
    ExplainCompleted {
        request_id: RuntimeRequestId,
        plan: String,
    },
    QueryCancelled {
        request_id: RuntimeRequestId,
    },
    QueryFailedDetailed {
        request_id: RuntimeRequestId,
        error: crate::DbErrorDto,
    },
    AgentCompleted {
        request_id: RuntimeRequestId,
        provider: String,
        message: crate::AgentDraft,
    },
    AgentProviderReady {
        provider: String,
        detail: String,
    },
    AgentFailed {
        request_id: RuntimeRequestId,
        message: String,
    },
    AgentToolCompleted {
        request_id: RuntimeRequestId,
        session_id: db_pro_core::domain::agent::AgentSessionId,
        run_id: db_pro_core::domain::agent::AgentRunId,
        document_id: String,
        result: db_pro_core::domain::agent::AgentToolResult,
    },
    AgentToolFailed {
        request_id: RuntimeRequestId,
        session_id: db_pro_core::domain::agent::AgentSessionId,
        run_id: db_pro_core::domain::agent::AgentRunId,
        document_id: String,
        error: db_pro_core::domain::agent_workflow::AgentToolError,
    },
    AgentWorkflow {
        request_id: RuntimeRequestId,
        event: AgentWorkflowEvent,
    },
    AgentConfigured {
        request_id: RuntimeRequestId,
        provider: String,
        detail: String,
    },
    SqlPredictionReady {
        request_id: RuntimeRequestId,
        document_id: String,
        document_version: u64,
        anchor: usize,
        replacement_range: (usize, usize),
        prediction: String,
    },
    SqlPredictionFailed {
        request_id: RuntimeRequestId,
        document_id: String,
        document_version: u64,
        anchor: usize,
        replacement_range: (usize, usize),
        message: String,
    },
    Failed {
        request_id: RuntimeRequestId,
        message: String,
    },
}

type CancelMap = Arc<Mutex<HashMap<RuntimeRequestId, oneshot::Sender<()>>>>;

struct QueryCancellation {
    sender: oneshot::Sender<()>,
    query_api: QueryApi,
    connection_id: String,
}

type QueryCancelMap = Arc<Mutex<HashMap<RuntimeRequestId, QueryCancellation>>>;

type PredictionCancelMap = Arc<Mutex<HashMap<RuntimeRequestId, oneshot::Sender<()>>>>;
type PredictionCooldown = Arc<Mutex<Option<std::time::Instant>>>;
type AgentRunMap = Arc<Mutex<HashMap<AgentRunId, AgentRunOrchestrator>>>;

struct AgentCancellation {
    handle: tokio::task::AbortHandle,
    session_id: db_pro_core::domain::agent::AgentSessionId,
    document_id: String,
    connection_id: Option<String>,
}

type AgentCancellationMap = Arc<Mutex<HashMap<AgentRunId, AgentCancellation>>>;

/// Spawn the async worker that translates native UI commands into application
/// service calls. The UI receives only typed events and never sees credentials
/// or infrastructure handles.
pub fn spawn_worker(
    runtime: Arc<DbProRuntime>,
    capacity: usize,
) -> (mpsc::Sender<RuntimeCommand>, mpsc::Receiver<RuntimeEvent>) {
    let (command_tx, mut command_rx) = mpsc::channel(capacity);
    let (event_tx, event_rx) = mpsc::channel(capacity);
    let cancellations: CancelMap = Arc::new(Mutex::new(HashMap::new()));
    let query_cancellations: QueryCancelMap = Arc::new(Mutex::new(HashMap::new()));
    let prediction_cancellations: PredictionCancelMap = Arc::new(Mutex::new(HashMap::new()));
    let prediction_cooldown: PredictionCooldown = Arc::new(Mutex::new(None));
    let agent_runs: AgentRunMap = Arc::new(Mutex::new(HashMap::new()));
    let agent_cancellations: AgentCancellationMap = Arc::new(Mutex::new(HashMap::new()));
    // Shared mutable cell: allows ConfigureAgent to hot-swap the provider key
    // while the worker is running (no restart required).
    let codex_provider: Arc<Mutex<Option<CodexProvider>>> = Arc::new(Mutex::new(CodexProvider::from_env()));

    tokio::spawn(async move {
        let (provider, detail) = {
            let guard = codex_provider.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            guard
                .as_ref()
                .map(|p| {
                    (
                        p.provider_name().to_owned(),
                        "Responses API · SQL drafts stay unexecuted".to_owned(),
                    )
                })
                .unwrap_or_else(|| {
                    (
                        "Offline draft".to_owned(),
                        "AI provider not configured · local drafts stay unexecuted".to_owned(),
                    )
                })
        };
        let _ = event_tx
            .send(RuntimeEvent::AgentProviderReady { provider, detail })
            .await;
        while let Some(command) = command_rx.recv().await {
            match command {
                RuntimeCommand::ListConnections { request_id } => {
                    let event = match runtime.connection_api().list().await {
                        Ok(connections) => RuntimeEvent::ConnectionsLoaded {
                            request_id,
                            connections,
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::IntrospectSchema {
                    request_id,
                    connection_id,
                    force_refresh,
                } => {
                    let schema_api = runtime.schema_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        tracing::info!(
                            request_id = request_id.0,
                            connection_id = %connection_id,
                            force_refresh,
                            "schema introspection started"
                        );
                        let event = match schema_api.introspect_summary(&connection_id, force_refresh).await {
                            Ok(schema) => {
                                tracing::info!(
                                    request_id = request_id.0,
                                    connection_id = %connection_id,
                                    tables = schema.tables.len(),
                                    "schema introspection completed"
                                );
                                RuntimeEvent::SchemaLoaded { request_id, schema }
                            }
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::LoadTableInfo {
                    request_id,
                    connection_id,
                    schema,
                    table,
                } => {
                    let event = match runtime.schema_api().table_info(&connection_id, &schema, &table).await {
                        Ok(table_info) => RuntimeEvent::TableInfoLoaded { request_id, table_info },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::LoadTableDdl {
                    request_id,
                    connection_id,
                    schema,
                    table,
                } => {
                    let event = match runtime.schema_api().table_ddl(&connection_id, &schema, &table).await {
                        Ok(sql) => RuntimeEvent::TableDdlLoaded { request_id, sql },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::ExecuteDdl {
                    request_id,
                    connection_id,
                    sql,
                } => {
                    let event = match runtime.schema_api().execute_ddl(&connection_id, &sql).await {
                        Ok(affected_rows) => RuntimeEvent::DdlCompleted {
                            request_id,
                            affected_rows,
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::LoadTableData {
                    request_id,
                    connection_id,
                    schema,
                    table,
                    limit,
                    offset,
                    filters,
                    sorts,
                } => {
                    let event = match runtime
                        .table_data_api()
                        .fetch_rows(&connection_id, &schema, &table, &filters, &sorts, limit, offset)
                        .await
                    {
                        Ok((result, total_rows)) => RuntimeEvent::TableDataLoaded {
                            request_id,
                            result,
                            total_rows,
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::ListQueryFolders {
                    request_id,
                    connection_id,
                } => {
                    let event = match runtime.query_api().list_folders(&connection_id).await {
                        Ok(folders) => RuntimeEvent::QueryFoldersLoaded {
                            request_id,
                            folders: folders
                                .into_iter()
                                .map(|folder| crate::QueryFolderSummary {
                                    id: folder.id.to_string(),
                                    name: folder.name,
                                })
                                .collect(),
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::ListSavedQueries {
                    request_id,
                    connection_id,
                } => {
                    let event = match runtime.query_api().list_saved_queries(&connection_id).await {
                        Ok(queries) => RuntimeEvent::SavedQueriesLoaded {
                            request_id,
                            queries: queries
                                .into_iter()
                                .map(|query| crate::SavedQuerySummary {
                                    id: query.id.to_string(),
                                    name: query.name,
                                    sql: query.sql,
                                    folder: query.folder,
                                })
                                .collect(),
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::SaveQuery {
                    request_id,
                    connection_id,
                    saved_query_id,
                    name,
                    sql,
                    folder,
                } => {
                    let event = match runtime
                        .query_api()
                        .save_query_with_id(
                            &connection_id,
                            saved_query_id.as_deref(),
                            &name,
                            &sql,
                            folder.as_deref(),
                        )
                        .await
                    {
                        Ok(query) => RuntimeEvent::QuerySaved {
                            request_id,
                            query: crate::SavedQuerySummary {
                                id: query.id.to_string(),
                                name: query.name,
                                sql: query.sql,
                                folder: query.folder,
                            },
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::CreateQueryFolder {
                    request_id,
                    connection_id,
                    name,
                } => {
                    let event = match runtime.query_api().create_folder(&connection_id, &name).await {
                        Ok(_) => RuntimeEvent::OperationCompleted {
                            request_id,
                            operation: "query-folder.created",
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::RenameSavedQuery { request_id, id, name } => {
                    let event = match uuid::Uuid::parse_str(&id) {
                        Ok(id) => match runtime.query_api().rename_saved_query(&id, &name).await {
                            Ok(()) => RuntimeEvent::OperationCompleted {
                                request_id,
                                operation: "query.renamed",
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.to_string(),
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::DeleteSavedQuery { request_id, id } => {
                    let event = match uuid::Uuid::parse_str(&id) {
                        Ok(id) => match runtime.query_api().delete_saved_query(&id).await {
                            Ok(()) => RuntimeEvent::OperationCompleted {
                                request_id,
                                operation: "query.deleted",
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.to_string(),
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::DeleteQueryFolder { request_id, id } => {
                    let event = match uuid::Uuid::parse_str(&id) {
                        Ok(id) => match runtime.query_api().delete_folder(&id).await {
                            Ok(()) => RuntimeEvent::OperationCompleted {
                                request_id,
                                operation: "query-folder.deleted",
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.to_string(),
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::UpdateTableRow {
                    request_id,
                    connection_id,
                    schema,
                    table,
                    column,
                    value,
                    pk_columns,
                    pk_values,
                } => {
                    let event = match runtime
                        .table_data_api()
                        .update_row(
                            &connection_id,
                            &schema,
                            &table,
                            &[column],
                            &[value],
                            &pk_columns,
                            &pk_values,
                        )
                        .await
                    {
                        Ok(_) => RuntimeEvent::OperationCompleted {
                            request_id,
                            operation: "table-row.updated",
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::DeleteTableRow {
                    request_id,
                    connection_id,
                    schema,
                    table,
                    pk_columns,
                    pk_values,
                } => {
                    let event = match runtime
                        .table_data_api()
                        .delete_row(&connection_id, &schema, &table, &pk_columns, &pk_values)
                        .await
                    {
                        Ok(_) => RuntimeEvent::OperationCompleted {
                            request_id,
                            operation: "table-row.deleted",
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::InsertTableRow {
                    request_id,
                    connection_id,
                    schema,
                    table,
                    columns,
                    values,
                } => {
                    let event = match runtime
                        .table_data_api()
                        .insert_row(&connection_id, &schema, &table, &columns, &values)
                        .await
                    {
                        Ok(_) => RuntimeEvent::OperationCompleted {
                            request_id,
                            operation: "table-row.inserted",
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::ApplyTableChanges {
                    request_id,
                    connection_id,
                    schema,
                    table,
                    changes,
                } => {
                    let event = match runtime
                        .table_data_api()
                        .apply_mutations_detailed(&connection_id, &schema, &table, &changes)
                        .await
                    {
                        Ok(_) => RuntimeEvent::OperationCompleted {
                            request_id,
                            operation: "table-changes.applied",
                        },
                        Err(failure) => RuntimeEvent::TableChangesFailed {
                            request_id,
                            code: failure.error.code.clone(),
                            message: failure.error.message,
                            statement_index: failure.statement_index,
                            rolled_back: failure.rolled_back,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::RunAgent {
                    request_id,
                    prompt,
                    context,
                } => {
                    let provider = codex_provider
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .clone();
                    let Some(provider) = provider else {
                        let _ = event_tx
                            .send(RuntimeEvent::AgentFailed {
                                request_id,
                                message:
                                    "AI provider is not configured. Open Agent settings (⚙) and enter a Groq or OpenAI API key."
                                        .to_owned(),
                            })
                            .await;
                        continue;
                    };
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match provider.respond(&prompt, &context).await {
                            Ok(message) => RuntimeEvent::AgentCompleted {
                                request_id,
                                provider: provider.provider_name().to_owned(),
                                message,
                            },
                            Err(error) => RuntimeEvent::AgentFailed {
                                request_id,
                                message: error.to_string(),
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::ExecuteAgentTool {
                    request_id,
                    request,
                    context,
                } => {
                    let session_id = request.session_id;
                    let run_id = request.run_id;
                    let document_id = request.document_id.clone();
                    let executor = AgentToolExecutor::new(Arc::clone(&runtime));
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match executor.execute(&request, &context).await {
                            Ok(result) => RuntimeEvent::AgentToolCompleted {
                                request_id,
                                session_id,
                                run_id,
                                document_id,
                                result,
                            },
                            Err(error) => RuntimeEvent::AgentToolFailed {
                                request_id,
                                session_id,
                                run_id,
                                document_id,
                                error,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::StartAgentWorkflow {
                    request_id,
                    prompt,
                    session,
                    document,
                    mode,
                    allow_read_only_auto_run,
                    context,
                } => {
                    let provider = codex_provider
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .clone();
                    let Some(provider) = provider else {
                        let _ = event_tx
                            .send(RuntimeEvent::AgentFailed {
                                request_id,
                                message: "AI provider is not configured. Open Agent settings and enter an API key."
                                    .to_owned(),
                            })
                            .await;
                        continue;
                    };
                    let workflow = AgentWorkflow::new(session.clone(), mode, allow_read_only_auto_run);
                    let execution_context = AgentExecutionContext::new(session, document, mode);
                    let orchestrator = AgentRunOrchestrator::new(
                        Box::new(provider),
                        Box::new(AgentToolExecutor::new(Arc::clone(&runtime))),
                        workflow,
                        execution_context,
                        prompt,
                        context,
                    );
                    let Ok(mut orchestrator) = orchestrator else {
                        let _ = event_tx
                            .send(RuntimeEvent::AgentFailed {
                                request_id,
                                message: "Agent workflow could not start because its document is unavailable."
                                    .to_owned(),
                            })
                            .await;
                        continue;
                    };
                    let run_id = match orchestrator.run_id() {
                        Ok(run_id) => run_id,
                        Err(error) => {
                            let _ = event_tx
                                .send(RuntimeEvent::AgentFailed {
                                    request_id,
                                    message: error.to_string(),
                                })
                                .await;
                            continue;
                        }
                    };
                    let session_id = orchestrator.workflow().session().id;
                    let document_id = orchestrator.workflow().session().document_id.clone();
                    let connection_id = orchestrator.workflow().session().connection_id.clone();
                    let event_tx = event_tx.clone();
                    let agent_runs = Arc::clone(&agent_runs);
                    let agent_cancellations = Arc::clone(&agent_cancellations);
                    let task_cancellations = Arc::clone(&agent_cancellations);
                    let task = tokio::spawn(async move {
                        let events = orchestrator.run().await;
                        send_agent_workflow_events(&event_tx, request_id, events).await;
                        if orchestrator.has_pending_confirmation() {
                            agent_runs
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner())
                                .insert(run_id, orchestrator);
                        }
                        task_cancellations
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .remove(&run_id);
                    });
                    agent_cancellations
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .insert(
                            run_id,
                            AgentCancellation {
                                handle: task.abort_handle(),
                                session_id,
                                document_id,
                                connection_id,
                            },
                        );
                }
                RuntimeCommand::ContinueAgentWorkflow {
                    request_id,
                    run_id,
                    approved,
                    current_document,
                    applied_patch,
                } => {
                    let Some(mut orchestrator) = agent_runs
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .remove(&run_id)
                    else {
                        let _ = event_tx
                            .send(RuntimeEvent::AgentFailed {
                                request_id,
                                message: format!("Agent run {run_id:?} is no longer active."),
                            })
                            .await;
                        continue;
                    };
                    let session_id = orchestrator.workflow().session().id;
                    let document_id = orchestrator.workflow().session().document_id.clone();
                    let connection_id = orchestrator.workflow().session().connection_id.clone();
                    let event_tx = event_tx.clone();
                    let agent_runs = Arc::clone(&agent_runs);
                    let agent_cancellations = Arc::clone(&agent_cancellations);
                    let task_cancellations = Arc::clone(&agent_cancellations);
                    let task = tokio::spawn(async move {
                        match orchestrator
                            .resume_confirmation(approved, current_document, applied_patch)
                            .await
                        {
                            Ok(events) => {
                                send_agent_workflow_events(&event_tx, request_id, events).await;
                                if orchestrator.has_pending_confirmation() {
                                    agent_runs
                                        .lock()
                                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                                        .insert(run_id, orchestrator);
                                }
                            }
                            Err(error) => {
                                let _ = event_tx
                                    .send(RuntimeEvent::AgentFailed {
                                        request_id,
                                        message: error.to_string(),
                                    })
                                    .await;
                            }
                        }
                        task_cancellations
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .remove(&run_id);
                    });
                    agent_cancellations
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .insert(
                            run_id,
                            AgentCancellation {
                                handle: task.abort_handle(),
                                session_id,
                                document_id,
                                connection_id,
                            },
                        );
                }
                RuntimeCommand::CancelAgentWorkflow { request_id, run_id } => {
                    let cancellation = agent_cancellations
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .remove(&run_id);
                    if let Some(cancellation) = cancellation {
                        cancellation.handle.abort();
                        if let Some(connection_id) = cancellation.connection_id {
                            let runtime_cancel = Arc::clone(&runtime);
                            tokio::spawn(async move {
                                let _ = runtime_cancel.query_api().cancel(&connection_id).await;
                            });
                        }
                        let event = AgentWorkflowEvent::Cancelled {
                            run_id,
                            session_id: cancellation.session_id,
                            document_id: cancellation.document_id,
                        };
                        let _ = event_tx.send(RuntimeEvent::AgentWorkflow { request_id, event }).await;
                        continue;
                    }
                    let Some(mut orchestrator) = agent_runs
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .remove(&run_id)
                    else {
                        let _ = event_tx
                            .send(RuntimeEvent::AgentFailed {
                                request_id,
                                message: format!("Agent run {run_id:?} is no longer active."),
                            })
                            .await;
                        continue;
                    };
                    let connection_id = orchestrator.workflow().session().connection_id.clone();
                    if let Some(connection_id) = connection_id {
                        let runtime_cancel = Arc::clone(&runtime);
                        tokio::spawn(async move {
                            let _ = runtime_cancel.query_api().cancel(&connection_id).await;
                        });
                    }
                    match orchestrator.cancel() {
                        Ok(event) => {
                            let _ = event_tx.send(RuntimeEvent::AgentWorkflow { request_id, event }).await;
                        }
                        Err(error) => {
                            let _ = event_tx
                                .send(RuntimeEvent::AgentFailed {
                                    request_id,
                                    message: error.to_string(),
                                })
                                .await;
                        }
                    }
                }
                RuntimeCommand::RequestSqlPrediction {
                    request_id,
                    document_id,
                    document_version,
                    anchor,
                    replacement_range,
                    context,
                } => {
                    if prediction_is_in_cooldown(&prediction_cooldown) {
                        let _ = event_tx
                            .send(RuntimeEvent::SqlPredictionFailed {
                                request_id,
                                document_id,
                                document_version,
                                anchor,
                                replacement_range,
                                message: "AI prediction temporarily unavailable; retry shortly".to_owned(),
                            })
                            .await;
                        continue;
                    }
                    let provider = codex_provider
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .clone();
                    let Some(provider) = provider else {
                        let _ = event_tx
                            .send(RuntimeEvent::SqlPredictionFailed {
                                request_id,
                                document_id,
                                document_version,
                                anchor,
                                replacement_range,
                                message: "AI provider is not configured".to_owned(),
                            })
                            .await;
                        continue;
                    };
                    let (cancel_tx, mut cancel_rx) = oneshot::channel();
                    prediction_cancellations
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .insert(request_id, cancel_tx);
                    let event_tx = event_tx.clone();
                    let cancellation_map = Arc::clone(&prediction_cancellations);
                    let prediction_cooldown = Arc::clone(&prediction_cooldown);
                    tokio::spawn(async move {
                        let provider_started_at = std::time::Instant::now();
                        tokio::select! {
                            _ = &mut cancel_rx => {}
                            prediction = provider.predict_sql(&context) => {
                                let provider_latency_ms = provider_started_at.elapsed().as_millis();
                                tracing::debug!(
                                    request_id = request_id.0,
                                    document_id = %document_id,
                                    provider_latency_ms,
                                    "SQL prediction provider completed"
                                );
                                let backoff = prediction.as_ref().err().and_then(prediction_backoff);
                                let event = match prediction {
                                    Ok(prediction) => RuntimeEvent::SqlPredictionReady {
                                        request_id,
                                        document_id,
                                    document_version,
                                    anchor,
                                    replacement_range,
                                    prediction,
                                    },
                                    Err(error) => RuntimeEvent::SqlPredictionFailed {
                                        request_id,
                                        document_id,
                                        document_version,
                                        anchor,
                                        replacement_range,
                                        message: error.to_string(),
                                    },
                                };
                                if let Some(duration) = backoff {
                                    *prediction_cooldown
                                        .lock()
                                        .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                                        Some(std::time::Instant::now() + duration);
                                }
                                let _ = event_tx.send(event).await;
                            }
                        }
                        cancellation_map
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .remove(&request_id);
                    });
                }
                RuntimeCommand::CreateConnection {
                    request_id,
                    config,
                    password,
                } => {
                    let event = match runtime.connection_api().create(config, &password).await {
                        Ok(_) => RuntimeEvent::OperationCompleted {
                            request_id,
                            operation: "connection.created",
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::UpdateConnection {
                    request_id,
                    connection_id,
                    config,
                    password,
                } => {
                    let event = match runtime
                        .connection_api()
                        .update(&connection_id, config, password.as_deref())
                        .await
                    {
                        Ok(()) => RuntimeEvent::OperationCompleted {
                            request_id,
                            operation: "connection.updated",
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::DeleteConnection {
                    request_id,
                    connection_id,
                } => {
                    let event = match runtime.connection_api().delete(&connection_id).await {
                        Ok(()) => RuntimeEvent::OperationCompleted {
                            request_id,
                            operation: "connection.deleted",
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::TestConnection {
                    request_id,
                    config,
                    password,
                } => {
                    let connection_api = runtime.connection_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match connection_api.test(&config, &password).await {
                            Ok(()) => RuntimeEvent::OperationCompleted {
                                request_id,
                                operation: "connection.tested",
                            },
                            Err(error) => {
                                tracing::warn!(
                                    request_id = request_id.0,
                                    error = %error.message,
                                    "test connection failed"
                                );
                                RuntimeEvent::Failed {
                                    request_id,
                                    message: error.message,
                                }
                            }
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::Connect {
                    request_id,
                    connection_id,
                } => {
                    let connection_api = runtime.connection_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        tracing::info!(
                            request_id = request_id.0,
                            connection_id = %connection_id,
                            "connection started"
                        );
                        let event = match connection_api.connect(&connection_id).await {
                            Ok(()) => {
                                tracing::info!(
                                    request_id = request_id.0,
                                    connection_id = %connection_id,
                                    "connection completed"
                                );
                                RuntimeEvent::Connected {
                                    request_id,
                                    connection_id,
                                }
                            }
                            Err(error) => {
                                tracing::warn!(
                                    request_id = request_id.0,
                                    connection_id = %connection_id,
                                    error = %error.message,
                                    "connection failed"
                                );
                                RuntimeEvent::Failed {
                                    request_id,
                                    message: error.message,
                                }
                            }
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::ExecuteQuery {
                    request_id,
                    connection_id,
                    sql,
                    params,
                } => {
                    let (cancel_tx, cancel_rx) = oneshot::channel();
                    let query_api = runtime.query_api();
                    query_cancellations
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .insert(
                            request_id,
                            QueryCancellation {
                                sender: cancel_tx,
                                query_api: query_api.clone(),
                                connection_id: connection_id.clone(),
                            },
                        );
                    let event_tx = event_tx.clone();
                    let query_cancellations = Arc::clone(&query_cancellations);
                    tokio::spawn(async move {
                        let bound: Vec<db_pro_core::domain::query::QueryParam> = params
                            .into_iter()
                            .map(db_pro_core::domain::query::QueryParam::Text)
                            .collect();
                        let result = tokio::select! {
                            result = query_api.execute_with_params(
                                &connection_id,
                                &sql,
                                &bound,
                                None,
                                None,
                            ) => result,
                            _ = cancel_rx => Err(crate::DbErrorDto {
                                code: "QUERY_CANCELLED".to_owned(),
                                message: "Query cancelled".to_owned(),
                                message_id: "error.query.cancelled".to_owned(),
                                retryable: false,
                                position: None,
                            }),
                        };
                        let event = match result {
                            Ok(result) => RuntimeEvent::QueryCompleted { request_id, result },
                            Err(error) if error.code == "QUERY_CANCELLED" => {
                                RuntimeEvent::QueryCancelled { request_id }
                            }
                            Err(error) => RuntimeEvent::QueryFailedDetailed { request_id, error },
                        };
                        query_cancellations
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .remove(&request_id);
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::ExecuteQueryMulti {
                    request_id,
                    connection_id,
                    sql,
                } => {
                    let (cancel_tx, cancel_rx) = oneshot::channel();
                    let query_api = runtime.query_api();
                    query_cancellations
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .insert(
                            request_id,
                            QueryCancellation {
                                sender: cancel_tx,
                                query_api: query_api.clone(),
                                connection_id: connection_id.clone(),
                            },
                        );
                    let event_tx = event_tx.clone();
                    let query_cancellations = Arc::clone(&query_cancellations);
                    tokio::spawn(async move {
                        let result = tokio::select! {
                            result = query_api.execute_multi(&connection_id, &sql, None, None) => result,
                            _ = cancel_rx => Err(crate::DbErrorDto {
                                code: "QUERY_CANCELLED".to_owned(),
                                message: "Query cancelled".to_owned(),
                                message_id: "error.query.cancelled".to_owned(),
                                retryable: false,
                                position: None,
                            }),
                        };
                        let event = match result {
                            Ok(output) => RuntimeEvent::QueryMultiCompleted { request_id, output },
                            Err(error) if error.code == "QUERY_CANCELLED" => {
                                RuntimeEvent::QueryCancelled { request_id }
                            }
                            Err(error) => RuntimeEvent::QueryFailedDetailed { request_id, error },
                        };
                        query_cancellations
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .remove(&request_id);
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::ExplainQuery {
                    request_id,
                    connection_id,
                    sql,
                    analyze,
                } => {
                    let query_api = runtime.query_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match query_api.explain(&connection_id, &sql, analyze).await {
                            Ok(plan) => match serde_json::to_string_pretty(&plan) {
                                Ok(plan) => RuntimeEvent::ExplainCompleted { request_id, plan },
                                Err(error) => RuntimeEvent::Failed {
                                    request_id,
                                    message: format!("failed to format query plan: {error}"),
                                },
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::Backup { request_id, options } => {
                    let _ = event_tx
                        .send(RuntimeEvent::OperationProgress {
                            request_id,
                            operation: "backup",
                            status: "started",
                        })
                        .await;
                    let (cancel_tx, cancel_rx) = oneshot::channel();
                    cancellations
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .insert(request_id, cancel_tx);
                    let backup_api = runtime.backup_api();
                    let event_tx = event_tx.clone();
                    let cancellations = Arc::clone(&cancellations);
                    tokio::spawn(async move {
                        let result = tokio::select! {
                            result = backup_api.backup(&options) => Some(result),
                            _ = cancel_rx => None,
                        };
                        cancellations
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .remove(&request_id);
                        let status = match result {
                            Some(Ok(result)) => {
                                let _ = event_tx
                                    .send(RuntimeEvent::BackupCompleted {
                                        request_id,
                                        output_path: result.output_path,
                                        size_bytes: result.size_bytes,
                                    })
                                    .await;
                                "completed"
                            }
                            Some(Err(error)) => {
                                let _ = event_tx
                                    .send(RuntimeEvent::Failed {
                                        request_id,
                                        message: error.message,
                                    })
                                    .await;
                                "failed"
                            }
                            None => "cancelled",
                        };
                        let _ = event_tx
                            .send(RuntimeEvent::OperationProgress {
                                request_id,
                                operation: "backup",
                                status,
                            })
                            .await;
                    });
                }
                RuntimeCommand::Restore { request_id, options } => {
                    let _ = event_tx
                        .send(RuntimeEvent::OperationProgress {
                            request_id,
                            operation: "restore",
                            status: "started",
                        })
                        .await;
                    let (cancel_tx, cancel_rx) = oneshot::channel();
                    cancellations
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .insert(request_id, cancel_tx);
                    let backup_api = runtime.backup_api();
                    let event_tx = event_tx.clone();
                    let cancellations = Arc::clone(&cancellations);
                    tokio::spawn(async move {
                        let result = tokio::select! {
                            result = backup_api.restore(&options) => Some(result),
                            _ = cancel_rx => None,
                        };
                        cancellations
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .remove(&request_id);
                        let status = match result {
                            Some(Ok(())) => "completed",
                            Some(Err(error)) => {
                                let _ = event_tx
                                    .send(RuntimeEvent::Failed {
                                        request_id,
                                        message: error.message,
                                    })
                                    .await;
                                "failed"
                            }
                            None => "cancelled",
                        };
                        let _ = event_tx
                            .send(RuntimeEvent::OperationProgress {
                                request_id,
                                operation: "restore",
                                status,
                            })
                            .await;
                    });
                }
                RuntimeCommand::MonitoringSnapshot {
                    request_id,
                    connection_id,
                } => {
                    let monitoring_api = runtime.monitoring_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match monitoring_api.snapshot(&connection_id).await {
                            Ok(snapshot) => RuntimeEvent::MonitoringSnapshotLoaded { request_id, snapshot },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::MonitoringCancelBackend {
                    request_id,
                    connection_id,
                    backend_id,
                } => {
                    let monitoring_api = runtime.monitoring_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match monitoring_api.cancel_backend(&connection_id, backend_id).await {
                            Ok(succeeded) => RuntimeEvent::MonitoringActionCompleted {
                                request_id,
                                action: "cancel",
                                backend_id,
                                succeeded,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::MonitoringTerminateBackend {
                    request_id,
                    connection_id,
                    backend_id,
                } => {
                    let monitoring_api = runtime.monitoring_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match monitoring_api.terminate_backend(&connection_id, backend_id).await {
                            Ok(succeeded) => RuntimeEvent::MonitoringActionCompleted {
                                request_id,
                                action: "terminate",
                                backend_id,
                                succeeded,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::MonitoringMaintenance {
                    request_id,
                    connection_id,
                    schema,
                    table,
                    action,
                    confirmed,
                } => {
                    let monitoring_api = runtime.monitoring_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match monitoring_api
                            .run_maintenance(&connection_id, schema.as_deref(), table.as_deref(), action, confirmed)
                            .await
                        {
                            Ok(()) => RuntimeEvent::MonitoringActionCompleted {
                                request_id,
                                action: "maintenance",
                                backend_id: 0,
                                succeeded: true,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::MonitoringStatStatements {
                    request_id,
                    connection_id,
                    sort,
                    limit,
                } => {
                    let monitoring_api = runtime.monitoring_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match monitoring_api.stat_statements(&connection_id, sort, limit).await {
                            Ok(workload) => RuntimeEvent::MonitoringWorkloadLoaded { request_id, workload },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::MonitoringResetStatStatements {
                    request_id,
                    connection_id,
                    confirmed,
                } => {
                    let monitoring_api = runtime.monitoring_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match monitoring_api.reset_stat_statements(&connection_id, confirmed).await {
                            Ok(()) => RuntimeEvent::MonitoringActionCompleted {
                                request_id,
                                action: "reset_stat_statements",
                                backend_id: 0,
                                succeeded: true,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::AuditEventsLoad {
                    request_id,
                    connection_id,
                    filter,
                    limit,
                } => {
                    let audit_api = runtime.audit_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match audit_api.load_page(&connection_id, filter, limit).await {
                            Ok(page) => RuntimeEvent::AuditPageLoaded { request_id, page },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::ListPgSettings {
                    request_id,
                    connection_id,
                } => {
                    let postgres_api = runtime.postgres_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match postgres_api.list_pg_settings(&connection_id).await {
                            Ok(snapshot) => RuntimeEvent::PgSettingsLoaded { request_id, snapshot },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::SetPgSettingSession {
                    request_id,
                    connection_id,
                    name,
                    value,
                } => {
                    let postgres_api = runtime.postgres_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match postgres_api.set_pg_setting_session(&connection_id, &name, &value).await {
                            Ok(()) => RuntimeEvent::PgSettingActionCompleted {
                                request_id,
                                action: "set",
                                name,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::ResetPgSettingSession {
                    request_id,
                    connection_id,
                    name,
                } => {
                    let postgres_api = runtime.postgres_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match postgres_api.reset_pg_setting_session(&connection_id, &name).await {
                            Ok(()) => RuntimeEvent::PgSettingActionCompleted {
                                request_id,
                                action: "reset",
                                name,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::ListFdwInventory {
                    request_id,
                    connection_id,
                } => {
                    let postgres_api = runtime.postgres_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match postgres_api.list_fdw_inventory(&connection_id).await {
                            Ok(inventory) => RuntimeEvent::FdwInventoryLoaded { request_id, inventory },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::CreateFdwServer {
                    request_id,
                    connection_id,
                    name,
                    fdw,
                    host,
                    dbname,
                    port,
                    confirmed,
                } => {
                    let postgres_api = runtime.postgres_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match postgres_api
                            .create_fdw_server(&connection_id, &name, &fdw, &host, &dbname, &port, confirmed)
                            .await
                        {
                            Ok(()) => RuntimeEvent::FdwActionCompleted {
                                request_id,
                                action: "create_server",
                                name,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::DropFdwServer {
                    request_id,
                    connection_id,
                    name,
                    cascade,
                    confirmed,
                } => {
                    let postgres_api = runtime.postgres_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match postgres_api
                            .drop_fdw_server(&connection_id, &name, cascade, confirmed)
                            .await
                        {
                            Ok(()) => RuntimeEvent::FdwActionCompleted {
                                request_id,
                                action: "drop_server",
                                name,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::ListReplicationInventory {
                    request_id,
                    connection_id,
                } => {
                    let postgres_api = runtime.postgres_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match postgres_api.list_replication_inventory(&connection_id).await {
                            Ok(inventory) => RuntimeEvent::ReplicationInventoryLoaded { request_id, inventory },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::CreatePublicationAll {
                    request_id,
                    connection_id,
                    name,
                    confirmed,
                } => {
                    let postgres_api = runtime.postgres_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match postgres_api
                            .create_publication_all(&connection_id, &name, confirmed)
                            .await
                        {
                            Ok(()) => RuntimeEvent::ReplicationActionCompleted {
                                request_id,
                                action: "create_publication",
                                name,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::DropPublication {
                    request_id,
                    connection_id,
                    name,
                    confirmed,
                } => {
                    let postgres_api = runtime.postgres_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match postgres_api.drop_publication(&connection_id, &name, confirmed).await {
                            Ok(()) => RuntimeEvent::ReplicationActionCompleted {
                                request_id,
                                action: "drop_publication",
                                name,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::DropSubscription {
                    request_id,
                    connection_id,
                    name,
                    confirmed,
                } => {
                    let postgres_api = runtime.postgres_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match postgres_api.drop_subscription(&connection_id, &name, confirmed).await {
                            Ok(()) => RuntimeEvent::ReplicationActionCompleted {
                                request_id,
                                action: "drop_subscription",
                                name,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::ListEventTriggers {
                    request_id,
                    connection_id,
                } => {
                    let postgres_api = runtime.postgres_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match postgres_api.list_event_triggers(&connection_id).await {
                            Ok(inventory) => RuntimeEvent::EventTriggerInventoryLoaded { request_id, inventory },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::CreateEventTrigger {
                    request_id,
                    connection_id,
                    name,
                    event,
                    function_ref,
                    tags_csv,
                    confirmed,
                } => {
                    let postgres_api = runtime.postgres_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match postgres_api
                            .create_event_trigger(&connection_id, &name, &event, &function_ref, &tags_csv, confirmed)
                            .await
                        {
                            Ok(()) => RuntimeEvent::EventTriggerActionCompleted {
                                request_id,
                                action: "create",
                                name,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::DropEventTrigger {
                    request_id,
                    connection_id,
                    name,
                    confirmed,
                } => {
                    let postgres_api = runtime.postgres_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match postgres_api.drop_event_trigger(&connection_id, &name, confirmed).await {
                            Ok(()) => RuntimeEvent::EventTriggerActionCompleted {
                                request_id,
                                action: "drop",
                                name,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::AlterEventTrigger {
                    request_id,
                    connection_id,
                    name,
                    mode,
                    confirmed,
                } => {
                    let postgres_api = runtime.postgres_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match postgres_api
                            .alter_event_trigger(&connection_id, &name, &mode, confirmed)
                            .await
                        {
                            Ok(()) => RuntimeEvent::EventTriggerActionCompleted {
                                request_id,
                                action: "alter",
                                name,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::ListUsers {
                    request_id,
                    connection_id,
                } => {
                    let user_api = runtime.user_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match user_api.list_users(&connection_id).await {
                            Ok(users) => RuntimeEvent::UsersLoaded { request_id, users },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::CreateRole {
                    request_id,
                    connection_id,
                    name,
                    login,
                } => {
                    let user_api = runtime.user_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match user_api.create_role(&connection_id, &name, login).await {
                            Ok(()) => RuntimeEvent::OperationCompleted {
                                request_id,
                                operation: "create_role",
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::DropRole {
                    request_id,
                    connection_id,
                    name,
                } => {
                    let user_api = runtime.user_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match user_api.drop_role(&connection_id, &name).await {
                            Ok(()) => RuntimeEvent::OperationCompleted {
                                request_id,
                                operation: "drop_role",
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::ListPrivileges {
                    request_id,
                    connection_id,
                    role_name,
                } => {
                    let user_api = runtime.user_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match user_api.list_privileges(&connection_id, &role_name).await {
                            Ok(privileges) => RuntimeEvent::PrivilegesLoaded {
                                request_id,
                                role_name,
                                privileges,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::ListTableRls {
                    request_id,
                    connection_id,
                    schema,
                    table,
                } => {
                    let rls_api = runtime.rls_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match rls_api.table_rls_state(&connection_id, &schema, &table).await {
                            Ok(state) => RuntimeEvent::TableRlsLoaded { request_id, state },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::AlterRole {
                    request_id,
                    connection_id,
                    name,
                    attributes,
                } => {
                    let user_api = runtime.user_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match user_api.alter_role(&connection_id, &name, attributes).await {
                            Ok(()) => RuntimeEvent::OperationCompleted {
                                request_id,
                                operation: "alter_role",
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::UpdateRolePassword {
                    request_id,
                    connection_id,
                    name,
                    password,
                } => {
                    let user_api = runtime.user_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        // Do not include password in any event/log payload.
                        let event = match user_api.update_password(&connection_id, &name, &password).await {
                            Ok(()) => RuntimeEvent::OperationCompleted {
                                request_id,
                                operation: "update_role_password",
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        drop(password);
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::ListMemberships {
                    request_id,
                    connection_id,
                    member,
                } => {
                    let user_api = runtime.user_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match user_api.list_memberships(&connection_id, &member).await {
                            Ok(memberships) => RuntimeEvent::MembershipsLoaded {
                                request_id,
                                member,
                                memberships,
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::GrantMembership {
                    request_id,
                    connection_id,
                    role,
                    member,
                } => {
                    let user_api = runtime.user_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match user_api.grant_membership(&connection_id, &role, &member).await {
                            Ok(()) => RuntimeEvent::OperationCompleted {
                                request_id,
                                operation: "grant_membership",
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::RevokeMembership {
                    request_id,
                    connection_id,
                    role,
                    member,
                } => {
                    let user_api = runtime.user_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match user_api.revoke_membership(&connection_id, &role, &member).await {
                            Ok(()) => RuntimeEvent::OperationCompleted {
                                request_id,
                                operation: "revoke_membership",
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::GrantPrivilege {
                    request_id,
                    connection_id,
                    role_name,
                    object_kind,
                    schema,
                    object_name,
                    privilege,
                } => {
                    let user_api = runtime.user_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match user_api
                            .grant_privilege(
                                &connection_id,
                                &role_name,
                                object_kind,
                                &schema,
                                &object_name,
                                &privilege,
                            )
                            .await
                        {
                            Ok(()) => RuntimeEvent::OperationCompleted {
                                request_id,
                                operation: "grant_privilege",
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::RevokePrivilege {
                    request_id,
                    connection_id,
                    role_name,
                    object_kind,
                    schema,
                    object_name,
                    privilege,
                } => {
                    let user_api = runtime.user_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match user_api
                            .revoke_privilege(
                                &connection_id,
                                &role_name,
                                object_kind,
                                &schema,
                                &object_name,
                                &privilege,
                            )
                            .await
                        {
                            Ok(()) => RuntimeEvent::OperationCompleted {
                                request_id,
                                operation: "revoke_privilege",
                            },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::DiffTableDataKeyed {
                    request_id,
                    source_id,
                    target_id,
                    schema,
                    table,
                    key_columns,
                    sample_limit,
                } => {
                    let data_diff_api = runtime.data_diff_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match data_diff_api
                            .diff_table_data_keyed(&source_id, &target_id, &schema, &table, &key_columns, sample_limit)
                            .await
                        {
                            Ok(diff) => RuntimeEvent::DataDiffLoaded { request_id, diff },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::CancelQuery { request_id } => {
                    let query_cancellation = query_cancellations
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .remove(&request_id);
                    if let Some(query_cancellation) = query_cancellation {
                        match query_cancellation
                            .query_api
                            .cancel(&query_cancellation.connection_id)
                            .await
                        {
                            Ok(()) => {
                                let _ = query_cancellation.sender.send(());
                            }
                            Err(error) => {
                                tracing::warn!(
                                    request_id = request_id.0,
                                    code = %error.code,
                                    "query cancellation was not applied"
                                );
                                let _ = event_tx
                                    .send(RuntimeEvent::Failed {
                                        request_id,
                                        message: format!("query cancellation failed: {}", error.message),
                                    })
                                    .await;
                            }
                        }
                    }
                }
                RuntimeCommand::CancelSqlPrediction { request_id } => {
                    if let Some(sender) = prediction_cancellations
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .remove(&request_id)
                    {
                        let _ = sender.send(());
                    }
                }
                RuntimeCommand::CancelOperation { request_id } => {
                    if let Some(sender) = cancellations
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .remove(&request_id)
                    {
                        let _ = sender.send(());
                    }
                }
                RuntimeCommand::ConfigureAgent { request_id, api_key } => {
                    // Build the new provider (Groq if key looks like one, else OpenAI).
                    let (endpoint, model, provider_name) = if api_key.trim_start().starts_with("gsk_") {
                        (
                            crate::agent::DEFAULT_GROQ_ENDPOINT,
                            crate::agent::DEFAULT_GROQ_MODEL,
                            "Groq",
                        )
                    } else {
                        (crate::agent::DEFAULT_ENDPOINT, crate::agent::DEFAULT_MODEL, "OpenAI")
                    };
                    let event = match CodexProvider::with_provider_pub(
                        api_key.trim().to_owned(),
                        endpoint,
                        model,
                        provider_name,
                    ) {
                        Ok(new_provider) => {
                            let detail = format!("{provider_name} Responses API · SQL drafts stay unexecuted");
                            *codex_provider.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) =
                                Some(new_provider);
                            tracing::info!(provider = provider_name, "AI provider reconfigured");
                            RuntimeEvent::AgentConfigured {
                                request_id,
                                provider: provider_name.to_owned(),
                                detail,
                            }
                        }
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: format!("AI provider configuration rejected: {error}"),
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
            }
        }
    });

    (command_tx, event_rx)
}

fn prediction_is_in_cooldown(cooldown: &PredictionCooldown) -> bool {
    let mut guard = cooldown.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    match *guard {
        Some(until) if until > std::time::Instant::now() => true,
        Some(_) => {
            *guard = None;
            false
        }
        None => false,
    }
}

async fn send_agent_workflow_events(
    event_tx: &mpsc::Sender<RuntimeEvent>,
    request_id: RuntimeRequestId,
    events: Vec<AgentWorkflowEvent>,
) {
    for event in events {
        if event_tx
            .send(RuntimeEvent::AgentWorkflow { request_id, event })
            .await
            .is_err()
        {
            break;
        }
    }
}

fn prediction_backoff(error: &CodexProviderError) -> Option<std::time::Duration> {
    match error {
        CodexProviderError::Http { status: 429, .. } => Some(std::time::Duration::from_secs(2)),
        CodexProviderError::Http { status, .. } if *status >= 500 => Some(std::time::Duration::from_secs(1)),
        CodexProviderError::Request(_) => Some(std::time::Duration::from_secs(1)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prediction_backoff_is_bounded_and_only_for_transient_provider_errors() {
        assert_eq!(
            prediction_backoff(&CodexProviderError::Http {
                status: 429,
                body: String::new(),
            }),
            Some(std::time::Duration::from_secs(2))
        );
        assert_eq!(
            prediction_backoff(&CodexProviderError::Http {
                status: 503,
                body: String::new(),
            }),
            Some(std::time::Duration::from_secs(1))
        );
        assert_eq!(prediction_backoff(&CodexProviderError::EmptyResponse), None);
    }
}
