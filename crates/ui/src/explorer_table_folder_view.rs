//! Presentation boundary for the Tables folder in the explorer.

use super::explorer_tree::{draw_codex_tree_row, draw_hint_row, CodexTreeRow};
use super::{DbProTheme, EXPLORER_ROW_HEIGHT};
use eframe::egui;
use lucide_icons::Icon;
use std::cell::RefCell;

pub(crate) struct TableFolderContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) schema: &'a str,
    pub(crate) total_tables: usize,
    pub(crate) matching_tables: usize,
    pub(crate) search_query: &'a str,
    pub(crate) native_runtime: &'a RefCell<crate::native_runtime_shell::RsUiShellRuntime>,
    pub(crate) tree_key: String,
    pub(crate) tree_parent_key: String,
}

pub(crate) struct TableFolderRender {
    pub(crate) is_open: bool,
}

impl TableFolderContext<'_> {
    pub(crate) fn draw_header(&self, ui: &mut egui::Ui) -> TableFolderRender {
        let folder_id = ui.make_persistent_id(("codex_tbl_folder", self.schema));
        let default_open = !self.search_query.is_empty();
        let mut collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), folder_id, default_open);
        if default_open && !collapsing.is_open() {
            collapsing.set_open(true);
            collapsing.store(ui.ctx());
        }
        let is_open = collapsing.is_open();
        let (response, chevron_clicked) = self.draw_row(ui, is_open);
        let activated = super::explorer_tree::sync_explorer_tree_item(
            self.native_runtime,
            crate::native_runtime_shell::ExplorerTreeItem {
                key: &self.tree_key,
                parent_key: Some(&self.tree_parent_key),
                label: "Tables",
                expanded: is_open,
                selected: false,
                bounds: response.rect,
                focused: response.has_focus(),
                clicked: response.clicked() || chevron_clicked,
            },
            ui,
            &response,
        );
        if activated || chevron_clicked {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
        }
        TableFolderRender {
            is_open: collapsing.is_open(),
        }
    }

    pub(crate) fn draw_empty_state(&self, ui: &mut egui::Ui) {
        let label = if self.total_tables == 0 {
            "No tables in schema"
        } else {
            "No matching tables"
        };
        draw_hint_row(ui, &self.theme, 4, Icon::Info, label);
    }

    pub(crate) fn draw_offscreen_row_spacer(&self, ui: &mut egui::Ui) {
        let _ = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), EXPLORER_ROW_HEIGHT),
            egui::Sense::hover(),
        );
    }

    pub(crate) fn draw_overflow_hint(&self, ui: &mut egui::Ui, visible_count: usize) {
        if self.matching_tables > visible_count {
            draw_hint_row(
                ui,
                &self.theme,
                4,
                Icon::Ellipsis,
                &format!("Showing {} of {} — refine filter", visible_count, self.matching_tables),
            );
        }
    }

    fn draw_row(&self, ui: &mut egui::Ui, is_open: bool) -> (egui::Response, bool) {
        draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 3,
                is_expandable: true,
                is_expanded: is_open,
                icon: Icon::Table2,
                icon_color: self.theme.info,
                label: "Tables",
                is_selected: false,
                is_dimmed: self.total_tables == 0,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: Some(self.count_label()),
                detail_text: None,
            },
        )
    }

    fn count_label(&self) -> String {
        if self.search_query.is_empty() {
            self.total_tables.to_string()
        } else {
            format!("{}/{}", self.matching_tables, self.total_tables)
        }
    }
}
