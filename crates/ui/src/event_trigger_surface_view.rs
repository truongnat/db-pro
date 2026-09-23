//! PostgreSQL event-trigger presentation and typed user intents.
use super::super::event_trigger_state::EventTriggerState;
use super::super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum EventTriggerSurfaceAction {
    Refresh,
    PreviewDrop(String),
    Alter {
        name: String,
        mode: String,
    },
    RequestDrop(String),
    PreviewCreate {
        name: String,
        event: String,
        function_ref: String,
        tags_csv: String,
    },
    Create,
    ClosePreview,
    ConfirmDrop(String),
    CancelDrop,
}

pub(super) struct EventTriggerSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut EventTriggerState,
}

impl EventTriggerSurfaceContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<EventTriggerSurfaceAction> {
        let mut actions = Vec::new();
        self.draw_header(ui, &mut actions);
        self.draw_inventory(ui, &mut actions);
        self.draw_create_form(ui, &mut actions);
        self.draw_preview(ui, &mut actions);
        self.draw_drop_confirmation(ui, &mut actions);
        actions
    }

    fn draw_header(&self, ui: &mut egui::Ui, actions: &mut Vec<EventTriggerSurfaceAction>) {
        ui.add_space(SPACE_MD);
        section_label(ui, "EVENT TRIGGERS", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new(
                "PostgreSQL-only · database-level DDL hooks · not table/row triggers · create requires existing function",
            )
            .small()
            .color(self.theme.text_muted),
        );
        if secondary_button_with_icon(ui, Icon::RefreshCw, "Load event triggers", self.theme).clicked() {
            actions.push(EventTriggerSurfaceAction::Refresh);
        }
        if let Some(error) = &self.state.event_trigger_error {
            ui.colored_label(self.theme.danger, error);
        }
    }

    fn draw_inventory(&self, ui: &mut egui::Ui, actions: &mut Vec<EventTriggerSurfaceAction>) {
        let Some(inventory) = self.state.event_trigger_inventory.as_ref() else {
            return;
        };
        ui.label(
            RichText::new(&inventory.message)
                .small()
                .color(self.theme.text_secondary),
        );
        for trigger in inventory.triggers.iter().take(50) {
            self.draw_trigger(ui, trigger, actions);
        }
    }

    fn draw_trigger(
        &self,
        ui: &mut egui::Ui,
        trigger: &db_pro_core::domain::event_trigger::EventTriggerInfo,
        actions: &mut Vec<EventTriggerSurfaceAction>,
    ) {
        let trigger_name = trigger.name.clone();
        card_frame(self.theme).show(ui, |ui| {
            ui.label(
                RichText::new(format!(
                    "{} · on {} · {} · fn={}",
                    trigger.name, trigger.event, trigger.enabled_label, trigger.function_signature
                ))
                .strong()
                .monospace(),
            );
            if !trigger.tags.is_empty() {
                ui.label(
                    RichText::new(format!("tags: {}", trigger.tags.join(", ")))
                        .small()
                        .color(self.theme.text_muted),
                );
            }
            ui.horizontal(|ui| {
                if ghost_button_with_icon(ui, Icon::FileCode2, "Preview DROP", self.theme).clicked() {
                    actions.push(EventTriggerSurfaceAction::PreviewDrop(trigger_name.clone()));
                }
                if ghost_button(ui, "Disable", self.theme).clicked() {
                    actions.push(EventTriggerSurfaceAction::Alter {
                        name: trigger_name.clone(),
                        mode: "disable".to_owned(),
                    });
                }
                if ghost_button(ui, "Enable", self.theme).clicked() {
                    actions.push(EventTriggerSurfaceAction::Alter {
                        name: trigger_name.clone(),
                        mode: "enable".to_owned(),
                    });
                }
                if danger_button(ui, "Drop…", self.theme).clicked() {
                    actions.push(EventTriggerSurfaceAction::RequestDrop(trigger_name.clone()));
                }
            });
        });
        ui.add_space(SPACE_XS);
    }

    fn draw_create_form(&mut self, ui: &mut egui::Ui, actions: &mut Vec<EventTriggerSurfaceAction>) {
        ui.add_space(SPACE_SM);
        ui.label(RichText::new("Create event trigger").small().strong());
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.state.event_trigger_create_name).hint_text("name"));
            ui.add(egui::TextEdit::singleline(&mut self.state.event_trigger_create_event).hint_text("event"));
        });
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.state.event_trigger_create_function).hint_text("schema.func()"),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.state.event_trigger_create_tags).hint_text("tags CSV optional"),
            );
        });
        ui.horizontal(|ui| {
            if ghost_button_with_icon(ui, Icon::FileCode2, "Preview CREATE", self.theme).clicked() {
                actions.push(EventTriggerSurfaceAction::PreviewCreate {
                    name: self.state.event_trigger_create_name.clone(),
                    event: self.state.event_trigger_create_event.clone(),
                    function_ref: self.state.event_trigger_create_function.clone(),
                    tags_csv: self.state.event_trigger_create_tags.clone(),
                });
            }
            if secondary_button(ui, "Create (confirm)", self.theme).clicked() {
                actions.push(EventTriggerSurfaceAction::Create);
            }
        });
    }

    fn draw_preview(&self, ui: &mut egui::Ui, actions: &mut Vec<EventTriggerSurfaceAction>) {
        let Some(preview) = self.state.event_trigger_ddl_preview.as_ref() else {
            return;
        };
        let mut open = true;
        Dialog::new(&mut open, "Event trigger DDL preview", self.theme)
            .width(560.0)
            .id_salt("event_trigger_preview_dialog")
            .show_framed_ctx(ui.ctx(), |frame| {
                frame.body(|ui| {
                    ui.label(RichText::new(preview).monospace());
                });
                frame.footer(|ui| {
                    if secondary_button(ui, "Close", self.theme).clicked() {
                        actions.push(EventTriggerSurfaceAction::ClosePreview);
                    }
                });
            });
        if !open {
            actions.push(EventTriggerSurfaceAction::ClosePreview);
        }
    }

    fn draw_drop_confirmation(&self, ui: &mut egui::Ui, actions: &mut Vec<EventTriggerSurfaceAction>) {
        let Some(name) = self.state.event_trigger_drop_confirm.as_ref() else {
            return;
        };
        let mut open = true;
        Dialog::new(&mut open, "Drop event trigger?", self.theme)
            .width(460.0)
            .id_salt("event_trigger_drop_dialog")
            .show_framed_ctx(ui.ctx(), |frame| {
                frame.body(|ui| {
                    ui.label(format!(
                        "Drop event trigger `{name}`? This changes global DDL hook behavior."
                    ));
                });
                frame.footer(|ui| {
                    if danger_button(ui, "Drop", self.theme).clicked() {
                        actions.push(EventTriggerSurfaceAction::ConfirmDrop(name.clone()));
                    }
                    if secondary_button(ui, "Cancel", self.theme).clicked() {
                        actions.push(EventTriggerSurfaceAction::CancelDrop);
                    }
                });
            });
        if !open {
            actions.push(EventTriggerSurfaceAction::CancelDrop);
        }
    }
}
