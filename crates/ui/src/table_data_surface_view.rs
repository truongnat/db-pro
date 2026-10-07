//! Table-data shell layout.
use super::*;
use crate::components::{Button, ButtonSize, ButtonVariant};

const PRESENTATION_RAIL_WIDTH: f32 = 40.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TableDataPresentation {
    Grid,
    Record,
}

pub(super) struct TableDataSurfaceContext {
    pub(super) theme: DbProTheme,
    pub(super) record_view_open: bool,
    pub(super) record_available: bool,
}

pub(super) fn draw_surface<F, R>(
    context: &TableDataSurfaceContext,
    ui: &mut egui::Ui,
    draw_content: F,
) -> (R, Option<TableDataPresentation>)
where
    F: FnOnce(&mut egui::Ui) -> R,
{
    ui.add_space(SPACE_XS);
    let data_width = ui.max_rect().width();
    const FOOTER_HEIGHT: f32 = 40.0;
    let available_height = (ui.max_rect().bottom() - ui.cursor().min.y - FOOTER_HEIGHT).max(0.0);
    let grid_height = available_height;
    let mut presentation = None;
    let content = ui
        .allocate_ui_with_layout(
            egui::vec2(data_width.max(0.0), grid_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                grid_frame(context.theme)
                    .show(ui, |ui| {
                        ui.set_min_width(data_width.max(0.0));
                        ui.set_min_height((grid_height - 2.0 * SPACE_XS).max(0.0));
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.horizontal_top(|ui| {
                            let height = ui.available_height();
                            ui.allocate_ui_with_layout(
                                egui::vec2(PRESENTATION_RAIL_WIDTH, height),
                                Layout::top_down(Align::Center),
                                |ui| {
                                    ui.set_max_width(PRESENTATION_RAIL_WIDTH);
                                    if rail_button(ui, context, Icon::Table, "Grid", !context.record_view_open, true) {
                                        presentation = Some(TableDataPresentation::Grid);
                                    }
                                    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                                        if rail_button(
                                            ui,
                                            context,
                                            Icon::PanelRight,
                                            "Record",
                                            context.record_view_open,
                                            context.record_available,
                                        ) {
                                            presentation = Some(TableDataPresentation::Record);
                                        }
                                    });
                                },
                            );
                            ui.separator();
                            ui.allocate_ui_with_layout(
                                egui::vec2(ui.available_width(), height),
                                Layout::top_down(Align::Min),
                                draw_content,
                            )
                            .inner
                        })
                        .inner
                    })
                    .inner
            },
        )
        .inner;
    (content, presentation)
}

fn rail_button(
    ui: &mut egui::Ui,
    context: &TableDataSurfaceContext,
    icon: Icon,
    label: &str,
    active: bool,
    enabled: bool,
) -> bool {
    let tooltip = if enabled {
        label
    } else {
        "Select exactly one row to view a record"
    };
    let response = Button::new(context.theme)
        .icon(icon)
        .access_label(label)
        .variant(if active {
            ButtonVariant::Secondary
        } else {
            ButtonVariant::Ghost
        })
        .size(ButtonSize::IconSm)
        .enabled(enabled)
        .tooltip(tooltip)
        .show(ui);
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, enabled, active, label));
    response.clicked()
}
