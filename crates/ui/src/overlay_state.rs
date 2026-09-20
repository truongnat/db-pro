/// State for transient shell overlays that are not owned by a single data view.
use super::{RequestId, UiCommand};

#[derive(Debug)]
pub(crate) struct OverlayState {
    pub(super) export_open: bool,
    pub(super) export_format: String,
    pub(super) export_path: String,
    pub(super) export_overwrite_pending: bool,
    pub(super) backup_output_path: String,
    pub(super) restore_input_path: String,
    pub(super) restore_confirmation: bool,
    pub(super) delete_confirmation_id: Option<String>,
    pub(super) folder_delete_confirmation: Option<String>,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            export_open: false,
            export_format: "CSV".to_owned(),
            export_path: String::new(),
            export_overwrite_pending: false,
            backup_output_path: String::new(),
            restore_input_path: String::new(),
            restore_confirmation: false,
            delete_confirmation_id: None,
            folder_delete_confirmation: None,
        }
    }
}

impl OverlayState {
    pub(super) fn pick_backup_command(&self, request_id: RequestId) -> UiCommand {
        UiCommand::PickBackupFile { request_id }
    }

    pub(super) fn backup_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::Backup {
            request_id,
            connection_id,
            output_path: self.backup_output_path.clone(),
            custom_format: false,
        }
    }

    pub(super) fn pick_restore_command(&self, request_id: RequestId) -> UiCommand {
        UiCommand::PickRestoreFile { request_id }
    }

    pub(super) fn restore_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::Restore {
            request_id,
            connection_id,
            input_path: self.restore_input_path.clone(),
            custom_format: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OverlayState;
    use crate::{RequestId, UiCommand};

    #[test]
    fn default_overlay_state_is_closed_and_non_destructive() {
        let state = OverlayState::default();

        assert!(!state.export_open);
        assert_eq!(state.export_format, "CSV");
        assert!(!state.export_overwrite_pending);
        assert!(!state.restore_confirmation);
        assert!(state.delete_confirmation_id.is_none());
        assert!(state.folder_delete_confirmation.is_none());
    }

    #[test]
    fn backup_and_restore_effects_read_overlay_paths() {
        let state = OverlayState {
            backup_output_path: "/tmp/backup.sql".to_owned(),
            restore_input_path: "/tmp/input.sql".to_owned(),
            ..OverlayState::default()
        };

        assert!(matches!(
            state.backup_command(RequestId(1), "source".to_owned()),
            UiCommand::Backup { output_path, connection_id, custom_format: false, .. }
                if output_path == "/tmp/backup.sql" && connection_id == "source"
        ));
        assert!(matches!(
            state.restore_command(RequestId(2), "source".to_owned()),
            UiCommand::Restore { input_path, connection_id, custom_format: false, .. }
                if input_path == "/tmp/input.sql" && connection_id == "source"
        ));
    }
}
