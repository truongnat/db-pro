/// State for transient shell overlays that are not owned by a single data view.
#[derive(Debug, Default)]
pub(crate) struct OverlayState {
    pub(super) export_open: bool,
    pub(super) export_format: ExportFormat,
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

/// File format the result-grid export dialog writes. Owned by the overlay
/// state so the format list, labels and the writer dispatch in
/// `query_dialogs_view.rs` cannot drift out of sync — a variant either exists
/// here or it cannot be selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ExportFormat {
    #[default]
    Csv,
    Tsv,
    Json,
    Markdown,
    SqlInsert,
    SqlCopy,
    Excel,
}

impl ExportFormat {
    /// Pickable order in the dialog — every variant, once.
    pub(crate) const ALL: [Self; 7] = [
        Self::Csv,
        Self::Tsv,
        Self::Json,
        Self::Markdown,
        Self::SqlInsert,
        Self::SqlCopy,
        Self::Excel,
    ];

    /// Short button label. These are file-format names, not UI copy, so they
    /// stay literal across locales.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Csv => "CSV",
            Self::Tsv => "TSV",
            Self::Json => "JSON",
            Self::Markdown => "Markdown",
            Self::SqlInsert => "INSERT",
            Self::SqlCopy => "COPY",
            Self::Excel => "Excel",
        }
    }

    /// `Some(delimiter)` for text-delimited formats; `None` means the writer
    /// owns the layout entirely (JSON, Markdown, SQL statements, Excel).
    pub(crate) fn delimiter(self) -> Option<&'static str> {
        match self {
            Self::Csv => Some(","),
            Self::Tsv => Some("\t"),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "tests/overlay_state_tests.rs"]
mod tests;
