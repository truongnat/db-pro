//! Tests for `../agent_patch.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn applies_patch_and_marks_document_dirty() {
        let mut document = crate::query::QueryDocument::new("query-1", "Query", "select 1");
        let patch = db_pro_core::domain::agent::AgentSqlPatch {
            document_id: document.id.clone(),
            expected_version: document.buffer.version(),
            range: (7, 8),
            replacement: "2".to_owned(),
        };

        let result = apply_to_document(&mut document, &patch);

        assert!(matches!(
            result,
            Ok(db_pro_core::domain::agent::AgentToolOutput::PatchApplied { .. })
        ));
        assert_eq!(document.text(), "select 2");
        assert!(document.dirty);
    }

    #[test]
    fn rejects_stale_document_before_mutating_it() {
        let mut document = crate::query::QueryDocument::new("query-1", "Query", "select 1");
        let patch = db_pro_core::domain::agent::AgentSqlPatch {
            document_id: document.id.clone(),
            expected_version: document.buffer.version() + 1,
            range: (7, 8),
            replacement: "2".to_owned(),
        };

        assert_eq!(
            apply_to_document(&mut document, &patch),
            Err(AgentPatchApplyError::DocumentChanged)
        );
        assert_eq!(document.text(), "select 1");
    }