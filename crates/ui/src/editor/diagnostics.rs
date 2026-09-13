pub use super::decorations::DiagnosticSeverity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub range: (usize, usize),
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: String,
}

impl Diagnostic {
    pub fn error(range: (usize, usize), message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            source: "sql-parser".to_owned(),
        }
    }

    pub fn warning(range: (usize, usize), message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            source: "sql-analyzer".to_owned(),
        }
    }
}
