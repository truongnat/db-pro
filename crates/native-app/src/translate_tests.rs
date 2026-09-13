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
