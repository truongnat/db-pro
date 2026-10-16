use crate::domain::connection::DriverType;

/// How a provider treats a backslash inside a quoted string literal.
///
/// This is a lexing concern only: it decides whether `\` before a quote keeps
/// the literal open. It is not a full SQL parser, and it does not model every
/// provider session setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SqlStringDialect {
    /// PostgreSQL with `standard_conforming_strings=on` (the server default since
    /// 9.1): a backslash is an ordinary character inside `'…'`. Only the `E'…'`
    /// escape-string form changes how the literal is scanned. This is the default
    /// because the reported regression is PostgreSQL's default behavior.
    #[default]
    Postgres,
    /// MySQL/MariaDB without `NO_BACKSLASH_ESCAPES` (the default `sql_mode`):
    /// `\` escapes the next byte inside both `'…'` and `"…"` string literals.
    Mysql,
    /// SQLite and SQL Server: backslash is always literal; only a doubled quote
    /// closes a string. Neither provider supports the `E'…'` escape-string form.
    Standard,
}

impl SqlStringDialect {
    pub fn from_driver(driver: DriverType) -> Self {
        match driver {
            DriverType::Postgres => Self::Postgres,
            DriverType::Mysql => Self::Mysql,
            DriverType::SQLite | DriverType::SqlServer => Self::Standard,
        }
    }
}

pub(super) fn special_token_end(bytes: &[u8], start: usize, dialect: SqlStringDialect) -> Option<usize> {
    match bytes[start] {
        b'\'' => Some(skip_quoted(bytes, start, b'\'', dialect)),
        b'"' => Some(skip_quoted(bytes, start, b'"', dialect)),
        b'-' if bytes.get(start + 1) == Some(&b'-') => Some(skip_line_comment(bytes, start)),
        b'/' if bytes.get(start + 1) == Some(&b'*') => Some(skip_block_comment(bytes, start)),
        b'$' => skip_dollar_quote(bytes, start),
        _ => None,
    }
}

pub(super) fn matching_parenthesis_end(sql: &str) -> Option<usize> {
    let bytes = sql.as_bytes();
    let mut depth = 0;
    let mut index = 0;
    while index < bytes.len() {
        index = match bytes[index] {
            b'\'' => skip_quoted(bytes, index, b'\'', SqlStringDialect::Postgres),
            b'"' => skip_quoted(bytes, index, b'"', SqlStringDialect::Postgres),
            b'-' if bytes.get(index + 1) == Some(&b'-') => skip_line_comment(bytes, index),
            b'/' if bytes.get(index + 1) == Some(&b'*') => skip_block_comment(bytes, index),
            b'$' => skip_dollar_quote(bytes, index).unwrap_or(index + 1),
            b'(' => {
                depth += 1;
                index + 1
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index + 1);
                }
                index + 1
            }
            _ => index + 1,
        };
    }

    None
}

pub(super) fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

pub(super) fn is_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

pub(super) fn scan_identifier(bytes: &[u8], start: usize) -> usize {
    let mut end = start + 1;
    while end < bytes.len() && is_identifier_continue(bytes[end]) {
        end += 1;
    }
    end
}

fn has_escape_string_prefix(bytes: &[u8], quote: usize) -> bool {
    quote > 0 && matches!(bytes[quote - 1], b'e' | b'E') && (quote == 1 || !is_identifier_continue(bytes[quote - 2]))
}

pub(super) fn skip_quoted(bytes: &[u8], start: usize, quote: u8, dialect: SqlStringDialect) -> usize {
    let backslash_escapes = match dialect {
        SqlStringDialect::Mysql => true,
        SqlStringDialect::Postgres => quote == b'\'' && has_escape_string_prefix(bytes, start),
        SqlStringDialect::Standard => false,
    };

    let mut index = start + 1;
    while index < bytes.len() {
        if backslash_escapes && bytes[index] == b'\\' {
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

pub(super) fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
    bytes[start + 2..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(bytes.len(), |offset| start + 2 + offset + 1)
}

pub(super) fn skip_block_comment(bytes: &[u8], start: usize) -> usize {
    let mut depth = 1usize;
    let mut index = start + 2;
    while index + 1 < bytes.len() {
        if &bytes[index..index + 2] == b"/*" {
            depth += 1;
            index += 2;
        } else if &bytes[index..index + 2] == b"*/" {
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

pub(super) fn skip_dollar_quote(bytes: &[u8], start: usize) -> Option<usize> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SqlTokenKind {
    Word,
    OpenParen,
    CloseParen,
    Comma,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SqlToken {
    pub(super) kind: SqlTokenKind,
    pub(super) start: usize,
    pub(super) end: usize,
}

pub(super) fn tokenize_sql(sql: &str) -> Vec<SqlToken> {
    let bytes = sql.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        if let Some(end) = match bytes[index] {
            b'\'' => Some(skip_quoted(bytes, index, b'\'', SqlStringDialect::Postgres)),
            b'"' => Some(skip_quoted(bytes, index, b'"', SqlStringDialect::Postgres)),
            b'-' if bytes.get(index + 1) == Some(&b'-') => Some(skip_line_comment(bytes, index)),
            b'/' if bytes.get(index + 1) == Some(&b'*') => Some(skip_block_comment(bytes, index)),
            b'$' => skip_dollar_quote(bytes, index),
            _ => None,
        } {
            index = end;
            continue;
        }

        let kind = match bytes[index] {
            b'(' => Some(SqlTokenKind::OpenParen),
            b')' => Some(SqlTokenKind::CloseParen),
            b',' => Some(SqlTokenKind::Comma),
            byte if is_identifier_start(byte) => {
                let end = scan_identifier(bytes, index);
                tokens.push(SqlToken {
                    kind: SqlTokenKind::Word,
                    start: index,
                    end,
                });
                index = end;
                continue;
            }
            _ => None,
        };

        if let Some(kind) = kind {
            tokens.push(SqlToken {
                kind,
                start: index,
                end: index + 1,
            });
        }
        index += 1;
    }

    tokens
}

pub(super) fn token_is_word(sql: &str, token: SqlToken, expected: &str) -> bool {
    token.kind == SqlTokenKind::Word && sql[token.start..token.end].eq_ignore_ascii_case(expected)
}
