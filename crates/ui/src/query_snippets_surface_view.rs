//! Query-snippet presentation and typed insertion intent.
use super::super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};

pub(super) enum QuerySnippetsAction {
    Insert(&'static str),
}

pub(super) struct QuerySnippetsContext {
    pub(super) theme: DbProTheme,
}

impl QuerySnippetsContext {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Option<QuerySnippetsAction> {
        let mut action = None;
        card_frame(self.theme).show(ui, |ui| {
            ui.label(egui::RichText::new("SQL snippets").strong());
            for (label, snippet) in query_snippets::builtin_sql_snippets() {
                if Button::new(self.theme)
                    .text(*label)
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    action = Some(QuerySnippetsAction::Insert(snippet));
                }
            }
        });
        action
    }
}
