//! Table data loading/error/empty placeholder rendering.
use super::*;
use lucide_icons::Icon;

pub(super) struct TableDataPlaceholderContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) error: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TableDataPlaceholderAction {
    Retry,
}

pub(super) fn draw_placeholder(
    context: &TableDataPlaceholderContext<'_>,
    ui: &mut egui::Ui,
    table_name: &str,
) -> Option<TableDataPlaceholderAction> {
    let mut action = None;
    grid_frame(context.theme).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(28.0);
            let failed = context.error.is_some();
            ui.label(icon_text(
                if failed {
                    Icon::TriangleAlert
                } else {
                    Icon::LoaderCircle
                },
                "",
                if failed {
                    context.theme.warning
                } else {
                    context.theme.accent
                },
            ));
            ui.add_space(8.0);
            ui.label(
                RichText::new(if failed {
                    format!("Data for {table_name} could not be loaded")
                } else {
                    format!("Loading data for {table_name}…")
                })
                .strong()
                .color(context.theme.text_primary),
            );
            if let Some(error) = context.error {
                ui.label(RichText::new(error).small().color(context.theme.text_secondary));
                ui.add_space(12.0);
                if secondary_button_with_icon(ui, Icon::RotateCcw, "Retry", context.theme).clicked() {
                    action = Some(TableDataPlaceholderAction::Retry);
                }
            } else {
                ui.label(
                    RichText::new("Rows will appear here with the shared result-grid controls.")
                        .small()
                        .color(context.theme.text_secondary),
                );
            }
            ui.add_space(28.0);
        });
    });
    action
}
