//! Tests for `../pg_settings_activity_view.rs` (moved out of the source file).
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