//! PostgreSQL integration tests using the fixture database.
//!
//! These tests connect to a real PostgreSQL instance with fixtures loaded.
//! The CI workflow starts a PostgreSQL 16 service container and loads
//! `fixtures/postgres/001_schema.sql` + `002_seed.sql`.
//!
//! Run locally with:
//!   DATABASE_URL=postgres://dbpro:dbpro_test@localhost:5432/dbpro_fixture \
//!   cargo test --package db-pro-infrastructure --test pg_integration -- --ignored
//!
//! Tests are marked `#[ignored]` so they only run when DATABASE_URL is set.

use db_pro_core::domain::connection::{ConnectionConfig, DriverType, SslMode};
use db_pro_core::ports::DbConnector;
use db_pro_infrastructure::postgres::connector::PostgresConnector;

/// Build a ConnectionConfig from the DATABASE_URL environment variable.
/// Returns None if DATABASE_URL is not set (tests will be skipped).
fn pg_config() -> Option<ConnectionConfig> {
    let url = std::env::var("DATABASE_URL").ok()?;
    // Parse postgres://user:pass@host:port/dbname
    let without_scheme = url.strip_prefix("postgres://")?;
    let (auth_host_db, _) = without_scheme.split_once('?').unwrap_or((&without_scheme, ""));

    // Split auth from host+db
    let (auth, host_db) = if auth_host_db.contains('@') {
        let (a, rest) = auth_host_db.rsplit_once('@').unwrap();
        (Some(a), rest)
    } else {
        (None, auth_host_db)
    };

    let (username, _password) = auth
        .map(|a| {
            let (u, p) = a.split_once(':').unwrap_or((a, ""));
            (u.to_string(), p.to_string())
        })
        .unwrap_or_else(|| ("postgres".to_string(), String::new()));

    let (host_port, database) = host_db.rsplit_once('/').unwrap_or((host_db, "postgres"));
    let (host, port) = if let Some((h, p)) = host_port.rsplit_once(':') {
        (h.to_string(), p.parse().unwrap_or(5432u16))
    } else {
        (host_port.to_string(), 5432u16)
    };

    Some(ConnectionConfig {
        name: "pg-test".into(),
        host,
        port,
        database: database.to_string(),
        username,
        driver: DriverType::Postgres,
        ssl_mode: SslMode::Disable,
        ssh_tunnel: None,
        query_timeout_ms: 30_000,
        max_rows: 10_000,
        color: None,
        tags: vec![],
        group: None,
        readonly: false,
    })
}

async fn setup() -> (PostgresConnector, db_pro_core::domain::connection::ConnectionHandle) {
    let config = pg_config().expect("DATABASE_URL must be set for PG integration tests");
    let password = std::env::var("DATABASE_URL")
        .ok()
        .and_then(|url| {
            url.strip_prefix("postgres://")
                .and_then(|s| s.split_once('@').map(|(auth, _)| auth))
                .and_then(|auth| auth.split_once(':').map(|(_, p)| p.to_string()))
        })
        .unwrap_or_default();

    let connector = PostgresConnector::new();
    let handle = connector.connect(&config, &password).await.expect("PG connect failed");
    (connector, handle)
}

// ═══════════════════════════════════════════════════════════════════════════
// Introspection tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore] // Requires DATABASE_URL
async fn pg_introspect_tables() {
    let (connector, handle) = setup().await;
    let result = connector.introspect(&handle).await.unwrap();

    let table_names: Vec<&str> = result.tables.iter().map(|t| t.name.as_str()).collect();
    assert!(table_names.contains(&"categories"), "should find categories table");
    assert!(table_names.contains(&"products"), "should find products table");
    assert!(table_names.contains(&"orders"), "should find orders table");
    assert!(table_names.contains(&"order_items"), "should find order_items table");
    assert!(table_names.contains(&"audit_logs"), "should find audit_logs table");
    assert!(table_names.contains(&"empty_table"), "should find empty_table");
    assert!(table_names.contains(&"employees"), "should find employees table");
    // Unicode and weird-name tables
    assert!(
        table_names.iter().any(|n| n.contains("Ünïcödé")),
        "should find unicode table"
    );
    assert!(
        table_names.iter().any(|n| n.contains("weird")),
        "should find weird-name table"
    );

    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn pg_introspect_triggers() {
    let (connector, handle) = setup().await;
    let result = connector.introspect(&handle).await.unwrap();

    // The fixture has a `products_updated_at` trigger.
    assert!(!result.triggers.is_empty(), "should find at least one trigger");

    let trigger = result
        .triggers
        .iter()
        .find(|t| t.name == "products_updated_at")
        .expect("fixture trigger products_updated_at should exist");

    assert_eq!(trigger.table_name, "products");
    assert_eq!(trigger.schema, "public");
    assert_eq!(trigger.timing, "BEFORE");
    assert_eq!(trigger.event, "UPDATE");
    assert!(trigger.enabled, "trigger should be enabled by default");
    // function_def should contain the CREATE FUNCTION body
    assert!(
        trigger.function_def.contains("update_timestamp"),
        "function_def should reference the trigger function"
    );

    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn pg_introspect_views() {
    let (connector, handle) = setup().await;
    let result = connector.introspect(&handle).await.unwrap();

    assert!(result.views.len() >= 2, "should find at least 2 views");
    let view_names: Vec<&str> = result.views.iter().map(|v| v.name.as_str()).collect();
    assert!(view_names.contains(&"active_users"), "should find active_users view");
    assert!(view_names.contains(&"order_summary"), "should find order_summary view");

    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn pg_introspect_indexes() {
    let (connector, handle) = setup().await;
    let result = connector.introspect(&handle).await.unwrap();

    assert!(result.indexes.len() >= 5, "should find at least 5 indexes");
    let idx_names: Vec<&str> = result.indexes.iter().map(|i| i.name.as_str()).collect();
    assert!(idx_names.contains(&"idx_products_category"));
    assert!(idx_names.contains(&"idx_orders_user"));

    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn pg_introspect_foreign_keys() {
    let (connector, handle) = setup().await;
    let result = connector.introspect(&handle).await.unwrap();

    // order_items has FKs to orders and products
    let oi_fks: Vec<_> = result
        .foreign_keys
        .iter()
        .filter(|fk| fk.from_table == "order_items")
        .collect();
    assert!(oi_fks.len() >= 2, "order_items should have at least 2 FKs");
    assert!(oi_fks
        .iter()
        .any(|fk| fk.from_column == "order_id" && fk.to_table == "orders"));
    assert!(oi_fks
        .iter()
        .any(|fk| fk.from_column == "product_id" && fk.to_table == "products"));

    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn pg_query_categories() {
    let (connector, handle) = setup().await;
    let result = connector
        .query(&handle, "SELECT * FROM categories ORDER BY id", &[])
        .await
        .unwrap();
    assert_eq!(result.row_count, 4);

    connector.disconnect(&handle).await.unwrap();
}
