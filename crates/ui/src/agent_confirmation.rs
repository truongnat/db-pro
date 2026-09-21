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
) -> Result<db_pro_core::domain::agent::AgentToolOutput, AgentConfirmationError> {
    let Some(db_pro_core::domain::agent::AgentToolOutput::PatchPreview { patch, .. }) = pending.preview.as_ref() else {
        return Err(AgentConfirmationError::PreviewUnavailable);
    };
    let Some(document) = documents.get_mut(target_index) else {
        return Err(AgentConfirmationError::DocumentUnavailable);
    };
    agent_patch::apply_to_document(document, patch).map_err(|error| match error {
        agent_patch::AgentPatchApplyError::DocumentChanged => AgentConfirmationError::DocumentChanged,
        agent_patch::AgentPatchApplyError::InvalidRange => AgentConfirmationError::InvalidRange,
    })
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
        });
    }

    let applied_patch = apply_approved_patch(documents, target_doc_index, pending)?;
    Ok(PreparedAgentConfirmation {
        current_document,
        applied_patch: Some(applied_patch),
    })
}
