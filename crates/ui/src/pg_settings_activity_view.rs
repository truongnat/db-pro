use super::command_dispatch::RuntimeCommandDispatcher;
use super::pg_settings_state::PgSettingsState;
use super::*;

#[path = "pg_settings_surface_view.rs"]
mod pg_settings_surface_view;

pub(super) struct PgSettingsActivityContext<'a, 'bridge> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut PgSettingsState,
    pub(super) connection_id: Option<&'a str>,
    pub(super) driver: &'a str,
    pub(super) command_dispatcher: &'a mut RuntimeCommandDispatcher<'bridge>,
    pub(super) feedback: &'a mut FeedbackState,
}

impl PgSettingsActivityContext<'_, '_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) {
        let actions = pg_settings_surface_view::PgSettingsSurfaceContext {
            theme: self.theme,
            state: self.state,
        }
        .draw(ui);
        self.apply_actions(actions);
    }

    fn apply_actions(&mut self, actions: Vec<pg_settings_surface_view::PgSettingsSurfaceAction>) {
        for action in actions {
            match action {
                pg_settings_surface_view::PgSettingsSurfaceAction::Refresh => self.request_pg_settings(),
                pg_settings_surface_view::PgSettingsSurfaceAction::BeginEdit { name, value } => {
                    self.state.pg_settings_edit_name = name;
                    self.state.pg_settings_edit_value = value;
                }
                pg_settings_surface_view::PgSettingsSurfaceAction::Reset(name) => {
                    self.reset_pg_setting_session(&name);
                }
                pg_settings_surface_view::PgSettingsSurfaceAction::PreviewAlterSystem { name, value } => {
                    self.state.pg_settings_preview =
                        db_pro_core::domain::pg_settings::preview_alter_system(&name, &value).ok();
                }
                pg_settings_surface_view::PgSettingsSurfaceAction::ApplySession { name, value } => {
                    if self.set_pg_setting_session(&name, &value) {
                        self.state.pg_settings_edit_name.clear();
                    }
                }
                pg_settings_surface_view::PgSettingsSurfaceAction::CancelEdit => {
                    self.state.pg_settings_edit_name.clear();
                }
                pg_settings_surface_view::PgSettingsSurfaceAction::ClosePreview => {
                    self.state.pg_settings_preview = None;
                }
            }
        }
    }

    fn request_pg_settings(&mut self) {
        let Some(connection_id) = self.connection_id else {
            self.state.pg_settings_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.driver.to_ascii_lowercase().contains("postgres") {
            self.state.pg_settings_error = Some("pg_settings is PostgreSQL-only".into());
            return;
        }
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(list_pg_settings_command(request_id, connection_id.to_owned()));
    }

    fn set_pg_setting_session(&mut self, name: &str, value: &str) -> bool {
        let Some(connection_id) = self.connection_id else {
            return false;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(set_pg_setting_session_command(
            request_id,
            connection_id.to_owned(),
            name.to_owned(),
            value.to_owned(),
        ))
    }

    fn reset_pg_setting_session(&mut self, name: &str) {
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(reset_pg_setting_session_command(
            request_id,
            connection_id.to_owned(),
            name.to_owned(),
        ));
    }

    fn dispatch(&mut self, command: UiCommand) -> bool {
        self.command_dispatcher.dispatch(command, self.feedback)
    }
}

pub(super) fn list_pg_settings_command(request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::ListPgSettings {
        request_id,
        connection_id,
    }
}

fn set_pg_setting_session_command(
    request_id: RequestId,
    connection_id: String,
    name: String,
    value: String,
) -> UiCommand {
    UiCommand::SetPgSettingSession {
        request_id,
        connection_id,
        name,
        value,
    }
}

fn reset_pg_setting_session_command(request_id: RequestId, connection_id: String, name: String) -> UiCommand {
    UiCommand::ResetPgSettingSession {
        request_id,
        connection_id,
        name,
    }
}

#[cfg(test)]
mod tests {
    use super::{pg_settings_surface_view::PgSettingsSurfaceAction, PgSettingsActivityContext, PgSettingsState};

    #[test]
    fn failed_session_setting_dispatch_preserves_edit_state() {
        let (mut bridge, command_rx, _event_tx) = crate::TaskBridge::with_channels();
        drop(command_rx);
        let mut dispatcher = crate::app::command_dispatch::RuntimeCommandDispatcher::new(&mut bridge);
        let mut state = PgSettingsState {
            pg_settings_edit_name: "work_mem".to_owned(),
            pg_settings_edit_value: "64MB".to_owned(),
            ..PgSettingsState::default()
        };
        let mut feedback = crate::app::FeedbackState::default();

        PgSettingsActivityContext {
            theme: crate::DbProTheme::default(),
            state: &mut state,
            connection_id: Some("conn-1"),
            driver: "PostgreSQL",
            command_dispatcher: &mut dispatcher,
            feedback: &mut feedback,
        }
        .apply_actions(vec![PgSettingsSurfaceAction::ApplySession {
            name: "work_mem".to_owned(),
            value: "64MB".to_owned(),
        }]);

        assert_eq!(state.pg_settings_edit_name, "work_mem");
    }
}
