//! Tests for `../table_copy.rs` (moved out of the source file).
use std::sync::Mutex;

use super::*;
use crate::domain::query::{ColumnMeta, Row};

struct MemoryCopy {
    queries: Mutex<Vec<(String, String)>>,
    executes: Mutex<Vec<(String, String, Vec<String>)>>,
    transaction_batch_sizes: Mutex<Vec<usize>>,
}

impl MemoryCopy {
    fn new() -> Self {
        Self {
            queries: Mutex::new(Vec::new()),
            executes: Mutex::new(Vec::new()),
            transaction_batch_sizes: Mutex::new(Vec::new()),
        }
    }
}

impl TableCopyConnector for MemoryCopy {
    fn quote(&self, _connection_id: &str, identifier: &str) -> Result<String, String> {
        Ok(format!("\"{}\"", identifier.replace('"', "\"\"")))
    }

    fn placeholder(&self, _connection_id: &str, index: usize) -> Result<String, String> {
        Ok(format!("${index}"))
    }

    fn query(&self, connection_id: &str, sql: &str) -> Result<QueryResult, String> {
        self.queries
            .lock()
            .expect("query log")
            .push((connection_id.to_owned(), sql.to_owned()));
        if connection_id == "prod" {
            Ok(result(
                &[("id", false), ("name", true)],
                vec![vec![text("1"), text("Alice")], vec![text("2"), text("Bob")]],
            ))
        } else if connection_id == "stage" {
            Ok(result(&[("id", false)], vec![vec![text("1")]]))
        } else {
            Err(format!("connection {connection_id} is not open"))
        }
    }

    fn execute_transaction(
        &self,
        connection_id: &str,
        statements: Vec<ParameterizedTransactionStatement>,
    ) -> Result<(), String> {
        self.transaction_batch_sizes
            .lock()
            .expect("transaction log")
            .push(statements.len());
        let mut executes = self.executes.lock().expect("execute log");
        for statement in statements {
            let values = statement
                .params
                .iter()
                .map(|param| match param {
                    QueryParam::Text(value) => value.clone(),
                    other => format!("{other:?}"),
                })
                .collect();
            executes.push((connection_id.to_owned(), statement.sql, values));
        }
        Ok(())
    }
}

fn text(value: &str) -> CellValue {
    CellValue::Text(value.to_owned())
}

fn result(columns: &[(&str, bool)], rows: Vec<Vec<CellValue>>) -> QueryResult {
    let row_count = rows.len() as u64;
    QueryResult {
        columns: columns
            .iter()
            .map(|(name, nullable)| ColumnMeta {
                name: (*name).to_owned(),
                data_type: "text".into(),
                nullable: *nullable,
            })
            .collect(),
        rows: rows.into_iter().map(Row).collect(),
        row_count,
        duration_ms: 0,
    }
}

fn request(format: CopyFormat, conflict: TransferConflictPolicy) -> TableCopyRequest {
    TableCopyRequest {
        source_connection_id: "prod".into(),
        target_connection_id: "stage".into(),
        tables: vec!["public.orders".into()],
        format,
        conflict,
        columns: Vec::new(),
    }
}

#[test]
fn copy_reads_source_and_target_and_applies_conflict() {
    let connector = MemoryCopy::new();
    let skipped = copy_tables(&connector, &request(CopyFormat::Csv, TransferConflictPolicy::Skip)).unwrap();
    assert_eq!(skipped.source_connection_id, "prod");
    assert_eq!(skipped.target_connection_id, "stage");
    assert_eq!(skipped.rows_written, 1);
    assert_eq!(skipped.rows_skipped, 1);
    assert!(skipped.body.starts_with("id,name\n"), "{}", skipped.body);
    assert!(skipped.body.contains("2,Bob"), "{}", skipped.body);
    assert!(!skipped.body.contains("1,Alice"), "{}", skipped.body);

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
    assert_eq!(executes[0].0, "stage");
    assert!(executes[0].1.starts_with("INSERT INTO"), "{}", executes[0].1);
    assert_eq!(executes[0].2, vec!["2".to_owned(), "Bob".to_owned()]);

    let connector = MemoryCopy::new();
    let json = copy_tables(&connector, &request(CopyFormat::Json, TransferConflictPolicy::Replace)).unwrap();
    assert!(json.body.contains("\"name\":\"Alice\""), "{}", json.body);
    assert!(json.body.contains("\"name\":\"Bob\""), "{}", json.body);
    assert_eq!(json.rows_written, 2);
    let executes = connector.executes.lock().expect("execute log");
    assert!(
        executes.iter().any(|(_, sql, _)| sql.starts_with("DELETE FROM")),
        "{executes:?}"
    );
    assert!(executes
        .iter()
        .any(|(_, _, params)| params == &vec!["2".to_owned(), "Bob".to_owned()]));

    let connector = MemoryCopy::new();
    let error = copy_tables(&connector, &request(CopyFormat::Csv, TransferConflictPolicy::Abort)).unwrap_err();
    assert!(error.contains("conflict on key 1"), "{error}");
    assert!(connector.executes.lock().expect("execute log").is_empty());
}

#[test]
fn copies_multiple_tables_in_one_target_transaction() {
    let mut request = request(CopyFormat::Csv, TransferConflictPolicy::Skip);
    request.tables.push("public.items".into());
    let connector = MemoryCopy::new();

    copy_tables(&connector, &request).expect("copy both tables");

    assert_eq!(
        *connector.transaction_batch_sizes.lock().expect("transaction log"),
        vec![2],
        "all target statements across selected tables must share one transaction"
    );
}

struct ReplaceFailureCopy {
    target: Mutex<Vec<(String, String)>>,
}

impl ReplaceFailureCopy {
    fn new() -> Self {
        Self {
            target: Mutex::new(vec![("1".into(), "Existing".into())]),
        }
    }
}

impl TableCopyConnector for ReplaceFailureCopy {
    fn quote(&self, _connection_id: &str, identifier: &str) -> Result<String, String> {
        Ok(format!("\"{}\"", identifier.replace('"', "\"\"")))
    }

    fn placeholder(&self, _connection_id: &str, index: usize) -> Result<String, String> {
        Ok(format!("${index}"))
    }

    fn query(&self, connection_id: &str, _sql: &str) -> Result<QueryResult, String> {
        if connection_id == "prod" {
            return Ok(result(
                &[("id", false), ("name", true)],
                vec![vec![text("1"), text("Replacement")]],
            ));
        }
        let rows = self
            .target
            .lock()
            .expect("target rows")
            .iter()
            .map(|(id, _)| vec![text(id)])
            .collect();
        Ok(result(&[("id", false)], rows))
    }

    fn execute_transaction(
        &self,
        _connection_id: &str,
        statements: Vec<ParameterizedTransactionStatement>,
    ) -> Result<(), String> {
        let mut target = self.target.lock().expect("target rows");
        let mut pending = target.clone();
        for statement in statements {
            let key = match statement.params.first() {
                Some(QueryParam::Text(value)) => value,
                _ => return Err("expected text key".into()),
            };
            if statement.sql.starts_with("DELETE FROM") {
                pending.retain(|(id, _)| id != key);
            } else if statement.sql.starts_with("INSERT INTO") {
                return Err("injected replacement insert failure".into());
            } else {
                return Err(format!("unexpected statement: {}", statement.sql));
            }
        }
        *target = pending;
        Ok(())
    }
}

#[test]
fn replace_failure_preserves_the_existing_target_row() {
    let connector = ReplaceFailureCopy::new();
    let error = copy_tables(&connector, &request(CopyFormat::Csv, TransferConflictPolicy::Replace))
        .expect_err("the injected replacement insert must fail");

    assert!(error.contains("injected replacement insert failure"), "{error}");
    assert_eq!(
        *connector.target.lock().expect("target rows"),
        vec![("1".into(), "Existing".into())],
        "a failed replacement must not delete the pre-existing target row"
    );
}
