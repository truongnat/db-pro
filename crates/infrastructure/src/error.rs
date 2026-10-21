use db_pro_core::domain::error::{ConstraintType, ConstraintViolationDetails, DbError};

// cc-scan:allow DEEP_NESTING — nesting mirrors structure depth
pub fn from_sqlx(err: sqlx::Error) -> DbError {
    match err {
        sqlx::Error::PoolTimedOut => DbError::ConnectionTimeout("connection pool timed out".into()),
        sqlx::Error::PoolClosed => DbError::ConnectionFailed("connection pool is closed".into()),
        sqlx::Error::Database(ref db_err) => {
            let (message, position) = database_message(db_err.as_ref());
            if let Some(code) = db_err.code() {
                let code_str = code.as_ref();
                // Authentication errors (SQLSTATE 28xxx)
                if code_str == "28P01" || code_str == "28000" || code_str == "28004" {
                    return DbError::AuthFailed(message);
                }
                // Connection exceptions (SQLSTATE 08xxx)
                if code_str.starts_with("08") {
                    return DbError::ConnectionRefused(message);
                }
                // Syntax errors (SQLSTATE 42xxx)
                if code_str.starts_with("42") {
                    // Permission denied (SQLSTATE 42501) — check before generic syntax
                    if code_str == "42501" {
                        return DbError::PermissionDenied(message);
                    }
                    return query_syntax_error(message, position);
                }
                // Constraint violations (SQLSTATE 23xxx)
                if let Some(ct) = constraint_type_from_sqlstate(code_str) {
                    return DbError::ConstraintViolation {
                        details: Box::new(ConstraintViolationDetails {
                            constraint_type: ct,
                            constraint: db_err.constraint().unwrap_or_default().to_string(),
                            table: db_err.table().unwrap_or_default().to_string(),
                            column: None,
                            message,
                        }),
                    };
                }
                // Database not found (SQLSTATE 3D000)
                if code_str == "3D000" {
                    return DbError::DatabaseNotFound(db_err.message().into());
                }
            }
            query_failed_error(message, position)
        }
        sqlx::Error::Io(e) => DbError::Io(e.to_string()),
        sqlx::Error::RowNotFound => DbError::NotFound("row not found".into()),
        other => DbError::Internal(other.to_string()),
    }
}

fn database_message(error: &dyn sqlx::error::DatabaseError) -> (String, Option<usize>) {
    let pg = error.as_error().downcast_ref::<sqlx::postgres::PgDatabaseError>();
    let position = pg.and_then(|error| match error.position() {
        Some(sqlx::postgres::PgErrorPosition::Original(position)) => Some(position),
        Some(sqlx::postgres::PgErrorPosition::Internal { .. }) | None => None,
    });
    // PL/pgSQL failures put the frame trace in the `Where` field and extra
    // context in `Detail`; dropping them left the Messages pane with only the
    // top-line message (#252). They are appended to the single failure payload
    // string the UI already renders — the structured severity/code fields stay
    // owned by the variant mapping above.
    // Driver limitation (verified against sqlx 0.8.6): server `NOTICE`/`WARNING`/
    // `INFO` messages arrive only as `NoticeResponse` frames that sqlx routes to
    // the `sqlx::postgres::notice` log target — no query-level API surfaces them,
    // so NOTICEs are not yet part of this payload.
    let mut message = error.message().to_owned();
    if let Some(pg) = pg {
        if let Some(detail) = pg.detail() {
            if !detail.trim().is_empty() {
                message.push_str("\nDETAIL: ");
                message.push_str(detail);
            }
        }
        if let Some(frame) = pg.r#where() {
            if !frame.trim().is_empty() {
                message.push_str("\nCONTEXT: ");
                message.push_str(frame);
            }
        }
    }
    (message, position)
}

fn query_syntax_error(message: String, position: Option<usize>) -> DbError {
    match position {
        Some(position) => DbError::QuerySyntaxAt { message, position },
        None => DbError::QuerySyntax(message),
    }
}

fn query_failed_error(message: String, position: Option<usize>) -> DbError {
    match position {
        Some(position) => DbError::QueryFailedAt { message, position },
        None => DbError::QueryFailed(message),
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

// cc-scan:allow DEEP_NESTING — nesting mirrors structure depth
pub fn from_rusqlite(err: rusqlite::Error) -> DbError {
    match err {
        rusqlite::Error::QueryReturnedNoRows => DbError::NotFound("no rows returned".into()),
        rusqlite::Error::SqliteFailure(ref ffi, ref msg) => {
            let message = msg.clone().unwrap_or_else(|| err.to_string());
            match ffi.code {
                rusqlite::ErrorCode::CannotOpen => DbError::ConnectionFailed(message),
                rusqlite::ErrorCode::ConstraintViolation => {
                    let constraint_type = match ffi.extended_code {
                        rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY => ConstraintType::PrimaryKey,
                        rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE => ConstraintType::Unique,
                        rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY => ConstraintType::ForeignKey,
                        rusqlite::ffi::SQLITE_CONSTRAINT_NOTNULL => ConstraintType::NotNull,
                        rusqlite::ffi::SQLITE_CONSTRAINT_CHECK => ConstraintType::Check,
                        code => ConstraintType::Other(code),
                    };
                    DbError::ConstraintViolation {
                        details: Box::new(ConstraintViolationDetails {
                            constraint_type,
                            constraint: String::new(),
                            table: String::new(),
                            column: None,
                            message,
                        }),
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

#[cfg(test)]
#[path = "tests/error_tests.rs"]
mod tests;
