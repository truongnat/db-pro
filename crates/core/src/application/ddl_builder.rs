use crate::ports::dialect::SqlDialect;

/// Column definition for DDL building.
///
/// **Warning**: `data_type` and `default` are raw SQL fragments — they are
/// inserted verbatim into the generated SQL without sanitization. Callers
/// must ensure these values come from trusted sources (e.g., the visual
/// DDL editor UI, not arbitrary user input).
pub struct ColumnDef {
    pub name: String,
    /// Raw SQL type expression (e.g. `"VARCHAR(255)"`, `"INTEGER"`). Not sanitized.
    pub data_type: String,
    pub nullable: bool,
    /// Raw SQL default expression (e.g. `"CURRENT_TIMESTAMP"`, `"0"`). Not sanitized.
    pub default: Option<String>,
    pub is_pk: bool,
}

pub fn build_create_table(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    columns: &[ColumnDef],
) -> Result<String, String> {
    let qualified = if schema.is_empty() {
        dialect.quote_identifier(table).to_string()
    } else {
        format!("{}.{}", dialect.quote_identifier(schema), dialect.quote_identifier(table))
    };

    let mut parts = Vec::new();
    let mut pk_cols = Vec::new();

    for col in columns {
        validate_raw_fragment(&col.data_type, "data_type")?;
        if let Some(ref default) = col.default {
            validate_raw_fragment(default, "default")?;
        }

        let mut def = format!(
            "    {} {}",
            dialect.quote_identifier(&col.name),
            col.data_type
        );
        if !col.nullable {
            def.push_str(" NOT NULL");
        }
        if let Some(ref default) = col.default {
            def.push_str(&format!(" DEFAULT {default}"));
        }
        if col.is_pk {
            pk_cols.push(dialect.quote_identifier(&col.name).to_string());
        }
        parts.push(def);
    }

    if !pk_cols.is_empty() {
        parts.push(format!("    PRIMARY KEY ({})", pk_cols.join(", ")));
    }

    Ok(format!("CREATE TABLE {qualified} (\n{}\n)", parts.join(",\n")))
}

pub fn build_alter_table_add_column(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    column: &ColumnDef,
) -> Result<String, String> {
    validate_raw_fragment(&column.data_type, "data_type")?;
    if let Some(ref default) = column.default {
        validate_raw_fragment(default, "default")?;
    }

    let qualified = qualify(dialect, schema, table);
    let mut def = format!(
        "ALTER TABLE {qualified} ADD COLUMN {} {}",
        dialect.quote_identifier(&column.name),
        column.data_type
    );
    if !column.nullable {
        def.push_str(" NOT NULL");
    }
    if let Some(ref default) = column.default {
        def.push_str(&format!(" DEFAULT {default}"));
    }
    Ok(def)
}

pub fn build_alter_table_drop_column(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    column_name: &str,
) -> String {
    let qualified = qualify(dialect, schema, table);
    format!(
        "ALTER TABLE {qualified} DROP COLUMN {}",
        dialect.quote_identifier(column_name)
    )
}

pub fn build_alter_table_rename(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    new_name: &str,
) -> String {
    let qualified = qualify(dialect, schema, table);
    format!(
        "ALTER TABLE {qualified} RENAME TO {}",
        dialect.quote_identifier(new_name)
    )
}

pub fn build_drop_table(dialect: &dyn SqlDialect, schema: &str, table: &str) -> String {
    let qualified = qualify(dialect, schema, table);
    format!("DROP TABLE {qualified}")
}

/// Build a CREATE VIEW statement.
///
/// **Warning**: `select_sql` is a raw SQL fragment inserted verbatim.
pub fn build_create_view(
    dialect: &dyn SqlDialect,
    schema: &str,
    name: &str,
    select_sql: &str,
) -> Result<String, String> {
    validate_raw_fragment(select_sql, "select_sql")?;
    let qualified = qualify(dialect, schema, name);
    Ok(format!("CREATE VIEW {qualified} AS\n{select_sql}"))
}

pub fn build_drop_view(dialect: &dyn SqlDialect, schema: &str, name: &str) -> String {
    let qualified = qualify(dialect, schema, name);
    format!("DROP VIEW {qualified}")
}

pub fn build_create_index(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    index_name: &str,
    columns: &[String],
    unique: bool,
) -> String {
    let qualified_table = qualify(dialect, schema, table);
    let qualified_index = qualify(dialect, schema, index_name);
    let cols: Vec<String> = columns.iter().map(|c| dialect.quote_identifier(c).to_string()).collect();
    let unique_kw = if unique { "UNIQUE " } else { "" };
    format!(
        "CREATE {unique_kw}INDEX {qualified_index} ON {qualified_table} ({})",
        cols.join(", ")
    )
}

pub fn build_drop_index(dialect: &dyn SqlDialect, schema: &str, index_name: &str) -> String {
    let qualified = qualify(dialect, schema, index_name);
    format!("DROP INDEX {qualified}")
}

/// Build a CREATE TRIGGER statement.
///
/// **Warning**: `timing`, `event`, and `body` are raw SQL fragments inserted verbatim.
pub fn build_create_trigger(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    trigger_name: &str,
    timing: &str,
    event: &str,
    body: &str,
) -> Result<String, String> {
    validate_raw_fragment(timing, "timing")?;
    validate_raw_fragment(event, "event")?;
    validate_raw_fragment(body, "body")?;

    let qualified_table = qualify(dialect, schema, table);
    let qualified_trigger = qualify(dialect, schema, trigger_name);
    Ok(format!(
        "CREATE TRIGGER {qualified_trigger}\n  {timing} {event} ON {qualified_table}\n  {body}"
    ))
}

pub fn build_drop_trigger(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    trigger_name: &str,
) -> String {
    let qualified_table = qualify(dialect, schema, table);
    let qualified_trigger = qualify(dialect, schema, trigger_name);
    format!("DROP TRIGGER {qualified_trigger} ON {qualified_table}")
}

fn qualify(dialect: &dyn SqlDialect, schema: &str, name: &str) -> String {
    if schema.is_empty() {
        dialect.quote_identifier(name).to_string()
    } else {
        format!(
            "{}.{}",
            dialect.quote_identifier(schema),
            dialect.quote_identifier(name)
        )
    }
}

/// Rejects raw SQL fragments that contain statement terminators or
/// keywords that would alter the intended DDL structure.
/// This is a guardrail, not a full sanitizer — callers must still
/// ensure values come from trusted sources.
fn validate_raw_fragment(value: &str, field_name: &str) -> Result<(), String> {
    if value.contains(';') {
        return Err(format!("{field_name} must not contain semicolons"));
    }
    let upper = value.to_ascii_uppercase();
    let tokens: Vec<&str> = upper
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|t| !t.is_empty())
        .collect();
    for token in &tokens {
        match *token {
            "DROP" | "DELETE" | "INSERT" | "UPDATE" => {
                return Err(format!("{field_name} must not contain {token} statements"));
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDialect;
    impl SqlDialect for TestDialect {
        fn placeholder(&self, _index: usize) -> String {
            "?".into()
        }
        fn quote_identifier(&self, name: &str) -> String {
            format!("\"{name}\"")
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
        let sql = build_create_index(
            &dialect,
            "public",
            "users",
            "idx_email",
            &["email".into()],
            true,
        );
        assert!(sql.contains("UNIQUE"));
        assert!(sql.contains("\"idx_email\""));
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
}
