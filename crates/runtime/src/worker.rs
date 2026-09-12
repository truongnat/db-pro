use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use db_pro_core::application::sql_builder::{SortClause, TableFilter};
use db_pro_core::domain::backup::{BackupOptions, RestoreOptions};
use db_pro_core::domain::query::{CellValue, QueryResult};
use tokio::sync::{mpsc, oneshot};

use crate::{AgentContext, CodexProvider, ConnectionSummary, DbProRuntime, QueryApi};

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
        filter: Option<TableFilter>,
        sort: Option<SortClause>,
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
    RunAgent {
        request_id: RuntimeRequestId,
        prompt: String,
        context: AgentContext,
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
    },
    ExplainQuery {
        request_id: RuntimeRequestId,
        connection_id: String,
        sql: String,
    },
    Backup {
        request_id: RuntimeRequestId,
        options: BackupOptions,
    },
    Restore {
        request_id: RuntimeRequestId,
        options: RestoreOptions,
    },
    CancelQuery {
        request_id: RuntimeRequestId,
    },
    CancelOperation {
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
    OperationCompleted {
        request_id: RuntimeRequestId,
        operation: &'static str,
    },
    Connected {
        request_id: RuntimeRequestId,
        connection_id: String,
    },
    QueryCompleted {
        request_id: RuntimeRequestId,
        result: QueryResult,
    },
    ExplainCompleted {
        request_id: RuntimeRequestId,
        plan: String,
    },
    QueryCancelled {
        request_id: RuntimeRequestId,
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
    let codex_provider = CodexProvider::from_env();

    tokio::spawn(async move {
        let (provider, detail) = codex_provider
            .as_ref()
            .map(|provider| {
                (
                    provider.provider_name().to_owned(),
                    "Responses API · SQL drafts stay unexecuted".to_owned(),
                )
            })
            .unwrap_or_else(|| {
                (
                    "Offline draft".to_owned(),
                    "AI provider not configured · local drafts stay unexecuted".to_owned(),
                )
            });
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
                    tracing::info!(
                        request_id = request_id.0,
                        connection_id = %connection_id,
                        force_refresh,
                        "schema introspection started"
                    );
                    let event = match runtime
                        .schema_api()
                        .introspect_summary(&connection_id, force_refresh)
                        .await
                    {
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
                    filter,
                    sort,
                } => {
                    let filters = filter.into_iter().collect::<Vec<_>>();
                    let sorts = sort.into_iter().collect::<Vec<_>>();
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
                    name,
                    sql,
                    folder,
                } => {
                    let event = match runtime
                        .query_api()
                        .save_query(&connection_id, &name, &sql, folder.as_deref())
                        .await
                    {
                        Ok(_) => RuntimeEvent::OperationCompleted {
                            request_id,
                            operation: "query.saved",
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
                RuntimeCommand::RunAgent {
                    request_id,
                    prompt,
                    context,
                } => {
                    let Some(provider) = codex_provider.clone() else {
                        let _ = event_tx
                            .send(RuntimeEvent::AgentFailed {
                                request_id,
                                message:
                                    "AI provider is not configured; set GROQ_API_KEY or OPENAI_API_KEY to enable it."
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
                    let event = match runtime.connection_api().test(&config, &password).await {
                        Ok(()) => RuntimeEvent::OperationCompleted {
                            request_id,
                            operation: "connection.tested",
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::Connect {
                    request_id,
                    connection_id,
                } => {
                    tracing::info!(
                        request_id = request_id.0,
                        connection_id = %connection_id,
                        "connection started"
                    );
                    let event = match runtime.connection_api().connect(&connection_id).await {
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
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::ExecuteQuery {
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
                            result = query_api.execute(&connection_id, &sql) => result,
                            _ = cancel_rx => Err(crate::DbErrorDto {
                                code: "QUERY_CANCELLED".to_owned(),
                                message: "Query cancelled".to_owned(),
                                message_id: "error.query.cancelled".to_owned(),
                                retryable: false,
                            }),
                        };
                        let event = match result {
                            Ok(result) => RuntimeEvent::QueryCompleted { request_id, result },
                            Err(error) if error.code == "QUERY_CANCELLED" => {
                                RuntimeEvent::QueryCancelled { request_id }
                            }
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
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
                } => {
                    let query_api = runtime.query_api();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = match query_api.explain(&connection_id, &sql).await {
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
                            }
                        }
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
            }
        }
    });

    (command_tx, event_rx)
}
