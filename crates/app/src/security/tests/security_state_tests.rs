//! Tests for `../security_state.rs` (moved out of the source file).
    use super::SecurityState;

    #[test]
    fn defaults_keep_security_scope_explicit() {
        let security = SecurityState::default();

        assert!(security.security_users.is_empty());
        assert_eq!(security.security_rls_schema, "public");
        assert!(!security.security_rls_confirm_apply);
    }

    #[test]
    fn rls_inspection_requires_schema_and_table() {
        let state = SecurityState::default();

        assert_eq!(
            state.table_rls_target(),
            Err("Schema and table are required for RLS inspect".to_owned())
        );
    }

    #[test]
    fn empty_rls_preview_cannot_dispatch_ddl() {
        let state = SecurityState::default();

        assert!(state.rls_preview_sql().is_none());

        let state = SecurityState {
            security_rls_preview_sql: " ALTER TABLE users ENABLE ROW LEVEL SECURITY ".to_owned(),
            ..SecurityState::default()
        };
        assert_eq!(state.rls_preview_sql().as_deref(), Some("ALTER TABLE users ENABLE ROW LEVEL SECURITY"));
    }