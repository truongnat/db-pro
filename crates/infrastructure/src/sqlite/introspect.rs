use db_pro_core::domain::error::DbError;
use db_pro_core::domain::schema::*;

/// Escape a SQLite identifier by doubling any embedded double-quote characters
/// and wrapping in double quotes. This is safe for interpolation into PRAGMA
/// statements, which do not support `?` parameter binding for table names.
fn escape_identifier(name: &str) -> String {
    let escaped = name.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

pub fn run_introspection(conn: &rusqlite::Connection) -> Result<IntrospectResult, DbError> {
    let tables = introspect_tables(conn)?;
    let columns = introspect_columns(conn)?;
    let indexes = introspect_indexes(conn)?;
    let foreign_keys = introspect_foreign_keys(conn)?;
    let views = introspect_views(conn)?;
    let triggers = introspect_triggers(conn)?;

    let mut primary_keys = Vec::new();
    for t in &tables {
        // PRAGMA does not support ? parameters for table names;
        // use safe identifier escaping instead.
        let safe_name = escape_identifier(&t.name);
        let mut stmt = conn
            .prepare(&format!(
                "SELECT name FROM pragma_table_info({safe_name}) WHERE pk > 0 ORDER BY pk"
            ))
            .map_err(crate::error::from_rusqlite)?;
        let pk_cols: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(crate::error::from_rusqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(crate::error::from_rusqlite)?;
        if !pk_cols.is_empty() {
            primary_keys.push(PrimaryKey {
                constraint_name: format!("{}_pk", t.name),
                columns: pk_cols,
                table_name: t.name.clone(),
                schema: "main".into(),
            });
        }
    }

    let schemas = vec![Schema { name: "main".into() }];

    Ok(IntrospectResult {
        schemas,
        tables,
        columns,
        primary_keys,
        indexes,
        foreign_keys,
        views,
        triggers,
        functions: Vec::new(),
    })
}

fn introspect_tables(conn: &rusqlite::Connection) -> Result<Vec<Table>, DbError> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .map_err(crate::error::from_rusqlite)?;
    let tables = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            Ok(Table {
                name,
                schema: "main".into(),
                row_count: None,
            })
        })
        .map_err(crate::error::from_rusqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(crate::error::from_rusqlite)?;
    Ok(tables)
}

fn introspect_columns(conn: &rusqlite::Connection) -> Result<Vec<Column>, DbError> {
    let mut table_stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'")
        .map_err(crate::error::from_rusqlite)?;
    let table_names: Vec<String> = table_stmt
        .query_map([], |row| row.get(0))
        .map_err(crate::error::from_rusqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(crate::error::from_rusqlite)?;

    let mut columns = Vec::new();
    for table_name in &table_names {
        // PRAGMA does not support ? parameters for table names;
        // use safe identifier escaping instead.
        let safe_name = escape_identifier(table_name);
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({safe_name})"))
            .map_err(crate::error::from_rusqlite)?;
        let mut cols: Vec<Column> = stmt
            .query_map([], |row| {
                let name: String = row.get(1)?;
                let data_type: String = row.get(2)?;
                let notnull: bool = row.get(3)?;
                let default: Option<String> = row.get(4)?;
                let pk: bool = row.get::<_, i32>(5).unwrap_or(0) > 0;
                Ok(Column {
                    name,
                    data_type,
                    nullable: !notnull,
                    default,
                    is_primary_key: pk,
                    table_name: table_name.clone(),
                    schema: "main".into(),
                })
            })
            .map_err(crate::error::from_rusqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(crate::error::from_rusqlite)?;
        columns.append(&mut cols);
    }
    Ok(columns)
}

fn introspect_indexes(conn: &rusqlite::Connection) -> Result<Vec<Index>, DbError> {
    let mut table_stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'")
        .map_err(crate::error::from_rusqlite)?;
    let table_names: Vec<String> = table_stmt
        .query_map([], |row| row.get(0))
        .map_err(crate::error::from_rusqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(crate::error::from_rusqlite)?;

    let mut indexes = Vec::new();
    for table_name in &table_names {
        // PRAGMA does not support ? parameters for table names;
        // use safe identifier escaping instead.
        let safe_name = escape_identifier(table_name);
        let mut stmt = conn
            .prepare(&format!("PRAGMA index_list({safe_name})"))
            .map_err(crate::error::from_rusqlite)?;
        let index_list: Vec<(String, bool)> = stmt
            .query_map([], |row| {
                let name: String = row.get(1)?;
                let unique: bool = row.get(2)?;
                Ok((name, unique))
            })
            .map_err(crate::error::from_rusqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(crate::error::from_rusqlite)?;

        for (index_name, unique) in index_list {
            // PRAGMA does not support ? parameters for index names;
            // use safe identifier escaping instead.
            let safe_idx = escape_identifier(&index_name);
            let mut info_stmt = conn
                .prepare(&format!("PRAGMA index_info({safe_idx})"))
                .map_err(crate::error::from_rusqlite)?;
            let cols: Vec<String> = info_stmt
                .query_map([], |row| row.get(2))
                .map_err(crate::error::from_rusqlite)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(crate::error::from_rusqlite)?;
            indexes.push(Index {
                name: index_name,
                columns: cols,
                unique,
                table_name: table_name.clone(),
                schema: "main".into(),
            });
        }
    }
    Ok(indexes)
}

fn introspect_foreign_keys(conn: &rusqlite::Connection) -> Result<Vec<ForeignKey>, DbError> {
    let mut table_stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'")
        .map_err(crate::error::from_rusqlite)?;
    let table_names: Vec<String> = table_stmt
        .query_map([], |row| row.get(0))
        .map_err(crate::error::from_rusqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(crate::error::from_rusqlite)?;

    let mut foreign_keys = Vec::new();
    for table_name in &table_names {
        // PRAGMA does not support ? parameters for table names;
        // use safe identifier escaping instead.
        let safe_name = escape_identifier(table_name);
        let mut stmt = conn
            .prepare(&format!("PRAGMA foreign_key_list({safe_name})"))
            .map_err(crate::error::from_rusqlite)?;
        let fks: Vec<ForeignKey> = stmt
            .query_map([], |row| {
                let _id: i32 = row.get(0)?;
                let _seq: i32 = row.get(1)?;
                let to_table: String = row.get(2)?;
                let from_column: String = row.get(3)?;
                let to_column: String = row.get(4)?;
                Ok(ForeignKey {
                    name: format!("{table_name}_{from_column}_fkey"),
                    from_table: table_name.clone(),
                    from_column,
                    to_table,
                    to_column,
                    schema: "main".into(),
                    to_schema: "main".into(),
                })
            })
            .map_err(crate::error::from_rusqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(crate::error::from_rusqlite)?;
        foreign_keys.extend(fks);
    }
    Ok(foreign_keys)
}

fn introspect_views(conn: &rusqlite::Connection) -> Result<Vec<View>, DbError> {
    let mut stmt = conn
        .prepare("SELECT name, sql FROM sqlite_master WHERE type = 'view' ORDER BY name")
        .map_err(crate::error::from_rusqlite)?;
    let views = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let definition: String = row.get(1)?;
            Ok(View {
                name,
                schema: "main".into(),
                definition,
            })
        })
        .map_err(crate::error::from_rusqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(crate::error::from_rusqlite)?;
    Ok(views)
}

fn introspect_triggers(conn: &rusqlite::Connection) -> Result<Vec<Trigger>, DbError> {
    let mut stmt = conn
        .prepare("SELECT name, tbl_name, sql FROM sqlite_master WHERE type = 'trigger' ORDER BY name")
        .map_err(crate::error::from_rusqlite)?;
    let triggers = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let table_name: String = row.get(1)?;
            let sql: Option<String> = row.get(2)?;
            let definition = sql.clone().unwrap_or_default();
            let (timing, event) = parse_sqlite_trigger_sql(&definition);
            Ok(Trigger {
                name,
                table_name,
                schema: String::new(),
                timing,
                event,
                definition,
                enabled: true,
            })
        })
        .map_err(crate::error::from_rusqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(crate::error::from_rusqlite)?;
    Ok(triggers)
}

/// Parse timing (BEFORE/AFTER/INSTEAD OF) and event (INSERT/UPDATE/DELETE) from
/// a SQLite CREATE TRIGGER SQL body. Returns (timing, event) with empty fallbacks.
///
/// Only the header portion (before BEGIN) is inspected for the event type,
/// because the trigger body may contain DML keywords that would produce
/// false positives (e.g. an AFTER UPDATE trigger whose body does INSERT).
fn parse_sqlite_trigger_sql(sql: &str) -> (String, String) {
    let upper = sql.to_uppercase();

    // Split at the first standalone BEGIN keyword to isolate the trigger header.
    // We search for " BEGIN" (with leading space) to avoid matching trigger names
    // or identifiers that contain "begin" as a substring (e.g. "begin_audit").
    let header = upper.find(" BEGIN").map_or(upper.as_str(), |idx| &upper[..idx]);

    let timing = if header.contains("INSTEAD OF") {
        "INSTEAD OF"
    } else if header.contains("BEFORE") {
        "BEFORE"
    } else if header.contains("AFTER") {
        "AFTER"
    } else {
        ""
    };

    let event = if header.contains("INSERT") {
        "INSERT"
    } else if header.contains("UPDATE") {
        "UPDATE"
    } else if header.contains("DELETE") {
        "DELETE"
    } else {
        ""
    };

    (timing.to_string(), event.to_string())
}
