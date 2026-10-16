//! Tests for `../actor.rs` (moved out of the source file).
use super::*;

fn memory_actor() -> SqliteActor {
    SqliteActor {
        conn: rusqlite::Connection::open_in_memory().expect("in-memory sqlite should open"),
    }
}

#[test]
fn batch_refuses_transaction_control_statements() {
    let actor = memory_actor();
    // A COMMIT inside the batch would escape the wrapping transaction and make
    // rollback a no-op — it must be refused before any statement runs (#359).
    let err = actor
        .handle_execute_batch(&[
            "CREATE TABLE a(x)".to_owned(),
            "COMMIT".to_owned(),
            "DROP TABLE a".to_owned(),
        ])
        .expect_err("batch containing COMMIT must be refused");
    assert!(err.to_string().contains("COMMIT"));
    // Nothing was executed: the table does not exist.
    let after = actor.handle_execute_batch(&["SELECT 1 FROM a".to_owned()]);
    assert!(after.is_err());
}

#[test]
fn batch_refuses_begin_and_savepoint() {
    let actor = memory_actor();
    for control in ["BEGIN", "SAVEPOINT s1", "ROLLBACK"] {
        let err = actor
            .handle_execute_batch(&["CREATE TABLE t(x)".to_owned(), control.to_owned()])
            .expect_err("transaction-control statements must be refused");
        assert!(err.to_string().contains("batch refused"));
    }
}

#[test]
fn batch_without_transaction_control_commits_atomically() {
    let actor = memory_actor();
    let total = actor
        .handle_execute_batch(&[
            "CREATE TABLE a(x INTEGER)".to_owned(),
            "INSERT INTO a VALUES (1), (2), (3)".to_owned(),
        ])
        .expect("ordinary batch should succeed");
    assert_eq!(total, 3);
}

#[test]
fn sqlite_connection_sets_a_finite_busy_timeout() {
    let conn = super::open_connection(":memory:").expect("in-memory connection should open");
    let timeout_ms: i64 = conn
        .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
        .expect("busy timeout pragma should be readable");
    assert_eq!(timeout_ms, super::SQLITE_BUSY_TIMEOUT.as_millis() as i64);
}

#[test]
fn actor_reports_handler_panic_and_processes_following_command() {
    let (sender, receiver) = std::sync::mpsc::channel();
    let actor = memory_actor();
    let worker = std::thread::spawn(move || actor.run(receiver));

    let (panic_tx, panic_rx) = tokio::sync::oneshot::channel();
    sender
        .send(SqliteCommand::PanicForTest { responder: panic_tx })
        .expect("panic test command should queue");
    let panic_result = panic_rx.blocking_recv().expect("panic result should arrive");
    assert!(matches!(panic_result, Err(DbError::Internal(message)) if message.contains("test handler")));

    let (query_tx, query_rx) = tokio::sync::oneshot::channel();
    sender
        .send(SqliteCommand::Execute {
            sql: "SELECT 1".to_owned(),
            params: Vec::new(),
            max_rows: 10,
            responder: query_tx,
        })
        .expect("query should queue after panic");
    let query_result = query_rx
        .blocking_recv()
        .expect("actor should respond after panic")
        .expect("query should succeed after panic");
    assert_eq!(query_result.rows.len(), 1);

    sender.send(SqliteCommand::Shutdown).expect("shutdown should queue");
    worker.join().expect("actor worker should exit cleanly");
}

#[test]
fn transaction_handler_panic_reports_unknown_outcome() {
    let failure = super::catch_transaction_handler_panic(
        "test transaction",
        || -> Result<Vec<db_pro_core::ports::TransactionStatementResult>, db_pro_core::ports::TransactionFailure> {
            panic!("simulated transaction panic")
        },
    )
    .expect_err("panic should become a transaction failure");

    assert_eq!(failure.phase, db_pro_core::ports::TransactionFailurePhase::Actor);
    assert_eq!(failure.outcome, db_pro_core::ports::TransactionFailureOutcome::Unknown);
    assert!(failure.error.to_string().contains("outcome is unknown"));
}
