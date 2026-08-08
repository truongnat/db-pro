use db_pro_core::domain::error::DbError;

pub fn from_sqlx(err: sqlx::Error) -> DbError {
    match err {
        sqlx::Error::PoolTimedOut => DbError::ConnectionTimeout("connection pool timed out".into()),
        sqlx::Error::PoolClosed => DbError::ConnectionFailed("connection pool is closed".into()),
        sqlx::Error::Database(ref db_err) => {
            if let Some(code) = db_err.code() {
                let code_str = code.as_ref();
                // Authentication errors (SQLSTATE 28xxx)
                if code_str == "28P01" || code_str == "28000" || code_str == "28004" {
                    return DbError::AuthFailed(db_err.message().into());
                }
                // Connection exceptions (SQLSTATE 08xxx)
                if code_str.starts_with("08") {
                    return DbError::ConnectionRefused(db_err.message().into());
                }
                // Syntax errors (SQLSTATE 42xxx)
                if code_str.starts_with("42") {
                    return DbError::QuerySyntax(db_err.message().into());
                }
                // Permission denied (SQLSTATE 42501)
                if code_str == "42501" {
                    return DbError::PermissionDenied(db_err.message().into());
                }
                // Unique constraint violation (SQLSTATE 23505)
                if code_str == "23505" {
                    return DbError::DataFailed(format!("unique constraint violation: {}", db_err.message()));
                }
                // Foreign key violation (SQLSTATE 23503)
                if code_str == "23503" {
                    return DbError::DataFailed(format!("foreign key violation: {}", db_err.message()));
                }
                // Not-null violation (SQLSTATE 23502)
                if code_str == "23502" {
                    return DbError::DataFailed(format!("not-null constraint violation: {}", db_err.message()));
                }
                // Check violation (SQLSTATE 23514)
                if code_str == "23514" {
                    return DbError::DataFailed(format!("check constraint violation: {}", db_err.message()));
                }
                // Database not found (SQLSTATE 3D000)
                if code_str == "3D000" {
                    return DbError::DatabaseNotFound(db_err.message().into());
                }
            }
            DbError::QueryFailed(db_err.message().into())
        }
        sqlx::Error::Io(e) => DbError::Io(e.to_string()),
        sqlx::Error::RowNotFound => DbError::NotFound("row not found".into()),
        other => DbError::Internal(other.to_string()),
    }
}

pub fn from_rusqlite(err: rusqlite::Error) -> DbError {
    match err {
        rusqlite::Error::QueryReturnedNoRows => DbError::NotFound("no rows returned".into()),
        rusqlite::Error::SqliteFailure(ref ffi, ref msg) => {
            let message = msg.clone().unwrap_or_else(|| err.to_string());
            match ffi.code {
                rusqlite::ErrorCode::CannotOpen => DbError::ConnectionFailed(message),
                rusqlite::ErrorCode::ConstraintViolation => DbError::DataFailed(message),
                _ => DbError::QueryFailed(message),
            }
        }
        other => DbError::Internal(other.to_string()),
    }
}

pub fn from_keyring(err: keyring::Error) -> DbError {
    DbError::EncryptionFailed(err.to_string())
}
