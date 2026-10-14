//! Tests for `../connector.rs` (moved out of the source file).
use super::*;

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
