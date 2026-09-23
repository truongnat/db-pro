//! Foreign-key table presentation and navigation intents.

use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::table::{Table, TableColumn};
use crate::UiTableInfo;
use egui::{Align, Layout, RichText};
use lucide_icons::Icon;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TableRelationsAction {
    OpenTable(String),
}

pub(super) struct TableRelationsContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) info: &'a UiTableInfo,
    pub(super) search: &'a mut String,
}

pub(super) fn draw_loading(theme: DbProTheme, ui: &mut egui::Ui) {
    ui.label(RichText::new("Table structure is still loading…").color(theme.text_muted));
}

impl TableRelationsContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<TableRelationsAction> {
        let mut actions = Vec::new();
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            self.draw_toolbar(ui);
            let filter = self.search.trim().to_lowercase();
            let matching = self
                .info
                .foreign_keys
                .iter()
                .filter(|foreign_key| matches_filter(foreign_key, &filter))
                .collect::<Vec<_>>();
            if matching.is_empty() {
                empty_state(
                    ui,
                    Icon::ArrowRightLeft,
                    "No foreign keys found",
                    "This table has no outgoing foreign keys or none match the search.",
                    self.theme,
                );
                return;
            }
            actions.extend(self.draw_table(ui, matching));
        });
        actions
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            section_label(ui, "FOREIGN KEYS", self.theme);
            ui.add_space(8.0);
            input(ui, self.search, "Filter foreign keys…", 220.0, self.theme);
            if !self.search.is_empty()
                && Button::new(self.theme)
                    .icon(Icon::X)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Clear filter")
                    .show(ui)
                    .clicked()
            {
                self.search.clear();
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("Total: {} foreign keys", self.info.foreign_keys.len()))
                        .font(font_caption())
                        .color(self.theme.text_muted),
                );
            });
        });
        ui.add_space(8.0);
    }

    fn draw_table(&self, ui: &mut egui::Ui, matching: Vec<&crate::UiTableForeignKey>) -> Vec<TableRelationsAction> {
        let mut actions = Vec::new();
        let columns = [
            TableColumn::new("Constraint Name").width(220.0),
            TableColumn::new("Source Columns").width(180.0),
            TableColumn::new("Target Table").width(200.0),
            TableColumn::new("Target Columns").width(180.0),
            TableColumn::new("Action"),
        ];
        egui::ScrollArea::horizontal()
            .id_salt("relations-table-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                Table::new(&columns, self.theme).row_height(34.0).show(
                    ui,
                    matching.len(),
                    |_| false,
                    |_| {},
                    |_| {},
                    |_| {},
                    |ui, row_idx, column_idx| {
                        self.draw_cell(ui, matching[row_idx], column_idx, &mut actions);
                    },
                );
            });
        actions
    }

    fn draw_cell(
        &self,
        ui: &mut egui::Ui,
        relation: &crate::UiTableForeignKey,
        column_idx: usize,
        actions: &mut Vec<TableRelationsAction>,
    ) {
        match column_idx {
            0 => {
                ui.horizontal(|ui| {
                    ui.label(icon_text(Icon::ArrowRightLeft, "", self.theme.accent));
                    ui.label(RichText::new(&relation.name).strong().color(self.theme.text_primary));
                });
            }
            1 => {
                ui.label(
                    RichText::new(relation.from_columns.join(", "))
                        .monospace()
                        .color(self.theme.text_secondary),
                );
            }
            2 => {
                ui.label(
                    RichText::new(format!("{}.{}", relation.to_schema, relation.to_table))
                        .strong()
                        .color(self.theme.text_primary),
                );
            }
            3 => {
                ui.label(
                    RichText::new(relation.to_columns.join(", "))
                        .monospace()
                        .color(self.theme.text_secondary),
                );
            }
            4 => {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "UPDATE {} · DELETE {}{}",
                            relation.on_update,
                            relation.on_delete,
                            deferrable_label(relation.deferrable, relation.initially_deferred)
                        ))
                        .font(font_caption())
                        .color(self.theme.text_muted),
                    )
                    .on_hover_text(format!("MATCH {}", relation.match_option));
                    if Button::new(self.theme)
                        .icon(Icon::ExternalLink)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Open referenced table")
                        .show(ui)
                        .clicked()
                    {
                        actions.push(TableRelationsAction::OpenTable(relation.to_table.clone()));
                    }
                });
            }
            _ => {}
        }
    }
}

fn matches_filter(foreign_key: &crate::UiTableForeignKey, filter: &str) -> bool {
    filter.is_empty()
        || foreign_key.name.to_lowercase().contains(filter)
        || foreign_key.to_table.to_lowercase().contains(filter)
        || foreign_key
            .from_columns
            .iter()
            .any(|column| column.to_lowercase().contains(filter))
}

fn deferrable_label(deferrable: bool, initially_deferred: bool) -> &'static str {
    if !deferrable {
        ""
    } else if initially_deferred {
        " · DEFERRABLE INITIALLY DEFERRED"
    } else {
        " · DEFERRABLE"
    }
}
