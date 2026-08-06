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

pub fn split_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut chars = sql.chars().peekable();
    let mut in_quote = false;

    while let Some(ch) = chars.next() {
        if in_quote {
            current.push(ch);
            if ch == '\'' {
                if chars.peek() == Some(&'\'') {
                    current.push(chars.next().unwrap());
                } else {
                    in_quote = false;
                }
            }
        } else if ch == '\'' {
            in_quote = true;
            current.push(ch);
        } else if ch == ';' {
            let stmt = current.trim().to_string();
            if !stmt.is_empty() {
                statements.push(stmt);
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }

    let stmt = current.trim().to_string();
    if !stmt.is_empty() {
        statements.push(stmt);
    }

    statements
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

    #[test]
    fn split_statements_basic() {
        let stmts = split_statements("SELECT 1; SELECT 2");
        assert_eq!(stmts, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn split_statements_trailing_semicolon() {
        let stmts = split_statements("SELECT 1; SELECT 2;");
        assert_eq!(stmts, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn split_statements_respects_quotes() {
        let stmts = split_statements("SELECT ';'; SELECT 2");
        assert_eq!(stmts, vec!["SELECT ';'", "SELECT 2"]);
    }

    #[test]
    fn split_statements_empty() {
        let stmts = split_statements("  ");
        assert!(stmts.is_empty());
    }
}
