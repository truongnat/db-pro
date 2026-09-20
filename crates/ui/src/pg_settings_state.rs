//! State owned by the PostgreSQL settings surface.

use super::{RequestId, UiCommand};

#[derive(Default)]
pub(super) struct PgSettingsState {
    pub(super) pg_settings: Option<db_pro_core::domain::pg_settings::PgSettingsSnapshot>,
    pub(super) pg_settings_filter: String,
    pub(super) pg_settings_edit_name: String,
    pub(super) pg_settings_edit_value: String,
    pub(super) pg_settings_preview: Option<db_pro_core::domain::pg_settings::PgSettingPreviewSql>,
    pub(super) pg_settings_error: Option<String>,
}

impl PgSettingsState {
    pub(super) fn list_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::ListPgSettings {
            request_id,
            connection_id,
        }
    }

    pub(super) fn set_session_command(
        &self,
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

    pub(super) fn reset_session_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        name: String,
    ) -> UiCommand {
        UiCommand::ResetPgSettingSession {
            request_id,
            connection_id,
            name,
        }
    }
}
