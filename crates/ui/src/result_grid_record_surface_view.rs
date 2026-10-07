//! Full-record inspector presentation and typed row actions.
use super::super::*;
use crate::components::{Button, ButtonSize, ButtonVariant, Table, TableColumn};
use crate::UiColumn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RecordInspectorAction {
    Close,
    Inspect(usize),
    CopyLabel(usize),
    CopyValue(usize),
}

pub(super) fn draw_record(context: &RecordInspectorContext<'_>, ui: &mut egui::Ui) -> Option<RecordInspectorAction> {
    let mut action = None;
    let columns = [TableColumn::fixed("Field", 200.0), TableColumn::new("Value")];
    egui::ScrollArea::vertical().id_salt("single_record_fields").show(ui, |ui| {
        Table::new(&columns, context.theme).vertical_grid(true).show(
            ui,
            context.columns.len(),
            |_| false,
            |_| {},
            |_| {},
            |_| {},
            |ui, field_index, column_index| {
                let cell = context.row.get(field_index).unwrap_or(&UiCell::Null);
                let raw = if column_index == 0 {
                    context.columns[field_index].name.clone()
                } else {
                    cell_inspector::cell_raw_text(cell)
                };
                let display = if raw.is_empty() { "(empty)" } else { &raw };
                let rect = ui.max_rect();
                ui.add(
                    egui::Label::new(egui::RichText::new(display).color(
                        if column_index == 1 && matches!(cell, UiCell::Null) {
                            context.theme.text_muted
                        } else {
                            context.theme.text_primary
                        },
                    ))
                    .truncate()
                    .sense(egui::Sense::hover()),
                );
                let response = ui.interact(
                    rect,
                    ui.id().with(("copy_record_cell", field_index, column_index)),
                    egui::Sense::click(),
                );
                response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, display));
                if response
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text(format!("Click to copy: {display}"))
                    .clicked()
                {
                    action = Some(if column_index == 0 {
                        RecordInspectorAction::CopyLabel(field_index)
                    } else {
                        RecordInspectorAction::CopyValue(field_index)
                    });
                }
            },
        );
    });
    action
}

pub(super) struct RecordInspectorContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) row_index: usize,
    pub(super) columns: &'a [UiColumn],
    pub(super) row: &'a [UiCell],
}

struct RecordColumn<'a> {
    index: usize,
    column: &'a UiColumn,
}

pub(super) fn draw(context: &RecordInspectorContext<'_>, ui: &mut egui::Ui) -> Option<RecordInspectorAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("Record · row {}", context.row_index)).strong());
        if Button::new(context.theme)
            .text("Close")
            .size(ButtonSize::Sm)
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked()
        {
            action = Some(RecordInspectorAction::Close);
        }
    });
    egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
        for (column_index, column) in context.columns.iter().enumerate() {
            if draw_row(
                context,
                ui,
                RecordColumn {
                    index: column_index,
                    column,
                },
            ) {
                action = Some(RecordInspectorAction::Inspect(column_index));
            }
        }
    });
    action
}

fn draw_row(context: &RecordInspectorContext<'_>, ui: &mut egui::Ui, record_column: RecordColumn<'_>) -> bool {
    let cell = context.row.get(record_column.index).unwrap_or(&UiCell::Null);
    let mut inspect = false;
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{}:", record_column.column.name))
                .small()
                .strong()
                .color(context.theme.text_secondary),
        );
        let preview = cell_preview(cell);
        ui.label(egui::RichText::new(preview).small().monospace());
        if Button::new(context.theme)
            .text("Inspect")
            .size(ButtonSize::Sm)
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked()
        {
            inspect = true;
        }
    });
    inspect
}

fn cell_preview(cell: &UiCell) -> String {
    let text = cell_inspector::cell_raw_text(cell);
    if text.chars().count() > 80 {
        format!("{}…", text.chars().take(79).collect::<String>())
    } else {
        text
    }
}
