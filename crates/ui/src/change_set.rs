use super::UiCell;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RowIdentity {
    pub(crate) original_pk_columns: Vec<String>,
    pub(crate) original_pk_values: Vec<UiCell>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum StagedChange {
    Update {
        identity: RowIdentity,
        current_row_index: Option<usize>,
        column_index: usize,
        column: String,
        data_type: String,
        original: UiCell,
        value: UiCell,
    },
    Delete {
        identity: RowIdentity,
        current_row_index: Option<usize>,
    },
    Insert {
        local_id: u64,
        columns: Vec<String>,
        values: Vec<UiCell>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum MutationTarget {
    Update {
        identity: RowIdentity,
        current_row_index: Option<usize>,
        columns: Vec<usize>,
    },
    Delete {
        identity: RowIdentity,
        current_row_index: Option<usize>,
    },
    Insert,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MutationFailure {
    pub statement_index: usize,
    pub target: Option<MutationTarget>,
    pub code: String,
    pub message: String,
    pub rolled_back: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct ChangeCounts {
    pub inserts: usize,
    pub updates: usize,
    pub deletes: usize,
}

impl ChangeCounts {
    pub fn total(self) -> usize {
        self.inserts + self.updates + self.deletes
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct ChangeSet {
    target_table: Option<String>,
    entries: Vec<StagedChange>,
}

impl ChangeSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn target_table(&self) -> Option<&str> {
        self.target_table.as_deref()
    }

    // allow: no active caller currently; keeps public setter symmetric with the ChangeSet API
    // rather than deleting and re-adding when diff targeting is used.
    #[allow(dead_code)]
    pub fn set_target_table(&mut self, table: Option<String>) {
        self.target_table = table;
    }

    pub fn ensure_target(&mut self, table: &str) -> bool {
        if let Some(ref current) = self.target_table {
            if current != table {
                return false;
            }
        } else {
            self.target_table = Some(table.to_owned());
        }
        true
    }

    pub fn clear(&mut self) {
        self.target_table = None;
        self.entries.clear();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, StagedChange> {
        self.entries.iter()
    }

    /// Keep one final value per edited cell and drop a no-op edit.
    pub fn stage_update(&mut self, change: StagedChange) {
        let StagedChange::Update {
            column_index,
            value,
            original,
            identity,
            current_row_index,
            ..
        } = &change
        else {
            self.entries.push(change);
            return;
        };

        if let Some(existing) = self.entries.iter_mut().find(|entry| {
            matches!(entry, StagedChange::Update {
                column_index: column,
                identity: existing_identity,
                ..
            } if *column == *column_index && existing_identity == identity)
        }) {
            let existing_matches_original = match existing {
                StagedChange::Update {
                    original: existing_original,
                    ..
                } => value == existing_original,
                _ => false,
            };
            if existing_matches_original {
                self.entries.retain(|entry| {
                    !matches!(entry, StagedChange::Update {
                        column_index: column,
                        identity: existing_identity,
                        ..
                    } if *column == *column_index && existing_identity == identity)
                });
                return;
            }

            if let StagedChange::Update {
                value: staged_value,
                current_row_index: staged_row_index,
                ..
            } = existing
            {
                *staged_value = value.clone();
                *staged_row_index = *current_row_index;
            }
        } else {
            if value != original {
                self.entries.push(change);
            }
        }
    }

    /// Deleting a server row supersedes all updates for that stable PK identity.
    pub fn stage_delete(&mut self, change: StagedChange) {
        let StagedChange::Delete { identity, .. } = &change else {
            self.entries.push(change);
            return;
        };
        self.entries.retain(|entry| {
            !matches!(entry, StagedChange::Update {
                identity: existing_identity,
                ..
            } if existing_identity == identity)
                && !matches!(entry, StagedChange::Delete {
                    identity: existing_identity,
                    ..
                } if existing_identity == identity)
        });
        self.entries.push(change);
    }

    pub fn stage_insert(&mut self, columns: Vec<String>, values: Vec<UiCell>) -> u64 {
        let local_id = self
            .entries
            .iter()
            .filter_map(|entry| match entry {
                StagedChange::Insert { local_id, .. } => Some(*local_id),
                _ => None,
            })
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        self.entries.push(StagedChange::Insert {
            local_id,
            columns,
            values,
        });
        local_id
    }

    // Kept as a domain operation so a future local-row delete can cancel an
    // insert without emitting SQL; the current insert dialog has no row target.
    #[allow(dead_code)]
    pub fn remove_insert(&mut self, local_id: u64) -> bool {
        let before = self.entries.len();
        self.entries
            .retain(|entry| !matches!(entry, StagedChange::Insert { local_id: id, .. } if *id == local_id));
        before != self.entries.len()
    }

    pub fn revert_cell(&mut self, identity: &RowIdentity, column_index: usize) -> bool {
        let before = self.entries.len();
        self.entries.retain(|entry| {
            !matches!(entry, StagedChange::Update {
                identity: entry_identity,
                column_index: column,
                ..
            } if entry_identity == identity && *column == column_index)
        });
        before != self.entries.len()
    }

    pub fn revert_row(&mut self, identity: &RowIdentity) -> bool {
        let before = self.entries.len();
        self.entries.retain(|entry| {
            !matches!(entry, StagedChange::Update { identity: entry_identity, .. } if entry_identity == identity)
                && !matches!(entry, StagedChange::Delete { identity: entry_identity, .. } if entry_identity == identity)
        });
        before != self.entries.len()
    }

    pub fn update_original_baseline(&mut self, identity: &RowIdentity, column_index: usize, new_original: UiCell) {
        for entry in self.entries.iter_mut() {
            if let StagedChange::Update {
                identity: entry_identity,
                column_index: changed_column,
                original,
                ..
            } = entry
            {
                if entry_identity == identity && *changed_column == column_index {
                    *original = new_original.clone();
                }
            }
        }
    }

    pub fn cell_value(&self, identity: &RowIdentity, column_index: usize) -> Option<UiCell> {
        self.entries.iter().rev().find_map(|entry| match entry {
            StagedChange::Update {
                identity: entry_identity,
                column_index: changed_column,
                value,
                ..
            } if entry_identity == identity && *changed_column == column_index => Some(value.clone()),
            _ => None,
        })
    }

    pub fn row_deleted(&self, identity: &RowIdentity) -> bool {
        self.entries.iter().any(|entry| {
            matches!(entry, StagedChange::Delete { identity: entry_identity, .. } if entry_identity == identity)
        })
    }

    pub fn row_has_changes(&self, identity: &RowIdentity) -> bool {
        self.entries.iter().any(|entry| match entry {
            StagedChange::Update {
                identity: entry_identity,
                ..
            }
            | StagedChange::Delete {
                identity: entry_identity,
                ..
            } => entry_identity == identity,
            StagedChange::Insert { .. } => false,
        })
    }

    pub fn counts(&self) -> ChangeCounts {
        let mut counts = ChangeCounts::default();
        let mut updated_rows = Vec::<RowIdentity>::new();
        for entry in &self.entries {
            match entry {
                StagedChange::Insert { .. } => counts.inserts += 1,
                StagedChange::Update { identity, .. } => {
                    if !updated_rows.iter().any(|existing| existing == identity) {
                        updated_rows.push(identity.clone());
                    }
                }
                StagedChange::Delete { .. } => counts.deletes += 1,
            }
        }
        counts.updates = updated_rows.len();
        counts
    }
}

impl From<Vec<StagedChange>> for ChangeSet {
    fn from(entries: Vec<StagedChange>) -> Self {
        Self {
            target_table: None,
            entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update(value: &str) -> StagedChange {
        StagedChange::Update {
            identity: RowIdentity {
                original_pk_columns: vec!["id".to_owned()],
                original_pk_values: vec![UiCell::Number("1".to_owned())],
            },
            current_row_index: Some(1),
            column_index: 2,
            column: "name".to_owned(),
            data_type: "TEXT".to_owned(),
            original: UiCell::Text("old".to_owned()),
            value: UiCell::Text(value.to_owned()),
        }
    }

    #[test]
    fn repeated_cell_updates_keep_only_final_value() {
        let mut changes = ChangeSet::new();
        changes.stage_update(update("one"));
        changes.stage_update(update("two"));

        assert_eq!(changes.counts().updates, 1);
        assert!(
            matches!(changes.iter().next(), Some(StagedChange::Update { value: UiCell::Text(value), .. }) if value == "two")
        );
    }

    #[test]
    fn updates_with_same_original_identity_merge_across_row_coordinates() {
        let mut changes = ChangeSet::new();
        changes.stage_update(update("one"));
        changes.stage_update(StagedChange::Update {
            identity: RowIdentity {
                original_pk_columns: vec!["id".to_owned()],
                original_pk_values: vec![UiCell::Number("1".to_owned())],
            },
            current_row_index: Some(99),
            column_index: 3,
            column: "active".to_owned(),
            data_type: "BOOLEAN".to_owned(),
            original: UiCell::Boolean(false),
            value: UiCell::Boolean(true),
        });

        assert_eq!(changes.counts().updates, 1);
        assert_eq!(changes.iter().count(), 2);
    }

    #[test]
    fn composite_identity_keeps_rows_with_shared_pk_prefix_separate() {
        let mut changes = ChangeSet::new();
        let first = RowIdentity {
            original_pk_columns: vec!["tenant_id".to_owned(), "item_id".to_owned()],
            original_pk_values: vec![UiCell::Number("1".to_owned()), UiCell::Number("10".to_owned())],
        };
        let second = RowIdentity {
            original_pk_columns: first.original_pk_columns.clone(),
            original_pk_values: vec![UiCell::Number("1".to_owned()), UiCell::Number("11".to_owned())],
        };
        changes.stage_update(StagedChange::Update {
            identity: first.clone(),
            current_row_index: Some(4),
            column_index: 2,
            column: "name".to_owned(),
            data_type: "text".to_owned(),
            original: UiCell::Text("old-a".to_owned()),
            value: UiCell::Text("new-a".to_owned()),
        });
        changes.stage_update(StagedChange::Update {
            identity: second,
            current_row_index: Some(1),
            column_index: 2,
            column: "name".to_owned(),
            data_type: "text".to_owned(),
            original: UiCell::Text("old-b".to_owned()),
            value: UiCell::Text("new-b".to_owned()),
        });

        assert_eq!(changes.counts().updates, 2);
        assert_eq!(changes.cell_value(&first, 2), Some(UiCell::Text("new-a".to_owned())));
    }

    #[test]
    fn staged_value_follows_identity_when_visual_row_changes() {
        let mut changes = ChangeSet::new();
        let identity = RowIdentity {
            original_pk_columns: vec!["id".to_owned()],
            original_pk_values: vec![UiCell::Number("7".to_owned())],
        };
        changes.stage_update(StagedChange::Update {
            identity: identity.clone(),
            current_row_index: Some(9),
            column_index: 1,
            column: "name".to_owned(),
            data_type: "text".to_owned(),
            original: UiCell::Text("old".to_owned()),
            value: UiCell::Text("new".to_owned()),
        });

        assert_eq!(changes.cell_value(&identity, 1), Some(UiCell::Text("new".to_owned())));
        assert!(changes.row_has_changes(&identity));
    }

    #[test]
    fn reverting_cell_removes_the_staged_update() {
        let mut changes = ChangeSet::new();
        changes.stage_update(update("one"));
        let identity = RowIdentity {
            original_pk_columns: vec!["id".to_owned()],
            original_pk_values: vec![UiCell::Number("1".to_owned())],
        };

        assert!(changes.revert_cell(&identity, 2));
        assert!(changes.is_empty());
    }

    #[test]
    fn delete_supersedes_all_updates_for_the_same_row() {
        let mut changes = ChangeSet::new();
        changes.stage_update(update("one"));
        changes.stage_update(StagedChange::Update {
            identity: RowIdentity {
                original_pk_columns: vec!["id".to_owned()],
                original_pk_values: vec![UiCell::Number("1".to_owned())],
            },
            current_row_index: Some(1),
            column_index: 3,
            column: "active".to_owned(),
            data_type: "BOOLEAN".to_owned(),
            original: UiCell::Boolean(false),
            value: UiCell::Boolean(true),
        });

        changes.stage_delete(StagedChange::Delete {
            identity: RowIdentity {
                original_pk_columns: vec!["id".to_owned()],
                original_pk_values: vec![UiCell::Number("1".to_owned())],
            },
            current_row_index: Some(42),
        });

        assert_eq!(changes.counts().updates, 0);
        assert_eq!(changes.counts().deletes, 1);
        assert!(matches!(changes.iter().next(), Some(StagedChange::Delete { .. })));
    }

    #[test]
    fn insert_delete_removes_the_local_insert_before_apply() {
        let mut changes = ChangeSet::new();
        let local_id = changes.stage_insert(vec!["name".to_owned()], vec![UiCell::Text("draft".to_owned())]);

        assert!(changes.remove_insert(local_id));
        assert!(changes.is_empty());
    }

    #[test]
    fn staging_original_value_reverts_noop_without_a_pending_change() {
        let mut changes = ChangeSet::new();
        changes.stage_update(update("one"));
        changes.stage_update(update("old"));

        assert!(changes.is_empty());
        assert_eq!(changes.counts().total(), 0);
    }
}
