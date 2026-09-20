//! Query result-pane header and result selection intent.
use super::*;
use egui::RichText;

pub(super) struct QueryResultsPaneContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) session: &'a QuerySessionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum QueryResultsPaneAction {
    SelectResult(usize),
    OpenExport,
}

pub(super) fn draw_results_header(
    context: &QueryResultsPaneContext<'_>,
    ui: &mut egui::Ui,
    result: Option<&UiQueryResult>,
) -> Option<QueryResultsPaneAction> {
    let mut action = None;
    let result_count = context.session.active_result_count();
    if result_count > 1 {
        ui.horizontal(|ui| {
            let active_index = context
                .session
                .documents
                .get(context.session.active_document_index)
                .map_or(0, |document| document.active_result_index);
            for index in 0..result_count {
                if ui
                    .selectable_label(active_index == index, format!("Result {}", index + 1))
                    .clicked()
                {
                    action = Some(QueryResultsPaneAction::SelectResult(index));
                }
            }
        });
    }
    ui.horizontal(|ui| {
        if let Some(value) = result {
            ui.label(
                RichText::new(format!("{} rows · {} ms", value.row_count, value.duration_ms))
                    .small()
                    .color(context.theme.text_muted),
            );
            if compact_button(ui, "Export", context.theme).clicked() {
                action = Some(QueryResultsPaneAction::OpenExport);
            }
        }
    });
    action
}
