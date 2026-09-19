use super::*;
#[cfg(test)]
use crate::RequestId;

/// A statement (or script) the classifier rates `Destructive`, held until the user
/// confirms the exact text the prompt displayed.
#[derive(Debug, Clone)]
pub(crate) struct PendingDestructiveRun {
    pub(crate) sql: String,
    pub(crate) execution_range: (usize, usize),
    pub(crate) version: u64,
    pub(crate) all_statements: bool,
}

pub(super) struct QueryHistoryRecord {
    pub(super) sql: String,
    pub(super) connection_id: Option<String>,
    pub(super) schema: Option<String>,
    pub(super) started_at: String,
    pub(super) status: UiQueryHistoryStatus,
    pub(super) duration_ms: u64,
    pub(super) row_count: Option<u64>,
    pub(super) affected_rows: Option<u64>,
    pub(super) error_code: Option<String>,
    pub(super) error_summary: Option<String>,
}

impl DbProApp {
    pub(super) fn apply_runtime_events(&mut self) -> bool {
        let events: Vec<UiEvent> = self
            .task_bridge
            .drain_events(crate::runtime::MAX_RUNTIME_EVENTS_PER_FRAME)
            .collect();
        let batch_was_full = events.len() == crate::runtime::MAX_RUNTIME_EVENTS_PER_FRAME;
        for event in events {
            self.apply_runtime_event(event);
        }
        batch_was_full
    }

    // Query completion/history/prediction: `events_query.rs`.
    // Shortcuts / dispatch: `events_query_dispatch.rs`.
}

#[cfg(test)]
mod row_reload_tests {
    use super::*;
    use crate::{UiColumn, UiTableColumn};

    fn row_result(name: &str) -> UiQueryResult {
        UiQueryResult {
            columns: vec![
                UiColumn {
                    name: "id".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: false,
                },
                UiColumn {
                    name: "name".to_owned(),
                    data_type: "text".to_owned(),
                    nullable: false,
                },
            ],
            rows: vec![vec![UiCell::Number("7".to_owned()), UiCell::Text(name.to_owned())]],
            row_count: 1,
            duration_ms: 0,
        }
    }

    fn row_reload_app() -> DbProApp {
        DbProApp {
            table_state: TableState {
                table_info: Some(UiTableInfo {
                    schema: "public".to_owned(),
                    name: "customers".to_owned(),
                    row_count: Some(1),
                    columns: vec![
                        UiTableColumn {
                            name: "id".to_owned(),
                            data_type: "integer".to_owned(),
                            nullable: false,
                            default: None,
                            is_primary_key: true,
                            ..Default::default()
                        },
                        UiTableColumn {
                            name: "name".to_owned(),
                            data_type: "text".to_owned(),
                            nullable: false,
                            default: None,
                            is_primary_key: false,
                            ..Default::default()
                        },
                    ],
                    primary_key: Some(vec!["id".to_owned()]),
                    indexes: Vec::new(),
                    foreign_keys: Vec::new(),
                    check_constraints: Vec::new(),
                    dependencies: Vec::new(),
                }),
                table_data_result: Some(row_result("local server value")),
                table_row_reload_request: Some(RequestId(9)),
                table_row_reload_identity: Some(RowIdentity {
                    original_pk_columns: vec!["id".to_owned()],
                    original_pk_values: vec![UiCell::Number("7".to_owned())],
                }),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn row_reload_merges_server_values_by_original_identity() {
        let mut app = row_reload_app();
        app.on_table_data_loaded(RequestId(9), row_result("fresh server value"), 1);

        let result = app
            .table_state
            .table_data_result
            .expect("table result should remain visible");
        assert_eq!(result.rows[0][1], UiCell::Text("fresh server value".to_owned()));
        assert!(app.table_state.table_row_reload_request.is_none());
    }

    #[test]
    fn row_reload_reports_deleted_row_without_dropping_local_result() {
        let mut app = row_reload_app();
        let empty = UiQueryResult {
            columns: row_result("unused").columns,
            rows: Vec::new(),
            row_count: 0,
            duration_ms: 0,
        };
        app.on_table_data_loaded(RequestId(9), empty, 0);

        assert_eq!(app.feedback.runtime_message, "Row was deleted");
        assert!(app.table_state.table_data_result.is_some());
    }

    #[test]
    fn deleting_non_active_connection_preserves_active_session() {
        let mut app = DbProApp {
            connection_lifecycle: ConnectionLifecycleState {
                connected: true,
                active_connection_id: Some("conn-a".to_owned()),
                pending_request: Some(RequestId(11)),
                pending_connection_id: Some("conn-b".to_owned()),
                failed_connection_ids: ["conn-a".to_owned(), "conn-b".to_owned()].into_iter().collect(),
                errors: [
                    ("conn-a".to_owned(), "stale".to_owned()),
                    ("conn-b".to_owned(), "gone".to_owned()),
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            },
            ..Default::default()
        };

        app.on_operation_completed(RequestId(11), "connection.deleted".to_owned());

        assert_eq!(app.connection_lifecycle.active_connection_id(), Some("conn-a"));
        assert!(app.connection_lifecycle.is_connected());
        assert!(app.connection_lifecycle.pending_connection_id.is_none());
        assert!(app.connection_lifecycle.failed_connection_ids.contains("conn-a"));
        assert!(!app.connection_lifecycle.failed_connection_ids.contains("conn-b"));
        assert_eq!(
            app.connection_lifecycle.errors.get("conn-a").map(String::as_str),
            Some("stale")
        );
        assert!(!app.connection_lifecycle.errors.contains_key("conn-b"));
    }

    #[test]
    fn deleting_active_connection_clears_session() {
        let mut app = DbProApp {
            connection_lifecycle: ConnectionLifecycleState {
                connected: true,
                active_connection_id: Some("conn-a".to_owned()),
                pending_request: Some(RequestId(12)),
                pending_connection_id: Some("conn-a".to_owned()),
                failed_connection_ids: ["conn-a".to_owned()].into_iter().collect(),
                ..Default::default()
            },
            ..Default::default()
        };

        app.on_operation_completed(RequestId(12), "connection.deleted".to_owned());

        assert!(app.connection_lifecycle.active_connection_id().is_none());
        assert!(!app.connection_lifecycle.is_connected());
        assert!(app.connection_lifecycle.pending_connection_id.is_none());
        assert!(app.connection_lifecycle.failed_connection_ids.is_empty());
    }
}
