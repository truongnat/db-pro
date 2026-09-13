pub use super::decorations::DiagnosticSeverity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSource {
    Parser,
    Delimiter,
    Database,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub range: (usize, usize),
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: DiagnosticSource,
    pub code: Option<String>,
}

impl Diagnostic {
    pub fn error(range: (usize, usize), message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            source: DiagnosticSource::Parser,
            code: None,
        }
    }

    pub fn warning(range: (usize, usize), message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            source: DiagnosticSource::Parser,
            code: None,
        }
    }

    pub fn delimiter(range: (usize, usize), message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            source: DiagnosticSource::Delimiter,
            code: None,
        }
    }

    pub fn database(range: (usize, usize), message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            source: DiagnosticSource::Database,
            code: None,
        }
    }

    pub fn database_with_code(range: (usize, usize), message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            source: DiagnosticSource::Database,
            code: Some(code.into()),
        }
    }
}
