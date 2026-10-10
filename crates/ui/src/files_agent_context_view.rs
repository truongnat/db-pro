// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use super::*;
use crate::components::agent_primitives::{ContextChip, ContextChipKind};
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
        egui::Frame {
            fill: self.theme.surface_elevated,
            stroke: egui::Stroke::new(1.0, self.theme.border_subtle),
            inner_margin: egui::Margin::same(SPACE_SM as i8),
            corner_radius: egui::CornerRadius::same(RADIUS_MD as u8),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(char::from(Icon::Bot).to_string())
                        .font(font_icon(ICON_DEFAULT))
                        .color(self.theme.accent),
                );
                ui.label(RichText::new("AGENT CONTEXT").font(font_caption()).strong().color(self.theme.text_secondary));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let popup_id = ui.make_persistent_id("files_agent_context_actions");
                    let more = Button::new(self.theme)
                        .icon(Icon::Ellipsis)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Context actions")
                        .access_label("Context actions")
                        .show(ui);
                    if more.clicked() {
                        egui::Popup::toggle_id(ui.ctx(), popup_id);
                    }
                    egui::Popup::new(popup_id, ui.ctx().clone(), &more, ui.layer_id())
                        .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
                        .open_memory(None)
                        .show(|ui| {
                            ui.set_min_width(180.0);
                            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
                            if Button::new(self.theme)
                                .icon(Icon::GitCompare)
                                .text("Check schema drift")
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .full_width(true)
                                .left_aligned()
                                .show(ui)
                                .clicked()
                            {
                                actions.push(FilesAgentContextAction::RefreshSchemaDrift);
                            }
                            if Button::new(self.theme)
                                .icon(Icon::Camera)
                                .text("Export schema snapshot")
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .full_width(true)
                                .left_aligned()
                                .show(ui)
                                .clicked()
                            {
                                actions.push(FilesAgentContextAction::ExportSchemaSnapshot);
                            }
                            if Button::new(self.theme)
                                .icon(Icon::Columns2)
                                .text("Toggle split editor")
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .full_width(true)
                                .left_aligned()
                                .show(ui)
                                .clicked()
                            {
                                actions.push(FilesAgentContextAction::ToggleSplitEditor);
                            }
                            ui.separator();
                            if Button::new(self.theme)
                                .icon(Icon::Trash2)
                                .text("Clear context")
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .full_width(true)
                                .left_aligned()
                                .show(ui)
                                .clicked()
                            {
                                actions.push(FilesAgentContextAction::Clear);
                            }
                        },
                    );
                });
            });

            ui.add_space(SPACE_XS);

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(SPACE_XS, SPACE_XXS);
                if Button::new(self.theme)
                    .icon(Icon::FileCode2)
                    .text("Active File")
                    .variant(ButtonVariant::Secondary)
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
                    .icon(Icon::TextSelect)
                    .text("Selection")
                    .variant(ButtonVariant::Secondary)
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
                    .icon(Icon::Table)
                    .text("Table")
                    .variant(ButtonVariant::Secondary)
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

            ui.add_space(SPACE_XS);

            if self.context_items.is_empty() {
                ui.label(
                    // cc-scan:allow LINE_TOO_LONG — literal must not wrap
                    RichText::new("Attach a file, selection, or table to give the agent context.")
                        .font(font_caption())
                        .color(self.theme.text_muted),
                );
            } else {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(SPACE_XS, SPACE_XS);
                    for item in self.context_items.iter().cloned() {
                        let short = crate::components::truncate_ellipsis(&item, 28);
                        if ContextChip::new(ContextChipKind::File, &short, self.theme)
                            .removable(true)
                            .show(ui)
                            .clicked()
                        {
                            actions.push(FilesAgentContextAction::RemoveItem(item));
                        }
                    }
                });
            }
        });
        actions
    }
}
