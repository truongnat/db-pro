use super::*;

/// Transaction-control keywords fall to the classifier's fail-safe `Write` default,
/// so a script that manages its own transaction is never treated as read-only. This
/// pins the contract the multi-statement execution path depends on: `BEGIN`/`COMMIT`/
/// `ROLLBACK` are not mutations, but they must not hide one either.
#[test]
fn transaction_control_batches_classify_as_write_and_never_hide_a_mutation() {
    assert_eq!(
        classify_script_safety("BEGIN; UPDATE users SET name = 'x' WHERE id = 1; COMMIT;"),
        Some(StatementSafety::Write)
    );
    assert_eq!(
        classify_script_safety("BEGIN; SELECT 1; ROLLBACK;"),
        Some(StatementSafety::Write)
    );
    assert_eq!(
        classify_script_safety("SELECT 1; SELECT 2;"),
        Some(StatementSafety::Read)
    );
    // A destructive statement inside its own transaction is still destructive, so the
    // run-all confirmation gate sees it.
    assert_eq!(
        classify_script_safety("BEGIN; DROP TABLE users; COMMIT;"),
        Some(StatementSafety::Destructive)
    );
}

#[test]
fn classify_select_is_read() {
    assert_eq!(classify_statement_safety("SELECT 1"), Some(StatementSafety::Read));
}

#[test]
fn classify_insert_is_write() {
    assert_eq!(
        classify_statement_safety("INSERT INTO t VALUES (1)"),
        Some(StatementSafety::Write)
    );
}

#[test]
fn classify_update_is_write() {
    assert_eq!(
        classify_statement_safety("UPDATE t SET x = 1"),
        Some(StatementSafety::Write)
    );
}

#[test]
fn classify_delete_with_where_is_write() {
    assert_eq!(
        classify_statement_safety("DELETE FROM t WHERE id = 1"),
        Some(StatementSafety::Write)
    );
}

#[test]
fn classify_delete_without_where_is_destructive() {
    assert_eq!(
        classify_statement_safety("DELETE FROM t"),
        Some(StatementSafety::Destructive)
    );
}

#[test]
fn delete_identifier_containing_where_is_still_destructive() {
    assert_eq!(
        classify_statement_safety(r#"DELETE FROM \"somewhere\""#),
        Some(StatementSafety::Destructive)
    );
    assert_eq!(
        classify_statement_safety("DELETE FROM t -- WHERE id = 1"),
        Some(StatementSafety::Destructive)
    );
    assert_eq!(
        classify_statement_safety("DELETE FROM t RETURNING $tag$ WHERE $tag$"),
        Some(StatementSafety::Destructive)
    );
}

#[test]
fn classify_drop_is_destructive() {
    assert_eq!(
        classify_statement_safety("DROP TABLE t"),
        Some(StatementSafety::Destructive)
    );
}

#[test]
fn classify_truncate_is_destructive() {
    assert_eq!(
        classify_statement_safety("TRUNCATE TABLE t"),
        Some(StatementSafety::Destructive)
    );
}

#[test]
fn classify_merge_delete_as_destructive_but_update_as_write() {
    assert_eq!(
        classify_statement_safety("MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN DELETE"),
        Some(StatementSafety::Destructive)
    );
    assert_eq!(
        classify_statement_safety(
            "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN UPDATE SET value = source.value"
        ),
        Some(StatementSafety::Write)
    );
    assert_eq!(
        classify_statement_safety(
            "MERGE INTO target USING (SELECT 'DELETE' AS action) source ON target.id = 1 WHEN MATCHED THEN UPDATE SET value = 1"
        ),
        Some(StatementSafety::Write)
    );
}

#[test]
fn classify_opaque_server_side_execution_as_destructive() {
    for sql in [
        "DO $body$ BEGIN DELETE FROM users; END $body$",
        "CALL cleanup_expired_rows()",
        "EXECUTE prepared_cleanup",
    ] {
        assert_eq!(classify_statement_safety(sql), Some(StatementSafety::Destructive));
        assert!(validate_against_policy(
            sql,
            &ConnectionSafetyPolicy {
                read_only: false,
                allow_ddl: true,
                allow_destructive: false,
                max_rows: None,
                query_timeout_ms: None,
            }
        )
        .is_err());
    }
}

#[test]
fn classify_create_is_ddl() {
    assert_eq!(
        classify_statement_safety("CREATE TABLE t (id INT)"),
        Some(StatementSafety::Ddl)
    );
}

#[test]
fn classify_alter_is_ddl() {
    assert_eq!(
        classify_statement_safety("ALTER TABLE t ADD COLUMN x INT"),
        Some(StatementSafety::Ddl)
    );
}

#[test]
fn classify_with_select_is_read() {
    assert_eq!(
        classify_statement_safety("WITH cte AS (SELECT id FROM t) SELECT * FROM cte"),
        Some(StatementSafety::Read)
    );
}

#[test]
fn classify_with_update_is_write() {
    assert_eq!(
        classify_statement_safety("WITH cte AS (SELECT id FROM t) UPDATE t SET x = 1"),
        Some(StatementSafety::Write)
    );
}

#[test]
fn read_only_policy_rejects_write() {
    let policy = ConnectionSafetyPolicy::read_only();
    assert!(validate_against_policy("SELECT 1", &policy).is_ok());
    assert!(validate_against_policy("INSERT INTO t VALUES (1)", &policy).is_err());
    assert!(validate_against_policy("UPDATE t SET x = 1", &policy).is_err());
    assert!(validate_against_policy("DELETE FROM t WHERE id = 1", &policy).is_err());
}

#[test]
fn full_access_policy_allows_everything() {
    let policy = ConnectionSafetyPolicy::full_access();
    assert!(validate_against_policy("SELECT 1", &policy).is_ok());
    assert!(validate_against_policy("INSERT INTO t VALUES (1)", &policy).is_ok());
    assert!(validate_against_policy("DROP TABLE t", &policy).is_ok());
    assert!(validate_against_policy("CREATE TABLE t (id INT)", &policy).is_ok());
}

#[test]
fn no_ddl_policy_rejects_create_alter() {
    let policy = ConnectionSafetyPolicy {
        read_only: false,
        allow_ddl: false,
        allow_destructive: true,
        max_rows: None,
        query_timeout_ms: None,
    };
    assert!(validate_against_policy("SELECT 1", &policy).is_ok());
    assert!(validate_against_policy("CREATE TABLE t (id INT)", &policy).is_err());
    assert!(validate_against_policy("ALTER TABLE t ADD COLUMN x INT", &policy).is_err());
}

#[test]
fn empty_sql_is_rejected() {
    let policy = ConnectionSafetyPolicy::full_access();
    assert!(validate_against_policy("  ", &policy).is_err());
}

#[test]
fn leading_comments_stripped() {
    assert_eq!(
        classify_statement_safety("-- comment\nSELECT 1"),
        Some(StatementSafety::Read)
    );
    assert_eq!(
        classify_statement_safety("/* block */ DROP TABLE t"),
        Some(StatementSafety::Destructive)
    );
}

#[test]
fn explain_plain_select_is_read() {
    assert_eq!(
        classify_statement_safety("EXPLAIN SELECT * FROM t"),
        Some(StatementSafety::Read)
    );
}

#[test]
fn explain_analyze_substrings_in_query_do_not_change_plain_explain_safety() {
    assert_eq!(
        classify_statement_safety("EXPLAIN SELECT 'ANALYZE' AS analyze_column"),
        Some(StatementSafety::Read)
    );
}

#[test]
fn explain_analyze_allows_comments_between_keywords() {
    assert_eq!(
        classify_statement_safety("EXPLAIN /* options */ ANALYZE /* query */ DELETE FROM t WHERE id = 1"),
        Some(StatementSafety::Write)
    );
}

#[test]
fn explain_analyze_select_is_read() {
    assert_eq!(
        classify_statement_safety("EXPLAIN ANALYZE SELECT * FROM t"),
        Some(StatementSafety::Read)
    );
}

#[test]
fn explain_analyze_delete_is_write() {
    // EXPLAIN ANALYZE actually executes the statement — must NOT be Read.
    assert_eq!(
        classify_statement_safety("EXPLAIN ANALYZE DELETE FROM users WHERE id = 1"),
        Some(StatementSafety::Write)
    );
}

#[test]
fn explain_analyze_insert_is_write() {
    assert_eq!(
        classify_statement_safety("EXPLAIN ANALYZE INSERT INTO t VALUES (1)"),
        Some(StatementSafety::Write)
    );
}

#[test]
fn explain_parenthesized_analyze_classifies_the_inner_statement() {
    assert_eq!(
        classify_statement_safety("EXPLAIN (ANALYZE true, FORMAT JSON) DELETE FROM t WHERE id = 1"),
        Some(StatementSafety::Write)
    );
    assert_eq!(
        classify_statement_safety("EXPLAIN (FORMAT JSON) DELETE FROM t"),
        Some(StatementSafety::Read)
    );
}

#[test]
fn explain_analyze_with_unicode_query_text_does_not_panic() {
    assert_eq!(
        classify_statement_safety("EXPLAIN ANALYZE SELECT 'straße'"),
        Some(StatementSafety::Read)
    );
}

#[test]
fn readonly_rejects_explain_analyze_delete() {
    let policy = ConnectionSafetyPolicy::read_only();
    assert!(validate_against_policy("EXPLAIN ANALYZE DELETE FROM users WHERE id = 1", &policy).is_err());
}

#[test]
fn mutating_cte_with_delete_is_write() {
    // Data-modifying CTE: even though outer query is SELECT, the CTE mutates.
    assert_eq!(
        classify_statement_safety("WITH deleted AS (DELETE FROM users WHERE id = 1 RETURNING *) SELECT * FROM deleted"),
        Some(StatementSafety::Write)
    );
}

#[test]
fn cte_classifier_respects_lexical_boundaries_and_comment_separators() {
    assert_eq!(
        classify_statement_safety("WITH deleted AS (DELETE/* comment */FROM users RETURNING *) SELECT * FROM deleted"),
        Some(StatementSafety::Destructive)
    );
    assert_eq!(
        classify_statement_safety(
            "WITH cte (delete_col) AS (SELECT \"DELETE FROM users\", $$UPDATE users$$) SELECT * FROM cte"
        ),
        Some(StatementSafety::Read)
    );
    assert_eq!(
        classify_statement_safety(r#"WITH cte AS (SELECT E'prefix\' DELETE FROM users') SELECT * FROM cte"#),
        Some(StatementSafety::Read)
    );
    assert_eq!(
        classify_statement_safety(
            "WITH deleted AS (DELETE FROM users USING (SELECT id FROM audit WHERE id > 0) a RETURNING *) SELECT * FROM deleted"
        ),
        Some(StatementSafety::Destructive)
    );
    assert!(validate_against_policy(
        "WITH deleted AS (DELETE/* comment */FROM users RETURNING *) SELECT * FROM deleted",
        &ConnectionSafetyPolicy::read_only()
    )
    .is_err());
}

#[test]
fn destructive_delete_inside_cte_remains_destructive() {
    assert_eq!(
        classify_statement_safety("WITH deleted AS (DELETE FROM users RETURNING *) SELECT * FROM deleted"),
        Some(StatementSafety::Destructive)
    );

    let policy = ConnectionSafetyPolicy {
        read_only: false,
        allow_ddl: true,
        allow_destructive: false,
        max_rows: None,
        query_timeout_ms: None,
    };
    assert!(validate_against_policy(
        "WITH deleted AS (DELETE FROM users RETURNING *) SELECT * FROM deleted",
        &policy
    )
    .is_err());
}

#[test]
fn cte_delete_requires_a_real_where_keyword() {
    assert_eq!(
        classify_statement_safety("WITH cte AS (SELECT 1) DELETE FROM users -- WHERE id = 1"),
        Some(StatementSafety::Destructive)
    );
    assert_eq!(
        classify_statement_safety("WITH cte AS (SELECT 1) DELETE FROM users WHERE id = 1"),
        Some(StatementSafety::Write)
    );
}

#[test]
fn mutating_cte_with_insert_is_write() {
    assert_eq!(
        classify_statement_safety("WITH moved AS (INSERT INTO t2 SELECT * FROM t1 RETURNING *) SELECT * FROM moved"),
        Some(StatementSafety::Write)
    );
}

#[test]
fn readonly_rejects_mutating_cte() {
    let policy = ConnectionSafetyPolicy::read_only();
    assert!(validate_against_policy(
        "WITH deleted AS (DELETE FROM users RETURNING *) SELECT * FROM deleted",
        &policy
    )
    .is_err());
}

#[test]
fn split_statements_basic() {
    let stmts = split_statements("SELECT 1; SELECT 2");
    assert_eq!(stmts, vec!["SELECT 1", "SELECT 2"]);
}

#[test]
fn split_statements_trailing_semicolon() {
    let stmts = split_statements("SELECT 1; SELECT 2;");
    assert_eq!(stmts, vec!["SELECT 1", "SELECT 2"]);
}

#[test]
fn split_statements_respects_quotes() {
    let stmts = split_statements("SELECT ';'; SELECT 2");
    assert_eq!(stmts, vec!["SELECT ';'", "SELECT 2"]);
}

#[test]
fn split_statements_respects_comments_and_dollar_quotes() {
    let stmts = split_statements("DO $body$ BEGIN PERFORM 1; /* nested ; comment */ PERFORM 2; END $body$; SELECT 2");
    assert_eq!(stmts.len(), 2);
    assert!(stmts[0].contains("PERFORM 1;"));
    assert_eq!(stmts[1], "SELECT 2");
}

#[test]
fn split_statements_drops_comment_only_tail() {
    assert_eq!(split_statements("SELECT 1; -- trailing comment\n"), vec!["SELECT 1"]);
}

#[test]
fn split_statements_does_not_treat_identifier_dollar_signs_as_quotes() {
    let stmts = split_statements("SELECT foo$bar$; SELECT 2");
    assert_eq!(stmts, vec!["SELECT foo$bar$", "SELECT 2"]);
}

#[test]
fn split_statements_empty() {
    let stmts = split_statements("  ");
    assert!(stmts.is_empty());
}

#[test]
fn script_safety_takes_the_most_dangerous_statement() {
    assert_eq!(
        classify_script_safety("SELECT 1; DROP TABLE t;"),
        Some(StatementSafety::Destructive)
    );
    assert_eq!(
        classify_script_safety("SELECT 1; UPDATE t SET a = 1"),
        Some(StatementSafety::Write)
    );
    assert_eq!(
        classify_script_safety("DROP TABLE t; SELECT 1"),
        Some(StatementSafety::Destructive)
    );
    assert_eq!(
        classify_script_safety("SELECT 1; CREATE TABLE t (id int); SELECT 2"),
        Some(StatementSafety::Ddl)
    );
    assert_eq!(
        classify_script_safety("SELECT 1; SELECT 2;"),
        Some(StatementSafety::Read)
    );
    assert_eq!(
        classify_script_safety("DROP TABLE t"),
        Some(StatementSafety::Destructive)
    );
    assert_eq!(classify_script_safety(""), None);
    assert_eq!(classify_script_safety(";"), None);
    assert_eq!(classify_script_safety("-- only a comment"), None);
}

#[test]
fn script_safety_ignores_semicolons_inside_literals_and_comments() {
    assert_eq!(
        classify_script_safety("SELECT ';'; SELECT 2"),
        Some(StatementSafety::Read)
    );
    assert_eq!(
        classify_script_safety("SELECT 1 -- ; not a statement\n"),
        Some(StatementSafety::Read)
    );
    assert_eq!(
        classify_script_safety("DO $body$ BEGIN PERFORM 1; END $body$"),
        Some(StatementSafety::Destructive)
    );
}

#[test]
fn transaction_control_verbs_are_recognised_in_every_form() {
    for (statement, verb) in [
        ("BEGIN", "BEGIN"),
        ("BEGIN;", "BEGIN"),
        ("begin", "BEGIN"),
        ("BEGIN TRANSACTION", "BEGIN"),
        ("BEGIN IMMEDIATE", "BEGIN"),
        ("START TRANSACTION", "START TRANSACTION"),
        ("COMMIT", "COMMIT"),
        ("COMMIT WORK", "COMMIT"),
        ("END", "END"),
        ("ROLLBACK", "ROLLBACK"),
        ("ROLLBACK TO SAVEPOINT before_insert", "ROLLBACK"),
        ("ABORT", "ABORT"),
        ("SAVEPOINT s1", "SAVEPOINT"),
        ("RELEASE SAVEPOINT s1", "RELEASE"),
        ("-- note\nCOMMIT", "COMMIT"),
        ("/* note */ COMMIT", "COMMIT"),
    ] {
        assert_eq!(
            transaction_control_verb(statement),
            Some(verb),
            "statement: {statement}"
        );
    }
}

#[test]
fn transaction_control_detection_does_not_match_inside_a_statement() {
    for statement in [
        "",
        "   ",
        "SELECT 1",
        // A keyword that merely starts with a control keyword.
        "BEGINNING",
        "COMMITTED",
        "ENDLESS",
        // Control words inside a statement are not transaction control.
        "SELECT 'begin' AS word",
        "SELECT 1 AS commit",
        "SELECT commit FROM audit_log",
        "INSERT INTO t (begin) VALUES (1)",
        // A PL/pgSQL body: the statement is a `DO`, not a `BEGIN`.
        "DO $body$ BEGIN PERFORM 1; END $body$",
        // `START` alone is not `START TRANSACTION`.
        "START",
    ] {
        assert_eq!(transaction_control_verb(statement), None, "statement: {statement}");
    }
}
