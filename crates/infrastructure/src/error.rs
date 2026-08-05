use db_pro_core::domain::error::DbError;

pub fn from_sqlx(err: sqlx::Error) -> DbError {
    match err {
        sqlx::Error::PoolTimedOut => DbError::Timeout("connection pool timed out".into()),
        sqlx::Error::PoolClosed => DbError::ConnectionFailed("connection pool is closed".into()),
        sqlx::Error::Database(ref db_err) => {
            if let Some(code) = db_err.code() {
                let code_str = code.as_ref();
                if code_str == "28P01" || code_str == "28000" || code_str == "28004" {
                    return DbError::AuthFailed(db_err.message().into());
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
            if ffi.code == rusqlite::ErrorCode::CannotOpen {
                DbError::ConnectionFailed(msg.clone().unwrap_or_else(|| "cannot open database".into()))
            } else {
                DbError::QueryFailed(msg.clone().unwrap_or_else(|| err.to_string()))
            }
        }
        other => DbError::Internal(other.to_string()),
    }
}

pub fn from_keyring(err: keyring::Error) -> DbError {
    DbError::EncryptionFailed(err.to_string())
}
