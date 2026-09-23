//! Shared interaction boundary for schema-object rows in the explorer.

use super::explorer_tree::{draw_codex_tree_row, CodexTreeRow};
use super::{context_action_menu, ctx_menu_item, is_context_menu_triggered, DbProTheme};
use eframe::egui;
use lucide_icons::Icon;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SchemaObjectRowAction {
    Open,
    OpenQuery,
    Modify,
    Drop,
    CopyName,
}

pub(crate) struct SchemaObjectRowContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) label: &'a str,
    pub(crate) icon: Icon,
    pub(crate) icon_color: egui::Color32,
    pub(crate) is_selected: bool,
    pub(crate) query_label: Option<&'a str>,
    pub(crate) query_icon: Icon,
    pub(crate) open_label: &'a str,
    pub(crate) open_icon: Icon,
    pub(crate) modify_label: Option<&'a str>,
    pub(crate) drop_label: Option<&'a str>,
    pub(crate) copy_label: &'a str,
}

impl SchemaObjectRowContext<'_> {
    pub(crate) fn draw(&self, ui: &mut egui::Ui) -> Vec<SchemaObjectRowAction> {
        let response = self.draw_row(ui);
        let is_context_menu = is_context_menu_triggered(&response, ui);
        let mut actions = self.draw_menu(ui, &response);
        if response.clicked() && !is_context_menu {
            actions.push(SchemaObjectRowAction::Open);
        }
        actions
    }

    fn draw_row(&self, ui: &mut egui::Ui) -> egui::Response {
        let (response, _) = draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 4,
                is_expandable: false,
                is_expanded: false,
                icon: self.icon,
                icon_color: if self.is_selected {
                    self.theme.accent
                } else {
                    self.icon_color
                },
                label: self.label,
                is_selected: self.is_selected,
                is_dimmed: false,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: None,
                detail_text: None,
            },
        );
        response
    }

    fn draw_menu(&self, ui: &mut egui::Ui, response: &egui::Response) -> Vec<SchemaObjectRowAction> {
        let mut menu = SchemaObjectMenu {
            theme: self.theme,
            actions: Vec::new(),
        };
        context_action_menu(ui, response, self.theme, |ui, close_menu| {
            if let Some(query_label) = self.query_label {
                menu.add_item(
                    ui,
                    close_menu,
                    SchemaObjectRowAction::OpenQuery,
                    self.query_icon,
                    query_label,
                );
            } else {
                menu.add_item(
                    ui,
                    close_menu,
                    SchemaObjectRowAction::Open,
                    self.open_icon,
                    self.open_label,
                );
            }
            if let Some(modify_label) = self.modify_label {
                menu.add_item(
                    ui,
                    close_menu,
                    SchemaObjectRowAction::Modify,
                    Icon::Pencil,
                    modify_label,
                );
            }
            if let Some(drop_label) = self.drop_label {
                menu.add_item(
                    ui,
                    close_menu,
                    SchemaObjectRowAction::Drop,
                    Icon::Trash2,
                    drop_label,
                );
            }
            menu.add_item(
                ui,
                close_menu,
                SchemaObjectRowAction::CopyName,
                Icon::Copy,
                self.copy_label,
            );
        });
        menu.actions
    }
}

struct SchemaObjectMenu {
    theme: DbProTheme,
    actions: Vec<SchemaObjectRowAction>,
}

impl SchemaObjectMenu {
    fn add_item(
        &mut self,
        ui: &mut egui::Ui,
        close_menu: &mut bool,
        action: SchemaObjectRowAction,
        icon: Icon,
        label: &str,
    ) {
        if ctx_menu_item(ui, Some(icon), label, None, self.theme.text_primary, self.theme).clicked() {
            self.actions.push(action);
            *close_menu = true;
        }
    }
}
