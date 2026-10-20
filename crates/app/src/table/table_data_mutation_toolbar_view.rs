// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
//! Mutation and table-status controls for the table-data toolbar.

use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use lucide_icons::Icon;

pub(super) struct TableDataMutationToolbarContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) can_mutate: bool,
    pub(super) connected: bool,
    pub(super) has_primary_key: bool,
    pub(super) read_only_notice: Option<&'a str>,
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
    let add_row = Button::new(context.theme)
        .text(t!("table.add_row"))
        .icon(Icon::Plus)
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Sm)
        .enabled(context.can_mutate)
        .tooltip(if context.can_mutate {
            t!("table.insert_new_row").into_owned()
        } else {
            t!("table.insert_unavailable").into_owned()
        })
        .show(ui);
    if add_row.clicked() {
        action = Some(TableDataMutationToolbarAction::AddRow);
    }
    if let Some(notice) = context.read_only_notice {
        ui.label(
            RichText::new(notice)
                .font(font_caption())
                .color(context.theme.warning),
        );
    }
    if context.can_mutate {
        if context.read_only_notice.is_none() && !context.has_primary_key {
            ui.label(
                RichText::new(t!("table.no_primary_key_edit").into_owned())
                    .font(font_caption())
                    .color(context.theme.warning),
            )
            .on_hover_text(t!("table.no_primary_key_hover").into_owned());
        }

        draw_pending_change_summary(context, ui, &mut action);
    } else if context.connected && context.read_only_notice.is_none() {
        Separator::vertical(context.theme).show(ui);
        ui.label(
            RichText::new(if context.has_primary_key {
                t!("table.read_only").into_owned()
            } else {
                t!("table.no_primary_key_edit").into_owned()
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
        crate::components::badge::Badge::new(
            t!("table.rows_selected", count = context.selected_rows).into_owned(),
            context.theme,
        )
            .variant(crate::components::badge::BadgeVariant::Secondary)
            .compact(true)
            .show(ui);
    }
    action
}

pub(super) fn draw_footer_controls(
    context: &TableDataMutationToolbarContext<'_>,
    ui: &mut egui::Ui,
) -> Option<TableDataMutationToolbarAction> {
    let mut action = None;
    let has_changes = !context.staged_changes.is_empty();
    let apply_enabled =
        context.can_mutate && has_changes && !context.staged_apply_pending && !context.has_data_edit_error;
    if Button::new(context.theme)
        .text(t!("common.save"))
        .icon(Icon::Check)
        .variant(ButtonVariant::Default)
        .size(ButtonSize::Sm)
        .enabled(apply_enabled)
        .tooltip(t!("table.save_staged_changes", key = format!("{}S", primary_modifier_label())).into_owned())
        .show(ui)
        .clicked()
    {
        action = Some(TableDataMutationToolbarAction::ApplyStagedChanges);
    }
    if Button::new(context.theme)
        .text(t!("table.discard_changes"))
        .icon(Icon::Undo2)
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Sm)
        .enabled(has_changes && !context.staged_apply_pending)
        .tooltip(t!("table.discard_staged_changes", key = format!("{}Z", primary_modifier_label())).into_owned())
        .show(ui)
        .clicked()
    {
        action = Some(if context.staged_changes.counts().total() > 1 {
            TableDataMutationToolbarAction::ConfirmDiscardChanges
        } else {
            TableDataMutationToolbarAction::DiscardStagedChanges
        });
    }
    if Button::new(context.theme)
        .text(t!("table.refresh_data"))
        .icon(Icon::RotateCcw)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .tooltip(t!("table.reload_table_data", key = "F5").into_owned())
        .show(ui)
        .clicked()
    {
        action = Some(if has_changes {
            TableDataMutationToolbarAction::RefreshBlocked
        } else {
            TableDataMutationToolbarAction::Refresh
        });
    }
    action
}

fn draw_pending_change_summary(
    context: &TableDataMutationToolbarContext<'_>,
    ui: &mut egui::Ui,
    action: &mut Option<TableDataMutationToolbarAction>,
) {
    if context.staged_changes.is_empty() {
        return;
    }
    Separator::vertical(context.theme).show(ui);
    let counts = context.staged_changes.counts();
    if Button::new(context.theme)
        .text(
            t!(
                "table.pending_changes_summary",
                total = counts.total(),
                inserts = counts.inserts,
                updates = counts.updates,
                deletes = counts.deletes
            )
            .into_owned(),
        )
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Sm)
        .tooltip(t!("table.open_pending_changes").into_owned())
        .show(ui)
        .clicked()
    {
        *action = Some(TableDataMutationToolbarAction::OpenPendingChanges);
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
    Separator::vertical(context.theme).show(ui);
    ui.label(
        RichText::new(format!(
            "{} · {}",
            failure.code,
            failure.target.as_ref().map_or_else(
                || t!("table.transaction_failed").into_owned(),
                |_| t!("table.mutation_failed", index = failure.statement_index.saturating_add(1)).into_owned(),
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
        .text(t!("table.reload_row"))
        .icon(Icon::RotateCcw)
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Sm)
        .tooltip(t!("table.reload_row_tooltip").into_owned())
        .show(ui)
        .clicked()
    {
        *action = Some(TableDataMutationToolbarAction::ReloadFailedMutation);
    }
    if Button::new(context.theme)
        .text(t!("table.discard_local_change"))
        .icon(Icon::Undo2)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .tooltip(t!("table.discard_local_change_tooltip").into_owned())
        .show(ui)
        .clicked()
    {
        *action = Some(TableDataMutationToolbarAction::DiscardFailedMutation);
    }
    if is_conflict {
        if Button::new(context.theme)
            .text(t!("table.resolve_conflict"))
            .icon(Icon::GitCompare)
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .tooltip(t!("table.resolve_conflict_tooltip").into_owned())
            .show(ui)
            .clicked()
        {
            *action = Some(TableDataMutationToolbarAction::ResolveConflict);
        }
        if Button::new(context.theme)
            .text(t!("table.retry"))
            .icon(Icon::RotateCcw)
            .variant(ButtonVariant::Default)
            .size(ButtonSize::Sm)
            .tooltip(t!("table.retry_tooltip").into_owned())
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
