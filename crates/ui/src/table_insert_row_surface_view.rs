//! Insert-row dialog surface and typed user intents.

use super::*;
use crate::components::alert::{Alert, AlertVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use crate::UiTableColumn;
use egui::{Frame, Margin, Rounding, Stroke};
use lucide_icons::Icon;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InsertRowDialogAction {
    Submit,
    Cancel,
}

pub(super) struct InsertRowDialogContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) info: &'a UiTableInfo,
    pub(super) values: &'a mut Vec<String>,
    pub(super) error: &'a str,
}

pub(super) fn draw_dialog(
    context: &mut InsertRowDialogContext<'_>,
    egui_context: &egui::Context,
) -> Option<InsertRowDialogAction> {
    let mut open = true;
    let mut action = None;
    let title = format!("Insert Row · {}", context.info.name);
    let description = format!(
        "Schema: {} · {} columns",
        context.info.schema,
        context.info.columns.len()
    );

    Dialog::new(&mut open, &title, context.theme)
        .description(&description)
        .width(620.0)
        .id_salt("insert_row_modal_dialog")
        .show_ctx(egui_context, |ui| {
            context.draw_quick_actions(ui);
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            context.draw_columns(ui);
            context.draw_error(ui);
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(10.0);
            action = context.draw_footer(ui);
        });

    action.or_else(|| (!open).then_some(InsertRowDialogAction::Cancel))
}

impl InsertRowDialogContext<'_> {
    fn draw_quick_actions(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Fill values or generate sample mock data:")
                    .font(font_caption())
                    .color(self.theme.text_secondary),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if Button::new(self.theme)
                    .icon(Icon::RotateCcw)
                    .text("Clear All")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Ghost)
                    .show(ui)
                    .on_hover_text("Clear all field inputs")
                    .clicked()
                {
                    self.values.iter_mut().for_each(String::clear);
                }
                if Button::new(self.theme)
                    .icon(Icon::Sparkles)
                    .text("Fill Required")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Secondary)
                    .show(ui)
                    .on_hover_text("Auto-generate sample values for empty required fields")
                    .clicked()
                {
                    self.fill_required_values();
                }
                if Button::new(self.theme)
                    .icon(Icon::Wand2)
                    .text("Generate All")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Secondary)
                    .show(ui)
                    .on_hover_text("Auto-generate sample mock data for all empty fields")
                    .clicked()
                {
                    self.fill_all_values();
                }
            });
        });
    }

    fn fill_required_values(&mut self) {
        for (index, column) in self.info.columns.iter().enumerate() {
            if !column.nullable && column.default.is_none() && self.values[index].trim().is_empty() {
                self.values[index] = table_editor_values::generate_sample_value(&column.name, &column.data_type);
            }
        }
    }

    fn fill_all_values(&mut self) {
        for (index, column) in self.info.columns.iter().enumerate() {
            if self.values[index].trim().is_empty() {
                self.values[index] = table_editor_values::generate_sample_value(&column.name, &column.data_type);
            }
        }
    }

    fn draw_columns(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .max_height(380.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for index in 0..self.info.columns.len() {
                    self.draw_column(ui, index);
                    ui.add_space(6.0);
                }
            });
    }

    fn draw_column(&mut self, ui: &mut egui::Ui, index: usize) {
        let column_name = self.info.columns[index].name.clone();
        let is_fk = self
            .info
            .foreign_keys
            .iter()
            .any(|foreign_key| foreign_key.from_columns.contains(&column_name));
        Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            rounding: Rounding::same(8.0),
            inner_margin: Margin::symmetric(12.0, 8.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            self.draw_column_header(ui, index, is_fk);
            ui.add_space(4.0);
            self.draw_column_editor(ui, index);
        });
    }

    fn draw_column_header(&mut self, ui: &mut egui::Ui, index: usize, is_fk: bool) {
        let column = &self.info.columns[index];
        ui.horizontal(|ui| {
            if column.is_primary_key {
                badge(ui, "PK", self.theme.accent_soft, self.theme.warning);
            } else if is_fk {
                badge(ui, "FK", self.theme.surface_active, self.theme.accent);
            }
            ui.label(
                RichText::new(&column.name)
                    .font(font_ui_label())
                    .strong()
                    .color(self.theme.text_primary),
            );
            ui.label(
                RichText::new(&column.data_type)
                    .monospace()
                    .size(11.5)
                    .color(self.theme.text_secondary),
            );
            self.draw_column_requirement(ui, column);
            if let Some(default) = &column.default {
                ui.label(
                    RichText::new(format!("default: {default}"))
                        .font(font_caption())
                        .color(self.theme.text_muted),
                );
            }
            self.draw_column_actions(ui, index, column);
        });
    }

    fn draw_column_requirement(&self, ui: &mut egui::Ui, column: &UiTableColumn) {
        if !column.nullable && column.default.is_none() {
            ui.label(RichText::new("*required").font(font_caption()).color(self.theme.danger));
        } else if column.nullable {
            ui.label(
                RichText::new("nullable")
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );
        }
    }

    fn draw_column_actions(&mut self, ui: &mut egui::Ui, index: usize, column: &UiTableColumn) {
        if !ColumnWritePolicy::read(column).is_writable() {
            return;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !self.values[index].is_empty()
                && Button::new(self.theme)
                    .icon(Icon::X)
                    .size(ButtonSize::IconSm)
                    .variant(ButtonVariant::Ghost)
                    .show(ui)
                    .on_hover_text("Clear this field")
                    .clicked()
            {
                self.values[index].clear();
            }
            if column.nullable
                && self.values[index] != "NULL"
                && Button::new(self.theme)
                    .text("NULL")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Ghost)
                    .show(ui)
                    .on_hover_text("Set value to literal NULL")
                    .clicked()
            {
                self.values[index] = "NULL".to_owned();
            }
            let lower_type = column.data_type.to_ascii_lowercase();
            let label = if lower_type.contains("uuid") {
                "UUID"
            } else if lower_type.contains("time") || lower_type.contains("date") {
                "Now"
            } else {
                "Gen"
            };
            if Button::new(self.theme)
                .icon(Icon::Wand2)
                .text(label)
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .on_hover_text(format!("Generate sample {} for {}", column.data_type, column.name))
                .clicked()
            {
                self.values[index] = table_editor_values::generate_sample_value(&column.name, &column.data_type);
            }
        });
    }

    fn draw_column_editor(&mut self, ui: &mut egui::Ui, index: usize) {
        let column = &self.info.columns[index];
        let is_null = self.values[index].trim().eq_ignore_ascii_case("null");
        if let Some(block) = ColumnWritePolicy::read(column).write_block() {
            ui.label(
                RichText::new(format!("Read-only — {}", block.reason()))
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );
            return;
        }
        let placeholder = if column.default.is_some() {
            "Leave empty for DEFAULT, or enter value / click Gen..."
        } else if column.nullable {
            "Enter value, click NULL, or click Gen..."
        } else {
            "Enter value or click Gen..."
        };
        let value = &mut self.values[index];
        let edit = egui::TextEdit::singleline(value)
            .hint_text(
                RichText::new(placeholder)
                    .font(font_caption())
                    .color(self.theme.text_muted),
            )
            .text_color(if is_null {
                self.theme.warning
            } else {
                self.theme.text_primary
            })
            .font(font_ui_label())
            .margin(Margin::symmetric(8.0, 6.0))
            .desired_width(ui.available_width());
        Frame {
            fill: self.theme.surface_elevated,
            stroke: Stroke::new(
                1.0,
                if is_null {
                    self.theme.warning.linear_multiply(0.6)
                } else {
                    self.theme.border_subtle
                },
            ),
            rounding: Rounding::same(6.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.add(edit);
        });
    }

    fn draw_error(&self, ui: &mut egui::Ui) {
        if self.error.is_empty() {
            return;
        }
        ui.add_space(8.0);
        Alert::new("Cannot Stage Insert", self.error, self.theme)
            .variant(AlertVariant::Destructive)
            .icon(Icon::AlertCircle)
            .show(ui);
    }

    fn draw_footer(&self, ui: &mut egui::Ui) -> Option<InsertRowDialogAction> {
        let mut action = None;
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .icon(Icon::Plus)
                .text("Stage Insert")
                .show(ui)
                .clicked()
            {
                action = Some(InsertRowDialogAction::Submit);
            }
            if Button::new(self.theme)
                .text("Cancel")
                .variant(ButtonVariant::Ghost)
                .show(ui)
                .clicked()
            {
                action = Some(InsertRowDialogAction::Cancel);
            }
        });
        action
    }
}
