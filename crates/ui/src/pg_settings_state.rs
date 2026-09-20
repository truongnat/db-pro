//! State owned by the PostgreSQL settings surface.

#[derive(Default)]
pub(super) struct PgSettingsState {
    pub(super) pg_settings: Option<db_pro_core::domain::pg_settings::PgSettingsSnapshot>,
    pub(super) pg_settings_filter: String,
    pub(super) pg_settings_edit_name: String,
    pub(super) pg_settings_edit_value: String,
    pub(super) pg_settings_preview: Option<db_pro_core::domain::pg_settings::PgSettingPreviewSql>,
    pub(super) pg_settings_error: Option<String>,
}
