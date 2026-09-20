use super::*;

impl DbProApp {
    pub(crate) fn draw_discard_changes_confirmation(&mut self, ui: &mut egui::Ui) {
        if !self.table.data.discard_changes_confirmation {
            return;
        }
        let counts = self.table.mutation.staged_changes.counts();
        let description = format!(
            "You have {} unapplied staged change(s) (+{} inserts, {} updates, {} deletes). Apply changes to database, discard them, or cancel navigation?",
            counts.total(),
            counts.inserts,
            counts.updates,
            counts.deletes
        );
        let mut open = true;
        let mut apply = false;
        let mut discard = false;
        let mut cancel = false;
        let theme = self.theme;
        Dialog::new(&mut open, "Unapplied Changes", theme)
            .description(&description)
            .id_salt("discard-table-changes")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if Button::new(theme)
                        .text("Apply Changes")
                        .variant(ButtonVariant::Default)
                        .show(ui)
                        .clicked()
                    {
                        apply = true;
                    }
                    if Button::new(theme)
                        .text("Discard Changes")
                        .variant(ButtonVariant::Destructive)
                        .show(ui)
                        .clicked()
                    {
                        discard = true;
                    }
                    if Button::new(theme)
                        .text("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .show(ui)
                        .clicked()
                    {
                        cancel = true;
                    }
                });
            });
        if apply {
            self.table.data.discard_changes_confirmation = false;
            self.apply_staged_changes();
        } else if discard {
            self.table.data.discard_changes_confirmation = false;
            let pending = self.workspace.pending_navigation_action.take();
            self.discard_staged_changes();
            if let Some(action) = pending {
                self.execute_pending_navigation(action);
            }
        } else if cancel || !open {
            self.table.data.discard_changes_confirmation = false;
            self.workspace.pending_navigation_action = None;
        }
    }

    pub(crate) fn draw_pending_changes_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.table.mutation.pending_changes_open {
            return;
        }
        let entries: Vec<StagedChange> = self.table.mutation.staged_changes.iter().cloned().collect();
        let mut groups: Vec<(Option<RowIdentity>, Option<u64>, Vec<StagedChange>)> = Vec::new();
        for entry in entries {
            let (identity, local_id) = match &entry {
                StagedChange::Update { identity, .. } | StagedChange::Delete { identity, .. } => {
                    (Some(identity.clone()), None)
                }
                StagedChange::Insert { local_id, .. } => (None, Some(*local_id)),
            };
            if let Some(group) = groups
                .iter_mut()
                .find(|(group_identity, group_local_id, _)| *group_identity == identity && *group_local_id == local_id)
            {
                group.2.push(entry);
            } else {
                groups.push((identity, local_id, vec![entry]));
            }
        }
        let mut open = true;
        enum PendingAction {
            Cell(RowIdentity, usize),
            Row(RowIdentity),
            Insert(u64),
        }
        let mut action = None;
        egui::Window::new("Pending changes")
            .open(&mut open)
            .resizable(true)
            .default_width(620.0)
            .show(ui.ctx(), |ui| {
                ui.label(
                    RichText::new("Review staged changes before Apply")
                        .small()
                        .color(self.theme.text_muted),
                );
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("{} row group(s)", groups.len()));
                    if ui.small_button("Discard All").clicked() {
                        self.table.data.discard_changes_confirmation = true;
                    }
                });
                egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                    for (identity, local_id, changes) in &groups {
                        let group_label = if let Some(identity) = identity {
                            format!(
                                "Row PK: [{}]",
                                identity
                                    .original_pk_values
                                    .iter()
                                    .map(crate::cell_text)
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        } else {
                            format!("New row · temporary identity #{}", local_id.unwrap_or_default())
                        };
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(group_label).strong());
                            if let Some(identity) = identity {
                                if ui.small_button("Revert Row").clicked() {
                                    action = Some(PendingAction::Row(identity.clone()));
                                }
                            } else if let Some(local_id) = local_id {
                                if ui.small_button("Remove Insert").clicked() {
                                    action = Some(PendingAction::Insert(*local_id));
                                }
                            }
                        });
                        for change in changes {
                            match change {
                                StagedChange::Update {
                                    identity,
                                    column_index,
                                    column,
                                    original,
                                    value,
                                    ..
                                } => {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(format!(
                                            "{column}: {} → {}",
                                            crate::cell_text(original),
                                            crate::cell_text(value)
                                        ));
                                        if ui.small_button("Revert Cell").clicked() {
                                            action = Some(PendingAction::Cell(identity.clone(), *column_index));
                                        }
                                    });
                                }
                                StagedChange::Delete { .. } => {
                                    ui.label("Delete row");
                                }
                                StagedChange::Insert { columns, .. } => {
                                    ui.label(format!("Insert ({})", columns.join(", ")));
                                }
                            }
                        }
                        ui.separator();
                    }
                });
            });
        if let Some(action) = action {
            match action {
                PendingAction::Cell(identity, column_index) => {
                    self.table.mutation.staged_changes.revert_cell(&identity, column_index);
                    self.table
                        .mutation
                        .clear_error_for_identity(&identity, Some(column_index));
                }
                PendingAction::Row(identity) => {
                    self.table.mutation.staged_changes.revert_row(&identity);
                    self.table.mutation.clear_error_for_identity(&identity, None);
                }
                PendingAction::Insert(local_id) => {
                    self.table.mutation.staged_changes.remove_insert(local_id);
                    self.table.mutation.table_mutation_error = None;
                }
            }
        }
        if !open {
            self.table.mutation.pending_changes_open = false;
        }
    }

    pub(crate) fn conflict_keep_mine(&mut self) {
        let Some(failure) = self.table.mutation.table_mutation_error.as_ref() else {
            return;
        };
        let Some(target) = failure.target.clone() else {
            return;
        };
        match &target {
            MutationTarget::Update { identity, .. } => {
                if let Some(result) = self.table.state.table_data_result.as_ref() {
                    let col_map: std::collections::HashMap<&str, usize> = result
                        .columns
                        .iter()
                        .enumerate()
                        .map(|(i, col)| (col.name.as_str(), i))
                        .collect();
                    if let Some(row_index) = result
                        .rows
                        .iter()
                        .position(|row| table_events::row_matches_identity(row, &col_map, identity))
                    {
                        let current_db_row = &result.rows[row_index];
                        for (col_idx, _col) in result.columns.iter().enumerate() {
                            if let Some(val) = current_db_row.get(col_idx) {
                                self.table.mutation.staged_changes.update_original_baseline(
                                    identity,
                                    col_idx,
                                    val.clone(),
                                );
                            }
                        }
                    }
                }
                self.table.mutation.table_mutation_retry_target = Some(target);
                self.table.mutation.table_mutation_error = None;
                self.table.mutation.conflict_dialog_open = false;
                self.apply_staged_changes();
            }
            MutationTarget::Delete { identity: _, .. } => {
                self.table.mutation.table_mutation_retry_target = Some(target);
                self.table.mutation.table_mutation_error = None;
                self.table.mutation.conflict_dialog_open = false;
                self.apply_staged_changes();
            }
            MutationTarget::Insert => {
                self.table.mutation.table_mutation_error = None;
                self.table.mutation.conflict_dialog_open = false;
                self.apply_staged_changes();
            }
        }
    }

    pub(crate) fn conflict_use_database(&mut self) {
        let Some(failure) = self.table.mutation.table_mutation_error.as_ref() else {
            return;
        };
        let Some(target) = failure.target.clone() else {
            return;
        };
        match &target {
            MutationTarget::Update { identity, .. } => {
                self.table.mutation.staged_changes.revert_row(identity);
                self.table.mutation.table_mutation_error = None;
                self.table.mutation.conflict_dialog_open = false;
                self.feedback
                    .show_info_toast("Reverted local changes; adopted database values");
            }
            MutationTarget::Delete { identity, .. } => {
                self.table.mutation.staged_changes.revert_row(identity);
                self.table.mutation.table_mutation_error = None;
                self.table.mutation.conflict_dialog_open = false;
                self.feedback.show_info_toast("Reverted staged delete");
            }
            MutationTarget::Insert => {
                self.table.mutation.table_mutation_error = None;
                self.table.mutation.conflict_dialog_open = false;
            }
        }
    }

    pub(crate) fn draw_conflict_dialog(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        if !self.table.mutation.conflict_dialog_open {
            return;
        }
        let Some(failure) = self.table.mutation.table_mutation_error.as_ref() else {
            self.table.mutation.conflict_dialog_open = false;
            return;
        };
        let Some(target) = failure.target.clone() else {
            self.table.mutation.conflict_dialog_open = false;
            return;
        };

        let mut open = true;
        let mut close_dialog = false;
        let mut keep_mine = false;
        let mut use_database = false;
        let mut retry = false;
        let mut discard = false;
        let theme = self.theme;

        egui::Window::new("Row Conflict Resolution")
            .open(&mut open)
            .resizable(true)
            .default_width(680.0)
            .show(ui.ctx(), |ui| {
                ui.label(
                    RichText::new("A concurrent modification or deletion was detected for this row.")
                        .small()
                        .color(theme.text_secondary),
                );
                ui.add_space(4.0);

                match &target {
                    MutationTarget::Update {
                        identity,
                        current_row_index,
                        columns: changed_col_indices,
                    } => {
                        let pk_str = identity
                            .original_pk_columns
                            .iter()
                            .zip(&identity.original_pk_values)
                            .map(|(c, v)| format!("{c}={}", crate::cell_text(v)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        ui.label(
                            RichText::new(format!("Primary Key: [{pk_str}]"))
                                .strong()
                                .color(theme.text_primary),
                        );
                        ui.add_space(8.0);

                        let current_row = self
                            .current_row_index_for_identity(identity, *current_row_index)
                            .and_then(|idx| result.rows.get(idx));

                        egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                            egui::Grid::new("conflict_diff_grid")
                                .num_columns(5)
                                .spacing([12.0, 6.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Column").strong().color(theme.text_muted));
                                    ui.label(RichText::new("Original Baseline").strong().color(theme.text_muted));
                                    ui.label(RichText::new("Local Staged (Mine)").strong().color(theme.text_muted));
                                    ui.label(RichText::new("Database Current").strong().color(theme.text_muted));
                                    ui.label(RichText::new("Status").strong().color(theme.text_muted));
                                    ui.end_row();

                                    for &col_idx in changed_col_indices {
                                        let col_name = result.columns.get(col_idx).map_or("?", |c| c.name.as_str());
                                        let local_val =
                                            self.table.mutation.staged_changes.cell_value(identity, col_idx);
                                        let staged_entry =
                                            self.table.mutation.staged_changes.iter().find(|e| match e {
                                                StagedChange::Update {
                                                    identity: id,
                                                    column_index: idx,
                                                    ..
                                                } => id == identity && *idx == col_idx,
                                                _ => false,
                                            });
                                        let orig_val = match staged_entry {
                                            Some(StagedChange::Update { original, .. }) => Some(original),
                                            _ => None,
                                        };
                                        let db_val = current_row.and_then(|r| r.get(col_idx));

                                        let orig_text = orig_val.map_or("—".to_owned(), crate::cell_text);
                                        let local_text = local_val.as_ref().map_or("—".to_owned(), crate::cell_text);
                                        let db_text = db_val.map_or("—".to_owned(), crate::cell_text);

                                        let is_conflict = local_text != db_text && orig_text != db_text;

                                        ui.label(RichText::new(col_name).strong().color(theme.text_primary));
                                        ui.label(RichText::new(orig_text).color(theme.text_secondary));
                                        ui.label(RichText::new(local_text).color(theme.accent).strong());
                                        ui.label(
                                            RichText::new(db_text)
                                                .color(if is_conflict { theme.warning } else { theme.text_primary })
                                                .strong(),
                                        );
                                        if is_conflict {
                                            ui.label(RichText::new("Conflict").color(theme.warning).strong());
                                        } else {
                                            ui.label(RichText::new("Match / Clean").color(theme.text_muted));
                                        }
                                        ui.end_row();
                                    }
                                });
                        });

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.horizontal_wrapped(|ui| {
                            if Button::new(theme)
                                .text("Keep Mine (Overwrite DB)")
                                .variant(ButtonVariant::Default)
                                .show(ui)
                                .on_hover_text(
                                    "Keep local staged changes, refresh the row baseline, and retry applying",
                                )
                                .clicked()
                            {
                                keep_mine = true;
                            }
                            if Button::new(theme)
                                .text("Use Database (Discard Mine)")
                                .variant(ButtonVariant::Secondary)
                                .show(ui)
                                .on_hover_text(
                                    "Discard local staged changes for this row and adopt current database values",
                                )
                                .clicked()
                            {
                                use_database = true;
                            }
                            if Button::new(theme)
                                .text("Reload & Retry")
                                .variant(ButtonVariant::Secondary)
                                .show(ui)
                                .on_hover_text("Reload the latest row from database, then retry the mutation")
                                .clicked()
                            {
                                retry = true;
                            }
                            if Button::new(theme)
                                .text("Discard Local Change")
                                .variant(ButtonVariant::Ghost)
                                .show(ui)
                                .clicked()
                            {
                                discard = true;
                            }
                        });
                    }
                    MutationTarget::Delete { identity, .. } => {
                        let pk_str = identity
                            .original_pk_columns
                            .iter()
                            .zip(&identity.original_pk_values)
                            .map(|(c, v)| format!("{c}={}", crate::cell_text(v)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        ui.label(
                            RichText::new(format!("Primary Key: [{pk_str}]"))
                                .strong()
                                .color(theme.text_primary),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(
                                "Could not delete row because it was modified or already removed in the database.",
                            )
                            .color(theme.warning),
                        );
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if Button::new(theme)
                                .text("Discard Staged Delete")
                                .variant(ButtonVariant::Default)
                                .show(ui)
                                .clicked()
                            {
                                discard = true;
                            }
                            if Button::new(theme)
                                .text("Reload & Retry Delete")
                                .variant(ButtonVariant::Secondary)
                                .show(ui)
                                .clicked()
                            {
                                retry = true;
                            }
                        });
                    }
                    MutationTarget::Insert => {
                        ui.label(RichText::new("Insert conflict or constraint failure.").color(theme.warning));
                        ui.add_space(12.0);
                        if Button::new(theme)
                            .text("Close")
                            .variant(ButtonVariant::Default)
                            .show(ui)
                            .clicked()
                        {
                            close_dialog = true;
                        }
                    }
                }
            });

        if !open || close_dialog {
            self.table.mutation.conflict_dialog_open = false;
        } else if keep_mine {
            self.conflict_keep_mine();
        } else if use_database {
            self.conflict_use_database();
        } else if retry {
            self.table.mutation.conflict_dialog_open = false;
            self.retry_failed_mutation_after_reload();
        } else if discard {
            self.table.mutation.conflict_dialog_open = false;
            self.discard_failed_mutation(false);
        }
    }
}
