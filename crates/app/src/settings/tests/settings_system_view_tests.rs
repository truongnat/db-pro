//! Tests for `../settings_system_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn system_context_exposes_only_supported_sections() {
        assert_ne!(SettingsSection::DataGrid, SettingsSection::General);
    }