// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
//! Interaction boundary for the database node under a connected explorer row.

use super::explorer_tree::{draw_codex_tree_row, CodexTreeRow};
use super::{context_action_menu, ctx_menu_item, is_context_menu_triggered, DbProTheme};
use eframe::egui;
use lucide_icons::Icon;

pub(crate) struct DatabaseNodeContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) connection_id: &'a str,
    pub(crate) database: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DatabaseNodeAction {
    RefreshSchema,
    CopyName,
}

pub(crate) struct DatabaseNodeRender {
    pub(crate) is_open: bool,
    pub(crate) actions: Vec<DatabaseNodeAction>,
}

impl DatabaseNodeContext<'_> {
    pub(crate) fn draw(&self, ui: &mut egui::Ui) -> DatabaseNodeRender {
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
        let is_context_menu = is_context_menu_triggered(&response, ui);
        let actions = self.draw_menu(ui, &response);

        if (response.clicked() && !is_context_menu) || chevron_clicked {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
        }
        DatabaseNodeRender {
            is_open: collapsing.is_open(),
            actions,
        }
    }

    fn draw_menu(&self, ui: &mut egui::Ui, response: &egui::Response) -> Vec<DatabaseNodeAction> {
        let mut actions = Vec::new();
        context_action_menu(ui, response, self.theme, |ui, close_menu| {
            if ctx_menu_item(
                ui,
                Some(Icon::RefreshCw),
                "Refresh Schema",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                actions.push(DatabaseNodeAction::RefreshSchema);
                *close_menu = true;
            }
            ui.separator();
            if ctx_menu_item(
                ui,
                Some(Icon::Copy),
                "Copy Path",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                actions.push(DatabaseNodeAction::CopyName);
                *close_menu = true;
            }
        });
        actions
    }

    fn database_label(&self) -> &str {
        if self.database.is_empty() {
            "database"
        } else {
            self.database
        }
    }
}
