//! Result-pane shell presentation and typed user intents.
use super::*;
use egui::RichText;
use lucide_icons::Icon;

pub(super) struct QueryResultsSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) result_count: usize,
    pub(super) active_result_index: usize,
    pub(super) result_names: &'a std::collections::HashMap<usize, String>,
    pub(super) pinned_results: &'a std::collections::BTreeSet<usize>,
    pub(super) result: Option<&'a UiQueryResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum QueryResultsSurfaceAction {
    SelectResult(usize),
    TogglePin(usize),
    CloseResult(usize),
    CloseOtherResults(usize),
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
                let is_pinned = context.pinned_results.contains(&context.active_result_index);
                if Button::new(context.theme)
                    .icon(if is_pinned { Icon::PinOff } else { Icon::Pin })
                    .variant(if is_pinned { ButtonVariant::Secondary } else { ButtonVariant::Ghost })
                    .size(ButtonSize::IconSm)
                    .tooltip(if is_pinned { "Unpin result tab" } else { "Pin result tab" })
                    .show(ui)
                    .clicked()
                {
                    action = Some(QueryResultsSurfaceAction::TogglePin(context.active_result_index));
                }

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
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
        for index in 0..context.result_count {
            let is_active = context.active_result_index == index;
            let is_pinned = context.pinned_results.contains(&index);
            let default_name = format!("Result {}", index + 1);
            let name = context.result_names.get(&index).unwrap_or(&default_name);
            let label = if is_pinned {
                format!("📌 {name}")
            } else {
                name.clone()
            };

            let btn = Button::new(context.theme)
                .text(label)
                .variant(if is_active { ButtonVariant::Secondary } else { ButtonVariant::Ghost })
                .size(ButtonSize::Sm)
                .show(ui);

            if btn.clicked() {
                *action = Some(QueryResultsSurfaceAction::SelectResult(index));
            }

            btn.context_menu(|ui| {
                if ui.button(if is_pinned { "Unpin Tab" } else { "Pin Tab" }).clicked() {
                    *action = Some(QueryResultsSurfaceAction::TogglePin(index));
                    ui.close_menu();
                }
                if ui.button("Close Tab").clicked() {
                    *action = Some(QueryResultsSurfaceAction::CloseResult(index));
                    ui.close_menu();
                }
                if context.result_count > 1 && ui.button("Close Other Tabs").clicked() {
                    *action = Some(QueryResultsSurfaceAction::CloseOtherResults(index));
                    ui.close_menu();
                }
            });
        }
    });
}
