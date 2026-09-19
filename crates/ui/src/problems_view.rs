//! Problem list helpers and diagnostics summary export.
use super::*;

impl DbProApp {
    pub(crate) fn collect_problem_entries(&self) -> Vec<ProblemEntry> {
        let mut entries = Vec::new();
        for (document_index, document) in self.query_session_state.documents.iter().enumerate() {
            for (diagnostic_index, diagnostic) in document.diagnostics.iter().enumerate() {
                let cursor = crate::editor::CursorPosition::from_offset(&document.buffer, diagnostic.range.0);
                entries.push(ProblemEntry {
                    document_index,
                    document_id: document.id.clone(),
                    document_title: document.title.clone(),
                    diagnostic_index,
                    severity: diagnostic.severity,
                    source: diagnostic.source,
                    message: diagnostic.message.clone(),
                    line: cursor.line,
                    column: cursor.col,
                    range: diagnostic.range,
                    has_fix: diagnostic.fix.is_some(),
                });
            }
        }
        for (index, diagnostic) in self
            .workspace
            .files
            .ide_workspace
            .workspace_diagnostics
            .iter()
            .enumerate()
        {
            let severity = match diagnostic.severity {
                ide_workspace::WorkspaceDiagnosticSeverity::Error => crate::editor::DiagnosticSeverity::Error,
                ide_workspace::WorkspaceDiagnosticSeverity::Warning => crate::editor::DiagnosticSeverity::Warning,
            };
            entries.push(ProblemEntry {
                document_index: usize::MAX,
                document_id: format!("{}::{}", diagnostic.root_id, diagnostic.relative_path),
                document_title: diagnostic.relative_path.clone(),
                diagnostic_index: index,
                severity,
                source: crate::editor::DiagnosticSource::Lint,
                message: diagnostic.message.clone(),
                line: diagnostic.line.saturating_sub(1),
                column: 0,
                range: (0, 0),
                has_fix: false,
            });
        }
        entries
    }

    pub(super) fn problem_matches_filters(&self, entry: &ProblemEntry) -> bool {
        let severity_ok = match self.query_editor.problems_severity_filter {
            ProblemsSeverityFilter::All => true,
            ProblemsSeverityFilter::Errors => entry.severity == crate::editor::DiagnosticSeverity::Error,
            ProblemsSeverityFilter::Warnings => entry.severity == crate::editor::DiagnosticSeverity::Warning,
        };
        let source_ok = match self.query_editor.problems_source_filter {
            ProblemsSourceFilter::All => true,
            ProblemsSourceFilter::Parser => entry.source == crate::editor::DiagnosticSource::Parser,
            ProblemsSourceFilter::Lint => entry.source == crate::editor::DiagnosticSource::Lint,
            ProblemsSourceFilter::Delimiter => entry.source == crate::editor::DiagnosticSource::Delimiter,
            ProblemsSourceFilter::Database => entry.source == crate::editor::DiagnosticSource::Database,
        };
        severity_ok && source_ok
    }

    pub(crate) fn navigate_to_problem(&mut self, document_index: usize, diagnostic_index: usize) {
        let Some(document) = self.query_session_state.documents.get(document_index) else {
            return;
        };
        let Some(diagnostic) = document.diagnostics.get(diagnostic_index).cloned() else {
            return;
        };
        if document_index != self.query_session_state.active_document_index {
            self.query_session_state.active_document_index = document_index;
        }
        let doc = &mut self.query_session_state.documents[document_index];
        let start = diagnostic.range.0.min(doc.buffer.len_bytes());
        let end = diagnostic.range.1.min(doc.buffer.len_bytes()).max(start);
        doc.cursor = crate::editor::CursorPosition::from_offset(&doc.buffer, start);
        doc.selection = crate::editor::SelectionRange::new(start, end);
        self.query_editor.query_cursor_line = doc.cursor.line + 1;
        self.query_editor.query_cursor_column = doc.cursor.col + 1;
        if start != end {
            self.query_session_state.selected_text = doc.buffer.slice(start, end).to_owned();
        } else {
            self.query_session_state.selected_text.clear();
        }
        self.workspace.activity = Activity::Problems;
        self.workspace.sidebar_open = true;
        self.workspace.active_tab = WorkspaceTab::Query;
        self.query_editor.problems_selected = Some((doc.id.clone(), diagnostic_index));
        self.feedback.runtime_message = format!("Jumped to problem in {}", doc.title);
    }

    /// Apply a deterministic lint quick-fix as one undoable buffer replace (#257).
    pub(crate) fn apply_problem_fix(&mut self, document_index: usize, diagnostic_index: usize) -> bool {
        let Some(document) = self.query_session_state.documents.get(document_index) else {
            return false;
        };
        let Some(diagnostic) = document.diagnostics.get(diagnostic_index).cloned() else {
            return false;
        };
        let Some(fix) = diagnostic.fix.clone() else {
            return false;
        };
        let (start, end) = diagnostic.range;
        if start > end || end > document.buffer.len_bytes() {
            return false;
        }
        if document_index != self.query_session_state.active_document_index {
            self.query_session_state.active_document_index = document_index;
        }
        let doc = &mut self.query_session_state.documents[document_index];
        doc.buffer.replace(start, end, &fix);
        let new_end = start + fix.len();
        doc.cursor = crate::editor::CursorPosition::from_offset(&doc.buffer, new_end);
        doc.selection = crate::editor::SelectionRange::new(start, new_end);
        doc.dirty = true;
        self.query_editor.query_cursor_line = doc.cursor.line + 1;
        self.query_editor.query_cursor_column = doc.cursor.col + 1;
        let title = doc.title.clone();
        self.workspace.active_tab = WorkspaceTab::Query;
        self.refresh_diagnostics();
        self.feedback.runtime_message = format!("Applied quick fix in {title}");
        true
    }

    pub(crate) fn build_diagnostics_summary(&self) -> db_pro_core::domain::diagnostics::DiagnosticsSummary {
        use db_pro_core::domain::diagnostics::{
            redact_sensitive, ConnectionDiagnostic, DiagnosticsSummary, DriverDiagnostic, ErrorDiagnostic,
        };

        let mut summary = DiagnosticsSummary::placeholder();
        summary.connections = self
            .connection
            .catalog
            .iter()
            .map(|connection| ConnectionDiagnostic {
                connection_id: connection.id.clone(),
                driver: connection.driver.clone(),
                host: redact_sensitive(&connection.host),
                port: connection.port,
                database: connection.database.clone(),
                username: connection.username.clone(),
                has_password: true,
                has_ssh: false,
                is_connected: self.connection.lifecycle.active_connection_id() == Some(connection.id.as_str()),
            })
            .collect();
        summary.runtime.active_connections = usize::from(self.connection.lifecycle.active_connection_id().is_some());
        summary.runtime.active_executions = self
            .query_session_state
            .documents
            .iter()
            .filter(|doc| {
                matches!(
                    doc.execution_state,
                    crate::query::query_document::QueryExecutionState::Running(_)
                )
            })
            .count();
        if self.has_runtime_error() && !self.feedback.runtime_message.trim().is_empty() {
            summary.recent_errors.push(ErrorDiagnostic {
                timestamp: chrono::Utc::now().to_rfc3339(),
                error_code: "UI_RUNTIME".to_owned(),
                message: redact_sensitive(&self.feedback.runtime_message),
                module: "ui".to_owned(),
            });
        }
        // Ensure MySQL stays listed alongside PG/SQLite in the shipped driver set.
        if !summary.drivers.iter().any(|d| d.driver == "mysql") {
            summary.drivers.push(DriverDiagnostic {
                driver: "mysql".into(),
                available: true,
            });
        }
        summary
    }

    /// Write a redacted diagnostics support bundle next to the backup path or temp (#214).
    pub(crate) fn export_support_bundle(&self) -> Result<String, String> {
        let summary = self.build_diagnostics_summary();
        let json = serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?;
        let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let file_name = format!("db-pro-support-bundle-{stamp}.json");
        let path = if !self.overlay.backup_output_path.trim().is_empty() {
            let parent = std::path::Path::new(self.overlay.backup_output_path.trim())
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            parent.join(&file_name)
        } else {
            std::env::temp_dir().join(&file_name)
        };
        std::fs::write(&path, json.as_bytes()).map_err(|e| e.to_string())?;
        Ok(path.display().to_string())
    }
}
