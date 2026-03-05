mod cancellation;
mod execution;
#[cfg(test)]
mod integration_tests;

use std::time::{Duration, Instant};

use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::SqlitePoolOptions;

use crate::models::{DbEngine, QueryRequest, QueryResult, SortDirection};
use crate::state::AppState;

use super::shared::{
    get_profile, sqlite_connect_options, to_connection_url, with_timeout,
    DEFAULT_CONNECTION_TEST_TIMEOUT_MS, DEFAULT_QUERY_TIMEOUT_MS, MAX_QUERY_TIMEOUT_MS,
    MIN_QUERY_TIMEOUT_MS,
};

const DEFAULT_PAGE_SIZE: u32 = 500;
const MAX_PAGE_SIZE: u32 = 5_000;
const NETWORK_CONNECT_TIMEOUT_SECONDS: u64 = 5;

pub async fn test_connection(connection_id: &str, state: &AppState) -> Result<String, String> {
    let profile = get_profile(connection_id, state)?;
    let started = Instant::now();

    with_timeout(
        DEFAULT_CONNECTION_TEST_TIMEOUT_MS,
        async {
            match profile.engine {
                DbEngine::Sqlite => {
                    let options = sqlite_connect_options(&profile)?;
                    let pool = SqlitePoolOptions::new()
                        .max_connections(1)
                        .connect_with(options)
                        .await
                        .map_err(|err| format!("Connection failed: {err}"))?;

                    sqlx::query("SELECT 1")
                        .execute(&pool)
                        .await
                        .map_err(|err| format!("Ping failed: {err}"))?;
                }
                DbEngine::Postgres => {
                    let url = to_connection_url(&profile)?;
                    let pool = PgPoolOptions::new()
                        .max_connections(1)
                        .acquire_timeout(Duration::from_secs(NETWORK_CONNECT_TIMEOUT_SECONDS))
                        .connect(&url)
                        .await
                        .map_err(|err| format!("Connection failed: {err}"))?;

                    sqlx::query("SELECT 1")
                        .execute(&pool)
                        .await
                        .map_err(|err| format!("Ping failed: {err}"))?;
                }
                DbEngine::Mysql => {
                    let url = to_connection_url(&profile)?;
                    let pool = MySqlPoolOptions::new()
                        .max_connections(1)
                        .acquire_timeout(Duration::from_secs(NETWORK_CONNECT_TIMEOUT_SECONDS))
                        .connect(&url)
                        .await
                        .map_err(|err| format!("Connection failed: {err}"))?;

                    sqlx::query("SELECT 1")
                        .execute(&pool)
                        .await
                        .map_err(|err| format!("Ping failed: {err}"))?;
                }
            }

            Ok(())
        },
        "Connection test",
    )
    .await?;

    Ok(format!("Connected in {} ms", started.elapsed().as_millis()))
}

pub async fn execute_query(request: QueryRequest, state: &AppState) -> Result<QueryResult, String> {
    let profile = get_profile(&request.connection_id, state)?;

    let sql = request.sql.trim();
    if sql.is_empty() {
        return Err("SQL is empty".to_string());
    }

    let page_size = request
        .page_size
        .or(request.limit)
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE) as usize;
    let offset = request.offset.unwrap_or(0);
    let timeout_ms = request
        .timeout_ms
        .unwrap_or(DEFAULT_QUERY_TIMEOUT_MS)
        .clamp(MIN_QUERY_TIMEOUT_MS, MAX_QUERY_TIMEOUT_MS);
    let pushdown = execution::QueryPushdownOptions {
        quick_filter: request
            .quick_filter
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        filter_columns: request
            .filter_columns
            .unwrap_or_default()
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect(),
        sort_column: request
            .sort_column
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        sort_direction: request.sort_direction.unwrap_or(SortDirection::Asc),
    };

    let cancellation = cancellation::register(state, &request.connection_id)?;

    let result = match profile.engine {
        DbEngine::Sqlite => {
            execution::execute_sqlite_query(
                &profile,
                sql,
                page_size,
                offset,
                timeout_ms,
                &pushdown,
                cancellation.clone(),
            )
            .await
        }
        DbEngine::Postgres => {
            execution::execute_postgres_query(
                &profile,
                sql,
                page_size,
                offset,
                timeout_ms,
                &pushdown,
                cancellation.clone(),
            )
            .await
        }
        DbEngine::Mysql => {
            execution::execute_mysql_query(
                &profile,
                sql,
                page_size,
                offset,
                timeout_ms,
                &pushdown,
                cancellation.clone(),
            )
            .await
        }
    };

    if let Err(err) = cancellation::clear(state, &request.connection_id, &cancellation) {
        eprintln!("Failed to clear running query state: {err}");
    }

    result
}

pub fn cancel_query(connection_id: &str, state: &AppState) -> Result<String, String> {
    if cancellation::cancel_for_connection(state, connection_id)? {
        return Ok("Cancellation requested.".to_string());
    }

    Ok("No running query to cancel.".to_string())
}
