//! Tests for `../query_dialog_surface_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn dialog_actions_keep_confirmation_boundaries_explicit() {
        assert_ne!(DestructiveDialogAction::Confirm, DestructiveDialogAction::Cancel);
        assert_ne!(ExportDialogAction::Export, ExportDialogAction::Overwrite);
    }