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
