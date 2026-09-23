//! Full-record inspector presentation and typed row actions.
use super::super::*;
use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::UiColumn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RecordInspectorAction {
    Close,
    Inspect(usize),
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
