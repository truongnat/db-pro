//! Monitoring session presentation and user intents.

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MonitoringSessionsAction {
    OpenSql(String),
    Cancel(i64),
    RequestTerminate(i64),
}

pub(super) struct MonitoringSessionsContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) snapshot: &'a db_pro_core::domain::monitoring::MonitoringSnapshot,
    pub(super) filter_active_only: &'a mut bool,
}

impl MonitoringSessionsContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<MonitoringSessionsAction> {
        let mut actions = Vec::new();
        self.draw_filter(ui);
        self.draw_idle_transactions(ui);
        let sessions = self.visible_sessions();
        if sessions.is_empty() {
            self.draw_empty(ui);
        } else {
            for session in sessions {
                actions.extend(self.draw_session(ui, session));
                ui.add_space(SPACE_SM);
            }
        }
        actions
    }

    fn draw_filter(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "SESSIONS", self.theme);
        ui.add_space(SPACE_SM);
        ui.checkbox(self.filter_active_only, "Active queries only");
        ui.add_space(SPACE_SM);
    }

    fn draw_idle_transactions(&self, ui: &mut egui::Ui) {
        let idle_xacts = self.snapshot.idle_in_transaction_sessions();
        if idle_xacts.is_empty() {
            return;
        }
        section_label(ui, "IDLE IN TRANSACTION", self.theme);
        ui.add_space(SPACE_SM);
        for session in idle_xacts.into_iter().take(20) {
            ui.label(
                RichText::new(format!(
                    "pid {} · xact_age={:?} ms · backend_age={:?} ms · {}",
                    session.backend_id,
                    session.xact_age_ms,
                    session.backend_age_ms,
                    session.username.as_deref().unwrap_or("?")
                ))
                .small()
                .color(self.theme.warning),
            );
        }
        ui.add_space(SPACE_MD);
    }

    fn visible_sessions(&self) -> Vec<&db_pro_core::domain::monitoring::MonitorSession> {
        if *self.filter_active_only {
            self.snapshot.active_queries()
        } else {
            self.snapshot.sessions.iter().collect()
        }
    }

    fn draw_empty(&self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(if self.snapshot.sessions.is_empty() {
                self.snapshot.message.as_str()
            } else {
                "No active queries right now."
            })
            .small()
            .color(self.theme.text_muted),
        );
    }

    fn draw_session(
        &self,
        ui: &mut egui::Ui,
        session: &db_pro_core::domain::monitoring::MonitorSession,
    ) -> Vec<MonitoringSessionsAction> {
        let mut actions = Vec::new();
        card_frame(self.theme).show(ui, |ui| {
            self.draw_session_header(ui, session);
            ui.label(
                RichText::new(format!(
                    "{} · {} · {}",
                    session.username.as_deref().unwrap_or("?"),
                    session.database.as_deref().unwrap_or("?"),
                    session.application_name.as_deref().unwrap_or("-")
                ))
                .small()
                .color(self.theme.text_secondary),
            );
            if let Some(query) = &session.query_text {
                self.draw_query(ui, session, query, &mut actions);
            }
        });
        actions
    }

    fn draw_session_header(&self, ui: &mut egui::Ui, session: &db_pro_core::domain::monitoring::MonitorSession) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("pid {}", session.backend_id))
                    .strong()
                    .monospace()
                    .color(self.theme.text_primary),
            );
            if session.is_current {
                badge(ui, "current", self.theme.surface_active, self.theme.text_secondary);
            }
            ui.label(
                RichText::new(session.state.clone().unwrap_or_else(|| "—".into()))
                    .small()
                    .color(self.theme.text_secondary),
            );
            if let Some(ms) = session.query_duration_ms {
                ui.label(
                    RichText::new(format!("query {ms} ms"))
                        .small()
                        .color(self.theme.text_muted),
                );
            }
            if let Some(ms) = session.xact_age_ms {
                ui.label(
                    RichText::new(format!("xact {ms} ms"))
                        .small()
                        .color(self.theme.text_muted),
                );
            }
            if session.idle_in_transaction {
                badge(ui, "idle-in-xact", self.theme.warning, self.theme.text_primary);
            }
        });
    }

    fn draw_query(
        &self,
        ui: &mut egui::Ui,
        session: &db_pro_core::domain::monitoring::MonitorSession,
        query: &str,
        actions: &mut Vec<MonitoringSessionsAction>,
    ) {
        let short = if query.len() > 120 {
            format!("{}…", &query.chars().take(119).collect::<String>())
        } else {
            query.to_owned()
        };
        ui.label(RichText::new(short).monospace().small().color(self.theme.text_primary));
        ui.horizontal(|ui| {
            if ghost_button_with_icon(ui, Icon::FileCode2, "Open SQL", self.theme).clicked() {
                actions.push(MonitoringSessionsAction::OpenSql(query.to_owned()));
            }
            if !session.is_current && secondary_button_with_icon(ui, Icon::Ban, "Cancel", self.theme).clicked() {
                actions.push(MonitoringSessionsAction::Cancel(session.backend_id));
            }
            if !session.is_current && danger_button(ui, "Terminate", self.theme).clicked() {
                actions.push(MonitoringSessionsAction::RequestTerminate(session.backend_id));
            }
        });
    }
}
