//! Interaction boundary for a table row in the database explorer.

use super::explorer_tree::{draw_codex_tree_row, CodexTreeRow};
use super::{context_action_menu, ctx_menu_item, is_context_menu_triggered, DbProTheme};
use eframe::egui;
use lucide_icons::Icon;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TableRowAction {
    OpenData,
    OpenStructure,
    OpenModifyTable,
    DropTable,
    OpenQuery,
    GenerateInsert,
    GenerateUpdate,
    GenerateDelete,
    OpenDdl,
    CopyName,
    CopyQualifiedName,
    AskAgent,
    RefreshSchema,
}

impl TableRowAction {
    pub(crate) fn selects_table(self) -> bool {
        matches!(
            self,
            Self::OpenData
                | Self::OpenStructure
                | Self::OpenModifyTable
                | Self::DropTable
                | Self::OpenQuery
                | Self::GenerateInsert
                | Self::GenerateUpdate
                | Self::GenerateDelete
                | Self::OpenDdl
                | Self::AskAgent
        )
    }
}

pub(crate) struct TableRowRender {
    pub(crate) is_open: bool,
    pub(crate) should_select: bool,
    pub(crate) actions: Vec<TableRowAction>,
}

pub(crate) struct TableRowContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) table: &'a str,
    pub(crate) is_selected: bool,
    pub(crate) has_details: bool,
}

impl TableRowContext<'_> {
    pub(crate) fn draw(&self, ui: &mut egui::Ui) -> TableRowRender {
        let table_details_id = ui.make_persistent_id(("codex_tbl_details", self.table));
        let mut collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), table_details_id, true);
        let is_open = collapsing.is_open();
        let (response, chevron_clicked) = self.draw_row(ui, is_open);
        let is_context_menu = is_context_menu_triggered(&response, ui);
        let actions = table_context_menu(ui, &response, self.theme);
        let should_select = !chevron_clicked
            && ((response.clicked() && !is_context_menu) || actions.iter().any(|action| action.selects_table()));

        if chevron_clicked && self.has_details {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
        }

        TableRowRender {
            is_open: collapsing.is_open(),
            should_select,
            actions,
        }
    }

    fn draw_row(&self, ui: &mut egui::Ui, is_open: bool) -> (egui::Response, bool) {
        draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 4,
                is_expandable: self.has_details,
                is_expanded: is_open,
                icon: Icon::Table2,
                icon_color: if self.is_selected {
                    self.theme.accent
                } else {
                    self.theme.text_secondary
                },
                label: self.table,
                is_selected: self.is_selected,
                is_dimmed: false,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: None,
                detail_text: None,
            },
        )
    }
}

fn table_context_menu(ui: &mut egui::Ui, response: &egui::Response, theme: DbProTheme) -> Vec<TableRowAction> {
    let mut actions = Vec::new();
    context_action_menu(ui, response, theme, |ui, close_menu| {
        let mut menu = TableMenu {
            close_menu,
            actions: &mut actions,
            theme,
        };
        menu.add_view_actions(ui);
        ui.separator();
        menu.add_ddl_workbench_actions(ui);
        ui.separator();
        menu.add_sql_actions(ui);
        ui.separator();
        menu.add_copy_actions(ui);
        ui.separator();
        menu.add_refresh_action(ui);
    });
    actions
}

struct TableMenu<'a> {
    close_menu: &'a mut bool,
    actions: &'a mut Vec<TableRowAction>,
    theme: DbProTheme,
}

struct TableMenuItem {
    action: TableRowAction,
    icon: Icon,
    label: &'static str,
}

impl TableMenuItem {
    fn new(action: TableRowAction, icon: Icon, label: &'static str) -> Self {
        Self { action, icon, label }
    }
}

impl TableMenu<'_> {
    fn add_view_actions(&mut self, ui: &mut egui::Ui) {
        self.add_item(
            TableMenuItem::new(TableRowAction::OpenData, Icon::Table2, "View Data"),
            ui,
        );
        self.add_item(
            TableMenuItem::new(TableRowAction::OpenStructure, Icon::Columns3, "View Structure"),
            ui,
        );
    }

    fn add_ddl_workbench_actions(&mut self, ui: &mut egui::Ui) {
        self.add_item(
            TableMenuItem::new(TableRowAction::OpenModifyTable, Icon::PenSquare, "Modify Table (Alter)..."),
            ui,
        );
        self.add_item(
            TableMenuItem::new(TableRowAction::DropTable, Icon::Trash2, "Drop Table..."),
            ui,
        );
    }

    fn add_sql_actions(&mut self, ui: &mut egui::Ui) {
        self.add_item(
            TableMenuItem::new(TableRowAction::OpenQuery, Icon::Play, "Generate SQL: SELECT *"),
            ui,
        );
        self.add_item(
            TableMenuItem::new(TableRowAction::GenerateInsert, Icon::Plus, "Generate SQL: INSERT"),
            ui,
        );
        self.add_item(
            TableMenuItem::new(TableRowAction::GenerateUpdate, Icon::Pencil, "Generate SQL: UPDATE"),
            ui,
        );
        self.add_item(
            TableMenuItem::new(TableRowAction::GenerateDelete, Icon::Trash2, "Generate SQL: DELETE"),
            ui,
        );
        self.add_item(
            TableMenuItem::new(TableRowAction::OpenDdl, Icon::Code2, "View DDL / CREATE Script"),
            ui,
        );
    }

    fn add_copy_actions(&mut self, ui: &mut egui::Ui) {
        self.add_item(
            TableMenuItem::new(TableRowAction::CopyQualifiedName, Icon::Copy, "Copy Qualified Name"),
            ui,
        );
        self.add_item(
            TableMenuItem::new(TableRowAction::CopyName, Icon::Copy, "Copy Table Name"),
            ui,
        );
        self.add_item(
            TableMenuItem::new(TableRowAction::AskAgent, Icon::Bot, "Ask Agent about table"),
            ui,
        );
    }

    fn add_refresh_action(&mut self, ui: &mut egui::Ui) {
        if ctx_menu_item(
            ui,
            Some(Icon::RotateCcw),
            "Refresh Schema",
            Some("F5"),
            self.theme.text_primary,
            self.theme,
        )
        .clicked()
        {
            self.actions.push(TableRowAction::RefreshSchema);
            *self.close_menu = true;
        }
    }

    fn add_item(&mut self, item: TableMenuItem, ui: &mut egui::Ui) {
        if ctx_menu_item(
            ui,
            Some(item.icon),
            item.label,
            None,
            self.theme.text_primary,
            self.theme,
        )
        .clicked()
        {
            self.actions.push(item.action);
            *self.close_menu = true;
        }
    }
}
