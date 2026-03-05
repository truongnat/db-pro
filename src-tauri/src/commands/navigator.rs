use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use sqlx::mysql::{MySqlPoolOptions, MySqlRow};
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::Row;

use crate::models::{
    ConnectionProfile, DbEngine, NavigatorColumn, NavigatorObject, NavigatorObjectKind,
    NavigatorSchema, NavigatorTree,
};
use crate::state::AppState;

use super::shared::{
    get_profile, sqlite_connect_options, to_connection_url, with_timeout,
    DEFAULT_NAVIGATOR_TIMEOUT_MS,
};

const NAVIGATOR_CONNECT_TIMEOUT_SECONDS: u64 = 5;
const POSTGRES_STATEMENT_TIMEOUT_MS: u64 = 8_000;

type ObjectRecord = (String, String, NavigatorObjectKind);
type ColumnRecord = (String, String, NavigatorColumn);

pub async fn load_navigator(
    connection_id: &str,
    state: &AppState,
) -> Result<NavigatorTree, String> {
    let profile = get_profile(connection_id, state)?;

    with_timeout(
        DEFAULT_NAVIGATOR_TIMEOUT_MS,
        async {
            match profile.engine {
                DbEngine::Sqlite => load_sqlite_navigator(&profile).await,
                DbEngine::Postgres => load_postgres_navigator(&profile).await,
                DbEngine::Mysql => load_mysql_navigator(&profile).await,
            }
        },
        "Schema navigator load",
    )
    .await
}

async fn load_sqlite_navigator(profile: &ConnectionProfile) -> Result<NavigatorTree, String> {
    let options = sqlite_connect_options(profile)?;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|err| format!("Failed to open SQLite database: {err}"))?;

    let rows = sqlx::query(
        "SELECT name, type
         FROM sqlite_master
         WHERE type IN ('table', 'view')
           AND name NOT LIKE 'sqlite_%'
         ORDER BY type, name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| format!("Failed to load SQLite schema metadata: {err}"))?;

    let mut tables = Vec::new();
    let mut views = Vec::new();
    let mut warnings = Vec::new();

    for row in rows {
        let name = sqlite_row_get_string(&row, "name");
        if name.is_empty() {
            continue;
        }

        let object_type = sqlite_row_get_string(&row, "type").to_ascii_lowercase();
        let kind = if object_type == "view" {
            NavigatorObjectKind::View
        } else {
            NavigatorObjectKind::Table
        };

        let columns = match load_sqlite_columns(&pool, &name).await {
            Ok(columns) => columns,
            Err(err) => {
                warnings.push(format!(
                    "Failed to inspect columns for {} '{}': {err}",
                    if kind == NavigatorObjectKind::Table {
                        "table"
                    } else {
                        "view"
                    },
                    name
                ));
                Vec::new()
            }
        };

        let object = NavigatorObject {
            name,
            kind: kind.clone(),
            columns,
        };

        if kind == NavigatorObjectKind::Table {
            tables.push(object);
        } else {
            views.push(object);
        }
    }

    tables.sort_by(|left, right| left.name.cmp(&right.name));
    views.sort_by(|left, right| left.name.cmp(&right.name));

    Ok(NavigatorTree {
        schemas: vec![NavigatorSchema {
            name: "main".to_string(),
            tables,
            views,
        }],
        warnings,
    })
}

async fn load_postgres_navigator(profile: &ConnectionProfile) -> Result<NavigatorTree, String> {
    let url = to_connection_url(profile)?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(NAVIGATOR_CONNECT_TIMEOUT_SECONDS))
        .connect(&url)
        .await
        .map_err(|err| format!("Connection failed: {err}"))?;

    sqlx::query(&format!(
        "SET statement_timeout = {}",
        POSTGRES_STATEMENT_TIMEOUT_MS
    ))
    .execute(&pool)
    .await
    .map_err(|err| format!("Failed to configure PostgreSQL timeout: {err}"))?;

    let object_rows = sqlx::query(
        "SELECT
             n.nspname AS schema_name,
             c.relname AS object_name,
             c.relkind::text AS object_kind
         FROM pg_catalog.pg_class c
         JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
         WHERE c.relkind IN ('r', 'p', 'f', 'v', 'm')
           AND n.nspname NOT IN ('pg_catalog', 'information_schema')
           AND n.nspname NOT LIKE 'pg_toast%'
         ORDER BY n.nspname, c.relname",
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| format!("Failed to load PostgreSQL object metadata: {err}"))?;

    let column_rows = sqlx::query(
        "SELECT
             n.nspname AS schema_name,
             c.relname AS object_name,
             a.attname AS column_name,
             pg_catalog.format_type(a.atttypid, a.atttypmod) AS data_type,
             NOT a.attnotnull AS is_nullable
         FROM pg_catalog.pg_attribute a
         JOIN pg_catalog.pg_class c ON c.oid = a.attrelid
         JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
         WHERE a.attnum > 0
           AND NOT a.attisdropped
           AND c.relkind IN ('r', 'p', 'f', 'v', 'm')
           AND n.nspname NOT IN ('pg_catalog', 'information_schema')
           AND n.nspname NOT LIKE 'pg_toast%'
         ORDER BY n.nspname, c.relname, a.attnum",
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| format!("Failed to load PostgreSQL column metadata: {err}"))?;

    let mut objects = Vec::new();
    for row in object_rows {
        let schema_name = pg_row_get_string(&row, "schema_name");
        let object_name = pg_row_get_string(&row, "object_name");
        if schema_name.is_empty() || object_name.is_empty() {
            continue;
        }

        let object_kind = pg_row_get_string(&row, "object_kind");
        let kind = if matches!(object_kind.as_str(), "v" | "m") {
            NavigatorObjectKind::View
        } else {
            NavigatorObjectKind::Table
        };

        objects.push((schema_name, object_name, kind));
    }

    let mut columns = Vec::new();
    for row in column_rows {
        let schema_name = pg_row_get_string(&row, "schema_name");
        let object_name = pg_row_get_string(&row, "object_name");
        let column_name = pg_row_get_string(&row, "column_name");
        if schema_name.is_empty() || object_name.is_empty() || column_name.is_empty() {
            continue;
        }

        let data_type = pg_row_get_string(&row, "data_type");
        let nullable = pg_row_get_bool(&row, "is_nullable").unwrap_or(true);

        columns.push((
            schema_name,
            object_name,
            NavigatorColumn {
                name: column_name,
                data_type: if data_type.is_empty() {
                    "unknown".to_string()
                } else {
                    data_type
                },
                nullable,
            },
        ));
    }

    Ok(build_navigator_tree(objects, columns, Vec::new()))
}

async fn load_mysql_navigator(profile: &ConnectionProfile) -> Result<NavigatorTree, String> {
    let url = to_connection_url(profile)?;
    let pool = MySqlPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(NAVIGATOR_CONNECT_TIMEOUT_SECONDS))
        .connect(&url)
        .await
        .map_err(|err| format!("Connection failed: {err}"))?;

    let object_rows = sqlx::query(
        "SELECT table_schema, table_name, table_type
         FROM information_schema.tables
         WHERE table_schema = DATABASE()
           AND table_type IN ('BASE TABLE', 'VIEW')
         ORDER BY table_schema, table_name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| format!("Failed to load MySQL object metadata: {err}"))?;

    let column_rows = sqlx::query(
        "SELECT table_schema, table_name, column_name, data_type, is_nullable
         FROM information_schema.columns
         WHERE table_schema = DATABASE()
         ORDER BY table_schema, table_name, ordinal_position",
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| format!("Failed to load MySQL column metadata: {err}"))?;

    let mut objects = Vec::new();
    for row in object_rows {
        let schema_name = mysql_row_get_string(&row, "table_schema");
        let object_name = mysql_row_get_string(&row, "table_name");
        if schema_name.is_empty() || object_name.is_empty() {
            continue;
        }

        let object_kind = mysql_row_get_string(&row, "table_type").to_ascii_uppercase();
        let kind = if object_kind == "VIEW" {
            NavigatorObjectKind::View
        } else {
            NavigatorObjectKind::Table
        };

        objects.push((schema_name, object_name, kind));
    }

    let mut columns = Vec::new();
    for row in column_rows {
        let schema_name = mysql_row_get_string(&row, "table_schema");
        let object_name = mysql_row_get_string(&row, "table_name");
        let column_name = mysql_row_get_string(&row, "column_name");
        if schema_name.is_empty() || object_name.is_empty() || column_name.is_empty() {
            continue;
        }

        let data_type = mysql_row_get_string(&row, "data_type");
        let nullable = mysql_row_get_string(&row, "is_nullable").eq_ignore_ascii_case("YES");

        columns.push((
            schema_name,
            object_name,
            NavigatorColumn {
                name: column_name,
                data_type: if data_type.is_empty() {
                    "unknown".to_string()
                } else {
                    data_type
                },
                nullable,
            },
        ));
    }

    Ok(build_navigator_tree(objects, columns, Vec::new()))
}

fn build_navigator_tree(
    object_records: Vec<ObjectRecord>,
    column_records: Vec<ColumnRecord>,
    warnings: Vec<String>,
) -> NavigatorTree {
    let mut columns_by_key: HashMap<(String, String), Vec<NavigatorColumn>> = HashMap::new();
    for (schema_name, object_name, column) in column_records {
        columns_by_key
            .entry((schema_name, object_name))
            .or_default()
            .push(column);
    }

    let mut schema_map: BTreeMap<String, NavigatorSchema> = BTreeMap::new();
    for (schema_name, object_name, kind) in object_records {
        let columns = columns_by_key
            .remove(&(schema_name.clone(), object_name.clone()))
            .unwrap_or_default();

        let schema = schema_map
            .entry(schema_name.clone())
            .or_insert_with(|| NavigatorSchema {
                name: schema_name,
                tables: Vec::new(),
                views: Vec::new(),
            });

        let object = NavigatorObject {
            name: object_name,
            kind: kind.clone(),
            columns,
        };

        if kind == NavigatorObjectKind::Table {
            schema.tables.push(object);
        } else {
            schema.views.push(object);
        }
    }

    let mut schemas = schema_map.into_values().collect::<Vec<_>>();
    for schema in &mut schemas {
        schema
            .tables
            .sort_by(|left, right| left.name.cmp(&right.name));
        schema
            .views
            .sort_by(|left, right| left.name.cmp(&right.name));
    }

    NavigatorTree { schemas, warnings }
}

async fn load_sqlite_columns(
    pool: &SqlitePool,
    object_name: &str,
) -> Result<Vec<NavigatorColumn>, String> {
    let escaped_name = object_name.replace('\'', "''");
    let sql = format!("PRAGMA table_info('{escaped_name}')");

    let rows = sqlx::query(&sql)
        .fetch_all(pool)
        .await
        .map_err(|err| format!("Failed to load SQLite column metadata: {err}"))?;

    let mut columns = Vec::new();
    for row in rows {
        let name = sqlite_row_get_string(&row, "name");
        if name.is_empty() {
            continue;
        }

        let data_type = sqlite_row_get_string(&row, "type");
        let nullable = sqlite_row_get_i64(&row, "notnull").unwrap_or(0) == 0;

        columns.push(NavigatorColumn {
            name,
            data_type: if data_type.is_empty() {
                "unknown".to_string()
            } else {
                data_type
            },
            nullable,
        });
    }

    Ok(columns)
}

fn pg_row_get_string(row: &PgRow, column: &str) -> String {
    if let Ok(value) = row.try_get::<String, _>(column) {
        return value;
    }
    if let Ok(value) = row.try_get::<Option<String>, _>(column) {
        return value.unwrap_or_default();
    }

    String::new()
}

fn pg_row_get_bool(row: &PgRow, column: &str) -> Option<bool> {
    if let Ok(value) = row.try_get::<bool, _>(column) {
        return Some(value);
    }
    if let Ok(value) = row.try_get::<Option<bool>, _>(column) {
        return value;
    }

    None
}

fn mysql_row_get_string(row: &MySqlRow, column: &str) -> String {
    if let Ok(value) = row.try_get::<String, _>(column) {
        return value;
    }
    if let Ok(value) = row.try_get::<Option<String>, _>(column) {
        return value.unwrap_or_default();
    }

    String::new()
}

fn sqlite_row_get_string(row: &SqliteRow, column: &str) -> String {
    if let Ok(value) = row.try_get::<String, _>(column) {
        return value;
    }
    if let Ok(value) = row.try_get::<Option<String>, _>(column) {
        return value.unwrap_or_default();
    }

    String::new()
}

fn sqlite_row_get_i64(row: &SqliteRow, column: &str) -> Option<i64> {
    if let Ok(value) = row.try_get::<i64, _>(column) {
        return Some(value);
    }
    if let Ok(value) = row.try_get::<i32, _>(column) {
        return Some(value as i64);
    }

    None
}
