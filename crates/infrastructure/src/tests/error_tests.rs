use super::*;

#[test]
fn sqlite_primary_key_error_keeps_its_constraint_identity() {
    let connection = rusqlite::Connection::open_in_memory().unwrap();
    connection
        .execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY); INSERT INTO t VALUES (1);")
        .unwrap();
    let error = connection.execute("INSERT INTO t VALUES (1)", []).unwrap_err();
    assert!(matches!(
        from_rusqlite(error),
        DbError::ConstraintViolation {
            constraint_type: ConstraintType::PrimaryKey,
            ..
        }
    ));
}

#[test]
fn sqlite_trigger_constraint_is_not_mislabeled_as_check() {
    let connection = rusqlite::Connection::open_in_memory().unwrap();
    connection.execute_batch(
        "CREATE TABLE t (id INTEGER); CREATE TRIGGER reject_insert BEFORE INSERT ON t BEGIN SELECT RAISE(ABORT, 'blocked by trigger'); END;"
    ).unwrap();
    let error = connection.execute("INSERT INTO t VALUES (1)", []).unwrap_err();
    assert!(matches!(
        from_rusqlite(error),
        DbError::ConstraintViolation { constraint_type: ConstraintType::Other(code), .. }
            if code == rusqlite::ffi::SQLITE_CONSTRAINT_TRIGGER
    ));
}

#[test]
fn sqlite_known_constraint_categories_remain_distinct() {
    let connection = rusqlite::Connection::open_in_memory().unwrap();
    connection.execute_batch(
        "PRAGMA foreign_keys = ON; CREATE TABLE parent (id INTEGER PRIMARY KEY); CREATE TABLE child (id INTEGER UNIQUE, parent_id INTEGER REFERENCES parent(id), label TEXT NOT NULL, value INTEGER CHECK (value > 0)); INSERT INTO child VALUES (1, NULL, 'valid', 1);"
    ).unwrap();
    for (sql, expected) in [
        ("INSERT INTO child VALUES (1, NULL, 'valid', 1)", ConstraintType::Unique),
        (
            "INSERT INTO child VALUES (2, 99, 'valid', 1)",
            ConstraintType::ForeignKey,
        ),
        ("INSERT INTO child VALUES (2, NULL, NULL, 1)", ConstraintType::NotNull),
        ("INSERT INTO child VALUES (2, NULL, 'valid', 0)", ConstraintType::Check),
    ] {
        let error = from_rusqlite(connection.execute(sql, []).unwrap_err());
        assert!(matches!(error, DbError::ConstraintViolation { constraint_type, .. } if constraint_type == expected));
    }
}
