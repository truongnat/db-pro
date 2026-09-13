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
}

impl Diagnostic {
    pub fn error(range: (usize, usize), message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            source: DiagnosticSource::Parser,
        }
    }

    pub fn warning(range: (usize, usize), message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            source: DiagnosticSource::Parser,
        }
    }

    pub fn delimiter(range: (usize, usize), message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            source: DiagnosticSource::Delimiter,
        }
    }

    pub fn database(range: (usize, usize), message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            source: DiagnosticSource::Database,
        }
    }
}
