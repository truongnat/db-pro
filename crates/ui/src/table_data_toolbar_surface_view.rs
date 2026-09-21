//! Unified table-data toolbar composition and typed intents.

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TableDataPaging {
    pub(super) page_range: String,
    pub(super) total_rows: u64,
    pub(super) total_known: bool,
    pub(super) has_next: bool,
    pub(super) has_previous: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TableDataToolbarAction {
    RequestData,
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
    CommitFilter,
    RemoveFilter(usize),
    ClearFilters,
    ReloadFromStart,
    SortBlocked,
    ResetPage,
}

pub(super) struct TableDataToolbarContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) table_name: &'a str,
    pub(super) result: &'a UiQueryResult,
    pub(super) column_names: &'a [String],
    pub(super) can_mutate: bool,
    pub(super) connected: bool,
    pub(super) has_primary_key: bool,
    pub(super) staged_changes: &'a ChangeSet,
    pub(super) staged_apply_pending: bool,
    pub(super) has_data_edit_error: bool,
    pub(super) failure: Option<&'a MutationFailure>,
    pub(super) selected_rows: usize,
    pub(super) data_query: &'a mut TableDataQueryState,
    pub(super) data: &'a mut TableDataState,
    pub(super) table_info: Option<&'a UiTableInfo>,
    pub(super) paging: &'a TableDataPaging,
}

pub(super) fn draw_toolbar(
    context: &mut TableDataToolbarContext<'_>,
    ui: &mut egui::Ui,
) -> Option<TableDataToolbarAction> {
    let mut action = None;
    toolbar_frame(context.theme).show(ui, |ui| {
        ui.horizontal(|ui| {
            action = context.draw_mutation_controls(ui);
            if action.is_some() {
                return;
            }
            if !context.column_names.is_empty() {
                ui.separator();
                action = context.draw_filter_controls(ui);
                if action.is_some() {
                    return;
                }
                action = context.draw_sort_controls(ui);
                if action.is_some() {
                    return;
                }
            }
            action = context.draw_pagination_controls(ui);
        });
    });
    action
}

impl TableDataToolbarContext<'_> {
    fn draw_mutation_controls(&mut self, ui: &mut egui::Ui) -> Option<TableDataToolbarAction> {
        let context = table_data_mutation_toolbar_view::TableDataMutationToolbarContext {
            theme: self.theme,
            can_mutate: self.can_mutate,
            connected: self.connected,
            has_primary_key: self.has_primary_key,
            staged_changes: self.staged_changes,
            staged_apply_pending: self.staged_apply_pending,
            has_data_edit_error: self.has_data_edit_error,
            failure: self.failure,
            selected_rows: self.selected_rows,
        };
        table_data_mutation_toolbar_view::draw_mutation_controls(&context, ui).map(Into::into)
    }

    fn draw_filter_controls(&mut self, ui: &mut egui::Ui) -> Option<TableDataToolbarAction> {
        let mut context = table_data_filter_view::TableDataFilterContext {
            theme: self.theme,
            table_name: self.table_name,
            result: self.result,
            column_names: self.column_names,
            data_query: self.data_query,
            data: self.data,
            table_info: self.table_info,
        };
        table_data_filter_view::draw_filter(&mut context, ui).map(Into::into)
    }

    fn draw_sort_controls(&mut self, ui: &mut egui::Ui) -> Option<TableDataToolbarAction> {
        let mut context = table_data_sort_view::TableDataSortContext {
            theme: self.theme,
            table_name: self.table_name,
            column_names: self.column_names,
            data_query: self.data_query,
            data: self.data,
            has_staged_changes: !self.staged_changes.is_empty(),
        };
        table_data_sort_view::draw_sort(&mut context, ui).map(Into::into)
    }

    fn draw_pagination_controls(&mut self, ui: &mut egui::Ui) -> Option<TableDataToolbarAction> {
        let mut context = table_data_pagination_view::TableDataPaginationContext {
            theme: self.theme,
            table_name: self.table_name,
            paging: self.paging,
            query: self.data_query,
            has_staged_changes: !self.staged_changes.is_empty(),
        };
        table_data_pagination_view::draw_pagination(&mut context, ui).map(Into::into)
    }
}

impl From<table_data_mutation_toolbar_view::TableDataMutationToolbarAction> for TableDataToolbarAction {
    fn from(action: table_data_mutation_toolbar_view::TableDataMutationToolbarAction) -> Self {
        use table_data_mutation_toolbar_view::TableDataMutationToolbarAction as Action;
        match action {
            Action::Refresh => Self::RequestData,
            Action::RefreshBlocked => Self::RefreshBlocked,
            Action::AddRow => Self::AddRow,
            Action::OpenPendingChanges => Self::OpenPendingChanges,
            Action::ApplyStagedChanges => Self::ApplyStagedChanges,
            Action::ConfirmDiscardChanges => Self::ConfirmDiscardChanges,
            Action::DiscardStagedChanges => Self::DiscardStagedChanges,
            Action::ReloadFailedMutation => Self::ReloadFailedMutation,
            Action::DiscardFailedMutation => Self::DiscardFailedMutation,
            Action::ResolveConflict => Self::ResolveConflict,
            Action::RetryFailedMutation => Self::RetryFailedMutation,
        }
    }
}

impl From<table_data_filter_view::TableDataFilterAction> for TableDataToolbarAction {
    fn from(action: table_data_filter_view::TableDataFilterAction) -> Self {
        match action {
            table_data_filter_view::TableDataFilterAction::CommitDraft => Self::CommitFilter,
            table_data_filter_view::TableDataFilterAction::ReloadData => Self::RequestData,
            table_data_filter_view::TableDataFilterAction::Remove(index) => Self::RemoveFilter(index),
            table_data_filter_view::TableDataFilterAction::ClearAll => Self::ClearFilters,
        }
    }
}

impl From<table_data_sort_view::TableDataSortAction> for TableDataToolbarAction {
    fn from(action: table_data_sort_view::TableDataSortAction) -> Self {
        match action {
            table_data_sort_view::TableDataSortAction::ReloadFromStart => Self::ReloadFromStart,
            table_data_sort_view::TableDataSortAction::BlockedByStagedChanges => Self::SortBlocked,
        }
    }
}

impl From<table_data_pagination_view::TableDataPaginationAction> for TableDataToolbarAction {
    fn from(action: table_data_pagination_view::TableDataPaginationAction) -> Self {
        match action {
            table_data_pagination_view::TableDataPaginationAction::RequestData => Self::RequestData,
            table_data_pagination_view::TableDataPaginationAction::ResetPage => Self::ResetPage,
        }
    }
}
