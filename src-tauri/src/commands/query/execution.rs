use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::TryStreamExt;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions, MySqlRow};
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::{Column, Row};

use crate::models::{ConnectionProfile, DbEngine, QueryResult, SortDirection};
use crate::state::QueryCancellation;

use super::super::shared::{sqlite_connect_options, to_connection_url, with_timeout};

const NETWORK_CONNECT_TIMEOUT_SECONDS: u64 = 5;

#[derive(Debug, Clone)]
pub struct QueryPushdownOptions {
    pub quick_filter: Option<String>,
    pub filter_columns: Vec<String>,
    pub sort_column: Option<String>,
    pub sort_direction: SortDirection,
}

pub async fn execute_sqlite_query(
    profile: &ConnectionProfile,
    sql: &str,
    page_size: usize,
    page_offset: u64,
    timeout_ms: u64,
    pushdown: &QueryPushdownOptions,
    cancellation: Arc<QueryCancellation>,
) -> Result<QueryResult, String> {
    let started = Instant::now();

    with_timeout(
        timeout_ms,
        async {
            if cancellation.is_cancelled() {
                return Err("Query cancelled".to_string());
            }

            let options = sqlite_connect_options(profile)?;
            let connect_started = Instant::now();
            let pool = tokio::select! {
                _ = cancellation.cancelled() => {
                    return Err("Query cancelled".to_string());
                }
                result = SqlitePoolOptions::new()
                    .max_connections(1)
                    .connect_with(options) => {
                    result.map_err(|err| format!("Connection failed: {err}"))?
                }
            };
            let connect_ms = connect_started.elapsed().as_millis();

            let first_word = leading_sql_keyword(sql);
            let is_row_query = is_row_query_statement(&first_word);

            if is_row_query {
                let fetch_started = Instant::now();
                let (paged_rows, has_more, fetch_mode) = fetch_sqlite_page(
                    &pool,
                    sql,
                    &first_word,
                    page_size,
                    page_offset,
                    pushdown,
                    &cancellation,
                )
                .await?;
                let fetch_ms = fetch_started.elapsed().as_millis();

                let columns = paged_rows
                    .first()
                    .map(|row| {
                        row.columns()
                            .iter()
                            .map(|column| column.name().to_string())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                let data = paged_rows
                    .iter()
                    .map(|row| {
                        (0..row.len())
                            .map(|index| sqlite_cell_to_string(row, index))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                return Ok(QueryResult {
                    columns,
                    rows: data,
                    affected_rows: 0,
                    execution_ms: started.elapsed().as_millis(),
                    message: build_row_page_message(
                        page_offset,
                        page_size as u32,
                        has_more,
                        paged_rows.len(),
                        format_diagnostics(&[("connect_ms", connect_ms), ("fetch_ms", fetch_ms)])
                            .with_text("fetch_mode", fetch_mode)
                            .render(),
                    ),
                    schema_changed: false,
                    is_row_query: true,
                    page_size: page_size as u32,
                    page_offset,
                    has_more,
                });
            }

            let done = tokio::select! {
                _ = cancellation.cancelled() => {
                    return Err("Query cancelled".to_string());
                }
                result = sqlx::query(sql).execute(&pool) => {
                    result.map_err(|err| format!("Execution failed: {err}"))?
                }
            };
            let execute_ms = started.elapsed().as_millis();

            let schema_changed = is_schema_change_statement(&first_word);

            Ok(QueryResult {
                columns: Vec::new(),
                rows: Vec::new(),
                affected_rows: done.rows_affected(),
                execution_ms: execute_ms,
                message: format!(
                    "Done. {} row(s) affected [{}]",
                    done.rows_affected(),
                    format_diagnostics(&[
                        ("connect_ms", connect_ms),
                        ("execute_ms", execute_ms.saturating_sub(connect_ms)),
                    ])
                    .render()
                ),
                schema_changed,
                is_row_query: false,
                page_size: 0,
                page_offset: 0,
                has_more: false,
            })
        },
        "Query execution",
    )
    .await
}

pub async fn execute_postgres_query(
    profile: &ConnectionProfile,
    sql: &str,
    page_size: usize,
    page_offset: u64,
    timeout_ms: u64,
    pushdown: &QueryPushdownOptions,
    cancellation: Arc<QueryCancellation>,
) -> Result<QueryResult, String> {
    let started = Instant::now();

    with_timeout(
        timeout_ms,
        async {
            if cancellation.is_cancelled() {
                return Err("Query cancelled".to_string());
            }

            let url = to_connection_url(profile)?;
            let connect_started = Instant::now();
            let pool = tokio::select! {
                _ = cancellation.cancelled() => {
                    return Err("Query cancelled".to_string());
                }
                result = PgPoolOptions::new()
                    .max_connections(1)
                    .acquire_timeout(Duration::from_secs(NETWORK_CONNECT_TIMEOUT_SECONDS))
                    .connect(&url) => {
                    result.map_err(|err| format!("Connection failed: {err}"))?
                }
            };
            let connect_ms = connect_started.elapsed().as_millis();

            let session_started = Instant::now();
            sqlx::query(&format!("SET statement_timeout = {timeout_ms}"))
                .execute(&pool)
                .await
                .map_err(|err| {
                    format!("Failed to configure PostgreSQL statement timeout: {err}")
                })?;
            sqlx::query(&format!("SET lock_timeout = {timeout_ms}"))
                .execute(&pool)
                .await
                .map_err(|err| format!("Failed to configure PostgreSQL lock timeout: {err}"))?;
            let session_ms = session_started.elapsed().as_millis();

            let first_word = leading_sql_keyword(sql);
            let is_row_query = is_row_query_statement(&first_word);

            if is_row_query {
                let fetch_started = Instant::now();
                let (paged_rows, has_more, fetch_mode) = fetch_postgres_page(
                    &pool,
                    sql,
                    &first_word,
                    page_size,
                    page_offset,
                    pushdown,
                    &cancellation,
                )
                .await?;
                let fetch_ms = fetch_started.elapsed().as_millis();

                let columns = paged_rows
                    .first()
                    .map(|row| {
                        row.columns()
                            .iter()
                            .map(|column| column.name().to_string())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                let data = paged_rows
                    .iter()
                    .map(|row| {
                        (0..row.len())
                            .map(|index| postgres_cell_to_string(row, index))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                return Ok(QueryResult {
                    columns,
                    rows: data,
                    affected_rows: 0,
                    execution_ms: started.elapsed().as_millis(),
                    message: build_row_page_message(
                        page_offset,
                        page_size as u32,
                        has_more,
                        paged_rows.len(),
                        format_diagnostics(&[
                            ("connect_ms", connect_ms),
                            ("session_ms", session_ms),
                            ("fetch_ms", fetch_ms),
                        ])
                        .with_text("fetch_mode", fetch_mode)
                        .render(),
                    ),
                    schema_changed: false,
                    is_row_query: true,
                    page_size: page_size as u32,
                    page_offset,
                    has_more,
                });
            }

            let done = tokio::select! {
                _ = cancellation.cancelled() => {
                    return Err("Query cancelled".to_string());
                }
                result = sqlx::query(sql).execute(&pool) => {
                    result.map_err(|err| format!("Execution failed: {err}"))?
                }
            };
            let execute_ms = started.elapsed().as_millis();

            let schema_changed = is_schema_change_statement(&first_word);

            Ok(QueryResult {
                columns: Vec::new(),
                rows: Vec::new(),
                affected_rows: done.rows_affected(),
                execution_ms: execute_ms,
                message: format!(
                    "Done. {} row(s) affected [{}]",
                    done.rows_affected(),
                    format_diagnostics(&[
                        ("connect_ms", connect_ms),
                        ("session_ms", session_ms),
                        (
                            "execute_ms",
                            execute_ms.saturating_sub(connect_ms + session_ms),
                        ),
                    ])
                    .render()
                ),
                schema_changed,
                is_row_query: false,
                page_size: 0,
                page_offset: 0,
                has_more: false,
            })
        },
        "Query execution",
    )
    .await
}

pub async fn execute_mysql_query(
    profile: &ConnectionProfile,
    sql: &str,
    page_size: usize,
    page_offset: u64,
    timeout_ms: u64,
    pushdown: &QueryPushdownOptions,
    cancellation: Arc<QueryCancellation>,
) -> Result<QueryResult, String> {
    let started = Instant::now();

    with_timeout(
        timeout_ms,
        async {
            if cancellation.is_cancelled() {
                return Err("Query cancelled".to_string());
            }

            let url = to_connection_url(profile)?;
            let connect_started = Instant::now();
            let pool = tokio::select! {
                _ = cancellation.cancelled() => {
                    return Err("Query cancelled".to_string());
                }
                result = MySqlPoolOptions::new()
                    .max_connections(1)
                    .acquire_timeout(Duration::from_secs(NETWORK_CONNECT_TIMEOUT_SECONDS))
                    .connect(&url) => {
                    result.map_err(|err| format!("Connection failed: {err}"))?
                }
            };
            let connect_ms = connect_started.elapsed().as_millis();

            let first_word = leading_sql_keyword(sql);
            let is_row_query = is_row_query_statement(&first_word);

            if is_row_query {
                let fetch_started = Instant::now();
                let (paged_rows, has_more, fetch_mode) = fetch_mysql_page(
                    &pool,
                    sql,
                    &first_word,
                    page_size,
                    page_offset,
                    pushdown,
                    &cancellation,
                )
                .await?;
                let fetch_ms = fetch_started.elapsed().as_millis();

                let columns = paged_rows
                    .first()
                    .map(|row| {
                        row.columns()
                            .iter()
                            .map(|column| column.name().to_string())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                let data = paged_rows
                    .iter()
                    .map(|row| {
                        (0..row.len())
                            .map(|index| mysql_cell_to_string(row, index))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                return Ok(QueryResult {
                    columns,
                    rows: data,
                    affected_rows: 0,
                    execution_ms: started.elapsed().as_millis(),
                    message: build_row_page_message(
                        page_offset,
                        page_size as u32,
                        has_more,
                        paged_rows.len(),
                        format_diagnostics(&[("connect_ms", connect_ms), ("fetch_ms", fetch_ms)])
                            .with_text("fetch_mode", fetch_mode)
                            .render(),
                    ),
                    schema_changed: false,
                    is_row_query: true,
                    page_size: page_size as u32,
                    page_offset,
                    has_more,
                });
            }

            let done = tokio::select! {
                _ = cancellation.cancelled() => {
                    return Err("Query cancelled".to_string());
                }
                result = sqlx::query(sql).execute(&pool) => {
                    result.map_err(|err| format!("Execution failed: {err}"))?
                }
            };
            let execute_ms = started.elapsed().as_millis();

            let schema_changed = is_schema_change_statement(&first_word);

            Ok(QueryResult {
                columns: Vec::new(),
                rows: Vec::new(),
                affected_rows: done.rows_affected(),
                execution_ms: execute_ms,
                message: format!(
                    "Done. {} row(s) affected [{}]",
                    done.rows_affected(),
                    format_diagnostics(&[
                        ("connect_ms", connect_ms),
                        ("execute_ms", execute_ms.saturating_sub(connect_ms)),
                    ])
                    .render()
                ),
                schema_changed,
                is_row_query: false,
                page_size: 0,
                page_offset: 0,
                has_more: false,
            })
        },
        "Query execution",
    )
    .await
}

async fn fetch_sqlite_page(
    pool: &SqlitePool,
    sql: &str,
    first_word: &str,
    page_size: usize,
    page_offset: u64,
    pushdown: &QueryPushdownOptions,
    cancellation: &Arc<QueryCancellation>,
) -> Result<(Vec<SqliteRow>, bool, &'static str), String> {
    ensure_pushdown_supported(first_word, pushdown)?;

    if supports_wrapped_pagination(first_word) {
        let paged_sql =
            build_wrapped_paged_sql(sql, page_size, page_offset, DbEngine::Sqlite, pushdown);
        let wrapped_query_result = tokio::select! {
            _ = cancellation.cancelled() => {
                return Err("Query cancelled".to_string());
            }
            result = sqlx::query(&paged_sql).fetch_all(pool) => {
                result
            }
        };

        match wrapped_query_result {
            Ok(mut rows) => {
                let has_more = rows.len() > page_size;
                if has_more {
                    rows.truncate(page_size);
                }
                return Ok((rows, has_more, "wrapped"));
            }
            Err(error) => {
                let wrapped_error = format!("Wrapped pagination failed: {error}");
                eprintln!("[db-pro][sqlite][query] {wrapped_error}");
                if requires_wrapped_pushdown(pushdown) {
                    return Err(wrapped_error);
                }
            }
        }
    }

    let mut stream = sqlx::query(sql).fetch(pool);
    let mut skipped = 0u64;
    let mut paged_rows = Vec::with_capacity(page_size);

    loop {
        let next_row = tokio::select! {
            _ = cancellation.cancelled() => {
                return Err("Query cancelled".to_string());
            }
            result = stream.try_next() => {
                result.map_err(|err| format!("Query failed: {err}"))?
            }
        };

        let Some(row) = next_row else {
            return Ok((paged_rows, false, "stream"));
        };

        if skipped < page_offset {
            skipped += 1;
            continue;
        }

        if paged_rows.len() < page_size {
            paged_rows.push(row);
            continue;
        }

        return Ok((paged_rows, true, "stream"));
    }
}

async fn fetch_postgres_page(
    pool: &PgPool,
    sql: &str,
    first_word: &str,
    page_size: usize,
    page_offset: u64,
    pushdown: &QueryPushdownOptions,
    cancellation: &Arc<QueryCancellation>,
) -> Result<(Vec<PgRow>, bool, &'static str), String> {
    ensure_pushdown_supported(first_word, pushdown)?;

    if supports_wrapped_pagination(first_word) {
        let paged_sql =
            build_wrapped_paged_sql(sql, page_size, page_offset, DbEngine::Postgres, pushdown);
        let wrapped_query_result = tokio::select! {
            _ = cancellation.cancelled() => {
                return Err("Query cancelled".to_string());
            }
            result = sqlx::query(&paged_sql).fetch_all(pool) => {
                result
            }
        };

        match wrapped_query_result {
            Ok(mut rows) => {
                let has_more = rows.len() > page_size;
                if has_more {
                    rows.truncate(page_size);
                }
                return Ok((rows, has_more, "wrapped"));
            }
            Err(error) => {
                let wrapped_error = format!("Wrapped pagination failed: {error}");
                eprintln!("[db-pro][postgres][query] {wrapped_error}");
                if requires_wrapped_pushdown(pushdown) {
                    return Err(wrapped_error);
                }
            }
        }
    }

    let mut stream = sqlx::query(sql).fetch(pool);
    let mut skipped = 0u64;
    let mut paged_rows = Vec::with_capacity(page_size);

    loop {
        let next_row = tokio::select! {
            _ = cancellation.cancelled() => {
                return Err("Query cancelled".to_string());
            }
            result = stream.try_next() => {
                result.map_err(|err| format!("Query failed: {err}"))?
            }
        };

        let Some(row) = next_row else {
            return Ok((paged_rows, false, "stream"));
        };

        if skipped < page_offset {
            skipped += 1;
            continue;
        }

        if paged_rows.len() < page_size {
            paged_rows.push(row);
            continue;
        }

        return Ok((paged_rows, true, "stream"));
    }
}

async fn fetch_mysql_page(
    pool: &MySqlPool,
    sql: &str,
    first_word: &str,
    page_size: usize,
    page_offset: u64,
    pushdown: &QueryPushdownOptions,
    cancellation: &Arc<QueryCancellation>,
) -> Result<(Vec<MySqlRow>, bool, &'static str), String> {
    ensure_pushdown_supported(first_word, pushdown)?;

    if supports_wrapped_pagination(first_word) {
        let paged_sql =
            build_wrapped_paged_sql(sql, page_size, page_offset, DbEngine::Mysql, pushdown);
        let wrapped_query_result = tokio::select! {
            _ = cancellation.cancelled() => {
                return Err("Query cancelled".to_string());
            }
            result = sqlx::query(&paged_sql).fetch_all(pool) => {
                result
            }
        };

        match wrapped_query_result {
            Ok(mut rows) => {
                let has_more = rows.len() > page_size;
                if has_more {
                    rows.truncate(page_size);
                }
                return Ok((rows, has_more, "wrapped"));
            }
            Err(error) => {
                let wrapped_error = format!("Wrapped pagination failed: {error}");
                eprintln!("[db-pro][mysql][query] {wrapped_error}");
                if requires_wrapped_pushdown(pushdown) {
                    return Err(wrapped_error);
                }
            }
        }
    }

    let mut stream = sqlx::query(sql).fetch(pool);
    let mut skipped = 0u64;
    let mut paged_rows = Vec::with_capacity(page_size);

    loop {
        let next_row = tokio::select! {
            _ = cancellation.cancelled() => {
                return Err("Query cancelled".to_string());
            }
            result = stream.try_next() => {
                result.map_err(|err| format!("Query failed: {err}"))?
            }
        };

        let Some(row) = next_row else {
            return Ok((paged_rows, false, "stream"));
        };

        if skipped < page_offset {
            skipped += 1;
            continue;
        }

        if paged_rows.len() < page_size {
            paged_rows.push(row);
            continue;
        }

        return Ok((paged_rows, true, "stream"));
    }
}

fn build_row_page_message(
    page_offset: u64,
    page_size: u32,
    has_more: bool,
    fetched_rows: usize,
    diagnostics: String,
) -> String {
    if fetched_rows == 0 {
        let mut message = "Fetched 0 row(s)".to_string();
        if !diagnostics.is_empty() {
            message.push_str(" [");
            message.push_str(&diagnostics);
            message.push(']');
        }
        return message;
    }

    let start = page_offset + 1;
    let end = page_offset + fetched_rows as u64;
    let page_index = page_offset / u64::from(page_size) + 1;

    let mut message = format!("Fetched rows {start}-{end} (page {page_index})");
    if has_more {
        message.push_str(" (more rows available)");
    }
    if !diagnostics.is_empty() {
        message.push_str(" [");
        message.push_str(&diagnostics);
        message.push(']');
    }
    message
}

fn format_diagnostics(values: &[(&str, u128)]) -> DiagnosticsBuilder {
    let mut parts = values
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>();
    parts.insert(0, "diag".to_string());
    DiagnosticsBuilder { parts }
}

struct DiagnosticsBuilder {
    parts: Vec<String>,
}

impl DiagnosticsBuilder {
    fn with_text(mut self, key: &str, value: &str) -> Self {
        self.parts.push(format!("{key}={value}"));
        self
    }

    fn render(self) -> String {
        self.parts.join(" ")
    }
}

fn leading_sql_keyword(sql: &str) -> String {
    let mut rest = sql.trim_start();

    loop {
        if let Some(comment) = rest.strip_prefix("--") {
            if let Some(next_line) = comment.find('\n') {
                rest = comment[(next_line + 1)..].trim_start();
                continue;
            }
            return String::new();
        }

        if let Some(comment) = rest.strip_prefix("/*") {
            if let Some(end_block) = comment.find("*/") {
                rest = comment[(end_block + 2)..].trim_start();
                continue;
            }
            return String::new();
        }

        break;
    }

    rest.split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn is_row_query_statement(first_word: &str) -> bool {
    matches!(
        first_word,
        "select" | "with" | "show" | "describe" | "explain" | "pragma"
    )
}

fn supports_wrapped_pagination(first_word: &str) -> bool {
    matches!(first_word, "select" | "with")
}

fn requires_wrapped_pushdown(pushdown: &QueryPushdownOptions) -> bool {
    pushdown
        .quick_filter
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        || pushdown
            .sort_column
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
}

fn ensure_pushdown_supported(
    first_word: &str,
    pushdown: &QueryPushdownOptions,
) -> Result<(), String> {
    if !requires_wrapped_pushdown(pushdown) || supports_wrapped_pagination(first_word) {
        return Ok(());
    }

    let statement = if first_word.is_empty() {
        "<unknown>"
    } else {
        first_word
    };

    Err(format!(
        "Filter/sort pushdown requires SELECT/WITH statements for wrapped pagination; got '{statement}'."
    ))
}

fn build_wrapped_paged_sql(
    sql: &str,
    page_size: usize,
    page_offset: u64,
    engine: DbEngine,
    pushdown: &QueryPushdownOptions,
) -> String {
    let base_sql = strip_trailing_semicolons(sql);
    let window_size = page_size.saturating_add(1);

    let mut wrapped_sql = format!("SELECT * FROM ({base_sql}) AS __db_pro_page");
    if let Some(filter_clause) = build_filter_clause(&engine, pushdown) {
        wrapped_sql.push_str(" WHERE ");
        wrapped_sql.push_str(&filter_clause);
    }
    if let Some(order_by_clause) = build_order_by_clause(&engine, pushdown) {
        wrapped_sql.push(' ');
        wrapped_sql.push_str(&order_by_clause);
    }

    wrapped_sql.push_str(&format!(" LIMIT {window_size} OFFSET {page_offset}"));
    wrapped_sql
}

fn build_filter_clause(engine: &DbEngine, pushdown: &QueryPushdownOptions) -> Option<String> {
    let quick_filter = pushdown.quick_filter.as_deref()?.trim();
    if quick_filter.is_empty() {
        return None;
    }

    let mut filter_columns = pushdown
        .filter_columns
        .iter()
        .map(|column| column.trim())
        .filter(|column| !column.is_empty())
        .collect::<Vec<_>>();
    filter_columns.sort_unstable();
    filter_columns.dedup();

    if filter_columns.is_empty() {
        return None;
    }

    let escaped_pattern = escape_sql_literal(&format!(
        "%{}%",
        escape_like_pattern(&quick_filter.to_ascii_lowercase())
    ));

    let expressions = filter_columns
        .into_iter()
        .map(|column| {
            let qualified = build_qualified_column_reference(engine, column);
            let cast_expr = match engine {
                DbEngine::Mysql => format!("LOWER(CAST({qualified} AS CHAR))"),
                DbEngine::Postgres | DbEngine::Sqlite => {
                    format!("LOWER(CAST({qualified} AS TEXT))")
                }
            };
            format!("{cast_expr} LIKE '{escaped_pattern}' ESCAPE '\\'")
        })
        .collect::<Vec<_>>();

    if expressions.is_empty() {
        return None;
    }

    Some(format!("({})", expressions.join(" OR ")))
}

fn build_order_by_clause(engine: &DbEngine, pushdown: &QueryPushdownOptions) -> Option<String> {
    let column = pushdown.sort_column.as_deref()?.trim();
    if column.is_empty() {
        return None;
    }

    let direction = match pushdown.sort_direction {
        SortDirection::Asc => "ASC",
        SortDirection::Desc => "DESC",
    };
    let qualified = build_qualified_column_reference(engine, column);

    Some(format!("ORDER BY {qualified} {direction}"))
}

fn build_qualified_column_reference(engine: &DbEngine, column: &str) -> String {
    format!("__db_pro_page.{}", quote_identifier(engine, column))
}

fn quote_identifier(engine: &DbEngine, identifier: &str) -> String {
    match engine {
        DbEngine::Mysql => format!("`{}`", identifier.replace('`', "``")),
        DbEngine::Postgres | DbEngine::Sqlite => {
            format!("\"{}\"", identifier.replace('"', "\"\""))
        }
    }
}

fn escape_like_pattern(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '%' => escaped.push_str("\\%"),
            '_' => escaped.push_str("\\_"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn escape_sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}

fn strip_trailing_semicolons(sql: &str) -> String {
    let mut current = sql.trim_end();
    while let Some(without_semicolon) = current.strip_suffix(';') {
        current = without_semicolon.trim_end();
    }
    current.to_string()
}

fn is_schema_change_statement(first_word: &str) -> bool {
    matches!(
        first_word,
        "create" | "alter" | "drop" | "truncate" | "rename"
    )
}

fn postgres_cell_to_string(row: &PgRow, index: usize) -> String {
    if let Ok(value) = row.try_get::<Option<String>, _>(index) {
        return value.unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<i64>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<i32>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<f64>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<f32>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<bool>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<Vec<u8>>, _>(index) {
        return value
            .map(|item| format!("<{} bytes>", item.len()))
            .unwrap_or_else(|| "NULL".to_string());
    }

    "<unrenderable>".to_string()
}

fn mysql_cell_to_string(row: &MySqlRow, index: usize) -> String {
    if let Ok(value) = row.try_get::<Option<String>, _>(index) {
        return value.unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<i64>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<i32>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<f64>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<f32>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<bool>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<Vec<u8>>, _>(index) {
        return value
            .map(|item| format!("<{} bytes>", item.len()))
            .unwrap_or_else(|| "NULL".to_string());
    }

    "<unrenderable>".to_string()
}

fn sqlite_cell_to_string(row: &SqliteRow, index: usize) -> String {
    if let Ok(value) = row.try_get::<Option<String>, _>(index) {
        return value.unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<i64>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<i32>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<f64>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<f32>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<bool>, _>(index) {
        return value
            .map(|item| item.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<Vec<u8>>, _>(index) {
        return value
            .map(|item| format!("<{} bytes>", item.len()))
            .unwrap_or_else(|| "NULL".to_string());
    }

    "<unrenderable>".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_pushdown(
        quick_filter: Option<&str>,
        sort_column: Option<&str>,
    ) -> QueryPushdownOptions {
        QueryPushdownOptions {
            quick_filter: quick_filter.map(ToString::to_string),
            filter_columns: quick_filter
                .map(|_| vec!["id".to_string(), "name".to_string()])
                .unwrap_or_default(),
            sort_column: sort_column.map(ToString::to_string),
            sort_direction: SortDirection::Asc,
        }
    }

    #[test]
    fn diagnostics_format_is_deterministic() {
        let diagnostics = format_diagnostics(&[("connect_ms", 12), ("fetch_ms", 44)])
            .with_text("fetch_mode", "wrapped")
            .render();

        assert_eq!(
            diagnostics,
            "diag connect_ms=12 fetch_ms=44 fetch_mode=wrapped"
        );
    }

    #[test]
    fn pushdown_requires_select_or_with_statements() {
        let pushdown = build_pushdown(Some("john"), None);
        let error = ensure_pushdown_supported("show", &pushdown)
            .expect_err("expected explicit error for unsupported pushdown");

        assert!(error.contains("requires SELECT/WITH"));
        assert!(error.contains("show"));
    }

    #[test]
    fn pushdown_accepts_select_statements() {
        let pushdown = build_pushdown(Some("john"), Some("id"));
        let result = ensure_pushdown_supported("select", &pushdown);

        assert!(result.is_ok());
    }
}
