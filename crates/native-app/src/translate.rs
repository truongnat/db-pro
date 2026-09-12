use super::*;

pub(crate) fn draft_to_domain(
    draft: UiConnectionDraft,
) -> Option<(db_pro_core::domain::connection::ConnectionConfig, String)> {
    let port = draft.port.parse::<u16>().ok()?;
    let driver = match draft.driver {
        UiDriver::Postgres => db_pro_core::domain::connection::DriverType::Postgres,
        UiDriver::Sqlite => db_pro_core::domain::connection::DriverType::SQLite,
    };
    let ssl_mode = match draft.ssl_mode {
        UiSslMode::Disable => db_pro_core::domain::connection::SslMode::Disable,
        UiSslMode::Require => db_pro_core::domain::connection::SslMode::Require,
        UiSslMode::VerifyCa => db_pro_core::domain::connection::SslMode::VerifyCa,
        UiSslMode::VerifyFull => db_pro_core::domain::connection::SslMode::VerifyFull,
    };
    let ssh_tunnel = if driver == db_pro_core::domain::connection::DriverType::Postgres && draft.ssh_tunnel_enabled {
        Some(db_pro_core::domain::connection::SshTunnelConfig {
            host: draft.ssh_host.clone(),
            port: draft.ssh_port.parse::<u16>().ok()?,
            user: draft.ssh_user.clone(),
            private_key_path: draft.ssh_private_key.clone(),
            password: None,
        })
    } else {
        None
    };
    Some((
        db_pro_core::domain::connection::ConnectionConfig {
            name: draft.name,
            host: draft.host,
            port,
            database: draft.database,
            username: draft.username,
            driver,
            ssl_mode,
            ssh_tunnel,
            query_timeout_ms: 30_000,
            max_rows: 500,
            color: None,
            tags: Vec::new(),
            group: None,
            readonly: draft.readonly,
        },
        if driver == db_pro_core::domain::connection::DriverType::Postgres {
            draft.password
        } else {
            String::new()
        },
    ))
}

pub(crate) fn translate_command(command: UiCommand) -> Option<RuntimeCommand> {
    match &command {
        UiCommand::OpenQuery
        | UiCommand::PickSqliteFile { .. }
        | UiCommand::PickSshPrivateKey { .. }
        | UiCommand::PickBackupFile { .. }
        | UiCommand::PickRestoreFile { .. } => None,
        UiCommand::ListQueryFolders { .. }
        | UiCommand::ListSavedQueries { .. }
        | UiCommand::SaveQuery { .. }
        | UiCommand::CreateQueryFolder { .. }
        | UiCommand::RenameSavedQuery { .. }
        | UiCommand::DeleteSavedQuery { .. }
        | UiCommand::DeleteQueryFolder { .. } => translate_query_command(command),
        UiCommand::ExecuteDdl { .. }
        | UiCommand::UpdateTableRow { .. }
        | UiCommand::DeleteTableRow { .. }
        | UiCommand::InsertTableRow { .. } => translate_table_command(command),
        UiCommand::RunAgent { .. }
        | UiCommand::IntrospectSchema { .. }
        | UiCommand::LoadTableInfo { .. }
        | UiCommand::LoadTableDdl { .. }
        | UiCommand::LoadTableData { .. } => translate_schema_command(command),
        UiCommand::ListConnections { .. }
        | UiCommand::CreateConnection { .. }
        | UiCommand::UpdateConnection { .. }
        | UiCommand::TestConnection { .. }
        | UiCommand::DeleteConnection { .. }
        | UiCommand::Connect { .. } => translate_connection_command(command),
        UiCommand::RunQuery { .. }
        | UiCommand::ExplainQuery { .. }
        | UiCommand::Backup { .. }
        | UiCommand::Restore { .. }
        | UiCommand::CancelQuery { .. } => translate_execution_command(command),
    }
}

fn translate_query_command(command: UiCommand) -> Option<RuntimeCommand> {
    match command {
        UiCommand::ListQueryFolders {
            request_id,
            connection_id,
        } => Some(RuntimeCommand::ListQueryFolders {
            request_id: runtime_request_id(request_id),
            connection_id,
        }),
        UiCommand::ListSavedQueries {
            request_id,
            connection_id,
        } => Some(RuntimeCommand::ListSavedQueries {
            request_id: runtime_request_id(request_id),
            connection_id,
        }),
        UiCommand::SaveQuery {
            request_id,
            connection_id,
            name,
            sql,
            folder,
        } => Some(RuntimeCommand::SaveQuery {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
            sql,
            folder,
        }),
        UiCommand::CreateQueryFolder {
            request_id,
            connection_id,
            name,
        } => Some(RuntimeCommand::CreateQueryFolder {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
        }),
        UiCommand::RenameSavedQuery { request_id, id, name } => Some(RuntimeCommand::RenameSavedQuery {
            request_id: runtime_request_id(request_id),
            id,
            name,
        }),
        UiCommand::DeleteSavedQuery { request_id, id } => Some(RuntimeCommand::DeleteSavedQuery {
            request_id: runtime_request_id(request_id),
            id,
        }),
        UiCommand::DeleteQueryFolder { request_id, id } => Some(RuntimeCommand::DeleteQueryFolder {
            request_id: runtime_request_id(request_id),
            id,
        }),
        _ => None,
    }
}

fn translate_table_command(command: UiCommand) -> Option<RuntimeCommand> {
    match command {
        UiCommand::ExecuteDdl {
            request_id,
            connection_id,
            sql,
        } => Some(RuntimeCommand::ExecuteDdl {
            request_id: runtime_request_id(request_id),
            connection_id,
            sql,
        }),
        UiCommand::UpdateTableRow {
            request_id,
            connection_id,
            schema,
            table,
            column,
            value,
            pk_columns,
            pk_values,
        } => {
            let value = ui_cell_to_domain(value)?;
            let pk_values = pk_values
                .into_iter()
                .map(ui_cell_to_domain)
                .collect::<Option<Vec<_>>>()?;
            Some(RuntimeCommand::UpdateTableRow {
                request_id: runtime_request_id(request_id),
                connection_id,
                schema,
                table,
                column,
                value,
                pk_columns,
                pk_values,
            })
        }
        UiCommand::DeleteTableRow {
            request_id,
            connection_id,
            schema,
            table,
            pk_columns,
            pk_values,
        } => {
            let pk_values = pk_values
                .into_iter()
                .map(ui_cell_to_domain)
                .collect::<Option<Vec<_>>>()?;
            Some(RuntimeCommand::DeleteTableRow {
                request_id: runtime_request_id(request_id),
                connection_id,
                schema,
                table,
                pk_columns,
                pk_values,
            })
        }
        UiCommand::InsertTableRow {
            request_id,
            connection_id,
            schema,
            table,
            columns,
            values,
        } => {
            let values = values.into_iter().map(ui_cell_to_domain).collect::<Option<Vec<_>>>()?;
            Some(RuntimeCommand::InsertTableRow {
                request_id: runtime_request_id(request_id),
                connection_id,
                schema,
                table,
                columns,
                values,
            })
        }
        _ => None,
    }
}

fn translate_schema_command(command: UiCommand) -> Option<RuntimeCommand> {
    match command {
        UiCommand::RunAgent {
            request_id,
            prompt,
            context,
        } => Some(RuntimeCommand::RunAgent {
            request_id: runtime_request_id(request_id),
            prompt,
            context: db_pro_runtime::AgentContext {
                connection_name: context.connection_name,
                driver: context.driver,
                tables: context.tables,
                columns: context.columns,
                schema: context.schema,
                selected_table: context.selected_table,
                selected_columns: context.selected_columns,
                current_sql: context.current_sql,
                result_summary: context.result_summary,
                explain_plan: context.explain_plan,
                last_error: context.last_error,
            },
        }),
        UiCommand::IntrospectSchema {
            request_id,
            connection_id,
            force_refresh,
        } => Some(RuntimeCommand::IntrospectSchema {
            request_id: runtime_request_id(request_id),
            connection_id,
            force_refresh,
        }),
        UiCommand::LoadTableInfo {
            request_id,
            connection_id,
            schema,
            table,
        } => Some(RuntimeCommand::LoadTableInfo {
            request_id: runtime_request_id(request_id),
            connection_id,
            schema,
            table,
        }),
        UiCommand::LoadTableDdl {
            request_id,
            connection_id,
            schema,
            table,
        } => Some(RuntimeCommand::LoadTableDdl {
            request_id: runtime_request_id(request_id),
            connection_id,
            schema,
            table,
        }),
        UiCommand::LoadTableData {
            request_id,
            connection_id,
            schema,
            table,
            limit,
            offset,
            filter,
            sort,
        } => Some(RuntimeCommand::LoadTableData {
            request_id: runtime_request_id(request_id),
            connection_id,
            schema,
            table,
            limit,
            offset,
            filter: filter.map(map_table_data_filter),
            sort: sort.map(map_table_data_sort),
        }),
        _ => None,
    }
}

fn translate_connection_command(command: UiCommand) -> Option<RuntimeCommand> {
    match command {
        UiCommand::ListConnections { request_id } => Some(RuntimeCommand::ListConnections {
            request_id: runtime_request_id(request_id),
        }),
        UiCommand::CreateConnection { request_id, draft } => {
            let (config, password) = draft_to_domain(draft)?;
            Some(RuntimeCommand::CreateConnection {
                request_id: runtime_request_id(request_id),
                config,
                password,
            })
        }
        UiCommand::UpdateConnection {
            request_id,
            connection_id,
            draft,
        } => {
            let (config, password) = draft_to_domain(draft)?;
            Some(RuntimeCommand::UpdateConnection {
                request_id: runtime_request_id(request_id),
                connection_id,
                config,
                password: Some(password),
            })
        }
        UiCommand::TestConnection { request_id, draft } => {
            let (config, password) = draft_to_domain(draft)?;
            Some(RuntimeCommand::TestConnection {
                request_id: runtime_request_id(request_id),
                config,
                password,
            })
        }
        UiCommand::DeleteConnection {
            request_id,
            connection_id,
        } => Some(RuntimeCommand::DeleteConnection {
            request_id: runtime_request_id(request_id),
            connection_id,
        }),
        UiCommand::Connect {
            request_id,
            connection_id,
        } => Some(RuntimeCommand::Connect {
            request_id: runtime_request_id(request_id),
            connection_id,
        }),
        _ => None,
    }
}

fn translate_execution_command(command: UiCommand) -> Option<RuntimeCommand> {
    match command {
        UiCommand::RunQuery {
            request_id,
            connection_id,
            sql,
        } => Some(RuntimeCommand::ExecuteQuery {
            request_id: runtime_request_id(request_id),
            connection_id,
            sql,
        }),
        UiCommand::ExplainQuery {
            request_id,
            connection_id,
            sql,
        } => Some(RuntimeCommand::ExplainQuery {
            request_id: runtime_request_id(request_id),
            connection_id,
            sql,
        }),
        UiCommand::Backup {
            request_id,
            connection_id,
            output_path,
            custom_format,
        } => Some(RuntimeCommand::Backup {
            request_id: runtime_request_id(request_id),
            options: db_pro_core::domain::backup::BackupOptions {
                connection_id,
                output_path,
                format: if custom_format {
                    db_pro_core::domain::backup::BackupFormat::Custom
                } else {
                    db_pro_core::domain::backup::BackupFormat::Plain
                },
                schemas: Vec::new(),
                tables: Vec::new(),
            },
        }),
        UiCommand::Restore {
            request_id,
            connection_id,
            input_path,
            custom_format,
        } => Some(RuntimeCommand::Restore {
            request_id: runtime_request_id(request_id),
            options: db_pro_core::domain::backup::RestoreOptions {
                connection_id,
                input_path,
                format: if custom_format {
                    db_pro_core::domain::backup::BackupFormat::Custom
                } else {
                    db_pro_core::domain::backup::BackupFormat::Plain
                },
            },
        }),
        UiCommand::CancelQuery { request_id } => Some(RuntimeCommand::CancelQuery {
            request_id: runtime_request_id(request_id),
        }),
        _ => None,
    }
}

fn map_table_data_filter(filter: UiTableDataFilter) -> TableFilter {
    TableFilter {
        column: filter.column,
        op: FilterOp::Like,
        value: CellValue::Text(format!("%{}%", filter.value)),
    }
}

fn map_table_data_sort(sort: UiTableDataSort) -> SortClause {
    SortClause {
        column: sort.column,
        direction: if sort.descending { SortDir::Desc } else { SortDir::Asc },
    }
}

fn map_cell(cell: db_pro_core::domain::query::CellValue) -> UiCell {
    use db_pro_core::domain::query::CellValue;

    match cell {
        CellValue::Null => UiCell::Null,
        CellValue::Bool(value) => UiCell::Boolean(value),
        CellValue::Int64(value) => UiCell::Number(value.to_string()),
        CellValue::Float64(value) => UiCell::Number(value.to_string()),
        CellValue::Text(value)
        | CellValue::Uuid(value)
        | CellValue::DateTime(value)
        | CellValue::Date(value)
        | CellValue::Time(value)
        | CellValue::Interval(value)
        | CellValue::Inet(value) => UiCell::Text(value),
        CellValue::Bytes(value) => UiCell::Bytes(format!("\\x{}", hex_encode(&value))),
        CellValue::Json(value) => UiCell::Json(value.to_string()),
    }
}

fn ui_cell_to_domain(cell: UiCell) -> Option<CellValue> {
    match cell {
        UiCell::Null => Some(CellValue::Null),
        UiCell::Boolean(value) => Some(CellValue::Bool(value)),
        UiCell::Number(value) => value
            .parse::<i64>()
            .map(CellValue::Int64)
            .or_else(|_| value.parse::<f64>().map(CellValue::Float64))
            .ok(),
        UiCell::Text(value) => Some(CellValue::Text(value)),
        UiCell::Json(value) => serde_json::from_str(&value).ok().map(CellValue::Json),
        UiCell::Bytes(value) => {
            let value = value.strip_prefix("\\x").unwrap_or(&value);
            if value.len() % 2 != 0 {
                return None;
            }
            (0..value.len())
                .step_by(2)
                .map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok())
                .collect::<Option<Vec<_>>>()
                .map(CellValue::Bytes)
        }
    }
}

fn map_table_info(info: db_pro_core::domain::schema::TableInfo) -> UiTableInfo {
    UiTableInfo {
        schema: info.table.schema,
        name: info.table.name,
        row_count: info.table.row_count,
        columns: info
            .columns
            .into_iter()
            .map(|column| UiTableColumn {
                name: column.name,
                data_type: column.data_type,
                nullable: column.nullable,
                default: column.default,
                is_primary_key: column.is_primary_key,
            })
            .collect(),
        primary_key: info.primary_key.map(|primary_key| primary_key.columns),
        indexes: info
            .indexes
            .into_iter()
            .map(|index| UiTableIndex {
                name: index.name,
                columns: index.columns,
                unique: index.unique,
            })
            .collect(),
        foreign_keys: info
            .foreign_keys
            .into_iter()
            .map(|foreign_key| UiTableForeignKey {
                name: foreign_key.name,
                from_columns: foreign_key.from_columns,
                to_schema: foreign_key.to_schema,
                to_table: foreign_key.to_table,
                to_columns: foreign_key.to_columns,
            })
            .collect(),
    }
}

fn map_query_result(result: db_pro_core::domain::query::QueryResult) -> UiQueryResult {
    UiQueryResult {
        row_count: result.row_count,
        duration_ms: result.duration_ms,
        columns: result
            .columns
            .into_iter()
            .map(|column| UiColumn {
                name: column.name,
                data_type: column.data_type,
                nullable: column.nullable,
            })
            .collect(),
        rows: result
            .rows
            .into_iter()
            .map(|row| row.0.into_iter().map(map_cell).collect())
            .collect(),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn translate_event(event: RuntimeEvent) -> Option<UiEvent> {
    match event {
        RuntimeEvent::ConnectionsLoaded {
            request_id,
            connections,
        } => translate_connections_loaded(request_id, connections),
        RuntimeEvent::SchemaLoaded { request_id, schema } => translate_schema_loaded(request_id, schema),
        RuntimeEvent::TableInfoLoaded { request_id, table_info } => translate_table_info_loaded(request_id, table_info),
        RuntimeEvent::TableDdlLoaded { request_id, sql } => translate_table_ddl_loaded(request_id, sql),
        RuntimeEvent::DdlCompleted {
            request_id,
            affected_rows,
        } => translate_ddl_completed(request_id, affected_rows),
        RuntimeEvent::QueryFoldersLoaded { request_id, folders } => translate_query_folders_loaded(request_id, folders),
        RuntimeEvent::SavedQueriesLoaded { request_id, queries } => translate_saved_queries_loaded(request_id, queries),
        RuntimeEvent::OperationProgress {
            request_id,
            operation,
            status,
        } => translate_operation_progress(request_id, operation, status),
        RuntimeEvent::BackupCompleted {
            request_id,
            output_path,
            size_bytes,
        } => translate_backup_completed(request_id, output_path, size_bytes),
        RuntimeEvent::OperationCompleted { request_id, operation } => {
            translate_operation_completed(request_id, operation)
        }
        RuntimeEvent::Connected {
            request_id,
            connection_id,
        } => translate_connected(request_id, connection_id),
        RuntimeEvent::QueryCompleted { request_id, result } => translate_query_completed(request_id, result),
        RuntimeEvent::ExplainCompleted { request_id, plan } => translate_explain_completed(request_id, plan),
        RuntimeEvent::TableDataLoaded {
            request_id,
            result,
            total_rows,
        } => translate_table_data_loaded(request_id, result, total_rows),
        RuntimeEvent::QueryCancelled { request_id } => translate_query_cancelled(request_id),
        RuntimeEvent::AgentCompleted {
            request_id,
            provider,
            message,
        } => translate_agent_completed(request_id, provider, message),
        RuntimeEvent::AgentProviderReady { provider, detail } => Some(UiEvent::AgentProviderReady { provider, detail }),
        RuntimeEvent::AgentFailed { request_id, message } => translate_agent_failed(request_id, message),
        RuntimeEvent::Failed { request_id, message } => translate_failed(request_id, message),
    }
}

fn ui_request_id(request_id: RuntimeRequestId) -> db_pro_ui::RequestId {
    db_pro_ui::RequestId(request_id.0)
}

fn runtime_request_id(id: RequestId) -> RuntimeRequestId {
    RuntimeRequestId(id.0)
}

fn translate_table_info_loaded(
    request_id: RuntimeRequestId,
    table_info: db_pro_core::domain::schema::TableInfo,
) -> Option<UiEvent> {
    Some(UiEvent::TableInfoLoaded {
        request_id: ui_request_id(request_id),
        table_info: map_table_info(table_info),
    })
}

fn translate_table_ddl_loaded(request_id: RuntimeRequestId, sql: String) -> Option<UiEvent> {
    Some(UiEvent::TableDdlLoaded {
        request_id: ui_request_id(request_id),
        sql,
    })
}

fn translate_ddl_completed(request_id: RuntimeRequestId, affected_rows: u64) -> Option<UiEvent> {
    Some(UiEvent::DdlCompleted {
        request_id: ui_request_id(request_id),
        affected_rows,
    })
}

fn translate_operation_progress(
    request_id: RuntimeRequestId,
    operation: &'static str,
    status: &'static str,
) -> Option<UiEvent> {
    Some(UiEvent::OperationProgress {
        request_id: ui_request_id(request_id),
        operation: operation.to_owned(),
        status: status.to_owned(),
    })
}

fn translate_backup_completed(request_id: RuntimeRequestId, output_path: String, size_bytes: u64) -> Option<UiEvent> {
    Some(UiEvent::BackupCompleted {
        request_id: ui_request_id(request_id),
        output_path,
        size_bytes,
    })
}

fn translate_operation_completed(request_id: RuntimeRequestId, operation: &'static str) -> Option<UiEvent> {
    Some(UiEvent::OperationCompleted {
        request_id: ui_request_id(request_id),
        operation: operation.to_owned(),
    })
}

fn translate_connected(request_id: RuntimeRequestId, connection_id: String) -> Option<UiEvent> {
    Some(UiEvent::Connected {
        request_id: ui_request_id(request_id),
        connection_id,
    })
}

fn translate_query_completed(
    request_id: RuntimeRequestId,
    result: db_pro_core::domain::query::QueryResult,
) -> Option<UiEvent> {
    Some(UiEvent::QueryCompleted {
        request_id: ui_request_id(request_id),
        result: map_query_result(result),
    })
}

fn translate_explain_completed(request_id: RuntimeRequestId, plan: String) -> Option<UiEvent> {
    Some(UiEvent::ExplainCompleted {
        request_id: ui_request_id(request_id),
        plan,
    })
}

fn translate_table_data_loaded(
    request_id: RuntimeRequestId,
    result: db_pro_core::domain::query::QueryResult,
    total_rows: u64,
) -> Option<UiEvent> {
    Some(UiEvent::TableDataLoaded {
        request_id: ui_request_id(request_id),
        result: map_query_result(result),
        total_rows,
    })
}

fn translate_query_cancelled(request_id: RuntimeRequestId) -> Option<UiEvent> {
    Some(UiEvent::QueryCancelled {
        request_id: ui_request_id(request_id),
    })
}

fn translate_agent_completed(
    request_id: RuntimeRequestId,
    provider: String,
    message: db_pro_runtime::AgentDraft,
) -> Option<UiEvent> {
    Some(UiEvent::AgentCompleted {
        request_id: ui_request_id(request_id),
        provider,
        message: AgentMessage {
            role: AgentRole::Assistant,
            content: message.content,
            sql: message.sql,
            requires_confirmation: message.requires_confirmation,
        },
    })
}

fn translate_agent_failed(request_id: RuntimeRequestId, message: String) -> Option<UiEvent> {
    Some(UiEvent::AgentFailed {
        request_id: ui_request_id(request_id),
        message,
    })
}

fn translate_failed(request_id: RuntimeRequestId, message: String) -> Option<UiEvent> {
    Some(UiEvent::QueryFailed {
        request_id: ui_request_id(request_id),
        message,
    })
}

fn translate_connections_loaded(
    request_id: RuntimeRequestId,
    connections: Vec<db_pro_runtime::ConnectionSummary>,
) -> Option<UiEvent> {
    Some(UiEvent::ConnectionsLoaded {
        request_id: ui_request_id(request_id),
        connections: connections
            .into_iter()
            .map(|connection| UiConnectionSummary {
                id: connection.id,
                name: connection.name,
                host: connection.host,
                port: connection.port,
                database: connection.database,
                username: connection.username,
                driver: connection.driver,
                readonly: connection.readonly,
            })
            .collect(),
    })
}

fn translate_schema_loaded(request_id: RuntimeRequestId, schema: db_pro_runtime::SchemaSummary) -> Option<UiEvent> {
    Some(UiEvent::SchemaLoaded {
        request_id: ui_request_id(request_id),
        schema: map_schema_summary(schema),
    })
}

fn map_schema_summary(schema: db_pro_runtime::SchemaSummary) -> UiSchemaSummary {
    UiSchemaSummary {
        schemas: schema.schemas,
        tables: schema.tables,
        columns: schema.columns,
        table_details: schema.table_details.into_iter().map(map_table_summary).collect(),
        views: schema.views.into_iter().map(map_view_summary).collect(),
        triggers: schema.triggers.into_iter().map(map_trigger_summary).collect(),
        functions: schema.functions.into_iter().map(map_function_summary).collect(),
    }
}

fn map_table_summary(table: db_pro_runtime::TableSummary) -> UiTableSummary {
    UiTableSummary {
        schema: table.schema,
        name: table.name,
        row_count: table.row_count,
        columns: table.columns.into_iter().map(map_schema_column).collect(),
        foreign_keys: table.foreign_keys.into_iter().map(map_foreign_key).collect(),
    }
}

fn map_schema_column(column: db_pro_runtime::ColumnSummary) -> UiSchemaColumn {
    UiSchemaColumn {
        name: column.name,
        data_type: column.data_type,
        nullable: column.nullable,
        is_primary_key: column.is_primary_key,
    }
}

fn map_foreign_key(foreign_key: db_pro_runtime::ForeignKeySummary) -> UiSchemaForeignKey {
    UiSchemaForeignKey {
        name: foreign_key.name,
        from_columns: foreign_key.from_columns,
        to_schema: foreign_key.to_schema,
        to_table: foreign_key.to_table,
        to_columns: foreign_key.to_columns,
    }
}

fn map_view_summary(view: db_pro_runtime::ViewSummary) -> UiViewSummary {
    UiViewSummary {
        schema: view.schema,
        name: view.name,
        definition: view.definition,
    }
}

fn map_trigger_summary(trigger: db_pro_runtime::TriggerSummary) -> UiTriggerSummary {
    UiTriggerSummary {
        schema: trigger.schema,
        name: trigger.name,
        table_name: trigger.table_name,
        timing: trigger.timing,
        event: trigger.event,
        definition: trigger.definition,
        enabled: trigger.enabled,
    }
}

fn map_function_summary(function: db_pro_runtime::FunctionSummary) -> UiFunctionSummary {
    UiFunctionSummary {
        schema: function.schema,
        name: function.name,
        routine_type: function.routine_type,
        data_type: function.data_type,
        definition: function.definition,
    }
}

fn translate_query_folders_loaded(
    request_id: RuntimeRequestId,
    folders: Vec<db_pro_runtime::QueryFolderSummary>,
) -> Option<UiEvent> {
    Some(UiEvent::QueryFoldersLoaded {
        request_id: ui_request_id(request_id),
        folders: folders
            .into_iter()
            .map(|folder| UiQueryFolderSummary {
                id: folder.id,
                name: folder.name,
            })
            .collect(),
    })
}

fn translate_saved_queries_loaded(
    request_id: RuntimeRequestId,
    queries: Vec<db_pro_runtime::SavedQuerySummary>,
) -> Option<UiEvent> {
    Some(UiEvent::SavedQueriesLoaded {
        request_id: ui_request_id(request_id),
        queries: queries
            .into_iter()
            .map(|query| UiSavedQuerySummary {
                id: query.id,
                name: query.name,
                sql: query.sql,
                folder: query.folder,
            })
            .collect(),
    })
}
