//! Tests for `../transfer_activity_surface_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn action_enum_keeps_transfer_commands_explicit() {
        assert_eq!(
            TransferActivityAction::RunSyntheticTransfer { cancel_midway: true },
            TransferActivityAction::RunSyntheticTransfer { cancel_midway: true }
        );
    }