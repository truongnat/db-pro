use super::*;
use crate::components::alert::{Alert, AlertVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use egui::{Frame, Margin, Rounding, Stroke};
use lucide_icons::Icon;

impl DbProApp {
    pub(crate) fn open_duplicate_row(&mut self, result: &UiQueryResult, row_index: usize) {
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to insert rows".to_owned();
            return;
        }
        let Some(info) = self.table.state.table_info.clone() else {
            self.feedback.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        let Some(row) = result.rows.get(row_index) else {
            return;
        };
        self.table.data.insert_row_values = table_editor_values::duplicate_row_values(&info, row);
        self.table.data.insert_row_error.clear();
        self.table.data.insert_row_open = true;
    }

    pub(crate) fn open_insert_row(&mut self) {
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to insert rows".to_owned();
            return;
        }
        let Some(info) = self.table.state.table_info.clone() else {
            self.feedback.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        self.table.data.insert_row_values = vec![String::new(); info.columns.len()];
        self.table.data.insert_row_error.clear();
        self.table.data.insert_row_open = true;
    }

    pub(crate) fn submit_insert_row(&mut self) {
        let Some(table) = self.schema_explorer.selected_table.clone() else {
            self.table.data.insert_row_error = "Select a table before inserting a row".to_owned();
            return;
        };
        let Some(info) = self.table.state.table_info.clone() else {
            self.table.data.insert_row_error = "Table structure is still loading".to_owned();
            return;
        };
        let (columns, values) =
            match table_editor_values::parse_insert_row_values(&info, &self.table.data.insert_row_values) {
                Ok(parsed) => parsed,
                Err(error) => {
                    self.table.data.insert_row_error = error;
                    return;
                }
            };
        self.table.mutation.staged_changes.ensure_target(&table);
        self.table.mutation.staged_changes.stage_insert(columns, values);
        self.table.data.insert_row_open = false;
        self.table.data.insert_row_error.clear();
        self.feedback.runtime_message = format!("Row staged for {}", table);
    }

    pub(super) fn draw_insert_row_dialog(&mut self, ctx: &egui::Context) {
        let Some(info) = self.table.state.table_info.clone() else {
            self.table.data.insert_row_open = false;
            return;
        };
        if self.table.data.insert_row_values.len() != info.columns.len() {
            self.table.data.insert_row_values = vec![String::new(); info.columns.len()];
        }
        let mut open = self.table.data.insert_row_open;
        let mut submit = false;
        let mut cancel = false;
        let title = format!("Insert Row · {}", info.name);
        let description = format!("Schema: {} · {} columns", info.schema, info.columns.len());

        Dialog::new(&mut open, &title, self.theme)
            .description(&description)
            .width(620.0)
            .id_salt("insert_row_modal_dialog")
            .show_ctx(ctx, |ui| {
                // Quick batch generation bar
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
                            for val in &mut self.table.data.insert_row_values {
                                val.clear();
                            }
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
                            for (index, column) in info.columns.iter().enumerate() {
                                if !column.nullable
                                    && column.default.is_none()
                                    && self.table.data.insert_row_values[index].trim().is_empty()
                                {
                                    self.table.data.insert_row_values[index] =
                                        table_editor_values::generate_sample_value(&column.name, &column.data_type);
                                }
                            }
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
                            for (index, column) in info.columns.iter().enumerate() {
                                if self.table.data.insert_row_values[index].trim().is_empty() {
                                    self.table.data.insert_row_values[index] =
                                        table_editor_values::generate_sample_value(&column.name, &column.data_type);
                                }
                            }
                        }
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Columns list
                egui::ScrollArea::vertical()
                    .max_height(380.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (index, column) in info.columns.iter().enumerate() {
                            let is_fk = info
                                .foreign_keys
                                .iter()
                                .any(|fk| fk.from_columns.contains(&column.name));

                            Frame {
                                fill: self.theme.surface_panel,
                                stroke: Stroke::new(1.0, self.theme.border_subtle),
                                rounding: Rounding::same(8.0),
                                inner_margin: Margin::symmetric(12.0, 8.0),
                                ..Default::default()
                            }
                            .show(ui, |ui| {
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
                                    if !column.nullable && column.default.is_none() {
                                        ui.label(
                                            RichText::new("*required").font(font_caption()).color(self.theme.danger),
                                        );
                                    } else if column.nullable {
                                        ui.label(
                                            RichText::new("nullable")
                                                .font(font_caption())
                                                .color(self.theme.text_muted),
                                        );
                                    }
                                    if let Some(ref def) = column.default {
                                        ui.label(
                                            RichText::new(format!("default: {def}"))
                                                .font(font_caption())
                                                .color(self.theme.text_muted),
                                        );
                                    }

                                    // Per-field actions
                                    if ColumnWritePolicy::read(column).is_writable() {
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if !self.table.data.insert_row_values[index].is_empty() {
                                                let clear_btn = Button::new(self.theme)
                                                    .icon(Icon::X)
                                                    .size(ButtonSize::IconSm)
                                                    .variant(ButtonVariant::Ghost)
                                                    .show(ui);
                                                if clear_btn.on_hover_text("Clear this field").clicked() {
                                                    self.table.data.insert_row_values[index].clear();
                                                }
                                            }

                                            if column.nullable && self.table.data.insert_row_values[index] != "NULL" {
                                                let null_btn = Button::new(self.theme)
                                                    .text("NULL")
                                                    .size(ButtonSize::Sm)
                                                    .variant(ButtonVariant::Ghost)
                                                    .show(ui);
                                                if null_btn.on_hover_text("Set value to literal NULL").clicked() {
                                                    self.table.data.insert_row_values[index] = "NULL".to_owned();
                                                }
                                            }

                                            let lower_dt = column.data_type.to_ascii_lowercase();
                                            let gen_text = if lower_dt.contains("uuid") {
                                                "UUID"
                                            } else if lower_dt.contains("time") || lower_dt.contains("date") {
                                                "Now"
                                            } else {
                                                "Gen"
                                            };
                                            let gen_btn = Button::new(self.theme)
                                                .icon(Icon::Wand2)
                                                .text(gen_text)
                                                .size(ButtonSize::Sm)
                                                .variant(ButtonVariant::Secondary)
                                                .show(ui);
                                            let tooltip =
                                                format!("Generate sample {} for {}", column.data_type, column.name);
                                            if gen_btn.on_hover_text(tooltip).clicked() {
                                                self.table.data.insert_row_values[index] =
                                                    table_editor_values::generate_sample_value(
                                                        &column.name,
                                                        &column.data_type,
                                                    );
                                            }
                                        });
                                    }
                                });

                                ui.add_space(4.0);

                                let is_null_val = self.table.data.insert_row_values[index]
                                    .trim()
                                    .eq_ignore_ascii_case("null");
                                let write_block = ColumnWritePolicy::read(column).write_block();
                                if let Some(block) = write_block {
                                    ui.label(
                                        RichText::new(format!("Read-only — {}", block.reason()))
                                            .font(font_caption())
                                            .color(self.theme.text_muted),
                                    );
                                } else {
                                    let placeholder = if column.default.is_some() {
                                        "Leave empty for DEFAULT, or enter value / click Gen..."
                                    } else if column.nullable {
                                        "Enter value, click NULL, or click Gen..."
                                    } else {
                                        "Enter value or click Gen..."
                                    };

                                    let val_ref = &mut self.table.data.insert_row_values[index];
                                    let edit = egui::TextEdit::singleline(val_ref)
                                        .hint_text(
                                            RichText::new(placeholder)
                                                .font(font_caption())
                                                .color(self.theme.text_muted),
                                        )
                                        .text_color(if is_null_val {
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
                                            if is_null_val {
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
                            });
                            ui.add_space(6.0);
                        }
                    });

                if !self.table.data.insert_row_error.is_empty() {
                    ui.add_space(8.0);
                    Alert::new("Cannot Stage Insert", &self.table.data.insert_row_error, self.theme)
                        .variant(AlertVariant::Destructive)
                        .icon(Icon::AlertCircle)
                        .show(ui);
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .icon(Icon::Plus)
                        .text("Stage Insert")
                        .show(ui)
                        .clicked()
                    {
                        submit = true;
                    }

                    if Button::new(self.theme)
                        .text("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .show(ui)
                        .clicked()
                    {
                        cancel = true;
                    }
                });
            });

        if submit {
            self.submit_insert_row();
        }
        if cancel || !open {
            self.table.data.insert_row_open = false;
            self.table.data.insert_row_error.clear();
        }
    }
}
