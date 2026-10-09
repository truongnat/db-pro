//! Interaction boundary for a schema node in the database explorer.

use super::explorer_tree::{draw_codex_tree_row, CodexTreeRow};
use super::{context_action_menu, ctx_menu_item, is_context_menu_triggered, DbProTheme};
use eframe::egui;
use lucide_icons::Icon;

pub(crate) struct SchemaNodeContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) connection_id: &'a str,
    pub(crate) schema: &'a str,
    pub(crate) is_active: bool,
    pub(crate) table_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SchemaNodeAction {
    Activate,
    OpenErDiagram,
    RefreshSchema,
    CopyName,
    DropSchema,
}

pub(crate) struct SchemaNodeRender {
    pub(crate) is_open: bool,
    pub(crate) should_activate: bool,
    pub(crate) actions: Vec<SchemaNodeAction>,
}

impl SchemaNodeContext<'_> {
    pub(crate) fn draw(&self, ui: &mut egui::Ui) -> SchemaNodeRender {
        let schema_id = ui.make_persistent_id(("codex_schema_node", self.connection_id, self.schema));
        let mut collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), schema_id, self.is_active);
        let is_open = collapsing.is_open();
        let (response, chevron_clicked) = self.draw_row(ui, is_open);
        let is_context_menu = is_context_menu_triggered(&response, ui);
        let actions = self.draw_menu(ui, &response);
        let should_activate = self.apply_interaction(
            ui,
            &mut collapsing,
            is_open,
            chevron_clicked,
            response.clicked() && !is_context_menu,
        );

        SchemaNodeRender {
            is_open: collapsing.is_open(),
            should_activate,
            actions,
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

    // cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
    fn draw_menu(&self, ui: &mut egui::Ui, response: &egui::Response) -> Vec<SchemaNodeAction> {
        let mut actions = Vec::new();
        context_action_menu(ui, response, self.theme, |ui, close_menu| {
            if !self.is_active {
                self.add_item(
                    ui,
                    &mut actions,
                    close_menu,
                    SchemaNodeAction::Activate,
                    Icon::FolderOpen,
                    "Activate Schema",
                );
            }
            self.add_item(
                ui,
                &mut actions,
                close_menu,
                SchemaNodeAction::OpenErDiagram,
                Icon::Workflow,
                "View ER Diagram",
            );
            self.add_item(
                ui,
                &mut actions,
                close_menu,
                SchemaNodeAction::RefreshSchema,
                Icon::RefreshCw,
                "Refresh Schema",
            );
            ui.separator();
            self.add_item(
                ui,
                &mut actions,
                close_menu,
                SchemaNodeAction::CopyName,
                Icon::Copy,
                "Copy Schema Name",
            );
            if ctx_menu_item(
                ui,
                Some(Icon::Trash2),
                "Drop Schema...",
                None,
                self.theme.danger,
                self.theme,
            )
            .clicked()
            {
                actions.push(SchemaNodeAction::DropSchema);
                *close_menu = true;
            }
        });
        actions
    }

    fn add_item(
        &self,
        ui: &mut egui::Ui,
        actions: &mut Vec<SchemaNodeAction>,
        close_menu: &mut bool,
        action: SchemaNodeAction,
        icon: Icon,
        label: &str,
    ) {
        if ctx_menu_item(ui, Some(icon), label, None, self.theme.text_primary, self.theme).clicked() {
            actions.push(action);
            *close_menu = true;
        }
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
            let now_open = !is_open;
            collapsing.set_open(now_open);
            collapsing.store(ui.ctx());
            // Expanding an inactive schema is intent to browse it: activate (lazy load) on open
            // instead of leaving an "Inactive schema — click to activate" hint behind.
            return now_open && !self.is_active;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_node_defaults_open_only_when_active() {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let theme = DbProTheme::light();
        for (is_active, expected_open) in [(false, false), (true, true)] {
            let mut seen_open = None;
            let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let render = SchemaNodeContext {
                        theme,
                        connection_id: "conn-1",
                        schema: "public",
                        is_active,
                        table_count: 0,
                    }
                    .draw(ui);
                    seen_open = Some(render.is_open);
                });
            });
            assert_eq!(seen_open, Some(expected_open));
        }
    }
}
