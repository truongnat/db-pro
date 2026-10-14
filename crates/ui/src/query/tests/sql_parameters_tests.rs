//! Tests for `../sql_parameters.rs` (moved out of the source file).
use super::*;

#[test]
fn discovers_numbered_and_named_outside_literals() {
    let found = discover_sql_parameters(
        "SELECT * FROM t WHERE id = $1 AND name = :customer -- $2\nAND note = ':skip' /* :also */",
    );
    let names: Vec<_> = found.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["$1", ":customer"]);
}

#[test]
fn discovers_positional_placeholders_in_order() {
    let found = discover_sql_parameters("SELECT ? FROM t WHERE a = ? AND b = ?");
    assert_eq!(found.len(), 3);
    assert!(found.iter().all(|p| p.kind == ParameterKind::Positional));
    assert_eq!(found[0].name, "?1");
    assert_eq!(found[2].name, "?3");
}

#[test]
fn prepare_rewrites_named_to_postgres_placeholders() {
    let mut values = HashMap::new();
    values.insert(":id".to_owned(), "7".to_owned());
    values.insert(":name".to_owned(), "Ada".to_owned());
    let prepared = prepare_bound_sql(
        "SELECT * FROM t WHERE id = :id AND name = :name",
        &values,
        PlaceholderStyle::NumberedDollar,
    )
    .expect("prepared");
    assert_eq!(prepared.sql, "SELECT * FROM t WHERE id = $1 AND name = $2");
    assert_eq!(prepared.values, vec!["7".to_owned(), "Ada".to_owned()]);
}

#[test]
fn prepare_rewrites_to_question_marks_for_sqlite() {
    let mut values = HashMap::new();
    values.insert("$1".to_owned(), "1".to_owned());
    values.insert(":name".to_owned(), "x".to_owned());
    let prepared = prepare_bound_sql(
        "SELECT $1, :name FROM t -- :skip\n",
        &values,
        PlaceholderStyle::QuestionMark,
    )
    .expect("prepared");
    assert_eq!(prepared.sql, "SELECT ?, ? FROM t -- :skip\n");
    assert_eq!(prepared.values, vec!["1".to_owned(), "x".to_owned()]);
}

#[test]
fn prepare_errors_when_value_missing() {
    let err = prepare_bound_sql("SELECT :id", &HashMap::new(), PlaceholderStyle::QuestionMark).expect_err("missing");
    assert_eq!(err, ":id");
}

#[test]
fn postgres_jsonb_question_mark_is_not_discovered_or_rewritten() {
    let sql = "SELECT data ? 'x'";
    assert!(discover_sql_parameters_for_driver(sql, "PostgreSQL").is_empty());
    assert!(discover_sql_parameters_for_driver(sql, "postgres").is_empty());
    let prepared = prepare_bound_sql(sql, &HashMap::new(), PlaceholderStyle::NumberedDollar).expect("no bind");
    assert_eq!(prepared.sql, sql);
    assert!(prepared.values.is_empty());

    let mut values = HashMap::new();
    values.insert(":id".to_owned(), "7".to_owned());
    let prepared = prepare_bound_sql(
        "SELECT data ? 'x' AND id = :id",
        &values,
        PlaceholderStyle::NumberedDollar,
    )
    .expect("prepared");
    assert_eq!(prepared.sql, "SELECT data ? 'x' AND id = $1");
    assert_eq!(prepared.values, vec!["7".to_owned()]);
}

#[test]
fn dollar_quotes_hide_inner_placeholders_but_not_outer_ones() {
    for sql in [
        "SELECT $$ $1 $$ || $1",
        "SELECT $tag$ $1 :id ? $tag$ || $1",
        "SELECT $func$ $1 $func$, $1",
    ] {
        let names: Vec<_> = discover_sql_parameters(sql).into_iter().map(|p| p.name).collect();
        assert_eq!(names, vec!["$1".to_owned()], "{sql}");
    }

    let mut values = HashMap::new();
    values.insert("$1".to_owned(), "7".to_owned());
    let prepared =
        prepare_bound_sql("SELECT $$ $1 $$ || $1", &values, PlaceholderStyle::NumberedDollar).expect("prepared");
    assert_eq!(prepared.sql, "SELECT $$ $1 $$ || $1");
    assert_eq!(prepared.values, vec!["7".to_owned()]);
}

#[test]
fn sql_server_rewrites_named_and_numbered_to_at_params() {
    let mut values = HashMap::new();
    values.insert(":id".to_owned(), "7".to_owned());
    values.insert("$1".to_owned(), "8".to_owned());
    let prepared = prepare_bound_sql("SELECT :id, $1", &values, PlaceholderStyle::AtPn).expect("prepared");
    assert_eq!(prepared.sql, "SELECT @p1, @p2");
    assert_eq!(prepared.values, vec!["7".to_owned(), "8".to_owned()]);

    let jsonb = "SELECT data ? 'x'";
    assert!(discover_sql_parameters_for_driver(jsonb, "sql server").is_empty());
    assert!(discover_sql_parameters_for_driver(jsonb, "mssql").is_empty());
    let prepared = prepare_bound_sql(jsonb, &HashMap::new(), PlaceholderStyle::AtPn).expect("operator");
    assert_eq!(prepared.sql, jsonb);
    assert!(prepared.values.is_empty());
}

#[test]
fn question_mark_style_still_binds_positional_values() {
    let mut values = HashMap::new();
    values.insert("?1".to_owned(), "9".to_owned());
    values.insert("?2".to_owned(), "10".to_owned());
    let prepared = prepare_bound_sql("SELECT ?, ?", &values, PlaceholderStyle::QuestionMark).expect("prepared");
    assert_eq!(prepared.sql, "SELECT ?, ?");
    assert_eq!(prepared.values, vec!["9".to_owned(), "10".to_owned()]);
    assert_eq!(discover_sql_parameters("SELECT '?' , ?").len(), 1);
    assert_eq!(discover_sql_parameters_for_driver("SELECT ?", "mysql").len(), 1);
    assert_eq!(discover_sql_parameters_for_driver("SELECT ?", "SQLite").len(), 1);
}

#[test]
fn placeholder_style_follows_the_driver() {
    assert_eq!(
        placeholder_style_for_driver("PostgreSQL"),
        PlaceholderStyle::NumberedDollar
    );
    assert_eq!(
        placeholder_style_for_driver("postgres"),
        PlaceholderStyle::NumberedDollar
    );
    assert_eq!(placeholder_style_for_driver("sql server"), PlaceholderStyle::AtPn);
    assert_eq!(placeholder_style_for_driver("SQLServer"), PlaceholderStyle::AtPn);
    assert_eq!(placeholder_style_for_driver("mssql"), PlaceholderStyle::AtPn);
    assert_eq!(placeholder_style_for_driver("MySQL"), PlaceholderStyle::QuestionMark);
    assert_eq!(placeholder_style_for_driver("sqlite"), PlaceholderStyle::QuestionMark);
}

#[test]
fn repeated_numbered_placeholder_binds_each_occurrence() {
    let mut values = HashMap::new();
    values.insert("$1".to_owned(), "7".to_owned());
    let prepared = prepare_bound_sql("SELECT $1, $1", &values, PlaceholderStyle::NumberedDollar).expect("prepared");
    assert_eq!(prepared.sql, "SELECT $1, $2");
    assert_eq!(prepared.values, vec!["7".to_owned(), "7".to_owned()]);
}
