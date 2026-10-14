// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
//! Presentation and intent mapping for schema-object folders in the Explorer.

use super::explorer_navigation::SchemaObjectActivation;
use super::explorer_schema_object_row_view::{SchemaObjectRowAction, SchemaObjectRowContext};
use super::explorer_tree::{draw_category_folder, CategoryFolder};
use super::{
    context_action_menu, ctx_menu_item, DbProTheme, SchemaExplorerState, SchemaObjectSelection, UiFunctionSummary,
    UiTriggerSummary, UiViewSummary,
};
use crate::components::Separator;
use eframe::egui;
use lucide_icons::Icon;

pub(super) enum SchemaObjectFolderAction {
    Open(SchemaObjectActivation),
    OpenQuery(String),
    ModifyView(UiViewSummary),
    DropObject { schema: String, name: String, kind: String },
    ToggleTrigger { schema: String, table: String, name: String, enable: bool },
    CopyName(String),
    RefreshSchema,
}

pub(super) struct SchemaObjectFoldersView<'a> {
    theme: DbProTheme,
    explorer: &'a SchemaExplorerState,
    trigger_toggle_unavailable_reason: Option<String>,
}

impl<'a> SchemaObjectFoldersView<'a> {
    pub(super) fn new(
        theme: DbProTheme,
        explorer: &'a SchemaExplorerState,
        trigger_toggle_unavailable_reason: Option<String>,
    ) -> Self {
        Self {
            theme,
            explorer,
            trigger_toggle_unavailable_reason,
        }
    }

    /// Shared schema/search predicate — mirrors the old `filter_by_schema`
    /// semantics without materialising a `Vec` before the folder-open check.
    fn object_in_scope(&self, schema: &str, item_schema: &str, name: &str, search_query: &str) -> bool {
        let schema_matches = self.explorer.schema.schemas.is_empty() || schema.is_empty() || item_schema == schema;
        schema_matches && (search_query.is_empty() || self.explorer.explorer_filter.name_matches(name, search_query))
    }

    pub(super) fn draw_views(
        &self,
        ui: &mut egui::Ui,
        schema: &str,
        search_query: &str,
    ) -> Vec<SchemaObjectFolderAction> {
        let in_scope = |view: &&UiViewSummary| self.object_in_scope(schema, &view.schema, &view.name, search_query);
        let count = self.explorer.schema.views.iter().filter(in_scope).count();
        let mut actions = Vec::new();
        let theme = self.theme;
        let folder_id = ui.make_persistent_id(("codex_views_folder", schema));
        let folder_response = draw_category_folder(
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
                for view in self.explorer.schema.views.iter().filter(in_scope) {
                    actions.extend(self.draw_view_row(ui, view));
                }
            },
        );
        self.draw_folder_menu(ui, &folder_response, schema, &mut actions);
        actions
    }

    pub(super) fn draw_functions(
        &self,
        ui: &mut egui::Ui,
        schema: &str,
        search_query: &str,
    ) -> Vec<SchemaObjectFolderAction> {
        let in_scope = |function: &&UiFunctionSummary| {
            self.object_in_scope(schema, &function.schema, &function.name, search_query)
        };
        let count = self.explorer.schema.functions.iter().filter(in_scope).count();
        let mut actions = Vec::new();
        let theme = self.theme;
        let folder_id = ui.make_persistent_id(("codex_functions_folder", schema));
        let folder_response = draw_category_folder(
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
                for function in self.explorer.schema.functions.iter().filter(in_scope) {
                    actions.extend(self.draw_function_row(ui, function));
                }
            },
        );
        self.draw_folder_menu(ui, &folder_response, schema, &mut actions);
        actions
    }

    pub(super) fn draw_triggers(
        &self,
        ui: &mut egui::Ui,
        schema: &str,
        search_query: &str,
    ) -> Vec<SchemaObjectFolderAction> {
        let in_scope =
            |trigger: &&UiTriggerSummary| self.object_in_scope(schema, &trigger.schema, &trigger.name, search_query);
        let count = self.explorer.schema.triggers.iter().filter(in_scope).count();
        let mut actions = Vec::new();
        let theme = self.theme;
        let folder_id = ui.make_persistent_id(("codex_triggers_folder", schema));
        let folder_response = draw_category_folder(
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
                for trigger in self.explorer.schema.triggers.iter().filter(in_scope) {
                    actions.extend(self.draw_trigger_row(ui, trigger));
                }
            },
        );
        self.draw_folder_menu(ui, &folder_response, schema, &mut actions);
        actions
    }

    /// Folder rows share one context menu: schema refresh plus copying the
    /// parent schema name. `context_action_menu` owns trigger detection.
    fn draw_folder_menu(
        &self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        schema: &str,
        actions: &mut Vec<SchemaObjectFolderAction>,
    ) {
        let theme = self.theme;
        let schema = schema.to_owned();
        context_action_menu(ui, response, theme, |ui, close_menu| {
            if ctx_menu_item(
                ui,
                Some(Icon::RefreshCw),
                "Refresh Schema",
                None,
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                actions.push(SchemaObjectFolderAction::RefreshSchema);
                *close_menu = true;
            }
            Separator::horizontal(theme).show(ui);
            if ctx_menu_item(
                ui,
                Some(Icon::Copy),
                "Copy Schema Name",
                None,
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                actions.push(SchemaObjectFolderAction::CopyName(schema));
                *close_menu = true;
            }
        });
    }

    fn draw_view_row(&self, ui: &mut egui::Ui, view: &UiViewSummary) -> Vec<SchemaObjectFolderAction> {
        let is_selected = matches!(
            self.explorer.selected_schema_object.as_ref(),
            Some(SchemaObjectSelection::View(name)) if name == &view.name
        );
        let from = super::result_grid_export::qualified_sql_name(&view.schema, &view.name);
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
            trigger_toggle_label: None,
            trigger_toggle_unavailable_reason: None,
        }
        .draw(ui);

        actions
            .into_iter()
            .filter_map(|action| match action {
                SchemaObjectRowAction::Open => Some(SchemaObjectFolderAction::Open(SchemaObjectActivation {
                    selection: SchemaObjectSelection::View(view.name.clone()),
                    schema: view.schema.clone(),
                    name: view.name.clone(),
                    kind: "view".to_owned(),
                })),
                SchemaObjectRowAction::OpenQuery => {
                    Some(SchemaObjectFolderAction::OpenQuery(format!("SELECT *\nFROM {from}\nLIMIT 100;")))
                }
                SchemaObjectRowAction::Modify => Some(SchemaObjectFolderAction::ModifyView(view_clone.clone())),
                SchemaObjectRowAction::Drop => Some(SchemaObjectFolderAction::DropObject {
                    schema: schema.clone(),
                    name: name.clone(),
                    kind: "VIEW".to_owned(),
                }),
                SchemaObjectRowAction::ToggleTrigger => None,
                SchemaObjectRowAction::CopyName => Some(SchemaObjectFolderAction::CopyName(view.name.clone())),
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
                trigger_toggle_label: None,
                trigger_toggle_unavailable_reason: None,
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
            Some(format!(
                "SELECT * FROM {}();",
                super::result_grid_export::qualified_sql_name(&function.schema, &function.name)
            )),
        )
    }

    fn draw_trigger_row(&self, ui: &mut egui::Ui, trigger: &UiTriggerSummary) -> Vec<SchemaObjectFolderAction> {
        let is_selected = matches!(
            self.explorer.selected_schema_object.as_ref(),
            Some(SchemaObjectSelection::Trigger(name)) if name == &trigger.name
        );
        let label = format!(
            "{} · {} · {}",
            trigger.name,
            trigger.event,
            if trigger.enabled { "Enabled" } else { "Disabled" }
        );
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
            trigger_toggle_label: trigger_toggle_menu_label(
                trigger.enabled,
                self.trigger_toggle_unavailable_reason.is_none(),
            ),
            trigger_toggle_unavailable_reason: self.trigger_toggle_unavailable_reason.as_deref(),
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
                SchemaObjectRowAction::OpenQuery | SchemaObjectRowAction::Modify => None,
                SchemaObjectRowAction::Drop => Some(SchemaObjectFolderAction::DropObject {
                    schema: schema.clone(),
                    name: name.clone(),
                    kind: "TRIGGER".to_owned(),
                }),
                SchemaObjectRowAction::ToggleTrigger => Some(SchemaObjectFolderAction::ToggleTrigger {
                    schema: trigger.schema.clone(),
                    table: trigger.table_name.clone(),
                    name: trigger.name.clone(),
                    enable: !trigger.enabled,
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
                SchemaObjectRowAction::ToggleTrigger => None,
                SchemaObjectRowAction::Drop => None,
                SchemaObjectRowAction::CopyName => Some(SchemaObjectFolderAction::CopyName(activation.name.clone())),
            })
            .collect()
    }
}
fn trigger_toggle_menu_label(enabled: bool, supported: bool) -> Option<&'static str> {
    if !supported {
        None
    } else if enabled {
        Some("Disable Trigger")
    } else {
        Some("Enable Trigger")
    }
}

#[cfg(test)]
mod tests {
    use super::trigger_toggle_menu_label;

    #[test]
    fn trigger_toggle_menu_offers_only_supported_inverse_action() {
        assert_eq!(trigger_toggle_menu_label(true, true), Some("Disable Trigger"));
        assert_eq!(trigger_toggle_menu_label(false, true), Some("Enable Trigger"));
        assert_eq!(trigger_toggle_menu_label(true, false), None);
    }
}
