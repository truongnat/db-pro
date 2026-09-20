use super::*;

/// Owns the local table-edit transaction and its asynchronous apply lifecycle.
#[derive(Debug)]
pub(crate) struct TableMutationState {
    pub(super) table_mutation_request: Option<crate::RequestId>,
    pub(super) staged_changes: ChangeSet,
    pub(super) pending_changes_open: bool,
    pub(super) staged_apply_request: Option<crate::RequestId>,
    pub(super) staged_apply_targets: Vec<MutationTarget>,
    pub(super) table_mutation_retry_after_reload: bool,
    pub(super) table_mutation_retry_target: Option<MutationTarget>,
    pub(super) table_mutation_error: Option<MutationFailure>,
    pub(super) conflict_dialog_open: bool,
}

impl Default for TableMutationState {
    fn default() -> Self {
        Self {
            table_mutation_request: None,
            staged_changes: ChangeSet::new(),
            pending_changes_open: false,
            staged_apply_request: None,
            staged_apply_targets: Vec::new(),
            table_mutation_retry_after_reload: false,
            table_mutation_retry_target: None,
            table_mutation_error: None,
            conflict_dialog_open: false,
        }
    }
}

impl TableMutationState {
    pub(crate) fn mutation_error_for_identity(&self, identity: &RowIdentity) -> bool {
        let Some(failure) = self.table_mutation_error.as_ref() else {
            return false;
        };
        matches!(
            failure.target.as_ref(),
            Some(MutationTarget::Update { identity: target, .. })
                | Some(MutationTarget::Delete { identity: target, .. }) if target == identity
        )
    }

    pub(crate) fn mutation_error_for_cell(&self, identity: &RowIdentity, column_index: usize) -> bool {
        let Some(failure) = self.table_mutation_error.as_ref() else {
            return false;
        };
        match failure.target.as_ref() {
            Some(MutationTarget::Update {
                identity: target,
                columns,
                ..
            }) => target == identity && columns.contains(&column_index),
            Some(MutationTarget::Delete { identity: target, .. }) => target == identity,
            Some(MutationTarget::Insert) | None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_table_mutation_state_is_idle_and_has_no_staged_changes() {
        let state = TableMutationState::default();

        assert!(state.staged_changes.is_empty());
        assert!(state.table_mutation_request.is_none());
        assert!(state.staged_apply_request.is_none());
        assert!(!state.pending_changes_open);
        assert!(!state.conflict_dialog_open);
    }
}
