//! Staged-table-change dialogs and typed user intents.

use super::change_set::ChangeCounts;
use super::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::dialog::Dialog;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DiscardChangesAction {
    Apply,
    Discard,
    Cancel,
}

pub(super) struct DiscardChangesContext {
    pub(super) theme: DbProTheme,
    pub(super) counts: ChangeCounts,
}

impl DiscardChangesContext {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Option<DiscardChangesAction> {
        let mut open = true;
        let mut action = None;
        let description = format!(
            "You have {} unapplied staged change(s) (+{} inserts, {} updates, {} deletes). Apply changes to database, discard them, or cancel navigation?",
            self.counts.total(),
            self.counts.inserts,
            self.counts.updates,
            self.counts.deletes
        );
        Dialog::new(&mut open, "Unapplied Changes", self.theme)
            .description(&description)
            .id_salt("discard-table-changes")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .text("Apply Changes")
                        .variant(ButtonVariant::Default)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(DiscardChangesAction::Apply);
                    }
                    if Button::new(self.theme)
                        .text("Discard Changes")
                        .variant(ButtonVariant::Destructive)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(DiscardChangesAction::Discard);
                    }
                    if Button::new(self.theme)
                        .text("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(DiscardChangesAction::Cancel);
                    }
                });
            });
        action.or_else(|| (!open).then_some(DiscardChangesAction::Cancel))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PendingChangesAction {
    RevertCell(RowIdentity, usize),
    RevertRow(RowIdentity),
    RemoveInsert(u64),
    RequestDiscardAll,
    Close,
}

pub(super) struct PendingChangesContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) changes: &'a [StagedChange],
}

impl PendingChangesContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<PendingChangesAction> {
        let mut actions = Vec::new();
        let groups = group_changes(self.changes);
        let mut open = true;
        Dialog::new(&mut open, "Pending changes", self.theme)
            .width(680.0)
            .id_salt("pending_table_changes_dialog")
            .show_framed_ctx(ui.ctx(), |frame| {
                frame.body(|ui| {
                    ui.label(
                        RichText::new("Review staged changes before Apply")
                            .small()
                            .color(self.theme.text_muted),
                    );
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(format!("{} row group(s)", groups.len()));
                        if ui.small_button("Discard All").clicked() {
                            actions.push(PendingChangesAction::RequestDiscardAll);
                        }
                    });
                    egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                        for group in &groups {
                            self.draw_group(ui, group, &mut actions);
                        }
                    });
                });
                frame.footer(|ui| {
                    if Button::new(self.theme)
                        .text("Close")
                        .variant(ButtonVariant::Ghost)
                        .show(ui)
                        .clicked()
                    {
                        actions.push(PendingChangesAction::Close);
                    }
                });
            });
        if !open {
            actions.push(PendingChangesAction::Close);
        }
        actions
    }

    fn draw_group(&self, ui: &mut egui::Ui, group: &ChangeGroup, actions: &mut Vec<PendingChangesAction>) {
        self.draw_group_header(ui, group, actions);
        self.draw_group_entries(ui, group, actions);
        ui.separator();
    }

    fn draw_group_header(&self, ui: &mut egui::Ui, group: &ChangeGroup, actions: &mut Vec<PendingChangesAction>) {
        let group_label = group.label();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(group_label).strong());
            if let Some(identity) = &group.identity {
                if ui.small_button("Revert Row").clicked() {
                    actions.push(PendingChangesAction::RevertRow(identity.clone()));
                }
            } else if let Some(local_id) = group.local_id {
                if ui.small_button("Remove Insert").clicked() {
                    actions.push(PendingChangesAction::RemoveInsert(local_id));
                }
            }
        });
    }

    fn draw_group_entries(&self, ui: &mut egui::Ui, group: &ChangeGroup, actions: &mut Vec<PendingChangesAction>) {
        for change in &group.changes {
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
                            actions.push(PendingChangesAction::RevertCell(identity.clone(), *column_index));
                        }
                    });
                }
                StagedChange::Delete { .. } => {
                    ui.label(RichText::new("Delete row").color(self.theme.warning));
                }
                StagedChange::Insert { columns, .. } => {
                    ui.label(format!("Insert ({})", columns.join(", ")));
                }
            }
        }
    }
}

struct ChangeGroup {
    identity: Option<RowIdentity>,
    local_id: Option<u64>,
    changes: Vec<StagedChange>,
}

impl ChangeGroup {
    fn label(&self) -> String {
        if let Some(identity) = &self.identity {
            return format!(
                "Row PK: [{}]",
                identity
                    .original_pk_values
                    .iter()
                    .map(crate::cell_text)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        match self.local_id {
            Some(local_id) => format!("New row · temporary identity #{local_id}"),
            None => "New row".to_owned(),
        }
    }
}

fn group_changes(changes: &[StagedChange]) -> Vec<ChangeGroup> {
    let mut groups = Vec::new();
    for entry in changes {
        let (identity, local_id) = match entry {
            StagedChange::Update { identity, .. } | StagedChange::Delete { identity, .. } => {
                (Some(identity.clone()), None)
            }
            StagedChange::Insert { local_id, .. } => (None, Some(*local_id)),
        };
        if let Some(group) = groups
            .iter_mut()
            .find(|group: &&mut ChangeGroup| group.identity == identity && group.local_id == local_id)
        {
            group.changes.push(entry.clone());
        } else {
            groups.push(ChangeGroup {
                identity,
                local_id,
                changes: vec![entry.clone()],
            });
        }
    }
    groups
}
