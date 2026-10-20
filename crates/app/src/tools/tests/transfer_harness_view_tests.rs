//! Tests for `../transfer_harness_view.rs` (moved out of the source file).
    use super::*;
    use crate::UiSchemaColumn;

    #[test]
    fn synthetic_seed_context_builds_sql_from_explicit_management_state() {
        let mut management = DatabaseManagementState::default();
        management.synthetic_data.synthetic_table = "public.users".to_owned();
        management.synthetic_data.synthetic_row_count = "2".to_owned();
        management.synthetic_data.synthetic_seed = "42".to_owned();
        management.synthetic_data.synthetic_null_pct = "0".to_owned();
        let table_details = vec![UiTableSummary {
            schema: "public".to_owned(),
            name: "users".to_owned(),
            row_count: None,
            columns: vec![UiSchemaColumn {
                name: "id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
                is_primary_key: true,
            }],
            foreign_keys: Vec::new(),
        }];
        let mut feedback = FeedbackState::default();
        let context = TransferHarnessContext {
            management: &mut management,
            feedback: &mut feedback,
            table_details: &table_details,
        };

        let (sql, count) = context.build_synthetic_seed_sql().expect("synthetic SQL");

        assert_eq!(count, 2);
        assert!(sql.contains("INSERT INTO \"public\".\"users\""));
    }

    struct OrdersCopy {
        queries: std::sync::Mutex<Vec<(String, String)>>,
        executes: std::sync::Mutex<Vec<(String, Vec<String>)>>,
    }

    impl OrdersCopy {
        fn new() -> Self {
            Self {
                queries: std::sync::Mutex::new(Vec::new()),
                executes: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    impl db_pro_core::application::TableCopyConnector for OrdersCopy {
        fn quote(&self, _connection_id: &str, identifier: &str) -> Result<String, String> {
            Ok(format!("\"{}\"", identifier.replace('"', "\"\"")))
        }

        fn placeholder(&self, _connection_id: &str, index: usize) -> Result<String, String> {
            Ok(format!("${index}"))
        }

        fn query(&self, connection_id: &str, sql: &str) -> Result<db_pro_core::domain::query::QueryResult, String> {
            use db_pro_core::domain::query::{CellValue, ColumnMeta, QueryResult, Row};
            self.queries
                .lock()
                .expect("query log")
                .push((connection_id.to_owned(), sql.to_owned()));
            let (columns, rows) = if connection_id == "prod" {
                (
                    vec![("id", false), ("name", true)],
                    vec![
                        vec![CellValue::Text("1".into()), CellValue::Text("Alice".into())],
                        vec![CellValue::Text("2".into()), CellValue::Text("Bob".into())],
                    ],
                )
            } else if connection_id == "stage" {
                (vec![("id", false)], vec![vec![CellValue::Text("1".into())]])
            } else {
                return Err(format!("connection {connection_id} is not open"));
            };
            let row_count = rows.len() as u64;
            Ok(QueryResult {
                columns: columns
                    .into_iter()
                    .map(|(name, nullable)| ColumnMeta {
                        name: name.into(),
                        data_type: "text".into(),
                        nullable,
                    })
                    .collect(),
                rows: rows.into_iter().map(Row).collect(),
                row_count,
                duration_ms: 0,
            })
        }

        fn execute_transaction(
            &self,
            connection_id: &str,
            statements: Vec<db_pro_core::ports::ParameterizedTransactionStatement>,
        ) -> Result<(), String> {
            for statement in statements {
                let values = statement
                    .params
                    .iter()
                    .filter_map(|param| match param {
                        db_pro_core::domain::query::QueryParam::Text(value) => Some(value.clone()),
                        _ => None,
                    })
                    .collect();
                self.executes
                    .lock()
                    .expect("execute log")
                    .push((format!("{connection_id} {}", statement.sql), values));
            }
            Ok(())
        }
    }

    fn choose_orders(management: &mut DatabaseManagementState, format: &str, conflict: &str) {
        management.transfer.source_connection_id = "prod".into();
        management.transfer.target_connection_id = "stage".into();
        management.transfer.copy_tables = "public.orders".into();
        management.transfer.copy_format = format.into();
        management.transfer.copy_conflict = conflict.into();
    }

    fn run_copy(management: &mut DatabaseManagementState, connector: &std::sync::Arc<OrdersCopy>) -> String {
        let mut feedback = FeedbackState::default();
        let table_details = Vec::new();
        let mut context = TransferHarnessContext {
            management,
            feedback: &mut feedback,
            table_details: &table_details,
        };
        // Copy runs on a worker thread (#421); tests drain it deterministically.
        context.run_chosen_copy(connector.clone());
        context.wait_for_pending();
        feedback.runtime_message
    }

    #[test]
    fn chosen_copy_reads_tables_from_empty_form() {
        use db_pro_core::domain::transfer::{
            TransferConflictPolicy, TransferSourceKind, TransferStatus, TransferTargetKind,
        };

        let mut management = DatabaseManagementState::default();
        assert!(management.transfer.copy_columns.is_empty());
        choose_orders(&mut management, "csv", "skip");
        let connector = std::sync::Arc::new(OrdersCopy::new());
        let message = run_copy(&mut management, &connector);
        assert!(message.contains("prod"), "{message}");
        assert!(message.contains("stage"), "{message}");
        assert!(!message.contains("src-conn"), "{message}");

        let job = &management.transfer.transfer_jobs[0];
        match &job.source {
            TransferSourceKind::Table {
                connection_id,
                schema,
                table,
            } => {
                assert_eq!(connection_id, "prod");
                assert_eq!(schema, "public");
                assert_eq!(table, "orders");
            }
            other => panic!("source {other:?}"),
        }
        match &job.target {
            TransferTargetKind::Table {
                connection_id,
                schema,
                table,
            } => {
                assert_eq!(connection_id, "stage");
                assert_eq!(schema, "public");
                assert_eq!(table, "orders");
            }
            other => panic!("target {other:?}"),
        }
        assert_eq!(job.options.conflict, TransferConflictPolicy::Skip);
        assert_eq!(job.status, TransferStatus::Succeeded);
        let body = management.transfer.last_copy_body.clone();
        assert!(body.starts_with("id,name\n"), "{body}");
        assert!(body.contains("2,Bob"), "{body}");
        assert!(!body.contains("1,Alice"), "{body}");
        assert!(!body.contains("src-conn"), "{body}");
        assert!(!body.contains("row-"), "{body}");
        let queries = connector.queries.lock().expect("query log");
        assert!(
            queries
                .iter()
                .any(|(id, sql)| id == "prod" && sql.contains("SELECT * FROM \"public\".\"orders\"")),
            "{queries:?}"
        );
        assert!(
            queries
                .iter()
                .any(|(id, sql)| id == "stage" && sql.contains("SELECT \"id\" FROM \"public\".\"orders\"")),
            "{queries:?}"
        );
        drop(queries);
        let executes = connector.executes.lock().expect("execute log");
        assert_eq!(executes.len(), 1, "{executes:?}");
        assert_eq!(executes[0].1, vec!["2".to_owned(), "Bob".to_owned()]);
        drop(executes);

        choose_orders(&mut management, "csv", "abort");
        let before = management.transfer.transfer_jobs.len();
        let abort = std::sync::Arc::new(OrdersCopy::new());
        let message = run_copy(&mut management, &abort);
        assert!(message.contains("conflict on key 1"), "{message}");
        assert_eq!(management.transfer.transfer_jobs.len(), before);
        assert_eq!(management.transfer.last_copy_body, body);
        assert!(abort.executes.lock().expect("execute log").is_empty());

        choose_orders(&mut management, "json", "replace");
        let replace = std::sync::Arc::new(OrdersCopy::new());
        let message = run_copy(&mut management, &replace);
        assert!(message.contains("json"), "{message}");
        let json = &management.transfer.last_copy_body;
        assert!(json.starts_with("[{"), "{json}");
        assert!(json.contains("\"name\":\"Alice\""), "{json}");
        assert!(json.contains("\"name\":\"Bob\""), "{json}");
        assert_eq!(
            management.transfer.transfer_jobs[0].options.conflict,
            TransferConflictPolicy::Replace
        );
        assert_eq!(management.transfer.transfer_jobs[0].status, TransferStatus::Succeeded);
        let writes = replace.executes.lock().expect("execute log");
        assert!(writes.iter().any(|(sql, _)| sql.contains("DELETE FROM")), "{writes:?}");
        assert!(writes.iter().any(|(_, params)| params == &["1".to_owned(), "Alice".to_owned()]));
        assert!(writes.iter().any(|(_, params)| params == &["2".to_owned(), "Bob".to_owned()]));
        drop(writes);

        let hub = include_str!("../tools_hub_surface_view.rs");
        assert!(!hub.contains("Cross-connection streaming"));
        assert!(hub.contains("Copy chosen tables between connections as CSV or JSON"));
    }