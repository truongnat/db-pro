pub use super::decorations::DiagnosticSeverity;

use super::syntax::SqlDialect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSource {
    Parser,
    Delimiter,
    Database,
    Lint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub range: (usize, usize),
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: DiagnosticSource,
    pub code: Option<String>,
    /// Deterministic replacement for `range` when a safe one-shot rewrite exists (#257).
    pub fix: Option<String>,
}

impl Diagnostic {
    pub fn error(range: (usize, usize), message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            source: DiagnosticSource::Parser,
            code: None,
            fix: None,
        }
    }

    pub fn warning(range: (usize, usize), message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            source: DiagnosticSource::Parser,
            code: None,
            fix: None,
        }
    }

    pub fn lint(range: (usize, usize), message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            source: DiagnosticSource::Lint,
            code: Some(code.into()),
            fix: None,
        }
    }

    pub fn lint_with_fix(
        range: (usize, usize),
        message: impl Into<String>,
        code: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            source: DiagnosticSource::Lint,
            code: Some(code.into()),
            fix: Some(fix.into()),
        }
    }

    pub fn delimiter(range: (usize, usize), message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            source: DiagnosticSource::Delimiter,
            code: None,
            fix: None,
        }
    }

    pub fn database(range: (usize, usize), message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            source: DiagnosticSource::Database,
            code: None,
            fix: None,
        }
    }

    pub fn database_with_code(range: (usize, usize), message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            source: DiagnosticSource::Database,
            code: Some(code.into()),
            fix: None,
        }
    }
}

const SYNTAX_CODE: &str = "syntax";

/// Drop the previous statement diagnostic and parse only the statement around `cursor`.
pub(crate) fn sync_statement_syntax_diagnostic(
    diagnostics: &mut Vec<Diagnostic>,
    text: &str,
    cursor: usize,
    dialect: SqlDialect,
) {
    diagnostics.retain(|diagnostic| diagnostic.code.as_deref() != Some(SYNTAX_CODE));
    if let Some(diagnostic) = statement_syntax_diagnostic(text, cursor, dialect) {
        diagnostics.push(diagnostic);
    }
}

/// Parse the semicolon-bounded statement under `cursor`. `None` when it parses.
pub(crate) fn statement_syntax_diagnostic(text: &str, cursor: usize, dialect: SqlDialect) -> Option<Diagnostic> {
    let (raw_start, raw_end) = statement_range_at(text, cursor, dialect);
    let raw = text.get(raw_start..raw_end)?;
    let leading = raw.len() - raw.trim_start().len();
    let slice = raw.trim();
    if slice.is_empty() {
        return None;
    }
    let error = parse_statement(slice, dialect).err()?;
    let message = error.to_string();
    let local = error_line_column(&message)
        .map(|(line, column)| byte_offset_at_line_col(slice, line, column))
        .unwrap_or(0)
        .min(slice.len());
    let local = floor_char_boundary(slice, local);
    let mut end = local;
    if end < slice.len() {
        end += slice[end..].chars().next().map(char::len_utf8).unwrap_or(0);
        while end < slice.len() && end - local < 64 {
            let Some(ch) = slice[end..].chars().next() else {
                break;
            };
            if ch.is_whitespace() || ch == ';' {
                break;
            }
            end += ch.len_utf8();
        }
    }
    if end <= local {
        end = (local + slice[local..].chars().next().map(char::len_utf8).unwrap_or(0)).max(local);
    }
    let doc_start = raw_start + leading + local;
    let doc_end = (raw_start + leading + end).clamp(doc_start, raw_end.max(doc_start));
    if doc_start >= doc_end {
        return None;
    }
    let mut diagnostic = Diagnostic::error((doc_start, doc_end), message);
    diagnostic.code = Some(SYNTAX_CODE.to_owned());
    Some(diagnostic)
}

fn parse_statement(slice: &str, dialect: SqlDialect) -> Result<(), sqlparser::parser::ParserError> {
    let parsed = match dialect {
        SqlDialect::Postgres => sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::PostgreSqlDialect {}, slice),
        SqlDialect::SQLite => sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::SQLiteDialect {}, slice),
        SqlDialect::Generic => sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::GenericDialect {}, slice),
    };
    parsed.map(|_| ())
}

fn statement_range_at(text: &str, cursor: usize, dialect: SqlDialect) -> (usize, usize) {
    let bytes = text.as_bytes();
    let cursor = cursor.min(text.len());
    let mut start = 0usize;
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'-' && index + 1 < bytes.len() && bytes[index + 1] == b'-' {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if bytes[index] == b'/' && index + 1 < bytes.len() && bytes[index + 1] == b'*' {
            index += 2;
            while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/') {
                index += 1;
            }
            if index + 1 < bytes.len() {
                index += 2;
            } else {
                index = bytes.len();
            }
            continue;
        }
        if bytes[index] == b'\'' {
            index = skip_quoted(bytes, index + 1, b'\'');
            continue;
        }
        if bytes[index] == b'"' || bytes[index] == b'`' || (bytes[index] == b'[' && dialect == SqlDialect::SQLite) {
            let close = match bytes[index] {
                b'"' => b'"',
                b'`' => b'`',
                b'[' => b']',
                _ => b'"',
            };
            index = skip_quoted(bytes, index + 1, close);
            continue;
        }
        if dialect != SqlDialect::SQLite && bytes[index] == b'$' {
            if let Some(next) = skip_dollar_quote(text, index) {
                index = next;
                continue;
            }
        }
        if bytes[index] == b';' {
            let end = index + 1;
            if cursor < end {
                return (start, end);
            }
            start = end;
            index = end;
            continue;
        }
        index += 1;
    }
    (start, text.len())
}

fn skip_quoted(bytes: &[u8], mut index: usize, close: u8) -> usize {
    while index < bytes.len() {
        if bytes[index] == close {
            index += 1;
            if close == b'\'' && index < bytes.len() && bytes[index] == b'\'' {
                index += 1;
                continue;
            }
            break;
        }
        if close == b'\'' && bytes[index] == b'\\' && index + 1 < bytes.len() {
            index += 2;
        } else {
            index += 1;
        }
    }
    index
}

fn skip_dollar_quote(text: &str, index: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let tag_end = bytes[index + 1..].iter().position(|byte| *byte == b'$')?;
    let tag_len = tag_end + 2;
    let tag = text.get(index..index + tag_len)?;
    let body = &tag[1..tag.len() - 1];
    let valid = body.is_empty() || body.chars().all(|ch| ch.is_alphanumeric() || ch == '_');
    if !valid {
        return None;
    }
    let mut next = index + tag_len;
    if let Some(close) = text[next..].find(tag) {
        next += close + tag_len;
    } else {
        next = text.len();
    }
    Some(next)
}

fn error_line_column(message: &str) -> Option<(usize, usize)> {
    let line_at = message.rfind("Line: ")?;
    let column_at = message.rfind("Column: ")?;
    let line = leading_usize(&message[line_at + "Line: ".len()..])?;
    let column = leading_usize(&message[column_at + "Column: ".len()..])?;
    if line == 0 || column == 0 {
        None
    } else {
        Some((line, column))
    }
}

fn leading_usize(text: &str) -> Option<usize> {
    let digits: String = text.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn byte_offset_at_line_col(text: &str, line: usize, column: usize) -> usize {
    let mut current_line = 1usize;
    let mut current_column = 1usize;
    for (offset, ch) in text.char_indices() {
        if current_line == line && current_column == column {
            return offset;
        }
        if ch == '\n' {
            current_line += 1;
            current_column = 1;
        } else {
            current_column += 1;
        }
    }
    text.len()
}

fn floor_char_boundary(text: &str, mut offset: usize) -> usize {
    if offset > text.len() {
        return text.len();
    }
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_diagnostic_covers_only_the_statement_around_the_cursor() {
        let sql = "SELECT 1; SELECT 'a;b' FROM t; SELECT FROM; SELECT 2;";
        let bad_at = sql.find("SELECT FROM").expect("bad statement");
        let diagnostic = statement_syntax_diagnostic(sql, bad_at + 4, SqlDialect::Postgres).expect("syntax error");
        assert_eq!(diagnostic.code.as_deref(), Some("syntax"));
        assert!(diagnostic.range.0 >= bad_at);
        assert!(diagnostic.range.1 <= bad_at + "SELECT FROM;".len());
        assert!(statement_syntax_diagnostic(sql, 0, SqlDialect::Postgres).is_none());

        let mut diagnostics = vec![Diagnostic::error((0, 1), "old")];
        diagnostics[0].code = Some("syntax".to_owned());
        sync_statement_syntax_diagnostic(&mut diagnostics, "SELECT 1;", 0, SqlDialect::Postgres);
        assert!(diagnostics.is_empty());
    }
}
