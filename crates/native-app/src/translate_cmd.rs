//! UiCommand → RuntimeCommand mapping.
use super::super::*;
use super::translate_map::*;

pub(crate) fn driver_from_ui(ui_driver: UiDriver) -> db_pro_core::domain::connection::DriverType {
    match ui_driver {
        UiDriver::Postgres => db_pro_core::domain::connection::DriverType::Postgres,
        UiDriver::Sqlite => db_pro_core::domain::connection::DriverType::SQLite,
        UiDriver::Mysql => db_pro_core::domain::connection::DriverType::Mysql,
        UiDriver::SqlServer => db_pro_core::domain::connection::DriverType::SqlServer,
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
            ssh_profile_id: {
                let id = draft.ssh_profile_id.trim();
                if id.is_empty() {
                    None
                } else {
                    Some(id.to_owned())
                }
            },
            ssl_root_cert_path: {
                let p = draft.ssl_root_cert_path.trim();
                if p.is_empty() {
                    None
                } else {
                    Some(p.to_owned())
                }
            },
            ssl_client_cert_path: {
                let p = draft.ssl_client_cert_path.trim();
                if p.is_empty() {
                    None
                } else {
                    Some(p.to_owned())
                }
            },
            ssl_client_key_path: {
                let p = draft.ssl_client_key_path.trim();
                if p.is_empty() {
                    None
                } else {
                    Some(p.to_owned())
                }
            },
            query_timeout_ms: 30_000,
            max_rows: 500,
            color: None,
            tags: {
                let mut tags: Vec<String> = draft
                    .tags
                    .split(',')
                    .map(|s| s.trim().to_owned())
                    .filter(|s| !s.is_empty())
                    .collect();
                if draft.auth_kind.eq_ignore_ascii_case("ephemeral_token") {
                    if !tags.iter().any(|t| t.eq_ignore_ascii_case("auth:ephemeral-token")) {
                        tags.push("auth:ephemeral-token".into());
                    }
                } else {
                    tags.retain(|t| !t.eq_ignore_ascii_case("auth:ephemeral-token"));
                }
                if !draft.cloud_preset.is_empty() {
                    let cloud_tag = format!("cloud:{}", draft.cloud_preset);
                    if !tags.iter().any(|t| t == &cloud_tag) {
                        tags.push(cloud_tag);
                    }
                }
                tags
            },
            group: {
                let g = draft.group.trim();
                if g.is_empty() {
                    None
                } else {
                    Some(g.to_owned())
                }
            },
            favorite: draft.favorite,
            environment: match draft.environment.to_ascii_lowercase().as_str() {
                "staging" => db_pro_core::domain::connection::ConnectionEnvironment::Staging,
                "production" => db_pro_core::domain::connection::ConnectionEnvironment::Production,
                "custom" => db_pro_core::domain::connection::ConnectionEnvironment::Custom,
                _ => db_pro_core::domain::connection::ConnectionEnvironment::Development,
            },
            readonly: draft.readonly,
        },
        if driver == db_pro_core::domain::connection::DriverType::SQLite {
            String::new()
        } else if draft.auth_kind.eq_ignore_ascii_case("ephemeral_token") {
            // Token is for the current connect attempt only; ConnectionService will not persist it.
            draft.password
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
        | UiCommand::PickWorkspaceFolder { .. } => None,
        UiCommand::SaveAgentApiKey { .. } | UiCommand::ForgetAgentApiKey { .. } => translate_agent_key_command(command),
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
        | UiCommand::MonitoringMaintenance { .. }
        | UiCommand::MonitoringStatStatements { .. }
        | UiCommand::MonitoringResetStatStatements { .. }
        | UiCommand::AuditEventsLoad { .. }
        | UiCommand::ListPgSettings { .. }
        | UiCommand::SetPgSettingSession { .. }
        | UiCommand::ResetPgSettingSession { .. }
        | UiCommand::ListFdwInventory { .. }
        | UiCommand::CreateFdwServer { .. }
        | UiCommand::DropFdwServer { .. }
        | UiCommand::ListReplicationInventory { .. }
        | UiCommand::CreatePublicationAll { .. }
        | UiCommand::DropPublication { .. }
        | UiCommand::DropSubscription { .. }
        | UiCommand::ListEventTriggers { .. }
        | UiCommand::CreateEventTrigger { .. }
        | UiCommand::DropEventTrigger { .. }
        | UiCommand::AlterEventTrigger { .. }
        | UiCommand::ListUsers { .. }
        | UiCommand::CreateRole { .. }
        | UiCommand::DropRole { .. }
        | UiCommand::ListPrivileges { .. }
        | UiCommand::ListTableRls { .. }
        | UiCommand::AlterRole { .. }
        | UiCommand::UpdateRolePassword { .. }
        | UiCommand::ListMemberships { .. }
        | UiCommand::GrantMembership { .. }
        | UiCommand::RevokeMembership { .. }
        | UiCommand::GrantPrivilege { .. }
        | UiCommand::RevokePrivilege { .. }
        | UiCommand::DiffTableDataKeyed { .. }
        | UiCommand::CancelQuery { .. }
        | UiCommand::RequestSqlPrediction { .. }
        | UiCommand::CancelSqlPrediction { .. } => translate_execution_command(command),
    }
}

fn translate_agent_key_command(command: UiCommand) -> Option<RuntimeCommand> {
    match command {
        UiCommand::SaveAgentApiKey { request_id, api_key } => Some(RuntimeCommand::ConfigureAgent {
            request_id: runtime_request_id(request_id),
            api_key,
        }),
        UiCommand::ForgetAgentApiKey { request_id } => Some(RuntimeCommand::ForgetAgent {
            request_id: runtime_request_id(request_id),
        }),
        _ => None,
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
            analyze,
        } => Some(RuntimeCommand::ExplainQuery {
            request_id: runtime_request_id(request_id),
            connection_id,
            sql,
            analyze,
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
        UiCommand::MonitoringMaintenance {
            request_id,
            connection_id,
            schema,
            table,
            action,
            confirmed,
        } => Some(RuntimeCommand::MonitoringMaintenance {
            request_id: runtime_request_id(request_id),
            connection_id,
            schema,
            table,
            action,
            confirmed,
        }),
        UiCommand::MonitoringStatStatements {
            request_id,
            connection_id,
            sort,
            limit,
        } => Some(RuntimeCommand::MonitoringStatStatements {
            request_id: runtime_request_id(request_id),
            connection_id,
            sort,
            limit,
        }),
        UiCommand::MonitoringResetStatStatements {
            request_id,
            connection_id,
            confirmed,
        } => Some(RuntimeCommand::MonitoringResetStatStatements {
            request_id: runtime_request_id(request_id),
            connection_id,
            confirmed,
        }),
        UiCommand::AuditEventsLoad {
            request_id,
            connection_id,
            filter,
            limit,
        } => Some(RuntimeCommand::AuditEventsLoad {
            request_id: runtime_request_id(request_id),
            connection_id,
            filter,
            limit,
        }),
        UiCommand::ListPgSettings {
            request_id,
            connection_id,
        } => Some(RuntimeCommand::ListPgSettings {
            request_id: runtime_request_id(request_id),
            connection_id,
        }),
        UiCommand::SetPgSettingSession {
            request_id,
            connection_id,
            name,
            value,
        } => Some(RuntimeCommand::SetPgSettingSession {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
            value,
        }),
        UiCommand::ResetPgSettingSession {
            request_id,
            connection_id,
            name,
        } => Some(RuntimeCommand::ResetPgSettingSession {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
        }),
        UiCommand::ListFdwInventory {
            request_id,
            connection_id,
        } => Some(RuntimeCommand::ListFdwInventory {
            request_id: runtime_request_id(request_id),
            connection_id,
        }),
        UiCommand::CreateFdwServer {
            request_id,
            connection_id,
            name,
            fdw,
            host,
            dbname,
            port,
            confirmed,
        } => Some(RuntimeCommand::CreateFdwServer {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
            fdw,
            host,
            dbname,
            port,
            confirmed,
        }),
        UiCommand::DropFdwServer {
            request_id,
            connection_id,
            name,
            cascade,
            confirmed,
        } => Some(RuntimeCommand::DropFdwServer {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
            cascade,
            confirmed,
        }),
        UiCommand::ListReplicationInventory {
            request_id,
            connection_id,
        } => Some(RuntimeCommand::ListReplicationInventory {
            request_id: runtime_request_id(request_id),
            connection_id,
        }),
        UiCommand::CreatePublicationAll {
            request_id,
            connection_id,
            name,
            confirmed,
        } => Some(RuntimeCommand::CreatePublicationAll {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
            confirmed,
        }),
        UiCommand::DropPublication {
            request_id,
            connection_id,
            name,
            confirmed,
        } => Some(RuntimeCommand::DropPublication {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
            confirmed,
        }),
        UiCommand::DropSubscription {
            request_id,
            connection_id,
            name,
            confirmed,
        } => Some(RuntimeCommand::DropSubscription {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
            confirmed,
        }),
        UiCommand::ListEventTriggers {
            request_id,
            connection_id,
        } => Some(RuntimeCommand::ListEventTriggers {
            request_id: runtime_request_id(request_id),
            connection_id,
        }),
        UiCommand::CreateEventTrigger {
            request_id,
            connection_id,
            name,
            event,
            function_ref,
            tags_csv,
            confirmed,
        } => Some(RuntimeCommand::CreateEventTrigger {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
            event,
            function_ref,
            tags_csv,
            confirmed,
        }),
        UiCommand::DropEventTrigger {
            request_id,
            connection_id,
            name,
            confirmed,
        } => Some(RuntimeCommand::DropEventTrigger {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
            confirmed,
        }),
        UiCommand::AlterEventTrigger {
            request_id,
            connection_id,
            name,
            mode,
            confirmed,
        } => Some(RuntimeCommand::AlterEventTrigger {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
            mode,
            confirmed,
        }),
        UiCommand::ListUsers {
            request_id,
            connection_id,
        } => Some(RuntimeCommand::ListUsers {
            request_id: runtime_request_id(request_id),
            connection_id,
        }),
        UiCommand::CreateRole {
            request_id,
            connection_id,
            name,
            login,
        } => Some(RuntimeCommand::CreateRole {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
            login,
        }),
        UiCommand::DropRole {
            request_id,
            connection_id,
            name,
        } => Some(RuntimeCommand::DropRole {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
        }),
        UiCommand::ListPrivileges {
            request_id,
            connection_id,
            role_name,
        } => Some(RuntimeCommand::ListPrivileges {
            request_id: runtime_request_id(request_id),
            connection_id,
            role_name,
        }),
        UiCommand::ListTableRls {
            request_id,
            connection_id,
            schema,
            table,
        } => Some(RuntimeCommand::ListTableRls {
            request_id: runtime_request_id(request_id),
            connection_id,
            schema,
            table,
        }),
        UiCommand::AlterRole {
            request_id,
            connection_id,
            name,
            attributes,
        } => Some(RuntimeCommand::AlterRole {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
            attributes,
        }),
        UiCommand::UpdateRolePassword {
            request_id,
            connection_id,
            name,
            password,
        } => Some(RuntimeCommand::UpdateRolePassword {
            request_id: runtime_request_id(request_id),
            connection_id,
            name,
            password,
        }),
        UiCommand::ListMemberships {
            request_id,
            connection_id,
            member,
        } => Some(RuntimeCommand::ListMemberships {
            request_id: runtime_request_id(request_id),
            connection_id,
            member,
        }),
        UiCommand::GrantMembership {
            request_id,
            connection_id,
            role,
            member,
        } => Some(RuntimeCommand::GrantMembership {
            request_id: runtime_request_id(request_id),
            connection_id,
            role,
            member,
        }),
        UiCommand::RevokeMembership {
            request_id,
            connection_id,
            role,
            member,
        } => Some(RuntimeCommand::RevokeMembership {
            request_id: runtime_request_id(request_id),
            connection_id,
            role,
            member,
        }),
        UiCommand::GrantPrivilege {
            request_id,
            connection_id,
            role_name,
            object_kind,
            schema,
            object_name,
            privilege,
        } => Some(RuntimeCommand::GrantPrivilege {
            request_id: runtime_request_id(request_id),
            connection_id,
            role_name,
            object_kind,
            schema,
            object_name,
            privilege,
        }),
        UiCommand::RevokePrivilege {
            request_id,
            connection_id,
            role_name,
            object_kind,
            schema,
            object_name,
            privilege,
        } => Some(RuntimeCommand::RevokePrivilege {
            request_id: runtime_request_id(request_id),
            connection_id,
            role_name,
            object_kind,
            schema,
            object_name,
            privilege,
        }),
        UiCommand::DiffTableDataKeyed {
            request_id,
            source_id,
            target_id,
            schema,
            table,
            key_columns,
            sample_limit,
        } => Some(RuntimeCommand::DiffTableDataKeyed {
            request_id: runtime_request_id(request_id),
            source_id,
            target_id,
            schema,
            table,
            key_columns,
            sample_limit,
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
