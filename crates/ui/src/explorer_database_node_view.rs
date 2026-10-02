//! Interaction boundary for the database node under a connected explorer row.

use super::explorer_tree::{draw_codex_tree_row, CodexTreeRow};
use super::DbProTheme;
use eframe::egui;
use lucide_icons::Icon;

pub(crate) struct DatabaseNodeContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) connection_id: &'a str,
    pub(crate) database: &'a str,
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

        if response.clicked() || chevron_clicked {
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
