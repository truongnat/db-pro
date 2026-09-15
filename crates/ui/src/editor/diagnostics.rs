pub use super::decorations::DiagnosticSeverity;

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
