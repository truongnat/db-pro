//! Visual query-builder root adapter (#247).
use super::*;
use crate::query::visual_builder::BuilderDialect;

#[path = "visual_query_builder_surface_view.rs"]
mod visual_query_builder_surface_view;

impl DbProApp {
    pub(super) fn visual_builder_dialect(&self) -> BuilderDialect {
        visual_query_builder_state::VisualQueryBuilderState::dialect_for_driver(self.active_driver())
    }

    pub(super) fn draw_visual_query_builder(&mut self, ui: &mut egui::Ui) {
        let action = {
            let dialect = self.visual_builder_dialect();
            let mut context = visual_query_builder_surface_view::VisualQueryBuilderContext {
                theme: self.theme,
                state: &mut self.query.editor.visual_builder,
                table_details: &self.schema.explorer.schema.table_details,
                views: &self.schema.explorer.schema.views,
                dialect,
            };
            context.draw(ui)
        };
        if let Some(action) = action {
            self.apply_visual_query_builder_action(action);
        }
    }

    fn apply_visual_query_builder_action(
        &mut self,
        action: visual_query_builder_surface_view::VisualQueryBuilderAction,
    ) {
        match action {
            visual_query_builder_surface_view::VisualQueryBuilderAction::ApplySql => self.apply_visual_builder_sql(),
            visual_query_builder_surface_view::VisualQueryBuilderAction::ImportFromEditor => {
                self.import_visual_builder_from_editor();
            }
            visual_query_builder_surface_view::VisualQueryBuilderAction::Clear => {
                self.query.editor.visual_builder.clear();
            }
        }
    }

    fn apply_visual_builder_sql(&mut self) {
        let dialect = self.visual_builder_dialect();
        self.query.editor.visual_builder.refresh_preview(dialect);
        if self.query.editor.visual_builder.sql_preview.is_empty() {
            return;
        }
        self.set_active_query_text(self.query.editor.visual_builder.sql_preview.clone());
        self.feedback.runtime_message = "Visual builder SQL applied to editor (not executed)".into();
    }

    fn import_visual_builder_from_editor(&mut self) {
        let sql = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .map(|document| document.buffer.text().to_owned())
            .unwrap_or_default();
        let dialect = self.visual_builder_dialect();
        match self.query.editor.visual_builder.import_sql(&sql, dialect) {
            Ok(()) => {
                self.feedback.runtime_message = "Imported supported SELECT into visual builder".into();
            }
            Err(error) => {
                self.query.editor.visual_builder.error = Some(format!("Import refused (keeping text editor): {error}"));
            }
        }
    }
}
