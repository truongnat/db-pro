use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::table::{Table, TableColumn};
use egui::{Align, Layout, RichText};
use lucide_icons::Icon;

impl DbProApp {
    /// Draw the Indexes tab: full index metadata table with search filter and unique badges.
    pub(super) fn draw_table_indexes_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table.state.table_info.clone() else {
            ui.label(RichText::new("Table structure is still loading…").color(self.theme.text_muted));
            return;
        };

        let mut selected_index = None;
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                section_label(ui, "INDEXES", self.theme);
                ui.add_space(8.0);
                input(
                    ui,
                    &mut self.table.state.table_metadata_search,
                    "Filter indexes…",
                    220.0,
                    self.theme,
                );
                if !self.table.state.table_metadata_search.is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Clear filter")
                        .show(ui)
                        .clicked()
                {
                    self.table.state.table_metadata_search.clear();
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("Total: {} indexes", info.indexes.len()))
                            .font(font_caption())
                            .color(self.theme.text_muted),
                    );
                });
            });
            ui.add_space(8.0);

            let filter_lower = self.table.state.table_metadata_search.trim().to_lowercase();
            let matching_indexes: Vec<_> = info
                .indexes
                .iter()
                .filter(|idx| {
                    if filter_lower.is_empty() {
                        true
                    } else {
                        idx.name.to_lowercase().contains(&filter_lower)
                            || idx.columns.iter().any(|c| c.to_lowercase().contains(&filter_lower))
                    }
                })
                .collect();

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

            let cols = [
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
                    Table::new(&cols, self.theme).row_height(34.0).show(
                        ui,
                        matching_indexes.len(),
                        |_| false,
                        |_| {},
                        |_| {},
                        |_| {},
                        |ui, row_idx, col_idx| {
                            let index = matching_indexes[row_idx];
                            match col_idx {
                                0 => {
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
                                            selected_index = Some(index.name.clone());
                                        }
                                    });
                                }
                                1 => {
                                    ui.label(
                                        RichText::new(index.columns.join(", "))
                                            .monospace()
                                            .color(self.theme.text_secondary),
                                    );
                                }
                                2 => {
                                    ui.label(
                                        RichText::new(&index.method)
                                            .monospace()
                                            .color(self.theme.text_secondary),
                                    );
                                }
                                3 => {
                                    let include_columns = if index.include_columns.is_empty() {
                                        "—".to_owned()
                                    } else {
                                        index.include_columns.join(", ")
                                    };
                                    ui.label(
                                        RichText::new(include_columns)
                                            .monospace()
                                            .color(self.theme.text_secondary),
                                    );
                                }
                                4 => {
                                    ui.label(
                                        RichText::new(index.predicate.as_deref().unwrap_or("—"))
                                            .monospace()
                                            .color(self.theme.text_secondary),
                                    )
                                    .on_hover_text(&index.definition);
                                }
                                5 => {
                                    ui.label(
                                        RichText::new(if index.unique {
                                            if index.primary {
                                                "PRIMARY KEY"
                                            } else {
                                                "Enforces uniqueness"
                                            }
                                        } else {
                                            "Active index"
                                        })
                                        .font(font_caption())
                                        .color(self.theme.text_muted),
                                    );
                                }
                                _ => {}
                            }
                        },
                    );
                });
        });
        if let Some(index_name) = selected_index {
            self.table.state.table_index_detail = Some(index_name);
        }
        self.draw_table_index_detail(ui.ctx(), &info);
    }

    fn draw_table_index_detail(&mut self, ctx: &egui::Context, info: &UiTableInfo) {
        let Some(index_name) = self.table.state.table_index_detail.clone() else {
            return;
        };
        let Some(index) = info.indexes.iter().find(|index| index.name == index_name) else {
            self.table.state.table_index_detail = None;
            return;
        };
        let mut open = true;
        egui::Window::new(format!("Index · {}", index.name))
            .open(&mut open)
            .resizable(true)
            .default_width(520.0)
            .show(ctx, |ui| {
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
        if !open {
            self.table.state.table_index_detail = None;
        }
    }
}
