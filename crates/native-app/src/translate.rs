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
            request_id: RuntimeRequestId(request_id.0),
            connection_id,
        }),
        UiCommand::ListSavedQueries {
            request_id,
            connection_id,
        } => Some(RuntimeCommand::ListSavedQueries {
            request_id: RuntimeRequestId(request_id.0),
            connection_id,
        }),
        UiCommand::SaveQuery {
            request_id,
            connection_id,
            name,
            sql,
            folder,
        } => Some(RuntimeCommand::SaveQuery {
            request_id: RuntimeRequestId(request_id.0),
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
            request_id: RuntimeRequestId(request_id.0),
            connection_id,
            name,
        }),
        UiCommand::RenameSavedQuery { request_id, id, name } => Some(RuntimeCommand::RenameSavedQuery {
            request_id: RuntimeRequestId(request_id.0),
            id,
            name,
        }),
        UiCommand::DeleteSavedQuery { request_id, id } => Some(RuntimeCommand::DeleteSavedQuery {
            request_id: RuntimeRequestId(request_id.0),
            id,
        }),
        UiCommand::DeleteQueryFolder { request_id, id } => Some(RuntimeCommand::DeleteQueryFolder {
            request_id: RuntimeRequestId(request_id.0),
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
            request_id: RuntimeRequestId(request_id.0),
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
                request_id: RuntimeRequestId(request_id.0),
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
                request_id: RuntimeRequestId(request_id.0),
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
                request_id: RuntimeRequestId(request_id.0),
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
            request_id: RuntimeRequestId(request_id.0),
            prompt,
            context: db_pro_runtime::AgentContext {
                connection_name: context.connection_name,
                driver: context.driver,
                tables: context.tables,
                columns: context.columns,
            },
        }),
        UiCommand::IntrospectSchema {
            request_id,
            connection_id,
            force_refresh,
        } => Some(RuntimeCommand::IntrospectSchema {
            request_id: RuntimeRequestId(request_id.0),
            connection_id,
            force_refresh,
        }),
        UiCommand::LoadTableInfo {
            request_id,
            connection_id,
            schema,
            table,
        } => Some(RuntimeCommand::LoadTableInfo {
            request_id: RuntimeRequestId(request_id.0),
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
            request_id: RuntimeRequestId(request_id.0),
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
            request_id: RuntimeRequestId(request_id.0),
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
            request_id: RuntimeRequestId(request_id.0),
        }),
        UiCommand::CreateConnection { request_id, draft } => {
            let (config, password) = draft_to_domain(draft)?;
            Some(RuntimeCommand::CreateConnection {
                request_id: RuntimeRequestId(request_id.0),
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
                request_id: RuntimeRequestId(request_id.0),
                connection_id,
                config,
                password: Some(password),
            })
        }
        UiCommand::TestConnection { request_id, draft } => {
            let (config, password) = draft_to_domain(draft)?;
            Some(RuntimeCommand::TestConnection {
                request_id: RuntimeRequestId(request_id.0),
                config,
                password,
            })
        }
        UiCommand::DeleteConnection {
            request_id,
            connection_id,
        } => Some(RuntimeCommand::DeleteConnection {
            request_id: RuntimeRequestId(request_id.0),
            connection_id,
        }),
        UiCommand::Connect {
            request_id,
            connection_id,
        } => Some(RuntimeCommand::Connect {
            request_id: RuntimeRequestId(request_id.0),
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
            request_id: RuntimeRequestId(request_id.0),
            connection_id,
            sql,
        }),
        UiCommand::Backup {
            request_id,
            connection_id,
            output_path,
            custom_format,
        } => Some(RuntimeCommand::Backup {
            request_id: RuntimeRequestId(request_id.0),
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
            request_id: RuntimeRequestId(request_id.0),
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
            request_id: RuntimeRequestId(request_id.0),
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
        RuntimeEvent::TableInfoLoaded { request_id, table_info } => Some(UiEvent::TableInfoLoaded {
            request_id: db_pro_ui::RequestId(request_id.0),
            table_info: map_table_info(table_info),
        }),
        RuntimeEvent::TableDdlLoaded { request_id, sql } => Some(UiEvent::TableDdlLoaded {
            request_id: db_pro_ui::RequestId(request_id.0),
            sql,
        }),
        RuntimeEvent::DdlCompleted {
            request_id,
            affected_rows,
        } => Some(UiEvent::DdlCompleted {
            request_id: db_pro_ui::RequestId(request_id.0),
            affected_rows,
        }),
        RuntimeEvent::QueryFoldersLoaded { request_id, folders } => translate_query_folders_loaded(request_id, folders),
        RuntimeEvent::SavedQueriesLoaded { request_id, queries } => translate_saved_queries_loaded(request_id, queries),
        RuntimeEvent::OperationProgress {
            request_id,
            operation,
            status,
        } => Some(UiEvent::OperationProgress {
            request_id: db_pro_ui::RequestId(request_id.0),
            operation: operation.to_owned(),
            status: status.to_owned(),
        }),
        RuntimeEvent::BackupCompleted {
            request_id,
            output_path,
            size_bytes,
        } => Some(UiEvent::BackupCompleted {
            request_id: db_pro_ui::RequestId(request_id.0),
            output_path,
            size_bytes,
        }),
        RuntimeEvent::OperationCompleted { request_id, operation } => Some(UiEvent::OperationCompleted {
            request_id: db_pro_ui::RequestId(request_id.0),
            operation: operation.to_owned(),
        }),
        RuntimeEvent::Connected {
            request_id,
            connection_id,
        } => Some(UiEvent::Connected {
            request_id: db_pro_ui::RequestId(request_id.0),
            connection_id,
        }),
        RuntimeEvent::QueryCompleted { request_id, result } => Some(UiEvent::QueryCompleted {
            request_id: db_pro_ui::RequestId(request_id.0),
            result: map_query_result(result),
        }),
        RuntimeEvent::TableDataLoaded {
            request_id,
            result,
            total_rows,
        } => Some(UiEvent::TableDataLoaded {
            request_id: db_pro_ui::RequestId(request_id.0),
            result: map_query_result(result),
            total_rows,
        }),
        RuntimeEvent::QueryCancelled { request_id } => Some(UiEvent::QueryCancelled {
            request_id: db_pro_ui::RequestId(request_id.0),
        }),
        RuntimeEvent::AgentCompleted { request_id, message } => Some(UiEvent::AgentCompleted {
            request_id: db_pro_ui::RequestId(request_id.0),
            provider: "Codex".to_owned(),
            message: AgentMessage {
                role: AgentRole::Assistant,
                content: message.content,
                sql: message.sql,
                requires_confirmation: message.requires_confirmation,
            },
        }),
        RuntimeEvent::AgentFailed { request_id, message } => Some(UiEvent::AgentFailed {
            request_id: db_pro_ui::RequestId(request_id.0),
            message,
        }),
        RuntimeEvent::Failed { request_id, message } => Some(UiEvent::QueryFailed {
            request_id: db_pro_ui::RequestId(request_id.0),
            message,
        }),
    }
}

fn translate_connections_loaded(
    request_id: RuntimeRequestId,
    connections: Vec<db_pro_runtime::ConnectionSummary>,
) -> Option<UiEvent> {
    Some(UiEvent::ConnectionsLoaded {
        request_id: db_pro_ui::RequestId(request_id.0),
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
        request_id: db_pro_ui::RequestId(request_id.0),
        schema: UiSchemaSummary {
            tables: schema.tables,
            columns: schema.columns,
            table_details: schema
                .table_details
                .into_iter()
                .map(|table| UiTableSummary {
                    schema: table.schema,
                    name: table.name,
                    row_count: table.row_count,
                    columns: table
                        .columns
                        .into_iter()
                        .map(|column| UiSchemaColumn {
                            name: column.name,
                            data_type: column.data_type,
                            nullable: column.nullable,
                            is_primary_key: column.is_primary_key,
                        })
                        .collect(),
                    foreign_keys: table
                        .foreign_keys
                        .into_iter()
                        .map(|foreign_key| UiSchemaForeignKey {
                            name: foreign_key.name,
                            from_columns: foreign_key.from_columns,
                            to_schema: foreign_key.to_schema,
                            to_table: foreign_key.to_table,
                            to_columns: foreign_key.to_columns,
                        })
                        .collect(),
                })
                .collect(),
            views: schema
                .views
                .into_iter()
                .map(|view| UiViewSummary {
                    schema: view.schema,
                    name: view.name,
                    definition: view.definition,
                })
                .collect(),
            triggers: schema
                .triggers
                .into_iter()
                .map(|trigger| UiTriggerSummary {
                    schema: trigger.schema,
                    name: trigger.name,
                    table_name: trigger.table_name,
                    timing: trigger.timing,
                    event: trigger.event,
                    definition: trigger.definition,
                    enabled: trigger.enabled,
                })
                .collect(),
            functions: schema
                .functions
                .into_iter()
                .map(|function| UiFunctionSummary {
                    schema: function.schema,
                    name: function.name,
                    routine_type: function.routine_type,
                    data_type: function.data_type,
                    definition: function.definition,
                })
                .collect(),
        },
    })
}

fn translate_query_folders_loaded(
    request_id: RuntimeRequestId,
    folders: Vec<db_pro_runtime::QueryFolderSummary>,
) -> Option<UiEvent> {
    Some(UiEvent::QueryFoldersLoaded {
        request_id: db_pro_ui::RequestId(request_id.0),
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
        request_id: db_pro_ui::RequestId(request_id.0),
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
