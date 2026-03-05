use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::commands::{connection, navigator};
use crate::models::{ConnectionProfile, DbEngine, QueryRequest, SortDirection};
use crate::sample;
use crate::state::AppState;

use super::{cancel_query, cancellation, execute_query};

struct SqliteHarness {
    state: AppState,
    connection_id: String,
    root_dir: PathBuf,
}

impl SqliteHarness {
    async fn new(test_name: &str) -> Self {
        let root_dir = create_unique_temp_dir(test_name);
        let storage_path = root_dir.join("connections.json");
        let sample_path = root_dir.join("db-pro-sample.sqlite");

        sample::ensure_sample_sqlite(&sample_path)
            .await
            .expect("sample bootstrap should succeed");
        let sample_profile = sample::sample_connection_profile(&sample_path);
        let connection_id = sample_profile.id.clone();

        let mut connections = HashMap::new();
        connections.insert(connection_id.clone(), sample_profile);

        Self {
            state: AppState::new(storage_path, connections),
            connection_id,
            root_dir,
        }
    }

    fn query_request(&self, sql: &str) -> QueryRequest {
        QueryRequest {
            connection_id: self.connection_id.clone(),
            sql: sql.to_string(),
            limit: None,
            page_size: None,
            offset: None,
            timeout_ms: None,
            quick_filter: None,
            filter_columns: None,
            sort_column: None,
            sort_direction: None,
        }
    }

    fn upsert_connection(&self, profile: ConnectionProfile) {
        let mut guard = self
            .state
            .connections
            .write()
            .expect("state write lock should succeed");
        guard.insert(profile.id.clone(), profile);
    }
}

impl Drop for SqliteHarness {
    fn drop(&mut self) {
        if let Err(err) = std::fs::remove_dir_all(&self.root_dir) {
            eprintln!(
                "Failed to remove temporary test directory '{}': {err}",
                self.root_dir.display()
            );
        }
    }
}

#[tokio::test]
async fn sample_sqlite_bootstrap_and_paging_query_work() {
    let harness = SqliteHarness::new("paging").await;

    let mut request = harness.query_request(
        "SELECT id, name
         FROM customers
         ORDER BY id",
    );
    request.page_size = Some(2);

    let result = execute_query(request, &harness.state)
        .await
        .expect("query should succeed");

    assert!(result.is_row_query);
    assert_eq!(result.columns, vec!["id".to_string(), "name".to_string()]);
    assert_eq!(result.rows.len(), 2);
    assert!(result.has_more);
    assert_eq!(result.page_size, 2);
    assert_eq!(result.page_offset, 0);
    assert_eq!(result.rows[0][0], "1");
    assert_eq!(result.rows[0][1], "Alice Nguyen");
}

#[tokio::test]
async fn sqlite_timeout_returns_deterministic_error() {
    let harness = SqliteHarness::new("timeout").await;

    let mut request = harness.query_request(
        "WITH RECURSIVE counter(x) AS (
            VALUES(1)
            UNION ALL
            SELECT x + 1 FROM counter WHERE x < 50000000
         )
         SELECT COUNT(*) AS total FROM counter",
    );
    request.timeout_ms = Some(1_000);

    let error = execute_query(request, &harness.state)
        .await
        .expect_err("query should time out");

    assert!(error.contains("Query execution timed out after 1000 ms"));
}

#[tokio::test]
async fn cancel_query_reports_running_and_idle_states() {
    let harness = SqliteHarness::new("cancel").await;

    let token = cancellation::register(&harness.state, &harness.connection_id)
        .expect("registering cancellation token should succeed");
    assert!(!token.is_cancelled());

    let running_message = cancel_query(&harness.connection_id, &harness.state)
        .expect("cancelling running query should succeed");
    assert_eq!(running_message, "Cancellation requested.");
    assert!(token.is_cancelled());

    cancellation::clear(&harness.state, &harness.connection_id, &token)
        .expect("clearing cancellation token should succeed");

    let idle_message = cancel_query(&harness.connection_id, &harness.state)
        .expect("idle cancellation should succeed");
    assert_eq!(idle_message, "No running query to cancel.");
}

#[tokio::test]
async fn ddl_marks_schema_changed_and_navigator_reflects_new_table() {
    let harness = SqliteHarness::new("ddl-navigator").await;

    let ddl_result = execute_query(
        harness.query_request(
            "CREATE TABLE IF NOT EXISTS audit_events (
                id INTEGER PRIMARY KEY,
                label TEXT NOT NULL
             )",
        ),
        &harness.state,
    )
    .await
    .expect("ddl should succeed");

    assert!(!ddl_result.is_row_query);
    assert!(ddl_result.schema_changed);

    let tree = navigator::load_navigator(&harness.connection_id, &harness.state)
        .await
        .expect("navigator load should succeed");
    let main_schema = tree
        .schemas
        .iter()
        .find(|schema| schema.name == "main")
        .expect("sqlite navigator should have main schema");

    assert!(
        main_schema
            .tables
            .iter()
            .any(|table| table.name == "audit_events"),
        "expected audit_events table in navigator"
    );
}

#[tokio::test]
async fn reset_connections_restores_only_sample_profile() {
    let harness = SqliteHarness::new("reset").await;

    let custom_path = harness.root_dir.join("custom.sqlite");
    harness.upsert_connection(ConnectionProfile {
        id: "custom-sqlite".to_string(),
        name: "Custom SQLite".to_string(),
        engine: DbEngine::Sqlite,
        path: Some(custom_path.to_string_lossy().to_string()),
        host: None,
        port: None,
        database: None,
        username: None,
        password: None,
    });

    let list = connection::reset_connections(&harness.state)
        .await
        .expect("reset connections should succeed");

    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, sample::SAMPLE_CONNECTION_ID);

    let guard = harness
        .state
        .connections
        .read()
        .expect("state read lock should succeed");
    assert_eq!(guard.len(), 1);
    assert!(guard.contains_key(sample::SAMPLE_CONNECTION_ID));
    assert!(!guard.contains_key("custom-sqlite"));
}

#[tokio::test]
async fn sqlite_pushdown_filter_and_sort_work_on_sample_data() {
    let harness = SqliteHarness::new("pushdown").await;

    let mut request = harness.query_request("SELECT id, name FROM customers");
    request.page_size = Some(10);
    request.quick_filter = Some("a".to_string());
    request.filter_columns = Some(vec!["name".to_string()]);
    request.sort_column = Some("name".to_string());
    request.sort_direction = Some(SortDirection::Desc);

    let result = execute_query(request, &harness.state)
        .await
        .expect("pushdown query should succeed");

    let names = result
        .rows
        .iter()
        .map(|row| row.get(1).cloned().unwrap_or_default())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec!["Bao Tran".to_string(), "Alice Nguyen".to_string()]
    );
}

fn create_unique_temp_dir(test_name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!(
        "db-pro-test-{test_name}-{}-{timestamp}",
        std::process::id()
    ));
    std::fs::create_dir_all(&path).expect("temporary test directory should be created");
    path
}
