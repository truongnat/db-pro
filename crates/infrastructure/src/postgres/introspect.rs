use db_pro_core::domain::error::DbError;
use db_pro_core::domain::schema::*;
use sqlx::postgres::PgRow;
use sqlx::Row as _;
use std::collections::HashSet;

/// Introspect a live catalog that other sessions can change under us.
///
/// The queries below call catalog functions (`pg_get_indexdef`, `pg_get_expr`,
/// `pg_get_constraintdef`, …) that are evaluated with a *fresh* catalog snapshot, so
/// an object created or dropped after this query's snapshot either decodes as a NULL
/// column (handled at the decode site) or makes the server raise a transient
/// catalog-cache error. A refresh is a read of whatever consistent-enough state the
/// catalog is in, so only that transient form is retried, a bounded number of times,
/// with a short backoff; every other error is returned unchanged. The failure this
/// replaces was `cache lookup failed for attribute 1 of relation …` and
/// `ColumnDecode { index: "definition", UnexpectedNullError }` — see
/// `docs/release/evidence/v01-runtime/providers/42-introspection-under-concurrent-ddl.md`.
pub async fn run_introspection(pool: &sqlx::PgPool) -> Result<IntrospectResult, DbError> {
    const ATTEMPTS: usize = 3;
    let mut attempt = 1;
    loop {
        match run_introspection_once(pool).await {
            Ok(result) => return Ok(result),
            Err(error) if attempt < ATTEMPTS && is_transient_catalog_error(&error) => {
                tokio::time::sleep(std::time::Duration::from_millis(10 * attempt as u64)).await;
                attempt += 1;
            }
            Err(error) => return Err(error),
        }
    }
}

/// The catalog races PostgreSQL reports while a concurrent session is running DDL.
/// Matching is deliberately narrow: a message this list does not name is a real
/// failure and must not be retried behind the user's back.
fn is_transient_catalog_error(error: &DbError) -> bool {
    let DbError::QueryFailed(message) = error else {
        return false;
    };
    [
        "cache lookup failed",
        "could not open relation with OID",
        "tuple concurrently updated",
        "cached plan must not change result type",
    ]
    .iter()
    .any(|marker| message.contains(marker))
}

async fn run_introspection_once(pool: &sqlx::PgPool) -> Result<IntrospectResult, DbError> {
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
    let unique_column_set: HashSet<(String, String, String)> = indexes
        .iter()
        .filter(|index| (index.unique || index.primary) && index.columns.len() == 1)
        .flat_map(|index| {
            index
                .columns
                .iter()
                .map(move |column| (index.schema.clone(), index.table_name.clone(), column.clone()))
        })
        .collect();

    let columns = raw_cols
        .into_iter()
        .map(
            |(schema, table, name, data_type, ordinal, nullable, default, is_identity, is_generated, collation)| {
                let is_pk = pk_column_set.contains(&(schema.clone(), table.clone(), name.clone()));
                let is_unique = unique_column_set.contains(&(schema.clone(), table.clone(), name.clone()));
                Column {
                    name,
                    data_type,
                    ordinal,
                    nullable,
                    default,
                    is_primary_key: is_pk,
                    is_unique,
                    is_identity,
                    is_generated,
                    collation,
                    table_name: table,
                    schema,
                }
            },
        )
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
type RawColumn = (
    String,
    String,
    String,
    String,
    usize,
    bool,
    Option<String>,
    bool,
    bool,
    Option<String>,
);

async fn introspect_columns_raw(pool: &sqlx::PgPool) -> Result<Vec<RawColumn>, DbError> {
    let rows = sqlx::query(
        r#"
        SELECT
            n.nspname AS table_schema,
            c.relname AS table_name,
            a.attname AS column_name,
            pg_catalog.format_type(a.atttypid, a.atttypmod) AS data_type,
            a.attnum::integer AS ordinal,
            NOT a.attnotnull AS is_nullable,
            pg_get_expr(d.adbin, d.adrelid) AS column_default,
            (a.attidentity <> '') AS is_identity,
            (a.attgenerated <> '') AS is_generated,
            CASE WHEN coll.collname = 'default' THEN NULL ELSE coll.collname END AS collation_name
        FROM pg_attribute a
        JOIN pg_class c ON a.attrelid = c.oid
        JOIN pg_namespace n ON c.relnamespace = n.oid
        LEFT JOIN pg_attrdef d ON a.attrelid = d.adrelid AND a.attnum = d.adnum
        LEFT JOIN pg_collation coll ON a.attcollation = coll.oid
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
            let ordinal: i32 = row.get("ordinal");
            let nullable: bool = row.get("is_nullable");
            let default: Option<String> = row.get("column_default");
            let is_identity: bool = row.get("is_identity");
            let is_generated: bool = row.get("is_generated");
            let collation: Option<String> = row.get("collation_name");
            (
                table_schema,
                table_name,
                column_name,
                data_type,
                usize::try_from(ordinal.max(0)).unwrap_or(0),
                nullable,
                default,
                is_identity,
                is_generated,
                collation,
            )
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
        SELECT
            n.nspname AS schema_name,
            tbl.relname AS table_name,
            idx.relname AS index_name,
            am.amname AS method,
            i.indisprimary AS is_primary,
            i.indisunique AS is_unique,
            pg_get_indexdef(i.indexrelid) AS definition,
            pg_get_expr(i.indpred, i.indrelid) AS predicate,
            ARRAY(
                SELECT att.attname
                FROM unnest(i.indkey) WITH ORDINALITY AS key(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = i.indrelid AND att.attnum = key.attnum
                WHERE key.ord <= i.indnkeyatts
                ORDER BY key.ord
            ) AS columns,
            ARRAY(
                SELECT att.attname
                FROM unnest(i.indkey) WITH ORDINALITY AS key(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = i.indrelid AND att.attnum = key.attnum
                WHERE key.ord > i.indnkeyatts
                ORDER BY key.ord
            ) AS include_columns
        FROM pg_index i
        JOIN pg_class idx ON idx.oid = i.indexrelid
        JOIN pg_class tbl ON tbl.oid = i.indrelid
        JOIN pg_namespace n ON n.oid = tbl.relnamespace
        JOIN pg_am am ON am.oid = idx.relam
        WHERE n.nspname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
        ORDER BY n.nspname, tbl.relname, idx.relname
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::from_sqlx)?;

    rows.into_iter()
        .map(|row| -> Result<Index, DbError> {
            let schema: String = row.get("schema_name");
            let table: String = row.get("table_name");
            let name: String = row.get("index_name");
            let method: String = row.get("method");
            let primary: bool = row.get("is_primary");
            let unique: bool = row.get("is_unique");
            // `pg_get_indexdef` is evaluated with a *fresh* catalog snapshot, so an
            // index created or dropped by another session after this query's
            // snapshot makes the function return NULL for a row the snapshot still
            // shows. A meanwhile-dropped index must not fail the whole refresh.
            let definition = optional_string(&row, "definition")?.unwrap_or_default();
            let predicate: Option<String> = row.get("predicate");
            let mut columns: Vec<String> = row.get("columns");
            if columns.is_empty() && !definition.is_empty() {
                columns = parse_index_columns(&definition);
            }
            let include_columns: Vec<String> = row.get("include_columns");

            Ok(Index {
                name,
                columns,
                unique,
                method,
                primary,
                include_columns,
                predicate,
                definition,
                origin: IndexOrigin::User,
                table_name: table,
                schema,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

/// Extract column names from a PostgreSQL index definition string.
///
/// Handles expressions like `USING btree (col1, col2)` or `USING hash (col1)`.
/// Also handles functional indexes with parenthesized expressions by tracking
/// parenthesis depth.
fn parse_index_columns(indexdef: &str) -> Vec<String> {
    let bytes = indexdef.as_bytes();
    let Some(open) = find_unquoted_open_parenthesis(bytes) else {
        return Vec::new();
    };

    let Some(close) = find_matching_parenthesis(bytes, open) else {
        return Vec::new();
    };

    split_index_columns(&indexdef[open + 1..close])
}

fn find_unquoted_open_parenthesis(bytes: &[u8]) -> Option<usize> {
    let mut quote = None;
    let mut index = 0;

    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(quote_byte) = quote {
            if quote_byte == b'\'' && byte == b'\\' {
                index = index.saturating_add(2);
                continue;
            }
            if byte == quote_byte {
                if bytes.get(index + 1) == Some(&quote_byte) {
                    index = index.saturating_add(2);
                    continue;
                }
                quote = None;
            }
        } else {
            match byte {
                b'\'' | b'"' => quote = Some(byte),
                b'(' => return Some(index),
                _ => {}
            }
        }
        index += 1;
    }

    None
}

fn find_matching_parenthesis(bytes: &[u8], open: usize) -> Option<usize> {
    let mut quote = None;
    let mut depth = 0usize;
    let mut index = open;

    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(quote_byte) = quote {
            if quote_byte == b'\'' && byte == b'\\' {
                index = index.saturating_add(2);
                continue;
            }
            if byte == quote_byte {
                if bytes.get(index + 1) == Some(&quote_byte) {
                    index = index.saturating_add(2);
                    continue;
                }
                quote = None;
            }
        } else {
            match byte {
                b'\'' | b'"' => quote = Some(byte),
                b'(' => depth += 1,
                b')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Some(index);
                    }
                }
                _ => {}
            }
        }

        index += 1;
    }

    None
}

fn split_index_columns(col_str: &str) -> Vec<String> {
    let mut columns = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;

    for ch in col_str.chars() {
        if let Some(quote_char) = quote {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if quote_char == '\'' && ch == '\\' {
                escaped = true;
            } else if ch == quote_char {
                quote = None;
            }
            continue;
        }

        match ch {
            '\'' | '"' => {
                quote = Some(ch);
                current.push(ch);
            }
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

const FOREIGN_KEY_SQL: &str = r#"
        SELECT
            con.conname AS constraint_name,
            nsp.nspname AS from_schema,
            cls.relname AS from_table,
            src_att.attname AS from_column,
            fnsp.nspname AS to_schema,
            fcls.relname AS to_table,
            dst_att.attname AS to_column,
            CASE con.confupdtype
                WHEN 'a' THEN 'NO ACTION'
                WHEN 'r' THEN 'RESTRICT'
                WHEN 'c' THEN 'CASCADE'
                WHEN 'n' THEN 'SET NULL'
                WHEN 'd' THEN 'SET DEFAULT'
                ELSE 'UNKNOWN'
            END AS on_update,
            CASE con.confdeltype
                WHEN 'a' THEN 'NO ACTION'
                WHEN 'r' THEN 'RESTRICT'
                WHEN 'c' THEN 'CASCADE'
                WHEN 'n' THEN 'SET NULL'
                WHEN 'd' THEN 'SET DEFAULT'
                ELSE 'UNKNOWN'
            END AS on_delete,
            CASE con.confmatchtype
                WHEN 'f' THEN 'FULL'
                WHEN 'p' THEN 'PARTIAL'
                ELSE 'SIMPLE'
            END AS match_option,
            con.condeferrable AS deferrable,
            con.condeferred AS initially_deferred
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
        "#;

type ForeignKeyGroupKey = (String, String, String, String, String);
type ForeignKeyGroupValue = (Vec<String>, Vec<String>, String, String, String, bool, bool);

async fn introspect_foreign_keys(pool: &sqlx::PgPool) -> Result<Vec<ForeignKey>, DbError> {
    let rows = sqlx::query(FOREIGN_KEY_SQL)
        .fetch_all(pool)
        .await
        .map_err(crate::error::from_sqlx)?;
    Ok(group_foreign_key_rows(rows))
}

fn group_foreign_key_rows(rows: Vec<sqlx::postgres::PgRow>) -> Vec<ForeignKey> {
    let mut map: std::collections::HashMap<ForeignKeyGroupKey, ForeignKeyGroupValue> = std::collections::HashMap::new();
    let mut order: Vec<ForeignKeyGroupKey> = Vec::new();

    for row in rows {
        let name: String = row.get("constraint_name");
        let from_table: String = row.get("from_table");
        let from_column: String = row.get("from_column");
        let to_table: String = row.get("to_table");
        let to_column: String = row.get("to_column");
        let schema: String = row.get("from_schema");
        let to_schema: String = row.get("to_schema");
        let on_update: String = row.get("on_update");
        let on_delete: String = row.get("on_delete");
        let match_option: String = row.get("match_option");
        let deferrable: bool = row.get("deferrable");
        let initially_deferred: bool = row.get("initially_deferred");

        let key = (
            name.clone(),
            from_table.clone(),
            to_table.clone(),
            schema.clone(),
            to_schema.clone(),
        );

        let (from_cols, to_cols, _, _, _, _, _) = map.entry(key.clone()).or_insert_with(|| {
            order.push(key.clone());
            (
                Vec::new(),
                Vec::new(),
                on_update.clone(),
                on_delete.clone(),
                match_option.clone(),
                deferrable,
                initially_deferred,
            )
        });
        from_cols.push(from_column);
        to_cols.push(to_column);
    }

    order
        .into_iter()
        .map(|(name, from_table, to_table, schema, to_schema)| {
            let (from_columns, to_columns, on_update, on_delete, match_option, deferrable, initially_deferred) = map
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
                on_update,
                on_delete,
                match_option,
                deferrable,
                initially_deferred,
            }
        })
        .collect()
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

    rows.into_iter()
        .map(|row| -> Result<CheckConstraint, DbError> {
            let name: String = row.get("constraint_name");
            let schema: String = row.get("schema_name");
            let table_name: String = row.get("table_name");
            // Same fresh-snapshot rule as `pg_get_indexdef`: a constraint dropped
            // after this query's snapshot makes `pg_get_constraintdef` return NULL.
            let definition = optional_string(&row, "definition")?.unwrap_or_default();
            Ok(CheckConstraint {
                name,
                table_name,
                schema,
                definition,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

fn required_string(row: &PgRow, column: &str) -> Result<String, DbError> {
    row.try_get(column).map_err(crate::error::from_sqlx)
}

fn optional_string(row: &PgRow, column: &str) -> Result<Option<String>, DbError> {
    row.try_get(column).map_err(crate::error::from_sqlx)
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

    rows.into_iter()
        .map(|row| -> Result<View, DbError> {
            let schema = required_string(&row, "table_schema")?;
            let name = required_string(&row, "table_name")?;
            // view_definition can be NULL for some edge-case views
            // (e.g. information_schema views with insufficient privileges).
            let definition = optional_string(&row, "view_definition")?.unwrap_or_default();
            Ok(View {
                name,
                schema,
                definition,
            })
        })
        .collect()
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
            COALESCE(pg_t.tgenabled, 'O')::text AS enabled_flag,
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

    rows.into_iter()
        .map(|row| -> Result<Trigger, DbError> {
            let name = required_string(&row, "trigger_name")?;
            let table_name = optional_string(&row, "event_object_table")?.unwrap_or_default();
            let schema = optional_string(&row, "event_object_schema")?.unwrap_or_default();
            let timing = optional_string(&row, "action_timing")?.unwrap_or_default();
            let event = optional_string(&row, "event_manipulation")?.unwrap_or_default();
            let definition = optional_string(&row, "action_statement")?.unwrap_or_default();
            let enabled_flag = optional_string(&row, "enabled_flag")?.unwrap_or_else(|| "O".into());
            let enabled = enabled_flag != "D";
            let function_def = optional_string(&row, "function_def")?.unwrap_or_default();
            Ok(Trigger {
                name,
                table_name,
                schema,
                timing,
                event,
                definition,
                function_def,
                enabled,
            })
        })
        .collect()
}

async fn introspect_functions(pool: &sqlx::PgPool) -> Result<Vec<Function>, DbError> {
    let rows = sqlx::query(
        r#"
        SELECT
            n.nspname AS routine_schema,
            p.proname AS routine_name,
            CASE p.prokind
                WHEN 'p' THEN 'PROCEDURE'
                ELSE 'FUNCTION'
            END AS routine_type,
            pg_get_function_result(p.oid) AS data_type,
            COALESCE(pg_get_functiondef(p.oid), '') AS definition,
            COALESCE(pg_get_function_identity_arguments(p.oid), '') AS identity_arguments,
            COALESCE(pg_get_function_arguments(p.oid), '') AS argument_list,
            COALESCE(l.lanname, '') AS language,
            CASE p.provolatile
                WHEN 'i' THEN 'IMMUTABLE'
                WHEN 's' THEN 'STABLE'
                ELSE 'VOLATILE'
            END AS volatility,
            p.prosecdef AS security_definer
        FROM pg_proc p
        JOIN pg_namespace n ON n.oid = p.pronamespace
        LEFT JOIN pg_language l ON l.oid = p.prolang
        WHERE n.nspname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
          AND p.prokind IN ('f', 'p')
        ORDER BY n.nspname, p.proname, pg_get_function_identity_arguments(p.oid)
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::from_sqlx)?;

    rows.into_iter()
        .map(|row| -> Result<Function, DbError> {
            let schema = required_string(&row, "routine_schema")?;
            let name = required_string(&row, "routine_name")?;
            let routine_type = required_string(&row, "routine_type")?;
            let data_type = optional_string(&row, "data_type")?.unwrap_or_default();
            let definition = required_string(&row, "definition")?;
            let identity_arguments = optional_string(&row, "identity_arguments")?.unwrap_or_default();
            let argument_list = optional_string(&row, "argument_list")?.unwrap_or_default();
            let language = optional_string(&row, "language")?.unwrap_or_default();
            let volatility = optional_string(&row, "volatility")?.unwrap_or_default();
            let security_definer = row.try_get::<bool, _>("security_definer").unwrap_or(false);
            Ok(Function {
                name,
                schema,
                routine_type,
                data_type,
                definition,
                identity_arguments,
                language,
                volatility,
                security_definer,
                parameters: parse_routine_parameters(&argument_list),
            })
        })
        .collect()
}

/// Parse `pg_get_function_arguments` text into structured parameters.
///
/// Examples:
/// - `order_id integer, OUT total numeric`
/// - `VARIADIC tags text[] DEFAULT '{}'::text[]`
fn parse_routine_parameters(argument_list: &str) -> Vec<RoutineParameter> {
    let trimmed = argument_list.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let mut params = Vec::new();
    for raw in split_routine_args(trimmed) {
        let part = raw.trim();
        if part.is_empty() {
            continue;
        }
        let mut mode = "IN".to_owned();
        let mut rest = part;
        for candidate in ["VARIADIC ", "INOUT ", "OUT ", "IN "] {
            if rest.len() >= candidate.len() && rest[..candidate.len()].eq_ignore_ascii_case(candidate) {
                mode = candidate.trim().to_ascii_uppercase();
                rest = rest[candidate.len()..].trim_start();
                break;
            }
        }
        let (type_and_name, default_expr, has_default) = if let Some(idx) = find_default_kw(rest) {
            (
                rest[..idx].trim(),
                rest[idx + "DEFAULT".len()..].trim().to_owned(),
                true,
            )
        } else {
            (rest, String::new(), false)
        };
        let (name, data_type) = split_name_and_type(type_and_name);
        params.push(RoutineParameter {
            name,
            data_type,
            mode,
            has_default,
            default_expr,
        });
    }
    params
}

fn split_routine_args(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for ch in input.chars() {
        match ch {
            '(' | '[' => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => {
                out.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        out.push(current);
    }
    out
}

fn find_default_kw(input: &str) -> Option<usize> {
    let upper = input.to_ascii_uppercase();
    upper.find(" DEFAULT ").map(|idx| idx + 1)
}

fn split_name_and_type(input: &str) -> (String, String) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return (String::new(), String::new());
    }
    if parts.len() == 1 {
        return (String::new(), parts[0].to_owned());
    }
    // name type...  OR  only type tokens when anonymous
    let first = parts[0];
    let looks_like_name = first
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && !first.contains('(')
        && !first.eq_ignore_ascii_case("double")
        && !first.eq_ignore_ascii_case("character")
        && !first.eq_ignore_ascii_case("timestamp")
        && !first.eq_ignore_ascii_case("time");
    if looks_like_name {
        (first.to_owned(), parts[1..].join(" "))
    } else {
        (String::new(), parts.join(" "))
    }
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
    fn test_quoted_identifier_with_parenthesis_and_comma() {
        let indexdef = "CREATE INDEX idx ON \"tbl(name)\" USING btree (\"a,b\", \"quoted\"\"name\")";
        let cols = parse_index_columns(indexdef);
        assert_eq!(cols, vec!["\"a,b\"", "\"quoted\"\"name\""]);
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
    fn parse_routine_parameters_handles_modes_and_defaults() {
        let params = parse_routine_parameters("order_id integer, OUT total numeric DEFAULT 0");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "order_id");
        assert_eq!(params[0].mode, "IN");
        assert_eq!(params[1].mode, "OUT");
        assert!(params[1].has_default);
        assert_eq!(params[1].default_expr, "0");
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
