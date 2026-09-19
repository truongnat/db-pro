//! Category folders of the Codex navigator: Views, Functions and Triggers.

use super::explorer_tree::{draw_category_folder, draw_codex_tree_row, CategoryFolder, CodexTreeRow};
use super::*;
use lucide_icons::Icon;

impl DbProApp {
    /// Views folder — only materialises the schema-scoped list when expanded.
    pub(super) fn draw_dbeaver_views_folder_lazy(&mut self, ui: &mut egui::Ui, schema: &str, count: usize) {
        let theme = self.theme;
        let folder_id = ui.make_persistent_id(("codex_views_folder", schema));
        draw_category_folder(
            ui,
            &theme,
            CategoryFolder {
                depth: 3,
                id: folder_id,
                icon: Icon::Eye,
                icon_color: theme.success,
                label: "Views",
                count,
                empty_label: Some("No views in schema"),
            },
            |ui| {
                let views = self.filter_by_schema(&self.schema.views, schema, |v| &v.schema);
                for view in &views {
                    self.draw_view_row(ui, view, &theme);
                }
            },
        );
    }

    /// Functions folder — deferred filter until the folder is open.
    pub(super) fn draw_dbeaver_functions_folder_lazy(&mut self, ui: &mut egui::Ui, schema: &str, count: usize) {
        let theme = self.theme;
        let folder_id = ui.make_persistent_id(("codex_functions_folder", schema));
        draw_category_folder(
            ui,
            &theme,
            CategoryFolder {
                depth: 3,
                id: folder_id,
                icon: Icon::Code2,
                icon_color: theme.code_type,
                label: "Functions",
                count,
                empty_label: Some("No functions in schema"),
            },
            |ui| {
                let functions = self.filter_by_schema(&self.schema.functions, schema, |f| &f.schema);
                for function in &functions {
                    self.draw_function_row(ui, function, &theme);
                }
            },
        );
    }

    /// Triggers folder — deferred filter until the folder is open.
    pub(super) fn draw_dbeaver_triggers_folder_lazy(&mut self, ui: &mut egui::Ui, schema: &str, count: usize) {
        let theme = self.theme;
        let folder_id = ui.make_persistent_id(("codex_triggers_folder", schema));
        draw_category_folder(
            ui,
            &theme,
            CategoryFolder {
                depth: 3,
                id: folder_id,
                icon: Icon::Zap,
                icon_color: theme.warning,
                label: "Triggers",
                count,
                empty_label: Some("No triggers in schema"),
            },
            |ui| {
                let triggers = self.filter_by_schema(&self.schema.triggers, schema, |t| &t.schema);
                for trigger in &triggers {
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
                icon_color: if is_selected { theme.accent } else { theme.success },
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
            let from = if view.schema.is_empty() {
                view.name.clone()
            } else {
                format!("{}.{}", view.schema, view.name)
            };
            self.set_active_query_text(format!("SELECT *\nFROM {from}\nLIMIT 100;"));
            self.workspace.active_tab = WorkspaceTab::Query;
        }
    }

    /// One function row: routines and procedures share a row, differing by icon.
    fn draw_function_row(&mut self, ui: &mut egui::Ui, function: &UiFunctionSummary, theme: &DbProTheme) {
        let is_selected = matches!(
            self.selected_schema_object.as_ref(),
            Some(SchemaObjectSelection::Function {
                name,
                identity_arguments
            }) if name == &function.name && identity_arguments == &function.identity_arguments
        );
        let icon = if function.routine_type.eq_ignore_ascii_case("procedure") {
            Icon::GitBranch
        } else {
            Icon::Code2
        };
        let label = if function.identity_arguments.is_empty() {
            format!("{} · {}", function.name, function.routine_type)
        } else {
            format!(
                "{}({}) · {}",
                function.name, function.identity_arguments, function.routine_type
            )
        };

        let (response, _) = draw_codex_tree_row(
            ui,
            theme,
            CodexTreeRow {
                depth: 4,
                is_expandable: false,
                is_expanded: false,
                icon,
                icon_color: if is_selected { theme.accent } else { theme.code_type },
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
                SchemaObjectSelection::Function {
                    name: function.name.clone(),
                    identity_arguments: function.identity_arguments.clone(),
                },
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
            self.set_active_query_text(format!("SELECT * FROM {}.{}();", function.schema, function.name));
            self.workspace.active_tab = WorkspaceTab::Query;
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
                icon_color: if is_selected { theme.accent } else { theme.warning },
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
        let mut open_trigger = false;
        let mut copy_name = false;
        let theme_copy = *theme;
        context_action_menu(ui, &response, theme_copy, |ui, close_menu| {
            if ctx_menu_item(
                ui,
                Some(Icon::Eye),
                "View Trigger",
                None,
                theme_copy.text_primary,
                theme_copy,
            )
            .clicked()
            {
                open_trigger = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Copy),
                "Copy Trigger Name",
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

        if (response.clicked() && !is_ctx) || open_trigger {
            self.open_schema_object(
                SchemaObjectSelection::Trigger(trigger.name.clone()),
                "",
                &trigger.name,
                "trigger",
            );
        }
        if copy_name {
            ui.output_mut(|o| o.copied_text = trigger.name.clone());
            self.runtime_message = format!("Copied `{}` to clipboard", trigger.name);
        }
    }

    /// Activates a schema object (view / function / trigger) in the workspace.
    /// `schema` is empty for objects that are not schema-qualified (triggers).
    pub(super) fn open_schema_object(
        &mut self,
        selection: SchemaObjectSelection,
        schema: &str,
        name: &str,
        kind: &str,
    ) {
        self.selected_schema_object = Some(selection.clone());
        self.schema_object_view = SchemaObjectView::Definition;
        self.selected_table = None;
        self.table_state.table_info = None;
        self.table_ddl = None;
        self.table_state.table_view = TableView::Ddl;
        self.workspace.active_tab = WorkspaceTab::SchemaObject;
        self.routine_drop_confirm = false;
        self.routine_ddl_preview = None;
        if let SchemaObjectSelection::Function {
            name: fn_name,
            identity_arguments,
        } = &selection
        {
            if let Some(function) = self
                .schema
                .functions
                .iter()
                .find(|f| &f.name == fn_name && &f.identity_arguments == identity_arguments)
                .cloned()
            {
                self.sync_routine_workbench_from(&function);
            }
        }
        self.runtime_message = if schema.is_empty() {
            format!("Opened {kind} {name}")
        } else {
            format!("Opened {kind} {schema}.{name}")
        };
    }
}
