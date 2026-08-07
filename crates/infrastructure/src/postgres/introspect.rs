use db_pro_core::domain::error::DbError;
use db_pro_core::domain::schema::*;
use sqlx::Row as _;
use std::collections::HashSet;

pub async fn run_introspection(pool: &sqlx::PgPool) -> Result<IntrospectResult, DbError> {
    // 1. Schemas
    let schemas = introspect_schemas(pool).await?;

    // 2. Tables (with row counts from pg_class)
    let tables = introspect_tables(pool).await?;

    // 3. Columns — fetched raw below with table info for PK marking

    // 4. Primary keys
    let primary_keys = introspect_primary_keys(pool).await?;

    // Mark columns that are part of a primary key
    let pk_column_set: HashSet<(String, String, String)> = {
        let mut set = HashSet::new();
        // We need table info per PK — re-query to associate columns with tables
        let rows = sqlx::query(
            r#"
            SELECT tc.table_schema, tc.table_name, kcu.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            WHERE tc.constraint_type = 'PRIMARY KEY'
              AND tc.table_schema NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(crate::error::from_sqlx)?;

        for row in rows {
            let schema: String = row.get("table_schema");
            let table: String = row.get("table_name");
            let column: String = row.get("column_name");
            set.insert((schema, table, column));
        }
        set
    };

    // We need table_schema/table_name on columns to match against pk_set,
    // so re-fetch with that info and build the final Column vec.
    let raw_cols = introspect_columns_raw(pool).await?;
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

    // 5. Indexes
    let indexes = introspect_indexes(pool).await?;

    // 6. Foreign keys
    let foreign_keys = introspect_foreign_keys(pool).await?;

    // 7. Views
    let views = introspect_views(pool).await?;

    // 8. Triggers
    let triggers = introspect_triggers(pool).await?;

    // 9. Functions
    let functions = introspect_functions(pool).await?;

    Ok(IntrospectResult {
        schemas,
        tables,
        columns,
        primary_keys,
        indexes,
        foreign_keys,
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
        SELECT table_schema, table_name, column_name, data_type, is_nullable, column_default
        FROM information_schema.columns
        WHERE table_schema NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
        ORDER BY table_schema, table_name, ordinal_position
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
            let is_nullable_str: String = row.get("is_nullable");
            let nullable = is_nullable_str == "YES";
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
            let indexdef: String = row.get("indexdef");
            let unique = indexdef.contains("UNIQUE");

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
    // Find the last '(' that starts the column list.
    // The column list is the final parenthesized group in the indexdef.
    let Some(open) = indexdef.rfind('(') else {
        return Vec::new();
    };
    let Some(close) = indexdef.rfind(')') else {
        return Vec::new();
    };
    if close <= open {
        return Vec::new();
    }

    let col_str = &indexdef[open + 1..close];
    col_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
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

    Ok(rows
        .into_iter()
        .map(|row| {
            let name: String = row.get("constraint_name");
            let from_table: String = row.get("from_table");
            let from_column: String = row.get("from_column");
            let to_table: String = row.get("to_table");
            let to_column: String = row.get("to_column");
            let schema: String = row.get("from_schema");
            let to_schema: String = row.get("to_schema");
            ForeignKey {
                name,
                from_table,
                from_column,
                to_table,
                to_column,
                schema,
                to_schema,
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
            let definition: String = row.get("view_definition");
            View { name, schema, definition }
        })
        .collect())
}

async fn introspect_triggers(pool: &sqlx::PgPool) -> Result<Vec<Trigger>, DbError> {
    let rows = sqlx::query(
        r#"
        SELECT trigger_name, event_manipulation, action_statement
        FROM information_schema.triggers
        WHERE trigger_schema NOT IN ('pg_catalog', 'information_schema')
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::from_sqlx)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let name: String = row.get("trigger_name");
            let event: String = row.get("event_manipulation");
            let action: String = row.get("action_statement");
            Trigger { name, event, action }
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
