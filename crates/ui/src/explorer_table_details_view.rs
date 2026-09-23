//! Presentation-only detail folders for a selected Explorer table.

use super::explorer_tree::{
    column_icon_and_color, draw_category_folder, draw_codex_tree_row, shorten_data_type, CategoryFolder, CodexTreeRow,
};
use super::{DbProTheme, UiTableInfo};
use eframe::egui;
use lucide_icons::Icon;

pub(super) struct TableDetailsView<'a> {
    theme: &'a DbProTheme,
}

impl<'a> TableDetailsView<'a> {
    pub(super) fn new(theme: &'a DbProTheme) -> Self {
        Self { theme }
    }

    pub(super) fn draw(&self, ui: &mut egui::Ui, table: &str, info: &UiTableInfo) {
        self.draw_columns(ui, table, info);
        self.draw_foreign_keys(ui, table, info);
        self.draw_indexes(ui, table, info);
    }

    fn draw_columns(&self, ui: &mut egui::Ui, table: &str, info: &UiTableInfo) {
        let theme = self.theme;
        let folder_id = ui.make_persistent_id(("codex_tbl_col_folder", table));
        draw_category_folder(
            ui,
            theme,
            CategoryFolder {
                depth: 5,
                id: folder_id,
                icon: Icon::Columns3,
                icon_color: theme.text_secondary,
                label: "Columns",
                count: info.columns.len(),
                empty_label: None,
            },
            |ui| {
                for column in &info.columns {
                    let is_fk = info
                        .foreign_keys
                        .iter()
                        .any(|foreign_key| foreign_key.from_columns.contains(&column.name));
                    let (icon, icon_color) =
                        column_icon_and_color(&column.data_type, column.is_primary_key, is_fk, theme);
                    let short_type = shorten_data_type(&column.data_type);
                    draw_codex_tree_row(
                        ui,
                        theme,
                        CodexTreeRow {
                            depth: 6,
                            is_expandable: false,
                            is_expanded: false,
                            icon,
                            icon_color,
                            label: &column.name,
                            is_selected: false,
                            is_dimmed: false,
                            status_dot: None,
                            badge_text: None,
                            badge_accent: false,
                            count_text: None,
                            detail_text: Some(&short_type),
                        },
                    );
                }
            },
        );
    }

    fn draw_foreign_keys(&self, ui: &mut egui::Ui, table: &str, info: &UiTableInfo) {
        let theme = self.theme;
        let folder_id = ui.make_persistent_id(("codex_tbl_fk_folder", table));
        draw_category_folder(
            ui,
            theme,
            CategoryFolder {
                depth: 5,
                id: folder_id,
                icon: Icon::ArrowRightLeft,
                icon_color: theme.text_secondary,
                label: "Foreign keys",
                count: info.foreign_keys.len(),
                empty_label: Some("No foreign keys"),
            },
            |ui| {
                for foreign_key in &info.foreign_keys {
                    draw_codex_tree_row(
                        ui,
                        theme,
                        CodexTreeRow {
                            depth: 6,
                            is_expandable: false,
                            is_expanded: false,
                            icon: Icon::Link,
                            icon_color: theme.info,
                            label: &foreign_key.name,
                            is_selected: false,
                            is_dimmed: false,
                            status_dot: None,
                            badge_text: None,
                            badge_accent: false,
                            count_text: None,
                            detail_text: Some(&foreign_key.to_table),
                        },
                    );
                }
            },
        );
    }

    fn draw_indexes(&self, ui: &mut egui::Ui, table: &str, info: &UiTableInfo) {
        let theme = self.theme;
        let folder_id = ui.make_persistent_id(("codex_tbl_idx_folder", table));
        draw_category_folder(
            ui,
            theme,
            CategoryFolder {
                depth: 5,
                id: folder_id,
                icon: Icon::List,
                icon_color: theme.text_secondary,
                label: "Indexes",
                count: info.indexes.len(),
                empty_label: Some("No indexes"),
            },
            |ui| {
                for index in &info.indexes {
                    draw_codex_tree_row(
                        ui,
                        theme,
                        CodexTreeRow {
                            depth: 6,
                            is_expandable: false,
                            is_expanded: false,
                            icon: Icon::Zap,
                            icon_color: theme.text_muted,
                            label: &index.name,
                            is_selected: false,
                            is_dimmed: false,
                            status_dot: None,
                            badge_text: None,
                            badge_accent: false,
                            count_text: None,
                            detail_text: None,
                        },
                    );
                }
            },
        );
    }
}
