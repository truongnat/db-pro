//! Pagination controls for the table-data surface.

use super::table_data_view::TableDataPaging;
use super::*;

pub(super) struct TableDataPaginationContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) table_name: &'a str,
    pub(super) paging: &'a TableDataPaging,
    pub(super) query: &'a mut TableDataQueryState,
    pub(super) has_staged_changes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TableDataPaginationAction {
    RequestData,
    ResetPage,
}

pub(super) fn draw_pagination(
    context: &mut TableDataPaginationContext<'_>,
    ui: &mut egui::Ui,
) -> Option<TableDataPaginationAction> {
    let mut action = None;
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if context.paging.total_known
            && context.paging.total_rows > 0
            && Button::new(context.theme)
                .text("Last")
                .icon(Icon::ChevronsRight)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .enabled(context.paging.has_next && !context.has_staged_changes)
                .tooltip("Last page")
                .show(ui)
                .clicked()
            && context.paging.has_next
            && !context.has_staged_changes
        {
            let last_page = context.paging.total_rows.saturating_sub(1) / context.query.limit;
            context.query.offset = last_page.saturating_mul(context.query.limit);
            action = Some(TableDataPaginationAction::RequestData);
        }

        if Button::new(context.theme)
            .icon(Icon::ChevronRight)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .enabled(context.paging.has_next && !context.has_staged_changes)
            .tooltip("Next page")
            .show(ui)
            .clicked()
            && !context.has_staged_changes
        {
            context.query.offset = context.query.offset.saturating_add(context.query.limit);
            action = Some(TableDataPaginationAction::RequestData);
        }

        ui.label(
            RichText::new(&context.paging.page_range)
                .font(font_caption())
                .color(context.theme.text_secondary),
        );

        if Button::new(context.theme)
            .icon(Icon::ChevronLeft)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .enabled(context.paging.has_previous && !context.has_staged_changes)
            .tooltip("Previous page")
            .show(ui)
            .clicked()
            && !context.has_staged_changes
        {
            context.query.offset = context.query.offset.saturating_sub(context.query.limit);
            action = Some(TableDataPaginationAction::RequestData);
        }

        if context.paging.total_known
            && context.paging.total_rows > 0
            && Button::new(context.theme)
                .text("First")
                .icon(Icon::ChevronsLeft)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .enabled(context.query.offset > 0 && !context.has_staged_changes)
                .tooltip("First page")
                .show(ui)
                .clicked()
            && context.query.offset > 0
            && !context.has_staged_changes
        {
            context.query.offset = 0;
            action = Some(TableDataPaginationAction::RequestData);
        }

        ui.separator();

        let previous_limit = context.query.limit;
        let limit_label = format!("{} / page", context.query.limit);
        egui::ComboBox::from_id_salt(("table-data-limit-select", context.table_name))
            .selected_text(
                RichText::new(&limit_label)
                    .size(11.0)
                    .color(context.theme.text_secondary),
            )
            .width(90.0)
            .show_ui(ui, |ui| {
                for limit in [50, 100, 250, 500, 1000] {
                    ui.selectable_value(&mut context.query.limit, limit, format!("{limit} / page"));
                }
            });
        if context.query.limit != previous_limit {
            action = Some(TableDataPaginationAction::ResetPage);
        }
    });
    action
}
