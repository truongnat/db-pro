//! Tests for `../query_save_dialog_surface_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn close_dialog_actions_are_distinct() {
        assert_ne!(DirtyCloseDialogAction::Save, DirtyCloseDialogAction::Discard);
        assert_ne!(SaveAsDialogAction::Save, SaveAsDialogAction::Cancel);
    }