//! Presentation and intent mapping for schema-object folders in the Explorer.

use super::explorer_navigation::SchemaObjectActivation;
use super::explorer_schema_object_row_view::{SchemaObjectRowAction, SchemaObjectRowContext};
use super::explorer_tree::{draw_category_folder, CategoryFolder};
use super::{
    DbProTheme, SchemaExplorerState, SchemaObjectSelection, UiFunctionSummary, UiTriggerSummary, UiViewSummary,
};
use eframe::egui;
use lucide_icons::Icon;

pub(super) enum SchemaObjectFolderAction {
    Open(SchemaObjectActivation),
    OpenQuery(String),
    ModifyView(UiViewSummary),
    DropObject { schema: String, name: String, kind: String },
    CopyName(String),
}

pub(super) struct SchemaObjectFoldersView<'a> {
    theme: DbProTheme,
    explorer: &'a SchemaExplorerState,
}

impl<'a> SchemaObjectFoldersView<'a> {
    pub(super) fn new(theme: DbProTheme, explorer: &'a SchemaExplorerState) -> Self {
        Self { theme, explorer }
    }

    pub(super) fn draw_views(
        &self,
        ui: &mut egui::Ui,
        schema: &str,
        search_query: &str,
        _count: usize,
    ) -> Vec<SchemaObjectFolderAction> {
        let views: Vec<_> = self
            .explorer
            .filter_by_schema(&self.explorer.schema.views, schema, |view| &view.schema)
            .into_iter()
            .filter(|v| search_query.is_empty() || v.name.to_ascii_lowercase().contains(search_query))
            .collect();
        let count = views.len();
        let mut actions = Vec::new();
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
                empty_label: Some(if search_query.is_empty() {
                    "No views in schema"
                } else {
                    "No matching views"
                }),
            },
            |ui| {
                for view in &views {
                    actions.extend(self.draw_view_row(ui, view));
                }
            },
        );
        actions
    }

    pub(super) fn draw_functions(
        &self,
        ui: &mut egui::Ui,
        schema: &str,
        search_query: &str,
        _count: usize,
    ) -> Vec<SchemaObjectFolderAction> {
        let functions: Vec<_> = self
            .explorer
            .filter_by_schema(&self.explorer.schema.functions, schema, |function| &function.schema)
            .into_iter()
            .filter(|f| search_query.is_empty() || f.name.to_ascii_lowercase().contains(search_query))
            .collect();
        let count = functions.len();
        let mut actions = Vec::new();
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
                empty_label: Some(if search_query.is_empty() {
                    "No functions in schema"
                } else {
                    "No matching functions"
                }),
            },
            |ui| {
                for function in &functions {
                    actions.extend(self.draw_function_row(ui, function));
                }
            },
        );
        actions
    }

    pub(super) fn draw_triggers(
        &self,
        ui: &mut egui::Ui,
        schema: &str,
        search_query: &str,
        _count: usize,
    ) -> Vec<SchemaObjectFolderAction> {
        let triggers: Vec<_> = self
            .explorer
            .filter_by_schema(&self.explorer.schema.triggers, schema, |trigger| &trigger.schema)
            .into_iter()
            .filter(|t| search_query.is_empty() || t.name.to_ascii_lowercase().contains(search_query))
            .collect();
        let count = triggers.len();
        let mut actions = Vec::new();
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
                empty_label: Some(if search_query.is_empty() {
                    "No triggers in schema"
                } else {
                    "No matching triggers"
                }),
            },
            |ui| {
                for trigger in &triggers {
                    actions.extend(self.draw_trigger_row(ui, trigger));
                }
            },
        );
        actions
    }

    fn draw_view_row(&self, ui: &mut egui::Ui, view: &UiViewSummary) -> Vec<SchemaObjectFolderAction> {
        let is_selected = matches!(
            self.explorer.selected_schema_object.as_ref(),
            Some(SchemaObjectSelection::View(name)) if name == &view.name
        );
        let from = if view.schema.is_empty() {
            view.name.clone()
        } else {
            format!("{}.{}", view.schema, view.name)
        };
        let view_clone = view.clone();
        let schema = view.schema.clone();
        let name = view.name.clone();
        let actions = SchemaObjectRowContext {
            theme: self.theme,
            label: &view.name,
            icon: Icon::Eye,
            icon_color: self.theme.success,
            is_selected,
            query_label: Some("Select Top 100 (Query)"),
            query_icon: Icon::Play,
            open_label: "Open View",
            open_icon: Icon::Eye,
            modify_label: Some("Modify View (Workbench)..."),
            drop_label: Some("Drop View..."),
            copy_label: "Copy View Name",
        }
        .draw(ui);

        actions
            .into_iter()
            .map(|action| match action {
                SchemaObjectRowAction::Open => SchemaObjectFolderAction::Open(SchemaObjectActivation {
                    selection: SchemaObjectSelection::View(view.name.clone()),
                    schema: view.schema.clone(),
                    name: view.name.clone(),
                    kind: "view".to_owned(),
                }),
                SchemaObjectRowAction::OpenQuery => {
                    SchemaObjectFolderAction::OpenQuery(format!("SELECT *\nFROM {from}\nLIMIT 100;"))
                }
                SchemaObjectRowAction::Modify => SchemaObjectFolderAction::ModifyView(view_clone.clone()),
                SchemaObjectRowAction::Drop => SchemaObjectFolderAction::DropObject {
                    schema: schema.clone(),
                    name: name.clone(),
                    kind: "VIEW".to_owned(),
                },
                SchemaObjectRowAction::CopyName => SchemaObjectFolderAction::CopyName(view.name.clone()),
            })
            .collect()
    }

    fn draw_function_row(&self, ui: &mut egui::Ui, function: &UiFunctionSummary) -> Vec<SchemaObjectFolderAction> {
        let is_selected = matches!(
            self.explorer.selected_schema_object.as_ref(),
            Some(SchemaObjectSelection::Function { name, identity_arguments })
                if name == &function.name && identity_arguments == &function.identity_arguments
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
        self.draw_row(
            ui,
            SchemaObjectRowContext {
                theme: self.theme,
                label: &label,
                icon,
                icon_color: self.theme.code_type,
                is_selected,
                query_label: Some("Open Call in Query"),
                query_icon: Icon::Play,
                open_label: "Open Routine",
                open_icon: Icon::Code2,
                modify_label: None,
                drop_label: None,
                copy_label: "Copy Routine Name",
            },
            SchemaObjectActivation {
                selection: SchemaObjectSelection::Function {
                    name: function.name.clone(),
                    identity_arguments: function.identity_arguments.clone(),
                },
                schema: function.schema.clone(),
                name: function.name.clone(),
                kind: "function".to_owned(),
            },
            Some(format!("SELECT * FROM {}.{}();", function.schema, function.name)),
        )
    }

    fn draw_trigger_row(&self, ui: &mut egui::Ui, trigger: &UiTriggerSummary) -> Vec<SchemaObjectFolderAction> {
        let is_selected = matches!(
            self.explorer.selected_schema_object.as_ref(),
            Some(SchemaObjectSelection::Trigger(name)) if name == &trigger.name
        );
        let label = format!("{} · {}", trigger.name, trigger.event);
        let schema = trigger.schema.clone();
        let name = trigger.name.clone();
        let actions = SchemaObjectRowContext {
            theme: self.theme,
            label: &label,
            icon: Icon::Zap,
            icon_color: self.theme.warning,
            is_selected,
            query_label: None,
            query_icon: Icon::Play,
            open_label: "View Trigger",
            open_icon: Icon::Eye,
            modify_label: None,
            drop_label: Some("Drop Trigger..."),
            copy_label: "Copy Trigger Name",
        }
        .draw(ui);

        actions
            .into_iter()
            .filter_map(|action| match action {
                SchemaObjectRowAction::Open => Some(SchemaObjectFolderAction::Open(SchemaObjectActivation {
                    selection: SchemaObjectSelection::Trigger(trigger.name.clone()),
                    schema: String::new(),
                    name: trigger.name.clone(),
                    kind: "trigger".to_owned(),
                })),
                SchemaObjectRowAction::OpenQuery => None,
                SchemaObjectRowAction::Modify => None,
                SchemaObjectRowAction::Drop => Some(SchemaObjectFolderAction::DropObject {
                    schema: schema.clone(),
                    name: name.clone(),
                    kind: "TRIGGER".to_owned(),
                }),
                SchemaObjectRowAction::CopyName => Some(SchemaObjectFolderAction::CopyName(trigger.name.clone())),
            })
            .collect()
    }

    fn draw_row(
        &self,
        ui: &mut egui::Ui,
        context: SchemaObjectRowContext<'_>,
        activation: SchemaObjectActivation,
        query: Option<String>,
    ) -> Vec<SchemaObjectFolderAction> {
        context
            .draw(ui)
            .into_iter()
            .filter_map(|action| match action {
                SchemaObjectRowAction::Open => Some(SchemaObjectFolderAction::Open(activation.clone())),
                SchemaObjectRowAction::OpenQuery => query.clone().map(SchemaObjectFolderAction::OpenQuery),
                SchemaObjectRowAction::Modify => None,
                SchemaObjectRowAction::Drop => None,
                SchemaObjectRowAction::CopyName => Some(SchemaObjectFolderAction::CopyName(activation.name.clone())),
            })
            .collect()
    }
}
