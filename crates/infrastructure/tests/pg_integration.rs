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

use db_pro_core::application::sql_builder::{build_count, FilterOp, TableFilter};
use db_pro_core::domain::connection::{ConnectionConfig, DriverType, SslMode};
use db_pro_core::domain::query::{CellValue, QueryParam};
use db_pro_core::ports::{DbConnector, TransactionFailureOutcome, TransactionFailurePhase};
use db_pro_infrastructure::postgres::connector::PostgresConnector;

/// Build a ConnectionConfig from the DATABASE_URL environment variable.
/// Returns None if DATABASE_URL is not set (tests will be skipped).
fn pg_config() -> Option<ConnectionConfig> {
    let url = std::env::var("DATABASE_URL").ok()?;
    // Parse postgres://user:pass@host:port/dbname
    let without_scheme = url.strip_prefix("postgres://")?;
    let (auth_host_db, _) = without_scheme.split_once('?').unwrap_or((without_scheme, ""));

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
        .any(|fk| fk.from_columns.contains(&"order_id".to_string()) && fk.to_table == "orders"));
    assert!(oi_fks
        .iter()
        .any(|fk| fk.from_columns.contains(&"product_id".to_string()) && fk.to_table == "products"));

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

#[tokio::test]
#[ignore]
async fn pg_query_preserves_numeric_and_enum_values() {
    let (connector, handle) = setup().await;
    let result = connector
        .query(
            &handle,
            "SELECT 12345678901234567890.12345::numeric AS amount, 'shipped'::order_status AS status",
            &[],
        )
        .await
        .unwrap();

    assert!(matches!(&result.rows[0].0[0], CellValue::Decimal(value) if value == "12345678901234567890.12345"));
    assert!(matches!(&result.rows[0].0[1], CellValue::Text(value) if value == "shipped"));

    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn pg_date_cell_filter_binds_as_date() {
    let (connector, handle) = setup().await;
    let dialect = connector.dialect(&handle).unwrap();
    let filter = TableFilter {
        column: "hire_date".into(),
        op: FilterOp::Eq,
        value: CellValue::Date("2024-01-15".into()),
    };
    let (sql, params) = build_count(dialect.as_ref(), "public", "employees", &[filter]);

    let result = connector.query(&handle, &sql, &params).await.unwrap();
    assert!(matches!(result.rows[0].0[0], CellValue::Int64(count) if count > 0));
    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn pg_query_decodes_native_temporal_and_network_values() {
    let (connector, handle) = setup().await;
    let result = connector
        .query(
            &handle,
            "SELECT TIME '12:34:56.123456' AS time_value, INTERVAL '1 day 2 hours' AS interval_value, INET '192.0.2.1/24' AS inet_value",
            &[],
        )
        .await
        .unwrap();

    assert!(matches!(&result.rows[0].0[0], CellValue::Time(value) if value == "12:34:56.123456"));
    assert!(matches!(&result.rows[0].0[1], CellValue::Interval(value) if value.contains("1 day")));
    assert!(matches!(&result.rows[0].0[2], CellValue::Inet(value) if value == "192.0.2.1/24"));
    connector.disconnect(&handle).await.unwrap();
}

/// #57 (B3): the structured and binary decoder paths hold on a live server —
/// JSON/JSONB stay structured, a UUID keeps its canonical text, BYTEA stays
/// byte-exact (a 0xde/0xff byte must not become mojibake), CIDR keeps address and
/// prefix — and a SQL NULL in any of those classes stays a null cell instead of
/// failing the row.
#[tokio::test]
#[ignore] // Requires DATABASE_URL
async fn pg_query_decodes_structured_and_binary_value_classes() {
    let (connector, handle) = setup().await;

    // Real fixture columns, including a NULL and a non-UTF8 byte sequence.
    let result = connector
        .query(
            &handle,
            "SELECT p.id, p.metadata, d.title, d.binary_data \
             FROM products p, documents d \
             WHERE p.sku = 'ELEC-001' AND d.title IN ('Binary Only', 'Empty Doc', 'README') \
             ORDER BY d.title",
            &[],
        )
        .await
        .unwrap();

    assert_eq!(result.row_count, 3);
    let rows = &result.rows;

    // UUID: canonical lowercase text, never a byte blob.
    assert!(
        matches!(&rows[0].0[0], CellValue::Uuid(value) if value == "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11"),
        "UUID must keep canonical text, got {:?}",
        rows[0].0[0]
    );
    // JSONB: structured, keyed access works.
    assert!(
        matches!(&rows[0].0[1], CellValue::Json(value) if value["brand"] == "TechCo" && value["weight_kg"] == 1.8),
        "JSONB must stay structured, got {:?}",
        rows[0].0[1]
    );

    // BYTEA: byte-exact in both directions, and a NULL stays NULL.
    assert!(
        matches!(&rows[0].0[3], CellValue::Bytes(value) if value == &vec![0xde, 0xad, 0xbe, 0xef]),
        "BYTEA must stay byte-exact, got {:?}",
        rows[0].0[3]
    );
    assert!(
        matches!(&rows[1].0[3], CellValue::Null),
        "a NULL BYTEA is a null cell, not empty bytes, got {:?}",
        rows[1].0[3]
    );
    assert!(
        matches!(&rows[2].0[3], CellValue::Bytes(value) if value == b"Hello"),
        "ASCII bytes must decode to their exact bytes, got {:?}",
        rows[2].0[3]
    );

    // Inline classes: JSON vs JSONB agree, CIDR keeps its network meaning, and a
    // SQL NULL is distinguishable from a JSON literal null.
    let classes = connector
        .query(
            &handle,
            "SELECT \
               '{\"a\":1,\"b\":[true,null]}'::json AS as_json, \
               '{\"a\":1,\"b\":[true,null]}'::jsonb AS as_jsonb, \
               '192.0.2.0/24'::cidr AS as_cidr, \
               NULL::jsonb AS as_null, \
               'null'::jsonb AS as_json_null",
            &[],
        )
        .await
        .unwrap();

    let cells = &classes.rows[0].0;
    assert!(
        matches!(&cells[0], CellValue::Json(value) if value["a"] == 1 && value["b"][0] == true),
        "JSON must decode to a structured value, got {:?}",
        cells[0]
    );
    assert!(
        matches!(&cells[1], CellValue::Json(value) if value["b"][1].is_null()),
        "JSONB must decode to a structured value, got {:?}",
        cells[1]
    );
    assert!(
        matches!(&cells[2], CellValue::Inet(value) if value == "192.0.2.0/24"),
        "CIDR must keep address and prefix, got {:?}",
        cells[2]
    );
    assert!(
        matches!(&cells[3], CellValue::Null),
        "a SQL NULL jsonb is a null cell, got {:?}",
        cells[3]
    );
    assert!(
        matches!(&cells[4], CellValue::Json(value) if value.is_null()),
        "a JSON literal null is a JSON value, not a SQL NULL, got {:?}",
        cells[4]
    );

    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn pg_typed_temporal_and_network_parameters_bind_without_casts() {
    let (connector, handle) = setup().await;
    let result = connector
        .query(
            &handle,
            "SELECT $1 = TIME '12:34:56.123456' AS time_matches, $2 = INTERVAL '1 day 2 hours' AS interval_matches, $3 = INET '192.0.2.1/24' AS inet_matches, $4 = TIMETZ '12:34:56.123456+02:00' AS timetz_matches, $5 = CIDR '192.0.2.0/24' AS cidr_matches",
            &[
                QueryParam::Time("12:34:56.123456".into()),
                QueryParam::Interval("1 days 02:00:00".into()),
                QueryParam::Inet("192.0.2.1/24".into()),
                QueryParam::Time("12:34:56.123456+02:00".into()),
                QueryParam::Inet("192.0.2.0/24".into()),
            ],
        )
        .await
        .unwrap();

    assert!(result.rows[0]
        .0
        .iter()
        .all(|cell| matches!(cell, CellValue::Bool(true))));
    connector.disconnect(&handle).await.unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════
// S7 — Gap-filling tests
// ═══════════════════════════════════════════════════════════════════════════

/// S7: Verify composite FK introspection details (column mapping, constraint identity).
#[tokio::test]
#[ignore]
async fn pg_composite_fk_detail() {
    let (connector, handle) = setup().await;
    let result = connector.introspect(&handle).await.unwrap();

    // order_items has a composite PK (order_id, product_id) with FKs to orders and products.
    let oi_fks: Vec<_> = result
        .foreign_keys
        .iter()
        .filter(|fk| fk.from_table == "order_items")
        .collect();
    assert!(oi_fks.len() >= 2, "order_items should have at least 2 FKs");

    // Verify FK to orders: order_id → id
    let fk_to_orders = oi_fks
        .iter()
        .find(|fk| fk.to_table == "orders")
        .expect("FK from order_items to orders should exist");
    assert_eq!(fk_to_orders.from_columns, vec!["order_id"]);
    assert_eq!(fk_to_orders.to_columns, vec!["id"]);
    assert_eq!(fk_to_orders.schema, "public");
    assert_eq!(fk_to_orders.to_schema, "public");

    // Verify FK to products: product_id → id
    let fk_to_products = oi_fks
        .iter()
        .find(|fk| fk.to_table == "products")
        .expect("FK from order_items to products should exist");
    assert_eq!(fk_to_products.from_columns, vec!["product_id"]);
    assert_eq!(fk_to_products.to_columns, vec!["id"]);

    // Verify composite PK is introspected correctly.
    let oi_pk = result
        .primary_keys
        .iter()
        .find(|pk| pk.table_name == "order_items")
        .expect("order_items should have a composite PK");
    assert_eq!(oi_pk.columns.len(), 2);
    assert_eq!(oi_pk.columns[0], "order_id");
    assert_eq!(oi_pk.columns[1], "product_id");

    connector.disconnect(&handle).await.unwrap();
}

/// S7: Verify index lifecycle — CREATE INDEX → introspect → DROP INDEX → verify gone.
#[tokio::test]
#[ignore]
async fn pg_index_lifecycle() {
    let (connector, handle) = setup().await;

    // Pre-cleanup: drop leftover index from a previous failed run.
    connector
        .execute(&handle, "DROP INDEX IF EXISTS idx_s7_test_lifecycle", &[])
        .await
        .unwrap();

    // Ensure the test index does not already exist.
    let result = connector.introspect(&handle).await.unwrap();
    assert!(
        !result.indexes.iter().any(|i| i.name == "idx_s7_test_lifecycle"),
        "test index should not exist before CREATE"
    );

    // CREATE INDEX.
    connector
        .execute(&handle, "CREATE INDEX idx_s7_test_lifecycle ON categories(name)", &[])
        .await
        .unwrap();

    // Introspect and verify the new index exists.
    let result = connector.introspect(&handle).await.unwrap();
    let idx = result
        .indexes
        .iter()
        .find(|i| i.name == "idx_s7_test_lifecycle")
        .expect("test index should exist after CREATE");
    assert_eq!(idx.columns, vec!["name"]);
    assert!(!idx.unique, "test index should not be unique");

    // DROP INDEX.
    connector
        .execute(&handle, "DROP INDEX idx_s7_test_lifecycle", &[])
        .await
        .unwrap();

    // Verify the index is gone.
    let result = connector.introspect(&handle).await.unwrap();
    assert!(
        !result.indexes.iter().any(|i| i.name == "idx_s7_test_lifecycle"),
        "test index should be gone after DROP"
    );

    connector.disconnect(&handle).await.unwrap();
}

/// S7: Verify special identifiers — unicode table/column, quoted names, reserved words.
#[tokio::test]
#[ignore]
async fn pg_special_identifiers() {
    let (connector, handle) = setup().await;
    let result = connector.introspect(&handle).await.unwrap();

    // Unicode table name: "Ünïcödé Üsers"
    let unicode_table = result
        .tables
        .iter()
        .find(|t| t.name.contains("Ünïcödé"))
        .expect("unicode table should exist");
    assert_eq!(unicode_table.schema, "public");

    // Unicode column: "émâil"
    let unicode_col = result
        .columns
        .iter()
        .find(|c| c.table_name.contains("Ünïcödé") && c.name == "émâil")
        .expect("unicode column should exist");
    assert!(!unicode_col.nullable, "émâil should be NOT NULL");

    // Weird-name table: "weird""name"
    let weird_table = result
        .tables
        .iter()
        .find(|t| t.name.contains("weird"))
        .expect("weird-name table should exist");
    assert_eq!(weird_table.schema, "public");

    // Column with spaces: "col with spaces"
    let spaces_col = result
        .columns
        .iter()
        .find(|c| c.table_name.contains("weird") && c.name == "col with spaces")
        .expect("column with spaces should exist");
    assert!(spaces_col.nullable);

    // Reserved word column: "SELECT"
    let reserved_col = result
        .columns
        .iter()
        .find(|c| c.table_name.contains("weird") && c.name == "SELECT")
        .expect("reserved-word column should exist");
    assert!(reserved_col.nullable);

    // Verify we can query these tables.
    let q = connector
        .query(&handle, "SELECT \"émâil\" FROM \"Ünïcödé Üsers\" ORDER BY id", &[])
        .await
        .unwrap();
    assert!(q.row_count > 0, "should be able to query unicode table");

    connector.disconnect(&handle).await.unwrap();
}

/// Core transaction contract: a failed statement must roll back an earlier
/// mutation before the connector reports the failure.
#[tokio::test]
#[ignore]
async fn pg_transaction_failure_rolls_back_prior_mutation() {
    let (connector, handle) = setup().await;
    let table = format!("core_tx_probe_{}", uuid::Uuid::new_v4().simple());
    let create = format!("CREATE TABLE \"{table}\" (id INTEGER)");
    let insert = format!("INSERT INTO \"{table}\" (id) VALUES (1)");
    let statements = vec![create, insert, "SELECT * FROM missing_core_tx_table".into()];

    let failure = connector
        .execute_transaction(&handle, &statements, &[false, false, true])
        .await
        .expect_err("the missing table must fail the transaction");

    assert_eq!(failure.statement_index, 2);
    assert_eq!(failure.phase, TransactionFailurePhase::Statement);
    assert_eq!(failure.outcome, TransactionFailureOutcome::RolledBack);
    assert_eq!(failure.results.len(), 2);
    assert!(failure.error.to_string().contains("missing_core_tx_table"));

    let relation = format!("SELECT to_regclass('public.\"{table}\"')");
    let relation_result = connector.query(&handle, &relation, &[]).await.unwrap();
    assert!(matches!(
        relation_result.rows[0].0[0],
        db_pro_core::domain::query::CellValue::Null
    ));
    let recovery = connector.query(&handle, "SELECT 1", &[]).await.unwrap();
    assert_eq!(recovery.row_count, 1);
    connector
        .execute(&handle, &format!("DROP TABLE IF EXISTS \"{table}\""), &[])
        .await
        .unwrap();
    connector.disconnect(&handle).await.unwrap();
}

/// Core transaction contract: timeout after a mutation must roll back before
/// the connector returns and leave the pool usable for a subsequent query.
#[tokio::test]
#[ignore]
async fn pg_transaction_timeout_rolls_back_prior_mutation() {
    let mut config = pg_config().expect("DATABASE_URL must be set for PG integration tests");
    config.query_timeout_ms = 1_000;
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
    let table = format!("core_tx_timeout_probe_{}", uuid::Uuid::new_v4().simple());
    let create = format!("CREATE TABLE \"{table}\" (id INTEGER)");
    let insert = format!("INSERT INTO \"{table}\" (id) VALUES (1)");
    let statements = vec![create, insert, "SELECT pg_sleep(2)".into()];

    let failure = connector
        .execute_transaction(&handle, &statements, &[false, false, true])
        .await
        .expect_err("the sleep must exceed the transaction deadline");

    assert_eq!(failure.statement_index, 2);
    assert_eq!(failure.phase, TransactionFailurePhase::Statement);
    assert_eq!(failure.outcome, TransactionFailureOutcome::RolledBack);
    assert!(matches!(
        failure.error,
        db_pro_core::domain::error::DbError::QueryTimeout { timeout_ms: 1_000 }
    ));
    assert_eq!(failure.results.len(), 2);

    let relation = format!("SELECT to_regclass('public.\"{table}\"')");
    let relation_result = connector.query(&handle, &relation, &[]).await.unwrap();
    assert!(matches!(
        relation_result.rows[0].0[0],
        db_pro_core::domain::query::CellValue::Null
    ));
    let recovery = connector.query(&handle, "SELECT 1", &[]).await.unwrap();
    assert_eq!(recovery.row_count, 1);
    connector
        .execute(&handle, &format!("DROP TABLE IF EXISTS \"{table}\""), &[])
        .await
        .unwrap();
    connector.disconnect(&handle).await.unwrap();
}

/// Core transaction contract: a deferred constraint failure at commit must be
/// reported as an unknown outcome rather than a confirmed rollback.
#[tokio::test]
#[ignore]
async fn pg_transaction_commit_failure_reports_unknown_outcome() {
    let (connector, handle) = setup().await;
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let parent = format!("core_tx_commit_parent_{suffix}");
    let child = format!("core_tx_commit_child_{suffix}");
    connector
        .execute(
            &handle,
            &format!("CREATE TABLE \"{parent}\" (id INTEGER PRIMARY KEY)"),
            &[],
        )
        .await
        .expect("parent table should be created");
    connector
        .execute(
            &handle,
            &format!(
                "CREATE TABLE \"{child}\" (parent_id INTEGER REFERENCES \"{parent}\"(id) DEFERRABLE INITIALLY DEFERRED)"
            ),
            &[],
        )
        .await
        .expect("child table should be created");

    let failure = connector
        .execute_transaction(
            &handle,
            &[
                format!("INSERT INTO \"{child}\" (parent_id) VALUES (999)"),
                "SELECT 1".into(),
            ],
            &[false, true],
        )
        .await
        .expect_err("deferred foreign-key violation must fail at commit");

    assert_eq!(failure.phase, TransactionFailurePhase::Commit);
    assert_eq!(failure.outcome, TransactionFailureOutcome::Unknown);
    assert_eq!(failure.statement_index, 2);
    assert_eq!(failure.results.len(), 2);
    assert!(failure.error.to_string().to_lowercase().contains("foreign key"));

    connector
        .execute(&handle, &format!("DROP TABLE IF EXISTS \"{child}\""), &[])
        .await
        .expect("child table cleanup should succeed");
    connector
        .execute(&handle, &format!("DROP TABLE IF EXISTS \"{parent}\""), &[])
        .await
        .expect("parent table cleanup should succeed");
    connector.disconnect(&handle).await.unwrap();
}

/// Core batch contract: a failed statement must roll back mutations that ran
/// earlier in the same batch.
#[tokio::test]
#[ignore]
async fn pg_execute_batch_failure_rolls_back_prior_mutation() {
    let (connector, handle) = setup().await;
    let table = format!("core_batch_failure_probe_{}", uuid::Uuid::new_v4().simple());
    let create = format!("CREATE TABLE \"{table}\" (id INTEGER)");
    let insert = format!("INSERT INTO \"{table}\" (id) VALUES (1)");
    let error = connector
        .execute_batch(
            &handle,
            &[create, insert, "SELECT * FROM missing_core_batch_table".into()],
        )
        .await
        .expect_err("the missing table must fail the batch");
    assert!(error.to_string().contains("missing_core_batch_table"));

    let relation = format!("SELECT to_regclass('public.\"{table}\"')");
    let relation_result = connector.query(&handle, &relation, &[]).await.unwrap();
    assert!(matches!(
        relation_result.rows[0].0[0],
        db_pro_core::domain::query::CellValue::Null
    ));
    let recovery = connector.query(&handle, "SELECT 1", &[]).await.unwrap();
    assert_eq!(recovery.row_count, 1);
    connector
        .execute(&handle, &format!("DROP TABLE IF EXISTS \"{table}\""), &[])
        .await
        .unwrap();
    connector.disconnect(&handle).await.unwrap();
}

/// Core batch contract: a timeout after a mutation must roll back before the
/// connector reports the timeout and leave the pool usable.
#[tokio::test]
#[ignore]
async fn pg_execute_batch_timeout_rolls_back_prior_mutation() {
    let mut config = pg_config().expect("DATABASE_URL must be set for PG integration tests");
    config.query_timeout_ms = 1_000;
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
    let table = format!("core_batch_timeout_probe_{}", uuid::Uuid::new_v4().simple());
    let create = format!("CREATE TABLE \"{table}\" (id INTEGER)");
    let insert = format!("INSERT INTO \"{table}\" (id) VALUES (1)");
    let error = connector
        .execute_batch(&handle, &[create, insert, "SELECT pg_sleep(2)".into()])
        .await
        .expect_err("the batch sleep must exceed its configured deadline");
    assert!(matches!(
        error,
        db_pro_core::domain::error::DbError::QueryTimeout { timeout_ms: 1_000 }
    ));

    let relation = format!("SELECT to_regclass('public.\"{table}\"')");
    let relation_result = connector.query(&handle, &relation, &[]).await.unwrap();
    assert!(matches!(
        relation_result.rows[0].0[0],
        db_pro_core::domain::query::CellValue::Null
    ));
    let recovery = connector.query(&handle, "SELECT 1", &[]).await.unwrap();
    assert_eq!(recovery.row_count, 1);
    connector
        .execute(&handle, &format!("DROP TABLE IF EXISTS \"{table}\""), &[])
        .await
        .unwrap();
    connector.disconnect(&handle).await.unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════
// Gate 5 B2 — temporal value classes decode without timezone invention (#56)
// ═══════════════════════════════════════════════════════════════════════════

/// A naive `timestamp` must keep its wall-clock reading: the decoder must not
/// reinterpret it as an instant, and the session timezone must not reach the cell.
#[tokio::test]
#[ignore] // Requires DATABASE_URL
async fn pg_timestamp_without_time_zone_keeps_wall_clock_value() {
    let (connector, handle) = setup().await;
    connector
        .execute(&handle, "SET TIME ZONE 'America/New_York'", &[])
        .await
        .unwrap();

    let result = connector
        .query(
            &handle,
            "SELECT TIMESTAMP '2024-03-15 10:20:30.123456' AS naive_stamp",
            &[],
        )
        .await
        .unwrap();

    assert_eq!(
        format!("{:?}", &result.rows[0].0[0]),
        "DateTime(\"2024-03-15T10:20:30.123456\")",
        "a naive timestamp must keep its wall-clock reading"
    );
    connector.disconnect(&handle).await.unwrap();
}

/// The whole temporal class matrix must decode to the canonical strings, and the
/// session timezone must not move any of them.
#[tokio::test]
#[ignore] // Requires DATABASE_URL
async fn pg_temporal_classes_decode_to_canonical_strings() {
    let (connector, handle) = setup().await;
    connector
        .execute(&handle, "SET TIME ZONE 'America/New_York'", &[])
        .await
        .unwrap();

    let result = connector
        .query(
            &handle,
            "SELECT DATE '2024-03-15', \
                    TIME '10:20:30.123456', \
                    TIMETZ '10:20:30.123456+05:30', \
                    TIMESTAMP '2024-03-15 10:20:30.123456', \
                    TIMESTAMPTZ '2024-03-15 10:20:30.123456+00', \
                    TIMESTAMP '2024-03-15 10:20:30', \
                    NULL::TIMESTAMP",
            &[],
        )
        .await
        .unwrap();

    let cells: Vec<String> = result.rows[0].0.iter().map(|cell| format!("{cell:?}")).collect();
    assert_eq!(
        cells,
        vec![
            "Date(\"2024-03-15\")".to_string(),
            "Time(\"10:20:30.123456\")".to_string(),
            "Time(\"10:20:30.123456+05:30\")".to_string(),
            "DateTime(\"2024-03-15T10:20:30.123456\")".to_string(),
            "DateTime(\"2024-03-15T10:20:30.123456Z\")".to_string(),
            "DateTime(\"2024-03-15T10:20:30.000000\")".to_string(),
            "Null".to_string(),
        ]
    );
    connector.disconnect(&handle).await.unwrap();
}

/// #58 (B4): a class the mapper has no explicit arm for must never be served as
/// mojibake and must never take the readable columns of the row down with it.
/// The wire format decides the representation: provider text stays text (canonical
/// enum/domain values), binary stays byte-exact (`\x` hex in the UI, read-only).
#[tokio::test]
#[ignore] // Requires DATABASE_URL
async fn pg_unsupported_binary_classes_stay_exact_and_keep_the_row_readable() {
    let (connector, handle) = setup().await;

    // Two classes that used to fail the whole query (money, a composite record),
    // one that used to come back as binary noise (a range), one array, and the
    // ordinary columns of the same row that must survive all of it.
    let result = connector
        .query(
            &handle,
            "SELECT                '[1,10)'::int4range AS range_value,                '12.34'::money AS money_value,                ROW(1, 'x')::record AS record_value,                '{a,b}'::text[] AS text_array,                'ok'::text AS note,                42 AS answer",
            &[],
        )
        .await
        .expect("an unsupported class must not fail the readable columns of the row");

    let cells = &result.rows[0].0;

    // int4range: flags 0x02 (lower bound inclusive) then big-endian bounds 1 and 10.
    match &cells[0] {
        CellValue::Bytes(bytes) => assert_eq!(
            bytes.as_slice(),
            [0x02, 0, 0, 0, 4, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0, 10],
            "a range must stay byte-exact"
        ),
        other => panic!("a range must not be decoded as text, got {other:?}"),
    }

    // money in binary form is an int64 count of cents.
    match &cells[1] {
        CellValue::Bytes(bytes) => {
            assert_eq!(bytes.len(), 8, "money is an int64 on the wire: {bytes:?}");
            let cents = i64::from_be_bytes(bytes.as_slice().try_into().expect("8 bytes"));
            assert_eq!(cents, 1234, "12.34 in cents");
        }
        other => panic!("money must not be decoded as text, got {other:?}"),
    }

    // A composite record has no flat representation: bytes, not text.
    match &cells[2] {
        CellValue::Bytes(bytes) => assert!(!bytes.is_empty(), "a record carries field data"),
        other => panic!("a record must not be decoded as text, got {other:?}"),
    }

    // Arrays arrive in the binary wire format and have no element parsing in v0.1,
    // so the fallback is byte-exact rather than an invented element rendering.
    match &cells[3] {
        CellValue::Bytes(bytes) => assert!(!bytes.is_empty(), "an array carries element data"),
        other => panic!("an array must not be decoded as text, got {other:?}"),
    }

    // The unrelated readable columns of the same row are intact.
    assert!(
        matches!(&cells[4], CellValue::Text(value) if value == "ok"),
        "the text column of the row must survive, got {:?}",
        cells[4]
    );
    assert!(
        matches!(&cells[5], CellValue::Int64(42)),
        "the integer column of the row must survive, got {:?}",
        cells[5]
    );

    connector.disconnect(&handle).await.unwrap();
}

/// #58 (B4): enum labels and domains decode deterministically — the enum label as
/// canonical text, a domain through its base type (PostgreSQL resolves the domain
/// to the underlying type on the wire), and a NULL stays NULL for both.
#[tokio::test]
#[ignore] // Requires DATABASE_URL
async fn pg_enum_and_domain_values_decode_to_canonical_values() {
    let (connector, handle) = setup().await;
    let result = connector
        .query(
            &handle,
            "SELECT                'shipped'::order_status AS enum_literal,                (SELECT status FROM orders WHERE id = 1) AS enum_column,                'YES'::information_schema.yes_or_no AS domain_over_text,                42::information_schema.cardinal_number AS domain_over_int,                NULL::information_schema.yes_or_no AS domain_null",
            &[],
        )
        .await
        .expect("enum and domain values must decode");

    let cells = &result.rows[0].0;
    assert!(
        matches!(&cells[0], CellValue::Text(value) if value == "shipped"),
        "an enum literal must keep its canonical label, got {:?}",
        cells[0]
    );
    assert!(
        matches!(&cells[1], CellValue::Text(value) if value == "delivered"),
        "an enum column must keep its canonical label, got {:?}",
        cells[1]
    );
    assert!(
        matches!(&cells[2], CellValue::Text(value) if value == "YES"),
        "a domain over text keeps canonical text, got {:?}",
        cells[2]
    );
    assert!(
        matches!(&cells[3], CellValue::Int64(42)),
        "a domain over integer decodes through its base type, got {:?}",
        cells[3]
    );
    assert!(
        matches!(&cells[4], CellValue::Null),
        "a NULL domain value is a null cell, got {:?}",
        cells[4]
    );

    connector.disconnect(&handle).await.unwrap();
}

/// #59 (B5): the decoder against the full fixture matrix — one column per value
/// class the contract defines, in `fixtures/postgres/decoder_matrix` (DDL in
/// `001_schema.sql`, row in `002_seed.sql`), asserted expected-vs-actual on a live
/// server, plus a second row that is NULL in every nullable column, so each class
/// is proven to survive a NULL.
#[tokio::test]
#[ignore] // Requires DATABASE_URL
async fn pg_decoder_matrix_covers_every_value_class() {
    let (connector, handle) = setup().await;
    let result = connector
        .query(&handle, "SELECT * FROM decoder_matrix ORDER BY id", &[])
        .await
        .expect("every fixture column must decode without unintended query failure");

    assert_eq!(result.row_count, 2, "one populated row and one NULL row");
    let row = &result.rows[0].0;
    assert_eq!(row.len(), 26, "one cell per decoder_matrix column");

    let checks: Vec<(&str, bool)> = vec![
        ("id int2", matches!(&row[0], CellValue::Int64(1))),
        ("flag bool", matches!(&row[1], CellValue::Bool(true))),
        ("count int4", matches!(&row[2], CellValue::Int64(2_147_483_647))),
        (
            "big_count int8",
            matches!(&row[3], CellValue::Int64(9_223_372_036_854_775_807)),
        ),
        (
            "ratio float4",
            matches!(&row[4], CellValue::Float64(value) if (*value - 1.5).abs() < f64::EPSILON),
        ),
        (
            "precise_ratio float8",
            matches!(&row[5], CellValue::Float64(value) if (*value - 0.1).abs() < f64::EPSILON),
        ),
        (
            "amount numeric(24,4)",
            matches!(&row[6], CellValue::Decimal(value) if value == "12345678901234567890.1234"),
        ),
        (
            "calendar_date",
            matches!(&row[7], CellValue::Date(value) if value == "2024-03-15"),
        ),
        (
            "wall_time",
            matches!(&row[8], CellValue::Time(value) if value == "10:20:30.123456"),
        ),
        (
            "zoned_time timetz",
            matches!(&row[9], CellValue::Time(value) if value == "10:20:30.123456+07:00"),
        ),
        (
            "local_stamp timestamp",
            matches!(&row[10], CellValue::DateTime(value) if value == "2024-03-15T10:20:30.123456"),
        ),
        (
            "instant timestamptz",
            matches!(&row[11], CellValue::DateTime(value) if value == "2024-03-15T10:20:30.123456Z"),
        ),
        (
            "span interval",
            matches!(&row[12], CellValue::Interval(value) if value == "1 mons 2 days 03:04:05.000006"),
        ),
        (
            "token uuid",
            matches!(&row[13], CellValue::Uuid(value) if value == "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11"),
        ),
        (
            "doc json",
            matches!(&row[14], CellValue::Json(value) if value["a"] == 1),
        ),
        (
            "payload jsonb",
            matches!(&row[15], CellValue::Json(value) if value["brand"] == "TechCo"),
        ),
        (
            "blob bytea",
            matches!(&row[16], CellValue::Bytes(value) if value == &vec![0xde, 0xad, 0xbe, 0xef]),
        ),
        (
            "address inet",
            matches!(&row[17], CellValue::Inet(value) if value.starts_with("192.0.2.1")),
        ),
        (
            "network cidr",
            matches!(&row[18], CellValue::Inet(value) if value == "192.0.2.0/24"),
        ),
        (
            "status enum",
            matches!(&row[19], CellValue::Text(value) if value == "shipped"),
        ),
        ("quantity domain", matches!(&row[20], CellValue::Int64(7))),
        (
            "postal domain",
            matches!(&row[21], CellValue::Text(value) if value == "SW1A 1AA"),
        ),
        ("labels array", matches!(&row[22], CellValue::Bytes(_))),
        ("slot range", matches!(&row[23], CellValue::Bytes(_))),
        ("pair composite", matches!(&row[24], CellValue::Bytes(_))),
        ("missing null", matches!(&row[25], CellValue::Null)),
    ];

    let wrong: Vec<&str> = checks.iter().filter(|(_, ok)| !ok).map(|(name, _)| *name).collect();
    assert!(wrong.is_empty(), "decoder matrix mismatch: {wrong:?}");

    // Provider type identity stays available to the policy layer for each column.
    let declared: Vec<&str> = result.columns.iter().map(|c| c.data_type.as_str()).collect();
    for expected in ["INT2", "NUMERIC", "TIMETZ", "TEXT[]", "INT4RANGE", "decoder_pair"] {
        assert!(
            declared.iter().any(|name| name.contains(expected)),
            "the declared type {expected} must reach the caller: {declared:?}"
        );
    }

    // Every nullable class in the second row is a null cell, not an error and not
    // empty bytes. The key and the flag are NOT NULL by DDL and keep their values.
    assert!(
        matches!(&result.rows[1].0[0], CellValue::Int64(2)),
        "the NOT NULL key column keeps its value"
    );
    assert!(
        matches!(&result.rows[1].0[1], CellValue::Bool(false)),
        "the NOT NULL flag column keeps its value"
    );
    for (index, cell) in result.rows[1].0.iter().enumerate().skip(2) {
        assert!(
            matches!(cell, CellValue::Null),
            "column {index} of the NULL row must be Null, got {cell:?}"
        );
    }

    connector.disconnect(&handle).await.unwrap();
}
