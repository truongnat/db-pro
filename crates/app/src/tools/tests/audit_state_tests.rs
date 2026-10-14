//! Tests for `../audit_state.rs` (moved out of the source file).
    use super::AuditState;

    #[test]
    fn audit_filter_preserves_filter_scope() {
        let state = AuditState {
            audit_filter_text: "alter".to_owned(),
            audit_filter_database: "app".to_owned(),
            audit_filter_username: "alice".to_owned(),
            audit_filter_severity: "warning".to_owned(),
            ..AuditState::default()
        };

        let filter = state.audit_filter();
        assert_eq!(filter.text, "alter");
        assert_eq!(filter.database, "app");
        assert_eq!(filter.username, "alice");
        assert_eq!(filter.severity, "warning");
        assert!(filter.command_tag.is_empty());
    }

    #[test]
    fn export_requires_loaded_page_and_selection() {
        let mut state = AuditState::default();

        assert_eq!(
            state.build_export_preview(),
            Err("Load an audit page before exporting".to_owned())
        );
    }