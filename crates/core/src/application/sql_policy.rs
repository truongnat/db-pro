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

pub fn split_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let bytes = sql.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if let Some(end) = special_token_end(bytes, index) {
            current.push_str(&sql[index..end]);
            index = end;
        } else if bytes[index] == b';' {
            push_statement(&mut statements, &current);
            current.clear();
            index += 1;
        } else {
            let Some(character) = sql[index..].chars().next() else {
                break;
            };
            current.push(character);
            index += character.len_utf8();
        }
    }

    push_statement(&mut statements, &current);
    statements
}

fn push_statement(statements: &mut Vec<String>, current: &str) {
    let statement = current.trim();
    if !strip_leading_comments(statement).trim().is_empty() {
        statements.push(statement.to_owned());
    }
}

fn special_token_end(bytes: &[u8], start: usize) -> Option<usize> {
    match bytes[start] {
        b'\'' => Some(scan_quoted(bytes, start, b'\'')),
        b'"' => Some(scan_quoted(bytes, start, b'"')),
        b'-' if bytes.get(start + 1) == Some(&b'-') => Some(scan_line_comment(bytes, start)),
        b'/' if bytes.get(start + 1) == Some(&b'*') => Some(scan_block_comment(bytes, start)),
        b'$' => scan_dollar_quote(bytes, start),
        _ => None,
    }
}

fn scan_quoted(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut index = start + 1;
    while index < bytes.len() {
        if quote == b'\'' && bytes[index] == b'\\' {
            index = (index + 2).min(bytes.len());
        } else if bytes[index] == quote {
            if bytes.get(index + 1) == Some(&quote) {
                index += 2;
            } else {
                return index + 1;
            }
        } else {
            index += 1;
        }
    }
    bytes.len()
}

fn scan_line_comment(bytes: &[u8], start: usize) -> usize {
    bytes[start + 2..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(bytes.len(), |offset| start + 2 + offset + 1)
}

fn scan_block_comment(bytes: &[u8], start: usize) -> usize {
    let mut depth = 1;
    let mut index = start + 2;
    while index + 1 < bytes.len() {
        if bytes[index] == b'/' && bytes[index + 1] == b'*' {
            depth += 1;
            index += 2;
        } else if bytes[index] == b'*' && bytes[index + 1] == b'/' {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return index;
            }
        } else {
            index += 1;
        }
    }
    bytes.len()
}

fn scan_dollar_quote(bytes: &[u8], start: usize) -> Option<usize> {
    if start > 0 && is_identifier_continue(bytes[start - 1]) {
        return None;
    }

    let delimiter_end = bytes[start + 1..]
        .iter()
        .position(|byte| *byte == b'$')
        .map(|offset| start + 1 + offset)?;
    let tag = &bytes[start + 1..delimiter_end];
    if !tag.is_empty() && (!is_identifier_start(tag[0]) || !tag[1..].iter().all(|byte| is_identifier_continue(*byte))) {
        return None;
    }

    let delimiter = &bytes[start..=delimiter_end];
    let content_start = delimiter_end + 1;
    bytes[content_start..]
        .windows(delimiter.len())
        .position(|window| window == delimiter)
        .map_or(Some(bytes.len()), |offset| {
            Some(content_start + offset + delimiter.len())
        })
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn strip_leading_comments(sql: &str) -> &str {
    let mut value = sql.trim_start();
    loop {
        if value.starts_with("--") {
            value = value
                .find('\n')
                .map(|index| &value[index + 1..])
                .unwrap_or("")
                .trim_start();
        } else if value.starts_with("/*") {
            value = value
                .find("*/")
                .map(|index| &value[index + 2..])
                .unwrap_or("")
                .trim_start();
        } else {
            return value;
        }
    }
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
    fn reject_multi_statement_ignores_literals_identifiers_and_comments() {
        assert!(reject_multi_statement("SELECT ';'").is_ok());
        assert!(reject_multi_statement("SELECT \"semi;colon\"").is_ok());
        assert!(reject_multi_statement("SELECT 1 -- ;\n").is_ok());
        assert!(reject_multi_statement("SELECT 1 /* ; */").is_ok());
        assert!(reject_multi_statement("DO $body$ BEGIN PERFORM 1; END $body$").is_ok());
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
    fn split_statements_respects_comments_and_dollar_quotes() {
        let stmts =
            split_statements("DO $body$ BEGIN PERFORM 1; /* nested ; comment */ PERFORM 2; END $body$; SELECT 2");
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].contains("PERFORM 1;"));
        assert_eq!(stmts[1], "SELECT 2");
    }

    #[test]
    fn split_statements_drops_comment_only_tail() {
        assert_eq!(split_statements("SELECT 1; -- trailing comment\n"), vec!["SELECT 1"]);
    }

    #[test]
    fn split_statements_does_not_treat_identifier_dollar_signs_as_quotes() {
        let stmts = split_statements("SELECT foo$bar$; SELECT 2");
        assert_eq!(stmts, vec!["SELECT foo$bar$", "SELECT 2"]);
    }

    #[test]
    fn split_statements_empty() {
        let stmts = split_statements("  ");
        assert!(stmts.is_empty());
    }
}
