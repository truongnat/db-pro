use super::schema_workbench::{SchemaWorkbenchMode, SchemaWorkbenchState};
use super::*;
use db_pro_core::domain::object_mutation::ObjectDependencyEdge;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SchemaWorkbenchSurfaceAction {
    SelectMode(SchemaWorkbenchMode),
    Form(schema_workbench_form::SchemaWorkbenchFormAction),
    Secondary(schema_workbench_secondary_view::SchemaWorkbenchSecondaryAction),
}

pub(super) struct SchemaWorkbenchSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) workbench: &'a mut SchemaWorkbenchState,
    pub(super) can_mutate: bool,
    pub(super) driver: &'a str,
    pub(super) edges: &'a [ObjectDependencyEdge],
}

impl SchemaWorkbenchSurfaceContext<'_> {
    pub(super) fn draw_sidebar(&mut self, ui: &mut egui::Ui) -> Option<SchemaWorkbenchSurfaceAction> {
        section_label(ui, "SCHEMA WORKBENCH", self.theme);
        ui.add_space(6.0);
        ui.label(
            RichText::new("Plan → preview → apply typed object mutations")
                .small()
                .color(self.theme.text_muted),
        );
        ui.add_space(10.0);
        for (mode, icon, label) in workbench_modes() {
            let selected = self.workbench.mode == mode;
            if sidebar_item(ui, icon, label, selected, self.theme).clicked() {
                return Some(SchemaWorkbenchSurfaceAction::SelectMode(mode));
            }
            ui.add_space(2.0);
        }
        None
    }

    pub(super) fn draw_main(&mut self, ui: &mut egui::Ui) -> Vec<SchemaWorkbenchSurfaceAction> {
        let mut actions = Vec::new();
        ui.set_min_width(ui.available_width());
        egui::ScrollArea::vertical()
            .id_salt("schema_workbench_scroll")
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.add_space(SPACE_SM);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Schema Workbench")
                            .font(font_subheading())
                            .strong()
                            .color(self.theme.text_primary),
                    );
                    badge(ui, self.driver, self.theme.accent_soft, self.theme.accent);
                });
                ui.add_space(SPACE_MD);

                match self.workbench.mode {
                    SchemaWorkbenchMode::Dependencies => self.draw_dependencies(ui),
                    SchemaWorkbenchMode::Docs => {
                        if let Some(action) = self.draw_docs(ui) {
                            actions.push(SchemaWorkbenchSurfaceAction::Secondary(action));
                        }
                    }
                    _ => self.draw_definition(ui, &mut actions),
                }
            });
        actions
    }

    fn draw_dependencies(&mut self, ui: &mut egui::Ui) {
        let mut context = schema_workbench_secondary_view::SchemaWorkbenchSecondaryContext {
            theme: self.theme,
            workbench: self.workbench,
            edges: self.edges,
        };
        schema_workbench_secondary_view::draw_dependency_navigator(&mut context, ui);
    }

    fn draw_docs(
        &mut self,
        ui: &mut egui::Ui,
    ) -> Option<schema_workbench_secondary_view::SchemaWorkbenchSecondaryAction> {
        let mut context = schema_workbench_secondary_view::SchemaWorkbenchSecondaryContext {
            theme: self.theme,
            workbench: self.workbench,
            edges: &[],
        };
        schema_workbench_secondary_view::draw_docs_export(&mut context, ui)
    }

    fn draw_definition(&mut self, ui: &mut egui::Ui, actions: &mut Vec<SchemaWorkbenchSurfaceAction>) {
        let form_width = ui.available_width().min(720.0);
        ui.allocate_ui_with_layout(
            egui::vec2(form_width, ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                card_frame(self.theme).show(ui, |ui| {
                    ui.set_min_width(form_width - 8.0);
                    section_label(ui, "DEFINITION", self.theme);
                    ui.add_space(SPACE_SM);
                    let mut context = schema_workbench_form::SchemaWorkbenchFormContext {
                        theme: self.theme,
                        workbench: self.workbench,
                        can_mutate: self.can_mutate,
                    };
                    if let Some(action) = schema_workbench_form::draw_workbench_form(&mut context, ui) {
                        actions.push(SchemaWorkbenchSurfaceAction::Form(action));
                    }
                });
                ui.add_space(SPACE_MD);
                let mut context = schema_workbench_form::SchemaWorkbenchFormContext {
                    theme: self.theme,
                    workbench: self.workbench,
                    can_mutate: self.can_mutate,
                };
                if let Some(action) = schema_workbench_form::draw_workbench_preview(&mut context, ui) {
                    actions.push(SchemaWorkbenchSurfaceAction::Form(action));
                }
            },
        );
    }
}

fn workbench_modes() -> [(SchemaWorkbenchMode, Icon, &'static str); 14] {
    [
        (SchemaWorkbenchMode::Table, Icon::Table2, "Table / columns"),
        (SchemaWorkbenchMode::Column, Icon::Columns3, "Column alter"),
        (SchemaWorkbenchMode::View, Icon::Eye, "Views"),
        (SchemaWorkbenchMode::Index, Icon::ListTree, "Indexes"),
        (SchemaWorkbenchMode::Constraint, Icon::Link, "Constraints"),
        (SchemaWorkbenchMode::Trigger, Icon::Zap, "Triggers"),
        (SchemaWorkbenchMode::Sequence, Icon::Hash, "Sequences"),
        (SchemaWorkbenchMode::Type, Icon::Shapes, "Types / enums"),
        (SchemaWorkbenchMode::SchemaDb, Icon::Database, "Schema / database"),
        (SchemaWorkbenchMode::Extension, Icon::Puzzle, "Extensions"),
        (SchemaWorkbenchMode::Comment, Icon::MessageSquareText, "Comments"),
        (SchemaWorkbenchMode::Partition, Icon::LayoutGrid, "Partitions"),
        (SchemaWorkbenchMode::Dependencies, Icon::GitBranch, "Dependencies"),
        (SchemaWorkbenchMode::Docs, Icon::FileText, "Docs export"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workbench_surface_keeps_mode_selection_typed() {
        assert_eq!(
            SchemaWorkbenchSurfaceAction::SelectMode(SchemaWorkbenchMode::Docs),
            SchemaWorkbenchSurfaceAction::SelectMode(SchemaWorkbenchMode::Docs)
        );
    }
}
