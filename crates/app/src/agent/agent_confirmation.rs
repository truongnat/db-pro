//! Document targeting and patch application for Agent confirmations.

use super::*;
use crate::query::QueryDocument;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AgentConfirmationError {
    PreviewUnavailable,
    DocumentUnavailable,
    DocumentChanged,
    InvalidRange,
}

#[derive(Debug)]
pub(super) struct PreparedAgentConfirmation {
    pub(super) current_document: Option<db_pro_core::domain::agent::AgentDocumentSnapshot>,
    pub(super) applied_patch: Option<db_pro_core::domain::agent::AgentToolOutput>,
    /// Undo data for the applied patch: the dispatch may still fail after the
    /// document mutated, and the buffer must then be restored (#409).
    pub(super) revert: Option<DocumentPatchRevert>,
}

/// Everything needed to undo an optimistically applied patch when the
/// `ContinueAgentRun` command never reaches the runtime worker (#409).
#[derive(Debug)]
pub(super) struct DocumentPatchRevert {
    pub(super) document_index: usize,
    pub(super) text: String,
    pub(super) dirty: bool,
    pub(super) cursor_offset: usize,
    pub(super) selection_anchor: usize,
    pub(super) selection_head: usize,
}

impl DocumentPatchRevert {
    /// Restore the document to its pre-apply state.
    pub(super) fn revert(self, documents: &mut [QueryDocument]) {
        let Some(document) = documents.get_mut(self.document_index) else {
            return;
        };
        document.set_text(self.text);
        document.dirty = self.dirty;
        document.cursor.set_offset(&document.buffer, self.cursor_offset);
        document.selection = crate::editor::selection::SelectionRange {
            anchor: self.selection_anchor,
            active: self.selection_head,
        };
    }
}

pub(super) fn target_document_index(documents: &[QueryDocument], fallback_index: usize, document_id: &str) -> usize {
    documents
        .iter()
        .position(|document| document.id == document_id)
        .unwrap_or(fallback_index)
}

pub(super) fn apply_approved_patch(
    documents: &mut [QueryDocument],
    target_index: usize,
    pending: &super::agent_workflow_state::AgentUiConfirmation,
) -> Result<(db_pro_core::domain::agent::AgentToolOutput, DocumentPatchRevert), AgentConfirmationError> {
    let Some(db_pro_core::domain::agent::AgentToolOutput::PatchPreview { patch, .. }) = pending.preview.as_ref() else {
        return Err(AgentConfirmationError::PreviewUnavailable);
    };
    let Some(document) = documents.get(target_index) else {
        return Err(AgentConfirmationError::DocumentUnavailable);
    };
    // Capture the pre-apply state before mutating so a failed `ContinueAgentRun`
    // dispatch can restore the exact buffer the user approved from (#409).
    let revert = DocumentPatchRevert {
        document_index: target_index,
        text: document.text().to_owned(),
        dirty: document.dirty,
        cursor_offset: document.cursor.offset,
        selection_anchor: document.selection.anchor,
        selection_head: document.selection.active,
    };
    let applied = agent_patch::apply_to_document(&mut documents[target_index], patch).map_err(|error| match error {
        agent_patch::AgentPatchApplyError::DocumentChanged => AgentConfirmationError::DocumentChanged,
        agent_patch::AgentPatchApplyError::InvalidRange => AgentConfirmationError::InvalidRange,
    })?;
    Ok((applied, revert))
}

pub(super) fn prepare_confirmation(
    documents: &mut [QueryDocument],
    fallback_index: usize,
    pending: &super::agent_workflow_state::AgentUiConfirmation,
    approved: bool,
) -> Result<PreparedAgentConfirmation, AgentConfirmationError> {
    let target_doc_index = target_document_index(documents, fallback_index, &pending.document_id);
    let current_document = documents
        .get(target_doc_index)
        .map(super::agent_context::document_snapshot);

    if !approved || pending.kind != db_pro_core::domain::agent_workflow::AgentConfirmationKind::ApplyPatch {
        return Ok(PreparedAgentConfirmation {
            current_document,
            applied_patch: None,
            revert: None,
        });
    }

    let (applied_patch, revert) = apply_approved_patch(documents, target_doc_index, pending)?;
    Ok(PreparedAgentConfirmation {
        current_document,
        applied_patch: Some(applied_patch),
        revert: Some(revert),
    })
}
