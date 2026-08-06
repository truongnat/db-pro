use db_pro_core::domain::error::DbError;
use db_pro_core::domain::schema::*;

pub fn run_introspection(conn: &rusqlite::Connection) -> Result<IntrospectResult, DbError> {
    let tables = introspect_tables(conn)?;
    let columns = introspect_columns(conn)?;
    let indexes = introspect_indexes(conn)?;
    let foreign_keys = introspect_foreign_keys(conn)?;
    let views = introspect_views(conn)?;
    let triggers = introspect_triggers(conn)?;

    let mut primary_keys = Vec::new();
    for t in &tables {
        let mut stmt = conn
            .prepare(&format!(
                "SELECT name FROM pragma_table_info('{}') WHERE pk > 0 ORDER BY pk",
                t.name
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
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info('{table_name}')"))
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
        let mut stmt = conn
            .prepare(&format!("PRAGMA index_list('{table_name}')"))
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
            let mut info_stmt = conn
                .prepare(&format!("PRAGMA index_info('{index_name}')"))
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
        let mut stmt = conn
            .prepare(&format!("PRAGMA foreign_key_list('{table_name}')"))
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
            Ok(View { name, schema: "main".into(), definition })
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
            let _table: String = row.get(1)?;
            let sql: Option<String> = row.get(2)?;
            Ok(Trigger {
                name,
                event: sql.unwrap_or_default(),
                action: String::new(),
            })
        })
        .map_err(crate::error::from_rusqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(crate::error::from_rusqlite)?;
    Ok(triggers)
}
