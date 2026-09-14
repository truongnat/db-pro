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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_multi_statement_basic() {
        assert!(reject_multi_statement("SELECT 1").is_ok());
        assert!(reject_multi_statement("SELECT 1; SELECT 2").is_err());
        assert!(reject_multi_statement("  ").is_err());
        assert!(reject_multi_statement("SELECT 1;").is_ok());
    }

    #[test]
    fn reject_multi_statement_ignores_literals_identifiers_and_comments() {
        assert!(reject_multi_statement("SELECT ';'").is_ok());
        assert!(reject_multi_statement("SELECT \"semi;colon\"").is_ok());
        assert!(reject_multi_statement("SELECT 1 -- ;\n").is_ok());
        assert!(reject_multi_statement("SELECT 1 /* ; */").is_ok());
        assert!(reject_multi_statement("DO $body$ BEGIN PERFORM 1; END $body$").is_ok());
    }
}
