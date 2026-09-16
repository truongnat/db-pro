//! Live SQL Server verification for issue #260.
//!
//! The tests are intentionally ignored in the normal workspace run. They need
//! a real SQL Server fixture and a `SQLSERVER_URL`, for example:
//!
//!   SQLSERVER_URL='sqlserver://sa:Pass%40123@127.0.0.1:1433/dbpro_fixture' \
//!   cargo test -p db-pro-infrastructure --test sqlserver_integration -- --ignored

use db_pro_core::domain::cloud_presets::parse_connection_snippet;
use db_pro_core::domain::connection::{ConnectionConfig, ConnectionHandle, DriverType, SslMode};
use db_pro_core::domain::query::{CellValue, QueryParam};
use db_pro_core::ports::{DbConnector, TransactionFailureOutcome, TransactionFailurePhase};
use db_pro_infrastructure::sqlserver::connector::SqlServerConnector;

fn sqlserver_config() -> Option<(ConnectionConfig, String)> {
    let raw = std::env::var("SQLSERVER_URL").ok()?;
    let parsed = parse_connection_snippet(&raw).ok()?;
    if parsed.driver != DriverType::SqlServer {
        return None;
    }
    let password = parsed.password.unwrap_or_default();
    Some((
        ConnectionConfig {
            name: "sqlserver-test".into(),
            host: parsed.host,
            port: parsed.port,
            database: parsed.database,
            username: parsed.username,
            driver: DriverType::SqlServer,
            ssl_mode: parsed.ssl_mode.unwrap_or(SslMode::Require),
            ssh_tunnel: None,
            ssh_profile_id: None,
            ssl_root_cert_path: None,
            ssl_client_cert_path: None,
            ssl_client_key_path: None,
            query_timeout_ms: 30_000,
            max_rows: 10_000,
            color: None,
            tags: vec![],
            group: None,
            favorite: false,
            environment: Default::default(),
            readonly: false,
        },
        password,
    ))
}

async fn setup() -> Option<(SqlServerConnector, ConnectionHandle)> {
    let (config, password) = sqlserver_config()?;
    let connector = SqlServerConnector::new();
    let handle = connector.connect(&config, &password).await.ok()?;
    Some((connector, handle))
}

#[tokio::test]
#[ignore = "requires SQLSERVER_URL and a live SQL Server fixture"]
async fn sqlserver_connects_and_queries() {
    let Some((connector, handle)) = setup().await else {
        eprintln!("skipping SQL Server integration test: SQLSERVER_URL is unavailable");
        return;
    };

    let result = connector
        .query(&handle, "SELECT CAST(1 AS bigint) AS one", &[])
        .await
        .expect("SQL Server query");
    assert_eq!(result.columns[0].name, "one");
    assert!(matches!(result.rows[0].0[0], CellValue::Int64(1)));
    connector.disconnect(&handle).await.expect("disconnect");
}

#[tokio::test]
#[ignore = "requires SQLSERVER_URL and a live SQL Server fixture"]
async fn sqlserver_preserves_parameterized_decimal_and_bigint() {
    let Some((connector, handle)) = setup().await else {
        eprintln!("skipping SQL Server integration test: SQLSERVER_URL is unavailable");
        return;
    };

    let result = connector
        .query(
            &handle,
            "SELECT CAST(@p0 AS decimal(38, 10)) AS exact_decimal, CAST(@p1 AS bigint) AS exact_bigint",
            &[
                QueryParam::Decimal("123456789012345678.1234567890".into()),
                QueryParam::Int64(9_223_372_036_854_775_000),
            ],
        )
        .await
        .expect("SQL Server parameterized query");
    assert!(matches!(
        &result.rows[0].0[0],
        CellValue::Decimal(value) if value == "123456789012345678.1234567890"
    ));
    assert!(matches!(
        result.rows[0].0[1],
        CellValue::Int64(9_223_372_036_854_775_000)
    ));
    connector.disconnect(&handle).await.expect("disconnect");
}

#[tokio::test]
#[ignore = "requires SQLSERVER_URL and a live SQL Server fixture"]
async fn sqlserver_introspects_catalog() {
    let Some((connector, handle)) = setup().await else {
        eprintln!("skipping SQL Server integration test: SQLSERVER_URL is unavailable");
        return;
    };

    let result = connector.introspect(&handle).await.expect("SQL Server introspection");
    assert!(!result.schemas.is_empty(), "SQL Server fixture should expose schemas");
    connector.disconnect(&handle).await.expect("disconnect");
}

#[tokio::test]
#[ignore = "requires SQLSERVER_URL and a live SQL Server fixture"]
async fn sqlserver_transaction_rolls_back_on_failure() {
    let Some((connector, handle)) = setup().await else {
        eprintln!("skipping SQL Server integration test: SQLSERVER_URL is unavailable");
        return;
    };

    connector
        .execute(&handle, "DROP TABLE IF EXISTS dbo.dbpro_transaction_probe", &[])
        .await
        .ok();
    connector
        .execute(
            &handle,
            "CREATE TABLE dbo.dbpro_transaction_probe (id bigint NOT NULL)",
            &[],
        )
        .await
        .expect("create transaction probe");
    let failure = connector
        .execute_transaction(
            &handle,
            &[
                "INSERT INTO dbo.dbpro_transaction_probe VALUES (1)".into(),
                "INSERT INTO dbo.dbpro_transaction_probe (missing_column) VALUES (2)".into(),
            ],
            &[false, false],
        )
        .await
        .expect_err("transaction should fail");
    assert_eq!(failure.phase, TransactionFailurePhase::Statement);
    assert_eq!(failure.outcome, TransactionFailureOutcome::RolledBack);
    connector
        .execute(&handle, "DROP TABLE dbo.dbpro_transaction_probe", &[])
        .await
        .expect("drop transaction probe");
    connector.disconnect(&handle).await.expect("disconnect");
}
