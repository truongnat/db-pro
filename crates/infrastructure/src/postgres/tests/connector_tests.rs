//! Tests for `../connector.rs` (moved out of the source file).
use super::{with_query_timeout, PostgresConnector, PostgresDialect};
use db_pro_core::application::sql_builder::{build_select, SortClause, SortDir};
use db_pro_core::domain::connection::ConnectionHandle;
use db_pro_core::domain::error::DbError;
use db_pro_core::ports::DbConnector;
use std::time::Duration;

#[test]
fn table_pagination_uses_postgres_placeholders() {
    let (sql, params) = build_select(
        &PostgresDialect,
        "public",
        "customers",
        &[],
        &[SortClause {
            column: "id".to_owned(),
            direction: SortDir::Asc,
        }],
        100,
        200,
    )
    .expect("PostgreSQL pagination should build");

    assert_eq!(
        sql,
        r#"SELECT * FROM "public"."customers" ORDER BY "id" ASC LIMIT $1 OFFSET $2"#
    );
    assert_eq!(params.len(), 2);
}

#[tokio::test]
async fn postgres_operation_timeout_returns_query_timeout() {
    let error = with_query_timeout(Duration::from_millis(1), async {
        tokio::time::sleep(Duration::from_millis(25)).await;
        Ok::<_, DbError>(())
    })
    .await
    .expect_err("operation should exceed its configured deadline");

    assert!(matches!(error, DbError::QueryTimeout { timeout_ms: 1 }));
}

/// Owned schema `execute_batch` has no live server in unit tests. This checks the
/// shipped function publishes `pg_backend_pid` and holds that guard across the statements.
fn owned_execute_batch_publishes_backend_before_statements() {
    let source = include_str!("../connector.rs");
    let start = source.find("async fn execute_batch(").expect("owned execute_batch");
    let rest = &source[start..];
    let end = rest
        .find("\n    async fn execute_transaction(")
        .expect("execute_transaction follows");
    let body = &rest[..end];
    let hold = body.find("let _batch_backend").expect("guard is held for the batch");
    let pid = body
        .find("SELECT pg_backend_pid()")
        .expect("pid is read on the batch connection");
    let track = body.find("self.track_backend(").expect("pid is published for Stop");
    let statements = body
        .find("for statement in statements")
        .expect("statements follow the publish");
    assert!(hold < pid && pid < track && track < statements);
}

#[tokio::test]
async fn recorded_batch_backend_is_what_cancel_targets() {
    owned_execute_batch_publishes_backend_before_statements();
    let connector = PostgresConnector::new();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(200))
        .connect_lazy("postgres://127.0.0.1:1/none")
        .expect("lazy pool does not connect");
    let handle_id = 7;
    connector.pools.write().await.insert(
        handle_id,
        super::PoolEntry {
            pool,
            query_timeout: Duration::from_secs(1),
            max_rows: 10,
            active_backends: std::sync::Mutex::new(Vec::new()),
            open_session: crate::transaction_session::SessionSlot::default(),
        },
    );
    let handle = ConnectionHandle::new(handle_id);
    connector
        .cancel(&handle)
        .await
        .expect("an idle batch has nothing to cancel");

    let guard = connector.track_backend(handle_id, 4242);
    assert!(guard.is_some(), "the batch must publish its backend");
    let error = connector
        .cancel(&handle)
        .await
        .expect_err("Stop must send pg_cancel_backend for the published pid");
    assert!(
        matches!(
            error,
            DbError::QueryFailed(_)
                | DbError::ConnectionFailed(_)
                | DbError::ConnectionTimeout(_)
                | DbError::QueryTimeout { .. }
        ),
        "cancel reached the server call instead of treating the batch as idle: {error:?}"
    );
    drop(guard);
    connector
        .cancel(&handle)
        .await
        .expect("the backend is dropped when the batch ends");
}

#[tokio::test]
async fn execute_batch_that_never_starts_does_not_leave_a_backend_published() {
    let connector = PostgresConnector::new();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(200))
        .connect_lazy("postgres://127.0.0.1:1/none")
        .expect("lazy pool does not connect");
    let handle_id = 8;
    connector.pools.write().await.insert(
        handle_id,
        super::PoolEntry {
            pool,
            query_timeout: Duration::from_millis(200),
            max_rows: 10,
            active_backends: std::sync::Mutex::new(Vec::new()),
            open_session: crate::transaction_session::SessionSlot::default(),
        },
    );
    let handle = ConnectionHandle::new(handle_id);
    let error = connector
        .execute_batch(&handle, &["CREATE TABLE batch_pid (id int)".to_owned()])
        .await
        .expect_err("begin cannot reach a server");
    assert!(
        matches!(
            error,
            DbError::QueryFailed(_)
                | DbError::ConnectionFailed(_)
                | DbError::ConnectionTimeout(_)
                | DbError::QueryTimeout { .. }
        ),
        "begin failed before a backend could be published: {error:?}"
    );
    connector
        .cancel(&handle)
        .await
        .expect("a batch that never started must not leave a backend for Stop");
}

#[test]
fn owned_execute_parameterized_transaction_publishes_backend_and_sets_search_path() {
    let source = include_str!("../connector.rs");
    let start = source
        .find("async fn execute_parameterized_transaction(")
        .expect("owned execute_parameterized_transaction");
    let rest = &source[start..];
    let end = rest.find("\n    async fn introspect(").expect("introspect follows");
    let body = &rest[..end];
    let hold = body
        .find("let _active_backend")
        .expect("guard is held for the parameterized transaction");
    let pid = body
        .find("SELECT pg_backend_pid()")
        .expect("pid is read on the parameterized transaction connection");
    let track = body.find("self.track_backend(").expect("pid is published for Stop");
    let search_path = body
        .find("super::search_path::set_local_search_path(")
        .expect("search path is set for the parameterized transaction");
    let statements = body
        .find("for (index, statement) in statements.iter().enumerate()")
        .expect("statements follow search path setup");
    assert!(hold < pid && pid < track && track < search_path && search_path < statements);
}

#[tokio::test]
async fn postgres_cancel_without_active_pool_returns_connection_failed() {
    let connector = PostgresConnector::new();
    let error = connector
        .cancel(&ConnectionHandle::new(1))
        .await
        .expect_err("cancel on unknown handle should fail");

    assert!(matches!(error, DbError::ConnectionFailed(_)));
}

#[test]
fn cancel_reports_backends_pg_cancel_backend_refused() {
    // `pg_cancel_backend` returning false (backend gone or role lacks pg_signal_backend)
    // must not read as a successful cancel (#481).
    let error = super::cancel_failures(vec![123, 456]).expect_err("refused pids must fail");
    let message = error.to_string();
    assert!(message.contains("123"), "names the refused pid: {message}");
    assert!(message.contains("456"), "names every refused pid: {message}");
    assert!(message.contains("cancel"), "identifies the failed action: {message}");
}

#[test]
fn cancel_with_no_refused_backends_succeeds() {
    assert!(super::cancel_failures(Vec::new()).is_ok());
}

#[test]
fn owned_cancel_reads_pg_cancel_backend_result() {
    // No live server in unit tests: check the shipped function fetches the boolean and
    // routes refused pids through the aggregation helper instead of execute().
    let source = include_str!("../connector.rs");
    let start = source.find("async fn cancel(").expect("owned cancel");
    let rest = &source[start..];
    let end = rest.find("\n    async fn ").expect("next trait fn follows");
    let body = &rest[..end];
    let scalar = body
        .find("query_scalar(\"SELECT pg_cancel_backend($1)\")")
        .expect("fetches the bool");
    let fetch = body.find("fetch_one(&pool)").expect("reads the row");
    let refused = body.find("cancel_failures(refused)").expect("aggregation is wired in");
    assert!(scalar < fetch && fetch < refused);
    assert!(!body.contains(".execute(&pool)"), "execute() hides a false result");
}
