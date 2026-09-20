//! Mutation and table-status controls for the table-data toolbar.

use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use lucide_icons::Icon;

pub(super) struct TableDataMutationToolbarContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) can_mutate: bool,
    pub(super) connected: bool,
    pub(super) has_primary_key: bool,
    pub(super) staged_changes: &'a ChangeSet,
    pub(super) staged_apply_pending: bool,
    pub(super) has_data_edit_error: bool,
    pub(super) failure: Option<&'a MutationFailure>,
    pub(super) selected_rows: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TableDataMutationToolbarAction {
    Refresh,
    RefreshBlocked,
    AddRow,
    OpenPendingChanges,
    ApplyStagedChanges,
    ConfirmDiscardChanges,
    DiscardStagedChanges,
    ReloadFailedMutation,
    DiscardFailedMutation,
    ResolveConflict,
    RetryFailedMutation,
}

pub(super) fn draw_mutation_controls(
    context: &TableDataMutationToolbarContext<'_>,
    ui: &mut egui::Ui,
) -> Option<TableDataMutationToolbarAction> {
    let mut action = None;
    if compact_button_with_icon(ui, Icon::RotateCcw, "Refresh", context.theme)
        .on_hover_text("Reload table data (F5)")
        .clicked()
    {
        action = Some(if context.staged_changes.is_empty() {
            TableDataMutationToolbarAction::Refresh
        } else {
            TableDataMutationToolbarAction::RefreshBlocked
        });
    }

    if context.can_mutate {
        ui.separator();
        if Button::new(context.theme)
            .text("Add Row")
            .icon(Icon::Plus)
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .tooltip("Insert new row")
            .show(ui)
            .clicked()
        {
            action = Some(TableDataMutationToolbarAction::AddRow);
        }

        if !context.has_primary_key {
            ui.label(
                RichText::new("Table has no primary key; safe row editing is unavailable.")
                    .font(font_caption())
                    .color(context.theme.warning),
            )
            .on_hover_text("This table has no primary key; safe row editing is unavailable. Inserts remain available.");
        }

        draw_staged_change_controls(context, ui, &mut action);
    } else if context.connected {
        ui.separator();
        ui.label(
            RichText::new(if context.has_primary_key {
                "Read-only"
            } else {
                "Table has no primary key; safe row editing is unavailable."
            })
            .font(font_caption())
            .color(if context.has_primary_key {
                context.theme.warning
            } else {
                context.theme.danger
            }),
        );
    }

    draw_failure_controls(context, ui, &mut action);

    if context.selected_rows > 1 {
        crate::components::badge::Badge::new(format!("{} rows selected", context.selected_rows), context.theme)
            .variant(crate::components::badge::BadgeVariant::Secondary)
            .compact(true)
            .show(ui);
    }
    action
}

fn draw_staged_change_controls(
    context: &TableDataMutationToolbarContext<'_>,
    ui: &mut egui::Ui,
    action: &mut Option<TableDataMutationToolbarAction>,
) {
    if context.staged_changes.is_empty() {
        return;
    }
    ui.separator();
    let counts = context.staged_changes.counts();
    if ui
        .small_button(format!(
            "{} pending · +{} ~{} -{}",
            counts.total(),
            counts.inserts,
            counts.updates,
            counts.deletes
        ))
        .on_hover_text("Open pending changes")
        .clicked()
    {
        *action = Some(TableDataMutationToolbarAction::OpenPendingChanges);
    }
    let apply_enabled = !context.staged_apply_pending && !context.has_data_edit_error;
    if Button::new(context.theme)
        .text("Apply")
        .icon(Icon::Check)
        .variant(ButtonVariant::Default)
        .size(ButtonSize::Sm)
        .enabled(apply_enabled)
        .tooltip(format!("Apply all staged changes ({}S)", primary_modifier_label()))
        .show(ui)
        .clicked()
    {
        *action = Some(TableDataMutationToolbarAction::ApplyStagedChanges);
    }
    if Button::new(context.theme)
        .text("Discard")
        .icon(Icon::Undo2)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .tooltip(format!("Discard all staged changes ({}Z)", primary_modifier_label()))
        .show(ui)
        .clicked()
    {
        *action = Some(if counts.total() > 1 {
            TableDataMutationToolbarAction::ConfirmDiscardChanges
        } else {
            TableDataMutationToolbarAction::DiscardStagedChanges
        });
    }
}

fn draw_failure_controls(
    context: &TableDataMutationToolbarContext<'_>,
    ui: &mut egui::Ui,
    action: &mut Option<TableDataMutationToolbarAction>,
) {
    let Some(failure) = context.failure else {
        return;
    };
    let is_conflict = failure.code == "CONFLICT";
    ui.separator();
    ui.label(
        RichText::new(format!(
            "{} · {}",
            failure.code,
            failure.target.as_ref().map_or_else(
                || "Transaction failed".to_owned(),
                |_| format!("Mutation #{} failed", failure.statement_index.saturating_add(1)),
            )
        ))
        .font(font_caption())
        .color(context.theme.danger),
    )
    .on_hover_text(failure.message.as_str());
    if failure.target.is_none() {
        return;
    }
    if Button::new(context.theme)
        .text("Reload Row")
        .icon(Icon::RotateCcw)
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Sm)
        .tooltip("Reload database values while keeping the local staged mutation")
        .show(ui)
        .clicked()
    {
        *action = Some(TableDataMutationToolbarAction::ReloadFailedMutation);
    }
    if Button::new(context.theme)
        .text("Discard Local Change")
        .icon(Icon::Undo2)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .tooltip("Revert only the failed staged mutation")
        .show(ui)
        .clicked()
    {
        *action = Some(TableDataMutationToolbarAction::DiscardFailedMutation);
    }
    if is_conflict {
        if Button::new(context.theme)
            .text("Resolve Conflict")
            .icon(Icon::GitCompare)
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .tooltip("Open 3-way conflict resolution panel")
            .show(ui)
            .clicked()
        {
            *action = Some(TableDataMutationToolbarAction::ResolveConflict);
        }
        if Button::new(context.theme)
            .text("Retry")
            .icon(Icon::RotateCcw)
            .variant(ButtonVariant::Default)
            .size(ButtonSize::Sm)
            .tooltip("Reload the row, then retry the staged mutation")
            .show(ui)
            .clicked()
        {
            *action = Some(TableDataMutationToolbarAction::RetryFailedMutation);
        }
    }
}

fn primary_modifier_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl"
    }
}
