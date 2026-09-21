//! Table-index presentation and typed detail intents.
use super::super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use crate::components::table::{Table, TableColumn};
use crate::UiTableIndex;
use egui::{Align, Layout, RichText};
use lucide_icons::Icon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TableIndexesAction {
    SelectIndex(String),
    CloseDetail,
}

pub(super) struct TableIndexesContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) info: &'a UiTableInfo,
    pub(super) search: &'a mut String,
    pub(super) selected_index: Option<&'a str>,
}

pub(super) fn draw_loading(theme: DbProTheme, ui: &mut egui::Ui) {
    ui.label(RichText::new("Table structure is still loading…").color(theme.text_muted));
}

impl TableIndexesContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> Vec<TableIndexesAction> {
        let mut actions = Vec::new();
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            self.draw_header(ui);
            ui.add_space(8.0);
            let matching_indexes = self.matching_indexes();
            if matching_indexes.is_empty() {
                empty_state(
                    ui,
                    Icon::List,
                    "No indexes found",
                    "This table has no indexes defined or none match the search.",
                    self.theme,
                );
                return;
            }
            self.draw_table(ui, &matching_indexes, &mut actions);
        });
        self.draw_detail(ctx, &mut actions);
        actions
    }

    fn draw_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            section_label(ui, "INDEXES", self.theme);
            ui.add_space(8.0);
            input(ui, self.search, "Filter indexes…", 220.0, self.theme);
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
                    RichText::new(format!("Total: {} indexes", self.info.indexes.len()))
                        .font(font_caption())
                        .color(self.theme.text_muted),
                );
            });
        });
    }

    fn matching_indexes(&self) -> Vec<&UiTableIndex> {
        let filter_lower = self.search.trim().to_lowercase();
        self.info
            .indexes
            .iter()
            .filter(|index| {
                filter_lower.is_empty()
                    || index.name.to_lowercase().contains(&filter_lower)
                    || index
                        .columns
                        .iter()
                        .any(|column| column.to_lowercase().contains(&filter_lower))
            })
            .collect()
    }

    fn draw_table(&self, ui: &mut egui::Ui, indexes: &[&UiTableIndex], actions: &mut Vec<TableIndexesAction>) {
        let columns = [
            TableColumn::new("Index Name").width(240.0),
            TableColumn::new("Indexed Columns").width(280.0),
            TableColumn::fixed("Method", 100.0),
            TableColumn::new("INCLUDE").width(180.0),
            TableColumn::new("Predicate").width(220.0),
            TableColumn::new("Status"),
        ];
        egui::ScrollArea::horizontal()
            .id_salt("indexes-table-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                Table::new(&columns, self.theme).row_height(34.0).show(
                    ui,
                    indexes.len(),
                    |_| false,
                    |_| {},
                    |_| {},
                    |_| {},
                    |ui, row_idx, col_idx| self.draw_cell(ui, indexes[row_idx], col_idx, actions),
                );
            });
    }

    fn draw_cell(
        &self,
        ui: &mut egui::Ui,
        index: &UiTableIndex,
        col_idx: usize,
        actions: &mut Vec<TableIndexesAction>,
    ) {
        match col_idx {
            0 => self.draw_name_cell(ui, index, actions),
            1 => self.draw_text_cell(ui, index.columns.join(", ")),
            2 => self.draw_text_cell(ui, index.method.clone()),
            3 => self.draw_text_cell(
                ui,
                if index.include_columns.is_empty() {
                    "—".to_owned()
                } else {
                    index.include_columns.join(", ")
                },
            ),
            4 => {
                ui.label(
                    RichText::new(index.predicate.as_deref().unwrap_or("—"))
                        .monospace()
                        .color(self.theme.text_secondary),
                )
                .on_hover_text(&index.definition);
            }
            5 => self.draw_status_cell(ui, index),
            _ => {}
        }
    }

    fn draw_name_cell(&self, ui: &mut egui::Ui, index: &UiTableIndex, actions: &mut Vec<TableIndexesAction>) {
        ui.horizontal(|ui| {
            ui.label(icon_text(
                if index.unique { Icon::BadgeCheck } else { Icon::List },
                "",
                if index.unique {
                    self.theme.accent
                } else {
                    self.theme.text_muted
                },
            ));
            let response = ui
                .label(RichText::new(&index.name).strong().color(self.theme.text_primary))
                .on_hover_text(&index.definition);
            if response.clicked() {
                actions.push(TableIndexesAction::SelectIndex(index.name.clone()));
            }
        });
    }

    fn draw_text_cell(&self, ui: &mut egui::Ui, text: String) {
        ui.label(RichText::new(text).monospace().color(self.theme.text_secondary));
    }

    fn draw_status_cell(&self, ui: &mut egui::Ui, index: &UiTableIndex) {
        let status = if index.unique {
            if index.primary {
                "PRIMARY KEY"
            } else {
                "Enforces uniqueness"
            }
        } else {
            "Active index"
        };
        ui.label(RichText::new(status).font(font_caption()).color(self.theme.text_muted));
    }

    fn draw_detail(&self, ctx: &egui::Context, actions: &mut Vec<TableIndexesAction>) {
        let Some(index_name) = self.selected_index else {
            return;
        };
        let Some(index) = self.info.indexes.iter().find(|index| index.name == index_name) else {
            return;
        };
        let mut open = true;
        Dialog::new(&mut open, format!("Index · {}", index.name), self.theme)
            .width(560.0)
            .id_salt("table_index_detail_dialog")
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    ui.label(RichText::new(&index.definition).monospace());
                    ui.separator();
                    ui.label(format!("Method: {}", index.method));
                    ui.label(format!("Primary: {} · Unique: {}", index.primary, index.unique));
                    ui.label(format!("Columns: {}", index.columns.join(", ")));
                    if !index.include_columns.is_empty() {
                        ui.label(format!("INCLUDE: {}", index.include_columns.join(", ")));
                    }
                    if let Some(predicate) = &index.predicate {
                        ui.label(format!("Predicate: {predicate}"));
                    }
                });
            });
        if !open {
            actions.push(TableIndexesAction::CloseDetail);
        }
    }
}
