//! Table-index presentation.
use super::super::*;
use crate::components::table::{Table, TableColumn};
use crate::tokens::SPACE_SM;
use crate::UiTableIndex;
use egui::RichText;
use lucide_icons::Icon;

pub(super) struct TableIndexesContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) info: &'a UiTableInfo,
    pub(super) search: &'a mut String,
}

pub(super) fn draw_loading(theme: DbProTheme, ui: &mut egui::Ui) {
    ui.label(RichText::new("Table structure is still loading…").color(theme.text_muted));
}

impl TableIndexesContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) {
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
            self.draw_table(ui, &matching_indexes);
        });
    }

    fn draw_header(&mut self, ui: &mut egui::Ui) {
        let count = self.info.indexes.len();
        table_workspace_surface_view::draw_metadata_filter_header(
            ui,
            self.theme,
            "Indexes",
            self.search,
            "Filter indexes…",
            &format!("Total: {count} {}", if count == 1 { "index" } else { "indexes" }),
        );
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

    fn draw_table(&self, ui: &mut egui::Ui, indexes: &[&UiTableIndex]) {
        egui::ScrollArea::horizontal()
            .id_salt("indexes-table-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let columns = index_columns(ui.available_width());
                Table::new(&columns, self.theme).row_height(34.0).show(
                    ui,
                    indexes.len(),
                    |_| false,
                    |_| {},
                    |_| {},
                    |_| {},
                    |ui, row_idx, col_idx| self.draw_cell(ui, indexes[row_idx], col_idx),
                );
            });
    }

    fn draw_cell(&self, ui: &mut egui::Ui, index: &UiTableIndex, col_idx: usize) {
        match col_idx {
            0 => self.draw_name_cell(ui, index),
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
                let pred_str = index.predicate.as_deref().unwrap_or("—");
                ui.add(
                    egui::Label::new(RichText::new(pred_str).monospace().color(self.theme.text_secondary)).truncate(),
                )
                .on_hover_text(format!("{}\n{}", index.name, index.definition));
            }
            5 => self.draw_status_cell(ui, index),
            _ => {}
        }
    }

    fn draw_name_cell(&self, ui: &mut egui::Ui, index: &UiTableIndex) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_SM;
            ui.label(icon_text(
                if index.unique { Icon::BadgeCheck } else { Icon::List },
                "",
                if index.unique {
                    self.theme.accent
                } else {
                    self.theme.text_muted
                },
            ));
            index_name_label(ui, index, self.theme);
        });
    }

    fn draw_text_cell(&self, ui: &mut egui::Ui, text: String) {
        ui.add(egui::Label::new(RichText::new(&text).monospace().color(self.theme.text_secondary)).truncate())
            .on_hover_text(&text);
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
        ui.add(egui::Label::new(RichText::new(status).font(font_caption()).color(self.theme.text_muted)).truncate())
            .on_hover_text(status);
    }
}

fn index_name_label(ui: &mut egui::Ui, index: &UiTableIndex, theme: DbProTheme) -> egui::Response {
    let response = ui
        .add(
            egui::Label::new(RichText::new(&index.name).strong().color(theme.text_primary))
                .truncate()
                .sense(egui::Sense::hover())
                .selectable(false),
        )
        .on_hover_text(format!("{}\n{}", index.name, index.definition));
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, ui.is_enabled(), &index.name));
    response
}

fn index_columns(available_width: f32) -> [TableColumn<'static>; 6] {
    let width = (available_width - 2.0).max(640.0);
    [
        TableColumn::new("Index Name").width(width * 0.20),
        TableColumn::new("Indexed Columns").width(width * 0.21),
        TableColumn::fixed("Method", width * 0.10),
        TableColumn::new("INCLUDE").width(width * 0.16),
        TableColumn::new("Predicate").width(width * 0.15),
        TableColumn::new("Status").width(width * 0.18),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{CentralPanel, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};

    fn test_index() -> UiTableIndex {
        UiTableIndex {
            name: "users_email_idx".to_owned(),
            columns: vec!["email".to_owned()],
            unique: true,
            method: "btree".to_owned(),
            primary: false,
            include_columns: Vec::new(),
            predicate: None,
            definition: "CREATE UNIQUE INDEX users_email_idx ON users (email)".to_owned(),
        }
    }

    #[test]
    fn index_name_does_not_activate_on_click() {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let index = test_index();
        let theme = DbProTheme::dark();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(420.0, 120.0));
        let mut label_rect = None;
        let _ = crate::test_frame::frame(&ctx, RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        }, |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                label_rect = Some(index_name_label(ui, &index, theme).rect);
            });
        });
        let click_pos = label_rect.expect("index label is laid out").center();

        let mut clicked = false;
        for pressed in [true, false] {
            let _ = crate::test_frame::frame(&ctx, RawInput {
                screen_rect: Some(screen),
                events: vec![Event::PointerButton {
                    pos: click_pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::default(),
                }],
                ..Default::default()
            }, |ctx| {
                CentralPanel::default().show(ctx, |ui| {
                    clicked = index_name_label(ui, &index, theme).clicked();
                });
            });
        }
        assert!(!clicked, "the index name is metadata, not a modal action");
    }

    #[test]
    fn index_columns_fit_the_standard_table_detail_viewport() {
        let available_width = 928.0;
        let columns = index_columns(available_width);
        let requested_width = columns.iter().map(|column| column.width.unwrap_or(80.0)).sum::<f32>();

        assert!(
            requested_width <= available_width,
            "index columns request {requested_width} pt from a {available_width} pt viewport"
        );
        assert!(
            columns[5].width.unwrap_or_default() >= 150.0,
            "Status needs room for its labels"
        );
    }
}
