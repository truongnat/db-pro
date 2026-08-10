use db_pro_core::domain::connection::{ConnectionConfig, DriverType, SslMode};
use db_pro_core::ports::DbConnector;
use db_pro_infrastructure::sqlite::connector::SQLiteConnector;

async fn setup_sqlite_connector() -> (SQLiteConnector, db_pro_core::domain::connection::ConnectionHandle) {
    let connector = SQLiteConnector::new();
    let config = ConnectionConfig {
        name: "schema-columns-atomicity".into(),
        host: String::new(),
        port: 0,
        database: ":memory:".into(),
        username: String::new(),
        driver: DriverType::SQLite,
        ssl_mode: SslMode::Disable,
        ssh_tunnel: None,
        query_timeout_ms: 10_000,
        max_rows: 1_000,
        color: None,
        tags: vec![],
        group: None,
        readonly: false,
    };

    let handle = connector.connect(&config, "").await.unwrap();
    (connector, handle)
}

#[tokio::test]
async fn execute_batch_middle_failure_rolls_back_prior_ddl() {
    let (connector, handle) = setup_sqlite_connector().await;

    let statements = vec![
        "CREATE TABLE rollback_probe (id INTEGER PRIMARY KEY)".to_string(),
        "THIS IS NOT VALID SQL".to_string(),
        "CREATE TABLE should_never_exist (id INTEGER PRIMARY KEY)".to_string(),
    ];

    let result = connector.execute_batch(&handle, &statements).await;
    assert!(result.is_err(), "middle-statement failure must fail the batch");

    let introspection = connector.introspect(&handle).await.unwrap();
    assert!(
        !introspection.tables.iter().any(|table| table.name == "rollback_probe"),
        "the first DDL statement must be rolled back when a later statement fails"
    );
    assert!(
        !introspection
            .tables
            .iter()
            .any(|table| table.name == "should_never_exist"),
        "statements after the failure must not execute"
    );
}
