//! Tests for `../overlay_state.rs` (moved out of the source file).
use crate::app::overlay_state::ExportFormat;
use super::OverlayState;

#[test]
fn default_overlay_state_is_closed_and_non_destructive() {
    let state = OverlayState::default();

    assert!(!state.export_open);
    assert_eq!(state.export_format, ExportFormat::Csv);
    assert!(!state.export_overwrite_pending);
    assert!(!state.restore_confirmation);
    assert!(state.connection_delete_confirmation_id.is_none());
    assert!(state.folder_delete_confirmation.is_none());
}

#[test]
fn copy_format_is_only_offered_for_postgres() {
    let offered: Vec<_> = ExportFormat::offered(true).collect();
    assert!(offered.contains(&ExportFormat::SqlCopy));

    let offered: Vec<_> = ExportFormat::offered(false).collect();
    assert!(!offered.contains(&ExportFormat::SqlCopy));
    assert_eq!(offered.len(), ExportFormat::ALL.len() - 1);
}

// #396: a written export must always end in the format's extension — missing or mismatched
// extensions are appended onto the typed name rather than replacing it.
#[test]
fn effective_export_path_appends_format_extension() {
    let state = OverlayState {
        export_path: "/tmp/results".to_owned(),
        export_format: ExportFormat::SqlInsert,
        ..Default::default()
    };
    assert_eq!(state.effective_export_path(), std::path::Path::new("/tmp/results.sql"));

    let mismatched = OverlayState {
        export_path: "/tmp/results.csv".to_owned(),
        export_format: ExportFormat::SqlInsert,
        ..Default::default()
    };
    assert_eq!(
        mismatched.effective_export_path(),
        std::path::Path::new("/tmp/results.csv.sql")
    );

    let correct = OverlayState {
        export_path: "/tmp/results.sql".to_owned(),
        export_format: ExportFormat::SqlCopy,
        ..Default::default()
    };
    assert_eq!(correct.effective_export_path(), std::path::Path::new("/tmp/results.sql"));

    let empty = OverlayState { export_path: "   ".to_owned(), ..Default::default() };
    assert_eq!(empty.effective_export_path(), std::path::PathBuf::new());
}
