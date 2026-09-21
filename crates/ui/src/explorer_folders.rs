//! Category folders of the Codex navigator: Views, Functions and Triggers.

use super::explorer_schema_object_folders_view::SchemaObjectFolderAction;
use super::*;

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
