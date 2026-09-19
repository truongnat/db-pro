/// State for transient shell overlays that are not owned by a single data view.
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

#[cfg(test)]
mod tests {
    use super::OverlayState;

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
}
