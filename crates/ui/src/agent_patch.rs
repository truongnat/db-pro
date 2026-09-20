//! Safe application of an Agent SQL patch to a query document.

use crate::query::QueryDocument;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AgentPatchApplyError {
    DocumentChanged,
    InvalidRange,
}

pub(super) fn apply_to_document(
    document: &mut QueryDocument,
    patch: &db_pro_core::domain::agent::AgentSqlPatch,
) -> Result<db_pro_core::domain::agent::AgentToolOutput, AgentPatchApplyError> {
    if document.id != patch.document_id || document.buffer.version() != patch.expected_version {
        return Err(AgentPatchApplyError::DocumentChanged);
    }
    let current = super::agent_context::document_snapshot(document);
    if patch
        .apply_to(&current.document_id, current.document_version, &current.sql)
        .is_err()
    {
        return Err(AgentPatchApplyError::InvalidRange);
    }

    let (start, end) = patch.range;
    let before = crate::editor::buffer::EditorSnapshot {
        cursor_offset: document.cursor.offset,
        anchor_offset: document.selection.anchor,
    };
    let new_cursor = start + patch.replacement.len();
    let after = crate::editor::buffer::EditorSnapshot {
        cursor_offset: new_cursor,
        anchor_offset: new_cursor,
    };
    document
        .buffer
        .replace_with_snapshot(start, end, &patch.replacement, before, after);
    document.cursor.set_offset(&document.buffer, new_cursor);
    document.selection = crate::editor::selection::SelectionRange::point(new_cursor);
    document.reanalyze(crate::editor::syntax::SqlDialect::Postgres);
    document.invalidate_prediction();
    document.completion = crate::editor::completion::CompletionState::default();
    document.execution_diagnostic = None;
    document
        .diagnostics
        .retain(|diagnostic| diagnostic.source != crate::editor::diagnostics::DiagnosticSource::Database);
    document.dirty = true;

    Ok(db_pro_core::domain::agent::AgentToolOutput::PatchApplied {
        document_id: patch.document_id.clone(),
        new_version: document.buffer.version(),
        range: patch.range,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_patch_and_marks_document_dirty() {
        let mut document = crate::query::QueryDocument::new("query-1", "Query", "select 1");
        let patch = db_pro_core::domain::agent::AgentSqlPatch {
            document_id: document.id.clone(),
            expected_version: document.buffer.version(),
            range: (7, 8),
            replacement: "2".to_owned(),
        };

        let result = apply_to_document(&mut document, &patch);

        assert!(matches!(
            result,
            Ok(db_pro_core::domain::agent::AgentToolOutput::PatchApplied { .. })
        ));
        assert_eq!(document.text(), "select 2");
        assert!(document.dirty);
    }

    #[test]
    fn rejects_stale_document_before_mutating_it() {
        let mut document = crate::query::QueryDocument::new("query-1", "Query", "select 1");
        let patch = db_pro_core::domain::agent::AgentSqlPatch {
            document_id: document.id.clone(),
            expected_version: document.buffer.version() + 1,
            range: (7, 8),
            replacement: "2".to_owned(),
        };

        assert_eq!(
            apply_to_document(&mut document, &patch),
            Err(AgentPatchApplyError::DocumentChanged)
        );
        assert_eq!(document.text(), "select 1");
    }
}
