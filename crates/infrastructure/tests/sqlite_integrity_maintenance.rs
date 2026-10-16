use std::sync::Arc;

use db_pro_core::domain::connection::{ConnectionConfig, ConnectionHandle, DriverType};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::monitoring::MaintenanceAction;
use db_pro_core::domain::query::CellValue;
use db_pro_core::ports::{DbConnector, MonitoringPort};
use db_pro_infrastructure::sqlite::connector::SQLiteConnector;
use db_pro_infrastructure::sqlite::monitoring::SqliteMonitoringPort;

async fn connect() -> (Arc<SQLiteConnector>, ConnectionHandle) {
    let connector = Arc::new(SQLiteConnector::new());
    let config = ConnectionConfig {
        name: "sqlite-integrity-maintenance".into(),
        driver: DriverType::SQLite,
        database: ":memory:".into(),
        ..ConnectionConfig::default()
    };
    let handle = connector.connect(&config, "").await.unwrap();
    (connector, handle)
}

#[tokio::test]
async fn sqlite_table_vacuum_is_rejected_without_database_fallback() {
    let (connector, handle) = connect().await;
    connector
        .execute(&handle, "CREATE TABLE main (id INTEGER PRIMARY KEY)", &[])
        .await
        .unwrap();
    let monitoring = SqliteMonitoringPort::new(connector.clone());
    for action in [MaintenanceAction::Vacuum, MaintenanceAction::VacuumAnalyze] {
        let error = monitoring
            .run_maintenance(&handle, None, Some("main".into()), action)
            .await
            .unwrap_err();
        assert!(matches!(error, DbError::Unsupported(_)));
    }
    connector
        .execute(&handle, "INSERT INTO main VALUES (1)", &[])
        .await
        .unwrap();
    let rows = connector.query(&handle, "SELECT id FROM main", &[]).await.unwrap();
    assert!(matches!(rows.rows[0].0[0], CellValue::Int64(1)));
    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
async fn sqlite_vacuum_analyze_also_collects_statistics() {
    let (connector, handle) = connect().await;
    connector
        .execute(
            &handle,
            "CREATE TABLE records (id INTEGER PRIMARY KEY, label TEXT)",
            &[],
        )
        .await
        .unwrap();
    connector
        .execute(&handle, "INSERT INTO records VALUES (1, 'a'), (2, 'b'), (3, 'c')", &[])
        .await
        .unwrap();
    connector
        .execute(&handle, "ATTACH DATABASE ':memory:' AS aux", &[])
        .await
        .unwrap();
    connector
        .execute(&handle, "CREATE TABLE aux.unrelated (id INTEGER)", &[])
        .await
        .unwrap();
    connector
        .execute(&handle, "INSERT INTO aux.unrelated VALUES (42)", &[])
        .await
        .unwrap();
    let monitoring = SqliteMonitoringPort::new(connector.clone());
    monitoring
        .run_maintenance(&handle, None, None, MaintenanceAction::VacuumAnalyze)
        .await
        .unwrap();
    let statistics = connector
        .query(&handle, "SELECT stat FROM sqlite_stat1 WHERE tbl = 'records'", &[])
        .await
        .unwrap();
    assert!(matches!(&statistics.rows[0].0[0], CellValue::Text(value) if value == "3"));
    let auxiliary = connector
        .query(
            &handle,
            "SELECT count(*) FROM aux.sqlite_master WHERE name = 'sqlite_stat1'",
            &[],
        )
        .await
        .unwrap();
    assert!(matches!(auxiliary.rows[0].0[0], CellValue::Int64(0)));
    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
async fn sqlite_analyze_respects_the_attached_schema_and_quoted_table() {
    let (connector, handle) = connect().await;
    connector
        .execute(&handle, "ATTACH DATABASE ':memory:' AS aux", &[])
        .await
        .unwrap();
    connector
        .execute(&handle, "CREATE TABLE main.\"quoted\"\"table\" (id INTEGER)", &[])
        .await
        .unwrap();
    connector
        .execute(&handle, "CREATE TABLE aux.\"quoted\"\"table\" (id INTEGER)", &[])
        .await
        .unwrap();
    connector
        .execute(&handle, "INSERT INTO aux.\"quoted\"\"table\" VALUES (1), (2)", &[])
        .await
        .unwrap();
    let monitoring = SqliteMonitoringPort::new(connector.clone());
    monitoring
        .run_maintenance(
            &handle,
            Some("aux".into()),
            Some("quoted\"table".into()),
            MaintenanceAction::Analyze,
        )
        .await
        .unwrap();
    let statistics = connector
        .query(
            &handle,
            "SELECT stat FROM aux.sqlite_stat1 WHERE tbl = 'quoted\"table'",
            &[],
        )
        .await
        .unwrap();
    assert!(matches!(&statistics.rows[0].0[0], CellValue::Text(value) if value == "2"));
    let untouched = connector
        .query(
            &handle,
            "SELECT count(*) FROM main.sqlite_master WHERE name = 'sqlite_stat1'",
            &[],
        )
        .await
        .unwrap();
    assert!(matches!(untouched.rows[0].0[0], CellValue::Int64(0)));
    connector.disconnect(&handle).await.unwrap();
}

#[tokio::test]
async fn fresh_sqlite_connector_enforces_foreign_keys_and_cascades() {
    let (connector, handle) = connect().await;
    let pragma = connector.query(&handle, "PRAGMA foreign_keys", &[]).await.unwrap();
    assert!(matches!(pragma.rows[0].0[0], CellValue::Int64(1)));
    connector
        .execute(&handle, "CREATE TABLE parent (id INTEGER PRIMARY KEY)", &[])
        .await
        .unwrap();
    connector
        .execute(
            &handle,
            "CREATE TABLE child (id INTEGER PRIMARY KEY, parent_id INTEGER REFERENCES parent(id) ON DELETE CASCADE)",
            &[],
        )
        .await
        .unwrap();
    assert!(matches!(
        connector
            .execute(&handle, "INSERT INTO child VALUES (1, 99)", &[])
            .await
            .unwrap_err(),
        DbError::ConstraintViolation {
            constraint_type: db_pro_core::domain::error::ConstraintType::ForeignKey,
            ..
        }
    ));
    connector
        .execute(&handle, "INSERT INTO parent VALUES (99)", &[])
        .await
        .unwrap();
    connector
        .execute(&handle, "INSERT INTO child VALUES (1, 99)", &[])
        .await
        .unwrap();
    connector
        .execute(&handle, "DELETE FROM parent WHERE id = 99", &[])
        .await
        .unwrap();
    let remaining = connector
        .query(&handle, "SELECT count(*) FROM child", &[])
        .await
        .unwrap();
    assert!(matches!(remaining.rows[0].0[0], CellValue::Int64(0)));
    connector.disconnect(&handle).await.unwrap();
}
