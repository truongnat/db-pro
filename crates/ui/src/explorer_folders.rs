//! Category folders of the Codex navigator: Views, Functions and Triggers.

use super::explorer_schema_object_row_view::{SchemaObjectRowAction, SchemaObjectRowContext};
use super::explorer_tree::{draw_category_folder, CategoryFolder};
use super::*;
use lucide_icons::Icon;

struct SchemaObjectActionInput<'a> {
    selection: SchemaObjectSelection,
    schema: &'a str,
    name: &'a str,
    kind: &'a str,
    query: Option<String>,
}

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
                let views = self
                    .schema
                    .explorer
                    .filter_by_schema(&self.schema.explorer.schema.views, schema, |view| &view.schema);
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
                let functions =
                    self.schema
                        .explorer
                        .filter_by_schema(&self.schema.explorer.schema.functions, schema, |function| {
                            &function.schema
                        });
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
                let triggers =
                    self.schema
                        .explorer
                        .filter_by_schema(&self.schema.explorer.schema.triggers, schema, |trigger| &trigger.schema);
                for trigger in &triggers {
                    self.draw_trigger_row(ui, trigger, &theme);
                }
            },
        );
    }

    /// One view row: selection state, "Open in Query" menu and activation.
    fn draw_view_row(&mut self, ui: &mut egui::Ui, view: &UiViewSummary, theme: &DbProTheme) {
        let is_selected = matches!(
            self.schema.explorer.selected_schema_object.as_ref(),
            Some(SchemaObjectSelection::View(s)) if s == &view.name
        );
        let from = if view.schema.is_empty() {
            view.name.clone()
        } else {
            format!("{}.{}", view.schema, view.name)
        };
        self.draw_schema_object_row(
            ui,
            SchemaObjectRowContext {
                theme: *theme,
                label: &view.name,
                icon: Icon::Eye,
                icon_color: theme.success,
                is_selected,
                query_label: Some("Select Top 100 (Query)"),
                query_icon: Icon::Play,
                open_label: "Open View",
                open_icon: Icon::Eye,
                copy_label: "Copy View Name",
            },
            SchemaObjectActionInput {
                selection: SchemaObjectSelection::View(view.name.clone()),
                schema: &view.schema,
                name: &view.name,
                kind: "view",
                query: Some(format!("SELECT *\nFROM {from}\nLIMIT 100;")),
            },
        );
    }

    /// One function row: routines and procedures share a row, differing by icon.
    fn draw_function_row(&mut self, ui: &mut egui::Ui, function: &UiFunctionSummary, theme: &DbProTheme) {
        let is_selected = matches!(
            self.schema.explorer.selected_schema_object.as_ref(),
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
        self.draw_schema_object_row(
            ui,
            SchemaObjectRowContext {
                theme: *theme,
                label: &label,
                icon,
                icon_color: theme.code_type,
                is_selected,
                query_label: Some("Open Call in Query"),
                query_icon: Icon::Play,
                open_label: "Open Routine",
                open_icon: Icon::Code2,
                copy_label: "Copy Routine Name",
            },
            SchemaObjectActionInput {
                selection: SchemaObjectSelection::Function {
                    name: function.name.clone(),
                    identity_arguments: function.identity_arguments.clone(),
                },
                schema: &function.schema,
                name: &function.name,
                kind: "function",
                query: Some(format!("SELECT * FROM {}.{}();", function.schema, function.name)),
            },
        );
    }

    /// One trigger row. Triggers carry no schema, so the message omits it.
    fn draw_trigger_row(&mut self, ui: &mut egui::Ui, trigger: &UiTriggerSummary, theme: &DbProTheme) {
        let is_selected = matches!(
            self.schema.explorer.selected_schema_object.as_ref(),
            Some(SchemaObjectSelection::Trigger(s)) if s == &trigger.name
        );
        let label = format!("{} · {}", trigger.name, trigger.event);
        self.draw_schema_object_row(
            ui,
            SchemaObjectRowContext {
                theme: *theme,
                label: &label,
                icon: Icon::Zap,
                icon_color: theme.warning,
                is_selected,
                query_label: None,
                query_icon: Icon::Play,
                open_label: "View Trigger",
                open_icon: Icon::Eye,
                copy_label: "Copy Trigger Name",
            },
            SchemaObjectActionInput {
                selection: SchemaObjectSelection::Trigger(trigger.name.clone()),
                schema: "",
                name: &trigger.name,
                kind: "trigger",
                query: None,
            },
        );
    }

    fn draw_schema_object_row(
        &mut self,
        ui: &mut egui::Ui,
        context: SchemaObjectRowContext<'_>,
        input: SchemaObjectActionInput<'_>,
    ) {
        for action in context.draw(ui) {
            self.apply_schema_object_row_action(action, &input, ui);
        }
    }

    fn apply_schema_object_row_action(
        &mut self,
        action: SchemaObjectRowAction,
        input: &SchemaObjectActionInput<'_>,
        ui: &mut egui::Ui,
    ) {
        match action {
            SchemaObjectRowAction::Open => {
                self.open_schema_object(input.selection.clone(), input.schema, input.name, input.kind);
            }
            SchemaObjectRowAction::OpenQuery => {
                if let Some(query) = input.query.as_ref() {
                    self.set_active_query_text(query.clone());
                    self.workspace.active_tab = WorkspaceTab::Query;
                }
            }
            SchemaObjectRowAction::CopyName => {
                ui.output_mut(|output| output.copied_text = input.name.to_owned());
                self.feedback.runtime_message = format!("Copied `{}` to clipboard", input.name);
            }
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
        self.schema.explorer.selected_schema_object = Some(selection.clone());
        self.schema.explorer.schema_object_view = SchemaObjectView::Definition;
        self.schema.explorer.selected_table = None;
        self.table.state.table_info = None;
        self.table.state.table_ddl = None;
        self.table.state.table_view = TableView::Ddl;
        self.workspace.active_tab = WorkspaceTab::SchemaObject;
        self.management.routine.routine_drop_confirm = false;
        self.management.routine.routine_ddl_preview = None;
        if let SchemaObjectSelection::Function {
            name: fn_name,
            identity_arguments,
        } = &selection
        {
            if let Some(function) = self
                .schema
                .explorer
                .schema
                .functions
                .iter()
                .find(|f| &f.name == fn_name && &f.identity_arguments == identity_arguments)
                .cloned()
            {
                self.sync_routine_workbench_from(&function);
            }
        }
        self.feedback.runtime_message = if schema.is_empty() {
            format!("Opened {kind} {name}")
        } else {
            format!("Opened {kind} {schema}.{name}")
        };
    }
}
