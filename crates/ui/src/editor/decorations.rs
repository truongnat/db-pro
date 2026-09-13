use super::syntax::SyntaxTokenKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecorationKind {
    Syntax(SyntaxTokenKind),
    CurrentLine,
    Selection,
    SearchMatch { is_active: bool },
    DiagnosticUnderline { severity: DiagnosticSeverity },
    ExecutionHighlight,
    GhostText(String),
    BracketMatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDecoration {
    pub range: (usize, usize),
    pub kind: DecorationKind,
}

impl EditorDecoration {
    pub fn new(range: (usize, usize), kind: DecorationKind) -> Self {
        Self { range, kind }
    }
}
