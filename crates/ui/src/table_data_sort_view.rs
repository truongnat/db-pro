//! Sort controls for the table-data surface.

use super::*;

pub(super) struct TableDataSortContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) table_name: &'a str,
    pub(super) column_names: &'a [String],
    pub(super) data_query: &'a mut TableDataQueryState,
    pub(super) data: &'a mut TableDataState,
    pub(super) has_staged_changes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TableDataSortAction {
    ReloadFromStart,
    BlockedByStagedChanges,
}

pub(super) fn draw_sort(context: &mut TableDataSortContext<'_>, ui: &mut egui::Ui) -> Option<TableDataSortAction> {
    let sort_active = !context.data_query.sorts.is_empty() || context.data.grid_sort_column.is_some();
    let sort_label = sort_label(context);
    let mut action = None;
    egui::ComboBox::from_id_salt(("table-unified-sort-col", context.table_name))
        .selected_text(RichText::new(&sort_label).size(11.5).color(if sort_active {
            context.theme.accent
        } else {
            context.theme.text_secondary
        }))
        .width(100.0)
        .show_ui(ui, |ui| {
            if ui.selectable_label(!sort_active, "Default (None)").clicked() {
                action = Some(if context.has_staged_changes {
                    TableDataSortAction::BlockedByStagedChanges
                } else {
                    context.data_query.sorts.clear();
                    context.data.grid_sort_column = None;
                    TableDataSortAction::ReloadFromStart
                });
            }
            for column in context.column_names {
                let is_selected =
                    context.data_query.sorts.first().map(|sort| sort.column.as_str()) == Some(column.as_str());
                if ui.selectable_label(is_selected, column.as_str()).clicked() {
                    if context.has_staged_changes {
                        action = Some(TableDataSortAction::BlockedByStagedChanges);
                    } else {
                        if is_selected {
                            if let Some(sort) = context.data_query.sorts.first_mut() {
                                sort.descending = !sort.descending;
                            }
                        } else {
                            context.data_query.sorts = vec![UiTableDataSort {
                                column: column.clone(),
                                descending: false,
                            }];
                        }
                        action = Some(TableDataSortAction::ReloadFromStart);
                    }
                }
            }
        });
    action
}

fn sort_label(context: &TableDataSortContext<'_>) -> String {
    if !context.data_query.sorts.is_empty() {
        let clauses = context
            .data_query
            .sorts
            .iter()
            .enumerate()
            .map(|(priority, sort)| {
                format!(
                    "{} {}{}",
                    sort.column,
                    if sort.descending { "↓" } else { "↑" },
                    priority + 1
                )
            })
            .collect::<Vec<_>>();
        format!("Sort: {}", clauses.join(", "))
    } else if let Some(index) = context.data.grid_sort_column {
        if let Some(column) = context.column_names.get(index) {
            format!("Sort: {column} {}", if context.data.grid_sort_desc { "↓" } else { "↑" })
        } else {
            "Sort".to_owned()
        }
    } else {
        "Sort".to_owned()
    }
}
