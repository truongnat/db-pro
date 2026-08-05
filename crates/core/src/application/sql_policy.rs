use crate::domain::error::DbError;

pub(crate) fn reject_multi_statement(sql: &str) -> Result<(), DbError> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err(DbError::QueryFailed("empty SQL statement".into()));
    }

    let trimmed = trimmed.trim_end_matches(';');
    let stripped = strip_single_quoted_strings(trimmed);
    if stripped.contains(';') {
        return Err(DbError::QueryFailed("multi-statement execution is disabled".into()));
    }

    Ok(())
}

fn strip_single_quoted_strings(sql: &str) -> String {
    let mut result = String::with_capacity(sql.len());
    let mut chars = sql.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            loop {
                match chars.next() {
                    Some('\'') => {
                        if chars.peek() == Some(&'\'') {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    None => break,
                    _ => {}
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

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
    fn strip_quoted_strings_handles_escapes() {
        assert_eq!(strip_single_quoted_strings("SELECT 'it''s'"), "SELECT ");
        assert_eq!(strip_single_quoted_strings("a;b'c;d'e"), "a;be");
    }
}
