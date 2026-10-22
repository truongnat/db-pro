//! Monitoring destructive-action confirmations and user intents.

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MonitoringConfirmationAction {
    /// `backend_start` travels with the pid so the runtime can refuse to signal
    /// a session whose pid was recycled after the snapshot (#501).
    ConfirmTerminate {
        backend_id: i64,
        backend_start: Option<String>,
    },
    CancelTerminate,
    ConfirmCancelBackend {
        backend_id: i64,
        backend_start: Option<String>,
    },
    DismissCancelBackend,
    ConfirmMaintenance(db_pro_core::domain::monitoring::MaintenanceAction),
    CancelMaintenance,
    ConfirmResetStatistics,
    CancelResetStatistics,
}

pub(super) struct MonitoringConfirmationContext<'a> {
    pub(super) theme: DbProTheme,
    /// Full session rows so the dialogs can show user/database/query context —
    /// a bare pid invites terminating the wrong backend (#386).
    pub(super) terminate_session: Option<&'a db_pro_core::domain::monitoring::MonitorSession>,
    pub(super) cancel_session: Option<&'a db_pro_core::domain::monitoring::MonitorSession>,
    pub(super) maintenance_action: Option<db_pro_core::domain::monitoring::MaintenanceAction>,
    /// `schema.table`, or `entire database` when no target is set.
    pub(super) maintenance_target: String,
    pub(super) reset_statistics: bool,
}

impl MonitoringConfirmationContext<'_> {
    pub(super) fn draw(&self, ctx: &egui::Context) -> Vec<MonitoringConfirmationAction> {
        let mut actions = Vec::new();
        if let Some(session) = self.terminate_session {
            actions.extend(self.draw_terminate(ctx, session));
        }
        if let Some(session) = self.cancel_session {
            actions.extend(self.draw_cancel_backend(ctx, session));
        }
        if let Some(action) = self.maintenance_action {
            actions.extend(self.draw_maintenance(ctx, action, &self.maintenance_target));
        }
        if self.reset_statistics {
            actions.extend(self.draw_reset_statistics(ctx));
        }
        actions
    }

    fn draw_terminate(
        &self,
        ctx: &egui::Context,
        session: &db_pro_core::domain::monitoring::MonitorSession,
    ) -> Vec<MonitoringConfirmationAction> {
        let backend_id = session.backend_id;
        let mut actions = Vec::new();
        let mut open = true;
        Dialog::new(&mut open, "Terminate session?", self.theme)
            .width(460.0)
            .id_salt("monitoring_terminate_dialog")
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    ui.label(format!(
                        "Terminate PostgreSQL backend pid {backend_id}? This disconnects the client."
                    ));
                    self.draw_session_context(ui, session);
                });
                frame.footer(|ui| {
                    if Button::new(self.theme)
                        .text("Terminate")
                        .variant(ButtonVariant::Destructive)
                        .show(ui)
                        .clicked()
                    {
                        actions.push(MonitoringConfirmationAction::ConfirmTerminate {
                            backend_id,
                            backend_start: session.backend_start.clone(),
                        });
                    }
                    if Button::new(self.theme)
                        .icon(Icon::X)
                        .text("Cancel")
                        .variant(ButtonVariant::Secondary)
                        .show(ui)
                        .clicked()
                    {
                        actions.push(MonitoringConfirmationAction::CancelTerminate);
                    }
                });
            });
        if !open {
            actions.push(MonitoringConfirmationAction::CancelTerminate);
        }
        actions
    }

    fn draw_cancel_backend(
        &self,
        ctx: &egui::Context,
        session: &db_pro_core::domain::monitoring::MonitorSession,
    ) -> Vec<MonitoringConfirmationAction> {
        let backend_id = session.backend_id;
        let mut actions = Vec::new();
        let mut open = true;
        Dialog::new(&mut open, "Cancel backend?", self.theme)
            .width(460.0)
            .id_salt("monitoring_cancel_backend_dialog")
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    ui.label(format!("Cancel backend pid {backend_id}?"));
                    self.draw_session_context(ui, session);
                });
                frame.footer(|ui| {
                    if Button::new(self.theme)
                        .text("Cancel backend")
                        .variant(ButtonVariant::Destructive)
                        .show(ui)
                        .clicked()
                    {
                        actions.push(MonitoringConfirmationAction::ConfirmCancelBackend {
                            backend_id,
                            backend_start: session.backend_start.clone(),
                        });
                    }
                    if Button::new(self.theme)
                        .icon(Icon::X)
                        .text("Keep running")
                        .variant(ButtonVariant::Secondary)
                        .show(ui)
                        .clicked()
                    {
                        actions.push(MonitoringConfirmationAction::DismissCancelBackend);
                    }
                });
            });
        if !open {
            actions.push(MonitoringConfirmationAction::DismissCancelBackend);
        }
        actions
    }

    /// Who the backend is and what it is running — the difference between
    /// killing a stuck report and a lock-holding transaction (#386).
    fn draw_session_context(&self, ui: &mut egui::Ui, session: &db_pro_core::domain::monitoring::MonitorSession) {
        ui.add_space(SPACE_XS);
        ui.label(
            RichText::new(format!(
                "{}@{} · {}",
                session.username.as_deref().unwrap_or("?"),
                session.database.as_deref().unwrap_or("?"),
                session.state.as_deref().unwrap_or("—"),
            ))
            .small()
            .color(self.theme.text_secondary),
        );
        if let Some(ms) = session.query_duration_ms {
            ui.label(
                RichText::new(format!("query running for {} ms", ms))
                    .small()
                    .color(self.theme.text_muted),
            );
        }
        if let Some(query) = session.query_text.as_deref().filter(|query| !query.trim().is_empty()) {
            let short: String = query.chars().take(120).collect();
            let ellipsis = if query.chars().count() > 120 { "…" } else { "" };
            ui.label(
                RichText::new(format!("{short}{ellipsis}"))
                    .monospace()
                    .small()
                    .color(self.theme.text_primary),
            );
        }
        if session.idle_in_transaction || session.state.as_deref() == Some("idle in transaction") {
            ui.colored_label(
                self.theme.warning,
                "This backend holds an open transaction — terminating it rolls back its work and may release locks other sessions wait on.",
            );
        }
    }

    fn draw_maintenance(
        &self,
        ctx: &egui::Context,
        action: db_pro_core::domain::monitoring::MaintenanceAction,
        target: &str,
    ) -> Vec<MonitoringConfirmationAction> {
        let mut actions = Vec::new();
        let mut open = true;
        Dialog::new(&mut open, "Run maintenance?", self.theme)
            .width(460.0)
            .id_salt("monitoring_maintenance_dialog")
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    ui.label(format!(
                        "Run {} on {target}? Long-running VACUUM can take locks.",
                        action.as_label()
                    ));
                });
                frame.footer(|ui| {
                    if Button::new(self.theme)
                        .text(action.as_label())
                        .variant(ButtonVariant::Destructive)
                        .show(ui)
                        .clicked()
                    {
                        actions.push(MonitoringConfirmationAction::ConfirmMaintenance(action));
                    }
                    if Button::new(self.theme)
                        .icon(Icon::X)
                        .text("Cancel")
                        .variant(ButtonVariant::Secondary)
                        .show(ui)
                        .clicked()
                    {
                        actions.push(MonitoringConfirmationAction::CancelMaintenance);
                    }
                });
            });
        if !open {
            actions.push(MonitoringConfirmationAction::CancelMaintenance);
        }
        actions
    }

    fn draw_reset_statistics(&self, ctx: &egui::Context) -> Vec<MonitoringConfirmationAction> {
        let mut actions = Vec::new();
        let mut open = true;
        Dialog::new(&mut open, "Reset pg_stat_statements?", self.theme)
            .width(460.0)
            .id_salt("monitoring_reset_statistics_dialog")
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    ui.label(
                        "This clears all accumulated statement statistics on the server. \
                         It is an administrative action and cannot be undone.",
                    );
                });
                frame.footer(|ui| {
                    if Button::new(self.theme)
                        .text("Reset statistics")
                        .variant(ButtonVariant::Destructive)
                        .show(ui)
                        .clicked()
                    {
                        actions.push(MonitoringConfirmationAction::ConfirmResetStatistics);
                    }
                    if Button::new(self.theme)
                        .icon(Icon::X)
                        .text("Cancel")
                        .variant(ButtonVariant::Secondary)
                        .show(ui)
                        .clicked()
                    {
                        actions.push(MonitoringConfirmationAction::CancelResetStatistics);
                    }
                });
            });
        if !open {
            actions.push(MonitoringConfirmationAction::CancelResetStatistics);
        }
        actions
    }
}
