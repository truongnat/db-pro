//! Concurrent table-mutation conflict presentation and typed intents.

use super::change_set::{ChangeSet, MutationTarget, StagedChange};
use super::*;
use crate::components::button::{Button, ButtonVariant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ConflictDialogAction {
    Close,
    KeepMine,
    UseDatabase,
    Retry,
    Discard,
}

pub(super) struct ConflictDialogContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) result: &'a UiQueryResult,
    pub(super) target: &'a MutationTarget,
    pub(super) staged_changes: &'a ChangeSet,
    pub(super) current_row: Option<&'a Vec<UiCell>>,
}

impl ConflictDialogContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Option<ConflictDialogAction> {
        let mut open = true;
        let mut action = None;
        egui::Window::new("Row Conflict Resolution")
            .open(&mut open)
            .resizable(true)
            .default_width(680.0)
            .show(ui.ctx(), |ui| {
                ui.label(
                    RichText::new("A concurrent modification or deletion was detected for this row.")
                        .small()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(4.0);
                match self.target {
                    MutationTarget::Update { identity, columns, .. } => {
                        self.draw_update_conflict(ui, identity, columns, &mut action)
                    }
                    MutationTarget::Delete { identity, .. } => {
                        self.draw_delete_conflict(ui, identity, &mut action);
                    }
                    MutationTarget::Insert => self.draw_insert_conflict(ui, &mut action),
                }
            });
        action.or_else(|| (!open).then_some(ConflictDialogAction::Close))
    }

    fn draw_update_conflict(
        &self,
        ui: &mut egui::Ui,
        identity: &super::change_set::RowIdentity,
        changed_columns: &[usize],
        action: &mut Option<ConflictDialogAction>,
    ) {
        self.draw_primary_key(ui, identity);
        ui.add_space(8.0);
        egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
            egui::Grid::new("conflict_diff_grid")
                .num_columns(5)
                .spacing([12.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    for (label, _) in [
                        ("Column", false),
                        ("Original Baseline", false),
                        ("Local Staged (Mine)", false),
                        ("Database Current", false),
                        ("Status", false),
                    ] {
                        ui.label(RichText::new(label).strong().color(self.theme.text_muted));
                    }
                    ui.end_row();
                    for &column_index in changed_columns {
                        self.draw_diff_row(ui, identity, column_index);
                    }
                });
        });
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if Button::new(self.theme)
                .text("Keep Mine (Overwrite DB)")
                .variant(ButtonVariant::Default)
                .show(ui)
                .on_hover_text("Keep local staged changes, refresh the row baseline, and retry applying")
                .clicked()
            {
                *action = Some(ConflictDialogAction::KeepMine);
            }
            if Button::new(self.theme)
                .text("Use Database (Discard Mine)")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .on_hover_text("Discard local staged changes for this row and adopt current database values")
                .clicked()
            {
                *action = Some(ConflictDialogAction::UseDatabase);
            }
            if Button::new(self.theme)
                .text("Reload & Retry")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .on_hover_text("Reload the latest row from database, then retry the mutation")
                .clicked()
            {
                *action = Some(ConflictDialogAction::Retry);
            }
            if Button::new(self.theme)
                .text("Discard Local Change")
                .variant(ButtonVariant::Ghost)
                .show(ui)
                .clicked()
            {
                *action = Some(ConflictDialogAction::Discard);
            }
        });
    }

    fn draw_diff_row(&self, ui: &mut egui::Ui, identity: &super::change_set::RowIdentity, column_index: usize) {
        let column_name = self
            .result
            .columns
            .get(column_index)
            .map_or("?", |column| column.name.as_str());
        let local_value = self.staged_changes.cell_value(identity, column_index);
        let original_value = self.staged_changes.iter().find_map(|change| match change {
            StagedChange::Update {
                identity: change_identity,
                column_index: change_column_index,
                original,
                ..
            } if change_identity == identity && *change_column_index == column_index => Some(original),
            _ => None,
        });
        let database_value = self.current_row.and_then(|row| row.get(column_index));
        let original_text = original_value.map_or("—".to_owned(), crate::cell_text);
        let local_text = local_value.as_ref().map_or("—".to_owned(), crate::cell_text);
        let database_text = database_value.map_or("—".to_owned(), crate::cell_text);
        let is_conflict = local_text != database_text && original_text != database_text;
        ui.label(RichText::new(column_name).strong().color(self.theme.text_primary));
        ui.label(RichText::new(original_text).color(self.theme.text_secondary));
        ui.label(RichText::new(local_text).color(self.theme.accent).strong());
        ui.label(
            RichText::new(database_text)
                .color(if is_conflict {
                    self.theme.warning
                } else {
                    self.theme.text_primary
                })
                .strong(),
        );
        ui.label(if is_conflict {
            RichText::new("Conflict").color(self.theme.warning).strong()
        } else {
            RichText::new("Match / Clean").color(self.theme.text_muted)
        });
        ui.end_row();
    }

    fn draw_delete_conflict(
        &self,
        ui: &mut egui::Ui,
        identity: &super::change_set::RowIdentity,
        action: &mut Option<ConflictDialogAction>,
    ) {
        self.draw_primary_key(ui, identity);
        ui.add_space(8.0);
        ui.label(
            RichText::new("Could not delete row because it was modified or already removed in the database.")
                .color(self.theme.warning),
        );
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("Discard Staged Delete")
                .variant(ButtonVariant::Default)
                .show(ui)
                .clicked()
            {
                *action = Some(ConflictDialogAction::Discard);
            }
            if Button::new(self.theme)
                .text("Reload & Retry Delete")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                *action = Some(ConflictDialogAction::Retry);
            }
        });
    }

    fn draw_insert_conflict(&self, ui: &mut egui::Ui, action: &mut Option<ConflictDialogAction>) {
        ui.label(RichText::new("Insert conflict or constraint failure.").color(self.theme.warning));
        ui.add_space(12.0);
        if Button::new(self.theme)
            .text("Close")
            .variant(ButtonVariant::Default)
            .show(ui)
            .clicked()
        {
            *action = Some(ConflictDialogAction::Close);
        }
    }

    fn draw_primary_key(&self, ui: &mut egui::Ui, identity: &super::change_set::RowIdentity) {
        let primary_key = identity
            .original_pk_columns
            .iter()
            .zip(&identity.original_pk_values)
            .map(|(column, value)| format!("{column}={}", crate::cell_text(value)))
            .collect::<Vec<_>>()
            .join(", ");
        ui.label(
            RichText::new(format!("Primary Key: [{primary_key}]"))
                .strong()
                .color(self.theme.text_primary),
        );
    }
}
