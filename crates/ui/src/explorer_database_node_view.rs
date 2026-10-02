//! Interaction boundary for the database node under a connected explorer row.

use super::explorer_tree::{draw_codex_tree_row, CodexTreeRow};
use super::DbProTheme;
use eframe::egui;
use lucide_icons::Icon;
use std::cell::RefCell;

pub(crate) struct DatabaseNodeContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) connection_id: &'a str,
    pub(crate) database: &'a str,
    pub(crate) native_runtime: &'a RefCell<crate::native_runtime_shell::RsUiShellRuntime>,
}

impl DatabaseNodeContext<'_> {
    pub(crate) fn draw(&self, ui: &mut egui::Ui) -> bool {
        let node_id = ui.make_persistent_id(("codex_db_node", self.connection_id, self.database));
        let mut collapsing = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), node_id, true);
        let is_open = collapsing.is_open();
        let (response, chevron_clicked) = draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 1,
                is_expandable: true,
                is_expanded: is_open,
                icon: Icon::Database,
                icon_color: self.theme.accent,
                label: self.database_label(),
                is_selected: false,
                is_dimmed: false,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: None,
                detail_text: None,
            },
        );

        let key = super::explorer_tree::database_tree_key(self.connection_id, self.database);
        let parent_key = super::explorer_tree::connection_tree_key(self.connection_id);
        let activated = super::explorer_tree::sync_explorer_tree_item(
            self.native_runtime,
            crate::native_runtime_shell::ExplorerTreeItem {
                key: &key,
                parent_key: Some(&parent_key),
                label: self.database_label(),
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
        collapsing.is_open()
    }

    fn database_label(&self) -> &str {
        if self.database.is_empty() {
            "database"
        } else {
            self.database
        }
    }
}
