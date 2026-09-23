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
mod tests {
    use super::query_editor_support::apply_completion_item;
    use super::*;

    fn table_completion(replacement_range: (usize, usize)) -> CompletionItem {
        CompletionItem {
            label: "users".to_owned(),
            insert_text: "users".to_owned(),
            kind: CompletionItemKind::Table,
            detail: Some("Table · public.users".to_owned()),
            documentation: Some("Table with 3 columns".to_owned()),
            replacement_range,
            sort_score: 900,
        }
    }

    #[test]
    fn completion_applies_only_to_the_document_version_that_produced_it() {
        let mut document = QueryDocument::new("query-1", "Query 1", "SELECT * FROM us");
        let version = document.buffer.version();
        let item = table_completion((14, 16));
        document.completion.open(
            16,
            version,
            egui::Pos2::ZERO,
            "us".to_owned(),
            vec![item.clone()],
            crate::editor::CompletionTriggerKind::Manual,
        );

        document.buffer.insert(16, "x");
        assert!(!apply_completion_item(&mut document, &item, SqlDialect::Postgres));
        assert_eq!(document.buffer.text(), "SELECT * FROM usx");
        assert!(!document.completion.is_open);
    }

    #[test]
    fn current_completion_replaces_only_its_declared_prefix() {
        let mut document = QueryDocument::new("query-1", "Query 1", "SELECT * FROM us");
        let version = document.buffer.version();
        let item = table_completion((14, 16));
        document.completion.open(
            16,
            version,
            egui::Pos2::ZERO,
            "us".to_owned(),
            vec![item.clone()],
            crate::editor::CompletionTriggerKind::Manual,
        );

        assert!(apply_completion_item(&mut document, &item, SqlDialect::Postgres));
        assert_eq!(document.buffer.text(), "SELECT * FROM users");
        assert_eq!(document.cursor.offset, document.buffer.len_bytes());
        assert!(!document.completion.is_open);
    }
}
