//! Search and server-filter controls for the table-data surface.

use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use egui::{FontFamily, FontId, Frame, Margin, Rounding, Stroke};
use lucide_icons::Icon;

pub(super) struct TableDataFilterContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) table_name: &'a str,
    pub(super) result: &'a UiQueryResult,
    pub(super) column_names: &'a [String],
    pub(super) data_query: &'a mut TableDataQueryState,
    pub(super) data: &'a mut TableDataState,
    pub(super) table_info: Option<&'a UiTableInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TableDataFilterAction {
    CommitDraft,
    ReloadData,
    Remove(usize),
    ClearAll,
}

pub(super) fn draw_filter(
    context: &mut TableDataFilterContext<'_>,
    ui: &mut egui::Ui,
) -> Option<TableDataFilterAction> {
    let mut action = draw_filter_editor(context, ui);
    if let Some(chip_action) = draw_filter_chips(context, ui) {
        action = Some(chip_action);
    }
    action
}

fn draw_filter_editor(context: &mut TableDataFilterContext<'_>, ui: &mut egui::Ui) -> Option<TableDataFilterAction> {
    let mut action = None;
    Frame {
        fill: context.theme.surface_elevated,
        stroke: Stroke::new(1.0, context.theme.border_subtle),
        rounding: Rounding::same(6.0),
        inner_margin: Margin::symmetric(8.0, 3.0),
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.horizontal(|ui| {
            draw_filter_scope(context, ui);
            ui.add(egui::Separator::default().vertical());
            let operator_changed = draw_filter_operator(context, ui);
            if let Some(filter_action) = draw_filter_input(context, ui, operator_changed) {
                action = Some(filter_action);
            }
        });
    });
    action
}

fn draw_filter_scope(context: &mut TableDataFilterContext<'_>, ui: &mut egui::Ui) {
    ui.label(
        RichText::new(char::from(Icon::Search).to_string())
            .font(FontId::new(12.0, FontFamily::Name("lucide".into())))
            .color(context.theme.text_muted),
    );
    let label = if context.data_query.filter_column.is_empty() {
        "All columns".to_owned()
    } else {
        context.data_query.filter_column.clone()
    };
    let color = if context.data_query.filter_column.is_empty() {
        context.theme.text_secondary
    } else {
        context.theme.accent
    };
    egui::ComboBox::from_id_salt(("table-unified-filter-col", context.table_name))
        .selected_text(RichText::new(label).size(12.0).color(color))
        .width(95.0)
        .show_ui(ui, |ui| {
            if ui
                .selectable_value(
                    &mut context.data_query.filter_column,
                    String::new(),
                    "All columns (Instant)",
                )
                .clicked()
            {
                context.data_query.filter_operator = UiTableFilterOperator::default();
                context.data.grid_filter = context.data_query.filter_value.clone();
            }
            for column in context.column_names {
                ui.selectable_value(&mut context.data_query.filter_column, column.clone(), column.as_str());
            }
        });
}

fn draw_filter_operator(context: &mut TableDataFilterContext<'_>, ui: &mut egui::Ui) -> bool {
    let data_type = context
        .table_info
        .and_then(|info| {
            info.columns
                .iter()
                .find(|column| column.name == context.data_query.filter_column)
        })
        .map(|column| column.data_type.clone())
        .or_else(|| {
            context
                .result
                .columns
                .iter()
                .find(|column| column.name == context.data_query.filter_column)
                .map(|column| column.data_type.clone())
        })
        .unwrap_or_else(|| "text".to_owned());
    let options = TableDataQueryState::filter_operator_options(&data_type);
    if !options
        .iter()
        .any(|(operator, _)| operator == &context.data_query.filter_operator)
    {
        context.data_query.filter_operator = options
            .first()
            .map(|(operator, _)| operator.clone())
            .unwrap_or_default();
    }
    let mut changed = false;
    egui::ComboBox::from_id_salt(("table-unified-filter-op", context.table_name))
        .selected_text(
            RichText::new(filter_operator_label(&context.data_query.filter_operator))
                .size(12.0)
                .color(context.theme.text_secondary),
        )
        .width(86.0)
        .show_ui(ui, |ui| {
            for (operator, label) in &options {
                if ui
                    .selectable_value(&mut context.data_query.filter_operator, operator.clone(), *label)
                    .clicked()
                {
                    changed = true;
                }
            }
        });
    changed
}

fn draw_filter_input(
    context: &mut TableDataFilterContext<'_>,
    ui: &mut egui::Ui,
    operator_changed: bool,
) -> Option<TableDataFilterAction> {
    let is_all_columns = context.data_query.filter_column.is_empty();
    let is_null_operator = matches!(
        context.data_query.filter_operator,
        UiTableFilterOperator::IsNull | UiTableFilterOperator::IsNotNull
    );
    let placeholder = if is_all_columns {
        "Search rows instantly…"
    } else if is_null_operator {
        "No value required"
    } else {
        "Filter value (Enter to query DB)…"
    };
    let edit_target = if is_all_columns {
        &mut context.data.grid_filter
    } else {
        &mut context.data_query.filter_value
    };
    let response = ui.add(
        egui::TextEdit::singleline(edit_target)
            .hint_text(RichText::new(placeholder).size(12.0).color(context.theme.text_muted))
            .font(FontId::proportional(12.0))
            .frame(false)
            .interactive(is_all_columns || !is_null_operator)
            .desired_width(210.0),
    );
    let should_commit = (response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter))
        || operator_changed)
        && !is_all_columns;
    if should_commit && is_null_operator {
        context.data_query.filter_value.clear();
    }
    let has_text = if is_all_columns {
        !context.data.grid_filter.is_empty()
    } else {
        !context.data_query.filter_value.is_empty()
    };
    if has_text && draw_clear_filter_button(context, ui) {
        if is_all_columns {
            context.data.grid_filter.clear();
        } else {
            let filter_column = context.data_query.filter_column.clone();
            context
                .data_query
                .filters
                .retain(|filter| filter.column != filter_column);
            context.data_query.filter_value.clear();
            context.data_query.filter_operator = UiTableFilterOperator::default();
            context.data_query.offset = 0;
            return Some(TableDataFilterAction::ReloadData);
        }
    }
    should_commit.then_some(TableDataFilterAction::CommitDraft)
}

fn draw_clear_filter_button(context: &TableDataFilterContext<'_>, ui: &mut egui::Ui) -> bool {
    Button::new(context.theme)
        .icon(Icon::X)
        .size(ButtonSize::IconSm)
        .variant(ButtonVariant::Ghost)
        .show(ui)
        .on_hover_text("Clear filter")
        .clicked()
}

fn draw_filter_chips(context: &mut TableDataFilterContext<'_>, ui: &mut egui::Ui) -> Option<TableDataFilterAction> {
    if context.data_query.filters.is_empty() {
        return None;
    }
    let mut remove_filter = None;
    let mut edit_filter = None;
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Filters:").small().color(context.theme.text_muted));
        for (index, filter) in context.data_query.filters.iter().enumerate() {
            let operator = filter_operator_chip_label(&filter.operator);
            let value = if filter.value.is_empty() {
                String::new()
            } else {
                format!(" {}", filter.value)
            };
            if ui
                .small_button(format!("{} {}{}", filter.column, operator, value))
                .on_hover_text("Edit filter")
                .clicked()
            {
                edit_filter = Some(index);
            }
            if ui.small_button("×").on_hover_text("Remove filter").clicked() {
                remove_filter = Some(index);
            }
        }
        if ui.small_button("Clear all").clicked() {
            remove_filter = Some(usize::MAX);
        }
    });
    if let Some(index) = edit_filter {
        if let Some(filter) = context.data_query.filters.get(index).cloned() {
            context.data_query.filter_column = filter.column;
            context.data_query.filter_operator = filter.operator;
            context.data_query.filter_value = filter.value;
            context.data_query.filter_editing = Some(index);
        }
    }
    remove_filter.map(|index| {
        if index == usize::MAX {
            TableDataFilterAction::ClearAll
        } else {
            TableDataFilterAction::Remove(index)
        }
    })
}

fn filter_operator_label(operator: &UiTableFilterOperator) -> &'static str {
    match operator {
        UiTableFilterOperator::Equals => "equals",
        UiTableFilterOperator::NotEquals => "not equals",
        UiTableFilterOperator::Contains => "contains",
        UiTableFilterOperator::StartsWith => "starts with",
        UiTableFilterOperator::EndsWith => "ends with",
        UiTableFilterOperator::GreaterThan => ">",
        UiTableFilterOperator::GreaterThanOrEqual => ">=",
        UiTableFilterOperator::LessThan => "<",
        UiTableFilterOperator::LessThanOrEqual => "<=",
        UiTableFilterOperator::IsNull => "IS NULL",
        UiTableFilterOperator::IsNotNull => "IS NOT NULL",
    }
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
