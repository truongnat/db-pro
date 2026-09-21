use super::*;

impl DbProApp {
    /// Draw the Foreign Keys tab: relations table with target jump and copy actions.
    pub(super) fn draw_table_relations_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table.state.table_info.clone() else {
            ui.label(RichText::new("Table structure is still loading…").color(self.theme.text_muted));
            return;
        };
        let actions = table_relations_surface_view::TableRelationsContext {
            theme: self.theme,
            info: &info,
            search: &mut self.table.state.table_metadata_search,
        }
        .draw(ui);
        for action in actions {
            match action {
                table_relations_surface_view::TableRelationsAction::OpenTable(table) => {
                    self.open_table(table);
                }
            }
        }
    }

    /// Draw the Constraints tab: categorized constraints (PK, FK, Unique, Check, NOT NULL).
    pub(super) fn draw_table_constraints_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table.state.table_info.clone() else {
            ui.label(RichText::new("Table structure is still loading…").color(self.theme.text_muted));
            return;
        };
        table_metadata_surface_view::TableMetadataContext {
            theme: self.theme,
            info: &info,
            search: &mut self.table.state.table_metadata_search,
            constraint_filter: &mut self.table.state.table_constraint_filter,
            dependency_filter: &mut self.table.state.table_dependency_filter,
        }
        .draw_constraints(ui);
    }

    /// Draw the Dependencies tab: real dependency graph entries (Incoming & Outgoing).
    pub(super) fn draw_table_dependencies_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table.state.table_info.clone() else {
            ui.label(RichText::new("Table structure is still loading…").color(self.theme.text_muted));
            return;
        };
        let actions = table_metadata_surface_view::TableMetadataContext {
            theme: self.theme,
            info: &info,
            search: &mut self.table.state.table_metadata_search,
            constraint_filter: &mut self.table.state.table_constraint_filter,
            dependency_filter: &mut self.table.state.table_dependency_filter,
        }
        .draw_dependencies(ui);
        for action in actions {
            match action {
                table_metadata_surface_view::TableMetadataAction::OpenTable(table) => {
                    self.open_table(table);
                }
            }
        }
    }
}
