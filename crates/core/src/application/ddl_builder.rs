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
        format!(
            "{}.{}",
            dialect.quote_identifier(schema),
            dialect.quote_identifier(table)
        )
    };

    let mut parts = Vec::new();
    let mut pk_cols = Vec::new();

    for col in columns {
        validate_raw_fragment(&col.data_type, "data_type")?;
        if let Some(ref default) = col.default {
            validate_raw_fragment(default, "default")?;
        }

        let mut def = format!("    {} {}", dialect.quote_identifier(&col.name), col.data_type);
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

pub fn build_alter_table_drop_column(dialect: &dyn SqlDialect, schema: &str, table: &str, column_name: &str) -> String {
    let qualified = qualify(dialect, schema, table);
    format!(
        "ALTER TABLE {qualified} DROP COLUMN {}",
        dialect.quote_identifier(column_name)
    )
}

pub fn build_alter_table_rename(dialect: &dyn SqlDialect, schema: &str, table: &str, new_name: &str) -> String {
    let qualified = qualify(dialect, schema, table);
    format!(
        "ALTER TABLE {qualified} RENAME TO {}",
        dialect.quote_identifier(new_name)
    )
}

pub fn build_alter_table_rename_column(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    old_name: &str,
    new_name: &str,
) -> String {
    let qualified = qualify(dialect, schema, table);
    format!(
        "ALTER TABLE {qualified} RENAME COLUMN {} TO {}",
        dialect.quote_identifier(old_name),
        dialect.quote_identifier(new_name)
    )
}

/// Column attributes for an `ALTER COLUMN`-style mutation (type change,
/// nullability toggle, default set/drop).
///
/// **Warning**: `data_type` and `default` are raw SQL fragments — they are
/// inserted verbatim into the generated SQL without sanitization (only the
/// shared `validate_raw_fragment` guardrail applies). Callers must ensure
/// these values come from trusted sources.
pub struct ColumnAlterDef {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    /// `Some(expr)` sets the default to `expr`; `None` drops the default where
    /// the provider's ALTER syntax expresses that (PostgreSQL, MySQL).
    pub default: Option<String>,
}

/// PostgreSQL-style `ALTER TABLE … ALTER COLUMN` with combined sub-actions:
/// `TYPE`, `SET|DROP NOT NULL`, `SET|DROP DEFAULT` in one statement.
pub fn build_alter_table_alter_column(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    column: &ColumnAlterDef,
) -> Result<String, String> {
    validate_raw_fragment(&column.data_type, "data_type")?;
    if let Some(default) = &column.default {
        validate_raw_fragment(default, "default")?;
    }
    if column.data_type.trim().is_empty() {
        return Err("data_type must not be empty".to_owned());
    }

    let qualified = qualify(dialect, schema, table);
    let name = dialect.quote_identifier(&column.name);
    let nullability = if column.nullable {
        "DROP NOT NULL"
    } else {
        "SET NOT NULL"
    };
    let default_clause = match &column.default {
        Some(default) => format!("ALTER COLUMN {name} SET DEFAULT {default}"),
        None => format!("ALTER COLUMN {name} DROP DEFAULT"),
    };
    Ok(format!(
        "ALTER TABLE {qualified} ALTER COLUMN {name} TYPE {}, ALTER COLUMN {name} {nullability}, {default_clause}",
        column.data_type
    ))
}

/// MySQL `ALTER TABLE … MODIFY COLUMN` — requires restating the full column
/// definition (type + nullability + default). Omitting `DEFAULT` drops any
/// existing default, matching PostgreSQL `DROP DEFAULT` semantics.
pub fn build_alter_table_modify_column(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    column: &ColumnAlterDef,
) -> Result<String, String> {
    validate_raw_fragment(&column.data_type, "data_type")?;
    if let Some(default) = &column.default {
        validate_raw_fragment(default, "default")?;
    }
    if column.data_type.trim().is_empty() {
        return Err("data_type must not be empty".to_owned());
    }

    let qualified = qualify(dialect, schema, table);
    let mut def = format!(
        "ALTER TABLE {qualified} MODIFY COLUMN {} {}",
        dialect.quote_identifier(&column.name),
        column.data_type
    );
    if !column.nullable {
        def.push_str(" NOT NULL");
    }
    if let Some(default) = &column.default {
        def.push_str(&format!(" DEFAULT {default}"));
    }
    Ok(def)
}

/// SQL Server `ALTER TABLE … ALTER COLUMN` — only type and nullability can be
/// altered this way; defaults live in named constraints and are intentionally
/// rejected rather than silently emitted or dropped.
pub fn build_alter_table_alter_column_sqlserver(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    column: &ColumnAlterDef,
) -> Result<String, String> {
    validate_raw_fragment(&column.data_type, "data_type")?;
    if column.data_type.trim().is_empty() {
        return Err("data_type must not be empty".to_owned());
    }
    if column.default.is_some() {
        return Err(
            "SQL Server column defaults live in named constraints; changing them is not supported by this ALTER path"
                .to_owned(),
        );
    }

    let qualified = qualify(dialect, schema, table);
    let nullability = if column.nullable { "NULL" } else { "NOT NULL" };
    Ok(format!(
        "ALTER TABLE {qualified} ALTER COLUMN {} {} {nullability}",
        dialect.quote_identifier(&column.name),
        column.data_type
    ))
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
    let cols: Vec<String> = columns
        .iter()
        .map(|c| dialect.quote_identifier(c).to_string())
        .collect();
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

pub fn build_drop_trigger(dialect: &dyn SqlDialect, schema: &str, table: &str, trigger_name: &str) -> String {
    let qualified_table = qualify(dialect, schema, table);
    let qualified_trigger = qualify(dialect, schema, trigger_name);
    format!("DROP TRIGGER {qualified_trigger} ON {qualified_table}")
}
/// Build a PostgreSQL table-trigger enable/disable statement.
pub fn build_set_trigger_enabled(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    trigger_name: &str,
    enabled: bool,
) -> String {
    let qualified_table = qualify(dialect, schema, table);
    let trigger_name = dialect.quote_identifier(trigger_name);
    let action = if enabled { "ENABLE" } else { "DISABLE" };
    format!("ALTER TABLE {qualified_table} {action} TRIGGER {trigger_name}")
}

pub fn build_create_materialized_view(
    dialect: &dyn SqlDialect,
    schema: &str,
    name: &str,
    select_sql: &str,
) -> Result<String, String> {
    validate_raw_fragment(select_sql, "select_sql")?;
    let qualified = qualify(dialect, schema, name);
    Ok(format!("CREATE MATERIALIZED VIEW {qualified} AS\n{select_sql}"))
}

pub fn build_refresh_materialized_view(dialect: &dyn SqlDialect, schema: &str, name: &str) -> String {
    let qualified = qualify(dialect, schema, name);
    format!("REFRESH MATERIALIZED VIEW {qualified}")
}

pub fn build_drop_materialized_view(dialect: &dyn SqlDialect, schema: &str, name: &str) -> String {
    let qualified = qualify(dialect, schema, name);
    format!("DROP MATERIALIZED VIEW {qualified}")
}

pub fn build_add_primary_key(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    name: &str,
    columns: &[String],
) -> String {
    let qualified = qualify(dialect, schema, table);
    let cols = quote_cols(dialect, columns);
    let constraint = dialect.quote_identifier(name);
    format!("ALTER TABLE {qualified} ADD CONSTRAINT {constraint} PRIMARY KEY ({cols})")
}

// cc-scan:allow TOO_MANY_PARAMS — context params passed through
pub fn build_add_unique(dialect: &dyn SqlDialect, schema: &str, table: &str, name: &str, columns: &[String]) -> String {
    let qualified = qualify(dialect, schema, table);
    let cols = quote_cols(dialect, columns);
    let constraint = dialect.quote_identifier(name);
    format!("ALTER TABLE {qualified} ADD CONSTRAINT {constraint} UNIQUE ({cols})")
}

pub fn build_add_check(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    name: &str,
    expression: &str,
) -> Result<String, String> {
    validate_raw_fragment(expression, "expression")?;
    let qualified = qualify(dialect, schema, table);
    let constraint = dialect.quote_identifier(name);
    Ok(format!(
        "ALTER TABLE {qualified} ADD CONSTRAINT {constraint} CHECK ({expression})"
    ))
}

// allow: columns and ref_columns are parallel arrays by positional index (column ↔ referenced column),
// matching the SQL clause order — bundling into an options struct would obscure that positional contract.
#[allow(clippy::too_many_arguments)]
pub fn build_add_foreign_key(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    name: &str,
    columns: &[String],
    ref_schema: &str,
    ref_table: &str,
    ref_columns: &[String],
    on_delete: Option<&str>,
    on_update: Option<&str>,
) -> Result<String, String> {
    if let Some(v) = on_delete {
        validate_fk_action(v)?;
    }
    if let Some(v) = on_update {
        validate_fk_action(v)?;
    }
    let qualified = qualify(dialect, schema, table);
    let constraint = dialect.quote_identifier(name);
    let cols = quote_cols(dialect, columns);
    let ref_qualified = qualify(dialect, ref_schema, ref_table);
    let ref_cols = quote_cols(dialect, ref_columns);
    let mut sql = format!(
        // cc-scan:allow LINE_TOO_LONG — literal must not wrap
        "ALTER TABLE {qualified} ADD CONSTRAINT {constraint} FOREIGN KEY ({cols}) REFERENCES {ref_qualified} ({ref_cols})"
    );
    if let Some(action) = on_delete {
        sql.push_str(&format!(" ON DELETE {action}"));
    }
    if let Some(action) = on_update {
        sql.push_str(&format!(" ON UPDATE {action}"));
    }
    Ok(sql)
}

pub fn build_drop_constraint(dialect: &dyn SqlDialect, schema: &str, table: &str, name: &str) -> String {
    let qualified = qualify(dialect, schema, table);
    let constraint = dialect.quote_identifier(name);
    format!("ALTER TABLE {qualified} DROP CONSTRAINT {constraint}")
}

pub fn build_create_sequence(
    dialect: &dyn SqlDialect,
    schema: &str,
    name: &str,
    start: Option<i64>,
    increment: Option<i64>,
    cycle: bool,
) -> String {
    let qualified = qualify(dialect, schema, name);
    let mut sql = format!("CREATE SEQUENCE {qualified}");
    if let Some(v) = start {
        sql.push_str(&format!(" START WITH {v}"));
    }
    if let Some(v) = increment {
        sql.push_str(&format!(" INCREMENT BY {v}"));
    }
    if cycle {
        sql.push_str(" CYCLE");
    }
    sql
}

pub fn build_drop_sequence(dialect: &dyn SqlDialect, schema: &str, name: &str) -> String {
    let qualified = qualify(dialect, schema, name);
    format!("DROP SEQUENCE {qualified}")
}

pub fn build_create_enum(
    dialect: &dyn SqlDialect,
    schema: &str,
    name: &str,
    values: &[String],
) -> Result<String, String> {
    if values.is_empty() {
        return Err("enum values must not be empty".into());
    }
    let qualified = qualify(dialect, schema, name);
    let rendered: Vec<String> = values.iter().map(|v| format!("'{}'", v.replace('\'', "''"))).collect();
    Ok(format!("CREATE TYPE {qualified} AS ENUM ({})", rendered.join(", ")))
}

pub fn build_drop_type(dialect: &dyn SqlDialect, schema: &str, name: &str) -> String {
    let qualified = qualify(dialect, schema, name);
    format!("DROP TYPE {qualified}")
}

pub fn build_create_schema(dialect: &dyn SqlDialect, name: &str) -> String {
    format!("CREATE SCHEMA {}", dialect.quote_identifier(name))
}

pub fn build_drop_schema(dialect: &dyn SqlDialect, name: &str, cascade: bool) -> String {
    let cascade_kw = if cascade { " CASCADE" } else { "" };
    format!("DROP SCHEMA {}{cascade_kw}", dialect.quote_identifier(name))
}

pub fn build_create_database(dialect: &dyn SqlDialect, name: &str) -> String {
    format!("CREATE DATABASE {}", dialect.quote_identifier(name))
}

pub fn build_drop_database(dialect: &dyn SqlDialect, name: &str) -> String {
    format!("DROP DATABASE {}", dialect.quote_identifier(name))
}

pub fn build_create_extension(dialect: &dyn SqlDialect, name: &str, schema: Option<&str>) -> String {
    let mut sql = format!("CREATE EXTENSION {}", dialect.quote_identifier(name));
    if let Some(schema) = schema {
        sql.push_str(&format!(" SCHEMA {}", dialect.quote_identifier(schema)));
    }
    sql
}

pub fn build_drop_extension(dialect: &dyn SqlDialect, name: &str, cascade: bool) -> String {
    let cascade_kw = if cascade { " CASCADE" } else { "" };
    format!("DROP EXTENSION {}{cascade_kw}", dialect.quote_identifier(name))
}

/// Emit CREATE [OR REPLACE] routine SQL. `definition_sql` must already be a full
/// CREATE statement (typically from `pg_get_functiondef` or the workbench editor).
pub fn build_create_routine(definition_sql: &str, replace: bool) -> Result<String, String> {
    let trimmed = definition_sql.trim();
    if trimmed.is_empty() {
        return Err("routine definition SQL is empty".into());
    }
    if !replace {
        return Ok(trimmed.to_owned());
    }
    let upper = trimmed.to_ascii_uppercase();
    if upper.starts_with("CREATE OR REPLACE ") {
        return Ok(trimmed.to_owned());
    }
    if let Some(rest) = trimmed
        .strip_prefix("CREATE ")
        .or_else(|| trimmed.strip_prefix("create "))
    {
        return Ok(format!("CREATE OR REPLACE {rest}"));
    }
    Ok(trimmed.to_owned())
}

pub fn build_drop_routine(
    dialect: &dyn SqlDialect,
    schema: &str,
    name: &str,
    routine_type: &str,
    identity_arguments: &str,
    cascade: bool,
    if_exists: bool,
) -> String {
    let kind = if routine_type.eq_ignore_ascii_case("PROCEDURE") {
        "PROCEDURE"
    } else {
        "FUNCTION"
    };
    let exists = if if_exists { "IF EXISTS " } else { "" };
    let cascade_kw = if cascade { " CASCADE" } else { "" };
    let qualified = format!(
        "{}.{}({})",
        dialect.quote_identifier(schema),
        dialect.quote_identifier(name),
        identity_arguments.trim()
    );
    format!("DROP {kind} {exists}{qualified}{cascade_kw}")
}

pub fn build_comment_on(
    dialect: &dyn SqlDialect,
    object_kind: &str,
    schema: Option<&str>,
    name: &str,
    parent: Option<&str>,
    comment: Option<&str>,
) -> Result<String, String> {
    validate_raw_fragment(object_kind, "object_kind")?;
    let target = match (schema, parent) {
        (Some(schema), Some(parent)) => {
            format!(
                "{}.{}",
                qualify(dialect, schema, parent),
                dialect.quote_identifier(name)
            )
        }
        (Some(schema), None) => qualify(dialect, schema, name),
        (None, _) => dialect.quote_identifier(name).to_string(),
    };
    let value = match comment {
        Some(text) => format!("'{}'", text.replace('\'', "''")),
        None => "NULL".into(),
    };
    Ok(format!("COMMENT ON {object_kind} {target} IS {value}"))
}

pub fn build_create_partition(
    dialect: &dyn SqlDialect,
    schema: &str,
    parent_table: &str,
    name: &str,
    bound_expression: &str,
) -> Result<String, String> {
    validate_raw_fragment(bound_expression, "bound_expression")?;
    let parent = qualify(dialect, schema, parent_table);
    let child = qualify(dialect, schema, name);
    Ok(format!(
        "CREATE TABLE {child} PARTITION OF {parent} FOR VALUES {bound_expression}"
    ))
}

fn quote_cols(dialect: &dyn SqlDialect, columns: &[String]) -> String {
    columns
        .iter()
        .map(|c| dialect.quote_identifier(c).to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn validate_fk_action(value: &str) -> Result<(), String> {
    match value.to_ascii_uppercase().as_str() {
        "NO ACTION" | "RESTRICT" | "CASCADE" | "SET NULL" | "SET DEFAULT" => Ok(()),
        other => Err(format!("unsupported FK action: {other}")),
    }
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

        let err = build_alter_table_alter_column(&dialect, "public", "users", &alter_column_def("  ", true, None))
            .unwrap_err();
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

        let sql =
            build_alter_table_modify_column(&dialect, "app", "users", &alter_column_def("TEXT", true, None)).unwrap();
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
}
