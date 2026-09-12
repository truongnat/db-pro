//! Category folders of the Codex navigator: Views, Functions and Triggers.

use super::explorer_tree::{draw_category_folder, draw_codex_tree_row, CategoryFolder, CodexTreeRow};
use super::*;
use egui::Color32;
use lucide_icons::Icon;

impl DbProApp {
    /// Views folder in Codex tree.
    pub(super) fn draw_dbeaver_views_folder(&mut self, ui: &mut egui::Ui, views: &[UiViewSummary]) {
        let theme = self.theme;
        let folder_id = ui.make_persistent_id("codex_views_folder");
        draw_category_folder(
            ui,
            &theme,
            CategoryFolder {
                depth: 3,
                id: folder_id,
                icon: Icon::Eye,
                icon_color: Color32::from_rgb(5, 150, 105), // emerald green
                label: "Views",
                count: views.len(),
                empty_label: Some("No views in schema"),
            },
            |ui| {
                for view in views {
                    self.draw_view_row(ui, view, &theme);
                }
            },
        );
    }

    /// Functions folder in Codex tree.
    pub(super) fn draw_dbeaver_functions_folder(&mut self, ui: &mut egui::Ui, functions: &[UiFunctionSummary]) {
        let theme = self.theme;
        let folder_id = ui.make_persistent_id("codex_functions_folder");
        draw_category_folder(
            ui,
            &theme,
            CategoryFolder {
                depth: 3,
                id: folder_id,
                icon: Icon::Code2,
                icon_color: Color32::from_rgb(124, 58, 237), // purple
                label: "Functions",
                count: functions.len(),
                empty_label: Some("No functions in schema"),
            },
            |ui| {
                for function in functions {
                    self.draw_function_row(ui, function, &theme);
                }
            },
        );
    }

    /// Triggers folder in Codex tree.
    pub(super) fn draw_dbeaver_triggers_folder(&mut self, ui: &mut egui::Ui, triggers: &[UiTriggerSummary]) {
        let theme = self.theme;
        let folder_id = ui.make_persistent_id("codex_triggers_folder");
        draw_category_folder(
            ui,
            &theme,
            CategoryFolder {
                depth: 3,
                id: folder_id,
                icon: Icon::Zap,
                icon_color: Color32::from_rgb(234, 88, 12), // amber-orange
                label: "Triggers",
                count: triggers.len(),
                empty_label: Some("No triggers in schema"),
            },
            |ui| {
                for trigger in triggers {
                    self.draw_trigger_row(ui, trigger, &theme);
                }
            },
        );
    }

    /// One view row: selection state, "Open in Query" menu and activation.
    fn draw_view_row(&mut self, ui: &mut egui::Ui, view: &UiViewSummary, theme: &DbProTheme) {
        let is_selected = matches!(
            self.selected_schema_object.as_ref(),
            Some(SchemaObjectSelection::View(s)) if s == &view.name
        );

        let (response, _) = draw_codex_tree_row(
            ui,
            theme,
            CodexTreeRow {
                depth: 4,
                is_expandable: false,
                is_expanded: false,
                icon: Icon::Eye,
                icon_color: if is_selected {
                    theme.accent
                } else {
                    Color32::from_rgb(5, 150, 105)
                },
                label: &view.name,
                is_selected,
                is_dimmed: false,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: None,
                detail_text: None,
            },
        );

        let is_ctx = is_context_menu_triggered(&response, ui);
        let mut open_query = false;
        let mut copy_name = false;
        let theme_copy = *theme;
        context_action_menu(ui, &response, theme_copy, |ui, close_menu| {
            if ctx_menu_item(
                ui,
                Some(Icon::Play),
                "Select Top 100 (Query)",
                None,
                theme_copy.text_primary,
                theme_copy,
            )
            .clicked()
            {
                open_query = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Copy),
                "Copy View Name",
                None,
                theme_copy.text_primary,
                theme_copy,
            )
            .clicked()
            {
                copy_name = true;
                *close_menu = true;
            }
        });

        if response.clicked() && !is_ctx {
            self.open_schema_object(
                SchemaObjectSelection::View(view.name.clone()),
                &view.schema,
                &view.name,
                "view",
            );
        }
        if copy_name {
            ui.output_mut(|o| o.copied_text = view.name.clone());
            self.runtime_message = format!("Copied `{}` to clipboard", view.name);
        }
        if open_query {
            self.query_text = format!("SELECT *\nFROM {}\nLIMIT 100;", view.name);
            self.active_tab = WorkspaceTab::Query;
        }
    }

    /// One function row: routines and procedures share a row, differing by icon.
    fn draw_function_row(&mut self, ui: &mut egui::Ui, function: &UiFunctionSummary, theme: &DbProTheme) {
        let is_selected = matches!(
            self.selected_schema_object.as_ref(),
            Some(SchemaObjectSelection::Function(s)) if s == &function.name
        );
        let icon = if function.routine_type.eq_ignore_ascii_case("procedure") {
            Icon::GitBranch
        } else {
            Icon::Code2
        };
        let label = format!("{} · {}", function.name, function.routine_type);

        let (response, _) = draw_codex_tree_row(
            ui,
            theme,
            CodexTreeRow {
                depth: 4,
                is_expandable: false,
                is_expanded: false,
                icon,
                icon_color: if is_selected {
                    theme.accent
                } else {
                    Color32::from_rgb(124, 58, 237)
                },
                label: &label,
                is_selected,
                is_dimmed: false,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: None,
                detail_text: None,
            },
        );

        let is_ctx = is_context_menu_triggered(&response, ui);
        let mut open_query = false;
        let mut copy_name = false;
        let theme_copy = *theme;
        context_action_menu(ui, &response, theme_copy, |ui, close_menu| {
            if ctx_menu_item(
                ui,
                Some(Icon::Play),
                "Open Call in Query",
                None,
                theme_copy.text_primary,
                theme_copy,
            )
            .clicked()
            {
                open_query = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Copy),
                "Copy Routine Name",
                None,
                theme_copy.text_primary,
                theme_copy,
            )
            .clicked()
            {
                copy_name = true;
                *close_menu = true;
            }
        });

        if response.clicked() && !is_ctx {
            self.open_schema_object(
                SchemaObjectSelection::Function(function.name.clone()),
                &function.schema,
                &function.name,
                "function",
            );
        }
        if copy_name {
            ui.output_mut(|o| o.copied_text = function.name.clone());
            self.runtime_message = format!("Copied `{}` to clipboard", function.name);
        }
        if open_query {
            self.query_text = format!("SELECT * FROM {}.{}();", function.schema, function.name);
            self.active_tab = WorkspaceTab::Query;
        }
    }

    /// One trigger row. Triggers carry no schema, so the message omits it.
    fn draw_trigger_row(&mut self, ui: &mut egui::Ui, trigger: &UiTriggerSummary, theme: &DbProTheme) {
        let is_selected = matches!(
            self.selected_schema_object.as_ref(),
            Some(SchemaObjectSelection::Trigger(s)) if s == &trigger.name
        );
        let label = format!("{} · {}", trigger.name, trigger.event);

        let (response, _) = draw_codex_tree_row(
            ui,
            theme,
            CodexTreeRow {
                depth: 4,
                is_expandable: false,
                is_expanded: false,
                icon: Icon::Zap,
                icon_color: if is_selected {
                    theme.accent
                } else {
                    Color32::from_rgb(234, 88, 12)
                },
                label: &label,
                is_selected,
                is_dimmed: false,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: None,
                detail_text: None,
            },
        );

        if response.clicked() {
            self.open_schema_object(
                SchemaObjectSelection::Trigger(trigger.name.clone()),
                "",
                &trigger.name,
                "trigger",
            );
        }
    }

    /// Activates a schema object (view / function / trigger) in the workspace.
    /// `schema` is empty for objects that are not schema-qualified (triggers).
    fn open_schema_object(&mut self, selection: SchemaObjectSelection, schema: &str, name: &str, kind: &str) {
        self.selected_schema_object = Some(selection);
        self.schema_object_view = SchemaObjectView::Definition;
        self.selected_table = None;
        self.table_info = None;
        self.table_ddl = None;
        self.table_view = TableView::Ddl;
        self.active_tab = WorkspaceTab::SchemaObject;
        self.runtime_message = if schema.is_empty() {
            format!("Opened {kind} {name}")
        } else {
            format!("Opened {kind} {schema}.{name}")
        };
    }
}
