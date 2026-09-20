use super::*;

impl DbProApp {
    pub(super) fn draw_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_tabs(ui);
        match self.workspace.active_tab {
            WorkspaceTab::Welcome => self.draw_welcome(ui),
            WorkspaceTab::Query => self.draw_query(ui),
            WorkspaceTab::Table => self.draw_table_workspace(ui),
            WorkspaceTab::SchemaObject => self.draw_schema_object_workspace(ui),
            WorkspaceTab::Diagram => self.draw_diagram(ui),
            WorkspaceTab::SchemaWorkbench => self.draw_schema_workbench(ui),
            WorkspaceTab::SchemaCompare => self.draw_schema_compare(ui),
            WorkspaceTab::ComponentGallery => self.draw_component_gallery(ui),
        }
        self.draw_discard_changes_confirmation(ui);
    }
}
