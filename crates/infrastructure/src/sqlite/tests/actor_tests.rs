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
