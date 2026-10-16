//! Public DataDiffService round-trip against independent real providers.
//! SQLite needs no fixture. PostgreSQL uses PGHOST/PGPORT/PGUSER/PGPASSWORD,
//! PGDATABASE and DATA_DIFF_TARGET_DATABASE (two distinct writable databases).

use std::sync::Arc;

use db_pro_core::application::{ConnectionRegistry, DataDiffService};
use db_pro_core::domain::connection::{ConnectionConfig, ConnectionId, DriverType, SslMode};
use db_pro_core::domain::query::QueryParam;
use db_pro_core::ports::DbConnector;
use db_pro_infrastructure::postgres::connector::PostgresConnector;
use db_pro_infrastructure::sqlite::connector::SQLiteConnector;

fn config(driver: DriverType, database: String) -> ConnectionConfig {
    ConnectionConfig {
        name: "typed-data-diff-smoke".into(),
        host: String::new(),
        port: 0,
        database,
        username: String::new(),
        driver,
        ssl_mode: SslMode::Disable,
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
    }
}

async fn round_trip(
    connector: Arc<dyn DbConnector>,
    source_config: ConnectionConfig,
    target_config: ConnectionConfig,
    password: &str,
) {
    let source = connector
        .connect(&source_config, password)
        .await
        .expect("source connect");
    let target = connector
        .connect(&target_config, password)
        .await
        .expect("target connect");
    let table = format!("data_diff_{}", uuid::Uuid::new_v4().simple());
    let qualified = connector.dialect(&target).unwrap().quote_identifier(&table);
    let create = format!("CREATE TABLE {qualified} (tenant TEXT, item TEXT, payload TEXT)");
    for handle in [&source, &target] {
        connector
            .execute(handle, &create, &[])
            .await
            .expect("create isolated fixture");
    }

    let source_rows = [
        [None, Some("stable"), None],
        [Some("∅"), Some("stable"), Some("∅")],
        [Some("a\u{1f}b"), Some("c"), Some("first")],
        [Some("a"), Some("b\u{1f}c"), Some("second")],
        [Some("insert"), Some("∅"), Some("∅")],
        [Some("same"), Some("same"), Some("same")],
        [Some("quote'"), Some("danger"), Some("x'); DELETE FROM orders; --")],
    ];
    let target_rows = [
        [None, Some("stable"), Some("∅")],
        [Some("∅"), Some("stable"), None],
        [Some("a\u{1f}b"), Some("c"), Some("old first")],
        [Some("a"), Some("b\u{1f}c"), Some("old second")],
        [None, Some("delete"), Some("obsolete")],
        [Some("same"), Some("same"), Some("same")],
        [Some("quote'"), Some("danger"), Some("old quoted")],
    ];
    for (handle, rows) in [(&source, &source_rows), (&target, &target_rows)] {
        let dialect = connector.dialect(handle).unwrap();
        let placeholders = (1..=3)
            .map(|index| dialect.placeholder(index))
            .collect::<Vec<_>>()
            .join(", ");
        let insert = format!("INSERT INTO {qualified} (tenant, item, payload) VALUES ({placeholders})");
        for row in rows {
            let params = row
                .iter()
                .map(|value| value.map_or(QueryParam::Null, |value| QueryParam::Text(value.into())))
                .collect::<Vec<_>>();
            assert_eq!(connector.execute(handle, &insert, &params).await.expect("seed"), 1);
        }
    }

    let registry = Arc::new(ConnectionRegistry::new());
    let source_id = ConnectionId::new();
    let target_id = ConnectionId::new();
    registry.register(source_id, source);
    registry.register(target_id, target);
    let service = DataDiffService::new(Box::new(Arc::clone(&connector)), registry);
    let keys = ["tenant".into(), "item".into()];
    let before = service
        .diff_table_data_keyed(&source_id, &target_id, "", &table, &keys, Some(100))
        .await
        .expect("public service diff");
    assert_eq!((before.source_row_count, before.target_row_count), (7, 7));
    assert_eq!(
        (before.added, before.removed, before.changed, before.equal),
        (1, 1, 5, 1)
    );
    assert!(!before.truncated);
    assert_eq!(before.sync_sql_preview.len(), 7);
    assert!(before
        .sync_sql_preview
        .iter()
        .any(|sql| { sql.starts_with("DELETE") && sql.contains("\"tenant\" IS NULL AND \"item\" = 'delete'") }));
    assert!(before.sync_sql_preview.iter().any(|sql| {
        sql.starts_with("UPDATE") && sql.contains("\"payload\" = '∅'") && sql.contains("\"tenant\" = '∅'")
    }));
    for sql in &before.sync_sql_preview {
        assert_eq!(connector.execute(&target, sql, &[]).await.expect(sql), 1, "{sql}");
    }
    let after = service
        .diff_table_data_keyed(&source_id, &target_id, "", &table, &keys, Some(100))
        .await
        .expect("recompare after applying preview");
    assert_eq!((after.added, after.removed, after.changed, after.equal), (0, 0, 0, 7));
    assert!(after.sync_sql_preview.is_empty());
    for handle in [&source, &target] {
        connector
            .execute(handle, &format!("DROP TABLE {qualified}"), &[])
            .await
            .expect("drop fixture");
        connector.disconnect(handle).await.expect("disconnect");
    }
}

#[tokio::test]
async fn sqlite_public_data_diff_typed_values_round_trip() {
    round_trip(
        Arc::new(SQLiteConnector::new()),
        config(DriverType::SQLite, ":memory:".into()),
        config(DriverType::SQLite, ":memory:".into()),
        "",
    )
    .await;
}

#[tokio::test]
#[ignore = "requires two distinct writable PostgreSQL databases and PG* credentials"]
async fn postgres_public_data_diff_typed_values_round_trip() {
    let database = std::env::var("PGDATABASE").expect("PGDATABASE is required");
    let target_database = std::env::var("DATA_DIFF_TARGET_DATABASE").expect("DATA_DIFF_TARGET_DATABASE is required");
    assert_ne!(
        database, target_database,
        "use two distinct disposable fixture databases"
    );
    let mut source = config(DriverType::Postgres, database);
    source.host = std::env::var("PGHOST").unwrap_or_else(|_| "127.0.0.1".into());
    source.port = std::env::var("PGPORT")
        .unwrap_or_else(|_| "5432".into())
        .parse()
        .expect("PGPORT");
    source.username = std::env::var("PGUSER").expect("PGUSER is required");
    let mut target = source.clone();
    target.database = target_database;
    round_trip(
        Arc::new(PostgresConnector::new()),
        source,
        target,
        &std::env::var("PGPASSWORD").unwrap_or_default(),
    )
    .await;
}
