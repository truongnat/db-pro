//! Tests for `../connector.rs` (moved out of the source file).
use super::*;

#[test]
fn transaction_control_statements_route_to_simple_batch() {
    for sql in [
        "BEGIN TRANSACTION",
        "begin tran",
        "  COMMIT TRANSACTION",
        "COMMIT",
        "ROLLBACK TRANSACTION",
        "rollback",
        "SAVE TRANSACTION DBPRO_PARAMETERIZED_TRANSACTION",
        "ROLLBACK TRANSACTION DBPRO_PARAMETERIZED_TRANSACTION",
        "BEGIN DISTRIBUTED TRANSACTION",
    ] {
        assert_eq!(
            sql_execution_path(sql, 0),
            SqlExecutionPath::SimpleBatch,
            "transaction control must not go through sp_executesql (error 266): {sql}"
        );
    }
}

#[test]
fn multi_statement_scripts_route_to_simple_batch() {
    assert_eq!(
        sql_execution_path("CREATE TABLE a (id int); CREATE INDEX ia ON a (id);", 0),
        SqlExecutionPath::SimpleBatch,
        "scripts must stay one server-side batch (SHOWPLAN-style session settings
         are handled by dedicated batch paths in `explain`, never through query)"
    );
    assert_eq!(
        sql_execution_path("CREATE TABLE a (id int); CREATE TABLE b (id int);", 0),
        SqlExecutionPath::SimpleBatch,
        "trailing semicolon must not turn a script into two statements"
    );
}

#[test]
fn single_statement_and_parameterized_sql_stay_on_rpc_path() {
    assert_eq!(
        sql_execution_path("SELECT CAST(@@TRANCOUNT AS bigint)", 0),
        SqlExecutionPath::ParameterizedRpc
    );
    assert_eq!(
        sql_execution_path("INSERT INTO t VALUES (@p1);", 0),
        SqlExecutionPath::ParameterizedRpc,
        "single statement with trailing semicolon stays RPC so DONE row counts survive"
    );
    assert_eq!(
        sql_execution_path("UPDATE t SET v = @p1 WHERE id = @p2", 2),
        SqlExecutionPath::ParameterizedRpc,
        "bound parameters can only travel through sp_executesql"
    );
    assert_eq!(
        sql_execution_path("BEGIN TRANSACTION", 1),
        SqlExecutionPath::ParameterizedRpc,
        "params must never be silently dropped from the wire"
    );
}

#[tokio::test]
async fn execute_transaction_reports_zero_statement_index_on_validation_failure() {
    let connector = SqlServerConnector::new();
    let handle = ConnectionHandle::new(999);
    let statements = vec!["SELECT 1".to_string(), "SELECT 2".to_string()];
    let read_statements = vec![true]; // Length mismatch: 2 statements vs 1 read flag

    let failure = connector
        .execute_transaction(&handle, &statements, &read_statements)
        .await
        .expect_err("mismatched statement lengths must fail validation");

    assert_eq!(failure.phase, TransactionFailurePhase::Validation);
    assert_eq!(
        failure.statement_index, 0,
        "statement_index must be 0 for Validation phase failure"
    );
    assert_eq!(failure.outcome, TransactionFailureOutcome::NotStarted);
}

#[tokio::test]
async fn execute_parameterized_transaction_empty_statements_and_unconnected_handle() {
    let connector = SqlServerConnector::new();
    let handle = ConnectionHandle::new(999);
    let statements: Vec<ParameterizedTransactionStatement> = vec![];

    let failure = connector
        .execute_parameterized_transaction(&handle, &statements)
        .await
        .expect_err("checking @@TRANCOUNT on disconnected handle must fail in Begin phase");

    assert_eq!(failure.phase, TransactionFailurePhase::Begin);
    assert_eq!(failure.statement_index, 0);
    assert_eq!(failure.outcome, TransactionFailureOutcome::NotStarted);
}

#[tokio::test]
async fn execute_transaction_reports_zero_statement_index_on_begin_failure() {
    let connector = SqlServerConnector::new();
    let handle = ConnectionHandle::new(999); // Handle not connected
    let statements = vec!["SELECT 1".to_string(), "SELECT 2".to_string()];
    let read_statements = vec![true, true];

    let failure = connector
        .execute_transaction(&handle, &statements, &read_statements)
        .await
        .expect_err("BEGIN TRANSACTION on unknown handle must fail");

    assert_eq!(failure.phase, TransactionFailurePhase::Begin);
    assert_eq!(
        failure.statement_index, 0,
        "statement_index must be 0 for Begin phase failure"
    );
    assert_eq!(failure.outcome, TransactionFailureOutcome::NotStarted);
}

#[tokio::test]
async fn execute_parameterized_transaction_reports_zero_statement_index_on_begin_failure() {
    let connector = SqlServerConnector::new();
    let handle = ConnectionHandle::new(999); // Handle not connected
    let statements = vec![
        ParameterizedTransactionStatement {
            sql: "INSERT INTO items VALUES (@p0)".to_string(),
            params: vec![QueryParam::Text("val".into())],
            expect_affected_rows: true,
            max_affected_rows: Some(1),
        },
        ParameterizedTransactionStatement {
            sql: "INSERT INTO items VALUES (@p0)".to_string(),
            params: vec![QueryParam::Text("val2".into())],
            expect_affected_rows: true,
            max_affected_rows: Some(1),
        },
    ];

    let failure = connector
        .execute_parameterized_transaction(&handle, &statements)
        .await
        .expect_err("BEGIN TRANSACTION on unknown handle must fail");

    assert_eq!(failure.phase, TransactionFailurePhase::Begin);
    assert_eq!(
        failure.statement_index, 0,
        "statement_index must be 0 for Begin phase failure"
    );
    assert_eq!(failure.outcome, TransactionFailureOutcome::NotStarted);
}

#[test]
fn kill_statement_formats_only_the_server_generated_spid() {
    // `KILL` rejects query parameters; the safety invariant is that the number can
    // only ever come from the `u32` decoded from `@@SPID`, never from user text.
    assert_eq!(kill_statement(52), "KILL 52");
    assert_eq!(kill_statement(u32::MAX), "KILL 4294967295");
}

#[test]
fn kill_missing_session_detection_matches_the_real_server_text_only() {
    assert!(kill_reported_missing_session(
        "Cannot find process ID with SPID 52. Msg 226, Level 16, State 1, Server x, Line 1"
    ));
    assert!(!kill_reported_missing_session(
        "Cannot KILL a system session. Msg 221, Level 16, State 1"
    ));
    assert!(!kill_reported_missing_session("permission denied"));
}

#[test]
fn active_statement_guard_counts_statements_including_on_drop() {
    let active = AtomicUsize::new(0);
    let _first = ActiveSqlServer::new(&active);
    assert_eq!(
        active.load(Ordering::SeqCst),
        1,
        "a running statement is visible to Stop"
    );
    {
        let _second = ActiveSqlServer::new(&active);
        assert_eq!(active.load(Ordering::SeqCst), 2);
    }
    assert_eq!(
        active.load(Ordering::SeqCst),
        1,
        "the guard decrements when a statement future is dropped, including on timeout"
    );
}

#[tokio::test]
async fn cancel_on_unknown_handle_reports_connection_failed_instead_of_unsupported() {
    let connector = SqlServerConnector::new();
    let error = connector
        .cancel(&ConnectionHandle::new(999))
        .await
        .expect_err("cancelling a handle that was never connected must fail");
    assert!(
        matches!(error, DbError::ConnectionFailed(_)),
        "cancel must be a supported port method now, not a capability lie: {error:?}"
    );
}

#[tokio::test]
async fn query_on_unknown_handle_fails_fast_instead_of_hanging() {
    // The deadline wraps a session lookup; an unknown handle must fail with the
    // stable ConnectionFailed message (guards the client() -> session() refactor).
    let connector = SqlServerConnector::new();
    let error = connector
        .query(&ConnectionHandle::new(999), "SELECT 1", &[])
        .await
        .expect_err("unknown handle must fail fast");
    assert!(matches!(error, DbError::ConnectionFailed(_)), "got {error:?}");
}
