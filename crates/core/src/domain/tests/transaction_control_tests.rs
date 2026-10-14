//! Tests for `../transaction_control.rs` (moved out of the source file).
use super::{transaction_control, TransactionControl};

#[test]
fn simple_controls_and_begin_modes_are_recognized() {
    assert_eq!(transaction_control("BEGIN"), Some(TransactionControl::Begin));
    assert_eq!(transaction_control("  begin ; "), Some(TransactionControl::Begin));
    assert_eq!(
        transaction_control("START TRANSACTION"),
        Some(TransactionControl::Begin)
    );
    assert_eq!(
        transaction_control("BEGIN ISOLATION LEVEL SERIALIZABLE"),
        Some(TransactionControl::Begin)
    );
    assert_eq!(transaction_control("COMMIT WORK"), Some(TransactionControl::Commit));
    assert_eq!(transaction_control("ROLLBACK"), Some(TransactionControl::Rollback));
}

#[test]
fn savepoints_and_chained_commit_are_not_session_edges() {
    assert_eq!(transaction_control("ROLLBACK TO SAVEPOINT s"), None);
    assert_eq!(transaction_control("COMMIT AND CHAIN"), None);
    assert_eq!(transaction_control("SELECT 1"), None);
}
