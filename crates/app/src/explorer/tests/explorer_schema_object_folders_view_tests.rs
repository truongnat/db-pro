//! Tests for `../explorer_schema_object_folders_view.rs` (moved out of the source file).
    use super::trigger_toggle_menu_label;

    #[test]
    fn trigger_toggle_menu_offers_only_supported_inverse_action() {
        assert_eq!(trigger_toggle_menu_label(true, true), Some("Disable Trigger"));
        assert_eq!(trigger_toggle_menu_label(false, true), Some("Enable Trigger"));
        assert_eq!(trigger_toggle_menu_label(true, false), None);
    }