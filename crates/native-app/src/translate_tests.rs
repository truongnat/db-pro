use super::*;

#[test]
fn prediction_command_keeps_document_routing_metadata() {
    let command = UiCommand::RequestSqlPrediction {
        request_id: RequestId(7),
        document_id: "query-7".to_owned(),
        document_version: 12,
        anchor: 48,
        replacement_range: (44, 48),
        context: db_pro_ui::editor::AiSqlContext {
            sql_before_cursor: "SELECT ".to_owned(),
            sql_after_cursor: String::new(),
            current_statement: "SELECT ".to_owned(),
            active_schema: "public".to_owned(),
            dialect: "PostgreSQL".to_owned(),
            referenced_tables: Vec::new(),
            table_aliases: std::collections::HashMap::new(),
            relevant_columns: Vec::new(),
            fk_neighbors: Vec::new(),
            cte_names: Vec::new(),
        },
    };
    let translated = translate_command(command).expect("prediction command must reach runtime");
    match translated {
        RuntimeCommand::RequestSqlPrediction {
            request_id,
            document_id,
            document_version,
            anchor,
            replacement_range,
            ..
        } => {
            assert_eq!(request_id, RuntimeRequestId(7));
            assert_eq!(document_id, "query-7");
            assert_eq!(document_version, 12);
            assert_eq!(anchor, 48);
            assert_eq!(replacement_range, (44, 48));
        }
        _ => panic!("unexpected runtime command"),
    }
}

#[test]
fn prediction_event_keeps_document_routing_metadata() {
    let event = RuntimeEvent::SqlPredictionReady {
        request_id: RuntimeRequestId(9),
        document_id: "query-9".to_owned(),
        document_version: 4,
        anchor: 21,
        replacement_range: (21, 21),
        prediction: "LIMIT 10".to_owned(),
    };
    let translated = translate_event(event).expect("prediction event must reach UI");
    match translated {
        UiEvent::SqlPredictionReady {
            request_id,
            document_id,
            document_version,
            anchor,
            replacement_range,
            prediction,
        } => {
            assert_eq!(request_id, RequestId(9));
            assert_eq!(document_id, "query-9");
            assert_eq!(document_version, 4);
            assert_eq!(anchor, 21);
            assert_eq!(replacement_range, (21, 21));
            assert_eq!(prediction, "LIMIT 10");
        }
        _ => panic!("unexpected UI event"),
    }
}

#[test]
fn structured_query_failure_event_keeps_database_position_and_code() {
    let event = RuntimeEvent::QueryFailedDetailed {
        request_id: RuntimeRequestId(11),
        error: db_pro_runtime::DbErrorDto {
            code: "QUERY_SYNTAX_ERROR".to_owned(),
            message: "syntax error".to_owned(),
            message_id: "error.query.syntax".to_owned(),
            retryable: false,
            position: Some(17),
        },
    };
    let translated = translate_event(event).expect("query failure must reach UI");

    match translated {
        UiEvent::QueryFailedDetailed {
            request_id,
            code,
            message,
            position,
        } => {
            assert_eq!(request_id, RequestId(11));
            assert_eq!(code, "QUERY_SYNTAX_ERROR");
            assert_eq!(message, "syntax error");
            assert_eq!(position, Some(17));
        }
        _ => panic!("unexpected UI event"),
    }
}

#[test]
fn save_query_command_preserves_existing_saved_query_identity() {
    let saved_query_id = "8f7d8d4e-8f30-4d94-bf4f-a7c55d6b2c44";
    let translated = translate_command(UiCommand::SaveQuery {
        request_id: RequestId(12),
        connection_id: "connection-1".to_owned(),
        saved_query_id: Some(saved_query_id.to_owned()),
        name: "Users".to_owned(),
        sql: "SELECT * FROM users".to_owned(),
        folder: None,
    })
    .expect("save command must reach runtime");

    match translated {
        RuntimeCommand::SaveQuery {
            saved_query_id: actual, ..
        } => {
            assert_eq!(actual.as_deref(), Some(saved_query_id));
        }
        _ => panic!("unexpected runtime command"),
    }
}

#[test]
fn multi_query_command_routes_to_runtime() {
    let translated = translate_command(UiCommand::RunQueryMulti {
        request_id: RequestId(13),
        connection_id: "connection-1".to_owned(),
        sql: "SELECT 1; SELECT 2;".to_owned(),
    })
    .expect("multi-query command must reach runtime");

    assert!(matches!(
        translated,
        RuntimeCommand::ExecuteQueryMulti {
            request_id: RuntimeRequestId(13),
            ..
        }
    ));
}

#[test]
fn multi_query_translation_uses_explicit_statement_result_kind() {
    let event = RuntimeEvent::QueryMultiCompleted {
        request_id: RuntimeRequestId(14),
        output: db_pro_core::application::MultiQueryResult {
            results: vec![
                db_pro_core::domain::query::QueryResult {
                    columns: Vec::new(),
                    rows: Vec::new(),
                    row_count: 3,
                    duration_ms: 1,
                },
                db_pro_core::domain::query::QueryResult {
                    columns: vec![db_pro_core::domain::query::ColumnMeta {
                        name: "value".to_owned(),
                        data_type: "integer".to_owned(),
                        nullable: false,
                    }],
                    rows: Vec::new(),
                    row_count: 0,
                    duration_ms: 1,
                },
            ],
            result_kinds: vec![
                db_pro_core::application::StatementResultKind::Command,
                db_pro_core::application::StatementResultKind::ResultSet,
            ],
            total_duration_ms: 2,
            error: None,
        },
    };

    let translated = translate_event(event).expect("multi-query event must reach UI");
    match translated {
        UiEvent::QueryMultiCompleted { output, .. } => {
            assert_eq!(output.statements[0].affected_rows, Some(3));
            assert!(output.statements[0].result_set.is_none());
            assert!(output.statements[1].affected_rows.is_none());
            assert!(output.statements[1].result_set.is_some());
        }
        _ => panic!("unexpected UI event"),
    }
}

/// The shipping IPC channel is the in-process `UiQueryResult`, not a serde DTO, so
/// the field mapping itself is the contract: every domain field must land in the
/// same-named UI field and every domain value class must keep its UI class. A
/// silently dropped field or a class collapsed into text fails here (Gate 5 A4).
#[test]
fn query_result_translation_keeps_every_field_and_value_class() {
    use db_pro_core::domain::query::{CellValue, ColumnMeta, QueryResult, Row};
    use db_pro_ui::{UiCell, UiEvent};

    let result = QueryResult {
        columns: vec![
            ColumnMeta {
                name: "amount".to_owned(),
                data_type: "numeric(20,4)".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "note".to_owned(),
                data_type: "text".to_owned(),
                nullable: true,
            },
        ],
        rows: vec![Row(vec![
            CellValue::Decimal("12345678901234567890.12345".to_owned()),
            CellValue::Null,
        ])],
        row_count: 1,
        duration_ms: 7,
    };

    let translated = translate_event(RuntimeEvent::QueryCompleted {
        request_id: RuntimeRequestId(21),
        result,
    })
    .expect("a completed query must reach the UI");

    match translated {
        UiEvent::QueryCompleted { result, .. } => {
            assert_eq!(result.row_count, 1, "row_count must not be dropped");
            assert_eq!(result.duration_ms, 7, "duration_ms must not be dropped");
            assert_eq!(result.columns.len(), 2, "no column may be dropped or added");
            assert_eq!(result.columns[0].name, "amount");
            assert_eq!(result.columns[0].data_type, "numeric(20,4)");
            assert!(result.columns[0].nullable);
            assert_eq!(result.columns[1].name, "note");
            assert_eq!(result.columns[1].data_type, "text");
            assert!(result.columns[1].nullable);
            assert_eq!(
                result.rows,
                vec![vec![
                    UiCell::Number("12345678901234567890.12345".to_owned()),
                    UiCell::Null
                ]],
                "the canonical decimal text and the null class must survive translation"
            );
        }
        _ => panic!("unexpected UI event"),
    }
}
