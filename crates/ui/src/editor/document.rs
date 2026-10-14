use super::{
    buffer::TextBuffer,
    syntax::{SqlDialect, SqlHighlighter, SyntaxToken, SyntaxTokenKind},
};

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
        let highlighter = SqlHighlighter::new(dialect);
        let tokens = highlighter.tokenize(buffer.text());
        Self::from_tokens(buffer.text(), &tokens, buffer.version())
    }

    /// Build statement ranges from already-computed highlight tokens (avoids a second tokenize).
    // cc-scan:allow COMPLEXITY,DEEP_NESTING — classifier/dispatch ladder — one case per branch
    pub fn from_tokens(text: &str, tokens: &[SyntaxToken], version: u64) -> Self {
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

        Self { statements, version }
    }

    pub fn current_statement_at(&self, offset: usize) -> Option<&SqlStatement> {
        self.statements
            .iter()
            .find(|stmt| offset >= stmt.range.0 && offset <= stmt.range.1)
    }

    /// Statement Run should execute for a caret that is not inside a selection.
    ///
    /// A blank line or comment after `;` still belongs to the statement that just
    /// ended, until the next statement starts. A caret sitting in leading
    /// whitespace belongs to the first statement. The full buffer is only used
    /// when this returns `None`.
    pub fn statement_for_caret(&self, offset: usize) -> Option<&SqlStatement> {
        if let Some(statement) = self.current_statement_at(offset) {
            return Some(statement);
        }
        let mut previous = None;
        for statement in &self.statements {
            if offset < statement.range.0 {
                return previous.or(Some(statement));
            }
            previous = Some(statement);
        }
        previous
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
    fn caret_in_the_gap_after_a_semicolon_belongs_to_that_statement() {
        let sql = "SELECT 1;\n\nSELECT 2;";
        let buf = TextBuffer::from_string(sql);
        let analysis = SqlDocumentAnalysis::analyze(&buf, SqlDialect::Postgres);

        let gap = analysis.statement_for_caret(10).unwrap();
        assert_eq!(gap.text, "SELECT 1;");

        let before = analysis.statement_for_caret(0).unwrap();
        assert_eq!(before.text, "SELECT 1;");

        let second = analysis.statement_for_caret(sql.find("SELECT 2").unwrap()).unwrap();
        assert_eq!(second.text, "SELECT 2;");

        let after_last = analysis.statement_for_caret(sql.len()).unwrap();
        assert_eq!(after_last.text, "SELECT 2;");
    }

    #[test]
    fn caret_before_the_first_statement_belongs_to_it() {
        let sql = "\n\nSELECT 1;";
        let buf = TextBuffer::from_string(sql);
        let analysis = SqlDocumentAnalysis::analyze(&buf, SqlDialect::Postgres);
        let statement = analysis.statement_for_caret(0).unwrap();
        assert_eq!(statement.text, "SELECT 1;");
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
