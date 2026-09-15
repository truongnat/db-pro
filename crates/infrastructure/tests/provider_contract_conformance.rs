use db_pro_core::domain::capabilities::DatabaseCapabilities;
use db_pro_core::domain::connection::{ConnectionConfig, DriverType};
use db_pro_core::domain::error::DbError;
use db_pro_core::ports::{DbConnector, ProviderFactory, TransactionFailureOutcome, TransactionFailurePhase};
use db_pro_infrastructure::connector::CompositeConnector;

fn sqlite_config() -> ConnectionConfig {
    ConnectionConfig {
        name: "test".into(),
        host: String::new(),
        port: 0,
        database: ":memory:".into(),
        username: String::new(),
        driver: DriverType::SQLite,
        ssl_mode: Default::default(),
        ssh_tunnel: None,
        query_timeout_ms: 30_000,
        max_rows: 500,
        color: None,
        tags: vec![],
        group: None,
        readonly: false,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. CompositeConnector dispatches connect/query/execute to the right factory
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn composite_connects_to_sqlite_via_factory() {
    let connector = CompositeConnector::new();
    let handle = connector.connect(&sqlite_config(), "").await.expect("connect");
    let result = connector.query(&handle, "SELECT 1 + 1", &[]).await.expect("query");
    assert!(!result.rows.is_empty());
    connector.disconnect(&handle).await.expect("disconnect");
}

#[tokio::test]
async fn composite_rejects_unknown_driver() {
    let connector = CompositeConnector::new();
    let mut config = sqlite_config();
    config.driver = DriverType::Mysql;
    let err = connector.connect(&config, "").await.expect_err("unsupported driver");
    assert!(matches!(err, DbError::Validation(_)));
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. capabilities() dispatches via factory
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn capabilities_match_driver() {
    let connector = CompositeConnector::new();
    let pg_caps = connector.capabilities(&ConnectionConfig {
        driver: DriverType::Postgres,
        ..sqlite_config()
    });
    assert!(pg_caps.schema.schemas);
    assert!(pg_caps.query.numbered_parameters);

    let sqlite_caps = connector.capabilities(&sqlite_config());
    assert!(!sqlite_caps.schema.schemas);
    assert!(sqlite_caps.query.positional_parameters);

    let mysql_caps = connector.capabilities(&ConnectionConfig {
        driver: DriverType::Mysql,
        ..sqlite_config()
    });
    assert!(mysql_caps.schema.schemas);
    assert!(!mysql_caps.features.tablespaces);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. SQLite execution path: query returns rows, execute returns affected count
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn sqlite_execute_returns_affected_rows() {
    let connector = CompositeConnector::new();
    let handle = connector.connect(&sqlite_config(), "").await.expect("connect");
    connector
        .execute(&handle, "CREATE TABLE t (id INTEGER)", &[])
        .await
        .expect("create");
    let affected = connector
        .execute(&handle, "INSERT INTO t VALUES (1)", &[])
        .await
        .expect("insert");
    assert_eq!(affected, 1);
    connector.disconnect(&handle).await.expect("disconnect");
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Transaction path: all-or-nothing rollback on failure
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn sqlite_transaction_rolls_back_on_failure() {
    let connector = CompositeConnector::new();
    let handle = connector.connect(&sqlite_config(), "").await.expect("connect");
    connector
        .execute(&handle, "CREATE TABLE txprobe (id INTEGER)", &[])
        .await
        .expect("create");

    let failure = connector
        .execute_transaction(
            &handle,
            &[
                "INSERT INTO txprobe VALUES (1)".to_owned(),
                "INVALID STATEMENT".to_owned(),
            ],
            &[false, true],
        )
        .await
        .expect_err("must fail");

    assert_eq!(failure.phase, TransactionFailurePhase::Statement);
    assert_eq!(failure.outcome, TransactionFailureOutcome::RolledBack);

    let count = connector
        .query(&handle, "SELECT count(*) FROM txprobe", &[])
        .await
        .expect("count");
    // The count should be 0 because the insert was rolled back.
    let cnt: i64 = match &count.rows[0].0[0] {
        db_pro_core::domain::query::CellValue::Int64(n) => *n,
        _ => panic!("unexpected cell"),
    };
    assert_eq!(cnt, 0, "the failed transaction must have rolled back");
    connector.disconnect(&handle).await.expect("disconnect");
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Introspection returns canonical schema model
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn sqlite_introspect_returns_canonical_model() {
    let connector = CompositeConnector::new();
    let handle = connector.connect(&sqlite_config(), "").await.expect("connect");
    connector
        .execute(&handle, "CREATE TABLE probe1 (id INTEGER PRIMARY KEY, name TEXT)", &[])
        .await
        .expect("create");
    let intro = connector.introspect(&handle).await.expect("introspect");
    assert!(!intro.tables.is_empty());
    assert!(intro.tables.iter().any(|t| t.name == "probe1"));
    connector.disconnect(&handle).await.expect("disconnect");
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Cancel is a no-op on SQLite (not supported at connector level for simple queries)
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn sqlite_cancel_is_safe() {
    let connector = CompositeConnector::new();
    let handle = connector.connect(&sqlite_config(), "").await.expect("connect");
    // Cancel on a handle with no running query should not panic or error.
    connector.cancel(&handle).await.expect("cancel is safe");
    connector.disconnect(&handle).await.expect("disconnect");
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Register factory at runtime
// ═══════════════════════════════════════════════════════════════════════════

struct StubFactory;
#[async_trait::async_trait]
impl ProviderFactory for StubFactory {
    fn driver(&self) -> DriverType {
        DriverType::Mysql
    }
    fn capabilities(&self, _config: &ConnectionConfig) -> DatabaseCapabilities {
        DatabaseCapabilities::mysql()
    }
    fn build(&self) -> Box<dyn DbConnector> {
        // This stub factory is never asked to build a connector because we only test capability dispatch.
        panic!("stub factory build should not be called in this test")
    }
    async fn test_connection(&self, _config: &ConnectionConfig, _password: &str) -> Result<(), DbError> {
        Err(DbError::Validation("stub".into()))
    }
}

#[test]
fn register_factory_enables_mysql_capabilities() {
    let mut connector = CompositeConnector::new();
    let previous = connector.register_factory(Box::new(StubFactory));
    assert!(previous.is_none(), "no previous MySQL factory");

    let caps = connector.capabilities(&ConnectionConfig {
        driver: DriverType::Mysql,
        ..sqlite_config()
    });
    assert!(!caps.features.tablespaces);
    assert!(caps.schema.schemas);
}
