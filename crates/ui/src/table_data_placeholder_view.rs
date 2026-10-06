//! Table data loading/error/empty placeholder rendering.
use super::*;
use lucide_icons::Icon;

pub(super) struct TableDataPlaceholderContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) error: Option<&'a str>,
    pub(super) empty: bool,
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
            let failed = context.error.is_some();
            let empty = context.empty;
            let icon = if failed {
                Icon::TriangleAlert
            } else if empty {
                Icon::Inbox
            } else {
                Icon::LoaderCircle
            };
            let color = if failed {
                context.theme.warning
            } else {
                context.theme.accent
            };
            ui.add_space(16.0);
            ui.label(icon_text(icon, "", color));
            ui.add_space(8.0);
            ui.label(
                RichText::new(if failed {
                    format!("Data for {table_name} could not be loaded")
                } else if empty {
                    format!("{table_name} has no rows")
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
            } else if empty {
                ui.label(
                    RichText::new("Add a row to start filling this table.")
                        .small()
                        .color(context.theme.text_secondary),
                );
            } else {
                ui.label(
                    RichText::new("Rows will appear here with the shared result-grid controls.")
                        .small()
                        .color(context.theme.text_secondary),
                );
            }
            ui.add_space(16.0);
        });
    });
    action
}
