//! Category folders of the Codex navigator: Views, Functions and Triggers.

use super::explorer_schema_object_folders_view::SchemaObjectFolderAction;
use super::schema_workbench::SchemaWorkbenchMode;
use super::*;
use db_pro_core::domain::object_mutation::ObjectAction;

impl DbProApp {
    pub(super) fn apply_schema_object_folder_action(&mut self, action: SchemaObjectFolderAction, ui: &mut egui::Ui) {
        match action {
            SchemaObjectFolderAction::Open(request) => {
                self.open_schema_object(request.selection, &request.schema, &request.name, &request.kind);
            }
            SchemaObjectFolderAction::OpenQuery(query) => {
                self.set_active_query_text(query);
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            SchemaObjectFolderAction::ModifyView(view) => {
                self.schema.workbench.mode = SchemaWorkbenchMode::View;
                self.schema.workbench.schema = view.schema.clone();
                self.schema.workbench.name = view.name.clone();
                self.schema.workbench.select_sql = view.definition.clone();
                self.schema.workbench.materialized = false;
                self.workspace.active_tab = WorkspaceTab::SchemaWorkbench;
                self.feedback.set_runtime_message(format!("Loaded view `{}.{}` into workbench", view.schema, view.name));
            }
            SchemaObjectFolderAction::DropObject { schema, name, kind } => {
                let mode = match kind.to_ascii_uppercase().as_str() {
                    "VIEW" => SchemaWorkbenchMode::View,
                    "TRIGGER" => SchemaWorkbenchMode::Trigger,
                    _ => SchemaWorkbenchMode::Table,
                };
                self.schema.workbench.mode = mode;
                self.schema.workbench.schema = schema.clone();
                self.schema.workbench.name = name.clone();
                self.schema.workbench.parent_table = name.clone();
                self.plan_workbench_action(ObjectAction::Drop);
                self.schema.workbench.apply_confirmation = true;
                self.workspace.active_tab = WorkspaceTab::SchemaWorkbench;
                self.feedback.set_runtime_message(format!("Planned drop for {kind} `{schema}.{name}`"));
            }
            SchemaObjectFolderAction::CopyName(name) => {
                ui.output_mut(|output| output.copied_text = name.clone());
                self.feedback.runtime_message = format!("Copied `{name}` to clipboard");
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
        explorer_navigation::SchemaObjectActivationContext::new(
            &mut self.schema.explorer,
            &mut self.table,
            &mut self.workspace,
            &mut self.management.routine,
            &mut self.feedback,
        )
        .open(explorer_navigation::SchemaObjectActivation {
            selection,
            schema: schema.to_owned(),
            name: name.to_owned(),
            kind: kind.to_owned(),
        });
    }
}
