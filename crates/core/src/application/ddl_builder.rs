use crate::ports::dialect::SqlDialect;

pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub is_pk: bool,
}

pub fn build_create_table(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    columns: &[ColumnDef],
) -> String {
    let qualified = if schema.is_empty() {
        dialect.quote_identifier(table).to_string()
    } else {
        format!("{}.{}", dialect.quote_identifier(schema), dialect.quote_identifier(table))
    };

    let mut parts = Vec::new();
    let mut pk_cols = Vec::new();

    for col in columns {
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

    format!("CREATE TABLE {qualified} (\n{}\n)", parts.join(",\n"))
}

pub fn build_alter_table_add_column(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    column: &ColumnDef,
) -> String {
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
    def
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

pub fn build_create_view(
    dialect: &dyn SqlDialect,
    schema: &str,
    name: &str,
    select_sql: &str,
) -> String {
    let qualified = qualify(dialect, schema, name);
    format!("CREATE VIEW {qualified} AS\n{select_sql}")
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

pub fn build_create_trigger(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    trigger_name: &str,
    timing: &str,
    event: &str,
    body: &str,
) -> String {
    let qualified_table = qualify(dialect, schema, table);
    let qualified_trigger = qualify(dialect, schema, trigger_name);
    format!(
        "CREATE TRIGGER {qualified_trigger}\n  {timing} {event} ON {qualified_table}\n  {body}"
    )
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
        let sql = build_create_table(&dialect, "public", "users", &cols);
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
}
