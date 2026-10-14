use super::*;
use crate::editor::SqlDialect;

#[cfg(test)]
use crate::editor::{CompletionItem, CompletionItemKind};

impl DbProApp {
    pub(super) fn draw_query_editor(&mut self, ui: &mut egui::Ui) {
        if self.query.session.active_document_index >= self.query.session.documents.len() {
            return;
        }
        let dialect = if self.query_capabilities().allows(|caps| caps.query.numbered_parameters) {
            SqlDialect::Postgres
        } else {
            SqlDialect::SQLite
        };
        let active_schema = self.active_query_schema().to_owned();
        let effects = {
            let mut context = query_editor_surface_view::QueryEditorSurfaceContext {
                theme: self.theme,
                query_editor: &mut self.query.editor,
                query_session: &mut self.query.session,
                preferences: &self.preferences,
                schema: &self.schema.explorer,
                active_schema: &active_schema,
                dialect,
            };
            context.draw_query_editor(ui)
        };
        self.apply_query_editor_effects(effects);
    }

    fn apply_query_editor_effects(&mut self, effects: query_editor_surface_view::QueryEditorEffects) {
        use query_editor_surface_view::QueryEditorAction as Action;

        for action in effects.actions {
            match action {
                Action::CancelPrediction { request_id } => {
                    self.dispatch_command(UiCommand::CancelSqlPrediction { request_id });
                }
                Action::RequestPrediction(request) => {
                    let request_id = self.next_request_id();
                    let command = UiCommand::RequestSqlPrediction {
                        request_id,
                        document_id: request.document_id.clone(),
                        document_version: request.document_version,
                        anchor: request.anchor,
                        replacement_range: request.replacement_range,
                        context: request.context,
                    };
                    if self.dispatch_command(command) {
                        self.query.session.commit_prediction_request(
                            &request.document_id,
                            request_id,
                            request.fingerprint,
                        );
                    }
                }
                Action::DispatchStatement => {
                    self.dispatch_query();
                }
                Action::DispatchAll => {
                    self.dispatch_query_all();
                }
                Action::SaveQuery => {
                    self.save_query_document();
                }
                Action::OpenIdentified(target) => match target {
                    query_editor_support::IdentifierNavigation::Table(table) => self.open_table(table),
                    query_editor_support::IdentifierNavigation::SchemaObject {
                        selection,
                        schema,
                        name,
                        kind,
                    } => self.open_schema_object(selection, &schema, &name, &kind),
                    query_editor_support::IdentifierNavigation::Missing(message) => {
                        self.feedback.set_runtime_message(message);
                    }
                },
            }
        }
    }

    pub(super) fn draw_floating_completion_popup(&mut self, ctx: &egui::Context) {
        let dialect = if self.query_capabilities().allows(|caps| caps.query.numbered_parameters) {
            SqlDialect::Postgres
        } else {
            SqlDialect::SQLite
        };
        let theme = self.theme;
        let Some(document) = self
            .query
            .session
            .documents
            .get_mut(self.query.session.active_document_index)
        else {
            return;
        };
        query_completion_popup_view::draw_floating_completion_popup(ctx, theme, dialect, document);
    }
}

#[cfg(test)]
#[path = "tests/query_editor_panel_tests.rs"]
mod tests;

