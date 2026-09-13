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
}

impl CachedSqlTokens {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_recompute(&mut self, buffer: &TextBuffer, dialect: SqlDialect) -> &[SyntaxToken] {
        if !self.initialized || self.version != buffer.version() || self.dialect != dialect {
            let highlighter = SqlHighlighter::new(dialect);
            self.tokens = highlighter.tokenize(buffer.text());
            self.version = buffer.version();
            self.dialect = dialect;
            self.initialized = true;
        }
        &self.tokens
    }

    pub fn tokens(&self) -> &[SyntaxToken] {
        &self.tokens
    }

    pub fn invalidate(&mut self) {
        self.initialized = false;
    }
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
}
