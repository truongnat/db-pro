use db_pro_core::domain::error::DbError;
use db_pro_core::domain::schema::*;
use sqlx::Row as _;
use std::collections::HashSet;

pub async fn run_introspection(pool: &sqlx::PgPool) -> Result<IntrospectResult, DbError> {
    // Run independent introspection queries in parallel
    let (schemas, tables, raw_cols, primary_keys, indexes, foreign_keys, check_constraints, views, triggers, functions) = tokio::join!(
        introspect_schemas(pool),
        introspect_tables(pool),
        introspect_columns_raw(pool),
        introspect_primary_keys(pool),
        introspect_indexes(pool),
        introspect_foreign_keys(pool),
        introspect_check_constraints(pool),
        introspect_views(pool),
        introspect_triggers(pool),
        introspect_functions(pool),
    );

    let schemas = schemas?;
    let tables = tables?;
    let raw_cols = raw_cols?;
    let primary_keys = primary_keys?;
    let indexes = indexes?;
    let foreign_keys = foreign_keys?;
    let check_constraints = check_constraints?;
    let views = views?;
    let triggers = triggers?;
    let functions = functions?;

    // Build PK column set from already-fetched primary_keys (no extra query)
    let pk_column_set: HashSet<(String, String, String)> = primary_keys
        .iter()
        .flat_map(|pk| {
            pk.columns
                .iter()
                .map(move |col| (pk.schema.clone(), pk.table_name.clone(), col.clone()))
        })
        .collect();

    let columns = raw_cols
        .into_iter()
        .map(|(schema, table, name, data_type, nullable, default)| {
            let is_pk = pk_column_set.contains(&(schema.clone(), table.clone(), name.clone()));
            Column {
                name,
                data_type,
                nullable,
                default,
                is_primary_key: is_pk,
                table_name: table,
                schema,
            }
        })
        .collect();

    Ok(IntrospectResult {
        schemas,
        tables,
        columns,
        primary_keys,
        indexes,
        foreign_keys,
        check_constraints,
        views,
        triggers,
        functions,
    })
}

async fn introspect_schemas(pool: &sqlx::PgPool) -> Result<Vec<Schema>, DbError> {
    let rows = sqlx::query(
        r#"
        SELECT schema_name
        FROM information_schema.schemata
        WHERE schema_name NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
        ORDER BY schema_name
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::from_sqlx)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let name: String = row.get("schema_name");
            Schema { name }
        })
        .collect())
}

async fn introspect_tables(pool: &sqlx::PgPool) -> Result<Vec<Table>, DbError> {
    // Fetch tables from information_schema
    let rows = sqlx::query(
        r#"
        SELECT table_name, table_schema
        FROM information_schema.tables
        WHERE table_type = 'BASE TABLE'
          AND table_schema NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
        ORDER BY table_schema, table_name
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::from_sqlx)?;

    // Fetch approximate row counts from pg_class + pg_namespace
    let count_rows = sqlx::query(
        r#"
        SELECT n.nspname AS schema_name, c.relname AS table_name, c.reltuples AS row_count
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relkind = 'r'
          AND n.nspname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::from_sqlx)?;

    let mut row_counts: std::collections::HashMap<(String, String), Option<u64>> = std::collections::HashMap::new();
    for row in count_rows {
        let schema: String = row.get("schema_name");
        let table: String = row.get("table_name");
        let raw: f32 = row.get("row_count");
        let count = if raw < 0.0 { None } else { Some(raw as u64) };
        row_counts.insert((schema, table), count);
    }

    Ok(rows
        .into_iter()
        .map(|row| {
            let name: String = row.get("table_name");
            let schema: String = row.get("table_schema");
            let row_count = row_counts.get(&(schema.clone(), name.clone())).copied().flatten();
            Table {
                name,
                schema,
                row_count,
            }
        })
        .collect())
}

/// Raw column row including schema/table for PK matching.
type RawColumn = (String, String, String, String, bool, Option<String>);

async fn introspect_columns_raw(pool: &sqlx::PgPool) -> Result<Vec<RawColumn>, DbError> {
    let rows = sqlx::query(
        r#"
        SELECT
            n.nspname AS table_schema,
            c.relname AS table_name,
            a.attname AS column_name,
            pg_catalog.format_type(a.atttypid, a.atttypmod) AS data_type,
            NOT a.attnotnull AS is_nullable,
            pg_get_expr(d.adbin, d.adrelid) AS column_default
        FROM pg_attribute a
        JOIN pg_class c ON a.attrelid = c.oid
        JOIN pg_namespace n ON c.relnamespace = n.oid
        LEFT JOIN pg_attrdef d ON a.attrelid = d.adrelid AND a.attnum = d.adnum
        WHERE n.nspname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
          AND a.attnum > 0
          AND NOT a.attisdropped
        ORDER BY n.nspname, c.relname, a.attnum
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::from_sqlx)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let table_schema: String = row.get("table_schema");
            let table_name: String = row.get("table_name");
            let column_name: String = row.get("column_name");
            let data_type: String = row.get("data_type");
            let nullable: bool = row.get("is_nullable");
            let default: Option<String> = row.get("column_default");
            (table_schema, table_name, column_name, data_type, nullable, default)
        })
        .collect())
}

async fn introspect_primary_keys(pool: &sqlx::PgPool) -> Result<Vec<PrimaryKey>, DbError> {
    let rows = sqlx::query(
        r#"
        SELECT tc.table_schema, tc.table_name, tc.constraint_name, kcu.column_name
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
            ON tc.constraint_name = kcu.constraint_name
            AND tc.table_schema = kcu.table_schema
        WHERE tc.constraint_type = 'PRIMARY KEY'
          AND tc.table_schema NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
        ORDER BY tc.table_schema, tc.table_name, tc.constraint_name, kcu.ordinal_position
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::from_sqlx)?;

    // Group columns by (schema, table, constraint_name) to support composite keys
    let mut map: std::collections::HashMap<(String, String, String), Vec<String>> = std::collections::HashMap::new();
    let mut order: Vec<(String, String, String)> = Vec::new();

    for row in rows {
        let schema: String = row.get("table_schema");
        let table: String = row.get("table_name");
        let constraint_name: String = row.get("constraint_name");
        let column_name: String = row.get("column_name");
        let key = (schema.clone(), table.clone(), constraint_name.clone());
        if !map.contains_key(&key) {
            order.push(key.clone());
        }
        map.entry(key).or_default().push(column_name);
    }

    Ok(order
        .into_iter()
        .map(|(schema, table, name)| {
            let columns = map
                .remove(&(schema.clone(), table.clone(), name.clone()))
                .unwrap_or_default();
            PrimaryKey {
                constraint_name: name,
                columns,
                table_name: table,
                schema,
            }
        })
        .collect())
}

async fn introspect_indexes(pool: &sqlx::PgPool) -> Result<Vec<Index>, DbError> {
    let rows = sqlx::query(
        r#"
        SELECT schemaname, tablename, indexname, indexdef
        FROM pg_indexes
        WHERE schemaname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
        ORDER BY schemaname, tablename, indexname
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::from_sqlx)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let schema: String = row.get("schemaname");
            let table: String = row.get("tablename");
            let name: String = row.get("indexname");
            let indexdef: String = row.try_get("indexdef").unwrap_or_default();
            let unique = indexdef.starts_with("CREATE UNIQUE INDEX");

            let columns = parse_index_columns(&indexdef);

            Index {
                name,
                columns,
                unique,
                table_name: table,
                schema,
            }
        })
        .collect())
}

/// Extract column names from a PostgreSQL index definition string.
///
/// Handles expressions like `USING btree (col1, col2)` or `USING hash (col1)`.
/// Also handles functional indexes with parenthesized expressions by tracking
/// parenthesis depth.
fn parse_index_columns(indexdef: &str) -> Vec<String> {
    let bytes = indexdef.as_bytes();
    let mut depth: i32 = 0;
    let mut last_top_level_open = None;

    for (i, &byte) in bytes.iter().enumerate() {
        match byte {
            b'(' => {
                if depth == 0 {
                    last_top_level_open = Some(i);
                }
                depth += 1;
            }
            b')' => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    let Some(open) = last_top_level_open else {
        return Vec::new();
    };

    let mut depth: i32 = 0;
    let mut close = None;
    for (i, &byte) in bytes.iter().enumerate().skip(open) {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }

    let Some(close) = close else {
        return Vec::new();
    };

    let col_str = &indexdef[open + 1..close];

    let mut columns = Vec::new();
    let mut current = String::new();
    let mut depth: i32 = 0;

    for ch in col_str.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    columns.push(trimmed);
                }
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        columns.push(trimmed);
    }

    columns
}

async fn introspect_foreign_keys(pool: &sqlx::PgPool) -> Result<Vec<ForeignKey>, DbError> {
    let rows = sqlx::query(
        r#"
        SELECT
            con.conname AS constraint_name,
            nsp.nspname AS from_schema,
            cls.relname AS from_table,
            src_att.attname AS from_column,
            fnsp.nspname AS to_schema,
            fcls.relname AS to_table,
            dst_att.attname AS to_column
        FROM pg_constraint con
        JOIN pg_namespace nsp ON nsp.oid = con.connamespace
        JOIN pg_class cls ON cls.oid = con.conrelid
        JOIN pg_class fcls ON fcls.oid = con.confrelid
        JOIN pg_namespace fnsp ON fnsp.oid = fcls.relnamespace
        JOIN LATERAL unnest(con.conkey) WITH ORDINALITY AS src(attnum, ord) ON true
        JOIN LATERAL unnest(con.confkey) WITH ORDINALITY AS dst(attnum, ord)
            ON dst.ord = src.ord
        JOIN pg_attribute src_att
            ON src_att.attrelid = cls.oid AND src_att.attnum = src.attnum
        JOIN pg_attribute dst_att
            ON dst_att.attrelid = fcls.oid AND dst_att.attnum = dst.attnum
        WHERE con.contype = 'f'
          AND nsp.nspname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
        ORDER BY nsp.nspname, cls.relname, con.conname, src.ord
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::from_sqlx)?;

    // Group columns by constraint name to support composite foreign keys
    #[allow(clippy::type_complexity)]
    let mut map: std::collections::HashMap<(String, String, String, String, String), (Vec<String>, Vec<String>)> =
        std::collections::HashMap::new();
    let mut order: Vec<(String, String, String, String, String)> = Vec::new();

    for row in rows {
        let name: String = row.get("constraint_name");
        let from_table: String = row.get("from_table");
        let from_column: String = row.get("from_column");
        let to_table: String = row.get("to_table");
        let to_column: String = row.get("to_column");
        let schema: String = row.get("from_schema");
        let to_schema: String = row.get("to_schema");

        let key = (
            name.clone(),
            from_table.clone(),
            to_table.clone(),
            schema.clone(),
            to_schema.clone(),
        );

        if !map.contains_key(&key) {
            order.push(key.clone());
            map.insert(key.clone(), (Vec::new(), Vec::new()));
        }

        let (from_cols, to_cols) = map.get_mut(&key).unwrap();
        from_cols.push(from_column);
        to_cols.push(to_column);
    }

    Ok(order
        .into_iter()
        .map(|(name, from_table, to_table, schema, to_schema)| {
            let (from_columns, to_columns) = map
                .remove(&(
                    name.clone(),
                    from_table.clone(),
                    to_table.clone(),
                    schema.clone(),
                    to_schema.clone(),
                ))
                .unwrap_or_default();
            ForeignKey {
                name,
                from_table,
                from_columns,
                to_table,
                to_columns,
                schema,
                to_schema,
            }
        })
        .collect())
}

async fn introspect_check_constraints(pool: &sqlx::PgPool) -> Result<Vec<CheckConstraint>, DbError> {
    let rows = sqlx::query(
        r#"
        SELECT
            con.conname AS constraint_name,
            nsp.nspname AS schema_name,
            cls.relname AS table_name,
            pg_get_constraintdef(con.oid) AS definition
        FROM pg_constraint con
        JOIN pg_namespace nsp ON nsp.oid = con.connamespace
        JOIN pg_class cls ON cls.oid = con.conrelid
        WHERE con.contype = 'c'
          AND nsp.nspname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
        ORDER BY nsp.nspname, cls.relname, con.conname
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::from_sqlx)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let name: String = row.get("constraint_name");
            let schema: String = row.get("schema_name");
            let table_name: String = row.get("table_name");
            let definition: String = row.get("definition");
            CheckConstraint {
                name,
                table_name,
                schema,
                definition,
            }
        })
        .collect())
}

async fn introspect_views(pool: &sqlx::PgPool) -> Result<Vec<View>, DbError> {
    let rows = sqlx::query(
        r#"
        SELECT table_schema, table_name, view_definition
        FROM information_schema.views
        WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::from_sqlx)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let schema: String = row.get("table_schema");
            let name: String = row.get("table_name");
            // view_definition can be NULL for some edge-case views
            // (e.g. information_schema views with insufficient privileges).
            let definition: String = row.try_get("view_definition").unwrap_or_default();
            View {
                name,
                schema,
                definition,
            }
        })
        .collect())
}

async fn introspect_triggers(pool: &sqlx::PgPool) -> Result<Vec<Trigger>, DbError> {
    let rows = sqlx::query(
        r#"
        SELECT
            t.trigger_name,
            t.event_object_table,
            t.event_object_schema,
            t.action_timing,
            t.event_manipulation,
            t.action_statement,
            COALESCE(pg_t.tgenabled, 'O') AS enabled_flag,
            COALESCE(pg_get_functiondef(pg_proc.oid), '') AS function_def
        FROM information_schema.triggers t
        LEFT JOIN (
            pg_trigger pg_t
            JOIN pg_class c ON c.oid = pg_t.tgrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
        ) ON pg_t.tgname = t.trigger_name
            AND n.nspname = t.event_object_schema
            AND c.relname = t.event_object_table
        LEFT JOIN pg_proc ON pg_proc.oid = pg_t.tgfoid
        WHERE t.trigger_schema NOT IN ('pg_catalog', 'information_schema')
        ORDER BY t.event_object_schema, t.event_object_table, t.trigger_name
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::from_sqlx)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let name: String = row.get("trigger_name");
            let table_name: String = row.try_get("event_object_table").unwrap_or_default();
            let schema: String = row.try_get("event_object_schema").unwrap_or_default();
            let timing: String = row.try_get("action_timing").unwrap_or_default();
            let event: String = row.try_get("event_manipulation").unwrap_or_default();
            let definition: String = row.try_get("action_statement").unwrap_or_default();
            let enabled_flag: String = row.try_get("enabled_flag").unwrap_or_else(|_| "O".into());
            let enabled = enabled_flag != "D";
            let function_def: String = row.try_get("function_def").unwrap_or_default();
            Trigger {
                name,
                table_name,
                schema,
                timing,
                event,
                definition,
                function_def,
                enabled,
            }
        })
        .collect())
}

async fn introspect_functions(pool: &sqlx::PgPool) -> Result<Vec<Function>, DbError> {
    let rows = sqlx::query(
        r#"
        SELECT routine_name, routine_type, data_type
        FROM information_schema.routines
        WHERE routine_schema NOT IN ('pg_catalog', 'information_schema')
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::from_sqlx)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let name: String = row.get("routine_name");
            let routine_type: String = row.try_get("routine_type").unwrap_or_default();
            let data_type: String = row.try_get("data_type").unwrap_or_default();
            Function {
                name,
                routine_type,
                data_type,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_columns() {
        let indexdef = "CREATE INDEX idx ON tbl USING btree (col1, col2)";
        let cols = parse_index_columns(indexdef);
        assert_eq!(cols, vec!["col1", "col2"]);
    }

    #[test]
    fn test_parse_single_column() {
        let indexdef = "CREATE INDEX idx ON tbl USING hash (id)";
        let cols = parse_index_columns(indexdef);
        assert_eq!(cols, vec!["id"]);
    }

    #[test]
    fn test_parse_functional_index() {
        let indexdef = "CREATE INDEX idx ON tbl USING btree (lower(name))";
        let cols = parse_index_columns(indexdef);
        assert_eq!(cols, vec!["lower(name)"]);
    }

    #[test]
    fn test_parse_nested_parentheses() {
        let indexdef = "CREATE INDEX idx ON tbl USING btree (lower(name), (a + b))";
        let cols = parse_index_columns(indexdef);
        assert_eq!(cols, vec!["lower(name)", "(a + b)"]);
    }

    #[test]
    fn test_parse_function_with_multiple_args() {
        let indexdef = "CREATE INDEX idx ON tbl USING btree (coalesce(a, b), c)";
        let cols = parse_index_columns(indexdef);
        assert_eq!(cols, vec!["coalesce(a, b)", "c"]);
    }

    #[test]
    fn test_parse_mixed_columns_and_functions() {
        let indexdef = "CREATE INDEX idx ON tbl USING btree (col1, lower(col2), (a * b + c))";
        let cols = parse_index_columns(indexdef);
        assert_eq!(cols, vec!["col1", "lower(col2)", "(a * b + c)"]);
    }

    #[test]
    fn test_parse_deeply_nested() {
        let indexdef = "CREATE INDEX idx ON tbl USING btree (func1(func2(x, y), z))";
        let cols = parse_index_columns(indexdef);
        assert_eq!(cols, vec!["func1(func2(x, y), z)"]);
    }

    #[test]
    fn test_parse_empty_parentheses() {
        let indexdef = "CREATE INDEX idx ON tbl USING btree ()";
        let cols = parse_index_columns(indexdef);
        assert_eq!(cols, Vec::<String>::new());
    }

    #[test]
    fn test_parse_no_parentheses() {
        let indexdef = "CREATE INDEX idx ON tbl";
        let cols = parse_index_columns(indexdef);
        assert_eq!(cols, Vec::<String>::new());
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let indexdef = "CREATE INDEX idx ON tbl USING btree (  col1  ,  col2  )";
        let cols = parse_index_columns(indexdef);
        assert_eq!(cols, vec!["col1", "col2"]);
    }

    #[test]
    fn test_parse_expression_with_spaces() {
        let indexdef = "CREATE INDEX idx ON tbl USING btree ((a + b) * c, lower(d))";
        let cols = parse_index_columns(indexdef);
        assert_eq!(cols, vec!["(a + b) * c", "lower(d)"]);
    }

    #[test]
    #[allow(clippy::type_complexity)]
    fn test_composite_fk_grouping() {
        // Simulate the grouping logic from introspect_foreign_keys
        let mut map: std::collections::HashMap<(String, String, String, String, String), (Vec<String>, Vec<String>)> =
            std::collections::HashMap::new();
        let mut order: Vec<(String, String, String, String, String)> = Vec::new();

        // Simulate rows for a composite FK with 2 columns
        let rows = vec![
            (
                "fk_composite".to_string(),
                "orders".to_string(),
                "customers".to_string(),
                "public".to_string(),
                "public".to_string(),
                "customer_id".to_string(),
                "id".to_string(),
            ),
            (
                "fk_composite".to_string(),
                "orders".to_string(),
                "customers".to_string(),
                "public".to_string(),
                "public".to_string(),
                "tenant_id".to_string(),
                "tenant_id".to_string(),
            ),
        ];

        for (name, from_table, to_table, schema, to_schema, from_col, to_col) in rows {
            let key = (
                name.clone(),
                from_table.clone(),
                to_table.clone(),
                schema.clone(),
                to_schema.clone(),
            );
            if !map.contains_key(&key) {
                order.push(key.clone());
                map.insert(key.clone(), (Vec::new(), Vec::new()));
            }
            let (from_cols, to_cols) = map.get_mut(&key).unwrap();
            from_cols.push(from_col);
            to_cols.push(to_col);
        }

        assert_eq!(order.len(), 1);
        let key = &order[0];
        let (from_columns, to_columns) = map.get(key).unwrap();
        assert_eq!(from_columns, &vec!["customer_id".to_string(), "tenant_id".to_string()]);
        assert_eq!(to_columns, &vec!["id".to_string(), "tenant_id".to_string()]);
    }

    #[test]
    #[allow(clippy::type_complexity)]
    fn test_multiple_separate_fks() {
        let mut map: std::collections::HashMap<(String, String, String, String, String), (Vec<String>, Vec<String>)> =
            std::collections::HashMap::new();
        let mut order: Vec<(String, String, String, String, String)> = Vec::new();

        // Simulate two separate FKs
        let rows = vec![
            (
                "fk_user".to_string(),
                "orders".to_string(),
                "users".to_string(),
                "public".to_string(),
                "public".to_string(),
                "user_id".to_string(),
                "id".to_string(),
            ),
            (
                "fk_product".to_string(),
                "orders".to_string(),
                "products".to_string(),
                "public".to_string(),
                "public".to_string(),
                "product_id".to_string(),
                "id".to_string(),
            ),
        ];

        for (name, from_table, to_table, schema, to_schema, from_col, to_col) in rows {
            let key = (
                name.clone(),
                from_table.clone(),
                to_table.clone(),
                schema.clone(),
                to_schema.clone(),
            );
            if !map.contains_key(&key) {
                order.push(key.clone());
                map.insert(key.clone(), (Vec::new(), Vec::new()));
            }
            let (from_cols, to_cols) = map.get_mut(&key).unwrap();
            from_cols.push(from_col);
            to_cols.push(to_col);
        }

        assert_eq!(order.len(), 2);
        assert_eq!(map.get(&order[0]).unwrap().0, vec!["user_id".to_string()]);
        assert_eq!(map.get(&order[1]).unwrap().0, vec!["product_id".to_string()]);
    }
}
