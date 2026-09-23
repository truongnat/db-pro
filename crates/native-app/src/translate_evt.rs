//! RuntimeEvent → UiEvent mapping.
use super::super::*;

use super::translate_cmd::*;
use super::translate_map::*;

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
        RuntimeEvent::MonitoringSnapshotLoaded { request_id, snapshot } => Some(UiEvent::MonitoringSnapshotLoaded {
            request_id: ui_request_id(request_id),
            snapshot,
        }),
        RuntimeEvent::MonitoringWorkloadLoaded { request_id, workload } => Some(UiEvent::MonitoringWorkloadLoaded {
            request_id: ui_request_id(request_id),
            workload,
        }),
        RuntimeEvent::AuditPageLoaded { request_id, page } => Some(UiEvent::AuditPageLoaded {
            request_id: ui_request_id(request_id),
            page,
        }),
        RuntimeEvent::PgSettingsLoaded { request_id, snapshot } => Some(UiEvent::PgSettingsLoaded {
            request_id: ui_request_id(request_id),
            snapshot,
        }),
        RuntimeEvent::PgSettingActionCompleted {
            request_id,
            action,
            name,
        } => Some(UiEvent::PgSettingActionCompleted {
            request_id: ui_request_id(request_id),
            action: action.to_owned(),
            name,
        }),
        RuntimeEvent::FdwInventoryLoaded { request_id, inventory } => Some(UiEvent::FdwInventoryLoaded {
            request_id: ui_request_id(request_id),
            inventory,
        }),
        RuntimeEvent::FdwActionCompleted {
            request_id,
            action,
            name,
        } => Some(UiEvent::FdwActionCompleted {
            request_id: ui_request_id(request_id),
            action: action.to_owned(),
            name,
        }),
        RuntimeEvent::ReplicationInventoryLoaded { request_id, inventory } => {
            Some(UiEvent::ReplicationInventoryLoaded {
                request_id: ui_request_id(request_id),
                inventory,
            })
        }
        RuntimeEvent::ReplicationActionCompleted {
            request_id,
            action,
            name,
        } => Some(UiEvent::ReplicationActionCompleted {
            request_id: ui_request_id(request_id),
            action: action.to_owned(),
            name,
        }),
        RuntimeEvent::EventTriggerInventoryLoaded { request_id, inventory } => {
            Some(UiEvent::EventTriggerInventoryLoaded {
                request_id: ui_request_id(request_id),
                inventory,
            })
        }
        RuntimeEvent::EventTriggerActionCompleted {
            request_id,
            action,
            name,
        } => Some(UiEvent::EventTriggerActionCompleted {
            request_id: ui_request_id(request_id),
            action: action.to_owned(),
            name,
        }),
        RuntimeEvent::MonitoringActionCompleted {
            request_id,
            action,
            backend_id,
            succeeded,
        } => Some(UiEvent::MonitoringActionCompleted {
            request_id: ui_request_id(request_id),
            action: action.to_owned(),
            backend_id,
            succeeded,
        }),
        RuntimeEvent::UsersLoaded { request_id, users } => Some(UiEvent::UsersLoaded {
            request_id: ui_request_id(request_id),
            users,
        }),
        RuntimeEvent::PrivilegesLoaded {
            request_id,
            role_name,
            privileges,
        } => Some(UiEvent::PrivilegesLoaded {
            request_id: ui_request_id(request_id),
            role_name,
            privileges,
        }),
        RuntimeEvent::MembershipsLoaded {
            request_id,
            member,
            memberships,
        } => Some(UiEvent::MembershipsLoaded {
            request_id: ui_request_id(request_id),
            member,
            memberships,
        }),
        RuntimeEvent::TableRlsLoaded { request_id, state } => Some(UiEvent::TableRlsLoaded {
            request_id: ui_request_id(request_id),
            state,
        }),
        RuntimeEvent::DataDiffLoaded { request_id, diff } => Some(UiEvent::DataDiffLoaded {
            request_id: ui_request_id(request_id),
            diff,
        }),
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
        RuntimeEvent::AgentProviderReady { provider, detail } => Some(UiEvent::AgentProviderReady { provider, detail }),
        RuntimeEvent::AgentFailed { request_id, message } => translate_agent_failed(request_id, message),
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
        RuntimeEvent::AgentForgotten { request_id } => Some(UiEvent::AgentForgotten {
            request_id: ui_request_id(request_id),
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
                tags: connection.tags,
                group: connection.group,
                favorite: connection.favorite,
                environment: connection.environment.as_label().to_owned(),
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
        identity_arguments: function.identity_arguments,
        language: function.language,
        volatility: function.volatility,
        security_definer: function.security_definer,
        parameters: function
            .parameters
            .into_iter()
            .map(|p| db_pro_ui::UiRoutineParameter {
                name: p.name,
                data_type: p.data_type,
                mode: p.mode,
                has_default: p.has_default,
                default_expr: p.default_expr,
            })
            .collect(),
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
