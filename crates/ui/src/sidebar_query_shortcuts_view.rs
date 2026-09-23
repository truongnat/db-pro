//! Built-in SQL snippets and scratch-query affordances.
use super::*;
use lucide_icons::Icon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SidebarQueryShortcutAction {
    InsertSnippet(String),
    NewScratch,
}

pub(super) struct SidebarQueryShortcutsContext {
    pub(super) theme: DbProTheme,
}

impl SidebarQueryShortcutsContext {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<SidebarQueryShortcutAction> {
        let mut actions = self.draw_snippets(ui);
        ui.add_space(14.0);
        section_label(ui, "SCRATCH", self.theme);
        ui.add_space(6.0);
        ui.label(
            RichText::new("Scratch tabs are disposable — use New scratch for throwaway SQL.")
                .small()
                .color(self.theme.text_muted),
        );
        if compact_button_with_icon(ui, Icon::FilePlus2, "Open scratch SQL", self.theme).clicked() {
            actions.push(SidebarQueryShortcutAction::NewScratch);
        }
        actions
    }

    fn draw_snippets(&self, ui: &mut egui::Ui) -> Vec<SidebarQueryShortcutAction> {
        let mut actions = Vec::new();
        section_label(ui, "SNIPPETS", self.theme);
        ui.add_space(6.0);
        for (label, snippet) in query_snippets::builtin_sql_snippets() {
            if sidebar_item(ui, Icon::Braces, label, false, self.theme)
                .on_hover_text(*snippet)
                .clicked()
            {
                actions.push(SidebarQueryShortcutAction::InsertSnippet((*snippet).to_owned()));
            }
        }
        actions
    }
}
