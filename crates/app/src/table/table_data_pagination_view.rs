//! Pagination controls for the table-data surface.

use super::table_data_toolbar_surface_view::TableDataPaging;
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
    if context.paging.inline_query_result {
        ui.label(
            RichText::new(&context.paging.page_range)
                .font(font_caption())
                .color(context.theme.text_secondary),
        );
        return None;
    }
    let mut action = None;
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        let show_navigation = should_show_navigation(context.paging);
        if show_navigation {
            if context.paging.total_known
                && context.paging.total_rows > 0
                && Button::new(context.theme)
                    .text(t!("table.last"))
                    .icon(Icon::ChevronsRight)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .enabled(context.paging.has_next && !context.has_staged_changes)
                    .tooltip(t!("table.last_page").into_owned())
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
                .tooltip(t!("table.next_page").into_owned())
                .access_label(t!("table.next_page").into_owned())
                .show(ui)
                .clicked()
                && !context.has_staged_changes
            {
                context.query.offset = context.query.offset.saturating_add(context.query.limit);
                action = Some(TableDataPaginationAction::RequestData);
            }
        }

        ui.label(
            RichText::new(&context.paging.page_range)
                .font(font_caption())
                .color(context.theme.text_secondary),
        );

        if show_navigation {
            if Button::new(context.theme)
                .icon(Icon::ChevronLeft)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .enabled(context.paging.has_previous && !context.has_staged_changes)
                .tooltip(t!("table.previous_page").into_owned())
                .access_label(t!("table.previous_page").into_owned())
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
                    .text(t!("table.first"))
                    .icon(Icon::ChevronsLeft)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .enabled(context.query.offset > 0 && !context.has_staged_changes)
                    .tooltip(t!("table.first_page").into_owned())
                    .show(ui)
                    .clicked()
                && context.query.offset > 0
                && !context.has_staged_changes
            {
                context.query.offset = 0;
                action = Some(TableDataPaginationAction::RequestData);
            }

            Separator::vertical(context.theme).show(ui);
        }

        let previous_limit = context.query.limit;
        const PAGE_LIMITS: [u64; 5] = [50, 100, 250, 500, 1000];
        let limit_labels: Vec<String> = PAGE_LIMITS
            .iter()
            .map(|limit| t!("table.page_limit", limit = limit).into_owned())
            .collect();
        let mut limit_selected = PAGE_LIMITS.iter().position(|limit| *limit == context.query.limit).unwrap_or(1);
        let limit_previous = limit_selected;
        // per-table salt keeps popup ids unique like the old combo's tuple salt
        crate::components::Select::new(
            format!("table-data-limit-select-{}", context.table_name),
            &mut limit_selected,
            &limit_labels,
        )
            .theme(context.theme)
            .width(104.0)
            .size(crate::components::SelectSize::Sm)
            .variant(crate::components::SelectVariant::Ghost)
            .show(ui);
        if limit_selected != limit_previous {
            context.query.limit = PAGE_LIMITS[limit_selected];
        }
        if context.query.limit != previous_limit {
            action = Some(TableDataPaginationAction::ResetPage);
        }
    });
    action
}

fn should_show_navigation(paging: &TableDataPaging) -> bool {
    paging.has_next || paging.has_previous
}

#[cfg(test)]
#[path = "tests/table_data_pagination_view_tests.rs"]
mod tests;

