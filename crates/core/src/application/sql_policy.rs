use crate::domain::error::DbError;

pub(crate) fn reject_multi_statement(sql: &str) -> Result<(), DbError> {
    let statements = split_statements(sql);
    if statements.is_empty() {
        return Err(DbError::QueryFailed("empty SQL statement".into()));
    }

    if statements.len() > 1 {
        return Err(DbError::QueryFailed("multi-statement execution is disabled".into()));
    }

    Ok(())
}

// The statement splitter lives in the domain layer (`crate::domain::safety`) so that the
// safety classifier and the execution policy share one token/quote/dollar-quote-aware
// implementation instead of two drifting copies.
pub use crate::domain::safety::split_statements;

/// Leftover migration markers. Apply must refuse these instead of creating a fake table.
pub fn placeholder_remains(sql: &str) -> bool {
    let lower = sql.to_ascii_lowercase();
    lower.contains("/* placeholder")
        || lower.contains("/* type from source")
        || lower.contains("/* columns from source")
}

#[cfg(test)]
#[path = "tests/sql_policy_tests.rs"]
mod tests;
