//! Monitoring surface composition and intent collection.
use super::*;

#[derive(Debug)]
pub(super) enum MonitoringSurfaceAction {
    Refresh,
    Sessions(monitoring_sessions_view::MonitoringSessionsAction),
    Workload(monitoring_workload_view::MonitoringWorkloadAction),
}

pub(super) struct MonitoringSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) connected: bool,
    pub(super) driver: &'a str,
    pub(super) connection_name: &'a str,
    pub(super) state: &'a mut MonitoringState,
}

impl MonitoringSurfaceContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<MonitoringSurfaceAction> {
        let mut actions = self.draw_header(ui);
        actions.extend(self.draw_snapshot(ui));
        actions
    }

    fn draw_header(&mut self, ui: &mut egui::Ui) -> Vec<MonitoringSurfaceAction> {
        let mut actions = Vec::new();
        let header_actions = monitoring_header_view::MonitoringHeaderContext {
            theme: self.theme,
            connected: self.connected,
            driver: self.driver,
            connection_name: self.connection_name,
            poll: &mut self.state.monitoring_poll,
        }
        .draw(ui);
        if header_actions
            .into_iter()
            .any(|action| matches!(action, monitoring_header_view::MonitoringHeaderAction::Refresh))
        {
            actions.push(MonitoringSurfaceAction::Refresh);
        }

        ui.add_space(SPACE_MD);
        if let Some(error) = &self.state.monitoring_error {
            ui.colored_label(self.theme.warning, error);
            ui.add_space(SPACE_SM);
        }

        actions
    }

    fn draw_snapshot(&mut self, ui: &mut egui::Ui) -> Vec<MonitoringSurfaceAction> {
        let mut actions = Vec::new();

        let Some(snapshot) = self.state.monitoring_snapshot.as_ref() else {
            if self.connected {
                ui.label(
                    RichText::new("Refresh to load sessions (or wait for auto-refresh).")
                        .small()
                        .color(self.theme.text_muted),
                );
            }
            return actions;
        };

        monitoring_snapshot_view::draw_health_and_local(ui, self.theme, snapshot);
        let session_actions = monitoring_sessions_view::MonitoringSessionsContext {
            theme: self.theme,
            snapshot,
            filter_active_only: &mut self.state.monitoring_filter_active_only,
        }
        .draw(ui);
        actions.extend(session_actions.into_iter().map(MonitoringSurfaceAction::Sessions));
        monitoring_snapshot_view::draw_server_stats(ui, self.theme, snapshot);
        let workload_actions = monitoring_workload_view::MonitoringWorkloadContext {
            theme: self.theme,
            workload: snapshot.workload.as_ref(),
            previous: self.state.monitoring_workload_prev.as_ref(),
            sort: self.state.monitoring_stat_sort,
            filter: &mut self.state.monitoring_workload_filter,
        }
        .draw(ui);
        actions.extend(workload_actions.into_iter().map(MonitoringSurfaceAction::Workload));
        actions
    }
}
