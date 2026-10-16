//! Tests for `../agent.rs` (moved out of the source file).
use super::*;

#[test]
fn patch_rejects_wrong_document_and_version() {
    let patch = AgentSqlPatch {
        document_id: "doc-a".to_owned(),
        expected_version: 4,
        range: (0, 6),
        replacement: "SELECT".to_owned(),
    };
    assert_eq!(
        patch.apply_to("doc-b", 4, "select 1"),
        Err(AgentPatchError::DocumentMismatch)
    );
    assert_eq!(
        patch.apply_to("doc-a", 5, "select 1"),
        Err(AgentPatchError::StaleVersion { expected: 4, actual: 5 })
    );
}

#[test]
fn patch_is_utf8_boundary_safe() {
    let patch = AgentSqlPatch {
        document_id: "doc".to_owned(),
        expected_version: 1,
        range: (1, 2),
        replacement: "x".to_owned(),
    };
    assert_eq!(patch.apply_to("doc", 1, "é"), Err(AgentPatchError::InvalidRange));
}

#[test]
fn safety_decision_never_auto_runs_mutations() {
    assert_eq!(
        execution_decision(AgentMode::Agent, AgentSqlSafety::ReadOnly, true),
        AgentExecutionDecision::Allowed
    );
    assert_eq!(
        execution_decision(AgentMode::Agent, AgentSqlSafety::Mutating, true),
        AgentExecutionDecision::RequiresConfirmation
    );
    assert_eq!(
        execution_decision(AgentMode::Agent, AgentSqlSafety::Destructive, true),
        AgentExecutionDecision::RequiresConfirmation
    );
    assert_eq!(
        execution_decision(AgentMode::Edit, AgentSqlSafety::ReadOnly, true),
        AgentExecutionDecision::Rejected
    );
}

#[test]
fn incomplete_sql_is_confirmation_gated() {
    assert_eq!(AgentSqlSafety::classify("SELECT"), AgentSqlSafety::ReadOnly);
    assert_eq!(AgentSqlSafety::classify(""), AgentSqlSafety::Unknown);
    assert_eq!(
        execution_decision(AgentMode::Agent, AgentSqlSafety::Unknown, true),
        AgentExecutionDecision::RequiresConfirmation
    );
}

#[test]
fn batch_is_classified_by_its_most_dangerous_statement() {
    assert_eq!(
        AgentSqlSafety::classify("SELECT 1; DROP TABLE t;"),
        AgentSqlSafety::Destructive
    );
    assert_eq!(
        AgentSqlSafety::classify("SELECT 1; DELETE FROM t WHERE id = 1"),
        AgentSqlSafety::Mutating
    );
    // A whole-table UPDATE is destructive — the same gate as DELETE without WHERE.
    assert_eq!(
        AgentSqlSafety::classify("SELECT 1; UPDATE t SET a = 1"),
        AgentSqlSafety::Destructive
    );
    assert_eq!(
        AgentSqlSafety::classify("SELECT 1; UPDATE t SET a = 1 WHERE id = 2"),
        AgentSqlSafety::Mutating
    );
    assert_eq!(
        AgentSqlSafety::classify("SELECT 1; CREATE TABLE t (id int)"),
        AgentSqlSafety::Mutating
    );
    assert_eq!(AgentSqlSafety::classify("SELECT 1; SELECT 2"), AgentSqlSafety::ReadOnly);
    assert_eq!(AgentSqlSafety::classify("DROP TABLE t"), AgentSqlSafety::Destructive);
}

#[test]
fn destructive_batch_is_never_auto_runnable() {
    assert_eq!(
        execution_decision(
            AgentMode::Agent,
            AgentSqlSafety::classify("SELECT 1; DROP TABLE t;"),
            true
        ),
        AgentExecutionDecision::RequiresConfirmation
    );
    assert_eq!(
        execution_decision(AgentMode::Agent, AgentSqlSafety::classify("SELECT 1; SELECT 2"), true),
        AgentExecutionDecision::Allowed
    );
}
