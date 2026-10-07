//! Apply the editor schema as `search_path` on the connection that runs the SQL.
//!
//! `quote_ident` keeps a mixed-case or spaced name as one identifier. Session
//! changes are restored before the connection returns to the pool. Inside a
//! transaction, `is_local` drops the setting on commit or rollback.

use db_pro_core::application::active_query_schema;
use db_pro_core::domain::error::DbError;
use sqlx::PgConnection;

pub(super) fn requested_schema() -> Option<String> {
    active_query_schema().filter(|schema| !schema.is_empty())
}

pub(super) async fn push_session_search_path(conn: &mut PgConnection) -> Result<Option<String>, DbError> {
    let Some(schema) = requested_schema() else {
        return Ok(None);
    };
    let previous: String = sqlx::query_scalar("SELECT current_setting('search_path')")
        .fetch_one(&mut *conn)
        .await
        .map_err(crate::error::from_sqlx)?;
    sqlx::query("SELECT set_config('search_path', quote_ident($1), false)")
        .bind(&schema)
        .execute(&mut *conn)
        .await
        .map_err(crate::error::from_sqlx)?;
    Ok(Some(previous))
}

pub(super) async fn pop_session_search_path(conn: &mut PgConnection, previous: Option<String>) -> Result<(), DbError> {
    let Some(previous) = previous else {
        return Ok(());
    };
    sqlx::query("SELECT set_config('search_path', $1, false)")
        .bind(previous)
        .execute(&mut *conn)
        .await
        .map_err(crate::error::from_sqlx)?;
    Ok(())
}

pub(super) async fn set_local_search_path(conn: &mut PgConnection) -> Result<(), DbError> {
    let Some(schema) = requested_schema() else {
        return Ok(());
    };
    sqlx::query("SELECT set_config('search_path', quote_ident($1), true)")
        .bind(schema)
        .execute(&mut *conn)
        .await
        .map_err(crate::error::from_sqlx)?;
    Ok(())
}

pub(super) fn combine<T>(result: Result<T, DbError>, restored: Result<(), DbError>) -> Result<T, DbError> {
    match (result, restored) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}
