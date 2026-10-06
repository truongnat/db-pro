//! Active server-filter chips for the table-data surface.

use super::*;

pub(super) struct TableDataFilterContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) data_query: &'a mut TableDataQueryState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TableDataFilterAction {
    Remove(usize),
    ClearAll,
}

pub(super) fn draw_filter_chips(
    context: &mut TableDataFilterContext<'_>,
    ui: &mut egui::Ui,
) -> Option<TableDataFilterAction> {
    if context.data_query.inline_query_result || context.data_query.filters.is_empty() {
        return None;
    }
    let mut action = None;
    ui.add_space(SPACE_XS);
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Filters").small().color(context.theme.text_muted));
        for (index, filter) in context.data_query.filters.iter().enumerate() {
            let operator = filter_operator_chip_label(&filter.operator);
            let value = if filter.value.is_empty() {
                String::new()
            } else {
                format!(" {}", filter.value)
            };
            ui.label(format!("{} {}{}", filter.column, operator, value));
            if ui.small_button("×").on_hover_text("Remove filter").clicked() {
                action = Some(TableDataFilterAction::Remove(index));
            }
        }
        if ui.small_button("Clear all").clicked() {
            action = Some(TableDataFilterAction::ClearAll);
        }
    });
    action
}

fn filter_operator_chip_label(operator: &UiTableFilterOperator) -> &'static str {
    match operator {
        UiTableFilterOperator::Equals => "=",
        UiTableFilterOperator::NotEquals => "!=",
        UiTableFilterOperator::Contains => "contains",
        UiTableFilterOperator::StartsWith => "starts",
        UiTableFilterOperator::EndsWith => "ends",
        UiTableFilterOperator::GreaterThan => ">",
        UiTableFilterOperator::GreaterThanOrEqual => ">=",
        UiTableFilterOperator::LessThan => "<",
        UiTableFilterOperator::LessThanOrEqual => "<=",
        UiTableFilterOperator::IsNull => "IS NULL",
        UiTableFilterOperator::IsNotNull => "IS NOT NULL",
    }
}
