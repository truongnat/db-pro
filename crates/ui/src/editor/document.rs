use super::buffer::TextBuffer;
use super::syntax::{SqlDialect, SqlHighlighter, SyntaxTokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlStatement {
    pub range: (usize, usize),
    pub text: String,
    pub has_semicolon: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SqlDocumentAnalysis {
    pub statements: Vec<SqlStatement>,
    pub version: u64,
}

impl SqlDocumentAnalysis {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn analyze(buffer: &TextBuffer, dialect: SqlDialect) -> Self {
        let text = buffer.text();
        let highlighter = SqlHighlighter::new(dialect);
        let tokens = highlighter.tokenize(text);

        let mut statements = Vec::new();
        let mut statement_start = None;
        let mut last_token_end = 0;

        for token in tokens {
            if token.kind == SyntaxTokenKind::Whitespace || token.kind == SyntaxTokenKind::Comment {
                continue;
            }

            if statement_start.is_none() {
                statement_start = Some(token.range.0);
            }

            if token.kind == SyntaxTokenKind::Punctuation && text[token.range.0..token.range.1] == *";" {
                if let Some(start) = statement_start.take() {
                    let end = token.range.1;
                    let statement_text = text[start..end].trim().to_owned();
                    if !statement_text.is_empty() {
                        statements.push(SqlStatement {
                            range: (start, end),
                            text: statement_text,
                            has_semicolon: true,
                        });
                    }
                }
            }
            last_token_end = token.range.1;
        }

        // Trailing statement without semicolon
        if let Some(start) = statement_start {
            let end = last_token_end.max(start);
            let statement_text = text[start..end].trim().to_owned();
            if !statement_text.is_empty() {
                statements.push(SqlStatement {
                    range: (start, end),
                    text: statement_text,
                    has_semicolon: false,
                });
            }
        }

        Self {
            statements,
            version: buffer.version(),
        }
    }

    pub fn current_statement_at(&self, offset: usize) -> Option<&SqlStatement> {
        self.statements
            .iter()
            .find(|stmt| offset >= stmt.range.0 && offset <= stmt.range.1)
    }

    pub fn all_statements(&self) -> &[SqlStatement] {
        &self.statements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statement_analysis_with_semicolon_inside_string() {
        let sql = "SELECT 'hello; world' AS greeting;\nSELECT id FROM users WHERE status = 'active';";
        let buf = TextBuffer::from_string(sql);
        let analysis = SqlDocumentAnalysis::analyze(&buf, SqlDialect::Postgres);
        assert_eq!(analysis.statements.len(), 2);
        assert_eq!(analysis.statements[0].text, "SELECT 'hello; world' AS greeting;");
        assert_eq!(
            analysis.statements[1].text,
            "SELECT id FROM users WHERE status = 'active';"
        );

        let stmt = analysis.current_statement_at(10).unwrap();
        assert_eq!(stmt.text, "SELECT 'hello; world' AS greeting;");

        let stmt2 = analysis.current_statement_at(40).unwrap();
        assert_eq!(stmt2.text, "SELECT id FROM users WHERE status = 'active';");
    }

    #[test]
    fn test_statement_analysis_with_dollar_quote() {
        let sql = "CREATE OR REPLACE FUNCTION test() RETURNS void AS $$ BEGIN SELECT 1; END; $$ LANGUAGE plpgsql;";
        let buf = TextBuffer::from_string(sql);
        let analysis = SqlDocumentAnalysis::analyze(&buf, SqlDialect::Postgres);
        assert_eq!(analysis.statements.len(), 1);
        assert_eq!(analysis.statements[0].text, sql);
    }
}
