use super::UiCell;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum StagedChange {
    Update {
        row_index: usize,
        column_index: usize,
        column: String,
        data_type: String,
        original: UiCell,
        value: UiCell,
        pk_columns: Vec<String>,
        pk_values: Vec<UiCell>,
    },
    Delete {
        row_index: usize,
        pk_columns: Vec<String>,
        pk_values: Vec<UiCell>,
    },
    Insert {
        columns: Vec<String>,
        values: Vec<UiCell>,
    },
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
    entries: Vec<StagedChange>,
}

impl ChangeSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn push(&mut self, change: StagedChange) {
        self.entries.push(change);
    }

    pub fn iter(&self) -> std::slice::Iter<'_, StagedChange> {
        self.entries.iter()
    }

    /// Keep one final value per edited cell and drop a no-op edit.
    pub fn stage_update(&mut self, change: StagedChange) {
        let StagedChange::Update {
            row_index,
            column_index,
            value,
            original,
            ..
        } = &change
        else {
            self.entries.push(change);
            return;
        };

        if value == original {
            self.entries.retain(|entry| {
                !matches!(entry, StagedChange::Update { row_index: row, column_index: column, .. } if *row == *row_index && *column == *column_index)
            });
            return;
        }

        if let Some(StagedChange::Update { value: staged_value, .. }) = self.entries.iter_mut().find(|entry| {
            matches!(entry, StagedChange::Update { row_index: row, column_index: column, .. } if *row == *row_index && *column == *column_index)
        }) {
            *staged_value = value.clone();
        } else {
            self.entries.push(change);
        }
    }

    /// Deleting a server row supersedes all updates for that row.
    pub fn stage_delete(&mut self, change: StagedChange) {
        let StagedChange::Delete { row_index, .. } = &change else {
            self.entries.push(change);
            return;
        };
        self.entries.retain(|entry| {
            !matches!(entry, StagedChange::Update { row_index: row, .. } if row == row_index)
                && !matches!(entry, StagedChange::Delete { row_index: row, .. } if row == row_index)
        });
        self.entries.push(change);
    }

    pub fn revert_cell(&mut self, row_index: usize, column_index: usize) -> bool {
        let before = self.entries.len();
        self.entries.retain(|entry| {
            !matches!(entry, StagedChange::Update { row_index: row, column_index: column, .. } if *row == row_index && *column == column_index)
        });
        before != self.entries.len()
    }

    pub fn revert_row(&mut self, row_index: usize) -> bool {
        let before = self.entries.len();
        self.entries.retain(|entry| {
            !matches!(entry, StagedChange::Update { row_index: row, .. } if *row == row_index)
                && !matches!(entry, StagedChange::Delete { row_index: row, .. } if *row == row_index)
        });
        before != self.entries.len()
    }

    pub fn counts(&self) -> ChangeCounts {
        let mut counts = ChangeCounts::default();
        let mut updated_rows = std::collections::BTreeSet::new();
        for entry in &self.entries {
            match entry {
                StagedChange::Insert { .. } => counts.inserts += 1,
                StagedChange::Update { row_index, .. } => {
                    updated_rows.insert(*row_index);
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
        Self { entries }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update(value: &str) -> StagedChange {
        StagedChange::Update {
            row_index: 1,
            column_index: 2,
            column: "name".to_owned(),
            data_type: "TEXT".to_owned(),
            original: UiCell::Text("old".to_owned()),
            value: UiCell::Text(value.to_owned()),
            pk_columns: vec!["id".to_owned()],
            pk_values: vec![UiCell::Number("1".to_owned())],
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
    fn reverting_cell_removes_the_staged_update() {
        let mut changes = ChangeSet::new();
        changes.stage_update(update("one"));

        assert!(changes.revert_cell(1, 2));
        assert!(changes.is_empty());
    }

    #[test]
    fn delete_supersedes_all_updates_for_the_same_row() {
        let mut changes = ChangeSet::new();
        changes.stage_update(update("one"));
        changes.stage_update(StagedChange::Update {
            row_index: 1,
            column_index: 3,
            column: "active".to_owned(),
            data_type: "BOOLEAN".to_owned(),
            original: UiCell::Boolean(false),
            value: UiCell::Boolean(true),
            pk_columns: vec!["id".to_owned()],
            pk_values: vec![UiCell::Number("1".to_owned())],
        });

        changes.stage_delete(StagedChange::Delete {
            row_index: 1,
            pk_columns: vec!["id".to_owned()],
            pk_values: vec![UiCell::Number("1".to_owned())],
        });

        assert_eq!(changes.counts().updates, 0);
        assert_eq!(changes.counts().deletes, 1);
        assert!(matches!(changes.iter().next(), Some(StagedChange::Delete { .. })));
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
