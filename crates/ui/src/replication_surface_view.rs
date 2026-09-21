//! Logical-replication presentation and typed user intents.
use super::super::replication_state::ReplicationState;
use super::super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ReplicationSurfaceAction {
    Refresh,
    PreviewDropPublication(String),
    RequestDropPublication(String),
    PreviewDropSubscription(String),
    RequestDropSubscription(String),
    PreviewCreatePublication(String),
    CreatePublication,
    ClosePreview,
    ConfirmDropPublication(String),
    ConfirmDropSubscription(String),
    CancelDropPublication,
    CancelDropSubscription,
}

pub(super) struct ReplicationSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut ReplicationState,
}

impl ReplicationSurfaceContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<ReplicationSurfaceAction> {
        let mut actions = Vec::new();
        self.draw_header(ui, &mut actions);
        self.draw_inventory(ui, &mut actions);
        self.draw_create_form(ui, &mut actions);
        self.draw_preview(ui, &mut actions);
        self.draw_drop_confirmations(ui, &mut actions);
        actions
    }

    fn draw_header(&self, ui: &mut egui::Ui, actions: &mut Vec<ReplicationSurfaceAction>) {
        ui.add_space(SPACE_MD);
        section_label(ui, "LOGICAL REPLICATION", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new(
                "PostgreSQL-only · subscription conninfo redacted · CREATE SUBSCRIPTION not offered (secrets)",
            )
            .small()
            .color(self.theme.text_muted),
        );
        if secondary_button_with_icon(ui, Icon::RefreshCw, "Load replication inventory", self.theme).clicked() {
            actions.push(ReplicationSurfaceAction::Refresh);
        }
        if let Some(error) = &self.state.replication_error {
            ui.colored_label(self.theme.danger, error);
        }
    }

    fn draw_inventory(&self, ui: &mut egui::Ui, actions: &mut Vec<ReplicationSurfaceAction>) {
        let Some(inventory) = self.state.replication_inventory.as_ref() else {
            return;
        };
        ui.label(
            RichText::new(&inventory.message)
                .small()
                .color(self.theme.text_secondary),
        );
        self.draw_publications(ui, &inventory.publications, actions);
        self.draw_subscriptions(ui, &inventory.subscriptions, actions);
        self.draw_slots(ui, &inventory.slots);
    }

    fn draw_publications(
        &self,
        ui: &mut egui::Ui,
        publications: &[db_pro_core::domain::replication::PublicationInfo],
        actions: &mut Vec<ReplicationSurfaceAction>,
    ) {
        for publication in publications.iter().take(40) {
            let name = publication.name.clone();
            card_frame(self.theme).show(ui, |ui| {
                ui.label(
                    RichText::new(format!(
                        "publication {} · all_tables={} · owner={:?}",
                        publication.name, publication.all_tables, publication.owner
                    ))
                    .strong()
                    .monospace(),
                );
                if !publication.tables.is_empty() {
                    ui.label(
                        RichText::new(format!("tables: {}", publication.tables.join(", ")))
                            .small()
                            .color(self.theme.text_muted),
                    );
                }
                ui.horizontal(|ui| {
                    if ghost_button_with_icon(ui, Icon::FileCode2, "Preview DROP", self.theme).clicked() {
                        actions.push(ReplicationSurfaceAction::PreviewDropPublication(name.clone()));
                    }
                    if danger_button(ui, "Drop…", self.theme).clicked() {
                        actions.push(ReplicationSurfaceAction::RequestDropPublication(name.clone()));
                    }
                });
            });
            ui.add_space(SPACE_XS);
        }
    }

    fn draw_subscriptions(
        &self,
        ui: &mut egui::Ui,
        subscriptions: &[db_pro_core::domain::replication::SubscriptionInfo],
        actions: &mut Vec<ReplicationSurfaceAction>,
    ) {
        for subscription in subscriptions.iter().take(40) {
            let name = subscription.name.clone();
            card_frame(self.theme).show(ui, |ui| {
                ui.label(
                    RichText::new(format!(
                        "subscription {} · enabled={} · slot={:?}",
                        subscription.name, subscription.enabled, subscription.slot_name
                    ))
                    .strong()
                    .monospace(),
                );
                ui.label(
                    RichText::new(format!(
                        "pubs={} · conninfo={}",
                        subscription.publications.join(","),
                        subscription.conninfo_redacted
                    ))
                    .small()
                    .color(self.theme.text_muted),
                );
                ui.horizontal(|ui| {
                    if ghost_button_with_icon(ui, Icon::FileCode2, "Preview DROP", self.theme).clicked() {
                        actions.push(ReplicationSurfaceAction::PreviewDropSubscription(name.clone()));
                    }
                    if danger_button(ui, "Drop…", self.theme).clicked() {
                        actions.push(ReplicationSurfaceAction::RequestDropSubscription(name.clone()));
                    }
                });
            });
            ui.add_space(SPACE_XS);
        }
    }

    fn draw_slots(&self, ui: &mut egui::Ui, slots: &[db_pro_core::domain::replication::ReplicationSlotInfo]) {
        for slot in slots.iter().take(40) {
            ui.label(
                RichText::new(format!(
                    "slot {} · type={:?} · active={} · restart={:?}",
                    slot.slot_name, slot.slot_type, slot.active, slot.restart_lsn
                ))
                .small()
                .monospace()
                .color(self.theme.text_secondary),
            );
        }
    }

    fn draw_create_form(&mut self, ui: &mut egui::Ui, actions: &mut Vec<ReplicationSurfaceAction>) {
        ui.add_space(SPACE_SM);
        ui.label(RichText::new("Create publication (FOR ALL TABLES)").small().strong());
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.state.replication_create_name).hint_text("publication name"));
            if ghost_button_with_icon(ui, Icon::FileCode2, "Preview CREATE", self.theme).clicked() {
                actions.push(ReplicationSurfaceAction::PreviewCreatePublication(
                    self.state.replication_create_name.clone(),
                ));
            }
            if secondary_button(ui, "Create (confirm)", self.theme).clicked() {
                actions.push(ReplicationSurfaceAction::CreatePublication);
            }
        });
    }

    fn draw_preview(&self, ui: &mut egui::Ui, actions: &mut Vec<ReplicationSurfaceAction>) {
        let Some(preview) = self.state.replication_ddl_preview.as_ref() else {
            return;
        };
        egui::Window::new("Replication DDL preview")
            .collapsible(false)
            .resizable(true)
            .default_width(480.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.label(RichText::new(preview).monospace());
                if secondary_button(ui, "Close", self.theme).clicked() {
                    actions.push(ReplicationSurfaceAction::ClosePreview);
                }
            });
    }

    fn draw_drop_confirmations(&self, ui: &mut egui::Ui, actions: &mut Vec<ReplicationSurfaceAction>) {
        if let Some(name) = self.state.replication_drop_publication.as_ref() {
            egui::Window::new("Drop publication?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(format!("Drop publication `{name}`?"));
                    ui.horizontal(|ui| {
                        if danger_button(ui, "Drop", self.theme).clicked() {
                            actions.push(ReplicationSurfaceAction::ConfirmDropPublication(name.clone()));
                        }
                        if secondary_button(ui, "Cancel", self.theme).clicked() {
                            actions.push(ReplicationSurfaceAction::CancelDropPublication);
                        }
                    });
                });
        }
        if let Some(name) = self.state.replication_drop_subscription.as_ref() {
            egui::Window::new("Drop subscription?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(format!(
                        "Drop subscription `{name}`? Conninfo is never shown or logged."
                    ));
                    ui.horizontal(|ui| {
                        if danger_button(ui, "Drop", self.theme).clicked() {
                            actions.push(ReplicationSurfaceAction::ConfirmDropSubscription(name.clone()));
                        }
                        if secondary_button(ui, "Cancel", self.theme).clicked() {
                            actions.push(ReplicationSurfaceAction::CancelDropSubscription);
                        }
                    });
                });
        }
    }
}
