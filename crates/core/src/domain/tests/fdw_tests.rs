//! Tests for `../fdw.rs` (moved out of the source file).
use super::*;

#[test]
fn redacts_password_options() {
    assert!(is_sensitive_option_key("password"));
    assert!(!is_sensitive_option_key("host"));
}

#[test]
fn preview_sql_quotes_idents() {
    let sql = preview_drop_server("remote_pg", true).unwrap();
    assert!(sql.contains("DROP SERVER IF EXISTS \"remote_pg\" CASCADE"));
    assert!(preview_create_server("s1", "postgres_fdw", "h", "db", "5432")
        .unwrap()
        .contains("FOREIGN DATA WRAPPER \"postgres_fdw\""));
}
