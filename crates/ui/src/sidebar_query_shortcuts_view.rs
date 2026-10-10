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
        ui.label(RichText::new("SCRATCH").font(font_caption()).strong().color(self.theme.text_secondary));
        ui.add_space(6.0);
        ui.label(
            RichText::new("Scratch tabs are disposable — use New scratch for throwaway SQL.")
                .small()
                .color(self.theme.text_muted),
        );
        if Button::new(self.theme)
            .icon(Icon::FilePlus2)
            .text("Open scratch SQL")
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            actions.push(SidebarQueryShortcutAction::NewScratch);
        }
        actions
    }

    fn draw_snippets(&self, ui: &mut egui::Ui) -> Vec<SidebarQueryShortcutAction> {
        let mut actions = Vec::new();
        ui.label(RichText::new("SNIPPETS").font(font_caption()).strong().color(self.theme.text_secondary));
        ui.add_space(6.0);
        for (label, trigger, snippet) in query_snippets::builtin_sql_snippets() {
            let response = sidebar_item(ui, Icon::Braces, label, false, self.theme);
            ui.painter().text(
                egui::pos2(response.rect.right() - SPACE_SM, response.rect.center().y),
                egui::Align2::RIGHT_CENTER,
                *trigger,
                font_mono_sm(),
                self.theme.text_muted,
            );
            if response.on_hover_text(*snippet).clicked() {
                actions.push(SidebarQueryShortcutAction::InsertSnippet((*snippet).to_owned()));
            }
        }
        actions
    }
}
