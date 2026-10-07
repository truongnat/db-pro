use super::buffer::TextBuffer;
use crate::DbProTheme;
use egui::Color32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SqlDialect {
    #[default]
    Postgres,
    SQLite,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxTokenKind {
    Keyword,
    Function,
    Type,
    Identifier,
    String,
    DollarQuote,
    Number,
    Comment,
    Operator,
    Punctuation,
    Whitespace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxToken {
    pub range: (usize, usize),
    pub kind: SyntaxTokenKind,
}

#[derive(Debug, Clone, Default)]
pub struct CachedSqlTokens {
    tokens: Vec<SyntaxToken>,
    version: u64,
    dialect: SqlDialect,
    initialized: bool,
    /// Start byte of the last splice. `None` when the last update tokenized from byte 0.
    last_incremental_from: Option<usize>,
}

impl CachedSqlTokens {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_recompute(&mut self, buffer: &TextBuffer, dialect: SqlDialect) -> &[SyntaxToken] {
        if self.initialized && self.version == buffer.version() && self.dialect == dialect {
            return &self.tokens;
        }
        if self.try_incremental(buffer, dialect) {
            return &self.tokens;
        }
        let highlighter = SqlHighlighter::new(dialect);
        self.tokens = highlighter.tokenize(buffer.text());
        self.version = buffer.version();
        self.dialect = dialect;
        self.initialized = true;
        self.last_incremental_from = None;
        &self.tokens
    }

    /// Retokenize the edited line and the next one, then splice. Falls back when the
    /// edit is not a single version step, spans more than two lines, or a string/comment
    /// token crosses that window.
    fn try_incremental(&mut self, buffer: &TextBuffer, dialect: SqlDialect) -> bool {
        if !self.initialized || self.dialect != dialect || self.version.wrapping_add(1) != buffer.version() {
            return false;
        }
        let Some((start, old_end, new_end)) = buffer.last_edit else {
            return false;
        };
        let text = buffer.text();
        if start > new_end || new_end > text.len() || old_end < start {
            return false;
        }
        let delta = new_end as isize - old_end as isize;
        let (edit_line, _) = buffer.offset_to_line_col(start.min(text.len()));
        let mut region_start = buffer.line_start_offset(edit_line);
        let anchor = if new_end > start {
            new_end - 1
        } else {
            start.min(text.len())
        };
        let (end_line, _) = buffer.offset_to_line_col(anchor);
        let last_line = buffer.line_count().saturating_sub(1);
        let through = (end_line + 1).min(last_line);
        let mut region_end = if through + 1 < buffer.line_count() {
            buffer.line_start_offset(through + 1)
        } else {
            text.len()
        };
        if region_end < new_end {
            region_end = new_end;
        }
        let mut region_end_old = match (region_end as isize).checked_sub(delta) {
            Some(value) if value >= 0 => value as usize,
            _ => return false,
        };

        for _ in 0..self.tokens.len() {
            let Some(token) = self
                .tokens
                .iter()
                .find(|token| token.range.0 < region_start && token.range.1 > region_start)
            else {
                break;
            };
            if is_literal(token.kind) {
                return false;
            }
            region_start = token.range.0;
        }
        for _ in 0..self.tokens.len() {
            let Some(token) = self
                .tokens
                .iter()
                .find(|token| token.range.0 < region_end_old && token.range.1 > region_end_old)
            else {
                break;
            };
            if is_literal(token.kind) {
                return false;
            }
            let extra = token.range.1 - region_end_old;
            region_end_old = token.range.1;
            region_end = region_end.saturating_add(extra);
            if region_end > text.len() {
                return false;
            }
        }
        if region_start > start || region_end < new_end || region_end_old < old_end {
            return false;
        }
        let Some(slice) = text.get(region_start..region_end) else {
            return false;
        };
        let highlighter = SqlHighlighter::new(dialect);
        let mut fresh = highlighter.tokenize(slice);
        for token in &mut fresh {
            token.range.0 += region_start;
            token.range.1 += region_start;
        }

        let mut spliced = Vec::with_capacity(self.tokens.len());
        for token in &self.tokens {
            if token.range.1 <= region_start {
                spliced.push(token.clone());
            }
        }
        spliced.extend(fresh);
        for token in &self.tokens {
            if token.range.0 >= region_end_old {
                let Some(shifted_start) = shift_offset(token.range.0, delta) else {
                    return false;
                };
                let Some(shifted_end) = shift_offset(token.range.1, delta) else {
                    return false;
                };
                spliced.push(SyntaxToken {
                    range: (shifted_start, shifted_end),
                    kind: token.kind,
                });
            }
        }
        if !tokens_cover(&spliced, text.len()) {
            return false;
        }
        self.tokens = spliced;
        self.version = buffer.version();
        self.dialect = dialect;
        self.initialized = true;
        self.last_incremental_from = Some(region_start);
        true
    }

    pub fn tokens(&self) -> &[SyntaxToken] {
        &self.tokens
    }

    pub fn token_at(&self, offset: usize) -> Option<&SyntaxToken> {
        let index = self.tokens.partition_point(|token| token.range.1 <= offset);
        self.tokens
            .get(index)
            .filter(|token| offset >= token.range.0 && offset < token.range.1)
    }

    pub fn is_in_string_or_comment(&self, offset: usize) -> bool {
        self.token_at(offset).is_some_and(|token| {
            matches!(
                token.kind,
                SyntaxTokenKind::String | SyntaxTokenKind::DollarQuote | SyntaxTokenKind::Comment
            )
        })
    }

    pub fn invalidate(&mut self) {
        self.initialized = false;
    }
}

fn is_literal(kind: SyntaxTokenKind) -> bool {
    matches!(
        kind,
        SyntaxTokenKind::String | SyntaxTokenKind::DollarQuote | SyntaxTokenKind::Comment
    )
}

fn shift_offset(offset: usize, delta: isize) -> Option<usize> {
    offset.checked_add_signed(delta)
}

fn tokens_cover(tokens: &[SyntaxToken], len: usize) -> bool {
    let mut cursor = 0usize;
    for token in tokens {
        if token.range.0 != cursor || token.range.1 < token.range.0 {
            return false;
        }
        cursor = token.range.1;
    }
    cursor == len
}

const SQL_KEYWORDS: &[&str] = &[
    "SELECT",
    "FROM",
    "WHERE",
    "GROUP",
    "BY",
    "HAVING",
    "ORDER",
    "ASC",
    "DESC",
    "NULLS",
    "FIRST",
    "LAST",
    "LIMIT",
    "OFFSET",
    "JOIN",
    "INNER",
    "LEFT",
    "RIGHT",
    "FULL",
    "OUTER",
    "CROSS",
    "NATURAL",
    "ON",
    "USING",
    "UNION",
    "ALL",
    "INTERSECT",
    "EXCEPT",
    "INSERT",
    "INTO",
    "VALUES",
    "UPDATE",
    "SET",
    "DELETE",
    "TRUNCATE",
    "CREATE",
    "ALTER",
    "DROP",
    "TABLE",
    "VIEW",
    "INDEX",
    "SCHEMA",
    "DATABASE",
    "AS",
    "DISTINCT",
    "AND",
    "OR",
    "NOT",
    "IN",
    "IS",
    "NULL",
    "LIKE",
    "ILIKE",
    "BETWEEN",
    "EXISTS",
    "CASE",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    "CAST",
    "PRIMARY",
    "KEY",
    "FOREIGN",
    "REFERENCES",
    "CONSTRAINT",
    "CHECK",
    "UNIQUE",
    "DEFAULT",
    "CASCADE",
    "RESTRICT",
    "DEFERRABLE",
    "INITIALLY",
    "DEFERRED",
    "RETURNING",
    "WITH",
    "RECURSIVE",
    "PARTITION",
    "OVER",
    "WINDOW",
    "ROW",
    "ROWS",
    "RANGE",
    "PRECEDING",
    "FOLLOWING",
    "CURRENT",
    "UNBOUNDED",
    "COALESCE",
    "NULLIF",
    "BEGIN",
    "COMMIT",
    "ROLLBACK",
    "TRANSACTION",
    "SAVEPOINT",
    "RELEASE",
    "EXPLAIN",
    "ANALYZE",
    "PRAGMA",
    "VACUUM",
    "REINDEX",
    "ATTACH",
    "DETACH",
    "CONFLICT",
    "DO",
    "NOTHING",
];

const SQL_TYPES: &[&str] = &[
    "INT",
    "INTEGER",
    "BIGINT",
    "SMALLINT",
    "TINYINT",
    "NUMERIC",
    "DECIMAL",
    "REAL",
    "DOUBLE",
    "PRECISION",
    "FLOAT",
    "SERIAL",
    "BIGSERIAL",
    "VARCHAR",
    "CHAR",
    "CHARACTER",
    "VARYING",
    "TEXT",
    "BOOLEAN",
    "BOOL",
    "DATE",
    "TIME",
    "TIMESTAMP",
    "TIMESTAMPTZ",
    "INTERVAL",
    "UUID",
    "JSON",
    "JSONB",
    "BYTEA",
    "BLOB",
    "CLOB",
    "INET",
    "CIDR",
    "MACADDR",
];

const SQL_FUNCTIONS: &[&str] = &[
    "COUNT",
    "SUM",
    "AVG",
    "MIN",
    "MAX",
    "ROUND",
    "CEIL",
    "FLOOR",
    "ABS",
    "LOWER",
    "UPPER",
    "TRIM",
    "LTRIM",
    "RTRIM",
    "LENGTH",
    "SUBSTRING",
    "SUBSTR",
    "REPLACE",
    "CONCAT",
    "POSITION",
    "STRPOS",
    "NOW",
    "CURRENT_DATE",
    "CURRENT_TIME",
    "CURRENT_TIMESTAMP",
    "DATE_TRUNC",
    "AGE",
    "EXTRACT",
    "TO_CHAR",
    "TO_DATE",
    "TO_TIMESTAMP",
    "TO_NUMBER",
    "ROW_NUMBER",
    "RANK",
    "DENSE_RANK",
    "PERCENT_RANK",
    "CUME_DIST",
    "NTILE",
    "LAG",
    "LEAD",
    "FIRST_VALUE",
    "LAST_VALUE",
    "NTH_VALUE",
    "ARRAY_AGG",
    "STRING_AGG",
    "GROUP_CONCAT",
    "JSON_AGG",
    "JSON_OBJECT",
    "JSONB_BUILD_OBJECT",
    "GEN_RANDOM_UUID",
    "MD5",
    "SHA256",
];

pub struct SqlHighlighter {
    pub dialect: SqlDialect,
}

impl SqlHighlighter {
    pub fn new(dialect: SqlDialect) -> Self {
        Self { dialect }
    }

    pub fn tokenize(&self, text: &str) -> Vec<SyntaxToken> {
        let mut tokens = Vec::new();
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i < len {
            let start = i;
            let b = bytes[i];

            // Whitespace
            if b.is_ascii_whitespace() {
                while i < len && bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                tokens.push(SyntaxToken {
                    range: (start, i),
                    kind: SyntaxTokenKind::Whitespace,
                });
                continue;
            }

            // Line comment: --
            if b == b'-' && i + 1 < len && bytes[i + 1] == b'-' {
                i += 2;
                while i < len && bytes[i] != b'\n' {
                    i += 1;
                }
                tokens.push(SyntaxToken {
                    range: (start, i),
                    kind: SyntaxTokenKind::Comment,
                });
                continue;
            }

            // Block comment: /* ... */
            if b == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
                i += 2;
                while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < len {
                    i += 2;
                } else {
                    i = len;
                }
                tokens.push(SyntaxToken {
                    range: (start, i),
                    kind: SyntaxTokenKind::Comment,
                });
                continue;
            }

            // PostgreSQL Dollar quote: $$ or $tag$
            if self.dialect != SqlDialect::SQLite && b == b'$' {
                if let Some(tag_end) = bytes[i + 1..].iter().position(|&x| x == b'$') {
                    let tag = &text[i..=i + 1 + tag_end];
                    // Valid dollar quote tag is $ident$ or $$
                    let is_valid_tag =
                        tag == "$$" || tag[1..tag.len() - 1].chars().all(|c| c.is_alphanumeric() || c == '_');
                    if is_valid_tag {
                        i += tag.len();
                        if let Some(close_idx) = text[i..].find(tag) {
                            i += close_idx + tag.len();
                        } else {
                            i = len;
                        }
                        tokens.push(SyntaxToken {
                            range: (start, i),
                            kind: SyntaxTokenKind::DollarQuote,
                        });
                        continue;
                    }
                }
            }

            // String literal: '...'
            if b == b'\'' {
                i += 1;
                while i < len {
                    if bytes[i] == b'\'' {
                        i += 1;
                        // Escaped quote ''
                        if i < len && bytes[i] == b'\'' {
                            i += 1;
                            continue;
                        }
                        break;
                    }
                    if bytes[i] == b'\\' && i + 1 < len {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                tokens.push(SyntaxToken {
                    range: (start, i),
                    kind: SyntaxTokenKind::String,
                });
                continue;
            }

            // Quoted Identifier: "..." or `...` or [...]
            if b == b'"' || b == b'`' || (b == b'[' && self.dialect == SqlDialect::SQLite) {
                let close_char = match b {
                    b'"' => b'"',
                    b'`' => b'`',
                    b'[' => b']',
                    _ => b'"',
                };
                i += 1;
                while i < len && bytes[i] != close_char {
                    i += 1;
                }
                if i < len {
                    i += 1;
                }
                tokens.push(SyntaxToken {
                    range: (start, i),
                    kind: SyntaxTokenKind::Identifier,
                });
                continue;
            }

            // Number: digits or .digits
            if b.is_ascii_digit() || (b == b'.' && i + 1 < len && bytes[i + 1].is_ascii_digit()) {
                while i < len
                    && (bytes[i].is_ascii_digit()
                        || bytes[i] == b'.'
                        || bytes[i] == b'e'
                        || bytes[i] == b'E'
                        || bytes[i] == b'_')
                {
                    i += 1;
                }
                tokens.push(SyntaxToken {
                    range: (start, i),
                    kind: SyntaxTokenKind::Number,
                });
                continue;
            }

            // Identifiers / Keywords / Types / Functions
            if b.is_ascii_alphabetic() || b == b'_' || b >= 128 {
                while i < len {
                    let c = bytes[i];
                    if c.is_ascii_alphanumeric() || c == b'_' || c == b'$' || c >= 128 {
                        i += 1;
                    } else {
                        break;
                    }
                }
                let word = &text[start..i];
                let upper = word.to_ascii_uppercase();
                let kind = if SQL_KEYWORDS.contains(&upper.as_str()) {
                    SyntaxTokenKind::Keyword
                } else if SQL_TYPES.contains(&upper.as_str()) {
                    SyntaxTokenKind::Type
                } else if SQL_FUNCTIONS.contains(&upper.as_str()) {
                    SyntaxTokenKind::Function
                } else {
                    SyntaxTokenKind::Identifier
                };
                tokens.push(SyntaxToken {
                    range: (start, i),
                    kind,
                });
                continue;
            }

            // Operators & Punctuation
            if b == b';' || b == b',' || b == b'(' || b == b')' || b == b'[' || b == b']' || b == b'{' || b == b'}' {
                i += 1;
                tokens.push(SyntaxToken {
                    range: (start, i),
                    kind: SyntaxTokenKind::Punctuation,
                });
                continue;
            }

            // Multi-char operators like >=, <=, !=, <>, ::, ->, ->>, etc.
            i += 1;
            while i < len {
                let c = bytes[i];
                if matches!(
                    c,
                    b'=' | b'<' | b'>' | b'!' | b'+' | b'-' | b'*' | b'/' | b'%' | b':' | b'|' | b'&' | b'~' | b'^'
                ) {
                    i += 1;
                } else {
                    break;
                }
            }
            tokens.push(SyntaxToken {
                range: (start, i),
                kind: SyntaxTokenKind::Operator,
            });
        }

        tokens
    }

    pub fn token_color(&self, kind: SyntaxTokenKind, theme: &DbProTheme) -> Color32 {
        match kind {
            SyntaxTokenKind::Keyword => theme.code_keyword,
            SyntaxTokenKind::Function => theme.code_function,
            SyntaxTokenKind::Type => theme.code_type,
            SyntaxTokenKind::Identifier => theme.code_variable,
            SyntaxTokenKind::String | SyntaxTokenKind::DollarQuote => theme.code_string,
            SyntaxTokenKind::Number => theme.code_number,
            SyntaxTokenKind::Comment => theme.code_comment,
            SyntaxTokenKind::Operator => theme.code_operator,
            SyntaxTokenKind::Punctuation => theme.code_punctuation,
            SyntaxTokenKind::Whitespace => theme.text_primary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_tokens_invalidation() {
        let mut buf = TextBuffer::from_string("SELECT 1;");
        let mut cache = CachedSqlTokens::new();

        let tokens1 = cache.get_or_recompute(&buf, SqlDialect::Postgres);
        assert_eq!(tokens1.len(), 4); // SELECT, ' ', 1, ';'

        // Unmodified buffer returns cached tokens
        let len_cached = cache.tokens().len();
        assert_eq!(len_cached, 4);

        // Edit buffer
        buf.insert(9, " SELECT 2;");
        let tokens2 = cache.get_or_recompute(&buf, SqlDialect::Postgres);
        assert_eq!(tokens2.len(), 9);
    }

    #[test]
    fn test_is_in_string_or_comment() {
        let buf = TextBuffer::from_string("SELECT 'hello world', -- comment\n1;");
        let mut cache = CachedSqlTokens::new();
        cache.get_or_recompute(&buf, SqlDialect::Postgres);

        assert!(cache.is_in_string_or_comment(10)); // inside 'hello world'
        assert!(!cache.is_in_string_or_comment(2)); // inside SELECT
        assert!(cache.is_in_string_or_comment(26)); // inside comment
    }

    fn large_sql() -> String {
        let mut text = String::from("SELECT id FROM orders WHERE status = 'open';\n");
        for index in 0..180 {
            text.push_str(&format!("SELECT col_{index} FROM table_{index} WHERE id = {index};\n"));
        }
        text
    }

    fn assert_tokens_match(actual: &[SyntaxToken], expected: &[SyntaxToken]) {
        assert_eq!(actual.len(), expected.len(), "token count");
        for (index, (left, right)) in actual.iter().zip(expected).enumerate() {
            assert_eq!(left, right, "token {index}");
        }
    }

    #[test]
    fn incremental_token_splice_matches_full_tokenize_outside_a_string() {
        let text = large_sql();
        let needle = "FROM table_90";
        let insert_at = text.find(needle).expect("needle") + 2;
        let mut buf = TextBuffer::from_string(&text);
        let mut cache = CachedSqlTokens::new();
        cache.get_or_recompute(&buf, SqlDialect::Postgres);
        buf.insert(insert_at, "X");

        let spliced = cache.get_or_recompute(&buf, SqlDialect::Postgres).to_vec();
        assert!(
            cache.last_incremental_from.is_some_and(|from| from > 0),
            "edit outside a string must not retokenize from byte 0"
        );
        let full = SqlHighlighter::new(SqlDialect::Postgres).tokenize(buf.text());
        assert_tokens_match(&spliced, &full);
        let string_at = buf.text().find("'open'").expect("string") + 2;
        assert!(cache.is_in_string_or_comment(string_at));
        assert!(!cache.is_in_string_or_comment(insert_at));
    }

    #[test]
    fn incremental_token_multiline_comment_falls_back_to_full_tokenize() {
        let mut commented = String::from("/*\n");
        commented.push_str(&large_sql());
        commented.push_str("*/\nSELECT ready;\n");
        let insert_at = commented.find("FROM table_90").expect("needle") + 2;
        let mut buf = TextBuffer::from_string(&commented);
        let mut cache = CachedSqlTokens::new();
        cache.get_or_recompute(&buf, SqlDialect::Postgres);
        buf.insert(insert_at, "X");

        let spliced = cache.get_or_recompute(&buf, SqlDialect::Postgres).to_vec();
        assert!(cache.last_incremental_from.is_none());
        let full = SqlHighlighter::new(SqlDialect::Postgres).tokenize(buf.text());
        assert_tokens_match(&spliced, &full);
    }
}
