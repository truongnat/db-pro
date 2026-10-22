//! Insert-row dialog surface and typed user intents.

use super::*;
use crate::components::alert::{Alert, AlertVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::config::DIALOG_FORM_BODY_HEIGHT;
use crate::components::dialog::Dialog;
use crate::UiTableColumn;
use egui::{CornerRadius, Frame, Margin, Stroke};
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
    let title = t!("table.insert_row_title", table = context.info.name).into_owned();
    let description = t!(
        "table.insert_row_description",
        schema = context.info.schema,
        count = context.info.columns.len()
    )
    .into_owned();

    Dialog::new(&mut open, &title, context.theme)
        .description(&description)
        .width(620.0)
        .body_height(DIALOG_FORM_BODY_HEIGHT)
        .id_salt("insert_row_modal_dialog")
        .show_ctx(egui_context, |ui| {
            context.draw_quick_actions(ui);
            ui.add_space(8.0);
            Separator::horizontal(context.theme).show(ui);
            ui.add_space(8.0);
            context.draw_columns(ui);
            context.draw_error(ui);
            ui.add_space(12.0);
            Separator::horizontal(context.theme).show(ui);
            ui.add_space(10.0);
            action = context.draw_footer(ui);
        });

    action.or_else(|| (!open).then_some(InsertRowDialogAction::Cancel))
}

impl InsertRowDialogContext<'_> {
    fn draw_quick_actions(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(t!("table.insert_row_quick_actions"))
                    .font(font_caption())
                    .color(self.theme.text_secondary),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if Button::new(self.theme)
                    .icon(Icon::RotateCcw)
                    .text(t!("table.insert_row_clear_all"))
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Ghost)
                    .show(ui)
                    .on_hover_text(t!("table.insert_row_clear_all_tooltip"))
                    .clicked()
                {
                    self.values.iter_mut().for_each(String::clear);
                }
                if Button::new(self.theme)
                    .icon(Icon::Sparkles)
                    .text(t!("table.insert_row_fill_required"))
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Secondary)
                    .show(ui)
                    .on_hover_text(t!("table.insert_row_fill_required_tooltip"))
                    .clicked()
                {
                    self.fill_required_values();
                }
                if Button::new(self.theme)
                    .icon(Icon::Wand2)
                    .text(t!("table.insert_row_generate_all"))
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Secondary)
                    .show(ui)
                    .on_hover_text(t!("table.insert_row_generate_all_tooltip"))
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
        ScrollArea::new(self.theme)
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
            corner_radius: CornerRadius::same(8.0 as u8),
            inner_margin: Margin::symmetric(12.0 as i8, 8.0 as i8),
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
                Badge::new("PK", self.theme)
                    .tint(self.theme.warning)
                    .compact(true)
                    .show(ui);
            } else if is_fk {
                Badge::new("FK", self.theme)
                    .tint(self.theme.accent)
                    .compact(true)
                    .show(ui);
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
                    RichText::new(t!("table.insert_row_default", value = default))
                        .font(font_caption())
                        .color(self.theme.text_muted),
                );
            }
            self.draw_column_actions(ui, index, column);
        });
    }

    fn draw_column_requirement(&self, ui: &mut egui::Ui, column: &UiTableColumn) {
        if !column.nullable && column.default.is_none() {
            ui.label(
                RichText::new(t!("table.required"))
                    .font(font_caption())
                    .color(self.theme.danger),
            );
        } else if column.nullable {
            ui.label(
                RichText::new(t!("table.nullable"))
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );
        }
    }

    // cc-scan:allow COMPLEXITY — classifier/dispatch ladder — one case per branch
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
                    .access_label(t!("table.insert_row_clear_field"))
                    .show(ui)
                    .on_hover_text(t!("table.insert_row_clear_field"))
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
                    .on_hover_text(t!("table.insert_row_set_null"))
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
                .on_hover_text(t!(
                    "table.insert_row_generate_column",
                    data_type = column.data_type,
                    column = column.name
                ))
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
                RichText::new(t!("table.read_only_reason", reason = block.reason()))
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );
            return;
        }
        let placeholder = if column.default.is_some() {
            t!("table.insert_row_default_placeholder")
        } else if column.nullable {
            t!("table.insert_row_nullable_placeholder")
        } else {
            t!("table.insert_row_required_placeholder")
        };
        let value = &mut self.values[index];
        let mut edit = crate::components::input::Input::new(value, placeholder, self.theme);
        if is_null {
            edit = edit.error_text(t!("table.insert_row_null_literal"))
        }
        edit.show(ui);
    }

    fn draw_error(&self, ui: &mut egui::Ui) {
        if self.error.is_empty() {
            return;
        }
        ui.add_space(8.0);
        Alert::new(t!("table.insert_row_error_title"), self.error, self.theme)
            .variant(AlertVariant::Destructive)
            .icon(Icon::AlertCircle)
            .show(ui);
    }

    fn draw_footer(&self, ui: &mut egui::Ui) -> Option<InsertRowDialogAction> {
        let mut action = None;
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .icon(Icon::Plus)
                .text(t!("table.stage_insert"))
                .show(ui)
                .clicked()
            {
                action = Some(InsertRowDialogAction::Submit);
            }
            if Button::new(self.theme)
                .text(t!("common.cancel"))
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
