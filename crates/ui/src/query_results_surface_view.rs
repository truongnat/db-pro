//! Result-pane shell presentation and typed user intents.
use super::*;
use egui::RichText;
use lucide_icons::Icon;

pub(super) struct QueryResultsSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) result_count: usize,
    pub(super) active_result_index: usize,
    pub(super) result: Option<&'a UiQueryResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum QueryResultsSurfaceAction {
    SelectResult(usize),
    OpenExport,
}

pub(super) fn draw_results<F>(
    context: &QueryResultsSurfaceContext<'_>,
    ui: &mut egui::Ui,
    mut draw_grid: F,
) -> Option<QueryResultsSurfaceAction>
where
    F: FnMut(&mut egui::Ui, &UiQueryResult),
{
    let results_width = ui.max_rect().width();
    let mut action = None;
    grid_frame(context.theme).show(ui, |ui| {
        ui.set_min_width(results_width.max(0.0));
        draw_result_selector(context, ui, &mut action);
        if let Some(result) = context.result {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{} rows · {} ms", result.row_count, result.duration_ms))
                        .small()
                        .color(context.theme.text_muted),
                );
                if compact_button(ui, "Export", context.theme).clicked() {
                    action = Some(QueryResultsSurfaceAction::OpenExport);
                }
            });
            draw_grid(ui, result);
        } else {
            ui.centered_and_justified(|ui| {
                empty_state(
                    ui,
                    Icon::Table2,
                    "No results yet",
                    "Run a query to populate this result grid.",
                    context.theme,
                );
            });
        }
    });
    action
}

fn draw_result_selector(
    context: &QueryResultsSurfaceContext<'_>,
    ui: &mut egui::Ui,
    action: &mut Option<QueryResultsSurfaceAction>,
) {
    if context.result_count <= 1 {
        return;
    }
    ui.horizontal(|ui| {
        for index in 0..context.result_count {
            if ui
                .selectable_label(context.active_result_index == index, format!("Result {}", index + 1))
                .clicked()
            {
                *action = Some(QueryResultsSurfaceAction::SelectResult(index));
            }
        }
    });
}
