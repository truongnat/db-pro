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
#[path = "tests/document_tests.rs"]
mod tests;
