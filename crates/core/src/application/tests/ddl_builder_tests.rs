//! Tests for `../ddl_builder.rs` (moved out of the source file).
use super::*;

struct TestDialect;
impl SqlDialect for TestDialect {
    fn placeholder(&self, _index: usize) -> String {
        "?".into()
    }
    fn quote_identifier(&self, name: &str) -> String {
        format!("\"{}\"", name.replace('"', "\"\""))
    }
}

#[test]
fn create_table_basic() {
    let dialect = TestDialect;
    let cols = vec![
        ColumnDef {
            name: "id".into(),
            data_type: "INTEGER".into(),
            nullable: false,
            default: None,
            is_pk: true,
        },
        ColumnDef {
            name: "name".into(),
            data_type: "TEXT".into(),
            nullable: true,
            default: None,
            is_pk: false,
        },
    ];
    let sql = build_create_table(&dialect, "public", "users", &cols).unwrap();
    assert!(sql.contains("CREATE TABLE"));
    assert!(sql.contains("\"public\".\"users\""));
    assert!(sql.contains("PRIMARY KEY"));
}

#[test]
fn drop_table_qualified() {
    let dialect = TestDialect;
    let sql = build_drop_table(&dialect, "public", "users");
    assert_eq!(sql, "DROP TABLE \"public\".\"users\"");
}

#[test]
fn create_index_unique() {
    let dialect = TestDialect;
    let sql = build_create_index(&dialect, "public", "users", "idx_email", &["email".into()], true);
    assert!(sql.contains("UNIQUE"));
    assert!(sql.contains("\"idx_email\""));
}

#[test]
fn pg_trigger_enable_disable_quotes_identifiers() {
    let dialect = TestDialect;
    assert_eq!(
        build_set_trigger_enabled(&dialect, "odd schema", "user\"table", "tri\"gger", true),
        "ALTER TABLE \"odd schema\".\"user\"\"table\" ENABLE TRIGGER \"tri\"\"gger\""
    );
    assert_eq!(
        build_set_trigger_enabled(&dialect, "public", "users", "audit", false),
        "ALTER TABLE \"public\".\"users\" DISABLE TRIGGER \"audit\""
    );
}

#[test]
fn validate_rejects_semicolon_in_data_type() {
    let dialect = TestDialect;
    let cols = vec![ColumnDef {
        name: "id".into(),
        data_type: "INTEGER; DROP TABLE users".into(),
        nullable: false,
        default: None,
        is_pk: true,
    }];
    let err = build_create_table(&dialect, "public", "users", &cols).unwrap_err();
    assert!(err.contains("semicolons"));
}

#[test]
fn validate_rejects_drop_in_default() {
    let dialect = TestDialect;
    let cols = vec![ColumnDef {
        name: "id".into(),
        data_type: "INTEGER".into(),
        nullable: false,
        default: Some("(SELECT DROP TABLE users)".into()),
        is_pk: true,
    }];
    let err = build_create_table(&dialect, "public", "users", &cols).unwrap_err();
    assert!(err.contains("DROP"));
}

#[test]
fn validate_rejects_semicolon_in_view_body() {
    let dialect = TestDialect;
    let err = build_create_view(&dialect, "public", "v1", "SELECT 1; DROP TABLE x").unwrap_err();
    assert!(err.contains("semicolons"));
}

#[test]
fn validate_accepts_identifier_containing_keyword_substring() {
    assert!(validate_raw_fragment("my_drop_column", "data_type").is_ok());
    assert!(validate_raw_fragment("last_update_time", "data_type").is_ok());
    assert!(validate_raw_fragment("auto_increment", "data_type").is_ok());
}
fn alter_column_def(data_type: &str, nullable: bool, default: Option<&str>) -> ColumnAlterDef {
    ColumnAlterDef {
        name: "email".into(),
        data_type: data_type.to_owned(),
        nullable,
        default: default.map(str::to_owned),
    }
}

#[test]
fn pg_alter_column_combines_type_nullable_and_default() {
    let dialect = TestDialect;
    let sql = build_alter_table_alter_column(
        &dialect,
        "public",
        "users",
        &alter_column_def("VARCHAR(320)", false, Some("'none@example.com'")),
    )
    .unwrap();
    assert_eq!(
            sql,
            "ALTER TABLE \"public\".\"users\" ALTER COLUMN \"email\" TYPE VARCHAR(320), ALTER COLUMN \"email\" SET NOT NULL, ALTER COLUMN \"email\" SET DEFAULT 'none@example.com'"
        );
}

#[test]
fn pg_alter_column_nullable_toggle_and_default_drop() {
    let dialect = TestDialect;
    let sql =
        build_alter_table_alter_column(&dialect, "public", "users", &alter_column_def("TEXT", true, None)).unwrap();
    assert!(sql.contains("ALTER COLUMN \"email\" TYPE TEXT"));
    assert!(sql.contains("ALTER COLUMN \"email\" DROP NOT NULL"));
    assert!(sql.contains("ALTER COLUMN \"email\" DROP DEFAULT"));
}

#[test]
fn pg_alter_column_quotes_identifiers_and_unqualified_table() {
    let dialect = TestDialect;
    let column = ColumnAlterDef {
        name: "weird name".into(),
        data_type: "INT".into(),
        nullable: true,
        default: None,
    };
    let sql = build_alter_table_alter_column(&dialect, "", "my table", &column).unwrap();
    assert!(sql.starts_with("ALTER TABLE \"my table\" ALTER COLUMN \"weird name\" TYPE INT"));
}

#[test]
fn pg_alter_column_rejects_unsafe_fragments_and_empty_type() {
    let dialect = TestDialect;
    let err = build_alter_table_alter_column(
        &dialect,
        "public",
        "users",
        &alter_column_def("TEXT; DROP TABLE users", true, None),
    )
    .unwrap_err();
    assert!(err.contains("semicolons"));

    let err = build_alter_table_alter_column(
        &dialect,
        "public",
        "users",
        &alter_column_def("TEXT", true, Some("nextval DELETE")),
    )
    .unwrap_err();
    assert!(err.contains("DELETE"));

    let err =
        build_alter_table_alter_column(&dialect, "public", "users", &alter_column_def("  ", true, None)).unwrap_err();
    assert!(err.contains("data_type"));
}

#[test]
fn mysql_modify_column_restates_full_definition() {
    let dialect = TestDialect;
    let sql = build_alter_table_modify_column(
        &dialect,
        "app",
        "users",
        &alter_column_def("VARCHAR(320)", false, Some("'x'")),
    )
    .unwrap();
    assert_eq!(
        sql,
        "ALTER TABLE \"app\".\"users\" MODIFY COLUMN \"email\" VARCHAR(320) NOT NULL DEFAULT 'x'"
    );

    let sql = build_alter_table_modify_column(&dialect, "app", "users", &alter_column_def("TEXT", true, None)).unwrap();
    assert_eq!(sql, "ALTER TABLE \"app\".\"users\" MODIFY COLUMN \"email\" TEXT");
}

#[test]
fn sqlserver_alter_column_type_and_nullable_only() {
    let dialect = TestDialect;
    let sql = build_alter_table_alter_column_sqlserver(
        &dialect,
        "dbo",
        "users",
        &alter_column_def("NVARCHAR(320)", false, None),
    )
    .unwrap();
    assert_eq!(
        sql,
        "ALTER TABLE \"dbo\".\"users\" ALTER COLUMN \"email\" NVARCHAR(320) NOT NULL"
    );

    let err = build_alter_table_alter_column_sqlserver(
        &dialect,
        "dbo",
        "users",
        &alter_column_def("NVARCHAR(320)", false, Some("'x'")),
    )
    .unwrap_err();
    assert!(err.contains("named constraints"));
}
