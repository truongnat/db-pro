use super::*;

#[path = "pg_settings_surface_view.rs"]
mod pg_settings_surface_view;

impl DbProApp {
    pub(super) fn draw_pg_settings_activity(&mut self, ui: &mut egui::Ui) {
        let actions = pg_settings_surface_view::PgSettingsSurfaceContext {
            theme: self.theme,
            state: &mut self.management.pg_settings,
        }
        .draw(ui);
        self.apply_pg_settings_surface_actions(actions);
    }

    fn apply_pg_settings_surface_actions(&mut self, actions: Vec<pg_settings_surface_view::PgSettingsSurfaceAction>) {
        for action in actions {
            match action {
                pg_settings_surface_view::PgSettingsSurfaceAction::Refresh => self.request_pg_settings(),
                pg_settings_surface_view::PgSettingsSurfaceAction::BeginEdit { name, value } => {
                    self.management.pg_settings.pg_settings_edit_name = name;
                    self.management.pg_settings.pg_settings_edit_value = value;
                }
                pg_settings_surface_view::PgSettingsSurfaceAction::Reset(name) => {
                    self.reset_pg_setting_session(&name);
                }
                pg_settings_surface_view::PgSettingsSurfaceAction::PreviewAlterSystem { name, value } => {
                    self.management.pg_settings.pg_settings_preview =
                        db_pro_core::domain::pg_settings::preview_alter_system(&name, &value).ok();
                }
                pg_settings_surface_view::PgSettingsSurfaceAction::ApplySession { name, value } => {
                    self.set_pg_setting_session(&name, &value);
                    self.management.pg_settings.pg_settings_edit_name.clear();
                }
                pg_settings_surface_view::PgSettingsSurfaceAction::CancelEdit => {
                    self.management.pg_settings.pg_settings_edit_name.clear();
                }
                pg_settings_surface_view::PgSettingsSurfaceAction::ClosePreview => {
                    self.management.pg_settings.pg_settings_preview = None;
                }
            }
        }
    }

    fn request_pg_settings(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.management.pg_settings.pg_settings_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        let driver = self.active_driver().to_ascii_lowercase();
        if !(driver.contains("postgres")) {
            self.management.pg_settings.pg_settings_error = Some("pg_settings is PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.pg_settings.list_command(request_id, connection_id));
    }

    fn set_pg_setting_session(&mut self, name: &str, value: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.pg_settings.set_session_command(
            request_id,
            connection_id,
            name.to_owned(),
            value.to_owned(),
        ));
    }

    fn reset_pg_setting_session(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.pg_settings.reset_session_command(
            request_id,
            connection_id,
            name.to_owned(),
        ));
    }
}
