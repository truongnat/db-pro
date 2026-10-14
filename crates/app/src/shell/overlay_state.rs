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
    pub(super) connection_delete_confirmation_id: Option<String>,
    /// Saved-query delete confirmation lives inside the queries sidebar
    /// surface; it must NOT share `connection_delete_confirmation_id`, which
    /// renders the connection delete modal in `draw_overlays`.
    pub(super) saved_query_delete_id: Option<String>,
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
            connection_delete_confirmation_id: None,
            saved_query_delete_id: None,
            folder_delete_confirmation: None,
        }
    }
}

#[cfg(test)]
#[path = "tests/overlay_state_tests.rs"]
mod tests;

