//! Deterministic visual fixtures; no database commands are dispatched here.
use super::*;

impl DbProApp {
    pub fn open_workstation_cleanup_for_capture(&mut self, surface: &str, light: bool) -> bool {
        if !matches!(
            surface,
            "welcome-agent"
                | "query-agent"
                | "query-agent-running"
                | "settings-general"
                | "settings-appearance"
                | "settings-advanced"
        ) {
            return false;
        }
        self.open_welcome_workspace_for_capture(light);
        match surface {
            "welcome-agent" => {}
            "query-agent" | "query-agent-running" => self.open_query_workspace_for_capture(),
            "settings-general" | "settings-appearance" | "settings-advanced" => {
                self.open_settings_workspace_for_capture();
                self.preferences.section = match surface {
                    "settings-appearance" => settings_model::SettingsSection::Appearance,
                    "settings-advanced" => settings_model::SettingsSection::Advanced,
                    _ => settings_model::SettingsSection::General,
                };
            }
            _ => return false,
        }
        self.preferences.dark_mode = !light;
        self.preferences.settings.appearance.dark_mode = !light;
        self.theme = if light { DbProTheme::light() } else { DbProTheme::dark() };
        self.workspace.agent_open = true;
        self.workspace.sidebar_open = !surface.starts_with("settings-");
        self.agent.provider_label = "Offline draft".to_owned();
        self.agent.provider_detail = "Visual fixture · no provider request is sent".to_owned();
        self.agent.auto_run_read_only = true;
        if let Some(mut connection) = self.connection.catalog.get(0).cloned() {
            connection.name = "Analytics warehouse · regional customer reporting and operations".to_owned();
            let id = connection.id.clone();
            self.connection.catalog.replace(vec![connection]);
            self.connection.lifecycle.set_active_connection_id(Some(id.clone()));
            if surface.starts_with("query-") {
                self.connection.lifecycle.set_connected(true);
                if let Some(document) = self
                    .query
                    .session
                    .documents
                    .get_mut(self.query.session.active_document_index)
                {
                    document.connection_id = Some(id);
                    document.schema = Some("customer_analytics_reporting_and_operations".to_owned());
                }
            }
        }
        self.schema.explorer.selected_schema = Some("customer_analytics_reporting_and_operations".to_owned());
        self.schema.explorer.selected_table = Some("monthly_customer_retention_and_revenue_summary".to_owned());
        if surface == "query-agent-running" {
            if let Some(document) = self
                .query
                .session
                .documents
                .get_mut(self.query.session.active_document_index)
            {
                document.execution_state = QueryExecutionState::Running(RequestId(9_000_001));
                document.execution_started_at = Some(std::time::Instant::now());
            }
        }
        true
    }
}
