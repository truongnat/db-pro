use super::*;
use crate::components::badge::{Badge, BadgeVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::table::{Table, TableColumn};
use egui::{Align, Layout, RichText};
use lucide_icons::Icon;

impl DbProApp {
    /// Draw the Structure tab: summary metrics, search bar, and detailed columns table.
    pub(super) fn draw_table_structure_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table.state.table_info.clone() else {
            self.draw_table_structure_placeholder(ui);
            return;
        };

        self.draw_table_structure_metrics(ui, &info);
        ui.add_space(8.0);
        self.draw_table_structure_columns_table(ui, &info);
        self.draw_table_column_detail(ui.ctx(), &info);
    }

    /// Top metric chips summarising columns, keys, and row count.
    fn draw_table_structure_metrics(&self, ui: &mut egui::Ui, info: &UiTableInfo) {
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                section_label(ui, "METRICS", self.theme);
                Badge::new(format!("{} columns", info.columns.len()), self.theme)
                    .variant(BadgeVariant::Default)
                    .show(ui);
                if let Some(pk) = &info.primary_key {
                    Badge::new(format!("PK: {}", pk.join(", ")), self.theme)
                        .variant(BadgeVariant::Warning)
                        .show(ui);
                }
                Badge::new(format!("{} indexes", info.indexes.len()), self.theme)
                    .variant(BadgeVariant::Secondary)
                    .show(ui);
                Badge::new(format!("{} foreign keys", info.foreign_keys.len()), self.theme)
                    .variant(BadgeVariant::Secondary)
                    .show(ui);
                if !info.check_constraints.is_empty() {
                    Badge::new(
                        format!("{} check constraints", info.check_constraints.len()),
                        self.theme,
                    )
                    .variant(BadgeVariant::Outline)
                    .show(ui);
                }
                if !info.dependencies.is_empty() {
                    Badge::new(format!("{} dependencies", info.dependencies.len()), self.theme)
                        .variant(BadgeVariant::Outline)
                        .show(ui);
                }
                if let Some(row_count) = info.row_count {
                    Badge::new(format!("{row_count} rows"), self.theme)
                        .variant(BadgeVariant::Secondary)
                        .show(ui);
                }
            });
        });
    }

    /// Columns table with search filter, type badges, nullability, PK/FK flags, and default expressions.
    fn draw_table_structure_columns_table(&mut self, ui: &mut egui::Ui, info: &UiTableInfo) {
        let mut selected_column = None;
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                section_label(ui, "COLUMNS", self.theme);
                ui.add_space(8.0);
                input(
                    ui,
                    &mut self.table.state.table_structure_search,
                    "Search columns or types…",
                    220.0,
                    self.theme,
                );
                if !self.table.state.table_structure_search.is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Clear filter")
                        .show(ui)
                        .clicked()
                {
                    self.table.state.table_structure_search.clear();
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("Total: {} columns", info.columns.len()))
                            .font(font_caption())
                            .color(self.theme.text_muted),
                    );
                });
            });
            ui.add_space(8.0);

            let filter_lower = self.table.state.table_structure_search.trim().to_lowercase();
            let matching_columns: Vec<_> = info
                .columns
                .iter()
                .filter(|c| {
                    if filter_lower.is_empty() {
                        true
                    } else {
                        c.name.to_lowercase().contains(&filter_lower)
                            || c.data_type.to_lowercase().contains(&filter_lower)
                            || c.default
                                .as_deref()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&filter_lower)
                    }
                })
                .collect();

            if matching_columns.is_empty() {
                empty_state(
                    ui,
                    Icon::Columns3,
                    "No columns match search",
                    "Try entering a different column name or data type.",
                    self.theme,
                );
                return;
            }

            let columns = [
                TableColumn::fixed("#", 46.0),
                TableColumn::new("Column Name").width(220.0),
                TableColumn::new("Data Type").width(180.0),
                TableColumn::fixed("Nullable", 110.0),
                TableColumn::fixed("Key", 90.0),
                TableColumn::new("Default Expression"),
            ];

            egui::ScrollArea::horizontal()
                .id_salt("structure-cols-scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    Table::new(&columns, self.theme).row_height(34.0).show(
                        ui,
                        matching_columns.len(),
                        |_| false,
                        |_| {},
                        |_| {},
                        |_| {},
                        |ui, row_idx, col_idx| {
                            let column = matching_columns[row_idx];
                            let is_fk = info
                                .foreign_keys
                                .iter()
                                .any(|fk| fk.from_columns.contains(&column.name));
                            match col_idx {
                                0 => {
                                    ui.label(
                                        RichText::new(column.ordinal.to_string())
                                            .font(font_caption())
                                            .color(self.theme.text_muted),
                                    );
                                }
                                1 => {
                                    let (icon, color) = if column.is_primary_key {
                                        (Icon::Key, self.theme.warning)
                                    } else if is_fk {
                                        (Icon::ArrowRightLeft, self.theme.accent)
                                    } else {
                                        (Icon::Columns3, self.theme.text_muted)
                                    };
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(char::from(icon).to_string())
                                                .font(egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())))
                                                .color(color),
                                        );
                                        ui.add_space(4.0);
                                        let name_response = ui
                                            .label(
                                                RichText::new(&column.name)
                                                    .font(font_ui_label())
                                                    .strong()
                                                    .color(self.theme.text_primary),
                                            )
                                            .on_hover_text(format!(
                                                "{}{}{}{}",
                                                if column.is_identity { "IDENTITY · " } else { "" },
                                                if column.is_generated { "GENERATED · " } else { "" },
                                                if column.is_unique { "UNIQUE · " } else { "" },
                                                column
                                                    .collation
                                                    .as_deref()
                                                    .map(|value| format!("COLLATION {value}"))
                                                    .unwrap_or_default()
                                            ));
                                        if name_response.clicked() {
                                            selected_column = Some(column.name.clone());
                                        }
                                    });
                                }
                                2 => {
                                    ui.label(
                                        RichText::new(&column.data_type)
                                            .monospace()
                                            .color(self.theme.text_secondary),
                                    )
                                    .on_hover_text("Full database type");
                                }
                                3 => {
                                    if column.nullable {
                                        Badge::new("NULL", self.theme)
                                            .variant(BadgeVariant::Secondary)
                                            .compact(true)
                                            .show(ui);
                                    } else {
                                        Badge::new("NOT NULL", self.theme)
                                            .variant(BadgeVariant::Outline)
                                            .compact(true)
                                            .show(ui);
                                    }
                                }
                                4 => {
                                    if column.is_primary_key {
                                        Badge::new("PK", self.theme)
                                            .variant(BadgeVariant::Warning)
                                            .compact(true)
                                            .show(ui);
                                    } else if is_fk {
                                        Badge::new("FK", self.theme)
                                            .variant(BadgeVariant::Default)
                                            .compact(true)
                                            .show(ui);
                                    } else if column.is_unique {
                                        Badge::new("UNIQUE", self.theme)
                                            .variant(BadgeVariant::Default)
                                            .compact(true)
                                            .show(ui);
                                    } else {
                                        ui.label(RichText::new("—").font(font_caption()).color(self.theme.text_muted));
                                    }
                                }
                                5 => {
                                    ui.label(
                                        RichText::new(column.default.as_deref().unwrap_or("—"))
                                            .font(font_caption())
                                            .color(self.theme.text_secondary),
                                    );
                                }
                                _ => {}
                            }
                        },
                    );
                });
        });
        if selected_column.is_some() {
            self.table.state.table_column_detail = selected_column;
        }
    }

    fn draw_table_column_detail(&mut self, ctx: &egui::Context, info: &UiTableInfo) {
        let Some(column_name) = self.table.state.table_column_detail.clone() else {
            return;
        };
        let Some(column) = info.columns.iter().find(|column| column.name == column_name) else {
            self.table.state.table_column_detail = None;
            return;
        };
        let mut open = true;
        egui::Window::new(format!("Column · {}", column.name))
            .open(&mut open)
            .resizable(false)
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.label(RichText::new(&column.data_type).monospace().strong());
                ui.separator();
                ui.label(format!("Ordinal: {}", column.ordinal));
                ui.label(format!("Nullable: {}", column.nullable));
                ui.label(format!("Default: {}", column.default.as_deref().unwrap_or("—")));
                ui.label(format!("Primary key: {}", column.is_primary_key));
                ui.label(format!("Unique: {}", column.is_unique));
                ui.label(format!("Identity: {}", column.is_identity));
                ui.label(format!("Generated: {}", column.is_generated));
                if let Some(collation) = &column.collation {
                    ui.label(format!("Collation: {collation}"));
                }
            });
        if !open {
            self.table.state.table_column_detail = None;
        }
    }
}
