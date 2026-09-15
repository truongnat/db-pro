use super::*;

fn driver_from_ui(ui_driver: UiDriver) -> db_pro_core::domain::connection::DriverType {
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
fn ui_ssl_mode(mode: db_pro_core::domain::connection::SslMode) -> UiSslMode {
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
        | UiCommand::CancelQuery { .. }
        | UiCommand::RequestSqlPrediction { .. }
        | UiCommand::CancelSqlPrediction { .. } => translate_execution_command(command),
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

fn map_table_mutation(mutation: UiTableMutation) -> Option<db_pro_core::application::TableDataMutation> {
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

fn map_table_data_filter(filter: UiTableDataFilter) -> TableFilter {
    let value = parse_filter_value(&filter.data_type, &filter.value);
    let (op, value) = match filter.operator {
        UiTableFilterOperator::Equals => (FilterOp::Eq, value),
        UiTableFilterOperator::NotEquals => (FilterOp::Neq, value),
        UiTableFilterOperator::Contains => (FilterOp::Like, CellValue::Text(format!("%{}%", filter.value))),
        UiTableFilterOperator::StartsWith => (FilterOp::Like, CellValue::Text(format!("{}%", filter.value))),
        UiTableFilterOperator::EndsWith => (FilterOp::Like, CellValue::Text(format!("%{}", filter.value))),
        UiTableFilterOperator::GreaterThan => (FilterOp::Gt, value),
        UiTableFilterOperator::GreaterThanOrEqual => (FilterOp::Gte, value),
        UiTableFilterOperator::LessThan => (FilterOp::Lt, value),
        UiTableFilterOperator::LessThanOrEqual => (FilterOp::Lte, value),
        UiTableFilterOperator::IsNull => (FilterOp::IsNull, CellValue::Null),
        UiTableFilterOperator::IsNotNull => (FilterOp::IsNotNull, CellValue::Null),
    };
    TableFilter {
        column: filter.column,
        op,
        value,
    }
}

fn parse_filter_value(data_type: &str, value: &str) -> CellValue {
    let normalized = data_type.to_ascii_lowercase();
    if normalized.contains("bool") {
        return value
            .parse::<bool>()
            .map(CellValue::Bool)
            .unwrap_or_else(|_| CellValue::Text(value.to_owned()));
    }
    if normalized.contains("int") || normalized.contains("serial") {
        return value
            .parse::<i64>()
            .map(CellValue::Int64)
            .unwrap_or_else(|_| CellValue::Text(value.to_owned()));
    }
    if normalized.contains("real") || normalized.contains("float") || normalized.contains("double") {
        return value
            .parse::<f64>()
            .map(CellValue::Float64)
            .unwrap_or_else(|_| CellValue::Text(value.to_owned()));
    }
    if normalized == "numeric"
        || normalized.starts_with("numeric(")
        || normalized == "decimal"
        || normalized.starts_with("decimal(")
    {
        return CellValue::Decimal(value.to_owned());
    }
    if normalized.contains("uuid") {
        return uuid::Uuid::parse_str(value)
            .map(|uuid| CellValue::Uuid(uuid.to_string()))
            .unwrap_or_else(|_| CellValue::Text(value.to_owned()));
    }
    if normalized.contains("timestamptz") || normalized.contains("timestamp with time zone") {
        return chrono::DateTime::parse_from_rfc3339(value)
            .map(|_| CellValue::DateTime(value.to_owned()))
            .unwrap_or_else(|_| CellValue::Text(value.to_owned()));
    }
    if normalized.contains("timestamp") {
        return CellValue::DateTime(value.to_owned());
    }
    if normalized == "date" {
        return CellValue::Date(value.to_owned());
    }
    if normalized.starts_with("time") {
        return CellValue::Time(value.to_owned());
    }
    CellValue::Text(value.to_owned())
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
        CellValue::Decimal(value) => UiCell::Number(value),
        CellValue::Text(value)
        | CellValue::Uuid(value)
        | CellValue::DateTime(value)
        | CellValue::Timestamp(value)
        | CellValue::TimestampTz(value)
        | CellValue::TimeTz(value)
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
            .ok()
            .or(Some(CellValue::Text(value))),
        UiCell::Text(value) => {
            if uuid::Uuid::parse_str(&value).is_ok() {
                Some(CellValue::Uuid(value))
            } else if chrono::DateTime::parse_from_rfc3339(&value).is_ok()
                || chrono::DateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S%z").is_ok()
                || chrono::NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S%.f").is_ok()
                || chrono::NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S").is_ok()
                || chrono::NaiveDateTime::parse_from_str(&value, "%Y-%m-%dT%H:%M:%S%.f").is_ok()
                || chrono::NaiveDateTime::parse_from_str(&value, "%Y-%m-%dT%H:%M:%S").is_ok()
            {
                Some(CellValue::DateTime(value))
            } else if chrono::NaiveDate::parse_from_str(&value, "%Y-%m-%d").is_ok() {
                Some(CellValue::Date(value))
            } else if chrono::NaiveTime::parse_from_str(&value, "%H:%M:%S").is_ok()
                || chrono::NaiveTime::parse_from_str(&value, "%H:%M:%S%.f").is_ok()
            {
                Some(CellValue::Time(value))
            } else {
                Some(CellValue::Text(value))
            }
        }
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

fn ui_cell_to_domain_typed(cell: UiCell, data_type: &str) -> Option<CellValue> {
    let normalized = data_type.to_ascii_lowercase();
    match cell {
        UiCell::Null => Some(CellValue::Null),
        UiCell::Number(value)
            if normalized == "numeric"
                || normalized.starts_with("numeric(")
                || normalized == "decimal"
                || normalized.starts_with("decimal(") =>
        {
            Some(CellValue::Decimal(value))
        }
        UiCell::Number(value)
            if normalized.contains("real") || normalized.contains("float") || normalized.contains("double") =>
        {
            value.parse::<f64>().ok().map(CellValue::Float64)
        }
        UiCell::Number(value) => value.parse::<i64>().ok().map(CellValue::Int64),
        UiCell::Boolean(value) => Some(CellValue::Bool(value)),
        UiCell::Text(value) if normalized.contains("uuid") => uuid::Uuid::parse_str(&value)
            .ok()
            .map(|uuid| CellValue::Uuid(uuid.to_string())),
        UiCell::Text(value)
            if normalized.contains("timestamptz") || normalized.contains("timestamp with time zone") =>
        {
            chrono::DateTime::parse_from_rfc3339(&value)
                .ok()
                .map(|_| CellValue::DateTime(value))
        }
        UiCell::Text(value) if normalized.contains("timestamp") => Some(CellValue::DateTime(value)),
        UiCell::Text(value) if normalized == "date" => Some(CellValue::Date(value)),
        UiCell::Text(value) if normalized.starts_with("time") => Some(CellValue::Time(value)),
        other => ui_cell_to_domain(other),
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
                ordinal: column.ordinal,
                nullable: column.nullable,
                default: column.default,
                is_primary_key: column.is_primary_key,
                is_unique: column.is_unique,
                is_identity: column.is_identity,
                is_generated: column.is_generated,
                collation: column.collation,
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
                method: index.method,
                primary: index.primary,
                include_columns: index.include_columns,
                predicate: index.predicate,
                definition: index.definition,
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
                on_update: foreign_key.on_update,
                on_delete: foreign_key.on_delete,
                match_option: foreign_key.match_option,
                deferrable: foreign_key.deferrable,
                initially_deferred: foreign_key.initially_deferred,
            })
            .collect(),
        check_constraints: info
            .check_constraints
            .into_iter()
            .map(|c| UiCheckConstraint {
                name: c.name,
                definition: c.definition,
            })
            .collect(),
        dependencies: info
            .dependencies
            .into_iter()
            .map(|d| UiTableDependency {
                name: d.name,
                schema: d.schema,
                kind: match d.kind {
                    db_pro_core::domain::schema::DependencyKind::Table => UiDependencyKind::Table,
                    db_pro_core::domain::schema::DependencyKind::View => UiDependencyKind::View,
                    db_pro_core::domain::schema::DependencyKind::ForeignKey => UiDependencyKind::ForeignKey,
                    db_pro_core::domain::schema::DependencyKind::Trigger => UiDependencyKind::Trigger,
                    db_pro_core::domain::schema::DependencyKind::Function => UiDependencyKind::Function,
                    db_pro_core::domain::schema::DependencyKind::Sequence => UiDependencyKind::Sequence,
                },
                direction: match d.direction {
                    db_pro_core::domain::schema::DependencyDirection::DependsOn => UiDependencyDirection::DependsOn,
                    db_pro_core::domain::schema::DependencyDirection::DependedBy => UiDependencyDirection::DependedBy,
                },
                details: d.details,
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
        RuntimeEvent::TableChangesFailed {
            request_id,
            code,
            message,
            statement_index,
            rolled_back,
        } => Some(UiEvent::TableChangesFailed {
            request_id: ui_request_id(request_id),
            code,
            message,
            statement_index,
            rolled_back,
        }),
        RuntimeEvent::Connected {
            request_id,
            connection_id,
        } => translate_connected(request_id, connection_id),
        RuntimeEvent::QueryCompleted { request_id, result } => translate_query_completed(request_id, result),
        RuntimeEvent::QueryMultiCompleted { request_id, output } => Some(UiEvent::QueryMultiCompleted {
            request_id: ui_request_id(request_id),
            output: map_multi_query_output(output),
        }),
        RuntimeEvent::QuerySaved { request_id, query } => Some(UiEvent::QuerySaved {
            request_id: ui_request_id(request_id),
            query: UiSavedQuerySummary {
                id: query.id,
                name: query.name,
                sql: query.sql,
                folder: query.folder,
            },
        }),
        RuntimeEvent::ExplainCompleted { request_id, plan } => translate_explain_completed(request_id, plan),
        RuntimeEvent::TableDataLoaded {
            request_id,
            result,
            total_rows,
        } => translate_table_data_loaded(request_id, result, total_rows),
        RuntimeEvent::QueryCancelled { request_id } => translate_query_cancelled(request_id),
        RuntimeEvent::QueryFailedDetailed { request_id, error } => Some(UiEvent::QueryFailedDetailed {
            request_id: ui_request_id(request_id),
            code: error.code,
            message: error.message,
            position: error.position,
        }),
        RuntimeEvent::AgentCompleted {
            request_id,
            provider,
            message,
        } => translate_agent_completed(request_id, provider, message),
        RuntimeEvent::AgentProviderReady { provider, detail } => Some(UiEvent::AgentProviderReady { provider, detail }),
        RuntimeEvent::AgentFailed { request_id, message } => translate_agent_failed(request_id, message),
        RuntimeEvent::AgentToolCompleted {
            request_id,
            session_id,
            run_id,
            document_id,
            result,
        } => Some(UiEvent::AgentToolCompleted {
            request_id: ui_request_id(request_id),
            session_id,
            run_id,
            document_id,
            result,
        }),
        RuntimeEvent::AgentToolFailed {
            request_id,
            session_id,
            run_id,
            document_id,
            error,
        } => Some(UiEvent::AgentToolFailed {
            request_id: ui_request_id(request_id),
            session_id,
            run_id,
            document_id,
            error,
        }),
        RuntimeEvent::AgentWorkflow { request_id, event } => Some(UiEvent::AgentWorkflow {
            request_id: ui_request_id(request_id),
            event,
        }),
        RuntimeEvent::AgentConfigured {
            request_id,
            provider,
            detail,
        } => Some(UiEvent::AgentConfigured {
            request_id: ui_request_id(request_id),
            provider,
            detail,
        }),
        RuntimeEvent::SqlPredictionReady {
            request_id,
            document_id,
            document_version,
            anchor,
            replacement_range,
            prediction,
        } => Some(UiEvent::SqlPredictionReady {
            request_id: ui_request_id(request_id),
            document_id,
            document_version,
            anchor,
            replacement_range,
            prediction,
        }),
        RuntimeEvent::SqlPredictionFailed {
            request_id,
            document_id,
            document_version,
            anchor,
            replacement_range,
            message,
        } => Some(UiEvent::SqlPredictionFailed {
            request_id: ui_request_id(request_id),
            document_id,
            document_version,
            anchor,
            replacement_range,
            message,
        }),
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

fn map_multi_query_output(output: db_pro_core::application::MultiQueryResult) -> UiQueryExecutionOutput {
    let db_pro_core::application::MultiQueryResult {
        results,
        result_kinds,
        total_duration_ms,
        error,
    } = output;
    let mut statements = results
        .into_iter()
        .enumerate()
        .map(|(statement_index, result)| {
            let kind = result_kinds.get(statement_index).copied().unwrap_or({
                // Compatibility for callers that construct the pre-kind
                // shape directly. Runtime-produced results always carry
                // the explicit core metadata.
                if result.columns.is_empty() {
                    db_pro_core::application::StatementResultKind::Command
                } else {
                    db_pro_core::application::StatementResultKind::ResultSet
                }
            });
            let affected_rows =
                matches!(kind, db_pro_core::application::StatementResultKind::Command).then_some(result.row_count);
            let message = affected_rows.map(|rows| format!("{rows} rows affected"));
            let duration_ms = result.duration_ms;
            UiStatementOutput {
                statement_index,
                result_set: matches!(kind, db_pro_core::application::StatementResultKind::ResultSet)
                    .then(|| map_query_result(result)),
                affected_rows,
                duration_ms,
                message,
                error: None,
            }
        })
        .collect::<Vec<_>>();
    if let Some((statement_index, error)) = error {
        statements.push(UiStatementOutput {
            statement_index,
            result_set: None,
            affected_rows: None,
            duration_ms: 0,
            message: None,
            error: Some(UiQueryError {
                code: error.code,
                message: error.message,
                position: error.position,
                detail: error.detail,
                hint: error.hint,
            }),
        });
        statements.sort_by_key(|statement| statement.statement_index);
    }
    UiQueryExecutionOutput {
        statements,
        total_duration_ms,
    }
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
                ssl_mode: ui_ssl_mode(connection.ssl_mode),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// One representative value per domain class, mapped through `map_cell`: the
    /// UI class is part of the shipping (non-serde) channel contract, so
    /// precision-sensitive values stay exact text, a null stays `Null`, bytes stay
    /// byte-safe hex and JSON stays structured text (Gate 5 A4).
    #[test]
    fn map_cell_keeps_one_ui_class_per_domain_value_class() {
        let cases = vec![
            (CellValue::Null, UiCell::Null),
            (CellValue::Bool(true), UiCell::Boolean(true)),
            (
                CellValue::Int64(9_007_199_254_740_993),
                UiCell::Number("9007199254740993".to_owned()),
            ),
            (CellValue::Float64(0.5), UiCell::Number("0.5".to_owned())),
            (
                CellValue::Decimal("12345678901234567890.12345".to_owned()),
                UiCell::Number("12345678901234567890.12345".to_owned()),
            ),
            (CellValue::Text("hello".to_owned()), UiCell::Text("hello".to_owned())),
            (
                CellValue::Uuid("3f2504e0-4f89-11d3-9a0c-0305e82c3301".to_owned()),
                UiCell::Text("3f2504e0-4f89-11d3-9a0c-0305e82c3301".to_owned()),
            ),
            (
                CellValue::DateTime("2024-03-15T10:20:30.123456".to_owned()),
                UiCell::Text("2024-03-15T10:20:30.123456".to_owned()),
            ),
            (
                CellValue::Timestamp("2024-03-15T10:20:30.123456".to_owned()),
                UiCell::Text("2024-03-15T10:20:30.123456".to_owned()),
            ),
            (
                CellValue::TimestampTz("2024-03-15T10:20:30.123456Z".to_owned()),
                UiCell::Text("2024-03-15T10:20:30.123456Z".to_owned()),
            ),
            (
                CellValue::TimeTz("10:20:30.123456+07:00".to_owned()),
                UiCell::Text("10:20:30.123456+07:00".to_owned()),
            ),
            (
                CellValue::Date("2024-03-15".to_owned()),
                UiCell::Text("2024-03-15".to_owned()),
            ),
            (
                CellValue::Time("10:20:30.123456".to_owned()),
                UiCell::Text("10:20:30.123456".to_owned()),
            ),
            (
                CellValue::Interval("1 mons 2 days 03:04:05.000006".to_owned()),
                UiCell::Text("1 mons 2 days 03:04:05.000006".to_owned()),
            ),
            (
                CellValue::Inet("192.168.0.1/24".to_owned()),
                UiCell::Text("192.168.0.1/24".to_owned()),
            ),
            (
                CellValue::Bytes(vec![0x00, 0xff, 0x10]),
                UiCell::Bytes("\\x00ff10".to_owned()),
            ),
            (
                CellValue::Json(serde_json::json!({ "a": 1 })),
                UiCell::Json("{\"a\":1}".to_owned()),
            ),
        ];

        for (domain, expected) in cases {
            assert_eq!(
                map_cell(domain.clone()),
                expected,
                "the UI class for {domain:?} changed"
            );
        }
    }

    #[test]
    fn test_ui_cell_to_domain_datetime_and_date() {
        let rfc3339 = "2026-09-12T09:50:00Z".to_string();
        assert!(matches!(
            ui_cell_to_domain(UiCell::Text(rfc3339.clone())),
            Some(CellValue::DateTime(v)) if v == rfc3339
        ));

        let standard_dt = "2026-09-12 16:50:00".to_string();
        assert!(matches!(
            ui_cell_to_domain(UiCell::Text(standard_dt.clone())),
            Some(CellValue::DateTime(v)) if v == standard_dt
        ));

        let date_only = "2026-09-12".to_string();
        assert!(matches!(
            ui_cell_to_domain(UiCell::Text(date_only.clone())),
            Some(CellValue::Date(v)) if v == date_only
        ));

        let time_only = "16:50:00".to_string();
        assert!(matches!(
            ui_cell_to_domain(UiCell::Text(time_only.clone())),
            Some(CellValue::Time(v)) if v == time_only
        ));

        let uuid_str = "550e8400-e29b-41d4-a716-446655440000".to_string();
        assert!(matches!(
            ui_cell_to_domain(UiCell::Text(uuid_str.clone())),
            Some(CellValue::Uuid(v)) if v == uuid_str
        ));
    }

    #[test]
    fn table_filter_operator_maps_to_parameterized_sql_filter_kind() {
        let filter = map_table_data_filter(UiTableDataFilter {
            column: "amount".to_owned(),
            data_type: "BIGINT".to_owned(),
            operator: UiTableFilterOperator::GreaterThanOrEqual,
            value: "100".to_owned(),
        });
        assert!(matches!(filter.op, FilterOp::Gte));
        assert!(matches!(filter.value, CellValue::Int64(100)));

        let decimal_filter = map_table_data_filter(UiTableDataFilter {
            column: "price".to_owned(),
            data_type: "NUMERIC(20,4)".to_owned(),
            operator: UiTableFilterOperator::Equals,
            value: "1234567890123456.1234".to_owned(),
        });
        assert!(matches!(decimal_filter.value, CellValue::Decimal(value) if value == "1234567890123456.1234"));

        let null_filter = map_table_data_filter(UiTableDataFilter {
            column: "deleted_at".to_owned(),
            data_type: "TIMESTAMPTZ".to_owned(),
            operator: UiTableFilterOperator::IsNull,
            value: String::new(),
        });
        assert!(matches!(null_filter.op, FilterOp::IsNull));
        assert!(matches!(null_filter.value, CellValue::Null));
    }

    #[test]
    fn map_table_info_preserves_check_constraints_for_native_ui() {
        use db_pro_core::domain::schema::{CheckConstraint, Table, TableInfo};

        let ui = map_table_info(TableInfo {
            table: Table {
                name: "orders".to_owned(),
                schema: "public".to_owned(),
                row_count: Some(3),
            },
            columns: Vec::new(),
            primary_key: None,
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: vec![CheckConstraint {
                name: "orders_total_positive".to_owned(),
                table_name: "orders".to_owned(),
                schema: "public".to_owned(),
                definition: "CHECK (total >= 0)".to_owned(),
            }],
            dependencies: Vec::new(),
        });

        assert_eq!(ui.check_constraints.len(), 1);
        assert_eq!(ui.check_constraints[0].name, "orders_total_positive");
        assert_eq!(ui.check_constraints[0].definition, "CHECK (total >= 0)");
    }
}
