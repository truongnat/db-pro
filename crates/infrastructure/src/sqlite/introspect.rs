use db_pro_core::domain::error::DbError;
use db_pro_core::domain::schema::*;
use rusqlite::OptionalExtension;
use std::collections::hash_map::Entry;

/// Escape a SQLite identifier by doubling any embedded double-quote characters
/// and wrapping in double quotes. This is safe for interpolation into PRAGMA
/// statements, which do not support `?` parameter binding for table names.
fn escape_identifier(name: &str) -> String {
    let escaped = name.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

pub fn run_introspection(conn: &rusqlite::Connection) -> Result<IntrospectResult, DbError> {
    let table_names = fetch_table_names(conn)?;
    let tables = introspect_tables(conn, &table_names)?;
    let columns = introspect_columns(conn, &table_names)?;
    let indexes = introspect_indexes(conn, &table_names)?;
    let foreign_keys = introspect_foreign_keys(conn, &table_names)?;
    let check_constraints = introspect_check_constraints(conn, &table_names)?;
    let views = introspect_views(conn)?;
    let triggers = introspect_triggers(conn)?;

    // Derive primary keys from already-fetched columns (no extra PRAGMA calls)
    let mut pk_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for col in &columns {
        if col.is_primary_key {
            pk_map.entry(col.table_name.clone()).or_default().push(col.name.clone());
        }
    }
    let primary_keys = pk_map
        .into_iter()
        .map(|(table_name, columns)| PrimaryKey {
            constraint_name: format!("{table_name}_pk"),
            columns,
            table_name,
            schema: "main".into(),
        })
        .collect();

    let schemas = vec![Schema { name: "main".into() }];

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
        functions: Vec::new(),
    })
}

fn fetch_table_names(conn: &rusqlite::Connection) -> Result<Vec<String>, DbError> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .map_err(crate::error::from_rusqlite)?;
    let names: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(crate::error::from_rusqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(crate::error::from_rusqlite)?;
    Ok(names)
}

fn introspect_tables(_conn: &rusqlite::Connection, table_names: &[String]) -> Result<Vec<Table>, DbError> {
    Ok(table_names
        .iter()
        .map(|name| Table {
            name: name.clone(),
            schema: "main".into(),
            row_count: None,
        })
        .collect())
}

fn introspect_columns(conn: &rusqlite::Connection, table_names: &[String]) -> Result<Vec<Column>, DbError> {
    let mut columns = Vec::new();
    for table_name in table_names {
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

fn introspect_indexes(conn: &rusqlite::Connection, table_names: &[String]) -> Result<Vec<Index>, DbError> {
    let mut indexes = Vec::new();
    for table_name in table_names {
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

fn sqlite_primary_key_columns(conn: &rusqlite::Connection, table_name: &str) -> Result<Vec<String>, DbError> {
    let safe_name = escape_identifier(table_name);
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({safe_name})"))
        .map_err(crate::error::from_rusqlite)?;
    let mut columns: Vec<(i32, String)> = stmt
        .query_map([], |row| {
            let name: String = row.get(1)?;
            let pk_ordinal: i32 = row.get(5)?;
            Ok((pk_ordinal, name))
        })
        .map_err(crate::error::from_rusqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(crate::error::from_rusqlite)?;

    columns.retain(|(pk_ordinal, _)| *pk_ordinal > 0);
    columns.sort_by_key(|(pk_ordinal, _)| *pk_ordinal);
    Ok(columns.into_iter().map(|(_, name)| name).collect())
}

fn introspect_foreign_keys(conn: &rusqlite::Connection, table_names: &[String]) -> Result<Vec<ForeignKey>, DbError> {
    let mut foreign_keys = Vec::new();
    let mut referenced_pk_cache: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for table_name in table_names {
        // PRAGMA does not support ? parameters for table names;
        // use safe identifier escaping instead.
        let safe_name = escape_identifier(table_name);
        let mut stmt = conn
            .prepare(&format!("PRAGMA foreign_key_list({safe_name})"))
            .map_err(crate::error::from_rusqlite)?;

        // SQLite returns NULL for the referenced column when the schema uses
        // shorthand syntax such as `REFERENCES parent`. Keep the FK sequence so
        // we can resolve that omitted target against the parent primary key.
        let rows: Vec<(i32, i32, String, String, Option<String>)> = stmt
            .query_map([], |row| {
                let id: i32 = row.get(0)?;
                let seq: i32 = row.get(1)?;
                let to_table: String = row.get(2)?;
                let from_column: String = row.get(3)?;
                let to_column: Option<String> = row.get(4)?;
                Ok((id, seq, to_table, from_column, to_column))
            })
            .map_err(crate::error::from_rusqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(crate::error::from_rusqlite)?;

        // Group columns by FK id to support composite foreign keys while keeping
        // the PRAGMA encounter order deterministic.
        let mut map: std::collections::HashMap<i32, (String, Vec<String>, Vec<String>)> =
            std::collections::HashMap::new();
        let mut order: Vec<i32> = Vec::new();

        for (id, seq, to_table, from_column, to_column) in rows {
            let resolved_to_column = match to_column {
                Some(column) => column,
                None => {
                    let pk_columns = if let Some(columns) = referenced_pk_cache.get(&to_table) {
                        columns
                    } else {
                        let columns = sqlite_primary_key_columns(conn, &to_table)?;
                        referenced_pk_cache.entry(to_table.clone()).or_insert(columns)
                    };
                    pk_columns.get(seq as usize).cloned().ok_or_else(|| {
                        DbError::IntrospectionFailed(format!(
                            "foreign key {table_name}_fk_{id} references {to_table} without a resolvable primary-key column at position {seq}"
                        ))
                    })?
                }
            };

            let (_, from_cols, to_cols) = match map.entry(id) {
                Entry::Vacant(entry) => {
                    order.push(id);
                    entry.insert((to_table, Vec::new(), Vec::new()))
                }
                Entry::Occupied(entry) => entry.into_mut(),
            };
            from_cols.push(from_column);
            to_cols.push(resolved_to_column);
        }

        for id in order {
            let (to_table, from_columns, to_columns) = map.remove(&id).unwrap();
            foreign_keys.push(ForeignKey {
                name: format!("{table_name}_fk_{id}"),
                from_table: table_name.clone(),
                from_columns,
                to_table,
                to_columns,
                schema: "main".into(),
                to_schema: "main".into(),
            });
        }
    }
    Ok(foreign_keys)
}

fn introspect_check_constraints(
    conn: &rusqlite::Connection,
    table_names: &[String],
) -> Result<Vec<CheckConstraint>, DbError> {
    let mut check_constraints = Vec::new();

    for table_name in table_names {
        // Get the CREATE TABLE SQL from sqlite_master
        let mut stmt = conn
            .prepare("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1")
            .map_err(crate::error::from_rusqlite)?;

        let create_sql: Option<String> = stmt
            .query_row([table_name], |row| row.get(0))
            .optional()
            .map_err(crate::error::from_rusqlite)?;

        if let Some(sql) = create_sql {
            // Parse CHECK constraints from the CREATE TABLE statement
            // This is a simplified parser that looks for CHECK(...) patterns
            let mut depth = 0;
            let mut in_check = false;
            let mut check_start = 0;
            let mut constraint_idx = 0;

            for (i, ch) in sql.char_indices() {
                match ch {
                    '(' => {
                        if !in_check && i >= 5 {
                            let preceding = &sql[check_start..i].trim();
                            if preceding.ends_with("CHECK") {
                                in_check = true;
                                check_start = i + 1;
                            }
                        }
                        depth += 1;
                    }
                    ')' => {
                        depth -= 1;
                        if in_check && depth == 0 {
                            let definition = sql[check_start..i].trim().to_string();
                            check_constraints.push(CheckConstraint {
                                name: format!("{table_name}_check_{constraint_idx}"),
                                table_name: table_name.clone(),
                                schema: "main".into(),
                                definition,
                            });
                            constraint_idx += 1;
                            in_check = false;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(check_constraints)
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
                schema: "main".into(),
                timing,
                event,
                definition,
                function_def: String::new(),
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
///
/// The search skips past double-quoted identifiers that may contain the
/// literal text "BEGIN" (e.g. a trigger named `"trg_BEGIN_audit"`).
fn parse_sqlite_trigger_sql(sql: &str) -> (String, String) {
    let upper = sql.to_uppercase();

    // Isolate the trigger header (everything before the standalone BEGIN keyword).
    // We walk forward past any double-quoted identifiers to avoid matching "BEGIN"
    // that appears inside quoted names like "trg_BEGIN_audit".
    let header = find_trigger_header(&upper).unwrap_or(upper.as_str());

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

/// Find the header portion of a CREATE TRIGGER statement by locating the
/// standalone BEGIN keyword, skipping past double-quoted identifiers.
fn find_trigger_header(upper: &str) -> Option<&str> {
    let mut search_from = 0;
    loop {
        let rel_pos = upper[search_from..].find(" BEGIN")?;
        let abs_pos = search_from + rel_pos;
        // Check whether the character right after " BEGIN" is inside a
        // double-quoted identifier that started before our match.
        let after_begin = abs_pos + " BEGIN".len();
        let preceding = &upper[..abs_pos];
        let quote_count = preceding.chars().filter(|&c| c == '"').count();
        if quote_count % 2 == 0 {
            // All quotes are balanced — this BEGIN is a standalone keyword.
            return Some(&upper[..abs_pos]);
        }
        // Odd quote count means we are inside a quoted identifier; skip past it.
        if let Some(end_quote) = upper[after_begin..].find('"') {
            search_from = after_begin + end_quote + 1;
        } else {
            // No closing quote found — fall back to this match.
            return Some(&upper[..abs_pos]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn introspection_resolves_shorthand_foreign_key_to_parent_primary_key() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE parent (id INTEGER PRIMARY KEY, name TEXT);\
             CREATE TABLE child (id INTEGER PRIMARY KEY, parent_id INTEGER REFERENCES parent);",
        )
        .unwrap();

        let result = run_introspection(&conn).unwrap();
        let foreign_key = result
            .foreign_keys
            .iter()
            .find(|fk| fk.from_table == "child")
            .expect("child shorthand foreign key should be introspected");

        assert_eq!(foreign_key.from_columns, vec!["parent_id"]);
        assert_eq!(foreign_key.to_table, "parent");
        assert_eq!(foreign_key.to_columns, vec!["id"]);
    }

    #[test]
    fn parse_standard_after_insert_trigger() {
        let sql = "CREATE TRIGGER tr_audit AFTER INSERT ON users BEGIN SELECT 1; END";
        let (timing, event) = parse_sqlite_trigger_sql(sql);
        assert_eq!(timing, "AFTER");
        assert_eq!(event, "INSERT");
    }

    #[test]
    fn parse_before_update_trigger() {
        let sql = "CREATE TRIGGER tr_validate BEFORE UPDATE ON orders BEGIN SELECT 1; END";
        let (timing, event) = parse_sqlite_trigger_sql(sql);
        assert_eq!(timing, "BEFORE");
        assert_eq!(event, "UPDATE");
    }

    #[test]
    fn parse_instead_of_delete_trigger() {
        let sql = "CREATE TRIGGER tr_block INSTEAD OF DELETE ON users BEGIN SELECT 1; END";
        let (timing, event) = parse_sqlite_trigger_sql(sql);
        assert_eq!(timing, "INSTEAD OF");
        assert_eq!(event, "DELETE");
    }

    #[test]
    fn parse_trigger_with_begin_in_name() {
        // The trigger name contains "BEGIN" which should NOT be treated as the keyword.
        let sql = "CREATE TRIGGER trg_BEGIN_audit AFTER INSERT ON users BEGIN SELECT 1; END";
        let (timing, event) = parse_sqlite_trigger_sql(sql);
        assert_eq!(timing, "AFTER");
        assert_eq!(event, "INSERT");
    }

    #[test]
    fn parse_trigger_with_quoted_begin_in_name() {
        // Quoted identifier containing "BEGIN" — must be skipped.
        let sql = r#"CREATE TRIGGER "trg_BEGIN_audit" AFTER UPDATE ON users BEGIN SELECT 1; END"#;
        let (timing, event) = parse_sqlite_trigger_sql(sql);
        assert_eq!(timing, "AFTER");
        assert_eq!(event, "UPDATE");
    }
}
