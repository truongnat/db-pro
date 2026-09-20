use super::ide_workspace::{ReplacePreview, SearchHit};
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FilesSearchAction {
    Find,
    PreviewReplace,
    ReplaceAll,
    Refactor,
    OpenSql(String),
}

pub(super) struct FilesSearchContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) search_query: &'a mut String,
    pub(super) replace_query: &'a mut String,
    pub(super) refactor_from: &'a mut String,
    pub(super) refactor_to: &'a mut String,
    pub(super) replace_previews: &'a [ReplacePreview],
    pub(super) search_hits: &'a [SearchHit],
}

impl FilesSearchContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<FilesSearchAction> {
        let mut actions = self.draw_search_controls(ui);
        actions.extend(self.draw_refactor_controls(ui));
        self.draw_replace_previews(ui);
        actions.extend(self.draw_search_hits(ui));
        actions
    }

    fn draw_search_controls(&mut self, ui: &mut egui::Ui) -> Vec<FilesSearchAction> {
        let mut actions = Vec::new();
        ui.add(
            egui::TextEdit::singleline(self.search_query)
                .hint_text("Find in files…")
                .desired_width(ui.available_width()),
        );
        ui.add_space(4.0);
        ui.add(
            egui::TextEdit::singleline(self.replace_query)
                .hint_text("Replace with…")
                .desired_width(ui.available_width()),
        );
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("Find")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesSearchAction::Find);
            }
            if Button::new(self.theme)
                .text("Preview")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesSearchAction::PreviewReplace);
            }
            if Button::new(self.theme)
                .text("Replace all")
                .variant(ButtonVariant::Destructive)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesSearchAction::ReplaceAll);
            }
        });
        actions
    }

    fn draw_refactor_controls(&mut self, ui: &mut egui::Ui) -> Vec<FilesSearchAction> {
        let mut actions = Vec::new();
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(self.refactor_from)
                    .hint_text("Rename from")
                    .desired_width(90.0),
            );
            ui.add(
                egui::TextEdit::singleline(self.refactor_to)
                    .hint_text("to")
                    .desired_width(90.0),
            );
            if Button::new(self.theme)
                .text("Refactor")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesSearchAction::Refactor);
            }
        });
        actions
    }

    fn draw_replace_previews(&self, ui: &mut egui::Ui) {
        if self.replace_previews.is_empty() {
            return;
        }
        ui.add_space(6.0);
        section_label(ui, "REPLACE PREVIEW", self.theme);
        for preview in self.replace_previews.iter().take(30) {
            ui.label(
                RichText::new(format!(
                    "{}::{} · {} hits",
                    preview.root_id, preview.relative_path, preview.replacements
                ))
                .small()
                .color(self.theme.text_secondary),
            );
        }
    }

    fn draw_search_hits(&self, ui: &mut egui::Ui) -> Vec<FilesSearchAction> {
        if self.search_hits.is_empty() {
            return Vec::new();
        }
        let mut actions = Vec::new();
        ui.add_space(6.0);
        section_label(ui, "SEARCH RESULTS", self.theme);
        ui.add_space(4.0);
        for hit in self.search_hits.iter().take(40) {
            let label = format!("{}:{}", hit.relative_path, hit.line);
            if sidebar_item(ui, Icon::Search, &label, false, self.theme)
                .on_hover_text(&hit.preview)
                .clicked()
                && hit.relative_path.ends_with(".sql")
            {
                actions.push(FilesSearchAction::OpenSql(format!(
                    "{}::{}",
                    hit.root_id, hit.relative_path
                )));
            }
        }
        actions
    }
}
