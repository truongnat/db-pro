//! Apply the editor schema as `search_path` on the connection that runs the SQL.
//!
//! `quote_ident` keeps a mixed-case or spaced name as one identifier. Session
//! changes are restored before the connection returns to the pool. Inside a
//! transaction, `is_local` drops the setting on commit or rollback; outside one
//! it would drop at the end of the `set_config` statement itself, so autocommit
//! runs install a session-wide path instead and the caller must disable pool
//! reuse if that run is abandoned before the restore.

use db_pro_core::application::active_query_schema;
use db_pro_core::domain::error::DbError;
use sqlx::PgConnection;

pub(super) fn requested_schema() -> Option<String> {
    active_query_schema().filter(|schema| !schema.is_empty())
}

pub(super) async fn push_session_search_path(
    conn: &mut PgConnection,
    in_transaction: bool,
) -> Result<Option<String>, DbError> {
    let Some(schema) = requested_schema() else {
        return Ok(None);
    };
    // `active_query_schema` contract: a single schema identifier. Reject
    // separators so a caller can never smuggle a second schema or expression
    // into the quoted literal (#432).
    if schema.contains([',', '"', ';']) {
        return Err(DbError::Validation(format!("invalid schema name `{schema}`")));
    }
    let previous: String = sqlx::query_scalar("SELECT current_setting('search_path')")
        .fetch_one(&mut *conn)
        .await
        .map_err(crate::error::from_sqlx)?;
    // Keep `public` reachable: extension objects and shared helpers must not
    // vanish when the picker selects a tenant schema.
    // `is_local` must follow the connection state: inside an explicit transaction
    // it keeps #456's timeout/cancel guarantee, while in autocommit PostgreSQL
    // drops a local setting at the end of this statement — the run itself would
    // still resolve names against the old path. There the caller closes the
    // connection if it is dropped before the matching pop.
    sqlx::query("SELECT set_config('search_path', quote_ident($1) || ', public', $2)")
        .bind(&schema)
        .bind(in_transaction)
        .execute(&mut *conn)
        .await
        .map_err(crate::error::from_sqlx)?;
    Ok(Some(previous))
}

pub(super) async fn pop_session_search_path(
    conn: &mut PgConnection,
    previous: Option<String>,
    in_transaction: bool,
) -> Result<(), DbError> {
    let Some(previous) = previous else {
        return Ok(());
    };
    // Mirror the push scope so a transaction-local restore never overwrites the
    // session default that the connection will fall back to on commit.
    sqlx::query("SELECT set_config('search_path', $1, $2)")
        .bind(previous)
        .bind(in_transaction)
        .execute(&mut *conn)
        .await
        .map_err(crate::error::from_sqlx)?;
    Ok(())
}

pub(super) async fn set_local_search_path(conn: &mut PgConnection) -> Result<(), DbError> {
    let Some(schema) = requested_schema() else {
        return Ok(());
    };
    if schema.contains([',', '"', ';']) {
        return Err(DbError::Validation(format!("invalid schema name `{schema}`")));
    }
    sqlx::query("SELECT set_config('search_path', quote_ident($1) || ', public', true)")
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
