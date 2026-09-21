//! Category folders of the Codex navigator: Views, Functions and Triggers.

use super::explorer_schema_object_folders_view::{SchemaObjectFolderAction, SchemaObjectFoldersView};
use super::*;

impl DbProApp {
    /// Views folder — only materialises the schema-scoped list when expanded.
    pub(super) fn draw_dbeaver_views_folder_lazy(&mut self, ui: &mut egui::Ui, schema: &str, count: usize) {
        let actions = SchemaObjectFoldersView::new(self.theme, &self.schema.explorer).draw_views(ui, schema, count);
        self.apply_schema_object_folder_actions(actions, ui);
    }

    /// Functions folder — deferred filter until the folder is open.
    pub(super) fn draw_dbeaver_functions_folder_lazy(&mut self, ui: &mut egui::Ui, schema: &str, count: usize) {
        let actions = SchemaObjectFoldersView::new(self.theme, &self.schema.explorer).draw_functions(ui, schema, count);
        self.apply_schema_object_folder_actions(actions, ui);
    }

    /// Triggers folder — deferred filter until the folder is open.
    pub(super) fn draw_dbeaver_triggers_folder_lazy(&mut self, ui: &mut egui::Ui, schema: &str, count: usize) {
        let actions = SchemaObjectFoldersView::new(self.theme, &self.schema.explorer).draw_triggers(ui, schema, count);
        self.apply_schema_object_folder_actions(actions, ui);
    }

    fn apply_schema_object_folder_actions(&mut self, actions: Vec<SchemaObjectFolderAction>, ui: &mut egui::Ui) {
        for action in actions {
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
