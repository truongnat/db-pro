use super::ide_workspace::WorkspaceFileNode;
use super::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FilesTreeAction {
    NewSql,
    NewFolder,
    ToggleDirectory(String),
    CreateSql(String),
    Delete(String),
    OpenFile(String),
    AddContext(String),
    FindReferences(String),
}

pub(super) struct FilesTreeContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) file_count: usize,
    pub(super) active_file_path: Option<&'a str>,
    pub(super) tree: &'a [WorkspaceFileNode],
    pub(super) expanded: &'a BTreeSet<String>,
}

impl FilesTreeContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<FilesTreeAction> {
        let mut actions = self.draw_toolbar(ui);
        ui.add_space(6.0);
        if let Some(path) = self.active_file_path {
            ui.label(RichText::new(path).small().monospace().color(self.theme.text_muted));
            ui.add_space(4.0);
        }
        section_label(ui, format!("FILES · {}", self.file_count), self.theme);
        ui.add_space(6.0);
        for node in self.tree {
            actions.extend(self.draw_node(ui, node, 0));
        }
        actions
    }

    fn draw_toolbar(&self, ui: &mut egui::Ui) -> Vec<FilesTreeAction> {
        let mut actions = Vec::new();
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("New SQL")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesTreeAction::NewSql);
            }
            if Button::new(self.theme)
                .text("New folder")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesTreeAction::NewFolder);
            }
        });
        actions
    }

    fn draw_node(&self, ui: &mut egui::Ui, node: &WorkspaceFileNode, depth: usize) -> Vec<FilesTreeAction> {
        let mut actions = Vec::new();
        let indent = depth as f32 * 12.0;
        ui.horizontal(|ui| {
            ui.add_space(indent);
            if node.is_dir {
                actions.extend(self.draw_directory_node(ui, node));
            } else {
                actions.extend(self.draw_file_node(ui, node));
            }
        });
        if node.is_dir && self.expanded.contains(&node.relative_path) {
            for child in &node.children {
                actions.extend(self.draw_node(ui, child, depth + 1));
            }
        }
        actions
    }

    fn draw_directory_node(&self, ui: &mut egui::Ui, node: &WorkspaceFileNode) -> Vec<FilesTreeAction> {
        let mut actions = Vec::new();
        let expanded = self.expanded.contains(&node.relative_path);
        let response = sidebar_item(
            ui,
            if expanded {
                Icon::ChevronDown
            } else {
                Icon::ChevronRight
            },
            &node.name,
            false,
            self.theme,
        );
        let (create_sql, delete_node) = self.directory_menu(ui, &response);
        if response.clicked() {
            actions.push(FilesTreeAction::ToggleDirectory(node.relative_path.clone()));
        }
        if create_sql {
            actions.push(FilesTreeAction::CreateSql(node.relative_path.clone()));
        }
        if delete_node {
            actions.push(FilesTreeAction::Delete(node.relative_path.clone()));
        }
        actions
    }

    fn directory_menu(&self, ui: &mut egui::Ui, response: &egui::Response) -> (bool, bool) {
        let mut create_sql = false;
        let mut delete_node = false;
        context_action_menu(ui, response, self.theme, |ui, close_menu| {
            if ctx_menu_item(
                ui,
                Some(Icon::FileCode2),
                "New SQL here",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                create_sql = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Trash2),
                "Delete folder",
                None,
                self.theme.danger,
                self.theme,
            )
            .clicked()
            {
                delete_node = true;
                *close_menu = true;
            }
        });
        (create_sql, delete_node)
    }

    fn draw_file_node(&self, ui: &mut egui::Ui, node: &WorkspaceFileNode) -> Vec<FilesTreeAction> {
        let mut actions = Vec::new();
        let icon = if node.name.ends_with(".sql") {
            Icon::FileCode2
        } else {
            Icon::FileText
        };
        let selected = self
            .active_file_path
            .is_some_and(|path| path == node.absolute_path.to_string_lossy());
        let response = sidebar_item(ui, icon, &node.name, selected, self.theme).on_hover_text(&node.relative_path);
        let (mut open_file, add_context, delete_node, find_refs) = self.file_menu(ui, &response);
        if response.clicked() && node.name.ends_with(".sql") {
            open_file = true;
        }
        if open_file && node.name.ends_with(".sql") {
            actions.push(FilesTreeAction::OpenFile(node.relative_path.clone()));
        }
        if add_context {
            actions.push(FilesTreeAction::AddContext(
                node.absolute_path.to_string_lossy().into_owned(),
            ));
        }
        if find_refs {
            actions.push(FilesTreeAction::FindReferences(
                node.name.trim_end_matches(".sql").to_owned(),
            ));
        }
        if delete_node {
            actions.push(FilesTreeAction::Delete(node.relative_path.clone()));
        }
        actions
    }

    fn file_menu(&self, ui: &mut egui::Ui, response: &egui::Response) -> (bool, bool, bool, bool) {
        let mut open_file = false;
        let mut add_context = false;
        let mut delete_node = false;
        let mut find_refs = false;
        context_action_menu(ui, response, self.theme, |ui, close_menu| {
            if ctx_menu_item(
                ui,
                Some(Icon::FileCode2),
                "Open",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                open_file = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Plus),
                "Add to Agent context",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                add_context = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Search),
                "Find references",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                find_refs = true;
                *close_menu = true;
            }
            if ctx_menu_item(ui, Some(Icon::Trash2), "Delete", None, self.theme.danger, self.theme).clicked() {
                delete_node = true;
                *close_menu = true;
            }
        });
        (open_file, add_context, delete_node, find_refs)
    }
}
