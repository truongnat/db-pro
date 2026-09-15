//! Live MySQL 8 verification for the #235 provider.
//!
//! These tests are marked `#[ignore]` so they only run when `DATABASE_URL` is set
//! to a MySQL connection string (e.g. `mysql://user:pass@host:3306/db`).
//!
//! `#[ignore]` alone is not enough: the CI command is
//! `cargo test --all -- --include-ignored` with a *PostgreSQL* `DATABASE_URL`, so every test here
//! also runs in an environment with no MySQL server. `setup()` returns `None` there and the test
//! skips with a reason, instead of failing the run on a fixture it cannot have.
//!
//! Run with: `cargo test --package db-pro-infrastructure --test mysql_integration -- --ignored`

use db_pro_core::domain::connection::{ConnectionConfig, ConnectionHandle, DriverType, SslMode};
use db_pro_core::ports::{DbConnector, TransactionFailureOutcome, TransactionFailurePhase};
use db_pro_infrastructure::mysql::connector::MySqlConnector;

fn mysql_config() -> Option<ConnectionConfig> {
    let url = std::env::var("DATABASE_URL").ok()?;
    if !url.starts_with("mysql://") {
        return None;
    }
    // Parse mysql://user:pass@host:port/db
    let without_prefix = url.strip_prefix("mysql://")?;
    let (auth, rest) = without_prefix.split_once('@')?;
    let (username, _password) = auth.split_once(':').unwrap_or((auth, ""));
    let (host_port, database) = rest.split_once('/')?;
    let (host, port) = host_port.split_once(':').unwrap_or((host_port, "3306"));
    Some(ConnectionConfig {
        name: "mysql-test".into(),
        host: host.into(),
        port: port.parse().ok().unwrap_or(3306),
        database: database.split('?').next().unwrap_or(database).into(),
        username: username.into(),
        driver: DriverType::Mysql,
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

async fn setup() -> Option<(MySqlConnector, ConnectionHandle, String)> {
    let config = mysql_config()?;
    let password = std::env::var("DATABASE_URL")
        .ok()
        .and_then(|url| {
            url.strip_prefix("mysql://")
                .and_then(|s| s.split_once('@').map(|(auth, _)| auth))
                .and_then(|auth| auth.split_once(':').map(|(_, p)| p.to_string()))
        })
        .unwrap_or_default();
    let database = config.database.clone();

    let connector = MySqlConnector::new();
    let handle = connector
        .connect(&config, &password)
        .await
        .expect("MySQL connect failed");
    Some((connector, handle, database))
}

#[tokio::test]
#[ignore] // Requires DATABASE_URL=mysql://...
async fn mysql_connects_and_executes_query() {
    let Some((connector, handle, _database)) = setup().await else {
        eprintln!("skipping MySQL integration test: DATABASE_URL is not a mysql:// URL");
        return;
    };

    let result = connector
        .query(&handle, "SELECT 1 + 1 AS two", &[])
        .await
        .expect("query");
    assert!(!result.rows.is_empty());
    let col_names: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(col_names, vec!["two"]);

    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires DATABASE_URL=mysql://...
async fn mysql_execute_returns_affected_rows() {
    let Some((connector, handle, _database)) = setup().await else {
        eprintln!("skipping MySQL integration test: DATABASE_URL is not a mysql:// URL");
        return;
    };

    connector
        .execute(&handle, "DROP TABLE IF EXISTS mysql_probe", &[])
        .await
        .ok();
    connector
        .execute(&handle, "CREATE TABLE mysql_probe (id INT, name VARCHAR(100))", &[])
        .await
        .expect("create");
    let affected = connector
        .execute(&handle, "INSERT INTO mysql_probe VALUES (1, 'hello')", &[])
        .await
        .expect("insert");
    assert_eq!(affected, 1);

    connector
        .execute(&handle, "DROP TABLE IF EXISTS mysql_probe", &[])
        .await
        .ok();
    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires DATABASE_URL=mysql://...
async fn mysql_transaction_rolls_back_on_failure() {
    let Some((connector, handle, _database)) = setup().await else {
        eprintln!("skipping MySQL integration test: DATABASE_URL is not a mysql:// URL");
        return;
    };

    connector
        .execute(&handle, "DROP TABLE IF EXISTS mysql_tx_probe", &[])
        .await
        .ok();
    connector
        .execute(&handle, "CREATE TABLE mysql_tx_probe (id INT)", &[])
        .await
        .expect("create");

    let failure = connector
        .execute_transaction(
            &handle,
            &[
                "INSERT INTO mysql_tx_probe VALUES (1)".to_owned(),
                "INVALID STATEMENT".to_owned(),
            ],
            &[false, true],
        )
        .await
        .expect_err("must fail");

    assert_eq!(failure.phase, TransactionFailurePhase::Statement);
    assert_eq!(failure.outcome, TransactionFailureOutcome::RolledBack);

    let count = connector
        .query(&handle, "SELECT count(*) FROM mysql_tx_probe", &[])
        .await
        .expect("count");
    let cnt: i64 = match &count.rows[0].0[0] {
        db_pro_core::domain::query::CellValue::Int64(n) => *n,
        _ => panic!("unexpected cell"),
    };
    assert_eq!(cnt, 0, "the failed transaction must have rolled back");

    connector
        .execute(&handle, "DROP TABLE IF EXISTS mysql_tx_probe", &[])
        .await
        .ok();
    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires DATABASE_URL=mysql://...
async fn mysql_introspect_returns_canonical_model() {
    let Some((connector, handle, _database)) = setup().await else {
        eprintln!("skipping MySQL integration test: DATABASE_URL is not a mysql:// URL");
        return;
    };

    connector
        .execute(&handle, "DROP TABLE IF EXISTS mysql_intro_probe", &[])
        .await
        .ok();
    connector
        .execute(
            &handle,
            "CREATE TABLE mysql_intro_probe (id INT PRIMARY KEY, name VARCHAR(100))",
            &[],
        )
        .await
        .expect("create");

    let intro = connector.introspect(&handle).await.expect("introspect");
    assert!(intro.tables.iter().any(|t| t.name == "mysql_intro_probe"));

    connector
        .execute(&handle, "DROP TABLE IF EXISTS mysql_intro_probe", &[])
        .await
        .ok();
    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires DATABASE_URL=mysql://...
async fn mysql_explain_returns_json() {
    let Some((connector, handle, _database)) = setup().await else {
        eprintln!("skipping MySQL integration test: DATABASE_URL is not a mysql:// URL");
        return;
    };

    let plan = connector.explain(&handle, "SELECT 1").await.expect("explain");
    assert!(plan.is_array());

    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires DATABASE_URL=mysql://...
async fn mysql_positional_parameters_bind_and_round_trip() {
    use db_pro_core::domain::query::{CellValue, QueryParam};
    use db_pro_infrastructure::connector::CompositeConnector;

    let Some((_connector, _handle, _database)) = setup().await else {
        eprintln!("skipping MySQL integration test: DATABASE_URL is not a mysql:// URL");
        return;
    };
    let config = mysql_config().expect("mysql config");
    let password = std::env::var("DATABASE_URL")
        .ok()
        .and_then(|url| {
            url.strip_prefix("mysql://")
                .and_then(|s| s.split_once('@').map(|(auth, _)| auth))
                .and_then(|auth| auth.split_once(':').map(|(_, p)| p.to_string()))
        })
        .unwrap_or_default();

    let connector = CompositeConnector::new();
    let handle = connector.connect(&config, &password).await.expect("connect");

    // Dialect must resolve for an active MySQL connection (#235 criterion 5 path).
    let dialect = connector.dialect(&handle).expect("mysql dialect");
    assert_eq!(dialect.placeholder(1), "?");
    assert_eq!(dialect.quote_identifier("order"), "`order`");

    connector
        .execute(&handle, "DROP TABLE IF EXISTS mysql_bind_probe", &[])
        .await
        .ok();
    connector
        .execute(
            &handle,
            "CREATE TABLE mysql_bind_probe (
                id BIGINT NOT NULL,
                label VARCHAR(64) NOT NULL,
                amount DECIMAL(20,4) NOT NULL,
                stamp DATETIME(6) NOT NULL,
                flag BOOLEAN NOT NULL,
                payload JSON NOT NULL
            )",
            &[],
        )
        .await
        .expect("create");

    let affected = connector
        .execute(
            &handle,
            "INSERT INTO mysql_bind_probe (id, label, amount, stamp, flag, payload)
             VALUES (?, ?, ?, ?, ?, ?)",
            &[
                QueryParam::Int64(7),
                QueryParam::Text("bound".into()),
                QueryParam::Decimal("123456789012345.6789".into()),
                QueryParam::DateTime("2024-03-15T10:20:30.123456".into()),
                QueryParam::Bool(true),
                QueryParam::Json(serde_json::json!({"k": "v"})),
            ],
        )
        .await
        .expect("bound insert");
    assert_eq!(affected, 1);

    let result = connector
        .query(
            &handle,
            "SELECT id, label, amount, stamp, flag, payload
             FROM mysql_bind_probe WHERE id = ? AND label = ?",
            &[QueryParam::Int64(7), QueryParam::Text("bound".into())],
        )
        .await
        .expect("bound select");
    assert_eq!(result.row_count, 1);
    let row = &result.rows[0].0;
    assert!(matches!(row[0], CellValue::Int64(7)), "id: {:?}", row[0]);
    assert!(
        matches!(&row[1], CellValue::Text(text) if text == "bound"),
        "label: {:?}",
        row[1]
    );
    match &row[2] {
        CellValue::Decimal(digits) => assert_eq!(digits, "123456789012345.6789"),
        other => panic!("expected Decimal, got {other:?}"),
    }
    match &row[3] {
        CellValue::Timestamp(value) => assert!(value.starts_with("2024-03-15T10:20:30.123456")),
        other => panic!("expected Timestamp, got {other:?}"),
    }
    assert!(matches!(row[4], CellValue::Bool(true)), "flag: {:?}", row[4]);
    match &row[5] {
        CellValue::Json(value) => assert_eq!(value, &serde_json::json!({"k": "v"})),
        other => panic!("expected Json, got {other:?}"),
    }

    connector
        .execute(&handle, "DROP TABLE IF EXISTS mysql_bind_probe", &[])
        .await
        .ok();
    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires DATABASE_URL=mysql://...
async fn mysql_introspects_named_check_constraints() {
    let Some((connector, handle, _database)) = setup().await else {
        eprintln!("skipping MySQL integration test: DATABASE_URL is not a mysql:// URL");
        return;
    };

    connector
        .execute(&handle, "DROP TABLE IF EXISTS mysql_check_probe", &[])
        .await
        .ok();
    connector
        .execute(
            &handle,
            "CREATE TABLE mysql_check_probe (
                id BIGINT NOT NULL,
                amount DECIMAL(10,2) NOT NULL,
                CONSTRAINT chk_amount_nonneg CHECK (amount >= 0)
            )",
            &[],
        )
        .await
        .expect("create with CHECK");

    let schema = connector.introspect(&handle).await.expect("introspect");
    let checks: Vec<_> = schema
        .check_constraints
        .iter()
        .filter(|c| c.table_name == "mysql_check_probe")
        .collect();
    assert_eq!(checks.len(), 1, "expected one CHECK on mysql_check_probe, got {checks:?}");
    assert_eq!(checks[0].name, "chk_amount_nonneg");
    assert!(
        checks[0].definition.to_lowercase().contains("amount"),
        "definition should mention amount: {}",
        checks[0].definition
    );

    connector
        .execute(&handle, "DROP TABLE IF EXISTS mysql_check_probe", &[])
        .await
        .ok();
    connector.disconnect(&handle).await.unwrap();
}
