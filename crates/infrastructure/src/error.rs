use db_pro_core::domain::error::{ConstraintType, DbError};

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
                    // Permission denied (SQLSTATE 42501) — check before generic syntax
                    if code_str == "42501" {
                        return DbError::PermissionDenied(db_err.message().into());
                    }
                    return DbError::QuerySyntax(db_err.message().into());
                }
                // Constraint violations (SQLSTATE 23xxx)
                if let Some(ct) = constraint_type_from_sqlstate(code_str) {
                    return DbError::ConstraintViolation {
                        constraint_type: ct,
                        constraint: db_err.constraint().unwrap_or_default().to_string(),
                        table: db_err.table().unwrap_or_default().to_string(),
                        column: None,
                        message: db_err.message().to_string(),
                    };
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

fn constraint_type_from_sqlstate(code: &str) -> Option<ConstraintType> {
    match code {
        "23505" => Some(ConstraintType::Unique),
        "23503" => Some(ConstraintType::ForeignKey),
        "23502" => Some(ConstraintType::NotNull),
        "23514" => Some(ConstraintType::Check),
        _ => None,
    }
}

pub fn from_rusqlite(err: rusqlite::Error) -> DbError {
    match err {
        rusqlite::Error::QueryReturnedNoRows => DbError::NotFound("no rows returned".into()),
        rusqlite::Error::SqliteFailure(ref ffi, ref msg) => {
            let message = msg.clone().unwrap_or_else(|| err.to_string());
            match ffi.code {
                rusqlite::ErrorCode::CannotOpen => DbError::ConnectionFailed(message),
                rusqlite::ErrorCode::ConstraintViolation => {
                    let constraint_type = match ffi.extended_code {
                        2067 => ConstraintType::Unique,    // SQLITE_CONSTRAINT_UNIQUE
                        787 => ConstraintType::ForeignKey, // SQLITE_CONSTRAINT_FOREIGNKEY
                        1299 => ConstraintType::NotNull,   // SQLITE_CONSTRAINT_NOTNULL
                        275 => ConstraintType::Check,      // SQLITE_CONSTRAINT_CHECK
                        _ => ConstraintType::Check,        // fallback for generic constraint
                    };
                    DbError::ConstraintViolation {
                        constraint_type,
                        constraint: String::new(),
                        table: String::new(),
                        column: None,
                        message,
                    }
                }
                _ => DbError::QueryFailed(message),
            }
        }
        other => DbError::Internal(other.to_string()),
    }
}

pub fn from_keyring(err: keyring::Error) -> DbError {
    DbError::EncryptionFailed(err.to_string())
}
