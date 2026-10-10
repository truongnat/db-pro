// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
//! Table-structure presentation and typed column-detail intents.
use super::super::*;
use crate::components::badge::{Badge, BadgeVariant};
use crate::components::table::{Table, TableColumn};
use crate::UiTableColumn;
use egui::RichText;
use lucide_icons::Icon;

pub(super) struct TableStructureContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) info: &'a UiTableInfo,
    pub(super) search: &'a mut String,
}

pub(super) fn draw_placeholder(theme: DbProTheme, error: Option<&str>, ui: &mut egui::Ui) {
    egui::Frame {
        fill: theme.surface_panel,
        inner_margin: egui::Margin::symmetric(SPACE_MD as i8, SPACE_SM as i8),
        stroke: egui::Stroke::new(STROKE_THIN, theme.border_subtle),
        corner_radius: egui::CornerRadius::same(RADIUS_SM as u8),
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(28.0);
            let failed = error.is_some();
            let icon = if failed { Icon::TriangleAlert } else { Icon::LoaderCircle };
            let color = if failed { theme.warning } else { theme.accent };
            ui.label(RichText::new(char::from(icon).to_string()).font(font_icon(ICON_XL)).color(color));
            ui.add_space(8.0);
            ui.label(
                RichText::new(if failed {
                    "Table structure could not be loaded"
                } else {
                    "Loading table structure…"
                })
                .strong()
                .color(theme.text_primary),
            );
            ui.label(
                RichText::new(error.unwrap_or("Columns, keys and indexes will appear here."))
                    .small()
                    .color(theme.text_secondary),
            );
            ui.add_space(28.0);
        });
    });
}

impl TableStructureContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) {
        self.draw_metrics(ui);
        ui.add_space(8.0);
        self.draw_columns(ui);
    }

    fn draw_metrics(&self, ui: &mut egui::Ui) {
        egui::Frame {
            fill: self.theme.surface_panel,
            inner_margin: egui::Margin::symmetric(SPACE_SM as i8, SPACE_XS as i8),
            stroke: egui::Stroke::new(STROKE_THIN, self.theme.border_subtle),
            corner_radius: egui::CornerRadius::same(RADIUS_SM as u8),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = SPACE_SM;
                ui.label(
                    RichText::new(t!("table.metrics"))
                        .font(font_caption())
                        .strong()
                        .color(self.theme.text_secondary),
                );
                Badge::new(format!("{} columns", self.info.columns.len()), self.theme)
                    .variant(BadgeVariant::Default)
                    .show(ui);
                if let Some(primary_key) = &self.info.primary_key {
                    Badge::new(format!("PK: {}", primary_key.join(", ")), self.theme)
                        .variant(BadgeVariant::Warning)
                        .show(ui);
                }
                Badge::new(format!("{} indexes", self.info.indexes.len()), self.theme)
                    .variant(BadgeVariant::Secondary)
                    .show(ui);
                Badge::new(format!("{} foreign keys", self.info.foreign_keys.len()), self.theme)
                    .variant(BadgeVariant::Secondary)
                    .show(ui);
                if !self.info.check_constraints.is_empty() {
                    Badge::new(
                        format!("{} check constraints", self.info.check_constraints.len()),
                        self.theme,
                    )
                    .variant(BadgeVariant::Outline)
                    .show(ui);
                }
                if !self.info.dependencies.is_empty() {
                    Badge::new(format!("{} dependencies", self.info.dependencies.len()), self.theme)
                        .variant(BadgeVariant::Outline)
                        .show(ui);
                }
                if let Some(row_count) = self.info.row_count {
                    Badge::new(format!("{row_count} rows"), self.theme)
                        .variant(BadgeVariant::Secondary)
                        .show(ui);
                }
            });
        });
    }

    fn draw_columns(&mut self, ui: &mut egui::Ui) {
        Card::new(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            self.draw_columns_header(ui);
            ui.add_space(8.0);
            let matching_columns = self.matching_columns();
            if matching_columns.is_empty() {
                EmptyState::new(
                    Icon::Columns3,
                    &t!("table.no_columns"),
                    "Try entering a different column name or data type.",
                    self.theme,
                )
                .show(ui);
                return;
            }
            self.draw_column_table(ui, &matching_columns);
        });
    }

    fn draw_columns_header(&mut self, ui: &mut egui::Ui) {
        table_workspace_surface_view::draw_metadata_filter_header(
            ui,
            self.theme,
            "Columns",
            self.search,
            "Search columns or types…",
            &format!("Total: {} columns", self.info.columns.len()),
        );
    }

    fn matching_columns(&self) -> Vec<&UiTableColumn> {
        let filter_lower = self.search.trim().to_lowercase();
        self.info
            .columns
            .iter()
            .filter(|column| {
                filter_lower.is_empty()
                    || column.name.to_lowercase().contains(&filter_lower)
                    || column.data_type.to_lowercase().contains(&filter_lower)
                    || column
                        .default
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&filter_lower)
            })
            .collect()
    }

    fn draw_column_table(&self, ui: &mut egui::Ui, matching_columns: &[&UiTableColumn]) {
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
                        let is_fk = self
                            .info
                            .foreign_keys
                            .iter()
                            .any(|foreign_key| foreign_key.from_columns.contains(&column.name));
                        self.draw_column_cell(ui, column, col_idx, is_fk);
                    },
                );
            });
    }

    fn draw_column_cell(&self, ui: &mut egui::Ui, column: &UiTableColumn, col_idx: usize, is_fk: bool) {
        match col_idx {
            0 => self.draw_ordinal_cell(ui, column),
            1 => self.draw_name_cell(ui, column, is_fk),
            2 => self.draw_data_type_cell(ui, column),
            3 => self.draw_nullable_cell(ui, column),
            4 => self.draw_key_cell(ui, column, is_fk),
            5 => self.draw_default_cell(ui, column),
            _ => {}
        }
    }

    fn draw_ordinal_cell(&self, ui: &mut egui::Ui, column: &UiTableColumn) {
        ui.label(
            RichText::new(column.ordinal.to_string())
                .font(font_caption())
                .color(self.theme.text_muted),
        );
    }

    fn draw_name_cell(&self, ui: &mut egui::Ui, column: &UiTableColumn, is_fk: bool) {
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
            let display_name = crate::components::truncate_ellipsis(&column.name, 32);
            ui.label(
                RichText::new(&display_name)
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
        });
    }

    fn draw_data_type_cell(&self, ui: &mut egui::Ui, column: &UiTableColumn) {
        let truncated = crate::components::truncate_ellipsis(&column.data_type, 24);
        let resp = ui.label(RichText::new(&truncated).monospace().color(self.theme.text_secondary));
        if column.data_type.chars().count() > 24 {
            resp.on_hover_text(&column.data_type);
        } else {
            resp.on_hover_text("Full database type");
        }
    }

    fn draw_nullable_cell(&self, ui: &mut egui::Ui, column: &UiTableColumn) {
        let (label, variant) = if column.nullable {
            ("NULL", BadgeVariant::Secondary)
        } else {
            ("NOT NULL", BadgeVariant::Outline)
        };
        Badge::new(label, self.theme).variant(variant).compact(true).show(ui);
    }

    fn draw_key_cell(&self, ui: &mut egui::Ui, column: &UiTableColumn, is_fk: bool) {
        let key = if column.is_primary_key {
            Some(("PK", BadgeVariant::Warning))
        } else if is_fk {
            Some(("FK", BadgeVariant::Default))
        } else if column.is_unique {
            Some(("UNIQUE", BadgeVariant::Default))
        } else {
            None
        };
        if let Some((label, variant)) = key {
            Badge::new(label, self.theme).variant(variant).compact(true).show(ui);
        } else {
            ui.label(RichText::new("—").font(font_caption()).color(self.theme.text_muted));
        }
    }

    fn draw_default_cell(&self, ui: &mut egui::Ui, column: &UiTableColumn) {
        if let Some(default_val) = column.default.as_deref() {
            let truncated = crate::components::truncate_ellipsis(default_val, 28);
            let resp = ui.label(
                RichText::new(&truncated)
                    .font(font_caption())
                    .color(self.theme.text_secondary),
            );
            if default_val.chars().count() > 28 {
                resp.on_hover_text(default_val);
            }
        } else {
            ui.label(RichText::new("—").font(font_caption()).color(self.theme.text_secondary));
        }
    }
}
