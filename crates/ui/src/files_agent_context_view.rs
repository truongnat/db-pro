use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use egui::{vec2, Align, Layout, RichText};
use lucide_icons::Icon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FilesAgentContextAction {
    Clear,
    RefreshSchemaDrift,
    ExportSchemaSnapshot,
    ToggleSplitEditor,
    AddItem(String),
    RemoveItem(String),
}

pub(super) struct ActiveQueryContext {
    pub(super) path: Option<String>,
    pub(super) title: String,
}

pub(super) struct FilesAgentContextView<'a> {
    pub(super) theme: DbProTheme,
    pub(super) context_items: &'a [String],
    pub(super) active_document: Option<&'a ActiveQueryContext>,
    pub(super) selected_text: &'a str,
    pub(super) selected_table: Option<&'a str>,
}

impl<'a> FilesAgentContextView<'a> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<FilesAgentContextAction> {
        let mut actions = Vec::new();
        ui.horizontal(|ui| {
            section_label(ui, "AGENT CONTEXT", self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if Button::new(self.theme)
                    .icon(Icon::Trash2)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Clear context")
                    .show(ui)
                    .clicked()
                {
                    actions.push(FilesAgentContextAction::Clear);
                }
                if Button::new(self.theme)
                    .icon(Icon::GitCompare)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Check schema drift")
                    .show(ui)
                    .clicked()
                {
                    actions.push(FilesAgentContextAction::RefreshSchemaDrift);
                }
                if Button::new(self.theme)
                    .icon(Icon::Camera)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Export schema snapshot")
                    .show(ui)
                    .clicked()
                {
                    actions.push(FilesAgentContextAction::ExportSchemaSnapshot);
                }
                if Button::new(self.theme)
                    .icon(Icon::Columns2)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Toggle split editor")
                    .show(ui)
                    .clicked()
                {
                    actions.push(FilesAgentContextAction::ToggleSplitEditor);
                }
            });
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("+ File")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .tooltip("Add active file to agent context")
                .show(ui)
                .clicked()
            {
                if let Some(document) = self.active_document {
                    actions.push(FilesAgentContextAction::AddItem(
                        document
                            .path
                            .clone()
                            .unwrap_or_else(|| format!("query:{}", document.title)),
                    ));
                }
            }
            if Button::new(self.theme)
                .text("+ Selection")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .tooltip("Add current SQL selection")
                .show(ui)
                .clicked()
                && !self.selected_text.trim().is_empty()
            {
                actions.push(FilesAgentContextAction::AddItem(format!(
                    "selection:{}",
                    self.selected_text.chars().take(80).collect::<String>()
                )));
            }
            if Button::new(self.theme)
                .text("+ Table")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .tooltip("Add selected table")
                .show(ui)
                .clicked()
            {
                if let Some(table) = self.selected_table {
                    actions.push(FilesAgentContextAction::AddItem(format!("table:{table}")));
                }
            }
        });
        if self.context_items.is_empty() {
            ui.label(
                RichText::new("No context chips yet — add a file, selection, or table.")
                    .small()
                    .color(self.theme.text_muted),
            );
        } else {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                for item in self.context_items.iter().cloned() {
                    let short = if item.len() > 28 {
                        format!("{}…", item.chars().take(27).collect::<String>())
                    } else {
                        item.clone()
                    };
                    if tag_chip(ui, &short, true, self.theme) {
                        actions.push(FilesAgentContextAction::RemoveItem(item));
                    }
                }
            });
        }
        actions
    }
}
