//! Result-grid toolbar rendering and user intent collection.
use super::*;
use egui::{Align, Layout, RichText};
use lucide_icons::Icon;

pub(super) struct ResultGridToolbarContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) data: &'a mut TableDataState,
    pub(super) editing: &'a mut TableEditingState,
    pub(super) feedback: &'a FeedbackState,
    pub(super) editable: bool,
    pub(super) matching_rows: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ResultGridToolbarAction {
    CopySelectedCell,
    CopySelectedRow,
    CopyVisibleCsv,
    CopyVisibleJson,
    InspectSelectedCell { row_index: usize, column_index: usize },
}

pub(super) fn draw_toolbar(
    context: &mut ResultGridToolbarContext<'_>,
    ui: &mut egui::Ui,
) -> Option<ResultGridToolbarAction> {
    let mut action = None;
    let modifier = primary_modifier_label();
    toolbar_frame(context.theme).show(ui, |ui| {
        ui.horizontal(|ui| {
            input(
                ui,
                &mut context.data.grid_filter,
                "Filter visible rows…",
                200.0,
                context.theme,
            );
            if !context.data.grid_filter.is_empty()
                && Button::new(context.theme)
                    .icon(Icon::X)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Clear filter")
                    .show(ui)
                    .clicked()
            {
                context.data.grid_filter.clear();
            }

            crate::components::badge::Badge::new(format!("{} rows", context.matching_rows), context.theme)
                .variant(crate::components::badge::BadgeVariant::Secondary)
                .compact(true)
                .show(ui);

            ui.separator();

            if Button::new(context.theme)
                .text("Copy Cell")
                .icon(Icon::Copy)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .tooltip(format!("Copy selected cell value ({modifier}C)"))
                .show(ui)
                .clicked()
            {
                action = Some(ResultGridToolbarAction::CopySelectedCell);
            }
            if Button::new(context.theme)
                .text("Copy Row")
                .icon(Icon::Table2)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .tooltip(format!(
                    "Copy entire selected row as tab-separated text ({modifier}Shift+C)"
                ))
                .show(ui)
                .clicked()
            {
                action = Some(ResultGridToolbarAction::CopySelectedRow);
            }
            if Button::new(context.theme)
                .text("CSV")
                .icon(Icon::FileSpreadsheet)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .tooltip("Copy visible rows as CSV")
                .show(ui)
                .clicked()
            {
                action = Some(ResultGridToolbarAction::CopyVisibleCsv);
            }
            if Button::new(context.theme)
                .text("JSON")
                .icon(Icon::Braces)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .tooltip("Copy visible rows as JSON array")
                .show(ui)
                .clicked()
            {
                action = Some(ResultGridToolbarAction::CopyVisibleJson);
            }
            if Button::new(context.theme)
                .text("Record")
                .icon(Icon::PanelRight)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .tooltip("Toggle record / value inspector panel")
                .show(ui)
                .clicked()
            {
                context.editing.record_inspector_open = !context.editing.record_inspector_open;
            }
            if let Some((row_index, column_index)) = context.data.selected_cell {
                if Button::new(context.theme)
                    .text("Inspect")
                    .icon(Icon::ScanSearch)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .tooltip("Open advanced value inspector for the selected cell")
                    .show(ui)
                    .clicked()
                {
                    action = Some(ResultGridToolbarAction::InspectSelectedCell {
                        row_index,
                        column_index,
                    });
                }
            }

            if !context.feedback.copy_status.is_empty() {
                crate::components::badge::Badge::new(&context.feedback.copy_status, context.theme)
                    .variant(crate::components::badge::BadgeVariant::Success)
                    .compact(true)
                    .show(ui);
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(if context.editable {
                        "Double-click / Enter to edit · Right-click for actions · Drag divider to resize"
                    } else {
                        "Click cell to select · Right-click for actions · Drag divider to resize"
                    })
                    .font(font_caption())
                    .color(context.theme.text_muted),
                );
            });
        });
    });
    ui.add_space(4.0);
    action
}

fn primary_modifier_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl"
    }
}
