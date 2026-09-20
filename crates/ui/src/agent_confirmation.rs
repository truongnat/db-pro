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
