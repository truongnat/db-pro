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
#[path = "tests/agent_patch_tests.rs"]
mod tests;

