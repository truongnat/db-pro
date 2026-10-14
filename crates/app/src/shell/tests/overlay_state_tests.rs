//! Tests for `../overlay_state.rs` (moved out of the source file).
    use super::OverlayState;

    #[test]
    fn default_overlay_state_is_closed_and_non_destructive() {
        let state = OverlayState::default();

        assert!(!state.export_open);
        assert_eq!(state.export_format, "CSV");
        assert!(!state.export_overwrite_pending);
        assert!(!state.restore_confirmation);
        assert!(state.connection_delete_confirmation_id.is_none());
        assert!(state.folder_delete_confirmation.is_none());
    }
