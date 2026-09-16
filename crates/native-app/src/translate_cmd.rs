//! UiCommand → RuntimeCommand mapping.
use super::super::*;
use super::translate_map::*;

pub(crate) fn driver_from_ui(ui_driver: UiDriver) -> db_pro_core::domain::connection::DriverType {
    match ui_driver {
        UiDriver::Postgres => db_pro_core::domain::connection::DriverType::Postgres,
        UiDriver::Sqlite => db_pro_core::domain::connection::DriverType::SQLite,
        UiDriver::Mysql => db_pro_core::domain::connection::DriverType::Mysql,
    }
}

pub(crate) fn draft_to_domain(
    draft: UiConnectionDraft,
) -> Option<(db_pro_core::domain::connection::ConnectionConfig, String)> {
    let port = draft.port.parse::<u16>().ok()?;
    let driver = driver_from_ui(draft.driver);
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
        if driver == db_pro_core::domain::connection::DriverType::SQLite {
            String::new()
        } else {
            draft.password
        },
    ))
}

/// Reverse of the `SslMode` arm in [`draft_to_domain`], used to prefill the
/// connection dialog from a stored connection.
pub(crate) fn ui_ssl_mode(mode: db_pro_core::domain::connection::SslMode) -> UiSslMode {
    match mode {
        db_pro_core::domain::connection::SslMode::Disable => UiSslMode::Disable,
        db_pro_core::domain::connection::SslMode::Require => UiSslMode::Require,
        db_pro_core::domain::connection::SslMode::VerifyCa => UiSslMode::VerifyCa,
        db_pro_core::domain::connection::SslMode::VerifyFull => UiSslMode::VerifyFull,
    }
}

pub(crate) fn translate_command(command: UiCommand) -> Option<RuntimeCommand> {
    match &command {
        UiCommand::OpenQuery
        | UiCommand::PickSqliteFile { .. }
        | UiCommand::PickSshPrivateKey { .. }
        | UiCommand::PickBackupFile { .. }
        | UiCommand::PickRestoreFile { .. }
        | UiCommand::PickWorkspaceFolder { .. }
        // Handled in the native command thread (keyring + ConfigureAgent):
        | UiCommand::SaveAgentApiKey { .. }
        => None,
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
        | UiCommand::InsertTableRow { .. }
        | UiCommand::ApplyTableChanges { .. } => translate_table_command(command),
        UiCommand::RunAgent { .. }
        | UiCommand::ExecuteAgentTool { .. }
        | UiCommand::StartAgentRun { .. }
        | UiCommand::ContinueAgentRun { .. }
        | UiCommand::CancelAgentRun { .. }
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
        | UiCommand::RunQueryMulti { .. }
        | UiCommand::ExplainQuery { .. }
        | UiCommand::Backup { .. }
        | UiCommand::Restore { .. }
        | UiCommand::MonitoringSnapshot { .. }
        | UiCommand::MonitoringCancelBackend { .. }
        | UiCommand::MonitoringTerminateBackend { .. }
        | UiCommand::CancelQuery { .. }
        | UiCommand::RequestSqlPrediction { .. }
        | UiCommand::CancelSqlPrediction { .. } => translate_execution_command(command),
    }
}

pub(crate) fn translate_query_command(command: UiCommand) -> Option<RuntimeCommand> {
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
            saved_query_id,
            name,
            sql,
            folder,
        } => Some(RuntimeCommand::SaveQuery {
            request_id: runtime_request_id(request_id),
            connection_id,
            saved_query_id,
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

pub(crate) fn translate_table_command(command: UiCommand) -> Option<RuntimeCommand> {
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
            data_type,
            value,
            pk_columns,
            pk_values,
        } => {
            let value = ui_cell_to_domain_typed(value, &data_type)?;
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
        UiCommand::ApplyTableChanges {
            request_id,
            connection_id,
            schema,
            table,
            changes,
        } => Some(RuntimeCommand::ApplyTableChanges {
            request_id: runtime_request_id(request_id),
            connection_id,
            schema,
            table,
            changes: changes
                .into_iter()
                .map(map_table_mutation)
                .collect::<Option<Vec<_>>>()?,
        }),
        _ => None,
    }
}

pub(crate) fn map_table_mutation(mutation: UiTableMutation) -> Option<db_pro_core::application::TableDataMutation> {
    match mutation {
        UiTableMutation::Update {
            columns,
            data_types,
            values,
            pk_columns,
            pk_values,
        } => {
            if columns.len() != data_types.len() || columns.len() != values.len() {
                return None;
            }
            Some(db_pro_core::application::TableDataMutation::Update {
                columns,
                values: values
                    .into_iter()
                    .zip(data_types)
                    .map(|(value, data_type)| ui_cell_to_domain_typed(value, &data_type))
                    .collect::<Option<Vec<_>>>()?,
                pk_columns,
                pk_values: pk_values
                    .into_iter()
                    .map(ui_cell_to_domain)
                    .collect::<Option<Vec<_>>>()?,
            })
        }
        UiTableMutation::Delete { pk_columns, pk_values } => {
            Some(db_pro_core::application::TableDataMutation::Delete {
                pk_columns,
                pk_values: pk_values
                    .into_iter()
                    .map(ui_cell_to_domain)
                    .collect::<Option<Vec<_>>>()?,
            })
        }
        UiTableMutation::Insert { columns, values } => Some(db_pro_core::application::TableDataMutation::Insert {
            columns,
            values: values.into_iter().map(ui_cell_to_domain).collect::<Option<Vec<_>>>()?,
        }),
    }
}

pub(crate) fn translate_agent_command(command: UiCommand) -> Option<RuntimeCommand> {
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
        UiCommand::ExecuteAgentTool {
            request_id,
            request,
            context,
        } => Some(RuntimeCommand::ExecuteAgentTool {
            request_id: runtime_request_id(request_id),
            request,
            context,
        }),
        UiCommand::StartAgentRun {
            request_id,
            prompt,
            session,
            document,
            mode,
            allow_read_only_auto_run,
            context,
        } => Some(RuntimeCommand::StartAgentWorkflow {
            request_id: runtime_request_id(request_id),
            prompt,
            session,
            document,
            mode,
            allow_read_only_auto_run,
            context,
        }),
        UiCommand::ContinueAgentRun {
            request_id,
            run_id,
            approved,
            current_document,
            applied_patch,
        } => Some(RuntimeCommand::ContinueAgentWorkflow {
            request_id: runtime_request_id(request_id),
            run_id,
            approved,
            current_document,
            applied_patch,
        }),
        UiCommand::CancelAgentRun { request_id, run_id } => Some(RuntimeCommand::CancelAgentWorkflow {
            request_id: runtime_request_id(request_id),
            run_id,
        }),
        _ => None,
    }
}

pub(crate) fn translate_schema_command(command: UiCommand) -> Option<RuntimeCommand> {
    if let Some(command) = translate_agent_command(command.clone()) {
        return Some(command);
    }
    match command {
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
            filters,
            sorts,
        } => Some(RuntimeCommand::LoadTableData {
            request_id: runtime_request_id(request_id),
            connection_id,
            schema,
            table,
            limit,
            offset,
            filters: filters.into_iter().map(map_table_data_filter).collect(),
            sorts: sorts.into_iter().map(map_table_data_sort).collect(),
        }),
        _ => None,
    }
}

pub(crate) fn translate_connection_command(command: UiCommand) -> Option<RuntimeCommand> {
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
            let password_opt = if password.is_empty() { None } else { Some(password) };
            Some(RuntimeCommand::UpdateConnection {
                request_id: runtime_request_id(request_id),
                connection_id,
                config,
                password: password_opt,
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

pub(crate) fn translate_execution_command(command: UiCommand) -> Option<RuntimeCommand> {
    match command {
        UiCommand::RunQuery {
            request_id,
            connection_id,
            sql,
            params,
        } => Some(RuntimeCommand::ExecuteQuery {
            request_id: runtime_request_id(request_id),
            connection_id,
            sql,
            params,
        }),
        UiCommand::RunQueryMulti {
            request_id,
            connection_id,
            sql,
        } => Some(RuntimeCommand::ExecuteQueryMulti {
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
        UiCommand::MonitoringSnapshot {
            request_id,
            connection_id,
        } => Some(RuntimeCommand::MonitoringSnapshot {
            request_id: runtime_request_id(request_id),
            connection_id,
        }),
        UiCommand::MonitoringCancelBackend {
            request_id,
            connection_id,
            backend_id,
        } => Some(RuntimeCommand::MonitoringCancelBackend {
            request_id: runtime_request_id(request_id),
            connection_id,
            backend_id,
        }),
        UiCommand::MonitoringTerminateBackend {
            request_id,
            connection_id,
            backend_id,
        } => Some(RuntimeCommand::MonitoringTerminateBackend {
            request_id: runtime_request_id(request_id),
            connection_id,
            backend_id,
        }),
        UiCommand::CancelQuery { request_id } => Some(RuntimeCommand::CancelQuery {
            request_id: runtime_request_id(request_id),
        }),
        UiCommand::RequestSqlPrediction {
            request_id,
            document_id,
            document_version,
            anchor,
            replacement_range,
            context,
        } => Some(RuntimeCommand::RequestSqlPrediction {
            request_id: runtime_request_id(request_id),
            document_id,
            document_version,
            anchor,
            replacement_range,
            context: db_pro_runtime::SqlPredictionContext {
                sql_before_cursor: context.sql_before_cursor,
                sql_after_cursor: context.sql_after_cursor,
                current_statement: context.current_statement,
                active_schema: context.active_schema,
                dialect: context.dialect,
                referenced_tables: context.referenced_tables,
                table_aliases: context.table_aliases,
                relevant_columns: context.relevant_columns,
                fk_neighbors: context.fk_neighbors,
                cte_names: context.cte_names,
            },
        }),
        UiCommand::CancelSqlPrediction { request_id } => Some(RuntimeCommand::CancelSqlPrediction {
            request_id: runtime_request_id(request_id),
        }),
        _ => None,
    }
}
