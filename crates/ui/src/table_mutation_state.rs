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
    pub(crate) fn row_reload_filters(
        info: &UiTableInfo,
        identity: &RowIdentity,
    ) -> Result<Vec<UiTableDataFilter>, String> {
        identity
            .original_pk_columns
            .iter()
            .zip(&identity.original_pk_values)
            .map(|(column, value)| {
                let data_type = info
                    .columns
                    .iter()
                    .find(|candidate| candidate.name == *column)
                    .map(|candidate| candidate.data_type.clone())
                    .ok_or_else(|| format!("Primary-key metadata is missing for {column}"))?;
                Ok(UiTableDataFilter {
                    column: column.clone(),
                    data_type,
                    operator: UiTableFilterOperator::Equals,
                    value: crate::cell_text(value),
                })
            })
            .collect()
    }

    pub(crate) fn build_apply_plan(&mut self) -> TableApplyPlan {
        let retry_target = self.table_mutation_retry_target.take();
        let mut changes = Vec::new();
        let mut targets = Vec::new();
        let mut deletes = Vec::new();
        let mut inserts = Vec::new();
        let mut updates = Vec::<(
            RowIdentity,
            Option<usize>,
            Vec<String>,
            Vec<String>,
            Vec<UiCell>,
            Vec<usize>,
        )>::new();

        for change in self.staged_changes.iter() {
            if retry_target
                .as_ref()
                .is_some_and(|target| !change_matches_target(change, target))
            {
                continue;
            }
            match change {
                StagedChange::Update {
                    identity,
                    current_row_index,
                    column_index,
                    column,
                    data_type,
                    value,
                    ..
                } => {
                    if let Some(entry) = updates.iter_mut().find(|entry| entry.0 == *identity) {
                        entry.2.push(column.clone());
                        entry.3.push(data_type.clone());
                        entry.4.push(value.clone());
                        entry.5.push(*column_index);
                    } else {
                        updates.push((
                            identity.clone(),
                            *current_row_index,
                            vec![column.clone()],
                            vec![data_type.clone()],
                            vec![value.clone()],
                            vec![*column_index],
                        ));
                    }
                }
                StagedChange::Delete {
                    identity,
                    current_row_index,
                } => deletes.push((
                    UiTableMutation::Delete {
                        pk_columns: identity.original_pk_columns.clone(),
                        pk_values: identity.original_pk_values.clone(),
                    },
                    MutationTarget::Delete {
                        identity: identity.clone(),
                        current_row_index: *current_row_index,
                    },
                )),
                StagedChange::Insert { columns, values, .. } => inserts.push((
                    UiTableMutation::Insert {
                        columns: columns.clone(),
                        values: values.clone(),
                    },
                    MutationTarget::Insert,
                )),
            }
        }

        for (change, target) in deletes {
            changes.push(change);
            targets.push(target);
        }
        for (identity, current_row_index, columns, data_types, values, column_indexes) in updates {
            changes.push(UiTableMutation::Update {
                columns,
                data_types,
                values,
                pk_columns: identity.original_pk_columns.clone(),
                pk_values: identity.original_pk_values.clone(),
            });
            targets.push(MutationTarget::Update {
                identity,
                current_row_index,
                columns: column_indexes,
            });
        }
        for (change, target) in inserts {
            changes.push(change);
            targets.push(target);
        }

        TableApplyPlan { changes, targets }
    }

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

    pub(crate) fn clear_error_for_identity(&mut self, identity: &RowIdentity, column_index: Option<usize>) -> bool {
        let clear =
            self.table_mutation_error
                .as_ref()
                .is_some_and(|failure| match (failure.target.as_ref(), column_index) {
                    (
                        Some(MutationTarget::Update {
                            identity: target,
                            columns,
                            ..
                        }),
                        Some(column),
                    ) => target == identity && columns.contains(&column),
                    (Some(MutationTarget::Update { identity: target, .. }), None)
                    | (Some(MutationTarget::Delete { identity: target, .. }), None) => target == identity,
                    (Some(MutationTarget::Delete { identity: target, .. }), Some(_)) => target == identity,
                    _ => false,
                });
        if clear {
            self.table_mutation_error = None;
        }
        clear
    }
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct TableApplyPlan {
    pub(super) changes: Vec<UiTableMutation>,
    pub(super) targets: Vec<MutationTarget>,
}

fn change_matches_target(change: &StagedChange, target: &MutationTarget) -> bool {
    match (change, target) {
        (
            StagedChange::Update { identity, .. },
            MutationTarget::Update {
                identity: target_identity,
                ..
            },
        )
        | (
            StagedChange::Delete { identity, .. },
            MutationTarget::Delete {
                identity: target_identity,
                ..
            },
        ) => identity == target_identity,
        (StagedChange::Insert { .. }, MutationTarget::Insert) => true,
        _ => false,
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

    #[test]
    fn clearing_matching_mutation_error_is_scoped_to_the_target_cell_or_row() {
        let identity = RowIdentity {
            original_pk_columns: vec!["id".to_owned()],
            original_pk_values: vec![UiCell::Number("7".to_owned())],
        };
        let mut state = TableMutationState {
            table_mutation_error: Some(MutationFailure {
                statement_index: 0,
                target: Some(MutationTarget::Update {
                    identity: identity.clone(),
                    current_row_index: Some(2),
                    columns: vec![1],
                }),
                code: "23505".to_owned(),
                message: "duplicate".to_owned(),
                rolled_back: true,
            }),
            ..Default::default()
        };

        assert!(!state.clear_error_for_identity(&identity, Some(0)));
        assert!(state.table_mutation_error.is_some());
        assert!(state.clear_error_for_identity(&identity, Some(1)));
        assert!(state.table_mutation_error.is_none());
    }

    #[test]
    fn apply_plan_groups_updates_and_preserves_delete_update_insert_order() {
        let identity = RowIdentity {
            original_pk_columns: vec!["id".to_owned()],
            original_pk_values: vec![UiCell::Number("7".to_owned())],
        };
        let mut state = TableMutationState::default();
        state.staged_changes.stage_update(StagedChange::Update {
            identity: identity.clone(),
            current_row_index: Some(2),
            column_index: 1,
            column: "name".to_owned(),
            data_type: "text".to_owned(),
            original: UiCell::Text("old".to_owned()),
            value: UiCell::Text("new".to_owned()),
        });
        state.staged_changes.stage_delete(StagedChange::Delete {
            identity: RowIdentity {
                original_pk_columns: vec!["id".to_owned()],
                original_pk_values: vec![UiCell::Number("8".to_owned())],
            },
            current_row_index: Some(3),
        });
        state
            .staged_changes
            .stage_insert(vec!["name".to_owned()], vec![UiCell::Text("inserted".to_owned())]);

        let plan = state.build_apply_plan();

        assert_eq!(plan.changes.len(), 3);
        assert_eq!(plan.targets.len(), 3);
        assert!(matches!(plan.changes[0], UiTableMutation::Delete { .. }));
        assert!(matches!(plan.changes[1], UiTableMutation::Update { .. }));
        assert!(matches!(plan.changes[2], UiTableMutation::Insert { .. }));
    }

    #[test]
    fn row_reload_filters_are_built_from_primary_key_metadata() {
        let info = UiTableInfo {
            schema: "public".to_owned(),
            name: "customers".to_owned(),
            row_count: None,
            columns: vec![
                crate::UiTableColumn {
                    name: "tenant_id".to_owned(),
                    data_type: "uuid".to_owned(),
                    ..Default::default()
                },
                crate::UiTableColumn {
                    name: "id".to_owned(),
                    data_type: "integer".to_owned(),
                    ..Default::default()
                },
            ],
            primary_key: Some(vec!["tenant_id".to_owned(), "id".to_owned()]),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        };
        let identity = RowIdentity {
            original_pk_columns: vec!["tenant_id".to_owned(), "id".to_owned()],
            original_pk_values: vec![UiCell::Text("tenant-a".to_owned()), UiCell::Number("7".to_owned())],
        };

        let filters = TableMutationState::row_reload_filters(&info, &identity).expect("filters");

        assert_eq!(filters.len(), 2);
        assert_eq!(filters[0].column, "tenant_id");
        assert_eq!(filters[0].data_type, "uuid");
        assert_eq!(filters[1].column, "id");
        assert_eq!(filters[1].value, "7");
    }
}
