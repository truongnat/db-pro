//! SQLite integration tests using the fixture database.
//!
//! These tests run against an in-memory SQLite database loaded with
//! the fixture schema and seed data from `fixtures/sqlite/fixture.sql`.
//!
//! Run with: `cargo test --package db-pro-infrastructure --test integration`

use db_pro_core::domain::connection::{ConnectionConfig, DriverType, SslMode};
use db_pro_infrastructure::sqlite::connector::SQLiteConnector;
use db_pro_core::ports::DbConnector;

/// Path to the SQLite fixture file, relative to workspace root.
const FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/sqlite/fixture.sql"
);

/// Create a SQLite connector with an in-memory fixture database.
async fn setup_fixture() -> (SQLiteConnector, db_pro_core::domain::connection::ConnectionHandle) {
    let connector = SQLiteConnector::new();
    let config = ConnectionConfig {
        name: "test-fixture".into(),
        host: String::new(),
        port: 0,
        database: ":memory:".into(),
        username: String::new(),
        driver: DriverType::SQLite,
        ssl_mode: SslMode::Disable,
        ssh_tunnel: None,
        query_timeout_ms: 30_000,
        max_rows: 10_000,
        color: None,
        tags: vec![],
        group: None,
    };

    let handle = connector.connect(&config, "").await.expect("connect failed");

    // Load fixture SQL.
    let fixture_sql = std::fs::read_to_string(FIXTURE_PATH)
        .unwrap_or_else(|e| panic!("failed to read fixture at {FIXTURE_PATH}: {e}"));

    // Split SQL into statements, respecting BEGIN...END blocks (triggers).
    for statement in split_sql_statements(&fixture_sql) {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }
        connector
            .execute(&handle, trimmed, &[])
            .await
            .unwrap_or_else(|e| panic!("fixture SQL failed for: {trimmed:?}: {e}"));
    }

    (connector, handle)
}

// ═══════════════════════════════════════════════════════════════════════════
// Connection tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn connect_and_query() {
    let (connector, handle) = setup_fixture().await;
    let result = connector.query(&handle, "SELECT 1 AS num", &[]).await.unwrap();
    assert_eq!(result.columns.len(), 1);
    assert_eq!(result.columns[0].name, "num");
    assert_eq!(result.rows.len(), 1);
    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
async fn test_connection_succeeds() {
    let connector = SQLiteConnector::new();
    let config = ConnectionConfig {
        name: "test".into(),
        host: String::new(),
        port: 0,
        database: ":memory:".into(),
        username: String::new(),
        driver: DriverType::SQLite,
        ssl_mode: SslMode::Disable,
        ssh_tunnel: None,
        query_timeout_ms: 5_000,
        max_rows: 100,
        color: None,
        tags: vec![],
        group: None,
    };
    connector.test_connection(&config, "").await.unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════
// Query tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn query_categories() {
    let (connector, handle) = setup_fixture().await;
    let result = connector
        .query(&handle, "SELECT * FROM categories ORDER BY id", &[])
        .await
        .unwrap();
    assert_eq!(result.row_count, 4);
    assert_eq!(result.columns.len(), 3); // id, name, description
}

#[tokio::test]
async fn query_products_with_json() {
    let (connector, handle) = setup_fixture().await;
    let result = connector
        .query(&handle, "SELECT name, metadata FROM products ORDER BY id LIMIT 1", &[])
        .await
        .unwrap();
    assert_eq!(result.row_count, 1);
    assert_eq!(result.columns[0].name, "name");
    assert_eq!(result.columns[1].name, "metadata");
}

#[tokio::test]
async fn query_unicode_table() {
    let (connector, handle) = setup_fixture().await;
    let result = connector
        .query(&handle, "SELECT * FROM \"Ünïcödé Üsers\" ORDER BY id", &[])
        .await
        .unwrap();
    assert_eq!(result.row_count, 4);
}

#[tokio::test]
async fn query_weird_name_table() {
    let (connector, handle) = setup_fixture().await;
    let result = connector
        .query(&handle, "SELECT * FROM \"weird\"\"name\" ORDER BY id", &[])
        .await
        .unwrap();
    assert_eq!(result.row_count, 2);
}

#[tokio::test]
async fn query_view_active_users() {
    let (connector, handle) = setup_fixture().await;
    let result = connector
        .query(&handle, "SELECT * FROM active_users ORDER BY id", &[])
        .await
        .unwrap();
    assert_eq!(result.row_count, 3); // 3 active users (carol is inactive)
}

#[tokio::test]
async fn query_view_order_summary() {
    let (connector, handle) = setup_fixture().await;
    let result = connector
        .query(&handle, "SELECT * FROM order_summary ORDER BY order_id", &[])
        .await
        .unwrap();
    assert_eq!(result.row_count, 4);
}

#[tokio::test]
async fn query_empty_table() {
    let (connector, handle) = setup_fixture().await;
    let result = connector
        .query(&handle, "SELECT * FROM empty_table", &[])
        .await
        .unwrap();
    assert_eq!(result.row_count, 0);
}

#[tokio::test]
async fn query_table_without_pk() {
    let (connector, handle) = setup_fixture().await;
    let result = connector
        .query(&handle, "SELECT * FROM audit_logs ORDER BY created_at", &[])
        .await
        .unwrap();
    assert_eq!(result.row_count, 3);
}

#[tokio::test]
async fn query_composite_pk() {
    let (connector, handle) = setup_fixture().await;
    let result = connector
        .query(&handle, "SELECT * FROM order_items ORDER BY order_id, product_id", &[])
        .await
        .unwrap();
    assert_eq!(result.row_count, 6);
}

#[tokio::test]
async fn query_with_invalid_sql() {
    let (connector, handle) = setup_fixture().await;
    let result = connector.query(&handle, "SELCT BROKEN", &[]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn query_documents_with_blob() {
    let (connector, handle) = setup_fixture().await;
    let result = connector
        .query(&handle, "SELECT title, binary_data FROM documents ORDER BY id", &[])
        .await
        .unwrap();
    assert_eq!(result.row_count, 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// Metadata / Introspection tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn introspect_tables() {
    let (connector, handle) = setup_fixture().await;
    let result = connector.introspect(&handle).await.unwrap();

    // Should find all fixture tables.
    let table_names: Vec<&str> = result.tables.iter().map(|t| t.name.as_str()).collect();
    assert!(table_names.contains(&"categories"));
    assert!(table_names.contains(&"products"));
    assert!(table_names.contains(&"orders"));
    assert!(table_names.contains(&"order_items"));
    assert!(table_names.contains(&"audit_logs"));
    assert!(table_names.contains(&"empty_table"));
    assert!(table_names.contains(&"documents"));
    assert!(table_names.contains(&"employees"));
    // Unicode and weird-name tables.
    assert!(table_names.iter().any(|n| n.contains("Ünïcödé")));
    assert!(table_names.iter().any(|n| n.contains("weird")));
}

#[tokio::test]
async fn introspect_columns_include_nullable() {
    let (connector, handle) = setup_fixture().await;
    let result = connector.introspect(&handle).await.unwrap();

    // categories.description is nullable.
    let desc_col = result.columns.iter().find(|c| c.name == "description").unwrap();
    assert!(desc_col.nullable);

    // categories.name is NOT NULL.
    let name_col = result.columns.iter().find(|c| c.name == "name" && c.table_name == "categories").unwrap();
    assert!(!name_col.nullable);
}

#[tokio::test]
async fn introspect_primary_keys() {
    let (connector, handle) = setup_fixture().await;
    let result = connector.introspect(&handle).await.unwrap();

    // categories has single-column PK.
    let cat_pk = result.primary_keys.iter().find(|pk| pk.table_name == "categories").unwrap();
    assert_eq!(cat_pk.columns, vec!["id"]);

    // order_items has composite PK.
    let oi_pk = result.primary_keys.iter().find(|pk| pk.table_name == "order_items").unwrap();
    assert_eq!(oi_pk.columns.len(), 2);

    // audit_logs has no PK.
    let has_audit_pk = result.primary_keys.iter().any(|pk| pk.table_name == "audit_logs");
    assert!(!has_audit_pk);
}

#[tokio::test]
async fn introspect_indexes() {
    let (connector, handle) = setup_fixture().await;
    let result = connector.introspect(&handle).await.unwrap();

    // Should find at least the fixture indexes.
    assert!(result.indexes.len() >= 3);
    let idx_names: Vec<&str> = result.indexes.iter().map(|i| i.name.as_str()).collect();
    assert!(idx_names.contains(&"idx_products_category"));
    assert!(idx_names.contains(&"idx_orders_user"));
}

#[tokio::test]
async fn introspect_foreign_keys() {
    let (connector, handle) = setup_fixture().await;
    let result = connector.introspect(&handle).await.unwrap();

    // orders.user_id references users.
    let order_fk = result.foreign_keys.iter().find(|fk| fk.from_table == "orders").unwrap();
    assert_eq!(order_fk.from_column, "user_id");
}

#[tokio::test]
async fn introspect_views() {
    let (connector, handle) = setup_fixture().await;
    let result = connector.introspect(&handle).await.unwrap();

    assert!(result.views.len() >= 2);
    let view_names: Vec<&str> = result.views.iter().map(|v| v.name.as_str()).collect();
    assert!(view_names.contains(&"active_users"));
    assert!(view_names.contains(&"order_summary"));
}

#[tokio::test]
async fn introspect_triggers() {
    let (connector, handle) = setup_fixture().await;
    let result = connector.introspect(&handle).await.unwrap();

    assert!(result.triggers.len() >= 1);
    assert!(result.triggers.iter().any(|t| t.name == "products_updated_at"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Data mutation tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn insert_and_query() {
    let (connector, handle) = setup_fixture().await;
    let affected = connector
        .execute(&handle, "INSERT INTO categories (name, description) VALUES ('Toys', 'Fun stuff')", &[])
        .await
        .unwrap();
    assert_eq!(affected, 1);

    let result = connector
        .query(&handle, "SELECT COUNT(*) AS cnt FROM categories", &[])
        .await
        .unwrap();
    assert_eq!(result.row_count, 1);
}

#[tokio::test]
async fn update_row() {
    let (connector, handle) = setup_fixture().await;
    let affected = connector
        .execute(&handle, "UPDATE categories SET name = 'Gadgets' WHERE id = 1", &[])
        .await
        .unwrap();
    assert_eq!(affected, 1);

    let result = connector
        .query(&handle, "SELECT name FROM categories WHERE id = 1", &[])
        .await
        .unwrap();
    // Verify the value via serialization.
    let json = serde_json::to_value(&result.rows[0].0[0]).unwrap();
    assert_eq!(json, serde_json::json!({"type": "text", "value": "Gadgets"}));
}

#[tokio::test]
async fn delete_row() {
    let (connector, handle) = setup_fixture().await;
    let affected = connector
        .execute(&handle, "DELETE FROM categories WHERE id = 4", &[])
        .await
        .unwrap();
    assert_eq!(affected, 1);

    let result = connector
        .query(&handle, "SELECT COUNT(*) AS cnt FROM categories", &[])
        .await
        .unwrap();
    assert_eq!(result.row_count, 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Explain tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn explain_select() {
    let (connector, handle) = setup_fixture().await;
    let plan = connector.explain(&handle, "SELECT * FROM categories WHERE id = 1").await.unwrap();
    // SQLite returns a JSON array of plan steps.
    assert!(plan.is_array() || plan.is_object() || plan.is_string());
}
