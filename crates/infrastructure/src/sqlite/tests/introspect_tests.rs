//! Tests for `../introspect.rs` (moved out of the source file).
use super::*;

#[test]
fn introspection_resolves_shorthand_foreign_key_to_parent_primary_key() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE parent (id INTEGER PRIMARY KEY, name TEXT);\
             CREATE TABLE child (id INTEGER PRIMARY KEY, parent_id INTEGER REFERENCES parent);",
    )
    .unwrap();

    let result = run_introspection(&conn).unwrap();
    let foreign_key = result
        .foreign_keys
        .iter()
        .find(|fk| fk.from_table == "child")
        .expect("child shorthand foreign key should be introspected");

    assert_eq!(foreign_key.from_columns, vec!["parent_id"]);
    assert_eq!(foreign_key.to_table, "parent");
    assert_eq!(foreign_key.to_columns, vec!["id"]);
}

#[test]
fn introspection_extracts_each_nested_check_constraint() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute(
        // cc-scan:allow LINE_TOO_LONG — literal must not wrap
        "CREATE TABLE accounts (id INTEGER, balance INTEGER CHECK (balance >= 0), name TEXT CHECK (length(name) > 0))",
        [],
    )
    .unwrap();

    let result = run_introspection(&conn).unwrap();

    assert_eq!(result.check_constraints.len(), 2);
    assert_eq!(result.check_constraints[0].definition, "balance >= 0");
    assert_eq!(result.check_constraints[1].definition, "length(name) > 0");
}

#[test]
fn introspection_ignores_check_text_in_literals_and_comments() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute(
            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
            "CREATE TABLE accounts (description TEXT DEFAULT 'CHECK (ignored)', balance INTEGER CHECK /* keep */ ((balance >= 0) AND (balance <= 100)), note TEXT -- CHECK (ignored)\n)",
            [],
        )
        .unwrap();

    let result = run_introspection(&conn).unwrap();

    assert_eq!(result.check_constraints.len(), 1);
    assert_eq!(
        result.check_constraints[0].definition,
        "(balance >= 0) AND (balance <= 100)"
    );
}

#[test]
fn introspection_preserves_named_check_constraints_and_escaped_quotes() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE people (
                id INTEGER,
                name TEXT,
                age INTEGER CHECK (age >= 0),
                CONSTRAINT chk_name CHECK (name <> 'O''Brien')
            )",
        [],
    )
    .unwrap();

    let result = run_introspection(&conn).unwrap();
    assert_eq!(result.check_constraints.len(), 2);
    assert_eq!(result.check_constraints[0].name, "people_check_0");
    assert_eq!(result.check_constraints[0].definition, "age >= 0");
    assert_eq!(result.check_constraints[1].name, "chk_name");
    assert_eq!(result.check_constraints[1].definition, "name <> 'O''Brien'");
}

#[test]
fn parse_check_constraints_stops_safely_on_unbalanced_parentheses() {
    let parsed = parse_check_constraints("CREATE TABLE t (id INTEGER CHECK (id > 0), bad CHECK ((id > 1)");
    // The first complete CHECK is kept; the truncated one must not be fabricated.
    assert_eq!(
        parsed,
        vec![ParsedCheckConstraint {
            name: None,
            definition: "id > 0".into(),
        }]
    );
}

#[test]
fn parse_check_constraints_is_case_insensitive_and_ordered() {
    let parsed = parse_check_constraints("CREATE TABLE t (a INT check (a > 0), CONSTRAINT Chk_B Check (b < 10))");
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].definition, "a > 0");
    assert_eq!(parsed[1].name.as_deref(), Some("Chk_B"));
    assert_eq!(parsed[1].definition, "b < 10");
}

#[test]
fn introspection_marks_unique_constraint_indexes_without_reusing_internal_names() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT UNIQUE);\
             CREATE INDEX idx_users_email ON users(email);",
    )
    .unwrap();

    let result = run_introspection(&conn).unwrap();

    assert!(result.indexes.iter().any(|index| index.name == "idx_users_email"));
    let unique_constraint = result
        .indexes
        .iter()
        .find(|index| index.name.starts_with("sqlite_autoindex_"))
        .expect("SQLite should expose the unique constraint origin");
    assert_eq!(unique_constraint.origin, IndexOrigin::UniqueConstraint);
    assert!(unique_constraint.unique);
    assert_eq!(unique_constraint.columns, vec!["email"]);
}

#[test]
fn introspection_omits_primary_key_autoindexes() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE composite_keys (left_id INTEGER, right_id INTEGER,\
             PRIMARY KEY (left_id, right_id));",
    )
    .unwrap();

    let result = run_introspection(&conn).unwrap();

    assert!(result.indexes.is_empty());
    assert_eq!(result.primary_keys[0].columns, vec!["left_id", "right_id"]);
}

#[test]
fn parse_standard_after_insert_trigger() {
    let sql = "CREATE TRIGGER tr_audit AFTER INSERT ON users BEGIN SELECT 1; END";
    let (timing, event) = parse_sqlite_trigger_sql(sql);
    assert_eq!(timing, "AFTER");
    assert_eq!(event, "INSERT");
}

#[test]
fn parse_before_update_trigger() {
    let sql = "CREATE TRIGGER tr_validate BEFORE UPDATE ON orders BEGIN SELECT 1; END";
    let (timing, event) = parse_sqlite_trigger_sql(sql);
    assert_eq!(timing, "BEFORE");
    assert_eq!(event, "UPDATE");
}

#[test]
fn parse_instead_of_delete_trigger() {
    let sql = "CREATE TRIGGER tr_block INSTEAD OF DELETE ON users BEGIN SELECT 1; END";
    let (timing, event) = parse_sqlite_trigger_sql(sql);
    assert_eq!(timing, "INSTEAD OF");
    assert_eq!(event, "DELETE");
}

#[test]
fn parse_trigger_with_begin_in_name() {
    // The trigger name contains "BEGIN" which should NOT be treated as the keyword.
    let sql = "CREATE TRIGGER trg_BEGIN_audit AFTER INSERT ON users BEGIN SELECT 1; END";
    let (timing, event) = parse_sqlite_trigger_sql(sql);
    assert_eq!(timing, "AFTER");
    assert_eq!(event, "INSERT");
}

#[test]
fn parse_trigger_with_quoted_begin_in_name() {
    // Quoted identifier containing "BEGIN" — must be skipped.
    let sql = r#"CREATE TRIGGER "trg_BEGIN_audit" AFTER UPDATE ON users BEGIN SELECT 1; END"#;
    let (timing, event) = parse_sqlite_trigger_sql(sql);
    assert_eq!(timing, "AFTER");
    assert_eq!(event, "UPDATE");
}

#[test]
fn parse_trigger_ignores_timing_and_event_words_in_quoted_names() {
    let sql = r#"CREATE TRIGGER "trg_BEFORE_INSERT" AFTER DELETE ON "table_AFTER_INSERT" BEGIN SELECT 1; END"#;
    let (timing, event) = parse_sqlite_trigger_sql(sql);
    assert_eq!(timing, "AFTER");
    assert_eq!(event, "DELETE");
}

#[test]
fn introspection_populates_columns_for_sqlite_views() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT, active INTEGER);\
             CREATE VIEW active_users AS SELECT id, email FROM users WHERE active = 1;",
    )
    .unwrap();

    let result = run_introspection(&conn).unwrap();
    assert_eq!(result.views.len(), 1);
    assert_eq!(result.views[0].name, "active_users");

    let view_cols: Vec<_> = result
        .columns
        .iter()
        .filter(|c| c.table_name == "active_users")
        .collect();
    assert_eq!(view_cols.len(), 2);
    assert_eq!(view_cols[0].name, "id");
    assert_eq!(view_cols[1].name, "email");
}
