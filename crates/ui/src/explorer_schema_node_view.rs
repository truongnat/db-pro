//! Interaction boundary for a schema node in the database explorer.

use super::explorer_tree::{draw_codex_tree_row, CodexTreeRow};
use super::DbProTheme;
use eframe::egui;
use lucide_icons::Icon;

pub(crate) struct SchemaNodeContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) connection_id: &'a str,
    pub(crate) schema: &'a str,
    pub(crate) is_active: bool,
    pub(crate) table_count: usize,
}

pub(crate) struct SchemaNodeRender {
    pub(crate) is_open: bool,
    pub(crate) should_activate: bool,
}

impl SchemaNodeContext<'_> {
    pub(crate) fn draw(&self, ui: &mut egui::Ui) -> SchemaNodeRender {
        let schema_id = ui.make_persistent_id(("codex_schema_node", self.connection_id, self.schema));
        let mut collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), schema_id, self.is_active);
        let is_open = collapsing.is_open();
        let (response, chevron_clicked) = self.draw_row(ui, is_open);
        let should_activate = self.apply_interaction(ui, &mut collapsing, is_open, chevron_clicked, response.clicked());

        SchemaNodeRender {
            is_open: collapsing.is_open(),
            should_activate,
        }
    }

    fn draw_row(&self, ui: &mut egui::Ui, is_open: bool) -> (egui::Response, bool) {
        draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 2,
                is_expandable: true,
                is_expanded: is_open,
                icon: if is_open { Icon::FolderOpen } else { Icon::Folder },
                icon_color: if self.is_active {
                    self.theme.warning
                } else {
                    self.theme.text_secondary
                },
                label: self.schema,
                is_selected: false,
                is_dimmed: !self.is_active,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: (self.table_count > 0).then(|| self.table_count.to_string()),
                detail_text: None,
            },
        )
    }

    fn apply_interaction(
        &self,
        ui: &egui::Ui,
        collapsing: &mut egui::collapsing_header::CollapsingState,
        is_open: bool,
        chevron_clicked: bool,
        row_clicked: bool,
    ) -> bool {
        if chevron_clicked {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
            return false;
        }
        if !row_clicked {
            return false;
        }
        if self.is_active {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
            false
        } else {
            collapsing.set_open(true);
            collapsing.store(ui.ctx());
            true
        }
    }
}
