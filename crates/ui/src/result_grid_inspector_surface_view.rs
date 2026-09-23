//! Advanced result-cell inspector presentation and typed effects.
use super::super::*;
use crate::components::{Button, ButtonSize, ButtonVariant, Dialog, SegmentedTabs};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ValueInspectorAction {
    CopyRaw,
    ExportBytes,
    Apply,
    Close,
}

pub(super) struct ValueInspectorContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) row_index: usize,
    pub(super) column_index: usize,
    pub(super) column_name: &'a str,
    pub(super) data_type: &'a str,
    pub(super) cell: UiCell,
    pub(super) kind: cell_inspector::CellInspectorKind,
    pub(super) write_block: Option<ColumnWriteBlock>,
    pub(super) writable: bool,
    pub(super) mode: &'a mut cell_inspector::CellInspectorMode,
    pub(super) value: &'a mut String,
    pub(super) error: &'a mut Option<String>,
}

pub(super) fn draw(
    context: &mut ValueInspectorContext<'_>,
    egui_context: &egui::Context,
) -> Option<ValueInspectorAction> {
    let mut action = None;
    let mut open = true;
    let title = format!("Value inspector · {}", context.column_name);
    Dialog::new(&mut open, title, context.theme)
        .width(620.0)
        .id_salt(("advanced-cell-inspector", context.row_index, context.column_index))
        .show_framed_ctx(egui_context, |frame| {
            frame.body(|ui| {
                draw_metadata(context, ui);
                draw_toolbar(context, ui, &mut action);
                ui.add_space(6.0);
                draw_value(context, ui);
                if let Some(error) = context.error.as_deref() {
                    ui.label(egui::RichText::new(error).small().color(context.theme.danger));
                }
            });
            frame.footer(|ui| draw_footer(context, ui, &mut action));
        });
    if !open {
        action = Some(ValueInspectorAction::Close);
    }
    action
}

fn draw_metadata(context: &ValueInspectorContext<'_>, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new(format!(
            "{} · {} · {}",
            context.data_type,
            if matches!(context.cell, UiCell::Null) {
                "NULL"
            } else {
                "value"
            },
            context
                .write_block
                .map(|block| block.reason())
                .unwrap_or(if context.writable {
                    "editable via ChangeSet"
                } else {
                    "view only"
                })
        ))
        .small()
        .color(context.theme.text_muted),
    );
}

fn draw_toolbar(context: &mut ValueInspectorContext<'_>, ui: &mut egui::Ui, action: &mut Option<ValueInspectorAction>) {
    let modes = modes_for_kind(context.kind);
    let mut selected_mode = modes.iter().position(|mode| *mode == *context.mode).unwrap_or_default();
    let mode_labels: Vec<&str> = modes.iter().map(|mode| mode.as_label()).collect();
    ui.horizontal(|ui| {
        SegmentedTabs::new(&mut selected_mode, &mode_labels, context.theme).show(ui);
        if let Some(mode) = modes.get(selected_mode) {
            *context.mode = *mode;
        }
        if Button::new(context.theme)
            .text("Copy raw")
            .size(ButtonSize::Sm)
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked()
        {
            *action = Some(ValueInspectorAction::CopyRaw);
        }
        if context.kind == cell_inspector::CellInspectorKind::Bytes
            && Button::new(context.theme)
                .text("Export bytes…")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
        {
            *action = Some(ValueInspectorAction::ExportBytes);
        }
    });
}

fn modes_for_kind(kind: cell_inspector::CellInspectorKind) -> &'static [cell_inspector::CellInspectorMode] {
    match kind {
        cell_inspector::CellInspectorKind::Json => &[
            cell_inspector::CellInspectorMode::Raw,
            cell_inspector::CellInspectorMode::Pretty,
            cell_inspector::CellInspectorMode::Tree,
        ],
        cell_inspector::CellInspectorKind::Bytes => &[
            cell_inspector::CellInspectorMode::Raw,
            cell_inspector::CellInspectorMode::Hex,
            cell_inspector::CellInspectorMode::Base64,
        ],
        _ => &[
            cell_inspector::CellInspectorMode::Raw,
            cell_inspector::CellInspectorMode::Pretty,
        ],
    }
}

fn draw_value(context: &mut ValueInspectorContext<'_>, ui: &mut egui::Ui) {
    match *context.mode {
        cell_inspector::CellInspectorMode::Raw => draw_raw(context, ui),
        cell_inspector::CellInspectorMode::Pretty => draw_pretty(context, ui),
        cell_inspector::CellInspectorMode::Tree => draw_tree(context, ui),
        cell_inspector::CellInspectorMode::Hex => draw_bytes(context, ui, true),
        cell_inspector::CellInspectorMode::Base64 => draw_bytes(context, ui, false),
    }
}

fn draw_raw(context: &mut ValueInspectorContext<'_>, ui: &mut egui::Ui) {
    let mut editor = egui::TextEdit::multiline(context.value)
        .desired_width(ui.available_width())
        .desired_rows(16);
    if !context.writable {
        editor = editor.interactive(false);
    }
    if ui.add(editor).changed() {
        *context.error = None;
    }
}

fn draw_pretty(context: &mut ValueInspectorContext<'_>, ui: &mut egui::Ui) {
    let mut pretty = cell_inspector::pretty_json(context.value).unwrap_or_else(|| context.value.clone());
    ui.add(
        egui::TextEdit::multiline(&mut pretty)
            .desired_width(ui.available_width())
            .desired_rows(16)
            .interactive(false),
    );
    if context.writable
        && Button::new(context.theme)
            .text("Use pretty as edit buffer")
            .size(ButtonSize::Sm)
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked()
    {
        *context.value = pretty;
        *context.mode = cell_inspector::CellInspectorMode::Raw;
    }
}

fn draw_tree(context: &ValueInspectorContext<'_>, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
        for line in cell_inspector::json_tree_lines(context.value, 400) {
            ui.label(egui::RichText::new(line).monospace().small());
        }
    });
}

fn draw_bytes(context: &ValueInspectorContext<'_>, ui: &mut egui::Ui, hex: bool) {
    let text = match cell_inspector::decode_bytes_payload(context.value) {
        Ok(bytes) => {
            ui.label(
                egui::RichText::new(cell_inspector::bytes_metadata(context.value))
                    .small()
                    .color(context.theme.text_secondary),
            );
            if hex {
                cell_inspector::encode_hex(&bytes)
            } else {
                cell_inspector::encode_base64(&bytes)
            }
        }
        Err(error) => format!("({} unavailable: {error})", if hex { "hex" } else { "base64" }),
    };
    let mut display = text;
    ui.add(
        egui::TextEdit::multiline(&mut display)
            .desired_width(ui.available_width())
            .desired_rows(if hex { 14 } else { 10 })
            .interactive(false),
    );
}

fn draw_footer(context: &ValueInspectorContext<'_>, ui: &mut egui::Ui, action: &mut Option<ValueInspectorAction>) {
    ui.horizontal(|ui| {
        if context.writable
            && Button::new(context.theme)
                .text("Apply to ChangeSet")
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
        {
            *action = Some(ValueInspectorAction::Apply);
        }
        if Button::new(context.theme)
            .text("Close")
            .size(ButtonSize::Sm)
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked()
        {
            *action = Some(ValueInspectorAction::Close);
        }
    });
}
