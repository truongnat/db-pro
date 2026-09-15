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
