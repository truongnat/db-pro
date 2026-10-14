//! Tests for `../table_copy.rs` (moved out of the source file).
use std::sync::Mutex;

use super::*;
use crate::domain::query::{ColumnMeta, Row};

struct MemoryCopy {
    queries: Mutex<Vec<(String, String)>>,
    executes: Mutex<Vec<(String, String, Vec<String>)>>,
}

impl MemoryCopy {
    fn new() -> Self {
        Self {
            queries: Mutex::new(Vec::new()),
            executes: Mutex::new(Vec::new()),
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

    fn execute(&self, connection_id: &str, sql: &str, params: &[QueryParam]) -> Result<u64, String> {
        let values = params
            .iter()
            .map(|param| match param {
                QueryParam::Text(value) => value.clone(),
                other => format!("{other:?}"),
            })
            .collect();
        self.executes
            .lock()
            .expect("execute log")
            .push((connection_id.to_owned(), sql.to_owned(), values));
        Ok(1)
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
